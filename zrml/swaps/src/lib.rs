//! # Swaps
//!
//! A module to handle swapping shares out for different ones. Allows
//! liquidity providers to deposit full outcome shares and earn fees.

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
mod macros;

mod bpow;
mod check_arithm_rslt;
mod events;
mod math;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use bpow::BPow;
use check_arithm_rslt::CheckArithmRslt;
use events::{CommonPoolEventParams, PoolAssetEvent, PoolAssetsEvent, SwapEvent};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{Get, ReservableCurrency},
};
use frame_system::ensure_signed;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{
    traits::{AccountIdConversion, Hash},
    DispatchError, DispatchResult, FixedPointNumber, FixedU128, ModuleId, RuntimeDebug,
};
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
use zrml_traits::{
    shares::{ReservableShares, Shares},
    swaps::Swaps,
};

pub const BASE: u128 = <FixedU128 as FixedPointNumber>::DIV;
pub const EXIT_FEE: FixedU128 = FixedU128::from_inner(0);

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Pool<B, Hash> {
    pub assets: Vec<Hash>,
    pub swap_fee: B,
    pub total_weight: B,
    pub weights: BTreeMap<Hash, B>,
}

impl<B, Hash: Ord> Pool<B, Hash> {
    pub fn bound(&self, asset: Hash) -> bool {
        let weight = BTreeMap::get(&self.weights, &asset);
        weight.is_some()
    }
}

pub trait Trait: frame_system::Trait {
    type Currency: ReservableCurrency<Self::AccountId>;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    // The fee for exiting a pool.
    type ExitFee: Get<FixedU128>;
    type MaxAssets: Get<usize>;
    type MaxInRatio: Get<FixedU128>;
    type MaxOutRatio: Get<FixedU128>;
    type MaxTotalWeight: Get<FixedU128>;
    type MaxWeight: Get<FixedU128>;
    // The minimum amount of liqudity required to bootstrap a pool.
    type MinLiquidity: Get<FixedU128>;
    type MinWeight: Get<FixedU128>;
    // The module identifier.
    type ModuleId: Get<ModuleId>;
    type Shares: ReservableShares<Self::AccountId, Self::Hash> + Shares<Self::AccountId, Self::Hash>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Swaps {
        Pools get(fn pools): map hasher(blake2_128_concat) u128 => Option<Pool<FixedU128, T::Hash>>;
        NextPoolId get(fn next_pool_id): u128;
    }
}

