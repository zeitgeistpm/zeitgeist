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
    Currency, ReservableCurrency, Get,
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{DispatchResult, DispatchError, ModuleId, RuntimeDebug, SaturatedConversion};
use sp_runtime::traits::{AccountIdConversion, Hash, Zero};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::convert::TryInto;
use sp_std::vec::Vec;
use zrml_traits::shares::{ReservableShares, Shares};
use zrml_traits::swaps::Swaps;

mod consts;
mod fixed;
mod math;

use fixed::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Pool<Balance, Hash> {
    pub assets: Vec<Hash>,
    pub swap_fee: Balance,
    pub total_weight: u128,
    pub weights: BTreeMap<Hash, u128>,
}

impl<Balance, Hash: Ord> Pool<Balance, Hash> {
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
    type MinWeight: Get<u128>;
    type MaxWeight: Get<u128>;
    type MaxTotalWeight: Get<u128>;
    type MaxAssets: Get<u128>;

    /// The minimum amount of liqudity required to bootstrap a pool.
    type MinLiquidity: Get<BalanceOf<Self>>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Swaps {
        Pools get(fn pools): map hasher(blake2_128_concat) u128 => Option<Pool<BalanceOf<T>, T::Hash>>;
        NextPoolId get(fn next_pool_id): u128;
    }
}

