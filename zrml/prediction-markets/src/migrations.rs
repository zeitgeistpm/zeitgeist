// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

use crate::{
    CacheSize, Config, Disputes, MarketIdOf, MarketIdsPerDisputeBlock, MarketIdsPerOpenBlock,
    MarketIdsPerOpenTimeFrame, MarketIdsPerReportBlock, Pallet,
};
use frame_support::{
    dispatch::Weight,
    log,
    pallet_prelude::PhantomData,
    storage::PrefixIterator,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    BoundedVec, Twox64Concat,
};
use sp_runtime::{traits::Saturating, SaturatedConversion};
extern crate alloc;
use alloc::vec::Vec;
use zeitgeist_primitives::{
    constants::BLOCKS_PER_DAY,
    types::{MarketPeriod, MarketStatus, PoolStatus},
};
use zrml_market_commons::MarketCommonsPalletApi;

const SWAPS_REQUIRED_STORAGE_VERSION: u16 = 2;
const SWAPS_NEXT_STORAGE_VERSION: u16 = 3;

const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 2;

const PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION_FOR_MIGRATE_MARKET_POOLS: u16 = 2;
const PREDICTION_MARKETS_NEXT_STORAGE_VERSION_FOR_MIGRATE_MARKET_POOLS: u16 = 3;

const PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION_FOR_CLEANUP_STORAGE_FOR_RESOLVED_MARKETS: u16 = 3;
const PREDICTION_MARKETS_NEXT_STORAGE_VERSION_FOR_CLEANUP_STORAGE_FOR_RESOLVED_MARKETS: u16 = 4;

const PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION_FOR_MIGRATE_MARKET_IDS_STORAGE: u16 = 4;
const PREDICTION_MARKETS_NEXT_STORAGE_VERSION_FOR_MIGRATE_MARKET_IDS_STORAGE: u16 = 5;

pub struct MigrateMarketPoolsBeforeOpen<T>(PhantomData<T>);

pub struct CleanUpStorageForResolvedOrClosedMarkets<T>(PhantomData<T>);

