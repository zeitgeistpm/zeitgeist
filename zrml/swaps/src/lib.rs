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
use sp_runtime::{DispatchResult, ModuleId, RuntimeDebug, SaturatedConversion};
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
    pub swap_fee: u128,
    pub weights: BTreeMap<Hash, u128>,
}

impl<Hash: Ord> Pool<Hash> {
    pub fn bound(&self, asset: Hash) -> bool {
        let weight = BTreeMap::get(&self.weights, &asset);
        weight.is_some()
    }
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: ReservableCurrency<Self::AccountId>;

    type Shares: Shares<Self::AccountId, BalanceOf<Self>, Self::Hash> + ReservableShares<Self::AccountId, BalanceOf<Self>, Self::Hash>;

    /// The module identifier.
    type ModuleId: Get<ModuleId>;

    // The fee for exiting a pool.
    type ExitFee: Get<BalanceOf<Self>>;

    type MaxInRatio: Get<BalanceOf<Self>>;

    type MaxOutRatio: Get<BalanceOf<Self>>;
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
        LimitOut,
        PoolDoesNotExist,
        MaxInRatio,
        MaxOutRatio,
        BadLimitPrice,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 0]
        fn join_pool(origin, pool_id: u128, pool_amount_out: BalanceOf<T>, max_amounts_in: Vec<BalanceOf<T>>) {
            let sender = ensure_signed(origin)?;

            let pool_shares_id = Self::pool_shares_id(pool_id);
            let pool_shares_total = T::Shares::total_supply(pool_shares_id);
            let ratio = pool_amount_out / pool_shares_total;
            ensure!(ratio != Zero::zero(), Error::<T>::MathApproximation);
         
            if let Some(pool) = Self::pools(pool_id) {
                let pool_account = Self::pool_account_id(pool_id);

                for i in 0..pool.assets.len() {
                    let asset = pool.assets[i];
                    let bal = T::Shares::free_balance(asset, &pool_account);
                    let asset_amount_in = ratio * bal;
                    ensure!(asset_amount_in != Zero::zero(), Error::<T>::MathApproximation);
                    ensure!(asset_amount_in <= max_amounts_in[i], Error::<T>::LimitIn);

                    // transfer asset_amount_in to the pool_account
                    T::Shares::transfer(asset, &sender, &pool_account, asset_amount_in)?;
                }

                Self::mint_pool_shares(pool_id, &sender, pool_amount_out)?;
                //emit event
            } else {
                Err(Error::<T>::PoolDoesNotExist)?;
            }
        }

        #[weight = 0]
        fn exit_pool(origin, pool_id: u128, pool_amount_in: BalanceOf<T>, min_amounts_out: Vec<BalanceOf<T>>) {
            let sender = ensure_signed(origin)?;

            let pool_shares_id = Self::pool_shares_id(pool_id);
            let pool_shares_total = T::Shares::total_supply(pool_shares_id);
            let exit_fee = pool_amount_in * T::ExitFee::get();
            let pool_amount_in_after_exit_fee = pool_amount_in - exit_fee;
            let ratio = pool_amount_in_after_exit_fee / pool_shares_total;
            ensure!(ratio != Zero::zero(), Error::<T>::MathApproximation);

            if let Some(pool) = Self::pools(pool_id) {
                let pool_account = Self::pool_account_id(pool_id);
                
                Self::burn_pool_shares(pool_id, &sender, pool_amount_in_after_exit_fee)?;
                // give the exit fee to the pool
                T::Shares::transfer(pool_shares_id, &sender, &pool_account, exit_fee)?;

                for i in 0..pool.assets.len() {
                    let asset = pool.assets[0];
                    let bal = T::Shares::free_balance(asset, &pool_account);
                    let asset_amount_out = ratio * bal;
                    ensure!(asset_amount_out != Zero::zero(), Error::<T>::MathApproximation);
                    ensure!(asset_amount_out >= min_amounts_out[i], Error::<T>::LimitOut);
                
                    T::Shares::transfer(asset, &pool_account, &sender, asset_amount_out)?;
                }

                //emit event
            } else {
                Err(Error::<T>::PoolDoesNotExist)?;
            }

        }

        #[weight = 0]
        fn swap_exact_amount_in(
            origin,
            pool_id: u128,
            asset_in: T::Hash,
            asset_amount_in: BalanceOf<T>,
            asset_out: T::Hash,
            min_amount_out: BalanceOf<T>,
            max_price: BalanceOf<T>,
        ) {
            let sender = ensure_signed(origin)?;
            
            if let Some(pool) = Self::pools(pool_id) {
                ensure!(pool.bound(asset_in), Error::<T>::AssetNotBound);
                ensure!(pool.bound(asset_out), Error::<T>::AssetNotBound);

                let pool_account = Self::pool_account_id(pool_id);
                let in_balance = T::Shares::free_balance(asset_in, &pool_account);
                ensure!(
                    asset_amount_in <= in_balance * T::MaxInRatio::get(),
                    Error::<T>::MaxInRatio,
                );

                let spot_price_before = Self::get_spot_price(pool_id, asset_in, asset_out);

                ensure!(spot_price_before <= max_price, Error::<T>::BadLimitPrice);

                let out_balance = T::Shares::free_balance(asset_out, &pool_account);

                let asset_amount_out: BalanceOf<T> = math::calc_out_given_in(
                    in_balance.saturated_into(),
                    *pool.weights.get(&asset_in).unwrap(),
                    out_balance.saturated_into(),
                    *pool.weights.get(&asset_out).unwrap(),
                    asset_amount_in.saturated_into(),
                    pool.swap_fee,
                ).saturated_into();

                ensure!(asset_amount_out >= min_amount_out, Error::<T>::LimitOut);

                // do the swap
                T::Shares::transfer(asset_in, &sender, &pool_account, asset_amount_in)?;
                T::Shares::transfer(asset_out, &pool_account, &sender, asset_amount_out)?;

                let spot_price_after = Self::get_spot_price(pool_id, asset_in, asset_out);

                ensure!(spot_price_after >= spot_price_before, Error::<T>::MathApproximation);
                ensure!(spot_price_after <= max_price, Error::<T>::BadLimitPrice);
                ensure!(spot_price_before <= asset_amount_in / asset_amount_out, Error::<T>::MathApproximation);

                // emit an event
            } else {
                Err(Error::<T>::PoolDoesNotExist)?;
            }
        }

        #[weight = 0]
        fn swap_exact_amount_out(
            origin,
            pool_id: u128,
            asset_in: T::Hash,
            max_amount_in: BalanceOf<T>,
            asset_out: T::Hash,
            asset_amount_out: BalanceOf<T>,
            max_price: BalanceOf<T>,
        ) {
            let sender = ensure_signed(origin)?;
            
            if let Some(pool) = Self::pools(pool_id) {
                ensure!(pool.bound(asset_in), Error::<T>::AssetNotBound);
                ensure!(pool.bound(asset_out), Error::<T>::AssetNotBound);

                let pool_account = Self::pool_account_id(pool_id);
                let out_balance = T::Shares::free_balance(asset_out, &pool_account);
                ensure!(asset_amount_out <= out_balance * T::MaxOutRatio::get(), Error::<T>::MaxOutRatio);


                let spot_price_before = Self::get_spot_price(pool_id, asset_in, asset_out);

                ensure!(spot_price_before <= max_price, Error::<T>::BadLimitPrice);

                let in_balance = T::Shares::free_balance(asset_in, &pool_account);
                let asset_amount_in: BalanceOf<T> = math::calc_in_given_out(
                    in_balance.saturated_into(),
                    *pool.weights.get(&asset_in).unwrap(),
                    out_balance.saturated_into(),
                    *pool.weights.get(&asset_out).unwrap(),
                    asset_amount_out.saturated_into(),
                    pool.swap_fee,
                ).saturated_into();

                ensure!(asset_amount_in <= max_amount_in, Error::<T>::LimitIn);

                // do the swap
                T::Shares::transfer(asset_in, &sender, &pool_account, asset_amount_in)?;
                T::Shares::transfer(asset_out, &pool_account, &sender, asset_amount_out)?;

                let spot_price_after = Self::get_spot_price(pool_id, asset_in, asset_out);

                ensure!(spot_price_after >= spot_price_before, Error::<T>::MathApproximation);
                ensure!(spot_price_after <= max_price, Error::<T>::BadLimitPrice);
                ensure!(spot_price_before <= asset_amount_in / asset_amount_out, Error::<T>::MathApproximation);

                // emit an event
            } else {
                Err(Error::<T>::PoolDoesNotExist)?;
            }
        }

        #[weight = 0]
        fn joinswap_extern_amount_in(
            origin,
            pool_id: u128,
            asset_in: T::Hash,
            asset_amount_in: BalanceOf<T>,
            min_pool_amount_out: BalanceOf<T>,
        ) {

        }

        #[weight = 0]
        fn joinswap_pool_amount_out(
            origin,
            pool_id: u128,
            asset_in: T::Hash,
            pool_amount_out: BalanceOf<T>,
            max_amount_in: BalanceOf<T>,
        ) {

        }

        #[weight = 0]
        fn exitswap_pool_amount_in(
            origin,
            pool_id: u128,
            asset_out: T::Hash,
            pool_amount_in: BalanceOf<T>,
            min_amount_out: BalanceOf<T>,
        ) {

        }

        #[weight = 0]
        fn exitswap_extern_amount_out(
            origin,
            pool_id: u128,
            asset_out: T::Hash,
            asset_amount_out: BalanceOf<T>,
            max_pool_amount_in: BalanceOf<T>,
        ) {

        }
    }
}

impl<T: Trait> Module<T> {
    pub fn create_pool(assets: Vec<T::Hash>, swap_fee: u128, weights: Vec<u128>) {
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
            swap_fee,
            weights: map,
        });
    }

    pub fn get_spot_price(pool_id: u128, asset_in: T::Hash, asset_out: T::Hash) -> BalanceOf<T> {
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
            ).saturated_into();
        } else {
            // Err(Error::<T>::PoolDoesNotExist)?;
            return Zero::zero();
        }
    }

    fn mint_pool_shares(pool_id: u128, to: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        let shares_id = Self::pool_shares_id(pool_id);
        T::Shares::generate(shares_id, to, amount)
    }

    fn burn_pool_shares(pool_id: u128, from: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        let shares_id = Self::pool_shares_id(pool_id);
        T::Shares::destroy(shares_id, from, amount)
    }

    fn pool_shares_id(pool_id: u128) -> T::Hash {
        ("zge/swaps", pool_id).using_encoded(<T as frame_system::Trait>::Hashing::hash)
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
