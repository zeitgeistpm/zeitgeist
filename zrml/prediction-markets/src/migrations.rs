use crate::{
    Config, LastTimeFrame, MarketIdsPerCloseBlock, MarketIdsPerCloseTimeFrame, MomentOf, Pallet,
};
use alloc::vec;
use frame_support::{
    dispatch::Weight,
    log,
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
};
use zeitgeist_primitives::{
    traits::Swaps as SwapsPalletApi,
    types::{Market, MarketPeriod, MarketStatus, PoolStatus},
};
use zrml_market_commons::MarketCommonsPalletApi;

const PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION: u16 = 0;
const PREDICTION_MARKETS_NEXT_STORAGE_VERSION: u16 = 1;
const SWAPS_REQUIRED_STORAGE_VERSION: u16 = 1;
const SWAPS_NEXT_STORAGE_VERSION: u16 = 2;

pub struct MigrateMarketIdsPerClose<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigrateMarketIdsPerClose<T> {
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);

        if StorageVersion::get::<Pallet<T>>() != PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "Skipping storage migration of MarketIdsPerClose*; prediction-markets already up \
                 to date"
            );
            return total_weight;
        }
        total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
        if utility::get_on_chain_storage_version_of_swaps_pallet() != SWAPS_REQUIRED_STORAGE_VERSION
        {
            log::info!(
                "Skipping storage migration of MarketIdsPerClose*; swaps already up to date"
            );
            return total_weight;
        }
        log::info!("Starting storage migration of MarketIdsPerClose*");

        let current_block = <frame_system::Pallet<T>>::block_number();
        let current_time_frame =
            Pallet::<T>::calculate_time_frame_of_moment(T::MarketCommons::now());
        total_weight = total_weight.saturating_add(T::DbWeight::get().reads(2));

        // Cache markets flagged for removal during iteration to avoid UB!
        let mut markets_to_reject = vec![];
        let mut close_or_reject_market =
            |market_id, market: Market<T::AccountId, T::BlockNumber, MomentOf<T>>| {
                match market.status {
                    MarketStatus::Active => {
                        let weight = Pallet::<T>::close_market(&market_id).unwrap_or(0);
                        weight
                    }
                    MarketStatus::Proposed => {
                        markets_to_reject.push((market_id, market));
                        0
                    }
                    _ => 0, // Closure shouldn't be called with other values.
                }
            };

        for (market_id, market) in T::MarketCommons::market_iter() {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

            match market.status {
                MarketStatus::Active | MarketStatus::Proposed => (),
                MarketStatus::Resolved => {
                    if let Ok(pool_id) = T::MarketCommons::market_pool(&market_id) {
                        // Since the market is resolved, the pool **should** be stale/closed and
                        // cleaned up, we only need to change the state.
                        let mut pool = match utility::get_pool::<T>(pool_id) {
                            Some(pool) => pool,
                            _ => continue,
                        };
                        pool.pool_status = PoolStatus::Clean;
                        utility::set_pool::<T>(pool_id, pool);
                    };
                    continue; // No need to check the range of a resolved market!
                }
                _ => {
                    // Close the pool, if the market is not active or already resolved. No need to
                    // single out Rikiddo pools - we don't have any of them on our networks.
                    if let Ok(pool_id) = T::MarketCommons::market_pool(&market_id) {
                        // This call is infallible in this context, so unwrap_or is safe.
                        let weight = T::Swaps::close_pool(pool_id).unwrap_or(0);
                        total_weight = total_weight.saturating_add(weight);
                    }
                    // Add read for MarketPool:
                    total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
                    continue;
                }
            };

            match market.period {
                MarketPeriod::Block(ref range) => {
                    if current_block < range.end {
                        let _ = MarketIdsPerCloseBlock::<T>::try_mutate(range.end, |ids| {
                            ids.try_push(market_id)
                        });
                        total_weight =
                            total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
                    } else {
                        let weight = close_or_reject_market(market_id, market);
                        total_weight = total_weight.saturating_add(weight);
                    }
                }
                MarketPeriod::Timestamp(ref range) => {
                    let end_time_frame = Pallet::<T>::calculate_time_frame_of_moment(range.end);
                    if current_time_frame < end_time_frame {
                        let _ =
                            MarketIdsPerCloseTimeFrame::<T>::try_mutate(end_time_frame, |ids| {
                                ids.try_push(market_id)
                            });
                        total_weight =
                            total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
                    } else {
                        let weight = close_or_reject_market(market_id, market);
                        total_weight = total_weight.saturating_add(weight);
                    }
                }
            };
        }

        // All markets flagged for removal are expired proposed.
        for (market_id, market) in markets_to_reject.into_iter() {
            // do_reject_market is infallible in this context, so unwrap_or is safe.
            let weight =
                Pallet::<T>::handle_expired_advised_market(&market_id, market).unwrap_or(0);
            total_weight = total_weight.saturating_add(weight);
        }

        LastTimeFrame::<T>::set(Some(current_time_frame));
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

        StorageVersion::new(PREDICTION_MARKETS_NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        utility::put_storage_version_of_swaps_pallet(SWAPS_NEXT_STORAGE_VERSION);
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(2));

        log::info!("Completed storage migration of MarketIdsPerClose*");
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
            match market.period {
                MarketPeriod::Timestamp(range) => {
                    let end_frame = Pallet::<T>::calculate_time_frame_of_moment(range.end);
                    let ids = MarketIdsPerCloseTimeFrame::<T>::get(end_frame);
                    let is_cached = ids.contains(&market_id);
                    // (We're ignoring Rikiddo markets)
                    if current_time_frame < end_frame {
                        assert!(
                            matches!(market.status, MarketStatus::Active | MarketStatus::Proposed),
                            "found unexpected status in active/proposed market {:?}: {:?}.",
                            market_id,
                            market.status
                        );
                        assert!(is_cached, "failed to find cache for market {:?}", market_id);
                    } else {
                        assert!(
                            matches!(
                                market.status,
                                MarketStatus::Closed
                                    | MarketStatus::Reported
                                    | MarketStatus::Disputed
                                    | MarketStatus::Resolved
                            ),
                            "found unexpected status in market {:?}: {:?}",
                            market_id,
                            market.status
                        );
                        // Note: Only checks if the market is cached in this time frame. The market
                        // might still be incorrectly cached, but we have no way of knowing this.
                        assert!(
                            !is_cached,
                            "unexpectedly found cache for market {:?} in frame {:?}",
                            market_id, end_frame,
                        );
                    }
                }
                MarketPeriod::Block(range) => {
                    // (We're ignoring Rikiddo markets)
                    let end_block = range.end;
                    let ids = MarketIdsPerCloseBlock::<T>::get(end_block);
                    let is_cached = ids.contains(&market_id);
                    if current_block < range.end {
                        assert!(
                            matches!(market.status, MarketStatus::Active | MarketStatus::Proposed),
                            "found unexpected status in active/proposed market {:?}: {:?}.",
                            market_id,
                            market.status
                        );
                        assert!(is_cached, "failed to find cache for market {:?}", market_id);
                    } else {
                        assert!(
                            matches!(
                                market.status,
                                MarketStatus::Closed
                                    | MarketStatus::Reported
                                    | MarketStatus::Disputed
                                    | MarketStatus::Resolved
                            ),
                            "found unexpected status in market {:?}: {:?}",
                            market_id,
                            market.status
                        );
                        // Note: Only checks if the market is cached in this block. The market
                        // might still be incorrectly cached, but we have no way of knowing this.
                        assert!(
                            !is_cached,
                            "unexpectedly found cache for market {:?} in block {:?}",
                            market_id, end_block,
                        );
                    }
                }
            }

            let pool_id = match T::MarketCommons::market_pool(&market_id) {
                Ok(pool_id) => pool_id,
                _ => continue,
            };
            let pool = match utility::get_pool::<T>(pool_id) {
                Some(pool) => pool,
                _ => continue,
            };
            // (Ignoring Rikiddo pools!)
            let pool_status_expected = match market.status {
                MarketStatus::Resolved => PoolStatus::Clean,
                MarketStatus::Active | MarketStatus::Proposed => PoolStatus::Active,
                _ => PoolStatus::Closed,
            };
            assert_eq!(
                pool.pool_status, pool_status_expected,
                "found unexpected pool status in pool {:?} of market {:?}: {:?}. Expected: {:?}",
                pool_id, market_id, pool.pool_status, pool_status_expected
            );
        }

        let last_time_frame = LastTimeFrame::<T>::get();
        let last_time_frame_expected = Some(current_time_frame);
        assert_eq!(
            last_time_frame, last_time_frame_expected,
            "found unexpected LastTimeFrame: {:?}. Expected: {:?}",
            last_time_frame, last_time_frame_expected,
        );

        let prediction_markets_storage_version = StorageVersion::get::<Pallet<T>>();
        assert_eq!(
            prediction_markets_storage_version, PREDICTION_MARKETS_NEXT_STORAGE_VERSION,
            "found unexpected prediction-markets pallet storage version. Found: {:?}. Expected: \
             {:?}",
            prediction_markets_storage_version, PREDICTION_MARKETS_NEXT_STORAGE_VERSION,
        );
        let swaps_storage_version = utility::get_on_chain_storage_version_of_swaps_pallet();
        assert_eq!(
            swaps_storage_version, SWAPS_NEXT_STORAGE_VERSION,
            "found unexpected swaps pallet storage version. Found: {:?}. Expected: {:?}",
            swaps_storage_version, SWAPS_NEXT_STORAGE_VERSION
        );

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
    use frame_support::{assert_err, assert_ok};
    use orml_traits::MultiCurrency;
    use zeitgeist_primitives::{
        constants::{BASE, MILLISECS_PER_BLOCK},
        types::{
            Asset, BlockNumber, MarketCreation, MarketDisputeMechanism, MarketType, MultiHash,
            PoolStatus, ScoringRule,
        },
    };

    #[test]
    fn test_on_runtime_upgrade_on_untouched_chain() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            MigrateMarketIdsPerClose::<Runtime>::on_runtime_upgrade();
        });
    }

    #[test]
    fn on_runtime_upgrade_updates_storage_versions() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            MigrateMarketIdsPerClose::<Runtime>::on_runtime_upgrade();
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
            let _ = Currency::deposit(Asset::Ztg, &ALICE, 1_000 * BASE);

            // Markets which end here will have to be closed on migration:
            let short_time: MomentOf<Runtime> = (5 * MILLISECS_PER_BLOCK).into();
            let short_time_frame = PredictionMarkets::calculate_time_frame_of_moment(short_time);
            // Markets which end here will end in the future:
            let long_time = 10 * short_time;
            let long_time_frame = PredictionMarkets::calculate_time_frame_of_moment(long_time);

            create_test_market_with_pool(MarketPeriod::Block(0..77), MarketStatus::Active, false);
            create_test_market(
                MarketPeriod::Block(0..77),
                MarketCreation::Permissionless,
                MarketStatus::Active,
            );
            create_test_market(
                MarketPeriod::Block(0..77),
                MarketCreation::Advised,
                MarketStatus::Proposed,
            );
            create_test_market_with_pool(
                MarketPeriod::Timestamp(0..long_time),
                MarketStatus::Active,
                false,
            );
            create_test_market(
                MarketPeriod::Timestamp(0..long_time),
                MarketCreation::Permissionless,
                MarketStatus::Active,
            );
            create_test_market(
                MarketPeriod::Timestamp(0..long_time),
                MarketCreation::Advised,
                MarketStatus::Proposed,
            );
            create_test_market_with_pool(MarketPeriod::Block(0..33), MarketStatus::Active, false);
            create_test_market(
                MarketPeriod::Block(0..33),
                MarketCreation::Permissionless,
                MarketStatus::Active,
            );
            create_test_market(
                MarketPeriod::Block(0..33),
                MarketCreation::Advised,
                MarketStatus::Proposed,
            );
            create_test_market_with_pool(
                MarketPeriod::Timestamp(0..short_time),
                MarketStatus::Active,
                false,
            );
            create_test_market(
                MarketPeriod::Timestamp(0..short_time),
                MarketCreation::Permissionless,
                MarketStatus::Active,
            );
            create_test_market(
                MarketPeriod::Timestamp(0..short_time),
                MarketCreation::Advised,
                MarketStatus::Proposed,
            );
            create_test_market_with_pool(
                MarketPeriod::Timestamp(0..short_time),
                MarketStatus::Resolved,
                true,
            );
            create_test_market_with_pool(
                MarketPeriod::Timestamp(0..short_time),
                MarketStatus::Disputed,
                false,
            );

            // Drain storage to simulate old code.
            MarketIdsPerCloseBlock::<Runtime>::drain().last();
            MarketIdsPerCloseTimeFrame::<Runtime>::drain().last();

            run_to_block(55);
            let now_time_stamp = 7 * short_time;
            let now_time_frame = PredictionMarkets::calculate_time_frame_of_moment(now_time_stamp);
            Timestamp::set_timestamp(now_time_stamp);

            MigrateMarketIdsPerClose::<Runtime>::on_runtime_upgrade();

            let auto_close_blocks_33 = MarketIdsPerCloseBlock::<Runtime>::get(33);
            assert_eq!(auto_close_blocks_33.len(), 0);
            let mut auto_close_blocks_77 = (*MarketIdsPerCloseBlock::<Runtime>::get(77)).clone();
            auto_close_blocks_77.sort(); // (Iteration above is without order)
            assert_eq!(auto_close_blocks_77, vec![0, 1, 2]);

            let auto_close_short = MarketIdsPerCloseTimeFrame::<Runtime>::get(short_time_frame);
            assert_eq!(auto_close_short.len(), 0);
            let mut auto_close_long =
                (*MarketIdsPerCloseTimeFrame::<Runtime>::get(long_time_frame)).clone();
            auto_close_long.sort(); // (Iteration above is without order)
            assert_eq!(*auto_close_long, vec![3, 4, 5]);

            // Check status and that only expired advised markets are removed.
            assert_eq!(MarketCommons::market(&0).unwrap().status, MarketStatus::Active);
            assert_eq!(MarketCommons::market(&1).unwrap().status, MarketStatus::Active);
            assert_eq!(MarketCommons::market(&2).unwrap().status, MarketStatus::Proposed);
            assert_eq!(MarketCommons::market(&3).unwrap().status, MarketStatus::Active);
            assert_eq!(MarketCommons::market(&4).unwrap().status, MarketStatus::Active);
            assert_eq!(MarketCommons::market(&5).unwrap().status, MarketStatus::Proposed);
            assert_eq!(MarketCommons::market(&6).unwrap().status, MarketStatus::Closed);
            assert_eq!(MarketCommons::market(&7).unwrap().status, MarketStatus::Closed);
            assert_eq!(MarketCommons::market(&9).unwrap().status, MarketStatus::Closed);
            assert_eq!(MarketCommons::market(&10).unwrap().status, MarketStatus::Closed);
            assert_eq!(MarketCommons::market(&12).unwrap().status, MarketStatus::Resolved);
            assert_eq!(MarketCommons::market(&13).unwrap().status, MarketStatus::Disputed);
            assert_err!(
                MarketCommons::market(&8),
                zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
            );
            assert_err!(
                MarketCommons::market(&11),
                zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
            );

            assert_eq!(Swaps::pool(0).unwrap().pool_status, PoolStatus::Active);
            assert_eq!(Swaps::pool(1).unwrap().pool_status, PoolStatus::Active);
            assert_eq!(Swaps::pool(2).unwrap().pool_status, PoolStatus::Closed);
            assert_eq!(Swaps::pool(3).unwrap().pool_status, PoolStatus::Closed);
            assert_eq!(Swaps::pool(4).unwrap().pool_status, PoolStatus::Clean);
            assert_eq!(Swaps::pool(5).unwrap().pool_status, PoolStatus::Closed);

            assert_eq!(LastTimeFrame::<Runtime>::get().unwrap(), now_time_frame);
        });
    }

    fn setup_chain() {
        StorageVersion::new(PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
        utility::put_storage_version_of_swaps_pallet(SWAPS_REQUIRED_STORAGE_VERSION);
    }

    fn create_test_market_with_pool(
        period: MarketPeriod<BlockNumber, MomentOf<Runtime>>,
        market_status: MarketStatus,
        pool_is_closed: bool,
    ) {
        let amount = 100 * BASE;
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            BOB,
            period,
            gen_metadata(0),
            MarketType::Categorical(5),
            MarketDisputeMechanism::Authorized(CHARLIE),
            amount,
            vec![BASE; 6],
        ));
        let market_id = MarketCommons::latest_market_id().unwrap();
        if pool_is_closed {
            let pool_id = MarketCommons::market_pool(&market_id).unwrap();
            assert_ok!(Swaps::close_pool(pool_id));
        }
        assert_ok!(MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        }));
    }

    fn create_test_market(
        period: MarketPeriod<BlockNumber, MomentOf<Runtime>>,
        market_creation: MarketCreation,
        market_status: MarketStatus,
    ) {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            period,
            gen_metadata(0),
            market_creation,
            MarketType::Categorical(5),
            MarketDisputeMechanism::Authorized(CHARLIE),
            ScoringRule::CPMM,
        ));
        let market_id = MarketCommons::latest_market_id().unwrap();
        assert_ok!(MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        }));
    }

    fn gen_metadata(byte: u8) -> MultiHash {
        let mut metadata = [byte; 50];
        metadata[0] = 0x15;
        metadata[1] = 0x30;
        MultiHash::Sha3_384(metadata)
    }
}