pub struct MigrateMarketIdsPerBlockStorage<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateMarketPoolsBeforeOpen<T> {
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);
        if utility::get_on_chain_storage_version_of_swaps_pallet() != SWAPS_REQUIRED_STORAGE_VERSION
        {
            log::info!("Skipping storage migration of market pools; swaps already up to date");
            return total_weight;
        }
        if StorageVersion::get::<Pallet<T>>()
            != PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION_FOR_MIGRATE_MARKET_POOLS
        {
            log::info!(
                "Skipping storage migration of market pools; prediction-markets already up to date"
            );
            return total_weight;
        }
        log::info!("Starting storage migration of market pools");

        let current_block = <frame_system::Pallet<T>>::block_number();
        let current_time_frame =
            Pallet::<T>::calculate_time_frame_of_moment(T::MarketCommons::now());
        total_weight = total_weight.saturating_add(T::DbWeight::get().reads(2));

        for (market_id, market) in T::MarketCommons::market_iter()
            .filter(|(_, market)| market.status == MarketStatus::Active)
        {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

            // No need to migrate if there's no pool.
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
            let pool_id = match T::MarketCommons::market_pool(&market_id) {
                Ok(pool_id) => pool_id,
                Err(_) => continue,
            };

            // Don't continue unless the market is not yet open.
            if match market.period {
                MarketPeriod::Block(ref range) => current_block >= range.start,
                MarketPeriod::Timestamp(ref range) => {
                    current_time_frame >= Pallet::<T>::calculate_time_frame_of_moment(range.start)
                }
            } {
                continue;
            }

            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
            let mut pool = match utility::get_pool::<T>(pool_id) {
                Some(pool) => pool,
                None => {
                    log::warn!("no pool found. market_id: {:?}. pool_id: {:?}", market_id, pool_id,);
                    continue;
                }
            };
            if pool.pool_status == PoolStatus::Active {
                pool.pool_status = PoolStatus::Initialized;
                utility::set_pool::<T>(pool_id, pool);
                total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

                // We also need to cache the market for auto-open.
                match market.period {
                    MarketPeriod::Block(ref range) => {
                        let _ = MarketIdsPerOpenBlock::<T>::try_mutate(&range.start, |ids| {
                            ids.try_push(market_id)
                        });
                    }
                    MarketPeriod::Timestamp(ref range) => {
                        let open_time_frame =
                            Pallet::<T>::calculate_time_frame_of_moment(range.start);
                        let _ =
                            MarketIdsPerOpenTimeFrame::<T>::try_mutate(&open_time_frame, |ids| {
                                ids.try_push(market_id)
                            });
                    }
                }
                total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
            } else {
                log::warn!(
                    "found pool with unexpected status. market_id: {:?}. pool_id: {:?}",
                    market_id,
                    pool_id,
                );
            }
        }

        StorageVersion::new(PREDICTION_MARKETS_NEXT_STORAGE_VERSION_FOR_MIGRATE_MARKET_POOLS)
            .put::<Pallet<T>>();
        utility::put_storage_version_of_swaps_pallet(SWAPS_NEXT_STORAGE_VERSION);
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(2));

        log::info!("Completed storage migration of market pools");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        let current_time_frame =
            Pallet::<T>::calculate_time_frame_of_moment(T::MarketCommons::now());
        let current_block = <frame_system::Pallet<T>>::block_number();

        for (market_id, market) in T::MarketCommons::market_iter() {
            let pool_id = match T::MarketCommons::market_pool(&market_id) {
                Ok(pool_id) => pool_id,
                Err(_) => continue,
            };
            let pool = match utility::get_pool::<T>(pool_id) {
                Some(pool) => pool,
                None => {
                    log::warn!("no pool found. market_id: {:?}. pool_id: {:?}", market_id, pool_id,);
                    continue;
                }
            };

            let not_yet_open = match market.period {
                MarketPeriod::Block(ref range) => current_block < range.start,
                MarketPeriod::Timestamp(ref range) => {
                    current_time_frame < Pallet::<T>::calculate_time_frame_of_moment(range.start)
                }
            };
            if not_yet_open {
                assert_eq!(
                    pool.pool_status,
                    PoolStatus::Initialized,
                    "found unexpected status in initialized pool. pool_id: {:?}. status: {:?}",
                    pool_id,
                    pool.pool_status
                );
            } else {
                // Check that pool status was not accidentally set to `Initialized`.
                assert_ne!(
                    pool.pool_status,
                    PoolStatus::Initialized,
                    "found unexpected status in non-initialized pool. pool_id: {:?}. status: {:?}",
                    pool_id,
                    pool.pool_status
                );
            }
        }
        Ok(())
    }
}

