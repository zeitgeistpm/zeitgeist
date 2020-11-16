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
    Currency, ReservableCurrency, ExistenceRequirement, WithdrawReasons,
};
// use frame_support::weights::Weight;
use frame_system::{self as system, ensure_signed};
use sp_runtime::RuntimeDebug;
use sp_runtime::traits::{
    CheckedSub, CheckedMul, Hash, Zero,
};
use sp_std::cmp;
use sp_std::collection::btree_map::BTreeMap;
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
    pub weights: BTreeMap<Hash, u128>;
}

impl<Hash> Pool<Hash> {
    pub fn bound(self, asset: Hash) -> bool {
        let weight = self.weights.get(asset);
        weight.is_some()
    }
}

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: ReservableCurrency<Self::AccountId>;

    type Shares: Shares<Self::AccountId, BalanceOf<Self>, Self::Hash> + ReservableShares<Self::AccountId, BalanceOf<Self>, Self::Hash>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Swaps {
        Pools get(fn pools): map hasher(blake2_128_concat) u128 => Pool;
        PoolToAssets get(fn pool_to_assets): map hasher(blake2_128_concat) T::Hash => Vec<T::Hash>;
    }
}

decl_event! {
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        Something(AccountId).
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        SomeError,
    }
}

decl_module! {
    pub struct Module<T: trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;


    }
}

impl<T: Trait> Module<T> {
    pub fn create_pool(assets: Vec<T::Hash>, weights: Vec<u128>) {
        ensure!(assets.len() <= T::MaxAssets::get(), Error::<T>::TooManyAssets)?;

        for i in 0..weights.len() {
            ensure!(weights[i] >= T::MinWeight, Error::<T>::BelowMinimumWeight)?;
            ensure!(weights[i] <= T::MaxWeight, Error::<T>::AboveMaximumWeight)?;
        }

        let next_pool_id = Self::next_pool_id();

        Pool::insert(next_pool_id, Pool {
            assets,
            weights,
        });
    }

    pub fn get_spot_price(pool_id: u128, asset_in: T::Hash, asset_out: T::Hash) -> u128 {
        if let pool = Self::pools(pool_id) {
            ensure!(pool.bound(asset_in), Error::<T>::AssetNotBound)?;
            ensure!(pool.bound(asset_out), Error::<T>::AssetNotBound)?;

            let pool_account = Self::pool_acount_id(pool_id);
            let in_balance = T::Shares::free_balance(asset_in, &pool_account);
            let in_weight = pool.weights.get(asset_in).unwrap();
            let out_balance = T::Shares::free_balance(asset_out, &pool_account);
            let out_weight = pool.weights.get(asset_out).unwrap();

            return math::calc_spot_price(
                in_balance,
                in_weight,
                out_balance,
                out_weight,
            );
        } else {
            Err(Error::<T>::PoolDoesNotExist)?;
        }
    }

    fn next_pool_id() -> u128 {
        let id = NextPoolId::get();
        NextPoolId::mutate(|n| *n += 1);
        id
    }
}
