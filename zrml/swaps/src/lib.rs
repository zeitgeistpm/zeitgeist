//! # Swaps
//!
//! A module to handle swapping shares out for different ones. Allows
//! liquidity providers to deposit full outcome shares and earn fees.

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
mod macros;

mod consts;
mod events;
mod fixed;
mod math;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use events::GenericPoolEvent;
use fixed::*;
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure};
use frame_support::traits::{Currency, Get, ReservableCurrency};
use frame_system::{self as system, ensure_signed};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{DispatchError, DispatchResult, ModuleId, RuntimeDebug, SaturatedConversion};
use sp_runtime::traits::{AccountIdConversion, Hash, Zero};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::convert::TryInto;
use sp_std::vec::Vec;
use zrml_traits::shares::{ReservableShares, Shares};
use zrml_traits::swaps::Swaps;

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

    type Shares: Shares<Self::AccountId, BalanceOf<Self>, Self::Hash>
        + ReservableShares<Self::AccountId, BalanceOf<Self>, Self::Hash>;

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
        AccountId = <T as frame_system::Trait>::AccountId
    {
        /// A new pool has been created. [pool_id, creator]
        PoolCreated(GenericPoolEvent<AccountId>),
        /// Someone has joined a pool. [pool_id, who]
        JoinedPool(GenericPoolEvent<AccountId>),
        /// Someone has exited a pool. [pool_id, who]
        ExitedPool(GenericPoolEvent<AccountId>),
        /// A swap has occurred. [pool_id]
        Swap(GenericPoolEvent<AccountId>),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        AboveMaximumWeight,
        AssetNotBound,
        BadLimitPrice,
        BelowMinimumWeight,
        InsufficientBalance,
        LimitIn,
        LimitOut,
        MathApproximation,
        MathApproximationDebug,
        MaxInRatio,
        MaxOutRatio,
        MaxTotalWeight,
        PoolDoesNotExist,
        ProvidedValuesLenMustEqualAssetsLen,
        TooManyAssets
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

            let _ = Self::do_create_pool(sender, assets, Zero::zero(), weights)?;
        }

        /// Joins a given set of assets provided from `origin` to `pool_id`.
        ///
        /// # Parameters
        ///
        /// * `origin`: Provider. The person whose assets should be transferred.
        /// * `pool_id`: Unique pool identifier.
        /// * `pool_amount`: Sum of all assets that are being transferred from the provider to the pool.
        /// * `max_assets_in`: List of asset upper bounds. No asset transfer should be greater
        /// than the provided values.
        #[weight = 0]
        fn join_pool(origin, pool_id: u128, pool_amount: BalanceOf<T>, max_assets_in: Vec<BalanceOf<T>>) {
            pool_in_and_out!(
                params: (max_assets_in, origin, pool_amount, pool_id),

                event: JoinedPool,
                transfer_asset: |amount, amount_bound, asset, pool_account, who| {
                    T::Shares::transfer(asset, who, pool_account, amount)?;
                    ensure!(amount <= amount_bound, Error::<T>::LimitIn);
                    Ok(())
                },
                transfer_pool: |_, _, who| Self::mint_pool_shares(pool_id, who, pool_amount)
            )
        }

        /// Retrieves a given set of assets from `pool_id` to `origin`.
        ///
        /// # Parameters
        ///
        /// * `origin`: Provider. The person whose assets should be received.
        /// * `pool_id`: Unique pool identifier.
        /// * `pool_amount`: Sum of all assets that are being transferred from the pool to the provider.
        /// * `min_assets_out`: List of asset lower bounds. No asset transfer should be lower
        /// than the provided values.
        #[weight = 0]
        fn exit_pool(origin, pool_id: u128, pool_amount: BalanceOf<T>, min_assets_out: Vec<BalanceOf<T>>) {
            let exit_fee_pct = T::ExitFee::get().saturated_into();
            let exit_fee = bmul(pool_amount.saturated_into(), exit_fee_pct).saturated_into();
            let pool_amount_minus_exit_fee = pool_amount - exit_fee;
            pool_in_and_out!(
                params: (min_assets_out, origin, pool_amount, pool_id),

                event: ExitedPool,
                transfer_asset: |amount, amount_bound, asset, pool_account, who| {
                    ensure!(amount >= amount_bound, Error::<T>::LimitOut);
                    T::Shares::transfer(asset, pool_account, who, amount)?;
                    Ok(())
                },
                transfer_pool: |pool_account_id, pool_shares_id, who| {
                    T::Shares::transfer(pool_shares_id, who, pool_account_id, exit_fee)?;
                    Self::burn_pool_shares(pool_id, who, pool_amount_minus_exit_fee)?;
                    Ok(())
                }
            )
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

            let pool = Self::pool_by_id(pool_id)?;

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
            let who = ensure_signed(origin)?;

            let pool = Self::pool_by_id(pool_id)?;

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
            T::Shares::transfer(asset_in, &who, &pool_account, asset_amount_in)?;
            T::Shares::transfer(asset_out, &pool_account, &who, asset_amount_out)?;

            let spot_price_after = Self::get_spot_price(pool_id, asset_in, asset_out);

            ensure!(spot_price_after >= spot_price_before, Error::<T>::MathApproximation);
            ensure!(spot_price_after <= max_price, Error::<T>::BadLimitPrice);
            ensure!(spot_price_before <= bdiv(asset_amount_in.saturated_into(), asset_amount_out.saturated_into()).saturated_into(), Error::<T>::MathApproximation);

            Self::deposit_event(RawEvent::Swap(GenericPoolEvent {
                pool_id,
                who
            }));
        }

        #[weight = 0]
        fn joinswap_extern_amount_in(
            origin,
            pool_id: u128,
            asset_in: T::Hash,
            asset_amount_in: BalanceOf<T>,
            min_pool_amount_out: BalanceOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            let pool = Self::pool_by_id(pool_id)?;

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

            Self::mint_pool_shares(pool_id, &who, pool_amount_out)?;
            T::Shares::transfer(asset_in, &who, &pool_account, asset_amount_in)?;

            Self::deposit_event(RawEvent::Swap(GenericPoolEvent {
                pool_id,
                who
            }));
        }

        #[weight = 0]
        fn joinswap_pool_amount_out(
            origin,
            pool_id: u128,
            asset_in: T::Hash,
            pool_amount_out: BalanceOf<T>,
            max_amount_in: BalanceOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            let pool = Self::pool_by_id(pool_id)?;

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

            Self::mint_pool_shares(pool_id, &who, pool_amount_out)?;
            T::Shares::transfer(asset_in, &who, &pool_account, asset_amount_in)?;

            Self::deposit_event(RawEvent::Swap(GenericPoolEvent {
                pool_id,
                who
            }));
        }

        #[weight = 0]
        fn exitswap_pool_amount_in(
            origin,
            pool_id: u128,
            asset_out: T::Hash,
            pool_amount_in: BalanceOf<T>,
            min_amount_out: BalanceOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            let pool = Self::pool_by_id(pool_id)?;

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

            Self::burn_pool_shares(pool_id, &who, pool_amount_in - exit_fee)?;
            T::Shares::transfer(asset_out, &pool_account, &who, asset_amount_out)?;

            Self::deposit_event(RawEvent::Swap(GenericPoolEvent {
                pool_id,
                who
            }));
        }

        #[weight = 0]
        fn exitswap_extern_amount_out(
            origin,
            pool_id: u128,
            asset_out: T::Hash,
            asset_amount_out: BalanceOf<T>,
            max_pool_amount_in: BalanceOf<T>,
        ) {
            let who = ensure_signed(origin)?;

            let pool = Self::pool_by_id(pool_id)?;

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

            Self::burn_pool_shares(pool_id, &who, pool_amount_in - exit_fee)?;
            // todo do something with exit fee
            T::Shares::transfer(asset_out, &pool_account, &who, asset_amount_out)?;

            Self::deposit_event(RawEvent::Swap(GenericPoolEvent {
                pool_id,
                who
            }));
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn pool_shares_id(pool_id: u128) -> T::Hash {
        ("zge/swaps", pool_id).using_encoded(<T as frame_system::Trait>::Hashing::hash)
    }

    pub fn pool_account_id(pool_id: u128) -> T::AccountId {
        T::ModuleId::get().into_sub_account(pool_id)
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
                0, //fee
            )
            .saturated_into();
        } else {
            // Err(Error::<T>::PoolDoesNotExist)?;
            return Zero::zero();
        }
    }

    fn mint_pool_shares(pool_id: u128, to: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
        let shares_id = Self::pool_shares_id(pool_id);
        T::Shares::generate(shares_id, to, amount)
    }

    fn burn_pool_shares(
        pool_id: u128,
        from: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> DispatchResult {
        let shares_id = Self::pool_shares_id(pool_id);
        T::Shares::destroy(shares_id, from, amount)
    }

    fn pool_master_account() -> T::AccountId {
        T::ModuleId::get().into_account()
    }

    fn inc_next_pool_id() -> u128 {
        let id = NextPoolId::get();
        NextPoolId::mutate(|n| *n += 1);
        id
    }

    fn get_denormalized_weight(pool_id: u128, asset: T::Hash) -> u128 {
        if let Some(pool) = Self::pools(pool_id) {
            if let Some(val) = pool.weights.get(&asset) {
                *val
            } else {
                0
            }
        } else {
            0
        }
    }

    fn get_normalized_weight(_pool_id: u128, _asset: T::Hash) -> u128 {
        // unimplemented
        0
    }

    fn pool_by_id(
        pool_id: u128
    ) -> Result<Pool<BalanceOf<T>, T::Hash>, Error<T>>
    where
        T: Trait
    {
        Self::pools(pool_id).ok_or(Error::<T>::PoolDoesNotExist.into())
    }
}

