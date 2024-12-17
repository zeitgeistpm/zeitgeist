// Copyright 2023-2024 Forecasting Technologies LTD.
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
    traits::LiquiditySharesManager,
    types::{MaxAssets, Pool, PoolType},
    AssetOf, BalanceOf, Config, LiquidityTreeOf, MarketIdToPoolId, Pallet, PoolCount, Pools,
};
use alloc::fmt::Debug;
use core::marker::PhantomData;
use frame_support::{
    storage::bounded_btree_map::BoundedBTreeMap,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    weights::Weight,
    CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
use log;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::Saturating;
use zeitgeist_primitives::math::checked_ops_res::CheckedAddRes;
use zrml_market_commons::MarketCommonsPalletApi;

cfg_if::cfg_if! {
    if #[cfg(feature = "try-runtime")] {
        use crate::{MarketIdOf};
        use alloc::{format, vec::Vec};
        use frame_support::{migration::storage_key_iter, pallet_prelude::Twox64Concat};
        use sp_runtime::DispatchError;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(feature = "try-runtime", test))] {
        const NEO_SWAPS: &[u8] = b"NeoSwaps";
        const POOLS: &[u8] = b"Pools";
    }
}

const NEO_SWAPS_REQUIRED_STORAGE_VERSION: u16 = 2;
const NEO_SWAPS_NEXT_STORAGE_VERSION: u16 = NEO_SWAPS_REQUIRED_STORAGE_VERSION + 1;

#[derive(
    CloneNoBound, Decode, Encode, Eq, MaxEncodedLen, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[scale_info(skip_type_params(S, T))]
pub struct OldPool<T, LSM, S>
where
    T: Config,
    LSM: Clone + Debug + LiquiditySharesManager<T> + PartialEq,
    S: Get<u32>,
{
    pub account_id: T::AccountId,
    pub reserves: BoundedBTreeMap<AssetOf<T>, BalanceOf<T>, S>,
    pub collateral: AssetOf<T>,
    pub liquidity_parameter: BalanceOf<T>,
    pub liquidity_shares_manager: LSM,
    pub swap_fee: BalanceOf<T>,
}

type OldPoolOf<T> = OldPool<T, LiquidityTreeOf<T>, MaxAssets>;

