//! # Swaps
//!
//! A module to handle swapping shares out for different ones. Allows
//! liquidity providers to deposit full outcome shares and earn fees.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, 
    ensure,
};
use frame_support::traits::{
    Currency, ReservableCurrency, ExistenceRequirement, WithdrawReasons, Get,
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{ModuleId, RuntimeDebug, SaturatedConversion};
use sp_runtime::traits::{
    AccountIdConversion, CheckedSub, CheckedMul, Hash, Zero,
};
use sp_std::cmp;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::vec::Vec;
use zrml_traits::shares::{ReservableShares, Shares};

mod consts;
mod math;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Pool<Hash> {
    pub assets: Vec<Hash>,
    pub weights: BTreeMap<Hash, u128>,
}

// impl<Hash> Pool<Hash> {
//     pub fn bound(&self, asset: Hash) -> bool {
//         let weight = self.weights.get(&asset);
//         weight.is_some()
//     }
// }

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: ReservableCurrency<Self::AccountId>;

    type Shares: Shares<Self::AccountId, BalanceOf<Self>, Self::Hash> + ReservableShares<Self::AccountId, BalanceOf<Self>, Self::Hash>;

    /// The module identifier.
    type ModuleId: Get<ModuleId>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Swaps {
        Pools get(fn pools): map hasher(blake2_128_concat) u128 => Option<Pool<T::Hash>>;
        NextPoolId: u128;
    }
}

decl_event! {
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        Something(AccountId),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        TooManyAssets,
        AssetNotBound,
        BelowMinimumWeight,
        AboveMaximumWeight,
        MathApproximation,
        LimitIn,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 0]
        fn join_pool(origin, pool_id: u128, pool_amount_out: u128, max_amounts_in: Vec<u128>) {

            let pool_shares_id = Self::pool_shares_id(pool_id);
            let pool_shares_total = T::Shares::total_supply(pool_shares_id);
            let ratio = pool_amount_out / pool_shares_total;
            ensure!(ratio != 0, Error::<T>::MathApproximation)?;
         
            if let Some(pool) = Self::pools(pool_id) {
                let pool_account = Self::pool_account_id(pool_id);

                for i in pool.assets.len() {
                    let asset = pool.assets[i];
                    let bal = T::Shares::free_balance(asset, &pool_account_id);
                    let asset_amount_in = ratio * bal;
                    ensure!(asset_amount_in != 0, Error::<T>::MathApproximation)?;
                    ensure!(asset_amount_in <= max_amounts_in[i], Error::<T>::LimitIn)?;


                }


            } else {
                Err(Error::<T>::PoolDoesNotExist)?;
            }
        }

        #[weight = 0]
        fn exit_pool(origin, pool_id: u128, pool_amount_in: u128, min_amounts_out: Vec<u128>) {

        }

        #[weight = 0]
        fn swap_exact_amount_in(
            origin,
            pool_id: u128,
            asset_in: T::Hash,
            asset_amount_in: u128,
            asset_out: T::Hash,
            min_amount_out: u128,
            max_price: u128,
        ) {

        }

        #[weight = 0]
        fn swap_exact_amount_out(
            origin,
            pool_id: u128,
            asset_in: T::Hash,
            max_amount_in: u128,
            asset_out: T::Hash,
            asset_amount_out: u128,
            max_price: u128,
        ) {

        }

        #[weight = 0]
        fn joinswap_extern_amount_in(
            origin,
            pool_id: u128,
            asset_in: T::Hash,
            asset_amount_in: u128,
            min_pool_amount_out: u128,
        ) {

        }

        #[weight = 0]
        fn joinswap_pool_amount_out(
            origin,
            pool_id: u128,
            asset_in: T::Hash,
            pool_amount_out: u128,
            max_amount_in: u128,
        ) {

        }

        #[weight = 0]
        fn exitswap_pool_amount_in(
            origin,
            pool_id: u128,
            asset_out: T::Hash,
            pool_amount_in: u128,
            min_amount_out: u128,
        ) {

        }

        #[weight = 0]
        fn exitswap_extern_amount_out(
            origin,
            pool_id: u128,
            asset_out: T::Hash,
            asset_amount_out: u128,
            max_pool_amount_in: u128,
        ) {

        }
    }
}

impl<T: Trait> Module<T> {
    pub fn create_pool(assets: Vec<T::Hash>, weights: Vec<u128>) {
        // ensure!(assets.len() <= T::MaxAssets::get(), Error::<T>::TooManyAssets)?;

        for i in 0..weights.len() {
            // ensure!(weights[i] >= T::MinWeight, Error::<T>::BelowMinimumWeight)?;
            // ensure!(weights[i] <= T::MaxWeight, Error::<T>::AboveMaximumWeight)?;
        }

        let next_pool_id = Self::next_pool_id();

        let mut map = BTreeMap::new();
        for i in 0..assets.len() {
            map.insert(assets[i], weights[i]);
        }

        <Pools<T>>::insert(next_pool_id, Pool {
            assets,
            weights: map,
        });
    }

    pub fn get_spot_price(pool_id: u128, asset_in: T::Hash, asset_out: T::Hash) -> u128 {
        if let Some(pool) = Self::pools(pool_id) {
            // ensure!(pool.bound(asset_in), Error::<T>::AssetNotBound)?;
            // ensure!(pool.bound(asset_out), Error::<T>::AssetNotBound)?;

            let pool_account = Self::pool_account_id(pool_id);
            let in_balance = T::Shares::free_balance(asset_in, &pool_account);
            let in_weight = pool.weights.get(&asset_in).unwrap();
            let out_balance = T::Shares::free_balance(asset_out, &pool_account);
            let out_weight = pool.weights.get(&asset_out).unwrap();

            return math::calc_spot_price(
                in_balance.saturated_into(),
                *in_weight,
                out_balance.saturated_into(),
                *out_weight,
                0 //fee
            );
        } else {
            // Err(Error::<T>::PoolDoesNotExist)?;
            return 0;
        }
    }

    fn pool_account_id(pool_id: u128) -> T::AccountId {
        T::ModuleId::get().into_sub_account(pool_id)
    }

    fn next_pool_id() -> u128 {
        let id = NextPoolId::get();
        NextPoolId::mutate(|n| *n += 1);
        id
    }
}
