// Copyright 2022-2024 Forecasting Technologies LTD.
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
mod types;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{
        events::{CommonPoolEventParams, PoolAssetEvent, PoolAssetsEvent, SwapEvent},
        types::{Pool, PoolStatus},
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
        ensure,
        pallet_prelude::{OptionQuery, StorageMap, StorageValue, ValueQuery},
        require_transactional,
        traits::{Get, IsType, StorageVersion},
        transactional,
        weights::Weight,
        Blake2_128Concat, PalletError, PalletId, Parameter,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use orml_traits::MultiCurrency;
    use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;
    use sp_arithmetic::traits::{Saturating, Zero};
    use sp_runtime::{
        traits::{AccountIdConversion, MaybeSerializeDeserialize, Member},
        DispatchError, DispatchResult, RuntimeDebug, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        constants::{BASE, CENT},
        math::{
            checked_ops_res::{CheckedAddRes, CheckedMulRes},
            fixed::FixedMul,
        },
        traits::{PoolSharesId, Swaps},
        types::PoolId,
    };

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type AssetOf<T> =
        <<T as Config>::AssetManager as MultiCurrency<AccountIdOf<T>>>::CurrencyId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::AssetManager as MultiCurrency<AccountIdOf<T>>>::Balance;
    pub(crate) type PoolOf<T> = Pool<AssetOf<T>, BalanceOf<T>>;

