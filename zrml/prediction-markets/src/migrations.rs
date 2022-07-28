use crate::{Config, MarketIdsPerOpenBlock, MarketIdsPerOpenTimeFrame, Pallet};
use frame_support::{
    dispatch::Weight,
    log,
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
};
use zeitgeist_primitives::types::{MarketPeriod, MarketStatus, PoolStatus};
use zrml_market_commons::MarketCommonsPalletApi;

const SWAPS_REQUIRED_STORAGE_VERSION: u16 = 2;
const SWAPS_NEXT_STORAGE_VERSION: u16 = 3;
const PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION: u16 = 2;
const PREDICTION_MARKETS_NEXT_STORAGE_VERSION: u16 = 3;

pub struct MigrateMarketPoolsBeforeOpen<T>(PhantomData<T>);

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
        if StorageVersion::get::<Pallet<T>>() != PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION {
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

            let not_yet_open = match market.period {
                MarketPeriod::Block(ref range) => current_block < range.start,
                MarketPeriod::Timestamp(ref range) => {
                    current_time_frame < Pallet::<T>::calculate_time_frame_of_moment(range.start)
                }
            };
            if not_yet_open {
                total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
                let mut pool = match utility::get_pool::<T>(pool_id) {
                    Some(pool) => pool,
                    None => {
                        log::warn!(
                            "no pool found. market_id: {:?}. pool_id: {:?}",
                            market_id,
                            pool_id,
                        );
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
                            let _ = MarketIdsPerOpenTimeFrame::<T>::try_mutate(
                                &open_time_frame,
                                |ids| ids.try_push(market_id),
                            );
                        }
                    }
                    total_weight =
                        total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
                } else {
                    log::warn!(
                        "found pool with unexpected status. market_id: {:?}. pool_id: {:?}",
                        market_id,
                        pool_id,
                    );
                }
            }
        }

        StorageVersion::new(PREDICTION_MARKETS_NEXT_STORAGE_VERSION).put::<Pallet<T>>();
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

    pub fn put_storage_version_of_swaps_pallet(value: u16) {
        let key = storage_prefix_of_swaps_pallet();
        unhashed::put(&key, &StorageVersion::new(value));
    }

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
    use crate::{mock::*, MomentOf};
    use frame_support::assert_ok;
    use orml_traits::MultiCurrency;
    use zeitgeist_primitives::{
        constants::{BASE, MILLISECS_PER_BLOCK},
        traits::Swaps as SwapsApi,
        types::{Asset, BlockNumber, MarketDisputeMechanism, MarketType, MultiHash, PoolStatus},
    };

    #[test]
    fn test_on_runtime_upgrade_on_untouched_chain() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            MigrateMarketPoolsBeforeOpen::<Runtime>::on_runtime_upgrade();
        });
    }

    #[test]
    fn on_runtime_upgrade_updates_storage_versions() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            MigrateMarketPoolsBeforeOpen::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<Pallet<Runtime>>(),
                PREDICTION_MARKETS_NEXT_STORAGE_VERSION
            );
            assert_eq!(
                utility::get_on_chain_storage_version_of_swaps_pallet(),
                SWAPS_NEXT_STORAGE_VERSION
            );
        });
    }

    #[test]
    fn test_on_runtime_upgrade_with_sample_markets() {
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

    fn setup_chain() {
        StorageVersion::new(PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
        utility::put_storage_version_of_swaps_pallet(SWAPS_REQUIRED_STORAGE_VERSION);
    }

    fn create_test_market_with_pool(period: MarketPeriod<BlockNumber, MomentOf<Runtime>>) {
        let amount = 100 * BASE;
        let category_count = 5;
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            BOB,
            period,
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

    fn gen_metadata(byte: u8) -> MultiHash {
        let mut metadata = [byte; 50];
        metadata[0] = 0x15;
        metadata[1] = 0x30;
        MultiHash::Sha3_384(metadata)
    }
}