impl<T: Trait> Swaps<T::AccountId, BalanceOf<T>, T::Hash> for Module<T> {
    /// Deploys a new pool with the given assets and weights.
    ///
    /// ## Arguments
    ///
    /// - `creator` - The account that is the creator of the pool. Must have enough
    ///               funds for each of the assets to cover the `MinLiqudity`.
    /// - `assets` - The assets that are used in the pool.
    /// - `swap_fee` - The fee applied to each swap.
    /// - `weights` - These are the denormalized weights (the raw weights).
    fn do_create_pool(
        who: T::AccountId,
        assets: Vec<T::Hash>,
        swap_fee: BalanceOf<T>,
        weights: Vec<u128>,
    ) -> sp_std::result::Result<u128, DispatchError> {
        check_provided_values_len_must_equal_assets_len::<T, _>(&assets, &weights)?;

        ensure!(
            assets.len() <= T::MaxAssets::get().try_into().unwrap(),
            Error::<T>::TooManyAssets
        );

        for weight in weights.iter().copied() {
            ensure!(weight >= T::MinWeight::get(), Error::<T>::BelowMinimumWeight);
            ensure!(weight <= T::MaxWeight::get(), Error::<T>::AboveMaximumWeight);
        }

        let amount = T::MinLiquidity::get();
        let next_pool_id = Self::inc_next_pool_id();
        let pool_account = Self::pool_account_id(next_pool_id);

        let mut map = BTreeMap::new();
        for (asset, weight) in assets.iter().copied().zip(weights.iter().copied()) {
            ensure!(
                T::Shares::free_balance(asset, &who) >= amount,
                Error::<T>::InsufficientBalance
            );
            T::Shares::transfer(asset, &who, &pool_account, amount)?;

            map.insert(asset, weight);
        }

        let total_weight = weights.into_iter().fold(0, |acc, x| acc + x);
        ensure!(total_weight <= T::MaxTotalWeight::get(), Error::<T>::MaxTotalWeight);

        <Pools<T>>::insert(
            next_pool_id,
            Pool {
                assets,
                swap_fee,
                total_weight,
                weights: map,
            },
        );

        let pool_shares_id = Self::pool_shares_id(next_pool_id);
        T::Shares::generate(pool_shares_id, &who, amount)?;

        Self::deposit_event(RawEvent::PoolCreated(GenericPoolEvent {
            pool_id: next_pool_id,
            who
        }));

        Ok(next_pool_id)
    }
}

fn check_provided_values_len_must_equal_assets_len<T, U>(
    assets: &[T::Hash],
    provided_values: &[U]
) -> Result<(), Error<T>>
where
    T: Trait
{
    if assets.len() != provided_values.len() {
        return Err(Error::<T>::ProvidedValuesLenMustEqualAssetsLen.into());
    }
    Ok(())
}