impl<T: Config> OnRuntimeUpgrade for CleanUpStorageForResolvedOrClosedMarkets<T> {
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);

        if StorageVersion::get::<Pallet<T>>()
            != PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION_FOR_CLEANUP_STORAGE_FOR_RESOLVED_MARKETS
        {
            log::info!("Skipping storage cleanup; prediction-markets already up to date");
            return total_weight;
        }

        total_weight = total_weight.saturating_add(T::DbWeight::get().reads(2));
        log::info!("Starting storage cleanup of CleanUpStorageForResolvedOrClosedMarkets");

        let dispute_period = BLOCKS_PER_DAY as u32;
        let current_block: T::BlockNumber = <frame_system::Pallet<T>>::block_number();
        let last_dp_end_block =
            current_block.saturating_sub(dispute_period.into()).saturating_sub(1_u32.into());
        type DisputeBlockToMarketIdsTuple<T> =
            (<T as frame_system::Config>::BlockNumber, BoundedVec<MarketIdOf<T>, CacheSize>);
        type IterType<T> = PrefixIterator<DisputeBlockToMarketIdsTuple<T>>;

        let market_ids_per_dispute_iterator: IterType<T> =
            frame_support::migration::storage_key_iter::<_, _, Twox64Concat>(
                b"PredictionMarkets",
                b"MarketIdsPerDisputeBlock",
            );

        let market_ids_tobe_removed_per_dispute: Vec<DisputeBlockToMarketIdsTuple<T>> =
            market_ids_per_dispute_iterator
                .filter(|(dispute_start_block, _market_ids)| {
                    *dispute_start_block <= last_dp_end_block
                })
                .collect();
        for (dispute_start_block, _market_ids) in market_ids_tobe_removed_per_dispute {
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
            MarketIdsPerDisputeBlock::<T>::remove(dispute_start_block);
        }

        let market_ids_per_report_iterator: IterType<T> =
            frame_support::migration::storage_key_iter::<_, _, Twox64Concat>(
                b"PredictionMarkets",
                b"MarketIdsPerReportBlock",
            );

        let market_ids_tobe_removed_per_report: Vec<_> = market_ids_per_report_iterator
            .filter(|(dispute_start_block, _market_ids)| *dispute_start_block <= last_dp_end_block)
            .collect();
        for (dispute_start_block, _market_ids) in market_ids_tobe_removed_per_report {
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
            MarketIdsPerReportBlock::<T>::remove(dispute_start_block);
        }
        StorageVersion::new(
            PREDICTION_MARKETS_NEXT_STORAGE_VERSION_FOR_CLEANUP_STORAGE_FOR_RESOLVED_MARKETS,
        )
        .put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

        log::info!("Completed storage cleanup of CleanUpStorageForResolvedOrClosedMarkets");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        let dispute_period = BLOCKS_PER_DAY as u32;
        let current_block: T::BlockNumber = <frame_system::Pallet<T>>::block_number();
        let last_dp_end_block =
            current_block.saturating_sub(dispute_period.into()).saturating_sub(1_u32.into());
        type DisputeBlockToMarketIdsTuple<T> =
            (<T as frame_system::Config>::BlockNumber, BoundedVec<MarketIdOf<T>, CacheSize>);
        type IterType<T> = PrefixIterator<DisputeBlockToMarketIdsTuple<T>>;

        let mut market_ids_per_dispute_iterator: IterType<T> =
            frame_support::migration::storage_key_iter::<_, _, Twox64Concat>(
                b"PredictionMarkets",
                b"MarketIdsPerDisputeBlock",
            );
        market_ids_per_dispute_iterator.try_for_each(
            |(dispute_start_block, market_ids)| -> Result<(), &'static str> {
                assert!(
                    dispute_start_block > last_dp_end_block,
                    "found unexpected storage key in MarketIdsPerDisputeBlock. \
                     dispute_start_block: {:?}, last_dp_end_block: {:?} market_ids: {:?}",
                    dispute_start_block,
                    last_dp_end_block,
                    market_ids
                );

                market_ids.iter().try_for_each(|market_id| -> Result<(), &'static str> {
                    let market = T::MarketCommons::market(market_id)
                        .map_err(|_| "invalid market_id found.")?;
                    assert!(
                        market.status == MarketStatus::Disputed,
                        "found unexpected market status. market_id: {:?}, status: {:?}",
                        market_id,
                        market.status
                    );
                    Ok(())
                })?;
                Ok(())
            },
        )?;
        let mut market_ids_per_reported_iterator: IterType<T> =
            frame_support::migration::storage_key_iter::<_, _, Twox64Concat>(
                b"PredictionMarkets",
                b"MarketIdsPerReportBlock",
            );
        market_ids_per_reported_iterator.try_for_each(
            |(dispute_start_block, market_ids)| -> Result<(), &'static str> {
                assert!(
                    dispute_start_block > last_dp_end_block,
                    "found unexpected storage key in MarketIdsPerReportBlock. \
                     dispute_start_block: {:?}, last_dp_end_block: {:?}, market_ids: {:?}",
                    dispute_start_block,
                    last_dp_end_block,
                    market_ids
                );

                market_ids.iter().try_for_each(|market_id| -> Result<(), &'static str> {
                    let market = T::MarketCommons::market(market_id)
                        .map_err(|_| "invalid market_id found.")?;
                    assert!(
                        matches!(market.status, MarketStatus::Reported | MarketStatus::Disputed),
                        "found unexpected market status. market_id: {:?}, status: {:?}",
                        market_id,
                        market.status
                    );
                    Ok(())
                })?;
                Ok(())
            },
        )?;
        Ok(())
    }
}