pub struct MigratePoolStorageItems<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for MigratePoolStorageItems<T>
where
    T: Config,
{
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let neo_swaps_version = StorageVersion::get::<Pallet<T>>();
        if neo_swaps_version != NEO_SWAPS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "MigratePoolStorageItems: neo-swaps version is {:?}, but {:?} is required",
                neo_swaps_version,
                NEO_SWAPS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("MigratePoolStorageItems: Starting...");
        let mut max_pool_id: T::PoolId = Default::default();
        for market_id in Pools::<T>::iter_keys() {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(2));
            if let Err(_) = T::MarketCommons::market(&market_id) {
                log::error!("MigratePoolStorageItems: Market {:?} not found", market_id);
                return total_weight;
            };
            let pool_id = market_id;
            max_pool_id = max_pool_id.max(pool_id);
        }
        let next_pool_count_id = if let Ok(id) = max_pool_id.checked_add_res(&1u8.into()) {
            id
        } else {
            log::error!("MigratePoolStorageItems: Pool id overflow");
            return total_weight;
        };
        let mut translated = 0u64;
        Pools::<T>::translate::<OldPoolOf<T>, _>(|market_id, pool| {
            translated.saturating_inc();
            let pool_id = market_id;
            MarketIdToPoolId::<T>::insert(pool_id, market_id);
            let assets = if let Ok(market) = T::MarketCommons::market(&market_id) {
                market.outcome_assets().try_into().ok()?
            } else {
                log::error!(
                    "MigratePoolStorageItems: Market {:?} not found. This should not happen, \
                     because it is checked above.",
                    market_id
                );
                pool.reserves.keys().cloned().collect::<Vec<_>>().try_into().ok()?
            };
            Some(Pool {
                account_id: pool.account_id,
                assets,
                reserves: pool.reserves,
                collateral: pool.collateral,
                liquidity_parameter: pool.liquidity_parameter,
                liquidity_shares_manager: pool.liquidity_shares_manager,
                swap_fee: pool.swap_fee,
                pool_type: PoolType::Standard(market_id),
            })
        });
        PoolCount::<T>::set(next_pool_count_id);
        log::info!("MigratePoolStorageItems: Upgraded {} pools.", translated);
        total_weight =
            total_weight.saturating_add(T::DbWeight::get().reads_writes(translated, translated));
        StorageVersion::new(NEO_SWAPS_NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("MigratePoolStorageItems: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, DispatchError> {
        let old_pools =
            storage_key_iter::<MarketIdOf<T>, OldPoolOf<T>, Twox64Concat>(NEO_SWAPS, POOLS)
                .collect::<BTreeMap<_, _>>();
        Ok(old_pools.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(previous_state: Vec<u8>) -> Result<(), DispatchError> {
        let old_pools: BTreeMap<MarketIdOf<T>, OldPoolOf<T>> =
            Decode::decode(&mut &previous_state[..])
                .map_err(|_| "Failed to decode state: Invalid state")?;
        let new_pool_count = Pools::<T>::iter().count();
        assert_eq!(old_pools.len(), new_pool_count);
        let mut max_pool_id: T::PoolId = Default::default();
        for (market_id, new_pool) in Pools::<T>::iter() {
            let old_pool =
                old_pools.get(&market_id).expect(&format!("Pool {:?} not found", market_id)[..]);
            max_pool_id = max_pool_id.max(market_id);
            assert_eq!(new_pool.account_id, old_pool.account_id);
            let market = T::MarketCommons::market(&market_id)?;
            let outcome_assets = market.outcome_assets();
            for asset in outcome_assets {
                assert!(new_pool.assets.contains(&asset));
            }
            assert_eq!(new_pool.assets.len(), outcome_assets.len());
            assert_eq!(new_pool.reserves, old_pool.reserves);
            assert_eq!(new_pool.collateral, old_pool.collateral);
            assert_eq!(new_pool.liquidity_parameter, old_pool.liquidity_parameter);
            assert_eq!(new_pool.liquidity_shares_manager, old_pool.liquidity_shares_manager);
            assert_eq!(new_pool.swap_fee, old_pool.swap_fee);
            assert_eq!(new_pool.pool_type, PoolType::Standard(market_id));

            assert_eq!(
                MarketIdToPoolId::<T>::get(market_id).expect("MarketIdToPoolId mapping not found"),
                market_id
            );
        }
        let next_pool_count_id = PoolCount::<T>::get();
        assert_eq!(next_pool_count_id, max_pool_id.checked_add_res(&1u8.into())?);
        log::info!(
            "MigratePoolStorageItems: Post-upgrade next pool count id is {}!",
            next_pool_count_id
        );
        log::info!("MigratePoolStorageItems: Post-upgrade pool count is {}!", new_pool_count);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        liquidity_tree::types::LiquidityTree,
        mock::{ExtBuilder, MarketCommons, Runtime, ALICE, BOB},
        MarketIdOf, PoolOf, Pools,
    };
    use alloc::collections::BTreeMap;
    use core::fmt::Debug;
    use frame_support::{migration::put_storage_value, StorageHasher, Twox64Concat};
    use parity_scale_codec::Encode;
    use sp_io::storage::root as storage_root;
    use sp_runtime::{Perbill, StateVersion};
    use zeitgeist_primitives::types::{
        Asset, Market, MarketCreation, MarketPeriod, MarketStatus, MarketType, ScoringRule,
    };

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            MigratePoolStorageItems::<Runtime>::on_runtime_upgrade();
            assert_eq!(StorageVersion::get::<Pallet<Runtime>>(), NEO_SWAPS_NEXT_STORAGE_VERSION);
        });
    }

    #[test]
    fn on_runtime_upgrade_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            StorageVersion::new(NEO_SWAPS_NEXT_STORAGE_VERSION).put::<Pallet<Runtime>>();
            let (_, new_pools) = construct_old_new_tuple();
            populate_test_data::<Twox64Concat, MarketIdOf<Runtime>, PoolOf<Runtime>>(
                NEO_SWAPS, POOLS, new_pools,
            );
            let tmp = storage_root(StateVersion::V1);
            MigratePoolStorageItems::<Runtime>::on_runtime_upgrade();
            assert_eq!(tmp, storage_root(StateVersion::V1));
        });
    }

    #[test]
    fn on_runtime_upgrade_correctly_updates_pools() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            let market_id = create_market();
            let (old_pools, new_pools) = construct_old_new_tuple();
            populate_test_data::<Twox64Concat, MarketIdOf<Runtime>, OldPoolOf<Runtime>>(
                NEO_SWAPS, POOLS, old_pools,
            );
            MigratePoolStorageItems::<Runtime>::on_runtime_upgrade();
            let actual = Pools::get(0u128).unwrap();
            assert_eq!(market_id, 0u128);
            assert_eq!(actual, new_pools[0]);
        });
    }

    fn set_up_version() {
        StorageVersion::new(NEO_SWAPS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
    }

    fn create_market() -> MarketIdOf<Runtime> {
        let base_asset = Asset::Ztg;
        let market = Market {
            market_id: 0u8.into(),
            base_asset,
            creation: MarketCreation::Permissionless,
            creator_fee: Perbill::zero(),
            creator: ALICE,
            oracle: BOB,
            metadata: vec![0, 50],
            market_type: MarketType::Categorical(3),
            period: MarketPeriod::Block(0u32.into()..1u32.into()),
            deadlines: Default::default(),
            scoring_rule: ScoringRule::AmmCdaHybrid,
            status: MarketStatus::Active,
            report: None,
            resolved_outcome: None,
            dispute_mechanism: None,
            bonds: Default::default(),
            early_close: None,
        };
        MarketCommons::push_market(market).unwrap()
    }

    fn construct_old_new_tuple() -> (Vec<OldPoolOf<Runtime>>, Vec<PoolOf<Runtime>>) {
        let account_id = 1;
        let mut reserves = BTreeMap::new();
        let asset_0 = Asset::CategoricalOutcome(0, 0);
        let asset_1 = Asset::CategoricalOutcome(0, 1);
        let asset_2 = Asset::CategoricalOutcome(0, 2);
        reserves.insert(asset_0, 4);
        reserves.insert(asset_1, 5);
        reserves.insert(asset_2, 6);
        let reserves: BoundedBTreeMap<AssetOf<Runtime>, BalanceOf<Runtime>, MaxAssets> =
            reserves.clone().try_into().unwrap();
        let collateral = Asset::Ztg;
        let liquidity_parameter = 5;
        let swap_fee = 6;
        let total_shares = 7;
        let fees = 8;

        let mut liquidity_shares_manager = LiquidityTree::new(account_id, total_shares).unwrap();
        liquidity_shares_manager.nodes.get_mut(0).unwrap().fees = fees;

        let old_pool = OldPoolOf {
            account_id,
            reserves: reserves.clone(),
            collateral,
            liquidity_parameter,
            liquidity_shares_manager: liquidity_shares_manager.clone(),
            swap_fee,
        };
        let new_pool = Pool {
            account_id,
            assets: vec![asset_0, asset_1, asset_2].try_into().unwrap(),
            reserves,
            collateral,
            liquidity_parameter,
            liquidity_shares_manager,
            swap_fee,
            pool_type: PoolType::Standard(0),
        };
        (vec![old_pool], vec![new_pool])
    }

    #[allow(unused)]
    fn populate_test_data<H, K, V>(pallet: &[u8], prefix: &[u8], data: Vec<V>)
    where
        H: StorageHasher,
        K: TryFrom<usize> + Encode,
        V: Encode + Clone,
        <K as TryFrom<usize>>::Error: Debug,
    {
        for (key, value) in data.iter().enumerate() {
            let storage_hash = utility::key_to_hash::<H, K>(K::try_from(key).unwrap());
            put_storage_value::<V>(pallet, prefix, &storage_hash, (*value).clone());
        }
    }
}

mod utility {
    use alloc::vec::Vec;
    use frame_support::StorageHasher;
    use parity_scale_codec::Encode;

    #[allow(unused)]
    pub fn key_to_hash<H, K>(key: K) -> Vec<u8>
    where
        H: StorageHasher,
        K: Encode,
    {
        key.using_encoded(H::hash).as_ref().to_vec()
    }
}
