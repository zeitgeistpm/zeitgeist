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
    traits::LiquiditySharesManager, types::Pool, AssetOf, BalanceOf, Config, LiquidityTreeOf,
    Pallet, Pools,
};
use alloc::collections::BTreeMap;
use core::marker::PhantomData;
use frame_support::{
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    weights::Weight,
};
use log;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{RuntimeDebug, Saturating};

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

const NEO_SWAPS_REQUIRED_STORAGE_VERSION: u16 = 1;
const NEO_SWAPS_NEXT_STORAGE_VERSION: u16 = NEO_SWAPS_REQUIRED_STORAGE_VERSION + 1;

#[derive(Clone, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct OldPool<T, LSM>
where
    T: Config,
    LSM: LiquiditySharesManager<T>,
{
    pub account_id: T::AccountId,
    pub reserves: BTreeMap<AssetOf<T>, BalanceOf<T>>,
    pub collateral: AssetOf<T>,
    pub liquidity_parameter: BalanceOf<T>,
    pub liquidity_shares_manager: LSM,
    pub swap_fee: BalanceOf<T>,
}

type OldPoolOf<T> = OldPool<T, LiquidityTreeOf<T>>;

pub struct MigratePoolReservesToBoundedBTreeMap<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for MigratePoolReservesToBoundedBTreeMap<T>
where
    T: Config,
{
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let neo_swaps_version = StorageVersion::get::<Pallet<T>>();
        if neo_swaps_version != NEO_SWAPS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "MigratePoolReservesToBoundedBTreeMap: neo-swaps version is {:?}, but {:?} is \
                 required",
                neo_swaps_version,
                NEO_SWAPS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("MigratePoolReservesToBoundedBTreeMap: Starting...");
        let mut translated = 0u64;
        Pools::<T>::translate::<OldPoolOf<T>, _>(|_, pool| {
            // Can't fail unless `MaxAssets` is misconfigured. If it fails after all, we delete the
            // pool. This may seem drastic, but is actually cleaner than trying some half-baked
            // recovery and allows us to do a manual recovery of funds.
            let reserves = pool.reserves.try_into().ok()?;
            translated.saturating_inc();
            Some(Pool {
                account_id: pool.account_id,
                reserves,
                collateral: pool.collateral,
                liquidity_parameter: pool.liquidity_parameter,
                liquidity_shares_manager: pool.liquidity_shares_manager,
                swap_fee: pool.swap_fee,
            })
        });
        log::info!("MigratePoolReservesToBoundedBTreeMap: Upgraded {} pools.", translated);
        total_weight =
            total_weight.saturating_add(T::DbWeight::get().reads_writes(translated, translated));
        StorageVersion::new(NEO_SWAPS_NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("MigratePoolReservesToBoundedBTreeMap: Done!");
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
        for (market_id, new_pool) in Pools::<T>::iter() {
            let old_pool =
                old_pools.get(&market_id).expect(&format!("Pool {:?} not found", market_id)[..]);
            assert_eq!(new_pool.account_id, old_pool.account_id);
            assert_eq!(new_pool.reserves.into_inner(), old_pool.reserves);
            assert_eq!(new_pool.collateral, old_pool.collateral);
            assert_eq!(new_pool.liquidity_parameter, old_pool.liquidity_parameter);
            assert_eq!(new_pool.liquidity_shares_manager, old_pool.liquidity_shares_manager);
            assert_eq!(new_pool.swap_fee, old_pool.swap_fee);
        }
        log::info!(
            "MigratePoolReservesToBoundedBTreeMap: Post-upgrade pool count is {}!",
            new_pool_count
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        liquidity_tree::types::LiquidityTree,
        mock::{ExtBuilder, Runtime},
        MarketIdOf, PoolOf, Pools,
    };
    use alloc::collections::BTreeMap;
    use core::fmt::Debug;
    use frame_support::{migration::put_storage_value, StorageHasher, Twox64Concat};
    use parity_scale_codec::Encode;
    use sp_io::storage::root as storage_root;
    use sp_runtime::StateVersion;
    use zeitgeist_primitives::types::Asset;

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            MigratePoolReservesToBoundedBTreeMap::<Runtime>::on_runtime_upgrade();
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
            MigratePoolReservesToBoundedBTreeMap::<Runtime>::on_runtime_upgrade();
            assert_eq!(tmp, storage_root(StateVersion::V1));
        });
    }

    #[test]
    fn on_runtime_upgrade_correctly_updates_markets() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            let (old_pools, new_pools) = construct_old_new_tuple();
            populate_test_data::<Twox64Concat, MarketIdOf<Runtime>, OldPoolOf<Runtime>>(
                NEO_SWAPS, POOLS, old_pools,
            );
            MigratePoolReservesToBoundedBTreeMap::<Runtime>::on_runtime_upgrade();
            let actual = Pools::get(0u128).unwrap();
            assert_eq!(actual, new_pools[0]);
        });
    }

    fn set_up_version() {
        StorageVersion::new(NEO_SWAPS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
    }

    fn construct_old_new_tuple() -> (Vec<OldPoolOf<Runtime>>, Vec<PoolOf<Runtime>>) {
        let account_id = 1;
        let mut old_reserves = BTreeMap::new();
        old_reserves.insert(Asset::CategoricalOutcome(2, 3), 4);
        let new_reserves = old_reserves.clone().try_into().unwrap();
        let collateral = Asset::Ztg;
        let liquidity_parameter = 5;
        let swap_fee = 6;
        let total_shares = 7;
        let fees = 8;

        let mut liquidity_shares_manager = LiquidityTree::new(account_id, total_shares).unwrap();
        liquidity_shares_manager.nodes.get_mut(0).unwrap().fees = fees;

        let old_pool = OldPoolOf {
            account_id,
            reserves: old_reserves,
            collateral,
            liquidity_parameter,
            liquidity_shares_manager: liquidity_shares_manager.clone(),
            swap_fee,
        };
        let new_pool = Pool {
            account_id,
            reserves: new_reserves,
            collateral,
            liquidity_parameter,
            liquidity_shares_manager,
            swap_fee,
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