impl<T: Config> OnRuntimeUpgrade for MigrateMarketIdsPerBlockStorage<T> {
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);

        if StorageVersion::get::<Pallet<T>>()
            != PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION_FOR_MIGRATE_MARKET_IDS_STORAGE
            || utility::get_on_chain_storage_version_of_market_commons_pallet()
                != MARKET_COMMONS_REQUIRED_STORAGE_VERSION
        {
            log::info!(
                "Skipping storage migration for MarketIds; prediction-markets already up to date"
            );
            return total_weight;
        }

        total_weight = total_weight.saturating_add(T::DbWeight::get().reads(2));
        log::info!("Starting storage cleanup of MigrateMarketIdsPerBlockStorage");

        type DisputeBlockToMarketIdsTuple<T> =
            (<T as frame_system::Config>::BlockNumber, BoundedVec<MarketIdOf<T>, CacheSize>);
        type IterType<T> = PrefixIterator<DisputeBlockToMarketIdsTuple<T>>;

        let market_ids_per_dispute_iterator: IterType<T> =
            frame_support::migration::storage_key_iter::<_, _, Twox64Concat>(
                b"PredictionMarkets",
                b"MarketIdsPerDisputeBlock",
            );

        let market_ids_per_dispute: Vec<DisputeBlockToMarketIdsTuple<T>> =
            market_ids_per_dispute_iterator.collect();
        for (dispute_start_block, market_ids) in market_ids_per_dispute {
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(2));
            // NOTE: These migration only makes sense on BS runtime so its fine to assume
            // DisputePeriod equal to BLOCKS_PER_DAY
            let dispute_period: T::BlockNumber = BLOCKS_PER_DAY.saturated_into::<u32>().into();
            let new_dispute_start_block = dispute_start_block.saturating_add(dispute_period);
            MarketIdsPerDisputeBlock::<T>::insert(new_dispute_start_block, market_ids);
            MarketIdsPerDisputeBlock::<T>::remove(dispute_start_block);
        }

        let market_ids_per_report_iterator: IterType<T> =
            frame_support::migration::storage_key_iter::<_, _, Twox64Concat>(
                b"PredictionMarkets",
                b"MarketIdsPerReportBlock",
            );

        let market_ids_per_report: Vec<_> = market_ids_per_report_iterator.collect();
        for (dispute_start_block, market_ids) in market_ids_per_report {
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(2));
            // NOTE: These migration only makes sense on BS runtime so its fine to assume
            // DisputePeriod equal to BLOCKS_PER_DAY
            let dispute_period: T::BlockNumber = BLOCKS_PER_DAY.saturated_into::<u32>().into();
            let new_dispute_start_block = dispute_start_block.saturating_add(dispute_period);
            MarketIdsPerReportBlock::<T>::insert(new_dispute_start_block, market_ids);
            MarketIdsPerReportBlock::<T>::remove(dispute_start_block);
        }
        StorageVersion::new(PREDICTION_MARKETS_NEXT_STORAGE_VERSION_FOR_MIGRATE_MARKET_IDS_STORAGE)
            .put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

        log::info!("Completed storage migration of MigrateMarketIdsPerBlockStorage");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        let dispute_period: T::BlockNumber = BLOCKS_PER_DAY.saturated_into::<u32>().into();
        for (key, market_ids) in MarketIdsPerDisputeBlock::<T>::iter() {
            for market_id in market_ids {
                let disputes = Disputes::<T>::get(&market_id);
                let dispute = disputes.last().ok_or("No dispute found")?;
                assert!(key == dispute.at + dispute_period)
            }
        }
        for (key, market_ids) in MarketIdsPerReportBlock::<T>::iter() {
            for market_id in market_ids {
                let market =
                    T::MarketCommons::market(&market_id).map_err(|_| "invalid market_id")?;
                let report = market.report.ok_or("No report found")?;
                assert!(key == report.at + dispute_period)
            }
        }
        Ok(())
    }
}
// We use these utilities to prevent having to make the swaps pallet a dependency of
// prediciton-markets. The calls are based on the implementation of `StorageVersion`, found here:
// https://github.com/paritytech/substrate/blob/bc7a1e6c19aec92bfa247d8ca68ec63e07061032/frame/support/src/traits/metadata.rs#L168-L230
// and previous migrations.
mod utility {
    use crate::{BalanceOf, Config, MarketIdOf};
    use alloc::vec::Vec;
    use frame_support::{
        migration::{get_storage_value, put_storage_value},
        storage::{storage_prefix, unhashed},
        traits::StorageVersion,
        Blake2_128Concat, StorageHasher,
    };
    use parity_scale_codec::Encode;
    use zeitgeist_primitives::types::{Pool, PoolId};

    const SWAPS: &[u8] = b"Swaps";
    const POOLS: &[u8] = b"Pools";

    fn storage_prefix_of_swaps_pallet() -> [u8; 32] {
        storage_prefix(b"Swaps", b":__STORAGE_VERSION__:")
    }

    pub fn storage_prefix_of_market_common_pallet() -> [u8; 32] {
        storage_prefix(b"MarketCommons", b":__STORAGE_VERSION__:")
    }

