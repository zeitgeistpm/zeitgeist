use crate::{BalanceOf, Config, Pallet};
use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use frame_support::{
    dispatch::Weight,
    log,
    pallet_prelude::PhantomData,
    storage::migration::{put_storage_value, storage_iter},
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use zeitgeist_primitives::types::{Asset, Pool, PoolStatus, ScoringRule};

const SWAPS: &[u8] = b"Swaps";
const POOLS: &[u8] = b"Pools";
const REQUIRED_STORAGE_VERSION: u16 = 0;
const NEXT_STORAGE_VERSION: u16 = 1;

#[derive(Decode, Encode, Clone)]
struct PoolDeprecated<Balance, MarketId>
where
    MarketId: MaxEncodedLen,
{
    pub assets: Vec<Asset<MarketId>>,
    pub base_asset: Option<Asset<MarketId>>,
    pub market_id: MarketId,
    pub pool_status: PoolStatus,
    pub scoring_rule: ScoringRule,
    pub swap_fee: Option<Balance>,
    pub total_subsidy: Option<Balance>,
    pub total_weight: Option<u128>,
    pub weights: Option<BTreeMap<Asset<MarketId>, u128>>,
}

pub struct MigratePoolBaseAsset<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for MigratePoolBaseAsset<T> {
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        if StorageVersion::get::<Pallet<T>>() != REQUIRED_STORAGE_VERSION {
            return 0;
        }
        let mut total_weight: Weight = 0;
        log::info!(
            "Starting migration of Pool::base_asset from Optional<Asset<MarketId>> to \
             Asset<MarketId>"
        );

        for (key, value) in
            storage_iter::<Option<PoolDeprecated<BalanceOf<T>, T::MarketId>>>(SWAPS, POOLS)
        {
            if let Some(old_pool) = value {
                let new_pool = Pool {
                    assets: old_pool.assets,
                    base_asset: old_pool.base_asset.unwrap_or(Asset::Ztg),
                    market_id: old_pool.market_id,
                    pool_status: old_pool.pool_status,
                    scoring_rule: old_pool.scoring_rule,
                    swap_fee: old_pool.swap_fee,
                    total_subsidy: old_pool.total_subsidy,
                    total_weight: old_pool.total_weight,
                    weights: old_pool.weights,
                };
                put_storage_value::<Option<Pool<BalanceOf<T>, T::MarketId>>>(
                    SWAPS,
                    POOLS,
                    &key,
                    Some(new_pool),
                );
                total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
            }
        
        total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
        }

        StorageVersion::new(NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

        log::info!(
            "Completed migration of Pool::base_asset from Optional<Asset<MarketId>> to \
             Asset<MarketId>"
        );
        total_weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{ExtBuilder, Runtime};
    use core::fmt::Debug;
    use frame_support::{storage::migration::get_storage_value, Blake2_128Concat, StorageHasher};
    use parity_scale_codec::Encode;
    use zeitgeist_primitives::types::{PoolId, ScalarPosition};

    type Balance = BalanceOf<Runtime>;
    type MarketId = <Runtime as Config>::MarketId;

    #[test]
    fn test_on_runtime_upgrade() {
        ExtBuilder::default().build().execute_with(|| {
            let (old_pools, expected_pools) = create_test_data();
            populate_test_data::<Blake2_128Concat, PoolId, Option<PoolDeprecated<Balance, MarketId>>>(
                SWAPS, POOLS, old_pools,
            );
            MigratePoolBaseAsset::<Runtime>::on_runtime_upgrade();
            for (key, pool_expected) in expected_pools.iter().enumerate() {
                let storage_hash = key_to_hash::<Blake2_128Concat, PoolId>(key);
                let pool_actual = get_storage_value::<Option<Pool<Balance, MarketId>>>(
                    SWAPS,
                    POOLS,
                    &storage_hash,
                )
                .unwrap()
                .unwrap(); // Unwrap twice because get_storage returns an option!
                assert_eq!(pool_actual, *pool_expected);
            }
        });
    }

    fn populate_test_data<H, K, V>(pallet: &[u8], prefix: &[u8], data: Vec<V>)
    where
        H: StorageHasher,
        K: TryFrom<usize> + Encode,
        V: Encode + Clone,
        <K as TryFrom<usize>>::Error: Debug,
    {
        for (key, value) in data.iter().enumerate() {
            let storage_hash = key_to_hash::<H, K>(key);
            put_storage_value::<V>(pallet, prefix, &storage_hash, (*value).clone());
        }
    }

    fn key_to_hash<H, K>(key: usize) -> Vec<u8>
    where
        H: StorageHasher,
        K: TryFrom<usize> + Encode,
        <K as TryFrom<usize>>::Error: Debug,
    {
        K::try_from(key).unwrap().using_encoded(H::hash).as_ref().to_vec()
    }

    fn create_test_data()
    -> (Vec<Option<PoolDeprecated<Balance, MarketId>>>, Vec<Pool<Balance, MarketId>>) {
        let mut weights = BTreeMap::new();
        weights.insert(Asset::ScalarOutcome(1, ScalarPosition::Long), 567);
        weights.insert(Asset::ScalarOutcome(1, ScalarPosition::Short), 678);
        let old_pools: Vec<Option<PoolDeprecated<Balance, MarketId>>> = vec![
            Some(PoolDeprecated {
                assets: vec![Asset::CategoricalOutcome(0, 0), Asset::CategoricalOutcome(0, 1)],
                base_asset: None,
                market_id: 0,
                pool_status: PoolStatus::Active,
                scoring_rule: ScoringRule::CPMM,
                swap_fee: None,
                total_subsidy: None,
                total_weight: None,
                weights: None,
            }),
            Some(PoolDeprecated {
                assets: vec![
                    Asset::ScalarOutcome(1, ScalarPosition::Long),
                    Asset::ScalarOutcome(1, ScalarPosition::Short),
                ],
                base_asset: Some(Asset::CombinatorialOutcome),
                market_id: 123,
                pool_status: PoolStatus::Stale,
                scoring_rule: ScoringRule::RikiddoSigmoidFeeMarketEma,
                swap_fee: Some(234),
                total_subsidy: Some(345),
                total_weight: Some(456),
                weights: Some(weights.clone()),
            }),
        ];
        let expected_pools: Vec<Pool<Balance, MarketId>> = vec![
            Pool {
                assets: vec![Asset::CategoricalOutcome(0, 0), Asset::CategoricalOutcome(0, 1)],
                base_asset: Asset::Ztg,
                market_id: 0,
                pool_status: PoolStatus::Active,
                scoring_rule: ScoringRule::CPMM,
                swap_fee: None,
                total_subsidy: None,
                total_weight: None,
                weights: None,
            },
            Pool {
                assets: vec![
                    Asset::ScalarOutcome(1, ScalarPosition::Long),
                    Asset::ScalarOutcome(1, ScalarPosition::Short),
                ],
                base_asset: Asset::CombinatorialOutcome,
                market_id: 123,
                pool_status: PoolStatus::Stale,
                scoring_rule: ScoringRule::RikiddoSigmoidFeeMarketEma,
                swap_fee: Some(234),
                total_subsidy: Some(345),
                total_weight: Some(456),
                weights: Some(weights.clone()),
            },
        ];
        (old_pools, expected_pools)
    }
}