    const MIN_BALANCE: u128 = CENT;
    pub(crate) const MAX_IN_RATIO: u128 = BASE / 3 + 1;
    pub(crate) const MAX_OUT_RATIO: u128 = BASE / 3 + 1;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Exchanges an LP's (liquidity provider's) pool shares for a proportionate amount of each
        /// of the pool's assets. The assets received are distributed according to the LP's
        /// percentage ownership of the pool.
        ///
        /// # Arguments
        ///
        /// * `pool_id`: The ID of the pool to withdraw from.
        /// * `pool_amount`: The amount of pool shares to burn.
        /// * `min_assets_out`: List of lower bounds on the assets received. The transaction is
        ///   rolled back if any of the specified lower bounds are violated.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)` where `n` is the number of assets in the specified pool.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::pool_exit(
            min_assets_out.len().min(T::MaxAssets::get().into()) as u32,
        ))]
        #[transactional]
        pub fn pool_exit(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: PoolId,
            #[pallet::compact] pool_amount: BalanceOf<T>,
            min_assets_out: Vec<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_pool_exit(who, pool_id, pool_amount, min_assets_out)
        }

        /// See [`zeitgeist_primitives::traits::Swaps::pool_exit_with_exact_asset_amount`].
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::pool_exit_with_exact_asset_amount())]
        #[transactional]
        pub fn pool_exit_with_exact_asset_amount(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: PoolId,
            asset_out: AssetOf<T>,
            #[pallet::compact] asset_amount: BalanceOf<T>,
            #[pallet::compact] max_pool_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_pool_exit_with_exact_asset_amount(
                who,
                pool_id,
                asset_out,
                asset_amount,
                max_pool_amount,
            )
        }

        /// Exchanges an exact amount of an LP's (liquidity provider's) pool shares for a
        /// proportionate amount of _one_ of the pool's assets. The assets received are distributed
        /// according to the LP's percentage ownership of the pool.
        ///
        /// # Arguments
        ///
        /// * `pool_id`: The ID of the pool to withdraw from.
        /// * `asset`: The asset received by the LP.
        /// * `asset_amount`: The amount of `asset` leaving the pool.
        /// * `pool_amount`: Pool amount that is entering the pool.
        /// * `min_asset_amount`: The minimum amount the LP asks to receive. The transaction is
        ///   rolled back if this bound is violated.
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
            asset: AssetOf<T>,
            #[pallet::compact] pool_amount: BalanceOf<T>,
            #[pallet::compact] min_asset_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_pool_exit_with_exact_pool_amount(
                who,
                pool_id,
                asset,
                pool_amount,
                min_asset_amount,
            )
        }

        /// Exchanges a proportional amount of each asset of the pool for pool shares.
        ///
        /// # Arguments
        ///
        /// * `pool_id`: The ID of the pool to join.
        /// * `pool_amount`: The amount of LP shares for this pool that should be minted to the
        ///   provider.
        /// * `max_assets_in`: List of upper bounds on the assets to move to the pool. The
        ///   transaction is rolled back if any of the specified lower bounds are violated.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)` where `n` is the number of assets in the specified pool
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
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_pool_join(who, pool_id, pool_amount, max_assets_in)
        }

        /// See [`zeitgeist_primitives::traits::Swaps::pool_join_with_exact_asset_amount`].
        ///
        /// # Weight
        ///
        /// Complexity: O(1)
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::pool_join_with_exact_asset_amount())]
        #[transactional]
        pub fn pool_join_with_exact_asset_amount(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: PoolId,
            asset_in: AssetOf<T>,
            #[pallet::compact] asset_amount: BalanceOf<T>,
            #[pallet::compact] min_pool_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_pool_join_with_exact_asset_amount(
                who,
                pool_id,
                asset_in,
                asset_amount,
                min_pool_amount,
            )
        }

        /// Exchanges an LP's (liquidity provider's) holdings of _one_ of the assets in the pool for
        /// an exact amount of pool shares.
        ///
        /// # Arguments
        ///
        /// * `pool_id`: The ID of the pool to withdraw from.
        /// * `asset`: The asset entering the pool.
        /// * `pool_amount`: Asset amount that is entering the pool.
        /// * `max_asset_amount`: The maximum amount of `asset` that the LP is willing to pay. The
        ///   transaction is rolled back if this bound is violated.
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
            asset: AssetOf<T>,
            #[pallet::compact] pool_amount: BalanceOf<T>,
            #[pallet::compact] max_asset_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_pool_join_with_exact_pool_amount(
                who,
                pool_id,
                asset,
                pool_amount,
                max_asset_amount,
            )
        }

        /// See [`zeitgeist_primitives::traits::Swaps::swap_exact_amount_in`].
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::swap_exact_amount_in())]
        #[transactional]
        pub fn swap_exact_amount_in(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: PoolId,
            asset_in: AssetOf<T>,
            #[pallet::compact] asset_amount_in: BalanceOf<T>,
            asset_out: AssetOf<T>,
            min_asset_amount_out: Option<BalanceOf<T>>,
            max_price: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_swap_exact_amount_in(
                who,
                pool_id,
                asset_in,
                asset_amount_in,
                asset_out,
                min_asset_amount_out,
                max_price,
            )
        }

        /// See [`zeitgeist_primitives::traits::Swaps::swap_exact_amount_out`].
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(10)]
        #[pallet::weight(T::WeightInfo::swap_exact_amount_out())]
        #[transactional]
        pub fn swap_exact_amount_out(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: PoolId,
            asset_in: AssetOf<T>,
            max_asset_amount_in: Option<BalanceOf<T>>,
            asset_out: AssetOf<T>,
            #[pallet::compact] asset_amount_out: BalanceOf<T>,
            max_price: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_swap_exact_amount_out(
                who,
                pool_id,
                asset_in,
                max_asset_amount_in,
                asset_out,
                asset_amount_out,
                max_price,
            )
        }

        /// Forcibly withdraw an LPs share. All parameters as in `exit`, except that `who` is the LP
        /// whose shares are withdrawn.
        ///
        /// Used in the migration from swaps to neo-swaps. Deprecated and scheduled for removal in
        /// v0.5.3.
        #[pallet::call_index(11)]
        #[pallet::weight(T::WeightInfo::pool_exit(
            min_assets_out.len().min(T::MaxAssets::get().into()) as u32,
        ))]
        #[transactional]
        pub fn force_pool_exit(
            origin: OriginFor<T>,
            who: T::AccountId,
            #[pallet::compact] pool_id: PoolId,
            #[pallet::compact] pool_amount: BalanceOf<T>,
            min_assets_out: Vec<BalanceOf<T>>,
        ) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            Self::do_pool_exit(who, pool_id, pool_amount, min_assets_out)
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type AssetManager: MultiCurrency<Self::AccountId, CurrencyId = Self::Asset>;

        type Asset: Parameter
            + Member
            + Copy
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + Ord
            + TypeInfo
            + PoolSharesId<PoolId>;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type WeightInfo: WeightInfoZeitgeist;

        /// The fee for exiting a pool.
        #[pallet::constant]
        type ExitFee: Get<BalanceOf<Self>>;

        /// The maximum number of assets allowed in a single pool.
        #[pallet::constant]
        type MaxAssets: Get<u16>;

        /// The maximum allowed swap fee.
        #[pallet::constant]
        type MaxSwapFee: Get<BalanceOf<Self>>;

        /// The maximum total weight of assets in a pool.
        #[pallet::constant]
        type MaxTotalWeight: Get<BalanceOf<Self>>;

        /// The maximum weight of each individual asset in a pool.
        #[pallet::constant]
        type MaxWeight: Get<BalanceOf<Self>>;

        /// The minimum number of assets allowed in a single pool.
        #[pallet::constant]
        type MinAssets: Get<u16>;

        /// The minimum weight of each individual asset in a pool.
        #[pallet::constant]
        type MinWeight: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The weight of an asset in a CPMM swap pool is greater than the upper weight cap.
        #[codec(index = 0)]
        AboveMaximumWeight,
        /// The asset in question could not be found within the pool.
        #[codec(index = 2)]
        AssetNotInPool,
        /// The spot price of an asset pair was greater than the specified limit.
        #[codec(index = 4)]
        BadLimitPrice,
        /// The weight of an asset in a CPMM swap pool is lower than the upper weight cap.
        #[codec(index = 5)]
        BelowMinimumWeight,
        /// Some funds could not be transferred due to a too low balance.
        #[codec(index = 6)]
        InsufficientBalance,
        /// Liquidity provided to new CPMM pool is less than the minimum allowed balance.
        #[codec(index = 7)]
        InsufficientLiquidity,
        /// Dispatch called on pool with invalid status.
        #[codec(index = 10)]
        InvalidPoolStatus,
        /// A function was called for a swaps pool that does not fulfill the state requirement.
        #[codec(index = 11)]
        InvalidStateTransition,
        /// A transferal of funds into a swaps pool was above a threshold specified by the sender.
        #[codec(index = 13)]
        LimitIn,
        /// No limit was specified for a swap.
        #[codec(index = 15)]
        LimitMissing,
        /// A transferal of funds out of a swaps pool was below a threshold specified by the
        /// receiver.
        #[codec(index = 16)]
        LimitOut,
        /// The custom math library yielded an invalid result (most times unexpected zero value).
        #[codec(index = 17)]
        MathApproximation,
        /// The proportion of an asset added into a pool in comparison to the amount
        /// of that asset in the pool is above the threshold specified by a constant.
        #[codec(index = 18)]
        MaxInRatio,
        /// The proportion of an asset taken from a pool in comparison to the amount
        /// of that asset in the pool is above the threshold specified by a constant.
        #[codec(index = 19)]
        MaxOutRatio,
        /// The total weight of all assets within a CPMM pool is above a threshold specified
        /// by a constant.
        #[codec(index = 20)]
        MaxTotalWeight,
        /// The pool in question does not exist.
        #[codec(index = 21)]
        PoolDoesNotExist,
        /// A pool balance dropped below the allowed minimum.
        #[codec(index = 22)]
        PoolDrain,
        /// The pool in question is inactive.
        #[codec(index = 23)]
        PoolIsNotActive,
        /// Two vectors do not have the same length (usually CPMM pool assets and weights).
        #[codec(index = 27)]
        ProvidedValuesLenMustEqualAssetsLen,
        /// The swap fee is higher than the allowed maximum.
        #[codec(index = 29)]
        SwapFeeTooHigh,
        /// Tried to create a pool that has less assets than the lower threshold specified by
        /// a constant.
        #[codec(index = 30)]
        TooFewAssets,
        /// Tried to create a pool that has more assets than the upper threshold specified by
        /// a constant.
        #[codec(index = 31)]
        TooManyAssets,
        /// Tried to create a pool with at least two identical assets.
        #[codec(index = 32)]
        SomeIdenticalAssets,
        /// Some amount in a transaction equals zero.
        #[codec(index = 35)]
        ZeroAmount,
        /// An unexpected error occurred. This is the result of faulty pallet logic and should be
        /// reported to the pallet maintainers.
        #[codec(index = 36)]
        Unexpected(UnexpectedError),
    }

    #[derive(Decode, Encode, Eq, PartialEq, PalletError, RuntimeDebug, TypeInfo)]
    pub enum UnexpectedError {
        StorageOverflow,
    }

    impl<T> From<UnexpectedError> for Error<T> {
        fn from(error: UnexpectedError) -> Error<T> {
            Error::<T>::Unexpected(error)
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// Share holder rewards were distributed.
        #[codec(index = 0)]
        DistributeShareHolderRewards {
            pool_id: PoolId,
            num_accounts_rewarded: u64,
            amount: BalanceOf<T>,
        },
        /// A new pool has been created.
        #[codec(index = 1)]
        PoolCreate {
            common: CommonPoolEventParams<AccountIdOf<T>>,
            pool: PoolOf<T>,
            pool_amount: BalanceOf<T>,
            pool_account: T::AccountId,
        },
        /// A pool was closed.
        #[codec(index = 2)]
        PoolClosed { pool_id: PoolId },
        /// A pool was cleaned up.
        #[codec(index = 3)]
        PoolCleanedUp { pool_id: PoolId },
        /// A pool was opened.
        #[codec(index = 4)]
        PoolActive { pool_id: PoolId },
        /// Someone has exited a pool.
        #[codec(index = 5)]
        PoolExit(PoolAssetsEvent<AccountIdOf<T>, AssetOf<T>, BalanceOf<T>>),
        /// Exits a pool given an exact amount of an asset.
        #[codec(index = 6)]
        PoolExitWithExactAssetAmount(PoolAssetEvent<AccountIdOf<T>, AssetOf<T>, BalanceOf<T>>),
        /// Exits a pool given an exact pool's amount.
        #[codec(index = 7)]
        PoolExitWithExactPoolAmount(PoolAssetEvent<AccountIdOf<T>, AssetOf<T>, BalanceOf<T>>),
        /// Someone has joined a pool.
        #[codec(index = 8)]
        PoolJoin(PoolAssetsEvent<AccountIdOf<T>, AssetOf<T>, BalanceOf<T>>),
        /// Joins a pool given an exact amount of an asset.
        #[codec(index = 9)]
        PoolJoinWithExactAssetAmount(PoolAssetEvent<AccountIdOf<T>, AssetOf<T>, BalanceOf<T>>),
        /// Joins a pool given an exact pool's amount.
        #[codec(index = 10)]
        PoolJoinWithExactPoolAmount(PoolAssetEvent<AccountIdOf<T>, AssetOf<T>, BalanceOf<T>>),
        /// Pool was manually destroyed.
        #[codec(index = 11)]
        PoolDestroyed { pool_id: PoolId },
        /// An exact amount of an asset is entering the pool.
        #[codec(index = 13)]
        SwapExactAmountIn(SwapEvent<AccountIdOf<T>, AssetOf<T>, BalanceOf<T>>),
        /// An exact amount of an asset is leaving the pool.
        #[codec(index = 14)]
        SwapExactAmountOut(SwapEvent<AccountIdOf<T>, AssetOf<T>, BalanceOf<T>>),
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub(crate) type Pools<T: Config> =
        StorageMap<_, Blake2_128Concat, PoolId, PoolOf<T>, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_pool_id)]
    pub(crate) type NextPoolId<T> = StorageValue<_, PoolId, ValueQuery>;

    impl<T: Config> Pallet<T> {
        #[require_transactional]
        fn do_pool_exit(
            who: T::AccountId,
            pool_id: PoolId,
            pool_amount: BalanceOf<T>,
            min_assets_out: Vec<BalanceOf<T>>,
        ) -> DispatchResult {
            ensure!(pool_amount != Zero::zero(), Error::<T>::ZeroAmount);
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
                    // If transferring to `who` triggers the existential deposit, burn the tokens
                    // instead.
                    let new_balance =
                        T::AssetManager::free_balance(asset, &who).checked_add_res(&amount)?;
                    if new_balance >= T::AssetManager::minimum_balance(asset) {
                        ensure!(amount >= amount_bound, Error::<T>::LimitOut);
                        T::AssetManager::transfer(asset, &pool_account_id, &who, amount)?;
                    } else {
                        ensure!(amount_bound.is_zero(), Error::<T>::LimitOut);
                        T::AssetManager::withdraw(asset, &pool_account_id, amount)?;
                    }
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
                who: who.clone(),
            };

            crate::utils::pool::<_, _, _, _, T>(params)
        }

        #[require_transactional]
        fn do_pool_exit_with_exact_pool_amount(
            who: AccountIdOf<T>,
            pool_id: PoolId,
            asset: AssetOf<T>,
            pool_amount: BalanceOf<T>,
            min_asset_amount: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(pool_amount != Zero::zero(), Error::<T>::ZeroAmount);
            let pool = Self::pool_by_id(pool_id)?;
            let pool_ref = &pool;
            Self::ensure_minimum_liquidity_shares(pool_id, &pool, pool_amount)?;

            let params = PoolExitWithExactAmountParams {
                asset,
                asset_amount: |asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let mul = total_supply.bmul(MAX_IN_RATIO.saturated_into())?;
                    ensure!(pool_amount <= mul, Error::<T>::MaxInRatio);
                    let asset_amount: BalanceOf<T> = crate::math::calc_single_out_given_pool_in(
                        asset_balance.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset)?.saturated_into(),
                        total_supply.saturated_into(),
                        pool.total_weight.saturated_into(),
                        pool_amount.saturated_into(),
                        pool.swap_fee.saturated_into(),
                        T::ExitFee::get().saturated_into(),
                    )?
                    .saturated_into();
                    ensure!(asset_amount != Zero::zero(), Error::<T>::ZeroAmount);
                    ensure!(asset_amount >= min_asset_amount, Error::<T>::LimitOut);
                    ensure!(
                        asset_amount <= asset_balance.bmul(MAX_OUT_RATIO.saturated_into())?,
                        Error::<T>::MaxOutRatio
                    );
                    Self::ensure_minimum_balance(pool_id, &pool, asset, asset_amount)?;
                    Ok(asset_amount)
                },
                bound: min_asset_amount,
                ensure_balance: |_| Ok(()),
                event: |evt| Self::deposit_event(Event::PoolExitWithExactPoolAmount(evt)),
                who,
                pool_amount: |_, _| Ok(pool_amount),
                pool_id,
                pool: pool_ref,
            };

            pool_exit_with_exact_amount::<_, _, _, _, T>(params)
        }

        #[require_transactional]
        fn do_pool_join(
            who: AccountIdOf<T>,
            pool_id: PoolId,
            pool_amount: BalanceOf<T>,
            max_assets_in: Vec<BalanceOf<T>>,
        ) -> DispatchResult {
            ensure!(pool_amount != Zero::zero(), Error::<T>::ZeroAmount);
            let pool = Self::pool_by_id(pool_id)?;
            ensure!(pool.status == PoolStatus::Open, Error::<T>::InvalidPoolStatus);
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
                    Ok(())
                },
                transfer_pool: || Self::mint_pool_shares(pool_id, &who, pool_amount),
                fee: |_| Ok(0u128.saturated_into()),
                who: who.clone(),
            };

            crate::utils::pool::<_, _, _, _, T>(params)
        }

        #[require_transactional]
        fn do_pool_exit_with_exact_asset_amount(
            who: AccountIdOf<T>,
            pool_id: PoolId,
            asset: AssetOf<T>,
            asset_amount: BalanceOf<T>,
            max_pool_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let pool = Self::pool_by_id(pool_id)?;
            Self::ensure_minimum_balance(pool_id, &pool, asset, asset_amount)?;
            let pool_ref = &pool;

            let params = PoolExitWithExactAmountParams {
                asset,
                asset_amount: |_, _| Ok(asset_amount),
                bound: max_pool_amount,
                ensure_balance: |asset_balance: BalanceOf<T>| {
                    ensure!(
                        asset_amount <= asset_balance.bmul(MAX_OUT_RATIO.saturated_into())?,
                        Error::<T>::MaxOutRatio
                    );
                    Ok(())
                },
                pool_amount: |asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let pool_amount: BalanceOf<T> = crate::math::calc_pool_in_given_single_out(
                        asset_balance.saturated_into(),
                        Self::pool_weight_rslt(pool_ref, &asset)?.saturated_into(),
                        total_supply.saturated_into(),
                        pool_ref.total_weight.saturated_into(),
                        asset_amount.saturated_into(),
                        pool_ref.swap_fee.saturated_into(),
                        T::ExitFee::get().saturated_into(),
                    )?
                    .saturated_into();
                    ensure!(pool_amount != Zero::zero(), Error::<T>::ZeroAmount);
                    ensure!(pool_amount <= max_pool_amount, Error::<T>::LimitIn);
                    Self::ensure_minimum_liquidity_shares(pool_id, &pool, pool_amount)?;
                    Ok(pool_amount)
                },
                event: |evt| Self::deposit_event(Event::PoolExitWithExactAssetAmount(evt)),
                who,
                pool_id,
                pool: pool_ref,
            };

            pool_exit_with_exact_amount::<_, _, _, _, T>(params)
        }

        #[require_transactional]
        fn do_pool_join_with_exact_asset_amount(
            who: AccountIdOf<T>,
            pool_id: PoolId,
            asset_in: AssetOf<T>,
            asset_amount: BalanceOf<T>,
            min_pool_amount: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(asset_amount != Zero::zero(), Error::<T>::ZeroAmount);
            let pool = Pallet::<T>::pool_by_id(pool_id)?;
            let pool_ref = &pool;
            let pool_account_id = Pallet::<T>::pool_account_id(&pool_id);

            let params = PoolJoinWithExactAmountParams {
                asset: asset_in,
                asset_amount: |_, _| Ok(asset_amount),
                bound: min_pool_amount,
                pool_amount: move |asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let mul = asset_balance.bmul(MAX_IN_RATIO.saturated_into())?;
                    ensure!(asset_amount <= mul, Error::<T>::MaxInRatio);
                    let pool_amount: BalanceOf<T> = crate::math::calc_pool_out_given_single_in(
                        asset_balance.saturated_into(),
                        Self::pool_weight_rslt(pool_ref, &asset_in)?.saturated_into(),
                        total_supply.saturated_into(),
                        pool_ref.total_weight.saturated_into(),
                        asset_amount.saturated_into(),
                        pool_ref.swap_fee.saturated_into(),
                    )?
                    .saturated_into();
                    ensure!(pool_amount >= min_pool_amount, Error::<T>::LimitOut);
                    Ok(pool_amount)
                },
                event: |evt| Self::deposit_event(Event::PoolJoinWithExactAssetAmount(evt)),
                who,
                pool_account_id: &pool_account_id,
                pool_id,
                pool: pool_ref,
            };

            pool_join_with_exact_amount::<_, _, _, T>(params)
        }

        #[require_transactional]
        fn do_pool_join_with_exact_pool_amount(
            who: AccountIdOf<T>,
            pool_id: PoolId,
            asset: AssetOf<T>,
            pool_amount: BalanceOf<T>,
            max_asset_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let pool = Pallet::<T>::pool_by_id(pool_id)?;
            let pool_account_id = Pallet::<T>::pool_account_id(&pool_id);

            let params = PoolJoinWithExactAmountParams {
                asset,
                asset_amount: |asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let mul = total_supply.bmul(MAX_OUT_RATIO.saturated_into())?;
                    ensure!(pool_amount <= mul, Error::<T>::MaxOutRatio);
                    let asset_amount: BalanceOf<T> = crate::math::calc_single_in_given_pool_out(
                        asset_balance.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset)?.saturated_into(),
                        total_supply.saturated_into(),
                        pool.total_weight.saturated_into(),
                        pool_amount.saturated_into(),
                        pool.swap_fee.saturated_into(),
                    )?
                    .saturated_into();
                    ensure!(asset_amount != Zero::zero(), Error::<T>::ZeroAmount);
                    ensure!(asset_amount <= max_asset_amount, Error::<T>::LimitIn);
                    ensure!(
                        asset_amount
                            <= asset_balance.checked_mul_res(&MAX_IN_RATIO.saturated_into())?,
                        Error::<T>::MaxInRatio
                    );
                    Ok(asset_amount)
                },
                bound: max_asset_amount,
                event: |evt| Self::deposit_event(Event::PoolJoinWithExactPoolAmount(evt)),
                pool_account_id: &pool_account_id,
                pool_amount: |_, _| Ok(pool_amount),
                pool_id,
                pool: &pool,
                who,
            };

            pool_join_with_exact_amount::<_, _, _, T>(params)
        }

        #[require_transactional]
        fn do_swap_exact_amount_in(
            who: T::AccountId,
            pool_id: PoolId,
            asset_in: AssetOf<T>,
            asset_amount_in: BalanceOf<T>,
            asset_out: AssetOf<T>,
            min_asset_amount_out: Option<BalanceOf<T>>,
            max_price: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            ensure!(
                min_asset_amount_out.is_some() || max_price.is_some(),
                Error::<T>::LimitMissing,
            );
            let pool = Pallet::<T>::pool_by_id(pool_id)?;
            let pool_account_id = Pallet::<T>::pool_account_id(&pool_id);
            ensure!(
                T::AssetManager::free_balance(asset_in, &who) >= asset_amount_in,
                Error::<T>::InsufficientBalance
            );

            let params = SwapExactAmountParams {
                // TODO(#1215): This probably doesn't need to be a closure.
                asset_amounts: || {
                    let balance_out = T::AssetManager::free_balance(asset_out, &pool_account_id);
                    let balance_in = T::AssetManager::free_balance(asset_in, &pool_account_id);
                    ensure!(
                        asset_amount_in <= balance_in.bmul(MAX_IN_RATIO.saturated_into())?,
                        Error::<T>::MaxInRatio
                    );
                    let asset_amount_out: BalanceOf<T> = crate::math::calc_out_given_in(
                        balance_in.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset_in)?.saturated_into(),
                        balance_out.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset_out)?.saturated_into(),
                        asset_amount_in.saturated_into(),
                        pool.swap_fee.saturated_into(),
                    )?
                    .saturated_into();

                    if let Some(maao) = min_asset_amount_out {
                        ensure!(asset_amount_out >= maao, Error::<T>::LimitOut);
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

            swap_exact_amount::<_, _, T>(params)
        }

        #[require_transactional]
        fn do_swap_exact_amount_out(
            who: AccountIdOf<T>,
            pool_id: PoolId,
            asset_in: AssetOf<T>,
            max_asset_amount_in: Option<BalanceOf<T>>,
            asset_out: AssetOf<T>,
            asset_amount_out: BalanceOf<T>,
            max_price: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let pool = Pallet::<T>::pool_by_id(pool_id)?;
            let pool_account_id = Pallet::<T>::pool_account_id(&pool_id);
            ensure!(max_asset_amount_in.is_some() || max_price.is_some(), Error::<T>::LimitMissing);
            Self::ensure_minimum_balance(pool_id, &pool, asset_out, asset_amount_out)?;

            let params = SwapExactAmountParams {
                asset_amounts: || {
                    let balance_out = T::AssetManager::free_balance(asset_out, &pool_account_id);
                    ensure!(
                        asset_amount_out <= balance_out.bmul(MAX_OUT_RATIO.saturated_into())?,
                        Error::<T>::MaxOutRatio,
                    );

                    let balance_in = T::AssetManager::free_balance(asset_in, &pool_account_id);
                    let asset_amount_in: BalanceOf<T> = crate::math::calc_in_given_out(
                        balance_in.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset_in)?.saturated_into(),
                        balance_out.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset_out)?.saturated_into(),
                        asset_amount_out.saturated_into(),
                        pool.swap_fee.saturated_into(),
                    )?
                    .saturated_into();

                    if let Some(maai) = max_asset_amount_in {
                        ensure!(asset_amount_in <= maai, Error::<T>::LimitIn);
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
                who,
            };

            swap_exact_amount::<_, _, T>(params)
        }

        pub fn get_spot_price(
            pool_id: &PoolId,
            asset_in: &AssetOf<T>,
            asset_out: &AssetOf<T>,
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

            let swap_fee = if with_fees { pool.swap_fee } else { BalanceOf::<T>::zero() };

            Ok(crate::math::calc_spot_price(
                balance_in.saturated_into(),
                in_weight.saturated_into(),
                balance_out.saturated_into(),
                out_weight.saturated_into(),
                swap_fee.saturated_into(),
            )?
            .saturated_into())
        }

        #[inline]
        pub fn pool_account_id(pool_id: &PoolId) -> T::AccountId {
            T::PalletId::get().into_sub_account_truncating((*pool_id).saturated_into::<u128>())
        }

        /// The minimum allowed balance of `asset` in a liquidity pool.
        pub(crate) fn min_balance(asset: AssetOf<T>) -> BalanceOf<T> {
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
        pub(crate) fn min_balance_of_pool(pool_id: PoolId, assets: &[AssetOf<T>]) -> BalanceOf<T> {
            assets
                .iter()
                .map(|asset| Self::min_balance(*asset))
                .max()
                .unwrap_or_else(|| MIN_BALANCE.saturated_into())
                .max(Self::min_balance(Self::pool_shares_id(pool_id)))
        }

        fn ensure_minimum_liquidity_shares(
            pool_id: PoolId,
            pool: &PoolOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            if pool.status == PoolStatus::Closed {
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
            pool: &PoolOf<T>,
            asset: AssetOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            // No need to prevent a clean pool from getting drained.
            if pool.status == PoolStatus::Closed {
                return Ok(());
            }
            let pool_account = Self::pool_account_id(&pool_id);
            let balance = T::AssetManager::free_balance(asset, &pool_account);
            let max_withdraw = balance.saturating_sub(Self::min_balance(asset).saturated_into());
            ensure!(amount <= max_withdraw, Error::<T>::PoolDrain);
            Ok(())
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
            T::AssetManager::withdraw(shares_id, from, amount)?;
            Ok(())
        }

        #[inline]
        pub(crate) fn check_provided_values_len_must_equal_assets_len<U>(
            assets: &[AssetOf<T>],
            provided_values: &[U],
        ) -> Result<(), Error<T>>
        where
            T: Config,
        {
            ensure!(
                assets.len() == provided_values.len(),
                Error::<T>::ProvidedValuesLenMustEqualAssetsLen
            );
            Ok(())
        }

        pub(crate) fn ensure_pool_is_active(pool: &PoolOf<T>) -> DispatchResult {
            match pool.status {
                PoolStatus::Open => Ok(()),
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

        pub(crate) fn pool_shares_id(pool_id: PoolId) -> AssetOf<T> {
            T::Asset::pool_shares_id(pool_id)
        }

        pub fn pool_by_id(pool_id: PoolId) -> Result<PoolOf<T>, DispatchError>
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

        /// Mutates a stored pool.
        pub(crate) fn mutate_pool<F>(pool_id: PoolId, mut cb: F) -> DispatchResult
        where
            F: FnMut(&mut PoolOf<T>) -> DispatchResult,
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
            pool: &PoolOf<T>,
            asset: &AssetOf<T>,
        ) -> Result<BalanceOf<T>, Error<T>> {
            pool.weights.get(asset).cloned().ok_or(Error::<T>::AssetNotInPool)
        }

        /// Calculate the exit fee percentage for `pool`.
        fn calc_exit_fee(pool: &PoolOf<T>) -> BalanceOf<T> {
            // We don't charge exit fees on closed or cleaned up pools (no need to punish LPs for
            // leaving the pool)!
            match pool.status {
                PoolStatus::Open => T::ExitFee::get().saturated_into(),
                _ => 0u128.saturated_into(),
            }
        }
    }

    impl<T> Swaps<T::AccountId> for Pallet<T>
    where
        T: Config,
    {
        type Asset = AssetOf<T>;
        type Balance = BalanceOf<T>;

        #[frame_support::transactional]
        fn create_pool(
            who: T::AccountId,
            assets: Vec<AssetOf<T>>,
            swap_fee: BalanceOf<T>,
            amount: BalanceOf<T>,
            weights: Vec<BalanceOf<T>>,
        ) -> Result<PoolId, DispatchError> {
            ensure!(assets.len() <= usize::from(T::MaxAssets::get()), Error::<T>::TooManyAssets);
            ensure!(assets.len() >= usize::from(T::MinAssets::get()), Error::<T>::TooFewAssets);
            let next_pool_id = Self::inc_next_pool_id()?;
            let pool_shares_id = Self::pool_shares_id(next_pool_id);
            let pool_account = Self::pool_account_id(&next_pool_id);
            let mut map = BTreeMap::new();
            let mut total_weight: BalanceOf<T> = Zero::zero();
            let mut sorted_assets = assets.clone();
            sorted_assets.sort();
            let has_duplicates = sorted_assets
                .iter()
                .zip(sorted_assets.iter().skip(1))
                .fold(false, |acc, (&x, &y)| acc || x == y);
            ensure!(!has_duplicates, Error::<T>::SomeIdenticalAssets);

            // `amount` must be larger than all minimum balances. As we deposit `amount`
            // liquidity shares, we must also ensure that `amount` is larger than the
            // existential deposit of the liquidity shares.
            ensure!(
                amount >= Self::min_balance_of_pool(next_pool_id, &assets),
                Error::<T>::InsufficientLiquidity
            );

            ensure!(swap_fee <= T::MaxSwapFee::get(), Error::<T>::SwapFeeTooHigh);
            Self::check_provided_values_len_must_equal_assets_len(&assets, &weights)?;

            for (asset, weight) in assets.iter().copied().zip(weights) {
                let free_balance = T::AssetManager::free_balance(asset, &who);
                ensure!(free_balance >= amount, Error::<T>::InsufficientBalance);
                ensure!(weight >= T::MinWeight::get(), Error::<T>::BelowMinimumWeight);
                ensure!(weight <= T::MaxWeight::get(), Error::<T>::AboveMaximumWeight);
                map.insert(asset, weight);
                total_weight = total_weight.checked_add_res(&weight)?;
                T::AssetManager::transfer(asset, &who, &pool_account, amount)?;
            }

            ensure!(total_weight <= T::MaxTotalWeight::get(), Error::<T>::MaxTotalWeight);
            T::AssetManager::deposit(pool_shares_id, &who, amount)?;

            let pool = Pool {
                assets: sorted_assets
                    .try_into()
                    .map_err(|_| Error::<T>::Unexpected(UnexpectedError::StorageOverflow))?,
                swap_fee,
                status: PoolStatus::Closed,
                total_weight,
                weights: map
                    .try_into()
                    .map_err(|_| Error::<T>::Unexpected(UnexpectedError::StorageOverflow))?,
            };

            Pools::<T>::insert(next_pool_id, pool.clone());

            Self::deposit_event(Event::PoolCreate {
                common: CommonPoolEventParams { pool_id: next_pool_id, who },
                pool,
                pool_amount: amount,
                pool_account,
            });

            Ok(next_pool_id)
        }

        fn close_pool(pool_id: PoolId) -> Result<Weight, DispatchError> {
            let asset_len =
                <Pools<T>>::try_mutate(pool_id, |opt_pool| -> Result<u32, DispatchError> {
                    let pool = opt_pool.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;
                    ensure!(pool.status == PoolStatus::Open, Error::<T>::InvalidStateTransition);
                    pool.status = PoolStatus::Closed;
                    Ok(pool.assets.len() as u32)
                })?;
            Self::deposit_event(Event::PoolClosed { pool_id });
            Ok(T::WeightInfo::close_pool(asset_len))
        }

        fn destroy_pool(pool_id: PoolId) -> Result<Weight, DispatchError> {
            let pool = Self::pool_by_id(pool_id)?;
            let pool_account = Self::pool_account_id(&pool_id);
            let asset_len = pool.assets.len() as u32;
            for asset in pool.assets.into_iter() {
                let amount = T::AssetManager::free_balance(asset, &pool_account);
                T::AssetManager::withdraw(asset, &pool_account, amount)?;
            }
            // NOTE: Currently we don't clean up accounts with pool_share_id.
            // TODO(#792): Remove pool_share_id asset for accounts! It may require storage migration.
            Pools::<T>::remove(pool_id);
            Self::deposit_event(Event::PoolDestroyed { pool_id });
            Ok(T::WeightInfo::destroy_pool(asset_len))
        }

        fn open_pool(pool_id: PoolId) -> Result<Weight, DispatchError> {
            Self::mutate_pool(pool_id, |pool| -> DispatchResult {
                ensure!(pool.status == PoolStatus::Closed, Error::<T>::InvalidStateTransition);
                pool.status = PoolStatus::Open;
                Ok(())
            })?;
            let pool = Pools::<T>::get(pool_id).ok_or(Error::<T>::PoolDoesNotExist)?;
            let asset_len = pool.assets.len() as u32;
            Self::deposit_event(Event::PoolActive { pool_id });
            Ok(T::WeightInfo::open_pool(asset_len))
        }

        fn pool_exit_with_exact_asset_amount(
            who: T::AccountId,
            pool_id: PoolId,
            asset: AssetOf<T>,
            asset_amount: BalanceOf<T>,
            max_pool_amount: BalanceOf<T>,
        ) -> DispatchResult {
            Self::do_pool_exit_with_exact_asset_amount(
                who,
                pool_id,
                asset,
                asset_amount,
                max_pool_amount,
            )
        }

        fn pool_join_with_exact_asset_amount(
            who: T::AccountId,
            pool_id: PoolId,
            asset_in: AssetOf<T>,
            asset_amount: BalanceOf<T>,
            min_pool_amount: BalanceOf<T>,
        ) -> DispatchResult {
            Self::do_pool_join_with_exact_asset_amount(
                who,
                pool_id,
                asset_in,
                asset_amount,
                min_pool_amount,
            )
        }

        fn swap_exact_amount_in(
            who: T::AccountId,
            pool_id: PoolId,
            asset_in: AssetOf<T>,
            asset_amount_in: BalanceOf<T>,
            asset_out: AssetOf<T>,
            min_asset_amount_out: Option<BalanceOf<T>>,
            max_price: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            Self::do_swap_exact_amount_in(
                who,
                pool_id,
                asset_in,
                asset_amount_in,
                asset_out,
                min_asset_amount_out,
                max_price,
            )
        }

        fn swap_exact_amount_out(
            who: T::AccountId,
            pool_id: PoolId,
            asset_in: AssetOf<T>,
            max_asset_amount_in: Option<BalanceOf<T>>,
            asset_out: AssetOf<T>,
            asset_amount_out: BalanceOf<T>,
            max_price: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            Self::do_swap_exact_amount_out(
                who,
                pool_id,
                asset_in,
                max_asset_amount_in,
                asset_out,
                asset_amount_out,
                max_price,
            )
        }
    }
}