    fn key_to_hash<H, K>(key: K) -> Vec<u8>
    where
        H: StorageHasher,
        K: Encode,
    {
        key.using_encoded(H::hash).as_ref().to_vec()
    }

    pub fn get_on_chain_storage_version_of_swaps_pallet() -> StorageVersion {
        let key = storage_prefix_of_swaps_pallet();
        unhashed::get_or_default(&key)
    }

    pub fn get_on_chain_storage_version_of_market_commons_pallet() -> StorageVersion {
        let key = storage_prefix_of_market_common_pallet();
        unhashed::get_or_default(&key)
    }

    pub fn put_storage_version_of_swaps_pallet(value: u16) {
        let key = storage_prefix_of_swaps_pallet();
        unhashed::put(&key, &StorageVersion::new(value));
    }

    // pub fn put_storage_version_of_market_commons_pallet(value: u16) {
    //     let key = storage_prefix_of_market_common_pallet();
    //     unhashed::put(&key, &StorageVersion::new(value));
    // }

    pub fn get_pool<T: Config>(pool_id: PoolId) -> Option<Pool<BalanceOf<T>, MarketIdOf<T>>> {
        let hash = key_to_hash::<Blake2_128Concat, PoolId>(pool_id);
        let pool_maybe =
            get_storage_value::<Option<Pool<BalanceOf<T>, MarketIdOf<T>>>>(SWAPS, POOLS, &hash);
        pool_maybe.unwrap_or(None)
    }

