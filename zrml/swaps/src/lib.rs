// Copyright 2022-2023 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
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
//
// This file incorporates work covered by the license above but
// published without copyright notice by Balancer Labs
// (<https://balancer.finance>, contact@balancer.finance) in the
// balancer-core repository
// <https://github.com/balancer-labs/balancer-core>.

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]

extern crate alloc;

#[macro_use]
mod utils;

mod benchmarks;
mod events;
pub mod fixed;
pub mod math;
pub mod migrations;
pub mod mock;
mod tests;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{
        events::{CommonPoolEventParams, PoolAssetEvent, PoolAssetsEvent, SwapEvent},
        utils::{
            pool_exit_with_exact_amount, pool_join_with_exact_amount, swap_exact_amount,
            PoolExitWithExactAmountParams, PoolJoinWithExactAmountParams, PoolParams,
            SwapExactAmountParams,
        },
        weights::*,
    };
    use alloc::{collections::btree_map::BTreeMap, vec, vec::Vec};
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::{DispatchResultWithPostInfo, Weight},
        ensure,
        pallet_prelude::{StorageMap, StorageValue, ValueQuery},
        traits::{Get, IsType, StorageVersion},
        transactional, Blake2_128Concat, PalletId,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use orml_traits::MultiCurrency;
    use sp_arithmetic::{
        traits::{Saturating, Zero},
        Perbill,
    };
    use sp_runtime::{
        traits::AccountIdConversion, DispatchError, DispatchResult, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        constants::CENT,
        math::{
            checked_ops_res::{CheckedAddRes, CheckedMulRes},
            fixed::{BaseProvider, FixedMul, ZeitgeistBase},
        },
        traits::{MarketCommonsPalletApi, Swaps, ZeitgeistAssetManager},
        types::{Asset, Pool, PoolId, PoolStatus, ScoringRule, SerdeWrapper},
    };
    use zrml_liquidity_mining::LiquidityMiningPalletApi;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(3);

    pub(crate) type BalanceOf<T> = <<T as Config>::AssetManager as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;

    const MIN_BALANCE: u128 = CENT;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
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
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)` where `n` is the number of assets in the specified pool
        // Using `min_assets_out.len()` is fine because we don't iterate over the assets before
        // verifying that `min_assets_out` has the correct length. We do limit the linear factor to
        // the maximum number of assets to prevent unnecessary spending in case of erroneous input,
        // though.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::pool_exit(
            min_assets_out.len().min(T::MaxAssets::get().into()) as u32
        ))]
        #[transactional]
        pub fn pool_exit(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: PoolId,
            #[pallet::compact] pool_amount: BalanceOf<T>,
            min_assets_out: Vec<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(pool_amount != Zero::zero(), Error::<T>::ZeroAmount);
            let who_clone = who.clone();
            let pool = Self::pool_by_id(pool_id)?;
            // If the pool is still in use, prevent a pool drain.
            Self::ensure_minimum_liquidity_shares(pool_id, &pool, pool_amount)?;
            let pool_account_id = Pallet::<T>::pool_account_id(&pool_id);
            let params = PoolParams {
                asset_bounds: min_assets_out,
                event: |evt| Self::deposit_event(Event::PoolExit(evt)),
                pool_account_id: &pool_account_id,
                pool_amount,
                pool_id,
                pool: &pool,
                transfer_asset: |amount, amount_bound, asset| {
                    Self::ensure_minimum_balance(pool_id, &pool, asset, amount)?;
                    ensure!(amount >= amount_bound, Error::<T>::LimitOut);
                    T::LiquidityMining::remove_shares(&who, &pool.market_id, amount);
                    T::AssetManager::transfer(asset, &pool_account_id, &who, amount)?;
                    Ok(())
                },
                transfer_pool: || {
                    Self::burn_pool_shares(pool_id, &who, pool_amount)?;
                    Ok(())
                },
                fee: |amount: BalanceOf<T>| {
                    let exit_fee_amount = amount.bmul(Self::calc_exit_fee(&pool))?;
                    Ok(exit_fee_amount)
                },
                who: who_clone,
            };
            crate::utils::pool::<_, _, _, _, T>(params)
        }

        /// Pool - Exit with exact pool amount
        ///
        /// Takes an asset from `pool_id` and transfers to `origin`. Differently from `pool_exit`,
        /// this method injects the exactly amount of `asset_amount` to `origin`.
        ///
        /// # Arguments
        ///
        /// * `origin`: Liquidity Provider (LP). The account whose assets should be received.
        /// * `pool_id`: Unique pool identifier.
        /// * `asset`: Asset leaving the pool.
        /// * `asset_amount`: Asset amount that is leaving the pool.
        /// * `max_pool_amount`: The calculated amount of assets for the pool must be equal or
        /// greater than the given value.
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::pool_exit_with_exact_asset_amount())]
        // MARK(non-transactional): Immediately calls and returns a transactional.
        pub fn pool_exit_with_exact_asset_amount(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: PoolId,
            asset: Asset<MarketIdOf<T>>,
            #[pallet::compact] asset_amount: BalanceOf<T>,
            #[pallet::compact] max_pool_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            <Self as Swaps<T::AccountId>>::pool_exit_with_exact_asset_amount(
                who,
                pool_id,
                asset,
                asset_amount,
                max_pool_amount,
            )
            .map(|_| ())
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
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::pool_exit_with_exact_pool_amount())]
        #[transactional]
        pub fn pool_exit_with_exact_pool_amount(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: PoolId,
            asset: Asset<MarketIdOf<T>>,
            #[pallet::compact] pool_amount: BalanceOf<T>,
            #[pallet::compact] min_asset_amount: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(pool_amount != Zero::zero(), Error::<T>::ZeroAmount);
            let pool = Self::pool_by_id(pool_id)?;
            let pool_ref = &pool;
            let who = ensure_signed(origin)?;
            let who_clone = who.clone();
            Self::ensure_minimum_liquidity_shares(pool_id, &pool, pool_amount)?;

            let params = PoolExitWithExactAmountParams {
                asset,
                asset_amount: |asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let mul: BalanceOf<T> = total_supply.bmul(T::MaxInRatio::get())?;
                    ensure!(pool_amount <= mul, Error::<T>::MaxInRatio);
                    let asset_amount: BalanceOf<T> = crate::math::calc_single_out_given_pool_in(
                        asset_balance.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset)?,
                        total_supply.saturated_into(),
                        pool.total_weight.ok_or(Error::<T>::PoolMissingWeight)?.saturated_into(),
                        pool_amount.saturated_into(),
                        pool.swap_fee.ok_or(Error::<T>::PoolMissingFee)?.saturated_into(),
                        T::ExitFee::get().saturated_into(),
                    )?
                    .saturated_into();
                    ensure!(asset_amount != Zero::zero(), Error::<T>::ZeroAmount);
                    ensure!(asset_amount >= min_asset_amount, Error::<T>::LimitOut);
                    ensure!(
                        asset_amount <= asset_balance.bmul(T::MaxOutRatio::get())?,
                        Error::<T>::MaxOutRatio
                    );
                    Self::ensure_minimum_balance(pool_id, &pool, asset, asset_amount)?;
                    T::LiquidityMining::remove_shares(&who, &pool_ref.market_id, asset_amount);
                    Ok(asset_amount)
                },
                bound: min_asset_amount,
                ensure_balance: |_| Ok(()),
                event: |evt| Self::deposit_event(Event::PoolExitWithExactPoolAmount(evt)),
                who: who_clone,
                pool_amount: |_, _| Ok(pool_amount),
                pool_id,
                pool: pool_ref,
            };
            pool_exit_with_exact_amount::<_, _, _, _, T>(params)
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
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)` where `n` is the number of assets in the specified pool
        // Using `min_assets_out.len()` is fine because we don't iterate over the assets before
        // verifying that `min_assets_out` has the correct length. We do limit the linear factor to
        // the maximum number of assets to prevent unnecessary spending in case of erroneous input,
        // though.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::pool_join(
            max_assets_in.len().min(T::MaxAssets::get().into()) as u32,
        ))]
        #[transactional]
        pub fn pool_join(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: PoolId,
            #[pallet::compact] pool_amount: BalanceOf<T>,
            max_assets_in: Vec<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            ensure!(pool_amount != Zero::zero(), Error::<T>::ZeroAmount);
            let pool = Self::pool_by_id(pool_id)?;
            ensure!(
                matches!(pool.pool_status, PoolStatus::Initialized | PoolStatus::Active),
                Error::<T>::InvalidPoolStatus,
            );
            let pool_account_id = Pallet::<T>::pool_account_id(&pool_id);

            let params = PoolParams {
                asset_bounds: max_assets_in,
                event: |evt| Self::deposit_event(Event::PoolJoin(evt)),
                pool_account_id: &pool_account_id,
                pool_amount,
                pool_id,
                pool: &pool,
                transfer_asset: |amount, amount_bound, asset| {
                    ensure!(amount <= amount_bound, Error::<T>::LimitIn);
                    T::AssetManager::transfer(asset, &who, &pool_account_id, amount)?;
                    T::LiquidityMining::add_shares(who.clone(), pool.market_id, amount);
                    Ok(())
                },
                transfer_pool: || Self::mint_pool_shares(pool_id, &who, pool_amount),
                fee: |_| Ok(0u128.saturated_into()),
                who: who.clone(),
            };

            crate::utils::pool::<_, _, _, _, T>(params)?;
            Ok(Some(T::WeightInfo::pool_join(pool.assets.len().saturated_into())).into())
        }

        /// Pool - Join with exact asset amount
        ///
        /// Joins an asset provided from `origin` to `pool_id`. Differently from `pool_join`,
        /// this method transfers the exactly amount of `asset_amount` to `pool_id`.
        ///
        /// # Arguments
        ///
        /// * `origin`: Liquidity Provider (LP). The account whose assets should be received.
        /// * `pool_id`: Unique pool identifier.
        /// * `asset_in`: Asset entering the pool.
        /// * `asset_amount`: Asset amount that is entering the pool.
        /// * `min_pool_amount`: The calculated amount for the pool must be equal or greater
        /// than the given value.
        ///
        /// # Weight
        ///
        /// Complexity: O(1)
        // MARK(non-transactional): Immediately calls and returns a transactional.
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::pool_join_with_exact_asset_amount())]
        pub fn pool_join_with_exact_asset_amount(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: PoolId,
            asset_in: Asset<MarketIdOf<T>>,
            #[pallet::compact] asset_amount: BalanceOf<T>,
            #[pallet::compact] min_pool_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            <Self as Swaps<T::AccountId>>::pool_join_with_exact_asset_amount(
                who,
                pool_id,
                asset_in,
                asset_amount,
                min_pool_amount,
            )
            .map(|_| ())
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
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::pool_join_with_exact_pool_amount())]
        #[transactional]
        pub fn pool_join_with_exact_pool_amount(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: PoolId,
            asset: Asset<MarketIdOf<T>>,
            #[pallet::compact] pool_amount: BalanceOf<T>,
            #[pallet::compact] max_asset_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let pool = Pallet::<T>::pool_by_id(pool_id)?;
            let pool_account_id = Pallet::<T>::pool_account_id(&pool_id);
            let who = ensure_signed(origin)?;
            let who_clone = who.clone();
            let params = PoolJoinWithExactAmountParams {
                asset,
                asset_amount: |asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let mul: BalanceOf<T> = total_supply.bmul(T::MaxOutRatio::get())?;
                    ensure!(pool_amount <= mul, Error::<T>::MaxOutRatio);
                    let asset_amount: BalanceOf<T> = crate::math::calc_single_in_given_pool_out(
                        asset_balance.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset)?,
                        total_supply.saturated_into(),
                        pool.total_weight.ok_or(Error::<T>::PoolMissingWeight)?.saturated_into(),
                        pool_amount.saturated_into(),
                        pool.swap_fee.ok_or(Error::<T>::PoolMissingFee)?.saturated_into(),
                    )?
                    .saturated_into();
                    ensure!(asset_amount != Zero::zero(), Error::<T>::ZeroAmount);
                    ensure!(asset_amount <= max_asset_amount, Error::<T>::LimitIn);
                    ensure!(
                        asset_amount <= asset_balance.checked_mul_res(&T::MaxInRatio::get())?,
                        Error::<T>::MaxInRatio
                    );
                    T::LiquidityMining::add_shares(who.clone(), pool.market_id, asset_amount);
                    Ok(asset_amount)
                },
                bound: max_asset_amount,
                event: |evt| Self::deposit_event(Event::PoolJoinWithExactPoolAmount(evt)),
                pool_account_id: &pool_account_id,
                pool_amount: |_, _| Ok(pool_amount),
                pool_id,
                pool: &pool,
                who: who_clone,
            };
            pool_join_with_exact_amount::<_, _, _, T>(params)
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
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)` if the scoring rule is CPMM, `O(n)` where `n` is the amount of
        /// assets if the scoring rule is Rikiddo.
        // TODO(#790): Replace with maximum of CPMM and Rikiddo benchmark!
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::swap_exact_amount_in_cpmm())]
        #[transactional]
        pub fn swap_exact_amount_in(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: PoolId,
            asset_in: Asset<MarketIdOf<T>>,
            #[pallet::compact] asset_amount_in: BalanceOf<T>,
            asset_out: Asset<MarketIdOf<T>>,
            min_asset_amount_out: Option<BalanceOf<T>>,
            max_price: Option<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let weight = <Self as Swaps<T::AccountId>>::swap_exact_amount_in(
                who,
                pool_id,
                asset_in,
                asset_amount_in,
                asset_out,
                min_asset_amount_out,
                max_price,
                false,
            )?;
            Ok(Some(weight).into())
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
        /// * `max_asset_amount_in`: Maximum asset amount that can enter the pool.
        /// * `asset_out`: Asset leaving the pool.
        /// * `asset_amount_out`: Amount that will be transferred from the pool to the provider.
        /// * `max_price`: Market price must be equal or less than the provided value.
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)` if the scoring rule is CPMM, `O(n)` where `n` is the amount of
        /// assets if the scoring rule is Rikiddo.
        // TODO(#790): Replace with maximum of CPMM and Rikiddo benchmark!
        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::swap_exact_amount_out_cpmm())]
        #[transactional]
        pub fn swap_exact_amount_out(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: PoolId,
            asset_in: Asset<MarketIdOf<T>>,
            max_asset_amount_in: Option<BalanceOf<T>>,
            asset_out: Asset<MarketIdOf<T>>,
            #[pallet::compact] asset_amount_out: BalanceOf<T>,
            max_price: Option<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let weight = <Self as Swaps<T::AccountId>>::swap_exact_amount_out(
                who,
                pool_id,
                asset_in,
                max_asset_amount_in,
                asset_out,
                asset_amount_out,
                max_price,
                false,
            )?;
            Ok(Some(weight).into())
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The fee for exiting a pool.
        #[pallet::constant]
        type ExitFee: Get<BalanceOf<Self>>;

        type LiquidityMining: LiquidityMiningPalletApi<
                AccountId = Self::AccountId,
                Balance = BalanceOf<Self>,
                BlockNumber = Self::BlockNumber,
                MarketId = MarketIdOf<Self>,
            >;

        type MarketCommons: MarketCommonsPalletApi<
                AccountId = Self::AccountId,
                BlockNumber = Self::BlockNumber,
                Balance = BalanceOf<Self>,
            >;

        #[pallet::constant]
        type MaxAssets: Get<u16>;

        #[pallet::constant]
        type MaxInRatio: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MaxOutRatio: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MaxSwapFee: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MaxTotalWeight: Get<u128>;

        #[pallet::constant]
        type MaxWeight: Get<u128>;

        #[pallet::constant]
        /// The minimum amount of assets in a pool.
        type MinAssets: Get<u16>;

        #[pallet::constant]
        type MinWeight: Get<u128>;

        /// The module identifier.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Shares of outcome assets and native currency
        type AssetManager: ZeitgeistAssetManager<Self::AccountId, CurrencyId = Asset<MarketIdOf<Self>>>;

        /// The weight information for swap's dispatchable functions.
        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The weight of an asset in a CPMM swap pool is greather than the upper weight cap.
        AboveMaximumWeight,
        /// The weight of an asset in a CPMM swap pool could not be found.
        AssetNotBound,
        /// The asset in question could not be found within the pool.
        AssetNotInPool,
        /// The base asset of the swaps pool was None although a value was expected.
        BaseAssetNotFound,
        /// The spot price of an asset pair was greater than the specified limit.
        BadLimitPrice,
        /// The weight of an asset in a CPMM swap pool is lower than the upper weight cap.
        BelowMinimumWeight,
        /// Some funds could not be transferred due to a too low balance.
        InsufficientBalance,
        /// Liquidity provided to new CPMM pool is less than the minimum allowed balance.
        InsufficientLiquidity,
        /// Could not create CPMM pool since no amount was specified.
        InvalidAmountArgument,
        /// Could not create CPMM pool since no fee was supplied.
        InvalidFeeArgument,
        /// Dispatch called on pool with invalid status.
        InvalidPoolStatus,
        /// A function that is only valid for pools with specific scoring rules was called for a
        /// pool with another scoring rule.
        InvalidScoringRule,
        /// A function was called for a swaps pool that does not fulfill the state requirement.
        InvalidStateTransition,
        /// Could not create CPMM pool since no weights were supplied.
        InvalidWeightArgument,
        /// A transferal of funds into a swaps pool was above a threshhold specified by the sender.
        LimitIn,
        /// Subsidy amount is too small.
        InvalidSubsidyAmount,
        /// No limit was specified for a swap.
        LimitMissing,
        /// A transferal of funds out of a swaps pool was below a threshhold specified by the
        /// receiver.
        LimitOut,
        /// The custom math library yielded an invalid result (most times unexpected zero value).
        MathApproximation,
        /// The proportion of an asset added into a pool in comparison to the amount
        /// of that asset in the pool is above the threshhold specified by a constant.
        MaxInRatio,
        /// The proportion of an asset taken from a pool in comparison to the amount
        /// of that asset in the pool is above the threshhold specified by a constant.
        MaxOutRatio,
        /// The total weight of all assets within a CPMM pool is above a treshhold specified
        /// by a constant.
        MaxTotalWeight,
        /// The pool in question does not exist.
        PoolDoesNotExist,
        /// A pool balance dropped below the allowed minimum.
        PoolDrain,
        /// The pool in question is inactive.
        PoolIsNotActive,
        /// The CPMM pool in question does not have a fee, although it should.
        PoolMissingFee,
        /// The Rikiddo pool in question does not have subsidy, although it should.
        PoolMissingSubsidy,
        /// The CPPM pool in question does not have weights, although it should.
        PoolMissingWeight,
        /// Two vectors do not have the same length (usually CPMM pool assets and weights).
        ProvidedValuesLenMustEqualAssetsLen,
        /// No swap fee information found for CPMM pool
        SwapFeeMissing,
        /// The swap fee is higher than the allowed maximum.
        SwapFeeTooHigh,
        /// Tried to create a pool that has less assets than the lower threshhold specified by
        /// a constant.
        TooFewAssets,
        /// Tried to create a pool that has more assets than the upper threshhold specified by
        /// a constant.
        TooManyAssets,
        /// Tried to create a pool with at least two identical assets.
        SomeIdenticalAssets,
        /// The pool does not support swapping the assets in question.
        UnsupportedTrade,
        /// The outcome asset specified as the winning asset was not found in the pool.
        WinningAssetNotFound,
        /// Some amount in a transaction equals zero.
        ZeroAmount,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// Share holder rewards were distributed. \[pool_id, num_accounts_rewarded, amount\]
        DistributeShareHolderRewards(PoolId, u64, BalanceOf<T>),
        /// A new pool has been created. \[CommonPoolEventParams, pool, pool_amount, pool_account\]
        PoolCreate(
            CommonPoolEventParams<<T as frame_system::Config>::AccountId>,
            Pool<BalanceOf<T>, MarketIdOf<T>>,
            BalanceOf<T>,
            T::AccountId,
        ),
        /// A pool was closed. \[pool_id\]
        PoolClosed(PoolId),
        /// A pool was cleaned up. \[pool_id\]
        PoolCleanedUp(PoolId),
        /// A pool was opened. \[pool_id\]
        PoolActive(PoolId),
        /// Someone has exited a pool. \[PoolAssetsEvent\]
        PoolExit(
            PoolAssetsEvent<
                <T as frame_system::Config>::AccountId,
                Asset<MarketIdOf<T>>,
                BalanceOf<T>,
            >,
        ),
        /// Exits a pool given an exact amount of an asset. \[PoolAssetEvent\]
        PoolExitWithExactAssetAmount(
            PoolAssetEvent<
                <T as frame_system::Config>::AccountId,
                Asset<MarketIdOf<T>>,
                BalanceOf<T>,
            >,
        ),
        /// Exits a pool given an exact pool's amount. \[PoolAssetEvent\]
        PoolExitWithExactPoolAmount(
            PoolAssetEvent<
                <T as frame_system::Config>::AccountId,
                Asset<MarketIdOf<T>>,
                BalanceOf<T>,
            >,
        ),
        /// Someone has joined a pool. \[PoolAssetsEvent\]
        PoolJoin(
            PoolAssetsEvent<
                <T as frame_system::Config>::AccountId,
                Asset<MarketIdOf<T>>,
                BalanceOf<T>,
            >,
        ),
        /// Joins a pool given an exact amount of an asset. \[PoolAssetEvent\]
        PoolJoinWithExactAssetAmount(
            PoolAssetEvent<
                <T as frame_system::Config>::AccountId,
                Asset<MarketIdOf<T>>,
                BalanceOf<T>,
            >,
        ),
        /// Joins a pool given an exact pool's amount. \[PoolAssetEvent\]
        PoolJoinWithExactPoolAmount(
            PoolAssetEvent<
                <T as frame_system::Config>::AccountId,
                Asset<MarketIdOf<T>>,
                BalanceOf<T>,
            >,
        ),
        /// Pool was manually destroyed. \[pool_id\]
        PoolDestroyed(PoolId),
        /// Pool destroyed due to insufficient subsidy. \[pool_id, \[(provider, subsidy), ...\]\]
        PoolDestroyedInSubsidyPhase(
            PoolId,
            Vec<(<T as frame_system::Config>::AccountId, BalanceOf<T>)>,
        ),
        /// An exact amount of an asset is entering the pool. \[SwapEvent\]
        SwapExactAmountIn(
            SwapEvent<<T as frame_system::Config>::AccountId, Asset<MarketIdOf<T>>, BalanceOf<T>>,
        ),
        /// An exact amount of an asset is leaving the pool. \[SwapEvent\]
        SwapExactAmountOut(
            SwapEvent<<T as frame_system::Config>::AccountId, Asset<MarketIdOf<T>>, BalanceOf<T>>,
        ),
        /// Fees were paid to the market creator. \[market_id , payer, payee, amount, asset\]
        MarketCreatorFeesPaid(
            MarketIdOf<T>,
            <T as frame_system::Config>::AccountId,
            <T as frame_system::Config>::AccountId,
            BalanceOf<T>,
            Asset<MarketIdOf<T>>,
        ),
        /// Fee payment to market creator failed (usually due to existential deposit requirements)
        /// \[market_id, payer, payee, amount, asset, error\]
        MarketCreatorFeePaymentFailed(
            MarketIdOf<T>,
            <T as frame_system::Config>::AccountId,
            <T as frame_system::Config>::AccountId,
            BalanceOf<T>,
            Asset<MarketIdOf<T>>,
            DispatchError,
        ),
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        PoolId,
        Option<Pool<BalanceOf<T>, MarketIdOf<T>>>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn next_pool_id)]
    pub type NextPoolId<T> = StorageValue<_, PoolId, ValueQuery>;

    impl<T: Config> Pallet<T> {
        pub fn get_spot_price(
            pool_id: &PoolId,
            asset_in: &Asset<MarketIdOf<T>>,
            asset_out: &Asset<MarketIdOf<T>>,
            with_fees: bool,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let pool = Self::pool_by_id(*pool_id)?;
            ensure!(pool.assets.binary_search(asset_in).is_ok(), Error::<T>::AssetNotInPool);
            ensure!(pool.assets.binary_search(asset_out).is_ok(), Error::<T>::AssetNotInPool);
            let pool_account = Self::pool_account_id(pool_id);
            let balance_in = T::AssetManager::free_balance(*asset_in, &pool_account);
            let balance_out = T::AssetManager::free_balance(*asset_out, &pool_account);
            let in_weight = Self::pool_weight_rslt(&pool, asset_in)?;
            let out_weight = Self::pool_weight_rslt(&pool, asset_out)?;

            let swap_fee = if with_fees {
                let swap_fee = pool.swap_fee.ok_or(Error::<T>::SwapFeeMissing)?;
                let market = T::MarketCommons::market(&pool.market_id)?;
                market
                    .creator_fee
                    .mul_floor(ZeitgeistBase::<u128>::get()?)
                    .checked_add(swap_fee.try_into().map_err(|_| Error::<T>::SwapFeeTooHigh)?)
                    .ok_or(Error::<T>::SwapFeeTooHigh)?
            } else {
                BalanceOf::<T>::zero().saturated_into()
            };

            Ok(crate::math::calc_spot_price(
                balance_in.saturated_into(),
                in_weight,
                balance_out.saturated_into(),
                out_weight,
                swap_fee,
            )?
            .saturated_into())
        }

        // Returns vector of pairs `(a, p)` where `a` ranges over all assets in the pool and `p` is
        // the spot price of swapping the base asset for `a` (including swap fees if `with_fees` is
        // `true`).
        pub fn get_all_spot_prices(
            pool_id: &PoolId,
            with_fees: bool,
        ) -> Result<Vec<(Asset<MarketIdOf<T>>, BalanceOf<T>)>, DispatchError> {
            let pool = Self::pool_by_id(*pool_id)?;
            pool.assets
                .into_iter()
                .map(|asset| {
                    let spot_price =
                        Self::get_spot_price(pool_id, &pool.base_asset, &asset, with_fees)?;
                    Ok((asset, spot_price))
                })
                .collect()
        }

        #[inline]
        pub fn pool_account_id(pool_id: &PoolId) -> T::AccountId {
            T::PalletId::get().into_sub_account_truncating((*pool_id).saturated_into::<u128>())
        }

        /// The minimum allowed balance of `asset` in a liquidity pool.
        pub(crate) fn min_balance(asset: Asset<MarketIdOf<T>>) -> BalanceOf<T> {
            T::AssetManager::minimum_balance(asset).max(MIN_BALANCE.saturated_into())
        }

        /// Returns the minimum allowed balance allowed for a pool with id `pool_id` containing
        /// `assets`.
        ///
        /// The minimum allowed balance is the maximum of all minimum allowed balances of assets
        /// contained in the pool, _including_ the pool shares asset. This ensures that none of the
        /// accounts involved are slashed when a pool is created with the minimum amount.
        ///
        /// **Should** only be called if `assets` is non-empty. Note that the existence of a pool
        /// with the specified `pool_id` is not mandatory.
        pub(crate) fn min_balance_of_pool(
            pool_id: PoolId,
            assets: &[Asset<MarketIdOf<T>>],
        ) -> BalanceOf<T> {
            assets
                .iter()
                .map(|asset| Self::min_balance(*asset))
                .max()
                .unwrap_or_else(|| MIN_BALANCE.saturated_into())
                .max(Self::min_balance(Self::pool_shares_id(pool_id)))
        }

        fn ensure_minimum_liquidity_shares(
            pool_id: PoolId,
            pool: &Pool<BalanceOf<T>, MarketIdOf<T>>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            if pool.pool_status == PoolStatus::Clean {
                return Ok(());
            }
            let pool_shares_id = Self::pool_shares_id(pool_id);
            let total_issuance = T::AssetManager::total_issuance(pool_shares_id);
            let max_withdraw =
                total_issuance.saturating_sub(Self::min_balance(pool_shares_id).saturated_into());
            ensure!(amount <= max_withdraw, Error::<T>::PoolDrain);
            Ok(())
        }

        fn ensure_minimum_balance(
            pool_id: PoolId,
            pool: &Pool<BalanceOf<T>, MarketIdOf<T>>,
            asset: Asset<MarketIdOf<T>>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            // No need to prevent a clean pool from getting drained.
            if pool.pool_status == PoolStatus::Clean {
                return Ok(());
            }
            let pool_account = Self::pool_account_id(&pool_id);
            let balance = T::AssetManager::free_balance(asset, &pool_account);
            let max_withdraw = balance.saturating_sub(Self::min_balance(asset).saturated_into());
            ensure!(amount <= max_withdraw, Error::<T>::PoolDrain);
            Ok(())
        }

        fn handle_creator_fee_transfer(
            market_id: MarketIdOf<T>,
            fee_asset: Asset<MarketIdOf<T>>,
            payer: T::AccountId,
            payee: T::AccountId,
            fee_amount: BalanceOf<T>,
        ) {
            if let Err(err) = T::AssetManager::transfer(fee_asset, &payer, &payee, fee_amount) {
                Self::deposit_event(Event::MarketCreatorFeePaymentFailed(
                    market_id, payer, payee, fee_amount, fee_asset, err,
                ));
            } else {
                Self::deposit_event(Event::MarketCreatorFeesPaid(
                    market_id, payer, payee, fee_amount, fee_asset,
                ));
            }
        }

        // Infallible, should fee transfer fail, the informant will keep the fees and an event is emitted.
        #[allow(clippy::too_many_arguments)]
        fn handle_creator_fees(
            amount: BalanceOf<T>,
            fee_asset: Asset<MarketIdOf<T>>,
            base_asset: Asset<MarketIdOf<T>>,
            fee: Perbill,
            payee: T::AccountId,
            payer: T::AccountId,
            pool_id: PoolId,
            market_id: MarketIdOf<T>,
        ) {
            if fee.is_zero() || payee == payer {
                return;
            };

            let mut fee_amount = fee.mul_floor(amount);

            if fee_asset != base_asset {
                let balance_before = T::AssetManager::free_balance(base_asset, &payer);
                let swap_result = <Self as Swaps<T::AccountId>>::swap_exact_amount_in(
                    payer.clone(),
                    pool_id,
                    fee_asset,
                    fee_amount,
                    base_asset,
                    None,
                    Some(<BalanceOf<T>>::saturated_from(u128::MAX)),
                    true,
                );

                if swap_result.is_err() {
                    Self::handle_creator_fee_transfer(
                        market_id, fee_asset, payer, payee, fee_amount,
                    );
                    return;
                }

                let balance_after = T::AssetManager::free_balance(base_asset, &payer);
                fee_amount = balance_after.saturating_sub(balance_before);
            }

            Self::handle_creator_fee_transfer(market_id, base_asset, payer, payee, fee_amount);
        }

        pub(crate) fn burn_pool_shares(
            pool_id: PoolId,
            from: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let shares_id = Self::pool_shares_id(pool_id);
            // Check that the account has at least as many free shares as we wish to burn!
            T::AssetManager::ensure_can_withdraw(shares_id, from, amount)
                .map_err(|_| Error::<T>::InsufficientBalance)?;
            let missing = T::AssetManager::slash(shares_id, from, amount);
            debug_assert!(
                missing.is_zero(),
                "Could not slash all of the amount. shares_id {:?}, who: {:?}, amount: {:?}.",
                shares_id,
                &from,
                amount,
            );
            Ok(())
        }

        #[inline]
        pub(crate) fn check_provided_values_len_must_equal_assets_len<U>(
            assets: &[Asset<MarketIdOf<T>>],
            provided_values: &[U],
        ) -> Result<(), Error<T>>
        where
            T: Config,
        {
            if assets.len() != provided_values.len() {
                return Err(Error::<T>::ProvidedValuesLenMustEqualAssetsLen);
            }
            Ok(())
        }

        pub(crate) fn check_if_pool_is_active(
            pool: &Pool<BalanceOf<T>, MarketIdOf<T>>,
        ) -> DispatchResult {
            match pool.pool_status {
                PoolStatus::Active => Ok(()),
                _ => Err(Error::<T>::PoolIsNotActive.into()),
            }
        }

        pub(crate) fn mint_pool_shares(
            pool_id: PoolId,
            to: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let shares_id = Self::pool_shares_id(pool_id);
            T::AssetManager::deposit(shares_id, to, amount)
        }

        pub(crate) fn pool_shares_id(pool_id: PoolId) -> Asset<MarketIdOf<T>> {
            Asset::PoolShare(SerdeWrapper(pool_id))
        }

        pub fn pool_by_id(
            pool_id: PoolId,
        ) -> Result<Pool<BalanceOf<T>, MarketIdOf<T>>, DispatchError>
        where
            T: Config,
        {
            Self::pools(pool_id).ok_or_else(|| Error::<T>::PoolDoesNotExist.into())
        }

        fn inc_next_pool_id() -> Result<PoolId, DispatchError> {
            let id = <NextPoolId<T>>::get();
            <NextPoolId<T>>::try_mutate(|n| {
                *n = n.checked_add_res(&1)?;
                Ok::<_, DispatchError>(())
            })?;
            Ok(id)
        }

        // Mutates a stored pool. Returns `Err` if `pool_id` does not exist.
        pub(crate) fn mutate_pool<F>(pool_id: PoolId, mut cb: F) -> DispatchResult
        where
            F: FnMut(&mut Pool<BalanceOf<T>, MarketIdOf<T>>) -> DispatchResult,
        {
            <Pools<T>>::try_mutate(pool_id, |pool| {
                let pool = if let Some(el) = pool {
                    el
                } else {
                    return Err(Error::<T>::PoolDoesNotExist.into());
                };
                cb(pool)
            })
        }

        fn pool_weight_rslt(
            pool: &Pool<BalanceOf<T>, MarketIdOf<T>>,
            asset: &Asset<MarketIdOf<T>>,
        ) -> Result<u128, Error<T>> {
            pool.weights
                .as_ref()
                .ok_or(Error::<T>::PoolMissingWeight)?
                .get(asset)
                .cloned()
                .ok_or(Error::<T>::AssetNotBound)
        }

        /// Calculate the exit fee percentage for `pool`.
        fn calc_exit_fee(pool: &Pool<BalanceOf<T>, MarketIdOf<T>>) -> BalanceOf<T> {
            // We don't charge exit fees on closed or cleaned up pools (no need to punish LPs for
            // leaving the pool)!
            match pool.pool_status {
                PoolStatus::Active => T::ExitFee::get().saturated_into(),
                _ => 0u128.saturated_into(),
            }
        }
    }

    impl<T> Swaps<T::AccountId> for Pallet<T>
    where
        T: Config,
    {
        type Balance = BalanceOf<T>;
        type MarketId = MarketIdOf<T>;

        /// Creates an initial active pool.
        ///
        /// # Arguments
        ///
        /// * `who`: The account that is the creator of the pool. Must have enough
        ///     funds for each of the assets to cover the `amount`.
        /// * `assets`: The assets that are used in the pool.
        /// * `base_asset`: The base asset in a prediction market swap pool (usually a currency).
        /// * `market_id`: The market id of the market the pool belongs to.
        /// * `swap_fee`: The fee applied to each swap on a CPMM pool, specified as fixed-point
        ///     ratio (0.1 equals 10% swap fee)
        /// * `amount`: The amount of each asset added to the pool; **may** be `None` only if
        ///     `scoring_rule` is `RikiddoSigmoidFeeMarketEma`.
        /// * `weights`: These are the raw/denormalized weights (mandatory if scoring rule is CPMM).
        #[frame_support::transactional]
        fn create_pool(
            who: T::AccountId,
            assets: Vec<Asset<MarketIdOf<T>>>,
            base_asset: Asset<MarketIdOf<T>>,
            market_id: MarketIdOf<T>,
            swap_fee: Option<BalanceOf<T>>,
            amount: Option<BalanceOf<T>>,
            weights: Option<Vec<u128>>,
        ) -> Result<PoolId, DispatchError> {
            ensure!(assets.len() <= usize::from(T::MaxAssets::get()), Error::<T>::TooManyAssets);
            ensure!(assets.len() >= usize::from(T::MinAssets::get()), Error::<T>::TooFewAssets);
            ensure!(assets.contains(&base_asset), Error::<T>::BaseAssetNotFound);
            let next_pool_id = Self::inc_next_pool_id()?;
            let pool_shares_id = Self::pool_shares_id(next_pool_id);
            let pool_account = Self::pool_account_id(&next_pool_id);
            let mut map = BTreeMap::new();
            let market = T::MarketCommons::market(&market_id)?;
            let mut total_weight = 0;
            let amount_unwrapped = amount.unwrap_or_else(BalanceOf::<T>::zero);
            let mut sorted_assets = assets.clone();
            sorted_assets.sort();
            let has_duplicates = sorted_assets
                .iter()
                .zip(sorted_assets.iter().skip(1))
                .fold(false, |acc, (&x, &y)| acc || x == y);
            ensure!(!has_duplicates, Error::<T>::SomeIdenticalAssets);
            ensure!(market.scoring_rule == ScoringRule::CPMM, Error::<T>::InvalidScoringRule);

            ensure!(amount.is_some(), Error::<T>::InvalidAmountArgument);
            // `amount` must be larger than all minimum balances. As we deposit `amount`
            // liquidity shares, we must also ensure that `amount` is larger than the
            // existential deposit of the liquidity shares.
            ensure!(
                amount_unwrapped >= Self::min_balance_of_pool(next_pool_id, &assets),
                Error::<T>::InsufficientLiquidity
            );

            let swap_fee_unwrapped = swap_fee.ok_or(Error::<T>::InvalidFeeArgument)?;
            let total_fee = market
                .creator_fee
                .mul_floor(ZeitgeistBase::<u128>::get()?)
                .checked_add(swap_fee_unwrapped.try_into().map_err(|_| Error::<T>::SwapFeeTooHigh)?)
                .ok_or(Error::<T>::SwapFeeTooHigh)?;

            let total_fee_as_balance =
                <BalanceOf<T>>::try_from(total_fee).map_err(|_| Error::<T>::SwapFeeTooHigh)?;

            ensure!(total_fee_as_balance <= T::MaxSwapFee::get(), Error::<T>::SwapFeeTooHigh);
            ensure!(total_fee <= ZeitgeistBase::<u128>::get()?, Error::<T>::SwapFeeTooHigh);

            let weights_unwrapped = weights.ok_or(Error::<T>::InvalidWeightArgument)?;
            Self::check_provided_values_len_must_equal_assets_len(&assets, &weights_unwrapped)?;

            for (asset, weight) in assets.iter().copied().zip(weights_unwrapped) {
                let free_balance = T::AssetManager::free_balance(asset, &who);
                ensure!(free_balance >= amount_unwrapped, Error::<T>::InsufficientBalance);
                ensure!(weight >= T::MinWeight::get(), Error::<T>::BelowMinimumWeight);
                ensure!(weight <= T::MaxWeight::get(), Error::<T>::AboveMaximumWeight);
                map.insert(asset, weight);
                total_weight = total_weight.checked_add_res(&weight)?;
                T::AssetManager::transfer(asset, &who, &pool_account, amount_unwrapped)?;
            }

            ensure!(total_weight <= T::MaxTotalWeight::get(), Error::<T>::MaxTotalWeight);
            T::AssetManager::deposit(pool_shares_id, &who, amount_unwrapped)?;

            let pool = Pool {
                assets: sorted_assets,
                base_asset,
                market_id,
                pool_status: PoolStatus::Initialized,
                scoring_rule: market.scoring_rule,
                swap_fee,
                total_subsidy: None,
                total_weight: Some(total_weight),
                weights: Some(map),
            };

            <Pools<T>>::insert(next_pool_id, Some(pool.clone()));

            Self::deposit_event(Event::PoolCreate(
                CommonPoolEventParams { pool_id: next_pool_id, who },
                pool,
                amount_unwrapped,
                pool_account,
            ));

            Ok(next_pool_id)
        }

        fn close_pool(pool_id: PoolId) -> Result<Weight, DispatchError> {
            let asset_len =
                <Pools<T>>::try_mutate(pool_id, |pool| -> Result<u32, DispatchError> {
                    let pool = pool.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;
                    ensure!(
                        matches!(pool.pool_status, PoolStatus::Initialized | PoolStatus::Active),
                        Error::<T>::InvalidStateTransition,
                    );
                    pool.pool_status = PoolStatus::Closed;
                    Ok(pool.assets.len() as u32)
                })?;
            Self::deposit_event(Event::PoolClosed(pool_id));
            Ok(T::WeightInfo::close_pool(asset_len))
        }

        fn destroy_pool(pool_id: PoolId) -> Result<Weight, DispatchError> {
            let pool = Self::pool_by_id(pool_id)?;
            let pool_account = Self::pool_account_id(&pool_id);
            let asset_len = pool.assets.len() as u32;
            for asset in pool.assets.into_iter() {
                let amount = T::AssetManager::free_balance(asset, &pool_account);
                let missing = T::AssetManager::slash(asset, &pool_account, amount);
                debug_assert!(
                    missing.is_zero(),
                    "Could not slash all of the amount. asset {:?}, pool_account: {:?}, amount: \
                     {:?}.",
                    asset,
                    &pool_account,
                    amount,
                );
            }
            // NOTE: Currently we don't clean up accounts with pool_share_id.
            // TODO(#792): Remove pool_share_id asset for accounts! It may require storage migration.
            Pools::<T>::remove(pool_id);
            Self::deposit_event(Event::PoolDestroyed(pool_id));
            Ok(T::WeightInfo::destroy_pool(asset_len))
        }

        fn open_pool(pool_id: PoolId) -> Result<Weight, DispatchError> {
            Self::mutate_pool(pool_id, |pool| -> DispatchResult {
                ensure!(
                    pool.pool_status == PoolStatus::Initialized,
                    Error::<T>::InvalidStateTransition
                );
                pool.pool_status = PoolStatus::Active;
                Ok(())
            })?;
            let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolDoesNotExist)?;
            let asset_len = pool.assets.len() as u32;
            Self::deposit_event(Event::PoolActive(pool_id));
            Ok(T::WeightInfo::open_pool(asset_len))
        }

        /// Pool - Exit with exact pool amount
        ///
        /// Takes an asset from `pool_id` and transfers to `origin`. Differently from `pool_exit`,
        /// this method injects the exactly amount of `asset_amount` to `origin`.
        ///
        /// # Arguments
        ///
        /// * `who`: Liquidity Provider (LP). The account whose assets should be received.
        /// * `pool_id`: Unique pool identifier.
        /// * `asset`: Asset leaving the pool.
        /// * `asset_amount`: Asset amount that is leaving the pool.
        /// * `max_pool_amount`: The calculated amount of assets for the pool must be equal or
        /// greater than the given value.
        #[frame_support::transactional]
        fn pool_exit_with_exact_asset_amount(
            who: T::AccountId,
            pool_id: PoolId,
            asset: Asset<MarketIdOf<T>>,
            asset_amount: BalanceOf<T>,
            max_pool_amount: BalanceOf<T>,
        ) -> Result<Weight, DispatchError> {
            let pool = Self::pool_by_id(pool_id)?;
            Self::ensure_minimum_balance(pool_id, &pool, asset, asset_amount)?;
            let pool_ref = &pool;
            let who_clone = who.clone();

            let params = PoolExitWithExactAmountParams {
                asset,
                asset_amount: |_, _| Ok(asset_amount),
                bound: max_pool_amount,
                ensure_balance: |asset_balance: BalanceOf<T>| {
                    ensure!(
                        asset_amount <= asset_balance.bmul(T::MaxOutRatio::get())?,
                        Error::<T>::MaxOutRatio
                    );
                    Ok(())
                },
                pool_amount: |asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let pool_amount: BalanceOf<T> = crate::math::calc_pool_in_given_single_out(
                        asset_balance.saturated_into(),
                        Self::pool_weight_rslt(pool_ref, &asset)?,
                        total_supply.saturated_into(),
                        pool_ref
                            .total_weight
                            .ok_or(Error::<T>::PoolMissingWeight)?
                            .saturated_into(),
                        asset_amount.saturated_into(),
                        pool_ref.swap_fee.ok_or(Error::<T>::PoolMissingFee)?.saturated_into(),
                        T::ExitFee::get().saturated_into(),
                    )?
                    .saturated_into();
                    ensure!(pool_amount != Zero::zero(), Error::<T>::ZeroAmount);
                    ensure!(pool_amount <= max_pool_amount, Error::<T>::LimitIn);
                    Self::ensure_minimum_liquidity_shares(pool_id, &pool, pool_amount)?;
                    T::LiquidityMining::remove_shares(&who, &pool_ref.market_id, asset_amount);
                    Ok(pool_amount)
                },
                event: |evt| Self::deposit_event(Event::PoolExitWithExactAssetAmount(evt)),
                who: who_clone,
                pool_id,
                pool: pool_ref,
            };
            let weight = T::WeightInfo::pool_exit_with_exact_asset_amount();
            pool_exit_with_exact_amount::<_, _, _, _, T>(params).map(|_| weight)
        }

        /// Pool - Join with exact asset amount
        ///
        /// Joins an asset provided from `origin` to `pool_id`. Differently from `pool_join`,
        /// this method transfers the exactly amount of `asset_amount` to `pool_id`.
        ///
        /// # Arguments
        ///
        /// * `who`: Liquidity Provider (LP). The account whose assets should be received.
        /// * `pool_id`: Unique pool identifier.
        /// * `asset_in`: Asset entering the pool.
        /// * `asset_amount`: Asset amount that is entering the pool.
        /// * `min_pool_amount`: The calculated amount for the pool must be equal or greater
        /// than the given value.
        #[frame_support::transactional]
        fn pool_join_with_exact_asset_amount(
            who: T::AccountId,
            pool_id: PoolId,
            asset_in: Asset<MarketIdOf<T>>,
            asset_amount: BalanceOf<T>,
            min_pool_amount: BalanceOf<T>,
        ) -> Result<Weight, DispatchError> {
            ensure!(asset_amount != Zero::zero(), Error::<T>::ZeroAmount);
            let pool = Pallet::<T>::pool_by_id(pool_id)?;
            let pool_ref = &pool;
            let pool_account_id = Pallet::<T>::pool_account_id(&pool_id);
            let who_clone = who.clone();

            let params = PoolJoinWithExactAmountParams {
                asset: asset_in,
                asset_amount: |_, _| Ok(asset_amount),
                bound: min_pool_amount,
                pool_amount: move |asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let mul: BalanceOf<T> = asset_balance.bmul(T::MaxInRatio::get())?;
                    ensure!(asset_amount <= mul, Error::<T>::MaxInRatio);
                    let pool_amount: BalanceOf<T> = crate::math::calc_pool_out_given_single_in(
                        asset_balance.saturated_into(),
                        Self::pool_weight_rslt(pool_ref, &asset_in)?,
                        total_supply.saturated_into(),
                        pool_ref
                            .total_weight
                            .ok_or(Error::<T>::PoolMissingWeight)?
                            .saturated_into(),
                        asset_amount.saturated_into(),
                        pool_ref.swap_fee.ok_or(Error::<T>::PoolMissingFee)?.saturated_into(),
                    )?
                    .saturated_into();
                    ensure!(pool_amount >= min_pool_amount, Error::<T>::LimitOut);
                    T::LiquidityMining::add_shares(who.clone(), pool_ref.market_id, asset_amount);
                    Ok(pool_amount)
                },
                event: |evt| Self::deposit_event(Event::PoolJoinWithExactAssetAmount(evt)),
                who: who_clone,
                pool_account_id: &pool_account_id,
                pool_id,
                pool: pool_ref,
            };
            let weight = T::WeightInfo::pool_join_with_exact_asset_amount();
            pool_join_with_exact_amount::<_, _, _, T>(params).map(|_| weight)
        }

        fn pool(pool_id: PoolId) -> Result<Pool<Self::Balance, MarketIdOf<T>>, DispatchError> {
            Self::pool_by_id(pool_id)
        }

        /// Swap - Exact amount in
        ///
        /// Swaps a given `asset_amount_in` of the `asset_in/asset_out` pair to `pool_id`.
        ///
        /// # Arguments
        ///
        /// * `who`: The account whose assets should be transferred.
        /// * `pool_id`: Unique pool identifier.
        /// * `asset_in`: Asset entering the pool.
        /// * `asset_amount_in`: Amount that will be transferred from the provider to the pool.
        /// * `asset_out`: Asset leaving the pool.
        /// * `min_asset_amount_out`: Minimum asset amount that can leave the pool.
        /// * `max_price`: Market price must be equal or less than the provided value.
        /// * `handle_fees`: Optional parameter to override the swap fee
        #[allow(clippy::too_many_arguments)]
        fn swap_exact_amount_in(
            who: T::AccountId,
            pool_id: PoolId,
            asset_in: Asset<MarketIdOf<T>>,
            mut asset_amount_in: BalanceOf<T>,
            asset_out: Asset<MarketIdOf<T>>,
            min_asset_amount_out: Option<BalanceOf<T>>,
            max_price: Option<BalanceOf<T>>,
            handle_fees: bool,
        ) -> Result<Weight, DispatchError> {
            ensure!(
                min_asset_amount_out.is_some() || max_price.is_some(),
                Error::<T>::LimitMissing,
            );

            let pool = Pallet::<T>::pool_by_id(pool_id)?;
            let pool_account_id = Pallet::<T>::pool_account_id(&pool_id);
            let market = T::MarketCommons::market(&pool.market_id)?;
            ensure!(market.scoring_rule == ScoringRule::CPMM, Error::<T>::InvalidScoringRule);
            let creator_fee = market.creator_fee;
            let mut fees_handled = false;

            if asset_in == pool.base_asset && !handle_fees {
                Self::handle_creator_fees(
                    asset_amount_in,
                    asset_in,
                    pool.base_asset,
                    creator_fee,
                    market.creator.clone(),
                    who.clone(),
                    pool_id,
                    pool.market_id,
                );

                let fee_amount = creator_fee.mul_floor(asset_amount_in);
                asset_amount_in = asset_amount_in.saturating_sub(fee_amount);
                fees_handled = true;
            }

            ensure!(
                T::AssetManager::free_balance(asset_in, &who) >= asset_amount_in,
                Error::<T>::InsufficientBalance
            );

            let balance_before = T::AssetManager::free_balance(asset_out, &who);
            let params = SwapExactAmountParams {
                // TODO This probably doesn't need to be a closure.
                asset_amounts: || {
                    let balance_out = T::AssetManager::free_balance(asset_out, &pool_account_id);
                    let balance_in = T::AssetManager::free_balance(asset_in, &pool_account_id);
                    ensure!(
                        asset_amount_in <= balance_in.bmul(T::MaxInRatio::get())?,
                        Error::<T>::MaxInRatio
                    );
                    let swap_fee = if handle_fees {
                        0u128
                    } else {
                        pool.swap_fee.ok_or(Error::<T>::PoolMissingFee)?.saturated_into()
                    };
                    let asset_amount_out: BalanceOf<T> = crate::math::calc_out_given_in(
                        balance_in.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset_in)?,
                        balance_out.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset_out)?,
                        asset_amount_in.saturated_into(),
                        swap_fee,
                    )?
                    .saturated_into();

                    if let Some(maao) = min_asset_amount_out {
                        let asset_amount_out_check = if fees_handled {
                            asset_amount_out
                        } else {
                            asset_amount_out.saturating_sub(creator_fee.mul_floor(asset_amount_out))
                        };

                        ensure!(asset_amount_out_check >= maao, Error::<T>::LimitOut);
                    }

                    Self::ensure_minimum_balance(pool_id, &pool, asset_out, asset_amount_out)?;

                    Ok([asset_amount_in, asset_amount_out])
                },
                asset_bound: min_asset_amount_out,
                asset_in,
                asset_out,
                event: |evt| Self::deposit_event(Event::SwapExactAmountIn(evt)),
                max_price,
                pool_account_id: &pool_account_id,
                pool_id,
                pool: &pool,
                who: who.clone(),
            };
            swap_exact_amount::<_, _, T>(params)?;

            if !fees_handled && !handle_fees {
                let balance_after = T::AssetManager::free_balance(asset_out, &who);
                let asset_amount_out = balance_after.saturating_sub(balance_before);

                Self::handle_creator_fees(
                    asset_amount_out,
                    asset_out,
                    pool.base_asset,
                    creator_fee,
                    market.creator.clone(),
                    who.clone(),
                    pool_id,
                    pool.market_id,
                );
            }

            Ok(T::WeightInfo::swap_exact_amount_in_cpmm())
        }

        /// Swap - Exact amount out
        ///
        /// Swaps a given `asset_amount_out` of the `asset_in/asset_out` pair to `origin`.
        ///
        /// # Arguments
        ///
        /// * `who`: The account whose assets should be transferred.
        /// * `pool_id`: Unique pool identifier.
        /// * `asset_in`: Asset entering the pool.
        /// * `max_amount_asset_in`: Maximum asset amount that can enter the pool.
        /// * `asset_out`: Asset leaving the pool.
        /// * `asset_amount_out`: Amount that will be transferred from the pool to the provider.
        /// * `max_price`: Market price must be equal or less than the provided value.
        /// * `handle_fees`: Whether additional fees are handled or not (sets LP fee to 0)
        #[allow(clippy::too_many_arguments)]
        fn swap_exact_amount_out(
            who: T::AccountId,
            pool_id: PoolId,
            asset_in: Asset<MarketIdOf<T>>,
            max_asset_amount_in: Option<BalanceOf<T>>,
            asset_out: Asset<MarketIdOf<T>>,
            mut asset_amount_out: BalanceOf<T>,
            max_price: Option<BalanceOf<T>>,
            handle_fees: bool,
        ) -> Result<Weight, DispatchError> {
            let pool = Pallet::<T>::pool_by_id(pool_id)?;
            let pool_account_id = Pallet::<T>::pool_account_id(&pool_id);
            ensure!(max_asset_amount_in.is_some() || max_price.is_some(), Error::<T>::LimitMissing);
            Self::ensure_minimum_balance(pool_id, &pool, asset_out, asset_amount_out)?;
            let market = T::MarketCommons::market(&pool.market_id)?;
            ensure!(market.scoring_rule == ScoringRule::CPMM, Error::<T>::InvalidScoringRule);
            let creator_fee = market.creator_fee;
            let mut fee_amount = BalanceOf::<T>::zero();

            let to_adjust_in_value = if asset_in == pool.base_asset {
                // Can't adjust the value inside the anonymous function for asset_amounts
                true
            } else {
                fee_amount = creator_fee.mul_floor(asset_amount_out);
                asset_amount_out = asset_amount_out.saturating_add(fee_amount);
                false
            };

            let params = SwapExactAmountParams {
                asset_amounts: || {
                    let balance_out = T::AssetManager::free_balance(asset_out, &pool_account_id);
                    ensure!(
                        asset_amount_out <= balance_out.bmul(T::MaxOutRatio::get(),)?,
                        Error::<T>::MaxOutRatio,
                    );

                    let balance_in = T::AssetManager::free_balance(asset_in, &pool_account_id);
                    let swap_fee = if handle_fees {
                        0u128
                    } else {
                        pool.swap_fee.ok_or(Error::<T>::PoolMissingFee)?.saturated_into()
                    };
                    let asset_amount_in: BalanceOf<T> = crate::math::calc_in_given_out(
                        balance_in.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset_in)?,
                        balance_out.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset_out)?,
                        asset_amount_out.saturated_into(),
                        swap_fee,
                    )?
                    .saturated_into();

                    if asset_in == pool.base_asset && !handle_fees && to_adjust_in_value {
                        Self::handle_creator_fees(
                            asset_amount_in,
                            asset_in,
                            pool.base_asset,
                            creator_fee,
                            market.creator.clone(),
                            who.clone(),
                            pool_id,
                            pool.market_id,
                        );
                    }

                    if let Some(maai) = max_asset_amount_in {
                        let asset_amount_in_check = if to_adjust_in_value {
                            let fee_amount = creator_fee.mul_floor(asset_amount_in);
                            asset_amount_in.saturating_add(fee_amount)
                        } else {
                            asset_amount_in
                        };

                        ensure!(asset_amount_in_check <= maai, Error::<T>::LimitIn);
                    }

                    Ok([asset_amount_in, asset_amount_out])
                },
                asset_bound: max_asset_amount_in,
                asset_in,
                asset_out,
                event: |evt| Self::deposit_event(Event::SwapExactAmountOut(evt)),
                max_price,
                pool_account_id: &pool_account_id,
                pool_id,
                pool: &pool,
                who: who.clone(),
            };
            swap_exact_amount::<_, _, T>(params)?;

            if !to_adjust_in_value && !handle_fees {
                asset_amount_out = asset_amount_out.saturating_sub(fee_amount);

                Self::handle_creator_fees(
                    asset_amount_out,
                    asset_out,
                    pool.base_asset,
                    creator_fee,
                    market.creator.clone(),
                    who.clone(),
                    pool_id,
                    pool.market_id,
                );
            }

            Ok(T::WeightInfo::swap_exact_amount_out_cpmm())
        }
    }
}