decl_event! {
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        /// A new pool has been created.
        PoolCreate(CommonPoolEventParams<AccountId>),
        /// Someone has exited a pool.
        PoolExit(PoolAssetsEvent<AccountId, FixedU128>),
        /// Exists a pool given an exact amount of an asset
        PoolExitWithExactAssetAmount(PoolAssetEvent<AccountId, FixedU128>),
        /// Exists a pool given an exact pool's amount
        PoolExitWithExactPoolAmount(PoolAssetEvent<AccountId, FixedU128>),
        /// Someone has joined a pool.
        PoolJoin(PoolAssetsEvent<AccountId, FixedU128>),
        /// Joins a pool given an exact amount of an asset
        PoolJoinWithExactAssetAmount(PoolAssetEvent<AccountId, FixedU128>),
        /// Joins a pool given an exact pool's amount
        PoolJoinWithExactPoolAmount(PoolAssetEvent<AccountId, FixedU128>),
        /// An exact amount of an asset is entering the pool
        SwapExactAmountIn(SwapEvent<AccountId, FixedU128>),
        /// An exact amount of an asset is leaving the pool
        SwapExactAmountOut(SwapEvent<AccountId, FixedU128>),
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
        fn create_pool(origin, assets: Vec<T::Hash>, weights: Vec<FixedU128>) {
            let who = ensure_signed(origin)?;
            let _ = Self::do_create_pool(who, assets, FixedU128::zero(), weights)?;
        }

        /// Pool - Exit
        ///
        /// Retrieves a given set of assets from `pool_id` to `origin`.
        ///
        /// # Arguments
        ///
        /// * `origin`: Liquidity Provider (LP). The account whose assets should be received.
        /// * `pool_id`: Unique pool identifier.
        /// * `pool_amount`: The amount of LP shares of this pool being burned based on the
        /// retrieved assets.
        /// * `min_assets_out`: List of asset lower bounds. No asset should be lower than the
        /// provided values.
        #[weight = 0]
        fn pool_exit(origin, pool_id: u128, pool_amount: FixedU128, min_assets_out: Vec<FixedU128>) {
            pool!(
                initial_params: (min_assets_out, origin, pool_amount, pool_id),

                event: PoolExit,
                transfer_asset: |amount, amount_bound, asset, pool_account, who| {
                    ensure!(amount >= amount_bound, Error::<T>::LimitOut);
                    T::Shares::transfer(asset, pool_account, who, amount)?;
                    Ok(())
                },
                transfer_pool: |pool_account_id, pool_shares_id, who| {
                    let exit_fee = pool_amount.check_mul_rslt(&T::ExitFee::get())?;
                    let pool_amount_minus_exit_fee = pool_amount.check_sub_rslt(&exit_fee)?;
                    T::Shares::transfer(pool_shares_id, who, pool_account_id, exit_fee)?;
                    Self::burn_pool_shares(pool_id, who, pool_amount_minus_exit_fee)?;
                    Ok(())
                }
            )
        }

        /// Pool - Exit with exact pool amount
        ///
        /// Takes an asset from `pool_id` and transfers to `origin`. Differently from `pool_exit`,
        /// this method injects the exactly amount of `asset_amount_out` to `origin`.
        ///
        /// # Arguments
        ///
        /// * `origin`: Liquidity Provider (LP). The account whose assets should be received.
        /// * `pool_id`: Unique pool identifier.
        /// * `asset`: Asset leaving the pool.
        /// * `asset_amount_out`: Asset amount that is leaving the pool.
        /// * `max_pool_amount`: The calculated amount of assets for the pool must the equal or
        /// greater than the given value.
        #[weight = 0]
        fn pool_exit_with_exact_asset_amount(
            origin,
            pool_id: u128,
            asset: T::Hash,
            asset_amount: FixedU128,
            max_pool_amount: FixedU128,
        ) {
            pool_exit_with_exact_amount!(
                initial_params: (origin, pool_id, asset),

                asset_amount: |_, _, _| Ok(asset_amount),
                bound: max_pool_amount,
                ensure_balance: |pool_balance: FixedU128| {
                    ensure!(
                        asset_amount <= pool_balance.check_mul_rslt(&T::MaxOutRatio::get())?,
                        Error::<T>::MaxOutRatio
                    );
                    Ok(())
                },
                event: PoolExitWithExactPoolAmount,
                pool_amount: |pool: &Pool<FixedU128, _>, pool_balance: FixedU128, total_supply: FixedU128| {
                    let pool_amount: FixedU128 = math::calc_pool_in_given_single_out(
                        pool_balance,
                        *pool.weights.get(&asset).unwrap(),
                        total_supply,
                        pool.total_weight,
                        asset_amount,
                        pool.swap_fee,
                    )?;
                    ensure!(pool_amount != FixedU128::zero(), Error::<T>::MathApproximation);
                    ensure!(pool_amount <= max_pool_amount, Error::<T>::LimitIn);
                    Ok(pool_amount)
                }
            )
        }

        /// Pool - Exit with exact pool amount
        ///
        /// Takes an asset from `pool_id` and transfers to `origin`. Differently from `pool_exit`,
        /// this method injects the exactly amount of `pool_amount` to `pool_id`.
        ///
        /// # Arguments
        ///
        /// * `origin`: Liquidity Provider (LP). The account whose assets should be received.
        /// * `pool_id`: Unique pool identifier.
        /// * `asset`: Asset leaving the pool.
        /// * `pool_amount`: Pool amount that is entering the pool.
        /// * `min_asset_amount`: The calculated amount for the asset must the equal or less
        /// than the given value.
        #[weight = 0]
        fn pool_exit_with_exact_pool_amount(
            origin,
            pool_id: u128,
            asset: T::Hash,
            pool_amount: FixedU128,
            min_asset_amount: FixedU128,
        ) {
            pool_exit_with_exact_amount!(
                initial_params: (origin, pool_id, asset),

                asset_amount: |pool: &Pool<FixedU128, _>, pool_balance: FixedU128, total_supply: FixedU128| {
                    let asset_amount: FixedU128 = math::calc_single_out_given_pool_in(
                        pool_balance,
                        *pool.weights.get(&asset).unwrap(),
                        total_supply,
                        pool.total_weight,
                        pool_amount,
                        pool.swap_fee,
                    )?;
                    ensure!(asset_amount >= min_asset_amount, Error::<T>::LimitOut);
                    ensure!(
                        asset_amount <= pool_balance.check_mul_rslt(&T::MaxOutRatio::get())?,
                        Error::<T>::MaxOutRatio
                    );
                    Ok(asset_amount)
                },
                bound: min_asset_amount,
                ensure_balance: |_| Ok(()),
                event: PoolExitWithExactAssetAmount,
                pool_amount: |_, _, _| Ok(pool_amount)
            )
        }

        /// Pool - Join
        ///
        /// Joins a given set of assets provided from `origin` to `pool_id`.
        ///
        /// # Arguments
        ///
        /// * `origin`: Liquidity Provider (LP). The account whose assets should be transferred.
        /// * `pool_id`: Unique pool identifier.
        /// * `pool_amount`: The amount of LP shares for this pool that should be minted to the provider.
        /// * `max_assets_in`: List of asset upper bounds. No asset should be greater than the
        /// provided values.
        #[weight = 0]
        fn pool_join(origin, pool_id: u128, pool_amount: FixedU128, max_assets_in: Vec<FixedU128>) {
            pool!(
                initial_params: (max_assets_in, origin, pool_amount, pool_id),

                event: PoolJoin,
                transfer_asset: |amount, amount_bound, asset, pool_account, who| {
                    ensure!(amount <= amount_bound, Error::<T>::LimitIn);
                    T::Shares::transfer(asset, who, pool_account, amount)?;
                    Ok(())
                },
                transfer_pool: |_, _, who| Self::mint_pool_shares(pool_id, who, pool_amount)
            )
        }

        /// Pool - Join with exact asset amount
        ///
        /// Joins an asset provided from `origin` to `pool_id`. Differently from `pool_join`,
        /// this method transfers the exactly amount of `asset_amount_in` to `pool_id`.
        ///
        /// # Arguments
        ///
        /// * `origin`: Liquidity Provider (LP). The account whose assets should be received.
        /// * `pool_id`: Unique pool identifier.
        /// * `asset_in`: Asset entering the pool.
        /// * `asset_amount_in`: Asset amount that is entering the pool.
        /// * `min_pool_amount`: The calculated amount for the pool must be equal or greater
        /// than the given value.
        #[weight = 0]
        fn pool_join_with_exact_asset_amount(
            origin,
            pool_id: u128,
            asset_in: T::Hash,
            asset_amount: FixedU128,
            min_pool_amount: FixedU128,
        ) {
            pool_join_with_exact_amount!(
                initial_params: (origin, pool_id, asset_in),

                asset_amount: |_, _, _| Ok(asset_amount),
                bound: min_pool_amount,
                event: PoolJoinWithExactAssetAmount,
                pool_amount: |pool: &Pool<FixedU128, _>, pool_balance: FixedU128, total_supply: FixedU128| {
                    let mul: FixedU128 = pool_balance.check_mul_rslt(&T::MaxInRatio::get())?;
                    ensure!(asset_amount <= mul, Error::<T>::MaxInRatio);
                    let pool_amount: FixedU128 = math::calc_pool_out_given_single_in(
                        pool_balance,
                        *pool.weights.get(&asset_in).unwrap(),
                        total_supply,
                        pool.total_weight,
                        asset_amount,
                        pool.swap_fee,
                    )?;
                    ensure!(pool_amount >= min_pool_amount, Error::<T>::LimitOut);
                    Ok(pool_amount)
                }
            )
        }

        /// Pool - Join with exact pool amount
        ///
        /// Joins an asset provided from `origin` to `pool_id`. Differently from `pool_join`,
        /// this method injects the exactly amount of `pool_amount` to `origin`.
        ///
        /// # Arguments
        ///
        /// * `origin`: Liquidity Provider (LP). The account whose assets should be received.
        /// * `pool_id`: Unique pool identifier.
        /// * `asset`: Asset entering the pool.
        /// * `pool_amount`: Asset amount that is entering the pool.
        /// * `max_asset_amount`: The calculated amount of assets for the pool must be equal or
        /// less than the given value.
        #[weight = 0]
        fn pool_join_with_exact_pool_amount(
            origin,
            pool_id: u128,
            asset: T::Hash,
            pool_amount: FixedU128,
            max_asset_amount: FixedU128,
        ) {
            pool_join_with_exact_amount!(
                initial_params: (origin, pool_id, asset),

                asset_amount: |pool: &Pool<FixedU128, _>, pool_balance: FixedU128, total_supply: FixedU128| {
                    let asset_amount: FixedU128 = math::calc_single_in_given_pool_out(
                        pool_balance,
                        *pool.weights.get(&asset).unwrap(),
                        total_supply,
                        pool.total_weight,
                        pool_amount,
                        pool.swap_fee,
                    )?;
                    ensure!(asset_amount != FixedU128::zero(), Error::<T>::MathApproximation);
                    ensure!(asset_amount <= max_asset_amount, Error::<T>::LimitIn);
                    ensure!(
                        asset_amount <= pool_balance.check_mul_rslt(&T::MaxInRatio::get())?,
                        Error::<T>::MaxInRatio
                    );
                    Ok(asset_amount)
                },
                bound: max_asset_amount,
                event: PoolJoinWithExactPoolAmount,
                pool_amount: |_, _, _| Ok(pool_amount)
            )
        }

        /// Swap - Exact amount in
        ///
        /// Swaps a given `asset_amount_in` of the `asset_in/asset_out` pair to `pool_id`.
        ///
        /// # Arguments
        ///
        /// * `origin`: Liquidity Provider (LP). The account whose assets should be transferred.
        /// * `pool_id`: Unique pool identifier.
        /// * `asset_in`: Asset entering the pool.
        /// * `asset_amount_in`: Amount that will be transferred from the provider to the pool.
        /// * `asset_out`: Asset leaving the pool.
        /// * `min_asset_amount_out`: Minimum asset amount that can leave the pool.
        /// * `max_price`: Market price must be equal or less than the provided value.
        #[weight = 0]
        fn swap_exact_amount_in(
            origin,
            pool_id: u128,
            asset_in: T::Hash,
            asset_amount_in: FixedU128,
            asset_out: T::Hash,
            min_asset_amount_out: FixedU128,
            max_price: FixedU128,
        ) {
            swap_exact_amount!(
                initial_params: (asset_in, asset_out, max_price, origin, pool_id),

                asset_amount_in: |_, _| Ok(asset_amount_in),
                asset_amount_out: |pool: &Pool<FixedU128, _>, pool_account_id| {
                    let balance_in = T::Shares::free_balance(asset_in, pool_account_id);
                    ensure!(
                        asset_amount_in <= balance_in.check_mul_rslt(&T::MaxInRatio::get())?,
                        Error::<T>::MaxInRatio
                    );

                    let balance_out = T::Shares::free_balance(asset_out, pool_account_id);
                    let asset_amount_out = math::calc_out_given_in(
                        balance_in,
                        *pool.weights.get(&asset_in).unwrap(),
                        balance_out,
                        *pool.weights.get(&asset_out).unwrap(),
                        asset_amount_in,
                        pool.swap_fee,
                    )?;
                    ensure!(asset_amount_out >= min_asset_amount_out, Error::<T>::LimitOut);

                    Ok(asset_amount_out)
                },
                asset_bound: min_asset_amount_out,
                event: SwapExactAmountIn
            )
        }

        /// Swap - Exact amount out
        ///
        /// Swaps a given `asset_amount_out` of the `asset_in/asset_out` pair to `origin`.
        ///
        /// # Arguments
        ///
        /// * `origin`: Liquidity Provider (LP). The account whose assets should be received.
        /// * `pool_id`: Unique pool identifier.
        /// * `asset_in`: Asset entering the pool.
        /// * `max_amount_asset_in`: Maximum asset amount that can enter the pool.
        /// * `asset_out`: Asset leaving the pool.
        /// * `asset_amount_out`: Amount that will be transferred from the pool to the provider.
        /// * `max_price`: Market price must be equal or less than the provided value.
        #[weight = 0]
        fn swap_exact_amount_out(
            origin,
            pool_id: u128,
            asset_in: T::Hash,
            max_amount_asset_in: FixedU128,
            asset_out: T::Hash,
            asset_amount_out: FixedU128,
            max_price: FixedU128,
        ) {
            swap_exact_amount!(
                initial_params: (asset_in, asset_out, max_price, origin, pool_id),

                asset_amount_in: |pool: &Pool<FixedU128, _>, pool_account_id| {
                    let balance_in = T::Shares::free_balance(asset_in, pool_account_id);

                    let balance_out = T::Shares::free_balance(asset_out, pool_account_id);
                    ensure!(
                        asset_amount_out <= balance_out.check_mul_rslt(&T::MaxOutRatio::get())?,
                        Error::<T>::MaxOutRatio,
                    );

                    let asset_amount_in = math::calc_in_given_out(
                        balance_in,
                        *pool.weights.get(&asset_in).unwrap(),
                        balance_out,
                        *pool.weights.get(&asset_out).unwrap(),
                        asset_amount_out,
                        pool.swap_fee,
                    )?;
                    ensure!(asset_amount_in <= max_amount_asset_in, Error::<T>::LimitIn);

                    Ok(asset_amount_in)
                },
                asset_amount_out: |_, _| Ok(asset_amount_out),
                asset_bound: max_amount_asset_in,
                event: SwapExactAmountOut
            )
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

    pub fn get_spot_price(
        pool_id: u128,
        asset_in: T::Hash,
        asset_out: T::Hash,
    ) -> Result<FixedU128, DispatchError> {
        if let Some(pool) = Self::pools(pool_id) {
            // ensure!(pool.bound(asset_in), Error::<T>::AssetNotBound)?;
            // ensure!(pool.bound(asset_out), Error::<T>::AssetNotBound)?;

            let pool_account = Self::pool_account_id(pool_id);
            let balance_in = T::Shares::free_balance(asset_in, &pool_account);
            let in_weight = pool.weights.get(&asset_in).unwrap();
            let balance_out = T::Shares::free_balance(asset_out, &pool_account);
            let out_weight = pool.weights.get(&asset_out).unwrap();

            math::calc_spot_price(
                balance_in,
                *in_weight,
                balance_out,
                *out_weight,
                FixedU128::zero(), //fee
            )
        } else {
            // Err(Error::<T>::PoolDoesNotExist)?;
            Ok(FixedU128::zero())
        }
    }

    fn mint_pool_shares(pool_id: u128, to: &T::AccountId, amount: FixedU128) -> DispatchResult {
        let shares_id = Self::pool_shares_id(pool_id);
        T::Shares::generate(shares_id, to, amount)
    }

    fn burn_pool_shares(pool_id: u128, from: &T::AccountId, amount: FixedU128) -> DispatchResult {
        let shares_id = Self::pool_shares_id(pool_id);
        T::Shares::destroy(shares_id, from, amount)
    }

    fn pool_master_account() -> T::AccountId {
        T::ModuleId::get().into_account()
    }

    fn inc_next_pool_id() -> u128 {
        let id = NextPoolId::get();
        NextPoolId::mutate(|n| *n = n.saturating_add(1));
        id
    }

    fn get_denormalized_weight(pool_id: u128, asset: T::Hash) -> FixedU128 {
        if let Some(pool) = Self::pools(pool_id) {
            if let Some(val) = pool.weights.get(&asset) {
                return *val;
            }
        }
        FixedU128::zero()
    }

    fn get_normalized_weight(_pool_id: u128, _asset: T::Hash) -> u128 {
        // unimplemented
        0
    }

    fn pool_by_id(pool_id: u128) -> Result<Pool<FixedU128, T::Hash>, Error<T>>
    where
        T: Trait,
    {
        Self::pools(pool_id).ok_or(Error::<T>::PoolDoesNotExist.into())
    }
}

impl<T> Swaps<T::AccountId, T::Hash> for Module<T>
where
    T: Trait,
{
    /// Deploys a new pool with the given assets and weights.
    ///
    /// # Arguments
    ///
    /// * `who`: The account that is the creator of the pool. Must have enough
    /// funds for each of the assets to cover the `MinLiqudity`.
    /// * `assets`: The assets that are used in the pool.
    /// * `swap_fee`: The fee applied to each swap.
    /// * `weights`: These are the denormalized weights (the raw weights).
    fn do_create_pool(
        who: T::AccountId,
        assets: Vec<T::Hash>,
        swap_fee: FixedU128,
        weights: Vec<FixedU128>,
    ) -> sp_std::result::Result<u128, DispatchError> {
        check_provided_values_len_must_equal_assets_len::<T, _>(&assets, &weights)?;

        ensure!(
            assets.len() <= T::MaxAssets::get(),
            Error::<T>::TooManyAssets
        );

        let amount = T::MinLiquidity::get();

        let next_pool_id = Self::inc_next_pool_id();
        let pool_account = Self::pool_account_id(next_pool_id);
        let mut map = BTreeMap::new();
        let mut total_weight = FixedU128::zero();

        for (asset, weight) in assets.iter().copied().zip(weights) {
            let free_balance = T::Shares::free_balance(asset, &who);
            ensure!(free_balance >= amount, Error::<T>::InsufficientBalance);
            ensure!(
                weight >= T::MinWeight::get(),
                Error::<T>::BelowMinimumWeight
            );
            ensure!(
                weight <= T::MaxWeight::get(),
                Error::<T>::AboveMaximumWeight
            );
            T::Shares::transfer(asset, &who, &pool_account, amount)?;
            map.insert(asset, weight);
            total_weight = total_weight.check_add_rslt(&weight)?;
        }

        ensure!(
            total_weight <= T::MaxTotalWeight::get(),
            Error::<T>::MaxTotalWeight
        );

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

        Self::deposit_event(RawEvent::PoolCreate(CommonPoolEventParams {
            pool_id: next_pool_id,
            who,
        }));

        Ok(next_pool_id)
    }
}

#[inline]
fn check_provided_values_len_must_equal_assets_len<T, U>(
    assets: &[T::Hash],
    provided_values: &[U],
) -> Result<(), Error<T>>
where
    T: Trait,
{
    if assets.len() != provided_values.len() {
        return Err(Error::<T>::ProvidedValuesLenMustEqualAssetsLen.into());
    }
    Ok(())
}
