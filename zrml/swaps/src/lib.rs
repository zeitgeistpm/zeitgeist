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
    use alloc::{collections::btree_map::BTreeMap, vec::Vec};
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::{DispatchResultWithPostInfo, Weight},
        ensure, log,
        pallet_prelude::{StorageDoubleMap, StorageMap, StorageValue, ValueQuery},
        storage::{with_transaction, TransactionOutcome},
        traits::{Get, IsType, StorageVersion},
        Blake2_128Concat, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_root, ensure_signed, pallet_prelude::OriginFor};
    use orml_traits::{BalanceStatus, MultiCurrency, MultiReservableCurrency};
    use parity_scale_codec::{Decode, Encode};
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedSub, Saturating, Zero},
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
            Asset, MarketType, OutcomeReport, Pool, PoolId, PoolStatus, ResultWithWeightInfo,
            ScoringRule, SerdeWrapper,
        },
    };
    use zrml_liquidity_mining::LiquidityMiningPalletApi;
    use zrml_rikiddo::{
        constants::{EMA_LONG, EMA_SHORT},
        traits::RikiddoMVPallet,
        types::{EmaMarketVolume, FeeSigmoid, RikiddoSigmoidMV},
    };

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

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
            Self::set_pool_as_stale(
                &market_type,
                pool_id,
                &outcome_report,
                &Self::pool_account_id(pool_id),
            )?;
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

        /// Pool - Remove subsidty from a pool that uses the Rikiddo scoring rule.
        ///
        /// Unreserves `pool_amount` of the base currency from being used as subsidy.
        /// If `amount` is greater than the amount reserved for subsidy by `origin`,
        /// then the whole amount reserved for subsidy will be unreserved.
        ///
        /// # Arguments
        ///
        /// * `origin`: Liquidity Provider (LP). The account whose assets should be unreserved.
        /// * `pool_id`: Unique pool identifier.
        /// * `amount`: The amount of base currency that should be removed from subsidy.
        #[pallet::weight(T::WeightInfo::pool_exit_subsidy())]
        pub fn pool_exit_subsidy(
            origin: OriginFor<T>,
            pool_id: PoolId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            <Pools<T>>::try_mutate(pool_id, |pool_opt| {
                let pool = pool_opt.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;

                ensure!(
                    pool.scoring_rule == ScoringRule::RikiddoSigmoidFeeMarketEma,
                    Error::<T>::InvalidScoringRule
                );
                let base_asset = pool.base_asset.ok_or(Error::<T>::BaseAssetNotFound)?;
                let mut real_amount = amount;
                let upper_bound;
                let transferred;

                if let Some(subsidy) = <SubsidyProviders<T>>::get(&pool_id, &who) {
                    upper_bound = subsidy;

                    if amount > subsidy {
                        real_amount = subsidy;
                    }

                    let missing = T::Shares::unreserve(base_asset, &who, real_amount);
                    transferred = real_amount.saturating_sub(missing);
                    let zero_balance = <BalanceOf<T>>::zero();

                    if missing > zero_balance {
                        log::warn!(
                            "[Swaps] Data inconsistency: More subsidy provided than currently \
                             reserved.
                        Pool: {:?}, User: {:?}, Unreserved: {:?}, Previously reserved: {:?}",
                            pool_id,
                            who,
                            transferred,
                            subsidy
                        );
                    }

                    let new_amount = subsidy.saturating_sub(transferred);
                    let total_subsidy = pool.total_subsidy.ok_or(Error::<T>::PoolMissingSubsidy)?;

                    if new_amount > zero_balance && missing == zero_balance {
                        <SubsidyProviders<T>>::insert(&pool_id, &who, new_amount);
                        pool.total_subsidy = Some(
                            total_subsidy
                                .checked_sub(&transferred)
                                .ok_or(ArithmeticError::Overflow)?,
                        );
                    } else {
                        let _ = <SubsidyProviders<T>>::take(&pool_id, &who);
                        pool.total_subsidy = Some(
                            total_subsidy.checked_sub(&subsidy).ok_or(ArithmeticError::Overflow)?,
                        );
                    }
                } else {
                    return Err(Error::<T>::NoSubsidyProvided.into());
                }

                Self::deposit_event(Event::<T>::PoolExitSubsidy(PoolAssetEvent {
                    asset: base_asset,
                    bound: upper_bound,
                    cpep: CommonPoolEventParams { pool_id, who },
                    transferred,
                }));

                Ok(())
            })
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
                    let mul: BalanceOf<T> =
                        bmul(total_supply.saturated_into(), T::MaxInRatio::get().saturated_into())?
                            .saturated_into();
                    ensure!(pool_amount <= mul, Error::<T>::MaxInRatio);
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
        ) -> DispatchResultWithPostInfo {
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

            let _ = crate::utils::pool::<_, _, _, T>(params)?;
            Ok(Some(T::WeightInfo::pool_join(pool.assets.len().saturated_into())).into())
        }

        /// Pool - Add subsidy to a pool that uses the Rikiddo scoring rule.
        ///
        /// Reserves `pool_amount` of the base currency to be added as subsidy on pool activation.
        ///
        /// # Arguments
        ///
        /// * `origin`: Liquidity Provider (LP). The account whose assets should be reserved.
        /// * `pool_id`: Unique pool identifier.
        /// * `amount`: The amount of base currency that should be added to subsidy.
        #[pallet::weight(T::WeightInfo::pool_join_subsidy())]
        pub fn pool_join_subsidy(
            origin: OriginFor<T>,
            pool_id: PoolId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            <Pools<T>>::try_mutate(pool_id, |pool_opt| {
                let pool = pool_opt.as_mut().ok_or(Error::<T>::PoolDoesNotExist)?;

                ensure!(
                    pool.scoring_rule == ScoringRule::RikiddoSigmoidFeeMarketEma,
                    Error::<T>::InvalidScoringRule
                );
                let base_asset = pool.base_asset.ok_or(Error::<T>::BaseAssetNotFound)?;
                T::Shares::reserve(base_asset, &who, amount)?;

                let total_subsidy = pool.total_subsidy.ok_or(Error::<T>::PoolMissingSubsidy)?;
                let _ = <SubsidyProviders<T>>::mutate(&pool_id, &who, |user_subsidy| {
                    if let Some(prev_val) = user_subsidy {
                        *prev_val += amount;
                    } else {
                        *user_subsidy = Some(amount);
                    }

                    pool.total_subsidy = Some(total_subsidy + amount);
                });

                Self::deposit_event(Event::<T>::PoolJoinSubsidy(PoolAssetEvent {
                    asset: base_asset,
                    bound: amount,
                    cpep: CommonPoolEventParams { pool_id, who },
                    transferred: amount,
                }));

                Ok(())
            })
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
                    let mul: BalanceOf<T> = bmul(
                        total_supply.saturated_into(),
                        T::MaxOutRatio::get().saturated_into(),
                    )?
                    .saturated_into();
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
        #[pallet::weight(T::WeightInfo::swap_exact_amount_in_rikiddo(T::MaxAssets::get().into()))]
        #[frame_support::transactional]
        pub fn swap_exact_amount_in(
            origin: OriginFor<T>,
            pool_id: PoolId,
            asset_in: Asset<T::MarketId>,
            asset_amount_in: BalanceOf<T>,
            asset_out: Asset<T::MarketId>,
            min_asset_amount_out: BalanceOf<T>,
            max_price: BalanceOf<T>,
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
        /// * `max_amount_asset_in`: Maximum asset amount that can enter the pool.
        /// * `asset_out`: Asset leaving the pool.
        /// * `asset_amount_out`: Amount that will be transferred from the pool to the provider.
        /// * `max_price`: Market price must be equal or less than the provided value.
        #[pallet::weight(T::WeightInfo::swap_exact_amount_out_rikiddo(T::MaxAssets::get().into()))]
        #[frame_support::transactional]
        pub fn swap_exact_amount_out(
            origin: OriginFor<T>,
            pool_id: PoolId,
            asset_in: Asset<T::MarketId>,
            max_amount_asset_in: BalanceOf<T>,
            asset_out: Asset<T::MarketId>,
            asset_amount_out: BalanceOf<T>,
            max_price: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let weight = <Self as Swaps<T::AccountId>>::swap_exact_amount_out(
                who,
                pool_id,
                asset_in,
                max_amount_asset_in,
                asset_out,
                asset_amount_out,
                max_price,
            )?;
            Ok(Some(weight).into())
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
            + From<u8>
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

        #[pallet::constant]
        /// The minimum amount of assets in a pool.
        type MinAssets: Get<u16>;

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
        /// The market was not started since the subsidy goal was not reached.
        InsufficientSubsidy,
        /// Could not create CPMM pool since no fee was supplied.
        InvalidFeeArgument,
        /// A function that is only valid for pools with specific scoring rules was called for a
        /// pool with another scoring rule.
        InvalidScoringRule,
        /// A function was called for a swaps pool that does not fulfill the state requirement.
        InvalidStateTransition,
        /// Could not create CPMM pool since no weights were supplied.
        InvalidWeightArgument,
        /// A transferal of funds into a swaps pool was above a threshhold specified by the sender.
        LimitIn,
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
        /// It was tried to remove subsidy from a pool which does not have subsidy provided by
        /// the address that tried to remove the subsidy.
        NoSubsidyProvided,
        /// The pool in question does not exist.
        PoolDoesNotExist,
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
        /// Tried to create a pool that has less assets than the lower threshhold specified by
        /// a constant.
        TooFewAssets,
        /// Tried to create a pool that has more assets than the upper threshhold specified by
        /// a constant.
        TooManyAssets,
        /// The pool does not support swapping the assets in question.
        UnsupportedTrade,
        /// The outcome asset specified as the winning asset was not found in the pool.
        WinningAssetNotFound,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// Share holder rewards were distributed. \[pool_id, num_accounts_rewarded, amount\]
        DistributeShareHolderRewards(PoolId, u64, BalanceOf<T>),
        /// A new pool has been created. \[CommonPoolEventParams, pool\]
        PoolCreate(
            CommonPoolEventParams<<T as frame_system::Config>::AccountId>,
            Pool<BalanceOf<T>, T::MarketId>,
        ),
        /// Someone has exited a pool. \[PoolAssetsEvent\]
        PoolExit(
            PoolAssetsEvent<
                <T as frame_system::Config>::AccountId,
                Asset<T::MarketId>,
                BalanceOf<T>,
            >,
        ),
        /// Someone has (partially) exited a pool by removing subsidy. \[PoolAssetEvent, amount\]
        PoolExitSubsidy(
            PoolAssetEvent<
                <T as frame_system::Config>::AccountId,
                Asset<T::MarketId>,
                BalanceOf<T>,
            >,
        ),
        /// Exits a pool given an exact amount of an asset. \[PoolAssetEvent\]
        PoolExitWithExactAssetAmount(
            PoolAssetEvent<
                <T as frame_system::Config>::AccountId,
                Asset<T::MarketId>,
                BalanceOf<T>,
            >,
        ),
        /// Exits a pool given an exact pool's amount. \[PoolAssetEvent\]
        PoolExitWithExactPoolAmount(
            PoolAssetEvent<
                <T as frame_system::Config>::AccountId,
                Asset<T::MarketId>,
                BalanceOf<T>,
            >,
        ),
        /// Someone has joined a pool. \[PoolAssetsEvent\]
        PoolJoin(
            PoolAssetsEvent<
                <T as frame_system::Config>::AccountId,
                Asset<T::MarketId>,
                BalanceOf<T>,
            >,
        ),
        /// Someone has joined a pool by providing subsidy. \[PoolAssetEvent, amount\]
        PoolJoinSubsidy(
            PoolAssetEvent<
                <T as frame_system::Config>::AccountId,
                Asset<T::MarketId>,
                BalanceOf<T>,
            >,
        ),
        /// Joins a pool given an exact amount of an asset. \[PoolAssetEvent\]
        PoolJoinWithExactAssetAmount(
            PoolAssetEvent<
                <T as frame_system::Config>::AccountId,
                Asset<T::MarketId>,
                BalanceOf<T>,
            >,
        ),
        /// Joins a pool given an exact pool's amount. \[PoolAssetEvent\]
        PoolJoinWithExactPoolAmount(
            PoolAssetEvent<
                <T as frame_system::Config>::AccountId,
                Asset<T::MarketId>,
                BalanceOf<T>,
            >,
        ),
        /// Total subsidy collected for a pool. \[pool_id, subsidy\]
        SubsidyCollected(PoolId, BalanceOf<T>),
        /// An exact amount of an asset is entering the pool. \[SwapEvent\]
        SwapExactAmountIn(
            SwapEvent<<T as frame_system::Config>::AccountId, Asset<T::MarketId>, BalanceOf<T>>,
        ),
        /// An exact amount of an asset is leaving the pool. \[SwapEvent\]
        SwapExactAmountOut(
            SwapEvent<<T as frame_system::Config>::AccountId, Asset<T::MarketId>, BalanceOf<T>>,
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
        Option<Pool<BalanceOf<T>, T::MarketId>>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn subsidy_providers)]
    pub type SubsidyProviders<T: Config> =
        StorageDoubleMap<_, Twox64Concat, PoolId, Twox64Concat, T::AccountId, BalanceOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn next_pool_id)]
    pub type NextPoolId<T> = StorageValue<_, PoolId, ValueQuery>;

    impl<T: Config> Pallet<T> {
        pub(crate) fn distribute_pool_share_rewards(
            pool: &Pool<BalanceOf<T>, T::MarketId>,
            pool_id: PoolId,
            base_asset: Asset<T::MarketId>,
            winning_asset: Asset<T::MarketId>,
            winner_payout_account: &T::AccountId,
        ) -> Weight {
            // CPMM handling of market profit not supported
            if pool.scoring_rule == ScoringRule::CPMM {
                return 1_000_000;
            }

            // Total pool shares
            let shares_id = Self::pool_shares_id(pool_id);
            let total_pool_shares = T::Shares::total_issuance(shares_id);

            // Total AMM balance
            let pool_account = Self::pool_account_id(pool_id);
            let total_amm_funds = T::Shares::free_balance(base_asset, &pool_account);

            // Total winning shares
            // The pool account still holds the winning asset, burn it.
            let free_balance = T::Shares::free_balance(winning_asset, &pool_account);
            let _ = T::Shares::withdraw(winning_asset, &pool_account, free_balance);
            let total_winning_assets = T::Shares::total_issuance(winning_asset);

            // Profit = AMM balance - total winning shares
            let amm_profit_checked = total_amm_funds.checked_sub(&total_winning_assets);
            let amm_profit = if let Some(profit) = amm_profit_checked {
                profit
            } else {
                // In case the AMM balance does not suffice to pay out every winner, the pool
                // rewards will not be distributed (requires investigation and update).
                log::error!(
                    "[Swaps] The AMM balance does not suffice to pay out every winner.
                    market_id: {:?}, pool_id: {:?}, total AMM balance: {:?}, total reward value: \
                     {:?}",
                    pool.market_id,
                    pool_id,
                    total_amm_funds,
                    total_winning_assets
                );
                return T::DbWeight::get().reads(6);
            };

            // Iterate through every share holder and exchange shares for rewards.
            let (total_accounts_num, share_accounts) =
                T::Shares::accounts_by_currency_id(shares_id);
            let share_accounts_num = share_accounts.len();

            for share_holder in share_accounts {
                let share_holder_account = share_holder.0;
                let share_holder_balance = share_holder.1.free;
                let reward_pct_unadjusted =
                    bdiv(share_holder_balance.saturated_into(), total_pool_shares.saturated_into())
                        .unwrap_or(0);

                // Seems like bdiv does arithmetic rounding. To ensure that we will have enough
                // reward for everyone and not run into an error, we'll round down in any case.
                let reward_pct = reward_pct_unadjusted.saturating_sub(1);
                let holder_reward_unadjusted =
                    bmul(amm_profit.saturated_into(), reward_pct).unwrap_or(0);

                // Same for bmul.
                let holder_reward = holder_reward_unadjusted.saturating_sub(1);

                let transfer_result = T::Shares::transfer(
                    base_asset,
                    &pool_account,
                    &share_holder_account,
                    holder_reward.saturated_into(),
                );

                // Should be impossible.
                if let Err(err) = transfer_result {
                    let current_amm_holding = T::Shares::free_balance(base_asset, &pool_account);
                    log::error!(
                        "[Swaps] The AMM failed to pay out the share holder reward.
                        market_id: {:?}, pool_id: {:?}, current AMM holding: {:?},
                        transfer size: {:?}, to: {:?}, error: {:?}",
                        pool.market_id,
                        pool_id,
                        current_amm_holding,
                        holder_reward,
                        share_holder_account,
                        err,
                    );

                    if current_amm_holding < holder_reward.saturated_into() {
                        let _ = T::Shares::transfer(
                            base_asset,
                            &pool_account,
                            &share_holder_account,
                            current_amm_holding.saturated_into(),
                        );
                    }
                }

                // We can use the lightweight withdraw here, since pool shares are not reserved.
                // We can ignore the result since the balance to transfer is the query result of
                // the free balance (always sufficient).
                let _ = T::Shares::withdraw(shares_id, &share_holder_account, share_holder_balance);
            }

            let remaining_pool_funds = T::Shares::free_balance(base_asset, &pool_account);
            // Transfer winner payout - Infallible since the balance was just read from storage.
            let _ = T::Shares::transfer(
                base_asset,
                &pool_account,
                winner_payout_account,
                remaining_pool_funds,
            );

            Self::deposit_event(Event::<T>::DistributeShareHolderRewards(
                pool_id,
                share_accounts_num.saturated_into(),
                amm_profit,
            ));

            T::WeightInfo::distribute_pool_share_rewards(
                total_accounts_num.saturated_into(),
                share_accounts_num.saturated_into(),
            )
        }

        pub fn get_spot_price(
            pool_id: PoolId,
            asset_in: Asset<T::MarketId>,
            asset_out: Asset<T::MarketId>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let pool = Self::pool_by_id(pool_id)?;
            ensure!(pool.assets.binary_search(&asset_in).is_ok(), Error::<T>::AssetNotInPool);
            ensure!(pool.assets.binary_search(&asset_out).is_ok(), Error::<T>::AssetNotInPool);
            let pool_account = Self::pool_account_id(pool_id);

            if pool.scoring_rule == ScoringRule::CPMM {
                let balance_in = T::Shares::free_balance(asset_in, &pool_account);
                let balance_out = T::Shares::free_balance(asset_out, &pool_account);
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
            ensure!(pool.pool_status == PoolStatus::Active, Error::<T>::PoolIsNotActive);
            let mut balances = Vec::new();
            let base_asset = pool.base_asset.ok_or(Error::<T>::BaseAssetNotFound)?;

            // Fees are estimated here. The error scales with the fee. For the future, we'll have
            // to figure out how to extract the fee out of the price when using Rikiddo.
            if asset_in == asset_out {
                return Ok(T::RikiddoSigmoidFeeMarketEma::fee(pool_id)?
                    .saturating_add(BASE.saturated_into()));
            }

            let mut balance_in = <BalanceOf<T>>::zero();
            let mut balance_out = <BalanceOf<T>>::zero();

            for asset in pool.assets.iter().filter(|asset| **asset != base_asset) {
                let issuance = T::Shares::total_issuance(*asset);

                if *asset == asset_in {
                    balance_in = issuance;
                } else if *asset == asset_out {
                    balance_out = issuance;
                }

                balances.push(issuance);
            }

            if asset_in == base_asset {
                T::RikiddoSigmoidFeeMarketEma::price(pool_id, balance_out, &balances)
            } else if asset_out == base_asset {
                let price_with_inverse_fee = bdiv(
                    BASE,
                    T::RikiddoSigmoidFeeMarketEma::price(pool_id, balance_in, &balances)?
                        .saturated_into(),
                )?
                .saturated_into();
                let fee_pct = T::RikiddoSigmoidFeeMarketEma::fee(pool_id)?.saturated_into();
                let fee_plus_one = BASE.saturating_add(fee_pct);
                let price_with_fee: u128 =
                    bmul(fee_plus_one, bmul(price_with_inverse_fee, fee_plus_one)?)?;
                Ok(price_with_fee.saturated_into())
            } else {
                let price_without_fee = bdiv(
                    T::RikiddoSigmoidFeeMarketEma::price(pool_id, balance_out, &balances)?
                        .saturated_into(),
                    T::RikiddoSigmoidFeeMarketEma::price(pool_id, balance_in, &balances)?
                        .saturated_into(),
                )?
                .saturated_into();
                let fee_pct = T::RikiddoSigmoidFeeMarketEma::fee(pool_id)?.saturated_into();
                let fee_plus_one = BASE.saturating_add(fee_pct);
                let price_with_fee: u128 = bmul(fee_plus_one, price_without_fee)?;
                Ok(price_with_fee.saturated_into())
            }
        }

        pub fn pool_account_id(pool_id: PoolId) -> T::AccountId {
            T::PalletId::get().into_sub_account(pool_id.saturated_into::<u128>())
        }

        pub(crate) fn burn_pool_shares(
            pool_id: PoolId,
            from: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let shares_id = Self::pool_shares_id(pool_id);
            // Check that the account has at least as many free shares as we wish to burn!
            T::Shares::ensure_can_withdraw(shares_id, from, amount)
                .map_err(|_| Error::<T>::InsufficientBalance)?;
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
        pub(crate) fn mutate_pool<F>(pool_id: PoolId, mut cb: F) -> DispatchResult
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

        fn _set_pool_as_stale_common(pool_id: PoolId) -> Result<Weight, DispatchError> {
            Self::mutate_pool(pool_id, |pool| {
                if pool.pool_status == PoolStatus::Stale {
                    return Ok(());
                }
                ensure!(pool.pool_status == PoolStatus::Active, Error::<T>::InvalidStateTransition);
                pool.pool_status = PoolStatus::Stale;
                Ok(())
            })?;
            Ok(0) // TODO Needs benchmark!
        }

        fn _set_pool_as_stale_categorical(
            pool_id: PoolId,
            outcome_report: &OutcomeReport,
            winner_payout_account: &T::AccountId,
        ) -> Result<Weight, DispatchError> {
            let mut extra_weight = 0;
            let mut total_assets = 0;

            Self::mutate_pool(pool_id, |pool| {
                // Find winning asset, remove losing assets from pool
                let base_asset = pool.base_asset.ok_or(Error::<T>::BaseAssetNotFound)?;
                let mut winning_asset: Result<_, DispatchError> =
                    Err(Error::<T>::WinningAssetNotFound.into());
                if let OutcomeReport::Categorical(winning_asset_idx) = outcome_report {
                    pool.assets.retain(|el| {
                        if let Asset::CategoricalOutcome(_, idx) = *el {
                            if idx == *winning_asset_idx {
                                winning_asset = Ok(*el);
                                return true;
                            };
                        }

                        *el == base_asset
                    });
                }

                total_assets = pool.assets.len();

                let winning_asset_unwrapped = winning_asset?;

                if pool.scoring_rule == ScoringRule::RikiddoSigmoidFeeMarketEma {
                    T::RikiddoSigmoidFeeMarketEma::destroy(pool_id)?;
                    let distribute_weight = Self::distribute_pool_share_rewards(
                        pool,
                        pool_id,
                        base_asset,
                        winning_asset_unwrapped,
                        winner_payout_account,
                    );
                    extra_weight = extra_weight.saturating_add(T::DbWeight::get().writes(1));
                    extra_weight = extra_weight.saturating_add(distribute_weight);
                }

                Ok(())
            })?;

            Ok(T::WeightInfo::set_pool_as_stale_without_reward_distribution(
                total_assets.saturated_into(),
            )
            .saturating_add(extra_weight))
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
        ///   Default is `Asset::Ztg`.
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
            ensure!(assets.len() <= usize::from(T::MaxAssets::get()), Error::<T>::TooManyAssets);
            ensure!(assets.len() >= usize::from(T::MinAssets::get()), Error::<T>::TooFewAssets);
            let amount = T::MinLiquidity::get();
            let next_pool_id = Self::inc_next_pool_id()?;
            let pool_shares_id = Self::pool_shares_id(next_pool_id);
            let pool_account = Self::pool_account_id(next_pool_id);
            let mut map = BTreeMap::new();
            let mut total_weight = 0;
            let base_asset_unwrapped = base_asset.unwrap_or(Asset::Ztg);
            ensure!(assets.contains(&base_asset_unwrapped), Error::<T>::BaseAssetNotFound);

            if scoring_rule == ScoringRule::CPMM {
                let _ = swap_fee.ok_or(Error::<T>::InvalidFeeArgument)?;
                let weights_unwrapped = weights.ok_or(Error::<T>::InvalidWeightArgument)?;
                Self::check_provided_values_len_must_equal_assets_len(&assets, &weights_unwrapped)?;

                for (asset, weight) in assets.iter().copied().zip(weights_unwrapped) {
                    let free_balance = T::Shares::free_balance(asset, &who);
                    ensure!(free_balance >= amount, Error::<T>::InsufficientBalance);
                    ensure!(weight >= T::MinWeight::get(), Error::<T>::BelowMinimumWeight);
                    ensure!(weight <= T::MaxWeight::get(), Error::<T>::AboveMaximumWeight);
                    map.insert(asset, weight);
                    total_weight = total_weight.check_add_rslt(&weight)?;
                    T::Shares::transfer(asset, &who, &pool_account, amount)?;
                }

                ensure!(total_weight <= T::MaxTotalWeight::get(), Error::<T>::MaxTotalWeight);
                T::Shares::deposit(pool_shares_id, &who, amount)?;
            } else {
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
            let pool = Pool {
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
            };

            <Pools<T>>::insert(next_pool_id, Some(pool.clone()));

            Self::deposit_event(Event::PoolCreate(
                CommonPoolEventParams { pool_id: next_pool_id, who },
                pool,
            ));

            Ok(next_pool_id)
        }

        /// All supporters will receive their reserved funds back and the pool is destroyed.
        ///
        /// # Arguments
        ///
        /// * `pool_id`: Unique pool identifier associated with the pool to be destroyed.
        fn destroy_pool_in_subsidy_phase(pool_id: PoolId) -> Result<Weight, DispatchError> {
            let mut total_providers = 0usize;

            Self::mutate_pool(pool_id, |pool| {
                // Ensure all preconditions are met.
                if pool.pool_status != PoolStatus::CollectingSubsidy {
                    return Err(Error::<T>::InvalidStateTransition.into());
                }

                let base_asset = pool.base_asset.ok_or(Error::<T>::BaseAssetNotFound)?;

                for provider in <SubsidyProviders<T>>::drain_prefix(pool_id) {
                    T::Shares::unreserve(base_asset, &provider.0, provider.1);
                    total_providers = total_providers.saturating_add(1);
                }

                if pool.scoring_rule == ScoringRule::RikiddoSigmoidFeeMarketEma {
                    T::RikiddoSigmoidFeeMarketEma::destroy(pool_id)?
                }

                Ok(())
            })?;

            Pools::<T>::remove(pool_id);
            Ok(T::WeightInfo::destroy_pool_in_subsidy_phase(total_providers.saturated_into()))
        }

        /// Pool will be marked as `PoolStatus::Active`, if the market is currently in subsidy
        /// state and all other conditions are met. Returns result=true if everything succeeded,
        /// result=false if not enough subsidy was collected and an error in all other cases.
        ///
        /// # Arguments
        ///
        /// * `pool_id`: Unique pool identifier associated with the pool to be made active.
        /// than the given value.
        fn end_subsidy_phase(pool_id: PoolId) -> Result<ResultWithWeightInfo<bool>, DispatchError> {
            let do_mutate = || {
                let mut total_providers = 0usize;
                let mut total_assets = 0;

                let result = Self::mutate_pool(pool_id, |pool| {
                    // Ensure all preconditions are met.
                    if pool.pool_status != PoolStatus::CollectingSubsidy {
                        return Err(Error::<T>::InvalidStateTransition.into());
                    }

                    let total_subsidy = pool.total_subsidy.ok_or(Error::<T>::PoolMissingSubsidy)?;
                    ensure!(total_subsidy >= T::MinSubsidy::get(), Error::<T>::InsufficientSubsidy);
                    let base_asset = pool.base_asset.ok_or(Error::<T>::BaseAssetNotFound)?;
                    let pool_account = Pallet::<T>::pool_account_id(pool_id);
                    let pool_shares_id = Self::pool_shares_id(pool_id);
                    let mut account_created = false;
                    let mut total_balance = <BalanceOf<T>>::zero();
                    total_assets = pool.assets.len();

                    // Transfer all reserved funds to the pool account and distribute pool shares.
                    for provider in <SubsidyProviders<T>>::drain_prefix(pool_id) {
                        total_providers = total_providers.saturating_add(1);
                        let provider_address = provider.0;
                        let subsidy = provider.1;

                        if !account_created {
                            T::Shares::unreserve(base_asset, &provider_address, subsidy);
                            T::Shares::transfer(
                                base_asset,
                                &provider_address,
                                &pool_account,
                                subsidy,
                            )?;
                            total_balance = subsidy;
                            T::Shares::deposit(pool_shares_id, &provider_address, subsidy)?;
                            account_created = true;
                            continue;
                        }

                        let remaining = T::Shares::repatriate_reserved(
                            base_asset,
                            &provider_address,
                            &pool_account,
                            subsidy,
                            BalanceStatus::Free,
                        )?;
                        let transfered = subsidy.saturating_sub(remaining);

                        if transfered != subsidy {
                            log::warn!(
                                "[Swaps] Data inconsistency: In end_subsidy_phase - More subsidy \
                                 provided than currently reserved.
                            Pool: {:?}, User: {:?}, Unreserved: {:?}, Previously reserved: {:?}",
                                pool_id,
                                provider_address,
                                transfered,
                                subsidy
                            );
                        }

                        T::Shares::deposit(pool_shares_id, &provider_address, transfered)?;
                        total_balance.saturating_add(transfered);
                    }

                    ensure!(total_balance >= T::MinSubsidy::get(), Error::<T>::InsufficientSubsidy);
                    pool.total_subsidy = Some(total_balance);

                    // Assign the initial set of outstanding assets to the pool account.
                    let outstanding_assets_per_event =
                        T::RikiddoSigmoidFeeMarketEma::initial_outstanding_assets(
                            pool_id,
                            pool.assets.len().saturated_into::<u32>().saturating_sub(1),
                            total_balance,
                        )?;

                    for asset in pool.assets.iter().filter(|e| **e != base_asset) {
                        T::Shares::deposit(*asset, &pool_account, outstanding_assets_per_event)?;
                    }

                    pool.pool_status = PoolStatus::Active;
                    Self::deposit_event(Event::SubsidyCollected(pool_id, total_balance));
                    Ok(())
                });

                if let Err(err) = result {
                    if err == Error::<T>::InsufficientSubsidy.into() {
                        return Ok(ResultWithWeightInfo {
                            result: false,
                            weight: T::WeightInfo::end_subsidy_phase(
                                total_assets.saturated_into(),
                                total_providers.saturated_into(),
                            ),
                        });
                    }

                    return Err(err);
                }

                Ok(ResultWithWeightInfo {
                    result: true,
                    weight: T::WeightInfo::end_subsidy_phase(
                        total_assets.saturated_into(),
                        total_providers.saturated_into(),
                    ),
                })
            };

            with_transaction(|| {
                let output = do_mutate();
                match output {
                    Ok(res) => {
                        if res.result {
                            TransactionOutcome::Commit(Ok(res))
                        } else {
                            TransactionOutcome::Rollback(Ok(res))
                        }
                    }
                    Err(err) => TransactionOutcome::Rollback(Err(err)),
                }
            })
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

        /// Mark a pool as stale, remove losing assets and distribute Rikiddo pool share rewards.
        ///
        /// # Arguments
        ///
        /// * `market_type`: Type of the market
        /// * `pool_id`: Unique pool identifier associated with the pool to be made stale.
        /// * `outcome_report`: The resulting outcome.
        /// * `winner_payout_account`: The account that exchanges winning assets against rewards.
        ///
        /// # Errors
        ///
        /// * Returns `Error::<T>::PoolDoesNotExist` if there is no pool with `pool_id`.
        /// * Returns `Error::<T>::WinningAssetNotFound` if the reported asset is not found in the
        ///   pool and the scoring rule is Rikiddo.
        /// * Returns `Error::<T>::InvalidStateTransition` if the pool is not active or already
        ///   stale
        #[frame_support::transactional]
        fn set_pool_as_stale(
            market_type: &MarketType,
            pool_id: PoolId,
            outcome_report: &OutcomeReport,
            winner_payout_account: &T::AccountId,
        ) -> Result<Weight, DispatchError> {
            let mut weight = 0;
            weight += Self::_set_pool_as_stale_common(pool_id)?;
            if let MarketType::Categorical(_) = market_type {
                weight += Self::_set_pool_as_stale_categorical(
                    pool_id,
                    outcome_report,
                    winner_payout_account,
                )?;
            }
            // (No extra work required for scalar markets!)
            Ok(weight)
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
        fn swap_exact_amount_in(
            who: T::AccountId,
            pool_id: PoolId,
            asset_in: Asset<T::MarketId>,
            asset_amount_in: BalanceOf<T>,
            asset_out: Asset<T::MarketId>,
            min_asset_amount_out: BalanceOf<T>,
            max_price: BalanceOf<T>,
        ) -> Result<Weight, DispatchError> {
            let pool = Pallet::<T>::pool_by_id(pool_id)?;
            let pool_account_id = Pallet::<T>::pool_account_id(pool_id);
            ensure!(
                T::Shares::free_balance(asset_in, &who) >= asset_amount_in,
                Error::<T>::InsufficientBalance
            );

            let params = SwapExactAmountParams {
                asset_amounts: || {
                    let asset_amount_out = if pool.scoring_rule == ScoringRule::CPMM {
                        let balance_out = T::Shares::free_balance(asset_out, &pool_account_id);
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
                        crate::math::calc_out_given_in(
                            balance_in.saturated_into(),
                            Self::pool_weight_rslt(&pool, &asset_in)?,
                            balance_out.saturated_into(),
                            Self::pool_weight_rslt(&pool, &asset_out)?,
                            asset_amount_in.saturated_into(),
                            pool.swap_fee.ok_or(Error::<T>::PoolMissingFee)?.saturated_into(),
                        )?
                        .saturated_into()
                    } else {
                        let base_asset = pool.base_asset.ok_or(Error::<T>::BaseAssetNotFound)?;
                        ensure!(asset_out == base_asset, Error::<T>::UnsupportedTrade);
                        ensure!(asset_in != asset_out, Error::<T>::UnsupportedTrade);

                        let mut outstanding_before =
                            Vec::<BalanceOf<T>>::with_capacity(pool.assets.len().saturating_sub(1));
                        let mut outstanding_after =
                            Vec::<BalanceOf<T>>::with_capacity(pool.assets.len().saturating_sub(1));

                        for asset in pool.assets.iter().filter(|e| **e != base_asset) {
                            let total_amount = T::Shares::total_issuance(*asset);
                            outstanding_before.push(total_amount);

                            if *asset == asset_in {
                                outstanding_after
                                    .push(total_amount.saturating_sub(asset_amount_in));
                            } else {
                                outstanding_after.push(total_amount);
                            }
                        }

                        let cost_before =
                            T::RikiddoSigmoidFeeMarketEma::cost(pool_id, &outstanding_before)?;
                        let cost_after =
                            T::RikiddoSigmoidFeeMarketEma::cost(pool_id, &outstanding_after)?;
                        cost_before.checked_sub(&cost_after).ok_or(ArithmeticError::Overflow)?
                    };
                    ensure!(asset_amount_out >= min_asset_amount_out, Error::<T>::LimitOut);

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
                who,
            };
            let _ = swap_exact_amount::<_, _, T>(params)?;

            if pool.scoring_rule == ScoringRule::CPMM {
                Ok(T::WeightInfo::swap_exact_amount_in_cpmm())
            } else {
                Ok(T::WeightInfo::swap_exact_amount_in_rikiddo(pool.assets.len().saturated_into()))
            }
        }

        fn swap_exact_amount_out(
            who: T::AccountId,
            pool_id: PoolId,
            asset_in: Asset<T::MarketId>,
            max_amount_asset_in: BalanceOf<T>,
            asset_out: Asset<T::MarketId>,
            asset_amount_out: BalanceOf<T>,
            max_price: BalanceOf<T>,
        ) -> Result<Weight, DispatchError> {
            let pool = Pallet::<T>::pool_by_id(pool_id)?;
            let pool_account_id = Pallet::<T>::pool_account_id(pool_id);
            let params = SwapExactAmountParams {
                asset_amounts: || {
                    let balance_out = T::Shares::free_balance(asset_out, &pool_account_id);
                    let asset_amount_in = if pool.scoring_rule == ScoringRule::CPMM {
                        ensure!(
                            asset_amount_out
                                <= bmul(
                                    balance_out.saturated_into(),
                                    T::MaxOutRatio::get().saturated_into()
                                )?
                                .saturated_into(),
                            Error::<T>::MaxOutRatio,
                        );

                        let balance_in = T::Shares::free_balance(asset_in, &pool_account_id);
                        crate::math::calc_in_given_out(
                            balance_in.saturated_into(),
                            Self::pool_weight_rslt(&pool, &asset_in)?,
                            balance_out.saturated_into(),
                            Self::pool_weight_rslt(&pool, &asset_out)?,
                            asset_amount_out.saturated_into(),
                            pool.swap_fee.ok_or(Error::<T>::PoolMissingFee)?.saturated_into(),
                        )?
                        .saturated_into()
                    } else {
                        let base_asset = pool.base_asset.ok_or(Error::<T>::BaseAssetNotFound)?;
                        ensure!(asset_in == base_asset, Error::<T>::UnsupportedTrade);
                        ensure!(asset_in != asset_out, Error::<T>::UnsupportedTrade);

                        let mut outstanding_before =
                            Vec::<BalanceOf<T>>::with_capacity(pool.assets.len().saturating_sub(1));
                        let mut outstanding_after =
                            Vec::<BalanceOf<T>>::with_capacity(pool.assets.len().saturating_sub(1));

                        for asset in pool.assets.iter().filter(|e| **e != base_asset) {
                            let total_amount = T::Shares::total_issuance(*asset);
                            outstanding_before.push(total_amount);

                            if *asset == asset_out {
                                outstanding_after.push(total_amount + asset_amount_out);
                            } else {
                                outstanding_after.push(total_amount);
                            }
                        }

                        let cost_before =
                            T::RikiddoSigmoidFeeMarketEma::cost(pool_id, &outstanding_before)?;
                        let cost_after =
                            T::RikiddoSigmoidFeeMarketEma::cost(pool_id, &outstanding_after)?;
                        cost_after.checked_sub(&cost_before).ok_or(ArithmeticError::Overflow)?
                    };

                    ensure!(asset_amount_in <= max_amount_asset_in, Error::<T>::LimitIn);
                    Ok([asset_amount_in, asset_amount_out])
                },
                asset_bound: max_amount_asset_in,
                asset_in,
                asset_out,
                event: |evt| Self::deposit_event(Event::SwapExactAmountOut(evt)),
                max_price,
                pool_account_id: &pool_account_id,
                pool_id,
                pool: &pool,
                who,
            };
            let _ = swap_exact_amount::<_, _, T>(params)?;

            if pool.scoring_rule == ScoringRule::CPMM {
                Ok(T::WeightInfo::swap_exact_amount_out_cpmm())
            } else {
                Ok(T::WeightInfo::swap_exact_amount_out_rikiddo(pool.assets.len().saturated_into()))
            }
        }
    }
}
