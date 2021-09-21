//! # Swaps
//!
//! A module to handle swapping shares out for different ones. Allows
//! liquidity providers to deposit full outcome shares and earn fees.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[macro_use]
mod utils;

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
        dispatch::Weight,
        ensure,
        pallet_prelude::{StorageMap, StorageValue, ValueQuery},
        traits::{Get, IsType},
        Blake2_128Concat, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_root, ensure_signed, pallet_prelude::OriginFor};
    use orml_traits::{BalanceStatus, MultiCurrency, MultiReservableCurrency};
    use parity_scale_codec::{Decode, Encode};
    use sp_runtime::{
        traits::{AccountIdConversion, Saturating, Zero},
        ArithmeticError, DispatchError, DispatchResult, SaturatedConversion,
    };
    use substrate_fixed::{
        traits::{FixedSigned, FixedUnsigned, LossyFrom},
        types::{
            extra::{U127, U128, U24, U31, U32},
            I9F23, U1F127,
        },
        FixedI128, FixedI32, FixedU128, FixedU32,
    };
    use zeitgeist_primitives::{
        constants::BASE,
        traits::{MarketId, Swaps, ZeitgeistMultiReservableCurrency},
        types::{
            Asset, MarketType, OutcomeReport, Pool, PoolId, PoolStatus, ScoringRule, SerdeWrapper,
            SubsidyProvider,
        },
    };
    use zrml_liquidity_mining::LiquidityMiningPalletApi;
    use zrml_rikiddo::{
        constants::{EMA_LONG, EMA_SHORT},
        traits::RikiddoMVPallet,
        types::{EmaMarketVolume, FeeSigmoid, RikiddoSigmoidMV},
    };

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Shares as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(T::WeightInfo::admin_set_pool_as_stale())]
        #[frame_support::transactional]
        pub fn admin_set_pool_as_stale(
            origin: OriginFor<T>,
            market_type: MarketType,
            pool_id: PoolId,
            outcome_report: OutcomeReport,
        ) -> DispatchResult {
            ensure_root(origin)?;
            Self::set_pool_as_stale(&market_type, pool_id, &outcome_report)
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
            let who = ensure_signed(origin)?;
            let who_clone = who.clone();
            let pool = Self::pool_by_id(pool_id)?;
            let pool_account_id = Pallet::<T>::pool_account_id(pool_id);
            let params = PoolParams {
                asset_bounds: min_assets_out,
                event: |evt| Self::deposit_event(Event::PoolExit(evt)),
                pool_account_id: &pool_account_id,
                pool_amount,
                pool_id,
                pool: &pool,
                transfer_asset: |amount, amount_bound, asset| {
                    ensure!(amount >= amount_bound, Error::<T>::LimitOut);
                    T::LiquidityMining::remove_shares(&who, &pool.market_id, amount);
                    T::Shares::transfer(asset, &pool_account_id, &who, amount)?;
                    Ok(())
                },
                transfer_pool: |pool_shares_id| {
                    let exit_fee_pct = T::ExitFee::get().saturated_into();
                    let exit_fee =
                        bmul(pool_amount.saturated_into(), exit_fee_pct)?.saturated_into();
                    let pool_amount_minus_exit_fee = pool_amount.check_sub_rslt(&exit_fee)?;
                    T::Shares::transfer(pool_shares_id, &who, &pool_account_id, exit_fee)?;
                    Self::burn_pool_shares(pool_id, &who, pool_amount_minus_exit_fee)?;
                    Ok(())
                },
                who: who_clone,
            };
            crate::utils::pool::<_, _, _, T>(params)
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
        #[pallet::weight(T::WeightInfo::pool_exit_with_exact_asset_amount())]
        pub fn pool_exit_with_exact_asset_amount(
            origin: OriginFor<T>,
            pool_id: PoolId,
            asset: Asset<T::MarketId>,
            asset_amount: BalanceOf<T>,
            max_pool_amount: BalanceOf<T>,
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
        #[pallet::weight(T::WeightInfo::pool_exit_with_exact_pool_amount())]
        #[frame_support::transactional]
        pub fn pool_exit_with_exact_pool_amount(
            origin: OriginFor<T>,
            pool_id: PoolId,
            asset: Asset<T::MarketId>,
            pool_amount: BalanceOf<T>,
            min_asset_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let pool = Self::pool_by_id(pool_id)?;
            let pool_ref = &pool;
            let who = ensure_signed(origin)?;
            let who_clone = who.clone();

            let params = PoolExitWithExactAmountParams {
                asset,
                asset_amount: |asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let asset_amount: BalanceOf<T> = crate::math::calc_single_out_given_pool_in(
                        asset_balance.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset)?,
                        total_supply.saturated_into(),
                        pool.total_weight.ok_or(Error::<T>::PoolMissingWeight)?.saturated_into(),
                        pool_amount.saturated_into(),
                        pool.swap_fee.ok_or(Error::<T>::PoolMissingFee)?.saturated_into(),
                    )?
                    .saturated_into();
                    ensure!(asset_amount >= min_asset_amount, Error::<T>::LimitOut);
                    ensure!(
                        asset_amount
                            <= bmul(
                                asset_balance.saturated_into(),
                                T::MaxOutRatio::get().saturated_into()
                            )?
                            .saturated_into(),
                        Error::<T>::MaxOutRatio
                    );
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
        #[pallet::weight(T::WeightInfo::pool_join(max_assets_in.len() as u32))]
        #[frame_support::transactional]
        pub fn pool_join(
            origin: OriginFor<T>,
            pool_id: PoolId,
            pool_amount: BalanceOf<T>,
            max_assets_in: Vec<BalanceOf<T>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let pool = Self::pool_by_id(pool_id)?;
            let pool_account_id = Pallet::<T>::pool_account_id(pool_id);

            Self::check_if_pool_is_active(&pool)?;
            let params = PoolParams {
                asset_bounds: max_assets_in,
                event: |evt| Self::deposit_event(Event::PoolJoin(evt)),
                pool_account_id: &pool_account_id,
                pool_amount,
                pool_id,
                pool: &pool,
                transfer_asset: |amount, amount_bound, asset| {
                    ensure!(amount <= amount_bound, Error::<T>::LimitIn);
                    T::Shares::transfer(asset, &who, &pool_account_id, amount)?;
                    T::LiquidityMining::add_shares(who.clone(), pool.market_id, amount);
                    Ok(())
                },
                transfer_pool: |_| Self::mint_pool_shares(pool_id, &who, pool_amount),
                who: who.clone(),
            };
            crate::utils::pool::<_, _, _, T>(params)
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
        #[pallet::weight(T::WeightInfo::pool_join_with_exact_asset_amount())]
        pub fn pool_join_with_exact_asset_amount(
            origin: OriginFor<T>,
            pool_id: PoolId,
            asset_in: Asset<T::MarketId>,
            asset_amount: BalanceOf<T>,
            min_pool_amount: BalanceOf<T>,
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
        #[pallet::weight(T::WeightInfo::pool_join_with_exact_pool_amount())]
        #[frame_support::transactional]
        pub fn pool_join_with_exact_pool_amount(
            origin: OriginFor<T>,
            pool_id: PoolId,
            asset: Asset<T::MarketId>,
            pool_amount: BalanceOf<T>,
            max_asset_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let pool = Pallet::<T>::pool_by_id(pool_id)?;
            let pool_account_id = Pallet::<T>::pool_account_id(pool_id);
            let who = ensure_signed(origin)?;
            let who_clone = who.clone();
            let params = PoolJoinWithExactAmountParams {
                asset,
                asset_amount: |asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let asset_amount: BalanceOf<T> = crate::math::calc_single_in_given_pool_out(
                        asset_balance.saturated_into(),
                        Self::pool_weight_rslt(&pool, &asset)?,
                        total_supply.saturated_into(),
                        pool.total_weight.ok_or(Error::<T>::PoolMissingWeight)?.saturated_into(),
                        pool_amount.saturated_into(),
                        pool.swap_fee.ok_or(Error::<T>::PoolMissingFee)?.saturated_into(),
                    )?
                    .saturated_into();
                    ensure!(asset_amount != Zero::zero(), Error::<T>::MathApproximation);
                    ensure!(asset_amount <= max_asset_amount, Error::<T>::LimitIn);
                    ensure!(
                        asset_amount <= asset_balance.check_mul_rslt(&T::MaxInRatio::get())?,
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
            let pool = Pallet::<T>::pool_by_id(pool_id)?;
            let pool_account_id = Pallet::<T>::pool_account_id(pool_id);
            let params = SwapExactAmountParams {
                asset_amounts: |price: BalanceOf<T>| {
                    let balance_in = T::Shares::free_balance(asset_in, &pool_account_id);
                    ensure!(
                        asset_amount_in
                            <= bmul(
                                balance_in.saturated_into(),
                                T::MaxInRatio::get().saturated_into()
                            )?
                            .saturated_into(),
                        Error::<T>::MaxInRatio
                    );

                    let asset_amount_out: BalanceOf<T>;

                    if pool.scoring_rule == ScoringRule::CPMM {
                        let balance_out = T::Shares::free_balance(asset_out, &pool_account_id);

                        asset_amount_out = crate::math::calc_out_given_in(
                            balance_in.saturated_into(),
                            Self::pool_weight_rslt(&pool, &asset_in)?,
                            balance_out.saturated_into(),
                            Self::pool_weight_rslt(&pool, &asset_out)?,
                            asset_amount_in.saturated_into(),
                            pool.swap_fee.ok_or(Error::<T>::PoolMissingFee)?.saturated_into(),
                        )?
                        .saturated_into();
                    } else {
                        asset_amount_out =
                            bdiv(asset_amount_in.saturated_into(), price.saturated_into())?
                                .saturated_into();
                    }
                    ensure!(asset_amount_out >= min_asset_amount_out, Error::<T>::LimitOut);

                    Ok([asset_amount_in, asset_amount_out])
                },
                asset_bound: min_asset_amount_out,
                asset_in,
                asset_out,
                event: |evt| Self::deposit_event(Event::SwapExactAmountIn(evt)),
                max_price,
                origin,
                pool_account_id: &pool_account_id,
                pool_id,
                pool: &pool,
            };
            swap_exact_amount::<_, _, T>(params)
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
            let pool = Pallet::<T>::pool_by_id(pool_id)?;
            let pool_account_id = Pallet::<T>::pool_account_id(pool_id);
            let params = SwapExactAmountParams {
                asset_amounts: |price: BalanceOf<T>| {
                    let balance_out = T::Shares::free_balance(asset_out, &pool_account_id);
                    ensure!(
                        asset_amount_out
                            <= bmul(
                                balance_out.saturated_into(),
                                T::MaxOutRatio::get().saturated_into()
                            )?
                            .saturated_into(),
                        Error::<T>::MaxOutRatio,
                    );

                    let asset_amount_in: BalanceOf<T>;

                    if pool.scoring_rule == ScoringRule::CPMM {
                        let balance_in = T::Shares::free_balance(asset_in, &pool_account_id);
                        asset_amount_in = crate::math::calc_in_given_out(
                            balance_in.saturated_into(),
                            Self::pool_weight_rslt(&pool, &asset_in)?,
                            balance_out.saturated_into(),
                            Self::pool_weight_rslt(&pool, &asset_out)?,
                            asset_amount_out.saturated_into(),
                            pool.swap_fee.ok_or(Error::<T>::PoolMissingFee)?.saturated_into(),
                        )?
                        .saturated_into();
                    } else {
                        asset_amount_in =
                            bmul(asset_amount_out.saturated_into(), price.saturated_into())?
                                .saturated_into();
                    }

                    ensure!(asset_amount_in <= max_amount_asset_in, Error::<T>::LimitIn);
                    Ok([asset_amount_in, asset_amount_out])
                },
                asset_bound: max_amount_asset_in,
                asset_in,
                asset_out,
                event: |evt| Self::deposit_event(Event::SwapExactAmountOut(evt)),
                max_price,
                origin,
                pool_account_id: &pool_account_id,
                pool_id,
                pool: &pool,
            };
            swap_exact_amount::<_, _, T>(params)
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The fee for exiting a pool.
        #[pallet::constant]
        type ExitFee: Get<BalanceOf<Self>>;

        /// Will be used for the fractional part of the fixed point numbers
        /// Calculation: Select FixedTYPE<UWIDTH>, such that TYPE = the type of Balance (i.e. FixedU128)
        /// Select the generic UWIDTH = floor(log2(10.pow(fractional_decimals)))
        type FixedTypeU: Decode
            + Encode
            + FixedUnsigned
            + From<u32>
            + LossyFrom<FixedU32<U24>>
            + LossyFrom<FixedU32<U32>>
            + LossyFrom<FixedU128<U128>>;

        /// Will be used for the fractional part of the fixed point numbers
        /// Calculation: Select FixedTYPE, such that it is the signed variant of FixedTypeU
        /// It is possible to reduce the fractional bit count by one, effectively eliminating
        /// conversion overflows when the MSB of the unsigned fixed type is set, but in exchange
        /// Reducing the fractional precision by one bit.
        type FixedTypeS: Decode
            + Encode
            + FixedSigned
            + From<I9F23>
            + LossyFrom<FixedI32<U24>>
            + LossyFrom<FixedI32<U31>>
            + LossyFrom<U1F127>
            + LossyFrom<FixedI128<U127>>
            + PartialOrd<I9F23>;

        type LiquidityMining: LiquidityMiningPalletApi<
            AccountId = Self::AccountId,
            Balance = BalanceOf<Self>,
            BlockNumber = Self::BlockNumber,
            MarketId = Self::MarketId,
        >;

        type MarketId: MarketId;

        #[pallet::constant]
        type MaxAssets: Get<u16>;

        #[pallet::constant]
        type MaxInRatio: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MaxOutRatio: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type MaxTotalWeight: Get<u128>;

        #[pallet::constant]
        type MaxWeight: Get<u128>;

        /// The minimum amount of liqudity required to bootstrap a pool.
        #[pallet::constant]
        type MinLiquidity: Get<BalanceOf<Self>>;

        /// The minimum amount of subsidy required to state transit a market into active state.
        /// Must be greater than 0, but can be arbitrarily close to 0.
        #[pallet::constant]
        type MinSubsidy: Get<BalanceOf<Self>>;
        #[pallet::constant]
        type MinWeight: Get<u128>;

        /// The module identifier.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The Rikiddo instance that uses a sigmoid fee and ema of market volume
        type RikiddoSigmoidFeeMarketEma: RikiddoMVPallet<
            Balance = BalanceOf<Self>,
            PoolId = PoolId,
            FU = Self::FixedTypeU,
            Rikiddo = RikiddoSigmoidMV<
                Self::FixedTypeU,
                Self::FixedTypeS,
                FeeSigmoid<Self::FixedTypeS>,
                EmaMarketVolume<Self::FixedTypeU>,
            >,
        >;

        /// The custom `MultiReservableCurrency` type
        type Shares: ZeitgeistMultiReservableCurrency<
            Self::AccountId,
            CurrencyId = Asset<Self::MarketId>,
        >;

        /// The weight information for swap's dispatchable functions.
        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
        AboveMaximumWeight,
        AssetNotBound,
        AssetNotInPool,
        BaseAssetNotFound,
        BadLimitPrice,
        BelowMinimumWeight,
        InsufficientBalance,
        InsufficientSubsidy,
        InvalidFeeArgument,
        InvalidStateTransition,
        InvalidWeightArgument,
        LimitIn,
        LimitOut,
        MathApproximation,
        MathApproximationDebug,
        MaxInRatio,
        MaxOutRatio,
        MaxTotalWeight,
        PoolDoesNotExist,
        PoolIsNotActive,
        PoolMissingWeight,
        PoolMissingFee,
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

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        PoolId,
        Option<Pool<BalanceOf<T>, T::MarketId>>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn subsidy_providers)]
    pub type SubsidyProviders<T: Config> =
        StorageMap<_, Twox64Concat, PoolId, SubsidyProvider<T::AccountId, BalanceOf<T>>>;

    #[pallet::storage]
    #[pallet::getter(fn next_pool_id)]
    pub type NextPoolId<T> = StorageValue<_, PoolId, ValueQuery>;

    impl<T: Config> Pallet<T> {
        pub fn get_spot_price(
            pool_id: PoolId,
            asset_in: Asset<T::MarketId>,
            asset_out: Asset<T::MarketId>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let pool = Self::pool_by_id(pool_id)?;
            let assets_slice = pool.assets.as_slice();
            ensure!(assets_slice.binary_search(&asset_in).is_ok(), Error::<T>::AssetNotInPool);
            ensure!(assets_slice.binary_search(&asset_out).is_ok(), Error::<T>::AssetNotInPool);
            let pool_account = Self::pool_account_id(pool_id);
            let balance_in = T::Shares::free_balance(asset_in, &pool_account);
            let balance_out = T::Shares::free_balance(asset_out, &pool_account);

            if pool.scoring_rule == ScoringRule::CPMM {
                let in_weight = Self::pool_weight_rslt(&pool, &asset_in)?;
                let out_weight = Self::pool_weight_rslt(&pool, &asset_out)?;

                return Ok(crate::math::calc_spot_price(
                    balance_in.saturated_into(),
                    in_weight,
                    balance_out.saturated_into(),
                    out_weight,
                    0,
                )?
                .saturated_into());
            }

            // Price when using Rikiddo.
            let mut balances = Vec::new();
            let base_asset = pool.base_asset.ok_or(Error::<T>::BaseAssetNotFound)?;

            for asset in pool.assets {
                if asset == base_asset {
                    continue;
                }

                if asset == asset_in {
                    balances.push(balance_in);
                    continue;
                } else if asset == asset_out {
                    balances.push(balance_out);
                    continue;
                }

                balances.push(T::Shares::free_balance(asset, &pool_account));
            }

            if asset_in == base_asset {
                T::RikiddoSigmoidFeeMarketEma::price(pool_id, balance_in, &balances)
            } else if asset_out == base_asset {
                Ok(bdiv(
                    BASE,
                    T::RikiddoSigmoidFeeMarketEma::price(pool_id, balance_out, &balances)?
                        .saturated_into(),
                )?
                .saturated_into())
            } else {
                let price_without_fee = bdiv(
                    T::RikiddoSigmoidFeeMarketEma::price(pool_id, balance_out, &balances)?
                        .saturated_into(),
                    T::RikiddoSigmoidFeeMarketEma::price(pool_id, balance_in, &balances)?
                        .saturated_into(),
                )?
                .saturated_into();
                let fee: u128 = bmul(
                    price_without_fee,
                    T::RikiddoSigmoidFeeMarketEma::fee(pool_id)?.saturated_into(),
                )?;
                Ok(price_without_fee.check_add_rslt(&fee)?.saturated_into())
            }
        }

        pub fn pool_account_id(pool_id: PoolId) -> T::AccountId {
            T::PalletId::get().into_sub_account(pool_id)
        }

        pub(crate) fn burn_pool_shares(
            pool_id: PoolId,
            from: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let shares_id = Self::pool_shares_id(pool_id);
            T::Shares::slash(shares_id, from, amount);
            Ok(())
        }

        #[inline]
        pub(crate) fn check_provided_values_len_must_equal_assets_len<U>(
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

        pub(crate) fn check_if_pool_is_active(
            pool: &Pool<BalanceOf<T>, T::MarketId>,
        ) -> DispatchResult {
            if pool.pool_status == PoolStatus::Active {
                Ok(())
            } else {
                Err(Error::<T>::PoolIsNotActive.into())
            }
        }

        pub(crate) fn mint_pool_shares(
            pool_id: PoolId,
            to: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let shares_id = Self::pool_shares_id(pool_id);
            T::Shares::deposit(shares_id, to, amount)
        }

        pub(crate) fn pool_shares_id(pool_id: PoolId) -> Asset<T::MarketId> {
            Asset::PoolShare(SerdeWrapper(pool_id))
        }

        pub(crate) fn pool_by_id(
            pool_id: PoolId,
        ) -> Result<Pool<BalanceOf<T>, T::MarketId>, Error<T>>
        where
            T: Config,
        {
            Self::pools(pool_id).ok_or(Error::<T>::PoolDoesNotExist)
        }

        fn inc_next_pool_id() -> Result<PoolId, DispatchError> {
            let id = <NextPoolId<T>>::get();
            <NextPoolId<T>>::try_mutate(|n| {
                *n = n.checked_add(1).ok_or(ArithmeticError::Overflow)?;
                Ok::<_, DispatchError>(())
            })?;
            Ok(id)
        }

        // Mutates a stored pool. Returns `Err` if `pool_id` does not exist.
        fn mutate_pool<F>(pool_id: PoolId, mut cb: F) -> DispatchResult
        where
            F: FnMut(&mut Pool<BalanceOf<T>, T::MarketId>) -> DispatchResult,
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
            pool: &Pool<BalanceOf<T>, T::MarketId>,
            asset: &Asset<T::MarketId>,
        ) -> Result<u128, Error<T>> {
            pool.weights
                .as_ref()
                .ok_or(Error::<T>::PoolMissingWeight)?
                .get(asset)
                .cloned()
                .ok_or(Error::<T>::AssetNotBound)
        }
    }

    impl<T> Swaps<T::AccountId> for Pallet<T>
    where
        T: Config,
    {
        type Balance = BalanceOf<T>;
        type MarketId = T::MarketId;

        /// Creates an initial active pool.
        ///
        /// # Arguments
        ///
        /// * `who`: The account that is the creator of the pool. Must have enough
        /// funds for each of the assets to cover the `MinLiqudity`.
        /// * `assets`: The assets that are used in the pool.
        /// * `base_asset`: The base asset in a prediction market swap pool (usually a currency).
        ///                 Optional if scoring rule is CPMM.
        /// * `market_id`: The market id of the market the pool belongs to.
        /// * `scoring_rule`: The scoring rule that's used to determine the asset prices.
        /// * `swap_fee`: The fee applied to each swap (mandatory if scoring rule is CPMM).
        /// * `weights`: These are the raw/denormalized weights (mandatory if scoring rule is CPMM).
        #[frame_support::transactional]
        fn create_pool(
            who: T::AccountId,
            mut assets: Vec<Asset<T::MarketId>>,
            base_asset: Option<Asset<T::MarketId>>,
            market_id: Self::MarketId,
            scoring_rule: ScoringRule,
            swap_fee: Option<BalanceOf<T>>,
            weights: Option<Vec<u128>>,
        ) -> Result<PoolId, DispatchError> {
            let mut weights_unwrapped = vec![T::MinWeight::get(); assets.len()];

            if scoring_rule == ScoringRule::CPMM {
                let _ = swap_fee.ok_or(Error::<T>::InvalidFeeArgument)?;
                weights_unwrapped = weights.ok_or(Error::<T>::InvalidWeightArgument)?;
                Self::check_provided_values_len_must_equal_assets_len(&assets, &weights_unwrapped)?;
            } else {
                ensure!(base_asset != None, Error::<T>::BaseAssetNotFound);
            }

            ensure!(assets.len() <= usize::from(T::MaxAssets::get()), Error::<T>::TooManyAssets);
            let amount = T::MinLiquidity::get();
            let next_pool_id = Self::inc_next_pool_id()?;
            let pool_account = Self::pool_account_id(next_pool_id);
            let mut map = BTreeMap::new();
            let mut total_weight = 0;

            for (asset, weight) in assets.iter().copied().zip(weights_unwrapped).filter(|pair| {
                if scoring_rule == ScoringRule::RikiddoSigmoidFeeMarketEma
                    && Some(pair.0) != base_asset
                {
                    return false;
                }

                true
            }) {
                let free_balance = T::Shares::free_balance(asset, &who);
                ensure!(free_balance >= amount, Error::<T>::InsufficientBalance);

                if scoring_rule == ScoringRule::CPMM {
                    ensure!(weight >= T::MinWeight::get(), Error::<T>::BelowMinimumWeight);
                    ensure!(weight <= T::MaxWeight::get(), Error::<T>::AboveMaximumWeight);
                    map.insert(asset, weight);
                    total_weight = total_weight.check_add_rslt(&weight)?;
                }

                T::Shares::transfer(asset, &who, &pool_account, amount)?;
            }

            ensure!(total_weight <= T::MaxTotalWeight::get(), Error::<T>::MaxTotalWeight);
            let pool_shares_id = Self::pool_shares_id(next_pool_id);
            T::Shares::deposit(pool_shares_id, &who, amount)?;

            if scoring_rule == ScoringRule::RikiddoSigmoidFeeMarketEma {
                let mut rikiddo_instance: RikiddoSigmoidMV<
                    T::FixedTypeU,
                    T::FixedTypeS,
                    FeeSigmoid<T::FixedTypeS>,
                    EmaMarketVolume<T::FixedTypeU>,
                > = Default::default();
                rikiddo_instance.ma_short.config.ema_period = EMA_SHORT;
                rikiddo_instance.ma_long.config.ema_period = EMA_LONG;
                rikiddo_instance.ma_long.config.ema_period_estimate_after = Some(EMA_SHORT);
                let _ = T::RikiddoSigmoidFeeMarketEma::create(next_pool_id, rikiddo_instance)?;
            }

            // Sort assets for future binary search, for example to check if an asset is included.
            let sort_assets = assets.as_mut_slice();
            sort_assets.sort();
            <Pools<T>>::insert(
                next_pool_id,
                Some(Pool {
                    assets,
                    base_asset,
                    market_id,
                    pool_status: if scoring_rule == ScoringRule::CPMM {
                        PoolStatus::Active
                    } else {
                        PoolStatus::CollectingSubsidy
                    },
                    scoring_rule,
                    swap_fee,
                    total_subsidy: if scoring_rule == ScoringRule::CPMM {
                        None
                    } else {
                        Some(BalanceOf::<T>::zero())
                    },
                    total_weight: if scoring_rule == ScoringRule::CPMM {
                        Some(total_weight)
                    } else {
                        None
                    },
                    weights: if scoring_rule == ScoringRule::CPMM { Some(map) } else { None },
                }),
            );

            Self::deposit_event(Event::PoolCreate(CommonPoolEventParams {
                pool_id: next_pool_id,
                who,
            }));

            Ok(next_pool_id)
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
            asset: Asset<T::MarketId>,
            asset_amount: BalanceOf<T>,
            max_pool_amount: BalanceOf<T>,
        ) -> Result<Weight, DispatchError> {
            let pool = Self::pool_by_id(pool_id)?;
            let pool_ref = &pool;
            let who_clone = who.clone();

            let params = PoolExitWithExactAmountParams {
                asset,
                asset_amount: |_, _| Ok(asset_amount),
                bound: max_pool_amount,
                ensure_balance: |asset_balance: BalanceOf<T>| {
                    ensure!(
                        asset_amount
                            <= bmul(
                                asset_balance.saturated_into(),
                                T::MaxOutRatio::get().saturated_into()
                            )?
                            .saturated_into(),
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
                    )?
                    .saturated_into();
                    ensure!(pool_amount != Zero::zero(), Error::<T>::MathApproximation);
                    ensure!(pool_amount <= max_pool_amount, Error::<T>::LimitIn);
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
            asset_in: Asset<T::MarketId>,
            asset_amount: BalanceOf<T>,
            min_pool_amount: BalanceOf<T>,
        ) -> Result<Weight, DispatchError> {
            let pool = Pallet::<T>::pool_by_id(pool_id)?;
            let pool_ref = &pool;
            let pool_account_id = Pallet::<T>::pool_account_id(pool_id);
            let who_clone = who.clone();

            let params = PoolJoinWithExactAmountParams {
                asset: asset_in,
                asset_amount: |_, _| Ok(asset_amount),
                bound: min_pool_amount,
                pool_amount: move |asset_balance: BalanceOf<T>, total_supply: BalanceOf<T>| {
                    let mul: BalanceOf<T> = bmul(
                        asset_balance.saturated_into(),
                        T::MaxInRatio::get().saturated_into(),
                    )?
                    .saturated_into();
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
            let weight = T::WeightInfo::pool_exit_with_exact_asset_amount();
            pool_join_with_exact_amount::<_, _, _, T>(params).map(|_| weight)
        }

        fn pool(pool_id: PoolId) -> Result<Pool<Self::Balance, Self::MarketId>, DispatchError> {
            Ok(Self::pool_by_id(pool_id)?)
        }

        /// Pool will be marked as `PoolStatus::Active`, if the market is currently in subsidy
        /// state and all other conditions are met.
        ///
        /// # Arguments
        ///
        /// * `pool_id`: Unique pool identifier associated with the pool to be made active.
        /// than the given value.
        #[frame_support::transactional]
        fn set_pool_as_active(pool_id: PoolId) -> DispatchResult {
            Self::mutate_pool(pool_id, |pool| {
                // Ensure all preconditions are met.
                if pool.pool_status == PoolStatus::Active {
                    return Ok(());
                } else if pool.pool_status != PoolStatus::CollectingSubsidy {
                    return Err(Error::<T>::InvalidStateTransition.into());
                }

                let total_subsidy = pool.total_subsidy.ok_or(Error::<T>::InsufficientSubsidy)?;
                ensure!(total_subsidy >= T::MinSubsidy::get(), Error::<T>::InsufficientSubsidy);
                let base_asset = pool.base_asset.ok_or(Error::<T>::BaseAssetNotFound)?;
                let pool_account = Pallet::<T>::pool_account_id(pool_id);
                let mut account_created = false;
                let mut total_balance = <BalanceOf<T>>::zero();
                let mut providers = Vec::new();

                // Transfer all reserved funds to the pool account and distribute pool shares.
                for provider in <SubsidyProviders<T>>::drain() {
                    if !account_created {
                        T::Shares::unreserve(base_asset, &provider.1.address, provider.1.subsidy);
                        T::Shares::transfer(
                            base_asset,
                            &provider.1.address,
                            &pool_account,
                            provider.1.subsidy,
                        )?;
                        total_balance = provider.1.subsidy;
                        account_created = true;
                    }

                    let transfered = T::Shares::repatriate_reserved(
                        base_asset,
                        &provider.1.address,
                        &pool_account,
                        provider.1.subsidy,
                        BalanceStatus::Free,
                    )?;

                    total_balance.saturating_add(transfered);
                    providers.push(provider);
                }

                // This can only happen if other pallets consumed some of the reserved
                // balance for subsidy.
                if total_balance != total_subsidy {
                    ensure!(total_balance >= T::MinSubsidy::get(), Error::<T>::InsufficientSubsidy);
                    pool.total_subsidy = Some(total_balance);
                }

                // TODO:
                // - Transfer pool shares to the users (Solve cost(q) = total_balance)
                // - Emit event: pool_id, SubsidyCollected
                // - Emit event: Pool State Transition - pool_id, state
                return Ok(());
            })
        }

        /// Pool will be marked as `PoolStatus::Stale`. If market is categorical, removes everything
        /// that is not ZTG or winning assets from the selected pool.
        ///
        /// Does nothing if pool is already stale. Returns `Err` if `pool_id` does not exist.
        ///
        /// # Arguments
        ///
        /// * `market_type`: Type of the market (e.g. categorical or scalar).
        /// * `pool_id`: Unique pool identifier associated with the pool to be made stale.
        /// * `outcome_report`: The resulting outcome.
        fn set_pool_as_stale(
            market_type: &MarketType,
            pool_id: PoolId,
            outcome_report: &OutcomeReport,
        ) -> DispatchResult {
            Self::mutate_pool(pool_id, |pool| {
                if pool.pool_status == PoolStatus::Stale {
                    return Ok(());
                }

                if let MarketType::Categorical(_) = market_type {
                    if let OutcomeReport::Categorical(winning_asset_idx) = outcome_report {
                        pool.assets.retain(|el| {
                            if let Asset::CategoricalOutcome(_, idx) = el {
                                idx == winning_asset_idx
                            } else {
                                matches!(el, Asset::Ztg)
                            }
                        });
                    }
                }

                pool.pool_status = PoolStatus::Stale;

                if pool.scoring_rule == ScoringRule::RikiddoSigmoidFeeMarketEma {
                    T::RikiddoSigmoidFeeMarketEma::destroy(pool_id)?
                }

                Ok(())
            })
        }
    }
}
