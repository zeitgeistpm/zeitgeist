// Copyright 2023-2025 Forecasting Technologies LTD.
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
    AssetOf, BalanceOf, Config, LiquidityTreeOf, MarketIdOf, MarketIdToPoolId, Pallet, PoolCount,
    Pools,
};
use alloc::{fmt::Debug, vec, vec::Vec};
use core::marker::PhantomData;
use frame_support::{
    migration::storage_key_iter,
    pallet_prelude::Twox64Concat,
    storage::bounded_btree_map::BoundedBTreeMap,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    weights::Weight,
    CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound,
};
use log;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{SaturatedConversion, Saturating};
use zeitgeist_primitives::math::checked_ops_res::CheckedAddRes;
use zrml_market_commons::MarketCommonsPalletApi;

cfg_if::cfg_if! {
    if #[cfg(feature = "try-runtime")] {
        use alloc::{format, collections::BTreeMap};
        use sp_runtime::DispatchError;
    }
}

const NEO_SWAPS: &[u8] = b"NeoSwaps";
const POOLS: &[u8] = b"Pools";

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

// https://substrate.stackexchange.com/questions/10472/pallet-storage-migration-fails-try-runtime-idempotent-test
// idempotent test fails, because of the manual storage version increment
// VersionedMigration is still an experimental feature for the currently used polkadot version
// that's why the idempotent test is ignored for this migration
pub struct MigratePoolStorageItems<T, RemovableMarketIds>(PhantomData<T>, RemovableMarketIds);

impl<T, RemovableMarketIds> OnRuntimeUpgrade for MigratePoolStorageItems<T, RemovableMarketIds>
where
    T: Config,
    RemovableMarketIds: Get<Vec<u32>>,
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
        // NeoSwaps: 7de9893ad4de67f3510fd09678a13412
        // Pools: 4c72016d74b63ae83d79b02efdb5528e
        // failed to decode pool with market id 880: 0x7de9893ad4de67f3510fd09678a134124c72016d74b63ae83d79b02efdb5528e00251b42e33e726f70030000000000000000000000000000
        // failed to decode pool with market id 878: 0x7de9893ad4de67f3510fd09678a134124c72016d74b63ae83d79b02efdb5528e0f7a0cea0db6ee406e030000000000000000000000000000
        // failed to decode pool with market id 882: 0x7de9893ad4de67f3510fd09678a134124c72016d74b63ae83d79b02efdb5528eb49736cf4bc6723372030000000000000000000000000000
        // failed to decode pool with market id 879: 0x7de9893ad4de67f3510fd09678a134124c72016d74b63ae83d79b02efdb5528ed857f1051e4281a76f030000000000000000000000000000
        // failed to decode pool with market id 877: 0x7de9893ad4de67f3510fd09678a134124c72016d74b63ae83d79b02efdb5528ee0edd4b43beb361f6d030000000000000000000000000000
        // The decode failure happens, because the old pool used a CampaignAsset as asset, which is not supported anymore, since the asset system overhaul has been reverted.

        let mut max_pool_id: T::PoolId = Default::default();
        for (market_id, _) in
            storage_key_iter::<MarketIdOf<T>, OldPoolOf<T>, Twox64Concat>(NEO_SWAPS, POOLS)
        {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(2));
            if T::MarketCommons::market(&market_id).is_err() {
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
        // Write for the PoolCount storage item
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("MigratePoolStorageItems: Upgraded {} pools.", translated);
        // Reads and writes for the Pools storage item
        total_weight =
            total_weight.saturating_add(T::DbWeight::get().reads_writes(translated, translated));
        // Read for the market and write for the MarketIdToPoolId storage item
        total_weight =
            total_weight.saturating_add(T::DbWeight::get().reads_writes(translated, translated));

        // remove pools that contain a corrupted campaign asset from the reverted asset system overhaul
        let mut corrupted_pools = vec![];
        for &market_id in RemovableMarketIds::get().iter() {
            let market_id = market_id.saturated_into::<MarketIdOf<T>>();
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(2));
            let is_corrupted =
                || Pools::<T>::contains_key(market_id) && Pools::<T>::get(market_id).is_none();
            if is_corrupted() {
                total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
                Pools::<T>::remove(market_id);
                corrupted_pools.push(market_id);
            } else {
                log::warn!(
                    "RemoveMarkets: Pool with market id {:?} was expected to be corrupted, but \
                     isn't.",
                    market_id
                );
            }
        }
        log::info!("RemovePools: Removed pools with market ids: {:?}.", corrupted_pools);
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
            for asset in &outcome_assets {
                assert!(new_pool.assets.contains(asset));
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
            "MigratePoolStorageItems: Post-upgrade next pool count id is {:?}!",
            next_pool_count_id
        );
        for &market_id in RemovableMarketIds::get().iter() {
            let market_id = market_id.saturated_into::<MarketIdOf<T>>();
            assert!(!Pools::<T>::contains_key(market_id));
            assert!(Pools::<T>::try_get(market_id).is_err());
        }
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

    struct RemovableMarketIds;
    impl Get<Vec<u32>> for RemovableMarketIds {
        fn get() -> Vec<u32> {
            vec![]
        }
    }

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            MigratePoolStorageItems::<Runtime, RemovableMarketIds>::on_runtime_upgrade();
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
            MigratePoolStorageItems::<Runtime, RemovableMarketIds>::on_runtime_upgrade();
            assert_eq!(tmp, storage_root(StateVersion::V1));
        });
    }

    #[test]
    fn on_runtime_upgrade_correctly_updates_pool_storages() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            create_markets(3);
            let (old_pools, new_pools) = construct_old_new_tuple();
            populate_test_data::<Twox64Concat, MarketIdOf<Runtime>, OldPoolOf<Runtime>>(
                NEO_SWAPS, POOLS, old_pools,
            );
            MigratePoolStorageItems::<Runtime, RemovableMarketIds>::on_runtime_upgrade();
            let actual = Pools::get(0u128).unwrap();
            assert_eq!(actual, new_pools[0]);
            let next_pool_count_id = PoolCount::<Runtime>::get();
            assert_eq!(next_pool_count_id, 3u128);
            assert_eq!(MarketIdToPoolId::<Runtime>::get(0u128).unwrap(), 0u128);
            assert_eq!(MarketIdToPoolId::<Runtime>::get(1u128).unwrap(), 1u128);
            assert_eq!(MarketIdToPoolId::<Runtime>::get(2u128).unwrap(), 2u128);
            assert!(MarketIdToPoolId::<Runtime>::get(3u128).is_none());
            assert!(MarketIdToPoolId::<Runtime>::iter_keys().count() == 3);
        });
    }

    fn set_up_version() {
        StorageVersion::new(NEO_SWAPS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
    }

    fn create_markets(count: u8) {
        for _ in 0..count {
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
            MarketCommons::push_market(market).unwrap();
        }
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
        (
            vec![old_pool.clone(), old_pool.clone(), old_pool.clone()],
            vec![new_pool.clone(), new_pool.clone(), new_pool.clone()],
        )
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