decl_event! {
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// A new pool has been created. [pool_id, creator]
        PoolCreated(u128, AccountId),
        /// Someone has joined a pool. [pool_id, who]
        JoinedPool(u128, AccountId),
        /// Someone has exited a pool. [pool_id, who]
        ExitedPool(u128, AccountId),
        /// A swap has occurred. [pool_id]
        Swap(u128),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        TooManyAssets,
        AssetNotBound,
        BelowMinimumWeight,
        AboveMaximumWeight,
        MaxTotalWeight,
        MathApproximation,
        MathApproximationDebug,
        LimitIn,
        LimitOut,
        PoolDoesNotExist,
        MaxInRatio,
        MaxOutRatio,
        BadLimitPrice,
        InsufficientBalance,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Temporary probably - The Swap is created per prediction market.
        #[weight = 0]
        fn create_pool(origin, assets: Vec<T::Hash>, weights: Vec<u128>) {
            let sender = ensure_signed(origin)?;
            
            let _result = Self::do_create_pool(sender, assets, Zero::zero(), weights).unwrap();
        }

        #[weight = 0]
        fn join_pool(origin, pool_id: u128, pool_amount_out: BalanceOf<T>, max_amounts_in: Vec<BalanceOf<T>>) {
            let sender = ensure_signed(origin)?;

            let pool_shares_id = Self::pool_shares_id(pool_id);
            let pool_shares_total = T::Shares::total_supply(pool_shares_id);
            let ratio: BalanceOf<T> = bdiv(pool_amount_out.saturated_into(), pool_shares_total.saturated_into()).saturated_into();
            ensure!(ratio != Zero::zero(), Error::<T>::MathApproximation);
         
            if let Some(pool) = Self::pools(pool_id) {
                let pool_account = Self::pool_account_id(pool_id);

                for i in 0..pool.assets.len() {
                    let asset = pool.assets[i];
                    let bal = T::Shares::free_balance(asset, &pool_account);
                    let asset_amount_in = bmul(ratio.saturated_into(), bal.saturated_into()).saturated_into();
                    ensure!(asset_amount_in != Zero::zero(), Error::<T>::MathApproximationDebug);
                    ensure!(asset_amount_in <= max_amounts_in[i], Error::<T>::LimitIn);

                    // transfer asset_amount_in to the pool_account
                    T::Shares::transfer(asset, &sender, &pool_account, asset_amount_in)?;
                }

                Self::mint_pool_shares(pool_id, &sender, pool_amount_out)?;

                Self::deposit_event(RawEvent::JoinedPool(pool_id, sender));
            } else {
                Err(Error::<T>::PoolDoesNotExist)?;
            }
        }

        #[weight = 0]
        fn exit_pool(origin, pool_id: u128, pool_amount_in: BalanceOf<T>, min_amounts_out: Vec<BalanceOf<T>>) {
            let sender = ensure_signed(origin)?;

            let pool_shares_id = Self::pool_shares_id(pool_id);
            let pool_shares_total = T::Shares::total_supply(pool_shares_id);
            let exit_fee = bmul(pool_amount_in.saturated_into(), T::ExitFee::get().saturated_into()).saturated_into();
            let pool_amount_in_after_exit_fee = pool_amount_in - exit_fee;
            let ratio: BalanceOf<T> = bdiv(pool_amount_in_after_exit_fee.saturated_into(), pool_shares_total.saturated_into()).saturated_into();
            ensure!(ratio != Zero::zero(), Error::<T>::MathApproximation);

            if let Some(pool) = Self::pools(pool_id) {
                let pool_account = Self::pool_account_id(pool_id);
                
                Self::burn_pool_shares(pool_id, &sender, pool_amount_in_after_exit_fee)?;
                // give the exit fee to the pool
                T::Shares::transfer(pool_shares_id, &sender, &pool_account, exit_fee)?;

                for i in 0..pool.assets.len() {
                    let asset = pool.assets[i];
                    let bal = T::Shares::free_balance(asset, &pool_account);
                    let asset_amount_out = bmul(ratio.saturated_into(), bal.saturated_into()).saturated_into();
                    ensure!(asset_amount_out != Zero::zero(), Error::<T>::MathApproximation);
                    ensure!(asset_amount_out >= min_amounts_out[i], Error::<T>::LimitOut);
                
                    T::Shares::transfer(asset, &pool_account, &sender, asset_amount_out)?;
                }

                Self::deposit_event(RawEvent::ExitedPool(pool_id, sender));
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
                    asset_amount_in <= bmul(in_balance.saturated_into(), T::MaxInRatio::get().saturated_into()).saturated_into(),
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
                    pool.swap_fee.saturated_into(),
                ).saturated_into();

                ensure!(asset_amount_out >= min_amount_out, Error::<T>::LimitOut);

                // do the swap
                T::Shares::transfer(asset_in, &sender, &pool_account, asset_amount_in)?;
                T::Shares::transfer(asset_out, &pool_account, &sender, asset_amount_out)?;

                let spot_price_after = Self::get_spot_price(pool_id, asset_in, asset_out);

                ensure!(spot_price_after >= spot_price_before, Error::<T>::MathApproximation);
                ensure!(spot_price_after <= max_price, Error::<T>::BadLimitPrice);
                ensure!(spot_price_before <= bdiv(asset_amount_in.saturated_into(), asset_amount_out.saturated_into()).saturated_into(), Error::<T>::MathApproximation);

                //todo emit an event
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
                ensure!(asset_amount_out <= bmul(out_balance.saturated_into(), T::MaxOutRatio::get().saturated_into()).saturated_into(), Error::<T>::MaxOutRatio);


                let spot_price_before = Self::get_spot_price(pool_id, asset_in, asset_out);

                ensure!(spot_price_before <= max_price, Error::<T>::BadLimitPrice);

                let in_balance = T::Shares::free_balance(asset_in, &pool_account);
                let asset_amount_in: BalanceOf<T> = math::calc_in_given_out(
                    in_balance.saturated_into(),
                    *pool.weights.get(&asset_in).unwrap(),
                    out_balance.saturated_into(),
                    *pool.weights.get(&asset_out).unwrap(),
                    asset_amount_out.saturated_into(),
                    pool.swap_fee.saturated_into(),
                ).saturated_into();

                ensure!(asset_amount_in <= max_amount_in, Error::<T>::LimitIn);

                // do the swap
                T::Shares::transfer(asset_in, &sender, &pool_account, asset_amount_in)?;
                T::Shares::transfer(asset_out, &pool_account, &sender, asset_amount_out)?;

                let spot_price_after = Self::get_spot_price(pool_id, asset_in, asset_out);

                ensure!(spot_price_after >= spot_price_before, Error::<T>::MathApproximation);
                ensure!(spot_price_after <= max_price, Error::<T>::BadLimitPrice);
                ensure!(spot_price_before <= bdiv(asset_amount_in.saturated_into(), asset_amount_out.saturated_into()).saturated_into(), Error::<T>::MathApproximation);

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
            let sender = ensure_signed(origin)?;

            if let Some(pool) = Self::pools(pool_id) {
                ensure!(pool.bound(asset_in), Error::<T>::AssetNotBound);

                let pool_account = Self::pool_account_id(pool_id);

                let in_balance = T::Shares::free_balance(asset_in, &pool_account);
                ensure!(asset_amount_in <= bmul(in_balance.saturated_into(), T::MaxInRatio::get().saturated_into()).saturated_into(), Error::<T>::MaxInRatio);

                let pool_shares_id = Self::pool_shares_id(pool_id);
                let total_supply = T::Shares::total_supply(pool_shares_id);

                let pool_amount_out: BalanceOf<T> = math::calc_pool_out_given_single_in(
                    in_balance.saturated_into(),
                    *pool.weights.get(&asset_in).unwrap(),
                    total_supply.saturated_into(),
                    pool.total_weight.saturated_into(),
                    asset_amount_in.saturated_into(),
                    pool.swap_fee.saturated_into(),
                ).saturated_into();

                ensure!(pool_amount_out >= min_pool_amount_out, Error::<T>::LimitOut);

                Self::mint_pool_shares(pool_id, &sender, pool_amount_out)?;
                T::Shares::transfer(asset_in, &sender, &pool_account, asset_amount_in)?;

                // emit an event
            } else {
                Err(Error::<T>::PoolDoesNotExist)?;
            }
        }

        #[weight = 0]
        fn joinswap_pool_amount_out(
            origin,
            pool_id: u128,
            asset_in: T::Hash,
            pool_amount_out: BalanceOf<T>,
            max_amount_in: BalanceOf<T>,
        ) {
            let sender = ensure_signed(origin)?;

            if let Some(pool) = Self::pools(pool_id) {
                ensure!(pool.bound(asset_in), Error::<T>::AssetNotBound);

                let pool_account = Self::pool_account_id(pool_id);
            
                let pool_shares_id = Self::pool_shares_id(pool_id);
                let total_supply = T::Shares::total_supply(pool_shares_id);

                let in_balance = T::Shares::free_balance(asset_in, &pool_account);

                let asset_amount_in: BalanceOf<T> = math::calc_single_in_given_pool_out(
                    in_balance.saturated_into(),
                    *pool.weights.get(&asset_in).unwrap(),
                    total_supply.saturated_into(),
                    pool.total_weight,
                    pool_amount_out.saturated_into(),
                    pool.swap_fee.saturated_into(),
                ).saturated_into();

                ensure!(asset_amount_in != Zero::zero(), Error::<T>::MathApproximation);
                ensure!(asset_amount_in <= max_amount_in, Error::<T>::LimitIn);

                ensure!(asset_amount_in <= bmul(in_balance.saturated_into(), T::MaxInRatio::get().saturated_into()).saturated_into(), Error::<T>::MaxInRatio);

                Self::mint_pool_shares(pool_id, &sender, pool_amount_out)?;
                T::Shares::transfer(asset_in, &sender, &pool_account, asset_amount_in)?;

                // emit an event
            } else {
                Err(Error::<T>::PoolDoesNotExist)?;
            }
        }

        #[weight = 0]
        fn exitswap_pool_amount_in(
            origin,
            pool_id: u128,
            asset_out: T::Hash,
            pool_amount_in: BalanceOf<T>,
            min_amount_out: BalanceOf<T>,
        ) {
            let sender = ensure_signed(origin)?;

            if let Some(pool) = Self::pools(pool_id) {
                ensure!(pool.bound(asset_out), Error::<T>::AssetNotBound);
                let pool_account = Self::pool_account_id(pool_id);

                let out_balance = T::Shares::free_balance(asset_out, &pool_account);
                let pool_shares_id = Self::pool_shares_id(pool_id);
                let total_supply = T::Shares::total_supply(pool_shares_id);

                let asset_amount_out: BalanceOf<T> = math::calc_single_out_given_pool_in(
                    out_balance.saturated_into(),
                    *pool.weights.get(&asset_out).unwrap(),
                    total_supply.saturated_into(),
                    pool.total_weight,
                    pool_amount_in.saturated_into(),
                    pool.swap_fee.saturated_into(),
                ).saturated_into();

                ensure!(asset_amount_out >= min_amount_out, Error::<T>::LimitOut);
                ensure!(asset_amount_out <= bmul(out_balance.saturated_into(), T::MaxOutRatio::get().saturated_into()).saturated_into(), Error::<T>::MaxOutRatio);

                let exit_fee = bmul(pool_amount_in.saturated_into(), T::ExitFee::get().saturated_into()).saturated_into();
                // todo handle exit_fee

                Self::burn_pool_shares(pool_id, &sender, pool_amount_in - exit_fee)?;
                T::Shares::transfer(asset_out, &pool_account, &sender, asset_amount_out)?;


                // emit an event
            } else {
                Err(Error::<T>::PoolDoesNotExist)?;
            }
        }

        #[weight = 0]
        fn exitswap_extern_amount_out(
            origin,
            pool_id: u128,
            asset_out: T::Hash,
            asset_amount_out: BalanceOf<T>,
            max_pool_amount_in: BalanceOf<T>,
        ) {
            let sender = ensure_signed(origin)?;

            if let Some(pool) = Self::pools(pool_id) {
                ensure!(pool.bound(asset_out), Error::<T>::AssetNotBound);

                let pool_account = Self::pool_account_id(pool_id);

                let out_balance = T::Shares::free_balance(asset_out, &pool_account);
                ensure!(asset_amount_out <= bmul(out_balance.saturated_into(), T::MaxOutRatio::get().saturated_into()).saturated_into(), Error::<T>::MaxOutRatio);
                
                let pool_shares_id = Self::pool_shares_id(pool_id);
                let total_supply = T::Shares::total_supply(pool_shares_id);

                let pool_amount_in: BalanceOf<T> = math::calc_pool_in_given_single_out(
                    out_balance.saturated_into(),
                    *pool.weights.get(&asset_out).unwrap(),
                    total_supply.saturated_into(),
                    pool.total_weight,
                    asset_amount_out.saturated_into(),
                    pool.swap_fee.saturated_into(),
                ).saturated_into();

                ensure!(pool_amount_in != Zero::zero(), Error::<T>::MathApproximation);
                ensure!(pool_amount_in <= max_pool_amount_in, Error::<T>::LimitIn);

                let exit_fee = bmul(pool_amount_in.saturated_into(), T::ExitFee::get().saturated_into()).saturated_into();

                Self::burn_pool_shares(pool_id, &sender, pool_amount_in - exit_fee)?;
                // todo do something with exit fee
                T::Shares::transfer(asset_out, &pool_account, &sender, asset_amount_out)?;

                // emit an event
            } else {
                Err(Error::<T>::PoolDoesNotExist)?;
            }
        }
    }
}

