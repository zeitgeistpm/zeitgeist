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
    CacheSize, Config, Disputes, MarketIdOf, MarketIdsPerDisputeBlock, MarketIdsPerReportBlock,
    Pallet,
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
use zeitgeist_primitives::constants::BLOCKS_PER_DAY;
use zrml_market_commons::MarketCommonsPalletApi;

const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 2;

const PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION_FOR_MIGRATE_MARKET_IDS_STORAGE: u16 = 4;
const PREDICTION_MARKETS_NEXT_STORAGE_VERSION_FOR_MIGRATE_MARKET_IDS_STORAGE: u16 = 5;

pub struct MigrateMarketIdsPerBlockStorage<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateMarketIdsPerBlockStorage<T> {
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(2);

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
        total_weight = total_weight
            .saturating_add(T::DbWeight::get().reads(market_ids_per_dispute.len() as u64));
        for (dispute_start_block, market_ids) in &market_ids_per_dispute {
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
            let dispute_duration = if cfg!(feature = "with-zeitgeist-runtime") {
                (4_u64 * BLOCKS_PER_DAY).saturated_into::<u32>().into()
            } else {
                // assuming battery-station
                BLOCKS_PER_DAY.saturated_into::<u32>().into()
            };

            let new_dispute_start_block = dispute_start_block.saturating_add(dispute_duration);
            MarketIdsPerDisputeBlock::<T>::insert(new_dispute_start_block, market_ids);
        }
        for (dispute_start_block, _market_ids) in market_ids_per_dispute {
            MarketIdsPerDisputeBlock::<T>::remove(dispute_start_block);
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        }

        let market_ids_per_report_iterator: IterType<T> =
            frame_support::migration::storage_key_iter::<_, _, Twox64Concat>(
                b"PredictionMarkets",
                b"MarketIdsPerReportBlock",
            );

