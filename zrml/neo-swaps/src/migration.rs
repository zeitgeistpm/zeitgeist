// Copyright 2023 Forecasting Technologies LTD.
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
    liquidity_tree::types::LiquidityTree,
    types::{Pool, SoloLp},
    Config, Pallet,
};
use frame_support::{
    dispatch::Weight,
    log,
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
};
use sp_runtime::traits::Saturating;

cfg_if::cfg_if! {
    if #[cfg(feature = "try-runtime")] {
        use crate::{MarketIdOf, Pools};
        use alloc::{collections::BTreeMap, format, vec::Vec};
        use frame_support::{migration::storage_key_iter, pallet_prelude::Twox64Concat};
        use parity_scale_codec::{Decode, Encode};
        use sp_runtime::traits::Zero;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(feature = "try-runtime", test))] {
        const NEO_SWAPS: &[u8] = b"NeoSwaps";
        const POOLS: &[u8] = b"Pools";
    }
}

const NEO_SWAPS_REQUIRED_STORAGE_VERSION: u16 = 0;
const NEO_SWAPS_NEXT_STORAGE_VERSION: u16 = NEO_SWAPS_REQUIRED_STORAGE_VERSION + 1;

type OldPoolOf<T> = Pool<T, SoloLp<T>>;

pub struct MigrateToLiquidityTree<T>(PhantomData<T>);

impl<T: Config + zrml_market_commons::Config> OnRuntimeUpgrade for MigrateToLiquidityTree<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let market_commons_version = StorageVersion::get::<Pallet<T>>();
        if market_commons_version != NEO_SWAPS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "MigrateToLiquidityTree: market-commons version is {:?}, but {:?} is required",
                market_commons_version,
                NEO_SWAPS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("MigrateToLiquidityTree: Starting...");
        let mut translated = 0u64;
        crate::Pools::<T>::translate::<OldPoolOf<T>, _>(|_, pool| {
            let solo = pool.liquidity_shares_manager;
            // This should never fail; if it does, then we just delete the entry.
            let mut liquidity_tree =
                LiquidityTree::new(solo.owner.clone(), solo.total_shares).ok()?;
            liquidity_tree.nodes.get_mut(0)?.fees = solo.fees; // Can't fail.
            translated.saturating_inc();
            Some(Pool {
                account_id: pool.account_id,
                reserves: pool.reserves,
                collateral: pool.collateral,
                liquidity_parameter: pool.liquidity_parameter,
                liquidity_shares_manager: liquidity_tree,
                swap_fee: pool.swap_fee,
            })
        });
        log::info!("MigrateToLiquidityTree: Upgraded {} pools.", translated);
        total_weight =
            total_weight.saturating_add(T::DbWeight::get().reads_writes(translated, translated));
        StorageVersion::new(NEO_SWAPS_NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("MigrateToLiquidityTree: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        let old_pools =
            storage_key_iter::<MarketIdOf<T>, OldPoolOf<T>, Twox64Concat>(NEO_SWAPS, POOLS)
                .collect::<BTreeMap<_, _>>();
        Ok(old_pools.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(previous_state: Vec<u8>) -> Result<(), &'static str> {
        let old_pools: BTreeMap<MarketIdOf<T>, OldPoolOf<T>> =
            Decode::decode(&mut &previous_state[..])
                .map_err(|_| "Failed to decode state: Invalid state")?;
        let new_pool_count = Pools::<T>::iter().count();
        assert_eq!(old_pools.len(), new_pool_count);
        for (market_id, new_pool) in Pools::<T>::iter() {
            let old_pool =
                old_pools.get(&market_id).expect(&format!("Pool {:?} not found", market_id)[..]);
            assert_eq!(new_pool.account_id, old_pool.account_id);
            assert_eq!(new_pool.reserves, old_pool.reserves);
            assert_eq!(new_pool.collateral, old_pool.collateral);
            assert_eq!(new_pool.liquidity_parameter, old_pool.liquidity_parameter);
            assert_eq!(new_pool.swap_fee, old_pool.swap_fee);
            let tree = new_pool.liquidity_shares_manager;
            let solo = &old_pool.liquidity_shares_manager;
            assert_eq!(tree.nodes.len(), 1);
            assert_eq!(tree.abandoned_nodes.len(), 0);
            assert_eq!(tree.account_to_index.len(), 1);
            let root = tree.nodes[0].clone();
            let account = root.account.clone();
            assert_eq!(root.account, Some(solo.owner.clone()));
            assert_eq!(root.stake, solo.total_shares);
            assert_eq!(root.fees, solo.fees);
            assert_eq!(root.descendant_stake, Zero::zero());
            assert_eq!(root.lazy_fees, Zero::zero());
            let address = account.unwrap();
            assert_eq!(tree.account_to_index.get(&address), Some(&0));
        }
        log::info!("MigrateToLiquidityTree: Post-upgrade pool count is {}!", new_pool_count);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock::{ExtBuilder, Runtime},
        MarketIdOf, PoolOf, Pools,
    };
    use alloc::collections::BTreeMap;
    use frame_support::{
        dispatch::fmt::Debug, migration::put_storage_value, storage_root, StateVersion,
        StorageHasher, Twox64Concat,
    };
    use parity_scale_codec::Encode;
    use zeitgeist_primitives::types::Asset;

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            MigrateToLiquidityTree::<Runtime>::on_runtime_upgrade();
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
            MigrateToLiquidityTree::<Runtime>::on_runtime_upgrade();
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
            MigrateToLiquidityTree::<Runtime>::on_runtime_upgrade();
            let actual = Pools::get(0u128).unwrap();
            assert_eq!(actual, new_pools[0]);
        });
    }

    fn set_up_version() {
        StorageVersion::new(NEO_SWAPS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
    }

    fn construct_old_new_tuple() -> (Vec<OldPoolOf<Runtime>>, Vec<PoolOf<Runtime>>) {
        let account_id = 1;
        let mut reserves = BTreeMap::new();
        reserves.insert(Asset::CategoricalOutcome(2, 3), 4);
        let collateral = Asset::Ztg;
        let liquidity_parameter = 5;
        let swap_fee = 6;
        let total_shares = 7;
        let fees = 8;

        let solo = SoloLp { owner: account_id, total_shares, fees };
        let mut liquidity_tree = LiquidityTree::new(account_id, total_shares).unwrap();
        liquidity_tree.nodes.get_mut(0).unwrap().fees = fees;

        let old_pool = OldPoolOf {
            account_id,
            reserves: reserves.clone(),
            collateral,
            liquidity_parameter,
            liquidity_shares_manager: solo,
            swap_fee,
        };
        let new_pool = Pool {
            account_id,
            reserves,
            collateral,
            liquidity_parameter,
            liquidity_shares_manager: liquidity_tree,
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