impl<T: Trait> Module<T> {
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

    pub fn pool_shares_id(pool_id: u128) -> T::Hash {
        ("zge/swaps", pool_id).using_encoded(<T as frame_system::Trait>::Hashing::hash)
    }

    fn pool_master_account() -> T::AccountId {
        T::ModuleId::get().into_account()
    }

    pub fn pool_account_id(pool_id: u128) -> T::AccountId {
        T::ModuleId::get().into_sub_account(pool_id)
    }

    fn inc_next_pool_id() -> u128 {
        let id = NextPoolId::get();
        NextPoolId::mutate(|n| *n += 1);
        id
    }
}

impl<T: Trait> Swaps<T::AccountId, BalanceOf<T>, T::Hash> for Module<T> {
    fn do_create_pool(creator: T::AccountId, assets: Vec<T::Hash>, swap_fee: BalanceOf<T>, weights: Vec<u128>) -> sp_std::result::Result<u128, DispatchError> {
        ensure!(assets.len() <= T::MaxAssets::get().try_into().unwrap(), Error::<T>::TooManyAssets);

        for i in 0..weights.len() {
            ensure!(weights[i] >= T::MinWeight::get(), Error::<T>::BelowMinimumWeight);
            ensure!(weights[i] <= T::MaxWeight::get(), Error::<T>::AboveMaximumWeight);
        }

        let amount = T::MinLiquidity::get();
        let next_pool_id = Self::inc_next_pool_id();
        let pool_account = Self::pool_account_id(next_pool_id);

        let mut map = BTreeMap::new();
        for i in 0..assets.len() {
            ensure!(T::Shares::free_balance(assets[i], &creator) >= amount, Error::<T>::InsufficientBalance);
            T::Shares::transfer(assets[i], &creator, &pool_account, amount)?;

            map.insert(assets[i], weights[i]);
        }

        let total_weight = weights.into_iter().fold(0, |acc, x| acc + x);
        ensure!(total_weight <= T::MaxTotalWeight::get(), Error::<T>::MaxTotalWeight);

        <Pools<T>>::insert(next_pool_id, Pool {
            assets,
            swap_fee,
            total_weight,
            weights: map,
        });

        let pool_shares_id = Self::pool_shares_id(next_pool_id);
        T::Shares::generate(pool_shares_id, &Self::pool_master_account(), amount)?;

        Self::deposit_event(RawEvent::PoolCreated(next_pool_id, creator));

        Ok(next_pool_id)
    }
}