        let market_ids_per_report: Vec<DisputeBlockToMarketIdsTuple<T>> =
            market_ids_per_report_iterator.collect();
        total_weight = total_weight
            .saturating_add(T::DbWeight::get().reads(market_ids_per_report.len() as u64));
        for (dispute_start_block, market_ids) in &market_ids_per_report {
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
            let dispute_duration = if cfg!(feature = "with-zeitgeist-runtime") {
                (4_u64 * BLOCKS_PER_DAY).saturated_into::<u32>().into()
            } else {
                // assuming battery-station
                BLOCKS_PER_DAY.saturated_into::<u32>().into()
            };

            let new_dispute_start_block = dispute_start_block.saturating_add(dispute_duration);
            MarketIdsPerReportBlock::<T>::insert(new_dispute_start_block, market_ids);
        }
        for (dispute_start_block, _market_ids) in market_ids_per_report {
            MarketIdsPerReportBlock::<T>::remove(dispute_start_block);
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
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
        for (key, market_ids) in MarketIdsPerDisputeBlock::<T>::iter() {
            for market_id in market_ids {
                let market =
                    T::MarketCommons::market(&market_id).map_err(|_| "invalid market_id")?;
                let disputes = Disputes::<T>::get(&market_id);
                let dispute = disputes.last().ok_or("No dispute found")?;
                assert_eq!(
                    key,
                    dispute.at + market.deadlines.dispute_duration,
                    "key in MarketIdsPerDisputeBlock must be equal to dispute.at + \
                     disputed_duration"
                );
            }
        }
        for (key, market_ids) in MarketIdsPerReportBlock::<T>::iter() {
            for market_id in market_ids {
                let market =
                    T::MarketCommons::market(&market_id).map_err(|_| "invalid market_id")?;
                let report = market.report.ok_or("No report found")?;
                assert_eq!(
                    key,
                    report.at + market.deadlines.dispute_duration,
                    "key in MarketIdsPerReportBlock must be equal to report.at + dispute_duration"
                );
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

    pub fn storage_prefix_of_market_common_pallet() -> [u8; 32] {
        storage_prefix(b"MarketCommons", b":__STORAGE_VERSION__:")
    }

    pub fn get_on_chain_storage_version_of_market_commons_pallet() -> StorageVersion {
        let key = storage_prefix_of_market_common_pallet();
        unhashed::get_or_default(&key)
    }

    #[allow(unused)]
    const SWAPS: &[u8] = b"Swaps";
    #[allow(unused)]
    const POOLS: &[u8] = b"Pools";
    #[allow(unused)]
    fn storage_prefix_of_swaps_pallet() -> [u8; 32] {
        storage_prefix(b"Swaps", b":__STORAGE_VERSION__:")
    }
    #[allow(unused)]
    fn key_to_hash<H, K>(key: K) -> Vec<u8>
    where
        H: StorageHasher,
        K: Encode,
    {
        key.using_encoded(H::hash).as_ref().to_vec()
    }
    #[allow(unused)]
    pub fn get_on_chain_storage_version_of_swaps_pallet() -> StorageVersion {
        let key = storage_prefix_of_swaps_pallet();
        unhashed::get_or_default(&key)
    }
    #[allow(unused)]
    pub fn put_storage_version_of_swaps_pallet(value: u16) {
        let key = storage_prefix_of_swaps_pallet();
        unhashed::put(&key, &StorageVersion::new(value));
    }
    #[allow(unused)]
    pub fn get_pool<T: Config>(pool_id: PoolId) -> Option<Pool<BalanceOf<T>, MarketIdOf<T>>> {
        let hash = key_to_hash::<Blake2_128Concat, PoolId>(pool_id);
        let pool_maybe =
            get_storage_value::<Option<Pool<BalanceOf<T>, MarketIdOf<T>>>>(SWAPS, POOLS, &hash);
        pool_maybe.unwrap_or(None)
    }
    #[allow(unused)]
    pub fn set_pool<T: Config>(pool_id: PoolId, pool: Pool<BalanceOf<T>, MarketIdOf<T>>) {
        let hash = key_to_hash::<Blake2_128Concat, PoolId>(pool_id);
        put_storage_value(SWAPS, POOLS, &hash, Some(pool));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{mock::*, CacheSize, Disputes};
    use alloc::{vec, vec::Vec};
    use frame_support::{assert_ok, storage::unhashed};
    use sp_runtime::traits::BlockNumberProvider;
    use zeitgeist_primitives::types::{
        Deadlines, Market, MarketCreation, MarketDispute, MarketDisputeMechanism, MarketPeriod,
        MarketStatus, MarketType, OutcomeReport, Report,
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    #[test]
    fn test_on_runtime_upgrade_on_untouched_chain() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            MigrateMarketIdsPerBlockStorage::<Runtime>::on_runtime_upgrade();
        });
    }

    #[test]
    fn on_runtime_upgrade_updates_storage_versions() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            MigrateMarketIdsPerBlockStorage::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<Pallet<Runtime>>(),
                PREDICTION_MARKETS_NEXT_STORAGE_VERSION_FOR_MIGRATE_MARKET_IDS_STORAGE
            );
        });
    }

    #[test]
    fn test_migrate_market_ids_on_runtime_upgrade() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();

            System::set_block_number(1);
            create_test_market();
            create_test_market();
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
        StorageVersion::new(
            PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION_FOR_MIGRATE_MARKET_IDS_STORAGE,
        )
        .put::<Pallet<Runtime>>();
        let key = utility::storage_prefix_of_market_common_pallet();
        unhashed::put(&key, &StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION));
    }

    fn create_test_market() {
        let dispute_duration = if cfg!(feature = "with-zeitgeist-runtime") {
            (4_u64 * BLOCKS_PER_DAY).saturated_into::<u32>().into()
        } else {
            // assuming battery-station
            BLOCKS_PER_DAY.saturated_into::<u32>().into()
        };

        let deadlines = Deadlines {
            grace_period: <Runtime as crate::Config>::MaxGracePeriod::get(),
            oracle_duration: <Runtime as crate::Config>::MaxOracleDuration::get(),
            dispute_duration,
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
}