    pub fn set_pool<T: Config>(pool_id: PoolId, pool: Pool<BalanceOf<T>, MarketIdOf<T>>) {
        let hash = key_to_hash::<Blake2_128Concat, PoolId>(pool_id);
        put_storage_value(SWAPS, POOLS, &hash, Some(pool));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{mock::*, CacheSize, Disputes, MomentOf};
    use alloc::{vec, vec::Vec};
    use frame_support::{assert_ok, storage::unhashed};
    use orml_traits::MultiCurrency;
    use sp_runtime::traits::BlockNumberProvider;
    use zeitgeist_primitives::{
        constants::{BASE, MILLISECS_PER_BLOCK},
        traits::Swaps as SwapsApi,
        types::{
            Asset, BlockNumber, Deadlines, Market, MarketCreation, MarketDispute,
            MarketDisputeMechanism, MarketPeriod, MarketType, MultiHash, OutcomeReport, PoolStatus,
            Report,
        },
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    #[test]
    fn test_on_runtime_upgrade_on_untouched_chain() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            MigrateMarketPoolsBeforeOpen::<Runtime>::on_runtime_upgrade();
            CleanUpStorageForResolvedOrClosedMarkets::<Runtime>::on_runtime_upgrade();
        });
    }

    #[test]
    fn on_runtime_upgrade_updates_storage_versions() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            MigrateMarketPoolsBeforeOpen::<Runtime>::on_runtime_upgrade();
            CleanUpStorageForResolvedOrClosedMarkets::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<Pallet<Runtime>>(),
                PREDICTION_MARKETS_NEXT_STORAGE_VERSION_FOR_CLEANUP_STORAGE_FOR_RESOLVED_MARKETS
            );
            assert_eq!(
                utility::get_on_chain_storage_version_of_swaps_pallet(),
                SWAPS_NEXT_STORAGE_VERSION
            );
        });
    }

    #[test]
    fn test_market_ids_per_open_block_on_runtime_upgrade() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 1_000 * BASE));

            // Markets which end here will have to be closed on migration:
            let time_11: MomentOf<Runtime> = (11 * MILLISECS_PER_BLOCK).into();
            let time_22: MomentOf<Runtime> = (22 * MILLISECS_PER_BLOCK).into();
            let time_33: MomentOf<Runtime> = (33 * MILLISECS_PER_BLOCK).into();
            let time_77: MomentOf<Runtime> = (77 * MILLISECS_PER_BLOCK).into();
            let time_11_frame = PredictionMarkets::calculate_time_frame_of_moment(time_11);
            let time_33_frame = PredictionMarkets::calculate_time_frame_of_moment(time_33);

            create_test_market_with_pool(MarketPeriod::Block(11..33));
            create_test_market_with_pool(MarketPeriod::Timestamp(time_11..time_33));
            create_test_market_with_pool(MarketPeriod::Block(33..77));
            create_test_market_with_pool(MarketPeriod::Timestamp(time_33..time_77));

            // Drain storage to simulate old code.
            MarketIdsPerOpenBlock::<Runtime>::drain().last();
            MarketIdsPerOpenTimeFrame::<Runtime>::drain().last();

            run_to_block(22);
            set_timestamp_for_on_initialize(time_22);
            MigrateMarketPoolsBeforeOpen::<Runtime>::on_runtime_upgrade();

            let auto_open_blocks_11 = MarketIdsPerOpenBlock::<Runtime>::get(11);
            assert_eq!(auto_open_blocks_11.len(), 0);
            let auto_open_blocks_33 = MarketIdsPerOpenBlock::<Runtime>::get(33);
            assert_eq!(auto_open_blocks_33, vec![2]);

            let auto_open_frames_11 = MarketIdsPerOpenTimeFrame::<Runtime>::get(time_11_frame);
            assert_eq!(auto_open_frames_11.len(), 0);
            let auto_open_frames_33 = MarketIdsPerOpenTimeFrame::<Runtime>::get(time_33_frame);
            assert_eq!(auto_open_frames_33, vec![3]);

            assert_eq!(Swaps::pool(0).unwrap().pool_status, PoolStatus::Active);
            assert_eq!(Swaps::pool(1).unwrap().pool_status, PoolStatus::Active);
            assert_eq!(Swaps::pool(2).unwrap().pool_status, PoolStatus::Initialized);
            assert_eq!(Swaps::pool(3).unwrap().pool_status, PoolStatus::Initialized);
        });
    }

    #[test]
    fn test_cleanup_storage_for_resolved_or_closed_market_on_runtime_upgrade() {
        ExtBuilder::default().build().execute_with(|| {
            StorageVersion::new(PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION_FOR_CLEANUP_STORAGE_FOR_RESOLVED_MARKETS)
                .put::<Pallet<Runtime>>();

            System::set_block_number(2);
            let market_ids = BoundedVec::<MarketIdOf<Runtime>, CacheSize>::try_from(vec![0, 1])
                .expect("BoundedVec creation failed");
            let dispute_block = System::current_block_number().saturating_sub(1_u32.into());
            let dispute_period : <Runtime as frame_system::Config>::BlockNumber = (BLOCKS_PER_DAY as u32).into();
            MarketIdsPerDisputeBlock::<Runtime>::insert(dispute_block, market_ids.clone());
            MarketIdsPerReportBlock::<Runtime>::insert(dispute_block, market_ids.clone());
            System::set_block_number(System::current_block_number() + dispute_period);
            assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(dispute_block).len(), 2);
            assert_eq!(MarketIdsPerReportBlock::<Runtime>::get(dispute_block).len(), 2);
            CleanUpStorageForResolvedOrClosedMarkets::<Runtime>::on_runtime_upgrade();
            assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(dispute_block).len(), 0);
            assert_eq!(MarketIdsPerReportBlock::<Runtime>::get(dispute_block).len(), 0);

            let dispute_block = System::current_block_number();
            MarketIdsPerDisputeBlock::<Runtime>::insert(dispute_block, market_ids.clone());
            MarketIdsPerReportBlock::<Runtime>::insert(dispute_block, market_ids);
            System::set_block_number(System::current_block_number() + dispute_period - 1);
            CleanUpStorageForResolvedOrClosedMarkets::<Runtime>::on_runtime_upgrade();
            // storage is untouched as DisputeDuration is not reached.
            assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(dispute_block).len(), 2);
            assert_eq!(MarketIdsPerReportBlock::<Runtime>::get(dispute_block).len(), 2);
        });
    }

    #[test]
    fn test_migrate_market_ids_on_runtime_upgrade() {
        ExtBuilder::default().build().execute_with(|| {
            StorageVersion::new(
                PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION_FOR_MIGRATE_MARKET_IDS_STORAGE,
            )
            .put::<Pallet<Runtime>>();
            let key = utility::storage_prefix_of_market_common_pallet();
            unhashed::put(&key, &StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION));

            System::set_block_number(1);
            create_test_maket();
            create_test_maket();
            let market_ids_reported =
                BoundedVec::<MarketIdOf<Runtime>, CacheSize>::try_from(vec![0])
                    .expect("boundedvec creation failed");
            let market_ids_disputed =
                BoundedVec::<MarketIdOf<Runtime>, CacheSize>::try_from(vec![1])
                    .expect("boundedvec creation failed");
            System::set_block_number(4);
            let dispute_block = System::current_block_number().saturating_sub(1_u32.into());
            MarketIdsPerDisputeBlock::<Runtime>::insert(dispute_block, market_ids_disputed.clone());
            MarketIdsPerReportBlock::<Runtime>::insert(dispute_block, market_ids_reported.clone());
            let report = Report {
                at: dispute_block,
                by: BOB,
                outcome: zeitgeist_primitives::types::OutcomeReport::Categorical(3),
            };

            assert_ok!(<MarketCommons as MarketCommonsPalletApi>::mutate_market(&0, |market| {
                market.report = Some(report);
                Ok(())
            }));

            let dispute = MarketDispute {
                at: dispute_block,
                by: EVE,
                outcome: OutcomeReport::Categorical(1),
            };
            let disputes = BoundedVec::<
                MarketDispute<
                    <Runtime as frame_system::Config>::AccountId,
                    <Runtime as frame_system::Config>::BlockNumber,
                >,
                <Runtime as Config>::MaxDisputes,
            >::try_from(vec![dispute])
            .expect("boundedvec creation failed");
            Disputes::<Runtime>::insert(1, disputes);
            MigrateMarketIdsPerBlockStorage::<Runtime>::on_runtime_upgrade();
            let market_reported = MarketCommons::market(&0).expect("invalid market_id");
            let market_disputed = MarketCommons::market(&1).expect("invalid market_id");
            assert_eq!(
                MarketIdsPerDisputeBlock::<Runtime>::get(
                    dispute_block + market_disputed.deadlines.dispute_duration
                ),
                market_ids_disputed
            );
            assert_eq!(
                MarketIdsPerReportBlock::<Runtime>::get(
                    dispute_block + market_reported.deadlines.dispute_duration
                ),
                market_ids_reported
            );
        });
    }

    fn setup_chain() {
        StorageVersion::new(PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION_FOR_MIGRATE_MARKET_POOLS)
            .put::<Pallet<Runtime>>();
        utility::put_storage_version_of_swaps_pallet(SWAPS_REQUIRED_STORAGE_VERSION);
    }

    fn create_test_market_with_pool(period: MarketPeriod<BlockNumber, MomentOf<Runtime>>) {
        let amount = 100 * BASE;
        let category_count = 5;
        let deadlines = Deadlines {
            oracle_delay: <Runtime as crate::Config>::MaxOracleDelay::get(),
            oracle_duration: <Runtime as crate::Config>::MaxOracleDuration::get(),
            dispute_duration: <Runtime as crate::Config>::MinDisputeDuration::get(),
        };
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            BOB,
            period,
            deadlines,
            gen_metadata(0),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::Authorized(CHARLIE),
            BASE / 10,
            amount,
            vec![BASE; category_count.into()],
        ));

        // Open pool to simulate old market creation.
        let market_id = MarketCommons::latest_market_id().unwrap();
        let pool_id = MarketCommons::market_pool(&market_id).unwrap();
        Swaps::open_pool(pool_id).unwrap();
    }

    fn create_test_maket() {
        let deadlines = Deadlines {
            oracle_delay: <Runtime as crate::Config>::MaxOracleDelay::get(),
            oracle_duration: <Runtime as crate::Config>::MaxOracleDuration::get(),
            dispute_duration: (BLOCKS_PER_DAY as u32).into(),
        };
        let mut metadata = [0; 50];
        metadata[0] = 0x15;
        metadata[1] = 0x30;
        let market = Market {
            creation: MarketCreation::Advised,
            creator_fee: 0,
            creator: ALICE,
            market_type: MarketType::Categorical(5),
            dispute_mechanism: MarketDisputeMechanism::Authorized(CHARLIE),
            metadata: Vec::from(metadata),
            oracle: BOB,
            period: MarketPeriod::Block(2..10),
            deadlines,
            report: None,
            resolved_outcome: None,
            status: MarketStatus::Active,
            scoring_rule: zeitgeist_primitives::types::ScoringRule::CPMM,
        };
        let _res = <MarketCommons as MarketCommonsPalletApi>::push_market(market);
    }

    fn gen_metadata(byte: u8) -> MultiHash {
        let mut metadata = [byte; 50];
        metadata[0] = 0x15;
        metadata[1] = 0x30;
        MultiHash::Sha3_384(metadata)
    }
}
