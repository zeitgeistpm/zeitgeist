//! # Swaps
//!
//! A module to handle swapping shares out for different ones. Allows
//! liquidity providers to deposit full outcome shares and earn fees.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[macro_use]
mod macros;

mod benchmarks;
mod check_arithm_rslt;
mod consts;
mod events;
mod fixed;
mod math;
mod migrations;
pub mod mock;
mod tests;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{
        check_arithm_rslt::CheckArithmRslt,
        events::{CommonPoolEventParams, PoolAssetEvent, PoolAssetsEvent, SwapEvent},
        fixed::{bdiv, bmul},
        weights::*,
    };
    use alloc::{collections::btree_map::BTreeMap, vec::Vec};
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::Weight,
        ensure,
        pallet_prelude::{StorageMap, StorageValue, ValueQuery},
        traits::{Get, Hooks, IsType},
        Blake2_128Concat, PalletId, Parameter,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use orml_traits::MultiCurrency;
    use sp_runtime::{
        traits::{AccountIdConversion, AtLeast32Bit, MaybeSerializeDeserialize, Member, Zero},
        DispatchError, DispatchResult, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        traits::{Swaps, ZeitgeistMultiReservableCurrency},
        types::{Asset, MarketType, OutcomeReport, Pool, PoolId, PoolStatus, SerdeWrapper},
    };

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Shares as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Temporary probably - The Swap is created per prediction market.
        #[pallet::weight(T::WeightInfo::create_pool(weights.len() as u32))]
        #[frame_support::transactional]
        pub fn create_pool(
            origin: OriginFor<T>,
            assets: Vec<Asset<T::MarketId>>,
            weights: Vec<u128>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let _ = <Self as Swaps<T::AccountId>>::create_pool(who, assets, Zero::zero(), weights)?;
            Ok(())
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
        #[pallet::weight(T::WeightInfo::pool_exit(min_assets_out.len() as u32))]
        #[frame_support::transactional]
        pub fn pool_exit(
            origin: OriginFor<T>,
            pool_id: PoolId,
            pool_amount: BalanceOf<T>,
            min_assets_out: Vec<BalanceOf<T>>,
        ) -> DispatchResult {
            let pool = Self::pool_by_id(pool_id)?;
            pool!(
                initial_params: (min_assets_out, origin, pool_amount, pool, pool_id),

                event: PoolExit,
                transfer_asset: |amount, amount_bound, asset, pool_account_id, who| {
                    ensure!(amount >= amount_bound, Error::<T>::LimitOut);
                    T::Shares::transfer(asset, pool_account_id, who, amount)?;
                    Ok(())
                },
                transfer_pool: |pool_account_id, pool_shares_id, who| {
                    let exit_fee_pct = T::ExitFee::get().saturated_into();
                    let exit_fee = bmul(pool_amount.saturated_into(), exit_fee_pct)?.saturated_into();
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
        #[pallet::weight(T::WeightInfo::pool_exit_with_exact_asset_amount())]
        #[frame_support::transactional]
        pub fn pool_exit_with_exact_asset_amount(
            origin: OriginFor<T>,
            pool_id: PoolId,
            asset: Asset<T::MarketId>,
            asset_amount: BalanceOf<T>,
            max_pool_amount: BalanceOf<T>,
        ) -> DispatchResult {
            pool_exit_with_exact_amount!(
                initial_params: (origin, pool_id, asset),

                asset_amount: |_, _, _| Ok(asset_amount),
                bound: max_pool_amount,
                ensure_balance: |asset_balance: BalanceOf<T>| {
                    ensure!(
                        asset_amount <= bmul(asset_balance.saturated_into(), T::MaxOutRatio::get().saturated_into())?.saturated_into(),
                        Error::<T>::MaxOutRatio
                    );
                    Ok(())
                },
                event: PoolExitWithExactAssetAmount,
                pool_amount: |pool: &Pool<BalanceOf<T>, _>, asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let pool_amount: BalanceOf<T> = crate::math::calc_pool_in_given_single_out(
                        asset_balance.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset)?,
                        total_supply.saturated_into(),
                        pool.total_weight,
                        asset_amount.saturated_into(),
                        pool.swap_fee.saturated_into(),
                    )?.saturated_into();
                    ensure!(pool_amount != Zero::zero(), Error::<T>::MathApproximation);
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
        #[pallet::weight(T::WeightInfo::pool_exit_with_exact_pool_amount())]
        #[frame_support::transactional]
        pub fn pool_exit_with_exact_pool_amount(
            origin: OriginFor<T>,
            pool_id: PoolId,
            asset: Asset<T::MarketId>,
            pool_amount: BalanceOf<T>,
            min_asset_amount: BalanceOf<T>,
        ) -> DispatchResult {
            pool_exit_with_exact_amount!(
                initial_params: (origin, pool_id, asset),

                asset_amount: |pool: &Pool<BalanceOf<T>, _>, asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let asset_amount: BalanceOf<T> = crate::math::calc_single_out_given_pool_in(
                        asset_balance.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset)?,
                        total_supply.saturated_into(),
                        pool.total_weight,
                        pool_amount.saturated_into(),
                        pool.swap_fee.saturated_into(),
                    )?.saturated_into();
                    ensure!(asset_amount >= min_asset_amount, Error::<T>::LimitOut);
                    ensure!(
                        asset_amount <= bmul(asset_balance.saturated_into(), T::MaxOutRatio::get().saturated_into())?.saturated_into(),
                        Error::<T>::MaxOutRatio
                    );
                    Ok(asset_amount)
                },
                bound: min_asset_amount,
                ensure_balance: |_| Ok(()),
                event: PoolExitWithExactPoolAmount,
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
        #[pallet::weight(T::WeightInfo::pool_join(max_assets_in.len() as u32))]
        #[frame_support::transactional]
        pub fn pool_join(
            origin: OriginFor<T>,
            pool_id: PoolId,
            pool_amount: BalanceOf<T>,
            max_assets_in: Vec<BalanceOf<T>>,
        ) -> DispatchResult {
            let pool = Self::pool_by_id(pool_id)?;
            Self::check_if_pool_is_active(&pool)?;
            pool!(
                initial_params: (max_assets_in, origin, pool_amount, pool, pool_id),

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
        #[pallet::weight(T::WeightInfo::pool_join_with_exact_asset_amount())]
        #[frame_support::transactional]
        pub fn pool_join_with_exact_asset_amount(
            origin: OriginFor<T>,
            pool_id: PoolId,
            asset_in: Asset<T::MarketId>,
            asset_amount: BalanceOf<T>,
            min_pool_amount: BalanceOf<T>,
        ) -> DispatchResult {
            pool_join_with_exact_amount!(
                initial_params: (origin, pool_id, asset_in),

                asset_amount: |_, _, _| Ok(asset_amount),
                bound: min_pool_amount,
                event: PoolJoinWithExactAssetAmount,
                pool_amount: |pool: &Pool<BalanceOf<T>, _>, asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let mul: BalanceOf<T> = bmul(
                        asset_balance.saturated_into(),
                        T::MaxInRatio::get().saturated_into()
                    )?.saturated_into();
                    ensure!(asset_amount <= mul, Error::<T>::MaxInRatio);
                    let pool_amount: BalanceOf<T> = crate::math::calc_pool_out_given_single_in(
                        asset_balance.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset_in)?,
                        total_supply.saturated_into(),
                        pool.total_weight.saturated_into(),
                        asset_amount.saturated_into(),
                        pool.swap_fee.saturated_into(),
                    )?.saturated_into();
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
        #[pallet::weight(T::WeightInfo::pool_join_with_exact_pool_amount())]
        #[frame_support::transactional]
        pub fn pool_join_with_exact_pool_amount(
            origin: OriginFor<T>,
            pool_id: PoolId,
            asset: Asset<T::MarketId>,
            pool_amount: BalanceOf<T>,
            max_asset_amount: BalanceOf<T>,
        ) -> DispatchResult {
            pool_join_with_exact_amount!(
                initial_params: (origin, pool_id, asset),

                asset_amount: |pool: &Pool<BalanceOf<T>, _>, asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let asset_amount: BalanceOf<T> = crate::math::calc_single_in_given_pool_out(
                        asset_balance.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset)?,
                        total_supply.saturated_into(),
                        pool.total_weight.saturated_into(),
                        pool_amount.saturated_into(),
                        pool.swap_fee.saturated_into(),
                    )?.saturated_into();
                    ensure!(asset_amount != Zero::zero(), Error::<T>::MathApproximation);
                    ensure!(asset_amount <= max_asset_amount, Error::<T>::LimitIn);
                    ensure!(
                        asset_amount <= asset_balance.check_mul_rslt(&T::MaxInRatio::get())?,
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
        #[pallet::weight(T::WeightInfo::swap_exact_amount_in())]
        #[frame_support::transactional]
        pub fn swap_exact_amount_in(
            origin: OriginFor<T>,
            pool_id: PoolId,
            asset_in: Asset<T::MarketId>,
            asset_amount_in: BalanceOf<T>,
            asset_out: Asset<T::MarketId>,
            min_asset_amount_out: BalanceOf<T>,
            max_price: BalanceOf<T>,
        ) -> DispatchResult {
            swap_exact_amount!(
                initial_params: (asset_in, asset_out, max_price, origin, pool_id),

                asset_amount_in: |_, _| Ok(asset_amount_in),
                asset_amount_out: |pool: &Pool<BalanceOf<T>, _>, pool_account_id| {
                    let balance_in = T::Shares::free_balance(asset_in, pool_account_id);
                    ensure!(
                        asset_amount_in <= bmul(balance_in.saturated_into(), T::MaxInRatio::get().saturated_into())?.saturated_into(),
                        Error::<T>::MaxInRatio
                    );

                    let balance_out = T::Shares::free_balance(asset_out, pool_account_id);

                    let asset_amount_out: BalanceOf<T> = crate::math::calc_out_given_in(
                        balance_in.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset_in)?,
                        balance_out.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset_out)?,
                        asset_amount_in.saturated_into(),
                        pool.swap_fee.saturated_into(),
                    )?.saturated_into();
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
        #[pallet::weight(T::WeightInfo::swap_exact_amount_out())]
        #[frame_support::transactional]
        pub fn swap_exact_amount_out(
            origin: OriginFor<T>,
            pool_id: PoolId,
            asset_in: Asset<T::MarketId>,
            max_amount_asset_in: BalanceOf<T>,
            asset_out: Asset<T::MarketId>,
            asset_amount_out: BalanceOf<T>,
            max_price: BalanceOf<T>,
        ) -> DispatchResult {
            swap_exact_amount!(
                initial_params: (asset_in, asset_out, max_price, origin, pool_id),

                asset_amount_in: |pool: &Pool<BalanceOf<T>, _>, pool_account_id| {
                    let balance_in = T::Shares::free_balance(asset_in, pool_account_id);

                    let balance_out = T::Shares::free_balance(asset_out, pool_account_id);
                    ensure!(
                        asset_amount_out <= bmul(balance_out.saturated_into(), T::MaxOutRatio::get().saturated_into())?.saturated_into(),
                        Error::<T>::MaxOutRatio,
                    );

                    let asset_amount_in: BalanceOf<T> = crate::math::calc_in_given_out(
                        balance_in.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset_in)?,
                        balance_out.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset_out)?,
                        asset_amount_out.saturated_into(),
                        pool.swap_fee.saturated_into(),
                    )?.saturated_into();
                    ensure!(asset_amount_in <= max_amount_asset_in, Error::<T>::LimitIn);

                    Ok(asset_amount_in)
                },
                asset_amount_out: |_, _| Ok(asset_amount_out),
                asset_bound: max_amount_asset_in,
                event: SwapExactAmountOut
            )
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The fee for exiting a pool.
        type ExitFee: Get<BalanceOf<Self>>;

        type MarketId: AtLeast32Bit
            + Copy
            + Default
            + MaybeSerializeDeserialize
            + Member
            + Parameter;

        type MaxAssets: Get<usize>;
        type MaxInRatio: Get<BalanceOf<Self>>;
        type MaxOutRatio: Get<BalanceOf<Self>>;
        type MaxTotalWeight: Get<u128>;
        type MaxWeight: Get<u128>;
        type MinWeight: Get<u128>;

        /// The minimum amount of liqudity required to bootstrap a pool.
        type MinLiquidity: Get<BalanceOf<Self>>;

        /// The module identifier.
        type PalletId: Get<PalletId>;

        type Shares: ZeitgeistMultiReservableCurrency<
            Self::AccountId,
            CurrencyId = Asset<Self::MarketId>,
        >;

        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
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
        PoolIsNotActive,
        ProvidedValuesLenMustEqualAssetsLen,
        TooManyAssets,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A new pool has been created. \[account\]
        PoolCreate(CommonPoolEventParams<<T as frame_system::Config>::AccountId>),
        /// Someone has exited a pool. \[account, amount\]
        PoolExit(PoolAssetsEvent<<T as frame_system::Config>::AccountId, BalanceOf<T>>),
        /// Exits a pool given an exact amount of an asset. \[account, amount\]
        PoolExitWithExactAssetAmount(
            PoolAssetEvent<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
        ),
        /// Exits a pool given an exact pool's amount. \[account, amount\]
        PoolExitWithExactPoolAmount(
            PoolAssetEvent<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
        ),
        /// Someone has joined a pool. \[account, amount\]
        PoolJoin(PoolAssetsEvent<<T as frame_system::Config>::AccountId, BalanceOf<T>>),
        /// Joins a pool given an exact amount of an asset. \[account, amount\]
        PoolJoinWithExactAssetAmount(
            PoolAssetEvent<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
        ),
        /// Joins a pool given an exact pool's amount. \[account, amount\]
        PoolJoinWithExactPoolAmount(
            PoolAssetEvent<<T as frame_system::Config>::AccountId, BalanceOf<T>>,
        ),
        /// An exact amount of an asset is entering the pool. \[account, amount\]
        SwapExactAmountIn(SwapEvent<<T as frame_system::Config>::AccountId, BalanceOf<T>>),
        /// An exact amount of an asset is leaving the pool. \[account, amount\]
        SwapExactAmountOut(SwapEvent<<T as frame_system::Config>::AccountId, BalanceOf<T>>),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            crate::migrations::_0_1_1_add_pool_status::migrate::<T>()
        }
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config> =
        StorageMap<_, Blake2_128Concat, u128, Option<Pool<BalanceOf<T>, T::MarketId>>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn next_pool_id)]
    pub type NextPoolId<T> = StorageValue<_, PoolId, ValueQuery>;

    impl<T: Config> Pallet<T> {
        fn burn_pool_shares(
            pool_id: PoolId,
            from: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let shares_id = Self::pool_shares_id(pool_id);
            T::Shares::slash(shares_id, from, amount);
            Ok(())
        }

        #[inline]
        fn check_provided_values_len_must_equal_assets_len<U>(
            assets: &[Asset<T::MarketId>],
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

        pub fn get_spot_price(
            pool_id: PoolId,
            asset_in: Asset<T::MarketId>,
            asset_out: Asset<T::MarketId>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let pool = Self::pool_by_id(pool_id)?;

            let pool_account = Self::pool_account_id(pool_id);
            let balance_in = T::Shares::free_balance(asset_in, &pool_account);
            let in_weight = Self::pool_weight_rslt(&pool, &asset_in)?;
            let balance_out = T::Shares::free_balance(asset_out, &pool_account);
            let out_weight = Self::pool_weight_rslt(&pool, &asset_out)?;

            Ok(crate::math::calc_spot_price(
                balance_in.saturated_into(),
                in_weight,
                balance_out.saturated_into(),
                out_weight,
                0,
            )?
            .saturated_into())
        }

        fn inc_next_pool_id() -> PoolId {
            let id = <NextPoolId<T>>::get();
            <NextPoolId<T>>::mutate(|n| *n += 1);
            id
        }

        fn check_if_pool_is_active(pool: &Pool<BalanceOf<T>, T::MarketId>) -> DispatchResult {
            if let PoolStatus::Active = pool.pool_status {
                Ok(())
            } else {
                Err(Error::<T>::PoolIsNotActive.into())
            }
        }

        fn mint_pool_shares(
            pool_id: PoolId,
            to: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let shares_id = Self::pool_shares_id(pool_id);
            T::Shares::deposit(shares_id, to, amount)
        }

        pub fn pool_account_id(pool_id: PoolId) -> T::AccountId {
            T::PalletId::get().into_sub_account(pool_id)
        }

        fn pool_by_id(pool_id: PoolId) -> Result<Pool<BalanceOf<T>, T::MarketId>, Error<T>>
        where
            T: Config,
        {
            Self::pools(pool_id).ok_or(Error::<T>::PoolDoesNotExist)
        }

        fn pool_weight_rslt(
            pool: &Pool<BalanceOf<T>, T::MarketId>,
            asset: &Asset<T::MarketId>,
        ) -> Result<u128, Error<T>> {
            pool.weights
                .get(asset)
                .cloned()
                .ok_or(Error::<T>::AssetNotBound)
        }

        fn set_pool_status(pool_id: PoolId, pool_status: PoolStatus) -> DispatchResult {
            <Pools<T>>::try_mutate(pool_id, |pool| {
                let pool = if let Some(el) = pool {
                    el
                } else {
                    return Err(Error::<T>::PoolDoesNotExist.into());
                };
                pool.pool_status = pool_status;
                Ok(())
            })
        }

        pub(crate) fn pool_shares_id(pool_id: PoolId) -> Asset<T::MarketId> {
            Asset::PoolShare(SerdeWrapper(pool_id))
        }
    }

    impl<T> Swaps<T::AccountId> for Pallet<T>
    where
        T: Config,
    {
        type Balance = BalanceOf<T>;
        type MarketId = T::MarketId;

        fn create_pool(
            who: T::AccountId,
            assets: Vec<Asset<T::MarketId>>,
            swap_fee: BalanceOf<T>,
            weights: Vec<u128>,
        ) -> Result<PoolId, DispatchError> {
            Self::check_provided_values_len_must_equal_assets_len(&assets, &weights)?;

            ensure!(
                assets.len() <= T::MaxAssets::get(),
                Error::<T>::TooManyAssets
            );

            let amount = T::MinLiquidity::get();

            let next_pool_id = Self::inc_next_pool_id();
            let pool_account = Self::pool_account_id(next_pool_id);
            let mut map = BTreeMap::new();
            let mut total_weight: u128 = 0;

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
                Some(Pool {
                    assets,
                    pool_status: PoolStatus::Active,
                    swap_fee,
                    total_weight,
                    weights: map,
                }),
            );

            let pool_shares_id = Self::pool_shares_id(next_pool_id);
            T::Shares::deposit(pool_shares_id, &who, amount)?;

            Self::deposit_event(Event::PoolCreate(CommonPoolEventParams {
                pool_id: next_pool_id,
                who,
            }));

            Ok(next_pool_id)
        }

        fn pool(pool_id: PoolId) -> Result<Pool<Self::Balance, Self::MarketId>, DispatchError> {
            Ok(Self::pool_by_id(pool_id)?)
        }

        fn set_pool_as_stale(
            market_type: &MarketType,
            pool_id: PoolId,
            outcome_report: &OutcomeReport,
        ) -> Result<(), DispatchError> {
            if let MarketType::Categorical(_) = market_type {
                if let OutcomeReport::Categorical(winning_asset_idx) = outcome_report {
                    <Pools<T>>::try_mutate(pool_id, |pool| {
                        let pool_mut = if let Some(el) = pool {
                            el
                        } else {
                            return Err(Error::<T>::PoolDoesNotExist.into());
                        };
                        pool_mut.assets.retain(|el| {
                            if let Asset::CategoricalOutcome(_, idx) = el {
                                idx == winning_asset_idx
                            } else {
                                matches!(el, Asset::Ztg)
                            }
                        });
                        Ok::<_, DispatchError>(())
                    })?;
                }
            }

            Self::set_pool_status(pool_id, PoolStatus::Stale)?;
            Ok(())
        }
    }
}
