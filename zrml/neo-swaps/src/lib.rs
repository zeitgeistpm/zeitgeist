// Copyright 2023 Forecasting Technologies LTD.
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

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod benchmarking;
mod consts;
mod math;
mod mock;
mod tests;
pub mod traits;
pub mod types;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{
        consts::MAX_ASSETS,
        math::{Math, MathOps},
        traits::{pool_operations::PoolOperations, LiquiditySharesManager},
        types::{FeeDistribution, Pool, SoloLp},
        weights::*,
    };
    use alloc::{collections::BTreeMap, vec, vec::Vec};
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        ensure,
        pallet_prelude::StorageMap,
        require_transactional,
        traits::{Get, IsType, StorageVersion},
        transactional, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use orml_traits::MultiCurrency;
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedSub, Saturating, Zero},
        DispatchError, DispatchResult, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        constants::{BASE, CENT},
        math::{
            checked_ops_res::{CheckedAddRes, CheckedSubRes},
            fixed::{FixedDiv, FixedMul},
        },
        traits::{CompleteSetOperationsApi, DeployPoolApi, DistributeFees},
        types::{Asset, MarketStatus, MarketType, ScalarPosition, ScoringRule},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    // These should not be config parameters to avoid misconfigurations.
    pub(crate) const MIN_SWAP_FEE: u128 = BASE / 1_000; // 0.1%.
    pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);
    pub(crate) const MAX_SPOT_PRICE: u128 = BASE - CENT / 2;
    pub(crate) const MIN_SPOT_PRICE: u128 = CENT / 2;
    pub(crate) const MIN_LIQUIDITY: u128 = BASE;

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type AssetOf<T> = Asset<MarketIdOf<T>>;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
    pub(crate) type AssetIndexType = u16;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type PoolOf<T> = Pool<T, SoloLp<T>>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type CompleteSetOperations: CompleteSetOperationsApi<
                AccountId = Self::AccountId,
                Balance = BalanceOf<Self>,
                MarketId = MarketIdOf<Self>,
            >;

        /// Distribute external fees. The fees are paid from the pool account, which in turn has
        /// received the fees from the trader.
        type ExternalFees: DistributeFees<
                Asset = AssetOf<Self>,
                AccountId = AccountIdOf<Self>,
                Balance = BalanceOf<Self>,
                MarketId = MarketIdOf<Self>,
            >;

        type MarketCommons: MarketCommonsPalletApi<AccountId = Self::AccountId, BlockNumber = Self::BlockNumber>;

        type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = AssetOf<Self>>;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type WeightInfo: WeightInfoZeitgeist;

        #[pallet::constant]
        type MaxSwapFee: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    #[pallet::getter(fn pools)]
    pub type Pools<T: Config> = StorageMap<_, Twox64Concat, MarketIdOf<T>, PoolOf<T>>;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// Informant bought a position. `amount_in` is the amount of collateral paid by `who`,
        /// including swap and external fees.
        BuyExecuted {
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            asset_out: AssetOf<T>,
            amount_in: BalanceOf<T>,
            amount_out: BalanceOf<T>,
            swap_fee_amount: BalanceOf<T>,
            external_fee_amount: BalanceOf<T>,
        },
        /// Informant sold a position. `amount_out` is the amount of collateral received by `who`,
        /// including swap and external fees.
        SellExecuted {
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            asset_in: AssetOf<T>,
            amount_in: BalanceOf<T>,
            amount_out: BalanceOf<T>,
            swap_fee_amount: BalanceOf<T>,
            external_fee_amount: BalanceOf<T>,
        },
        /// Liquidity provider withdrew fees.
        FeesWithdrawn { who: T::AccountId, market_id: MarketIdOf<T>, amount: BalanceOf<T> },
        /// Liquidity provider joined the pool.
        JoinExecuted {
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            pool_shares_amount: BalanceOf<T>,
            amounts_in: Vec<BalanceOf<T>>,
            new_liquidity_parameter: BalanceOf<T>,
        },
        /// Liquidity provider left the pool.
        ExitExecuted {
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            pool_shares_amount: BalanceOf<T>,
            amounts_out: Vec<BalanceOf<T>>,
            new_liquidity_parameter: BalanceOf<T>,
        },
        /// Pool was createed.
        PoolDeployed {
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            account_id: T::AccountId,
            reserves: BTreeMap<AssetOf<T>, BalanceOf<T>>,
            collateral: AssetOf<T>,
            liquidity_parameter: BalanceOf<T>,
            pool_shares_amount: BalanceOf<T>,
            swap_fee: BalanceOf<T>,
        },
        /// Pool was destroyed.
        PoolDestroyed {
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            amounts_out: Vec<BalanceOf<T>>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The number of assets in the pool is above the allowed maximum.
        AssetCountAboveMax,
        /// Amount paid is above the specified maximum.
        AmountInAboveMax,
        /// Amount received is below the specified minimum.
        AmountOutBelowMin,
        /// Specified asset was not found in this pool.
        AssetNotFound,
        /// Market already has an associated pool.
        DuplicatePool,
        /// Incorrect asset count.
        IncorrectAssetCount,
        // Length of `max_amounts_in`, `max_amounts_out` or `spot_prices` must be equal to the
        // number of outcomes in the market.
        IncorrectVecLen,
        /// User doesn't own enough pool shares.
        InsufficientPoolShares,
        /// The liquidity in the pool is too low.
        LiquidityTooLow,
        /// Sum of spot prices is not `1`.
        InvalidSpotPrices,
        /// Market's trading mechanism is not LMSR.
        InvalidTradingMechanism,
        /// Pool can only be traded on if the market is active.
        MarketNotActive,
        /// Deploying pools is only supported for scalar or binary markets.
        MarketNotBinaryOrScalar,
        /// Some calculation failed. This shouldn't happen.
        MathError,
        /// The user is not allowed to execute this command.
        NotAllowed,
        /// This feature is not yet implemented.
        NotImplemented,
        /// Some value in the operation is too large or small.
        NumericalLimits,
        /// Outstanding fees prevent liquidity withdrawal.
        OutstandingFees,
        /// Specified market does not have a pool.
        PoolNotFound,
        /// Spot price is above the allowed maximum.
        SpotPriceAboveMax,
        /// Spot price is below the allowed minimum.
        SpotPriceBelowMin,
        /// Pool's swap fee exceeds the allowed upper limit.
        SwapFeeAboveMax,
        /// Pool's swap fee is below the allowed lower limit.
        SwapFeeBelowMin,
        /// This shouldn't happen.
        Unexpected,
        /// Specified monetary amount is zero.
        ZeroAmount,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Buy outcome tokens from the specified market.
        ///
        /// The `amount_in` is paid in collateral. The transaction fails if the amount of outcome
        /// tokens received is smaller than `min_amount_out`. The user must correctly specify the
        /// number of outcomes for benchmarking reasons.
        ///
        /// # Parameters
        ///
        /// - `origin`: The origin account making the purchase.
        /// - `market_id`: Identifier for the market related to the trade.
        /// - `asset_count`: Number of assets in the pool.
        /// - `asset_out`: Asset to be purchased.
        /// - `amount_in`: Amount of collateral paid by the user.
        /// - `min_amount_out`: Minimum number of outcome tokens the user expects to receive.
        ///
        /// # Complexity
        ///
        /// Depends on the implementation of `CompleteSetOperationsApi` and `ExternalFees`; when
        /// using the canonical implementations, the runtime complexity is `O(asset_count)`.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::buy())]
        #[transactional]
        pub fn buy(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            asset_count: AssetIndexType,
            asset_out: AssetOf<T>,
            #[pallet::compact] amount_in: BalanceOf<T>,
            #[pallet::compact] min_amount_out: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let asset_count_real = T::MarketCommons::market(&market_id)?.outcomes();
            ensure!(asset_count == asset_count_real, Error::<T>::IncorrectAssetCount);
            Self::do_buy(who, market_id, asset_out, amount_in, min_amount_out)?;
            Ok(Some(T::WeightInfo::buy()).into())
        }

        /// Sell outcome tokens to the specified market.
        ///
        /// The `amount_in` is paid in outcome tokens. The transaction fails if the amount of outcome
        /// tokens received is smaller than `min_amount_out`. The user must correctly specify the
        /// number of outcomes for benchmarking reasons.
        ///
        /// # Parameters
        ///
        /// - `origin`: The origin account making the sale.
        /// - `market_id`: Identifier for the market related to the trade.
        /// - `asset_count`: Number of assets in the pool.
        /// - `asset_in`: Asset to be sold.
        /// - `amount_in`: Amount of outcome tokens paid by the user.
        /// - `min_amount_out`: Minimum amount of collateral the user expects to receive.
        ///
        /// # Complexity
        ///
        /// Depends on the implementation of `CompleteSetOperationsApi` and `ExternalFees`; when
        /// using the canonical implementations, the runtime complexity is `O(asset_count)`.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::sell())]
        #[transactional]
        pub fn sell(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            asset_count: AssetIndexType,
            asset_in: AssetOf<T>,
            #[pallet::compact] amount_in: BalanceOf<T>,
            #[pallet::compact] min_amount_out: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let asset_count_real = T::MarketCommons::market(&market_id)?.outcomes();
            ensure!(asset_count == asset_count_real, Error::<T>::IncorrectAssetCount);
            Self::do_sell(who, market_id, asset_in, amount_in, min_amount_out)?;
            Ok(Some(T::WeightInfo::sell()).into())
        }

        /// Join the liquidity pool for the specified market.
        ///
        /// The LP receives pool shares in exchange for staking outcome tokens into the pool. The
        /// `max_amounts_in` vector specifies the maximum number of each outcome token that the LP is
        /// willing to deposit. These amounts are used to adjust the outcome balances in the pool
        /// according to the new proportion of pool shares owned by the LP.
        ///
        /// Note that the user must acquire the outcome tokens in a separate transaction, either by
        /// buying from the pool or by using complete set operations.
        ///
        /// # Parameters
        ///
        /// - `market_id`: Identifier for the market related to the pool.
        /// - `pool_shares_amount`: The number of new pool shares the LP will receive.
        /// - `max_amounts_in`: Vector of the maximum amounts of each outcome token the LP is
        ///   willing to deposit (with outcomes specified in the order of `MarketCommonsApi`).
        ///
        /// # Complexity
        ///
        /// `O(n)` where `n` is the number of assets in the pool.
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::join())]
        #[transactional]
        pub fn join(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] pool_shares_amount: BalanceOf<T>,
            max_amounts_in: Vec<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let asset_count = T::MarketCommons::market(&market_id)?.outcomes();
            ensure!(max_amounts_in.len() == asset_count as usize, Error::<T>::IncorrectVecLen);
            Self::do_join(who, market_id, pool_shares_amount, max_amounts_in)?;
            Ok(Some(T::WeightInfo::join()).into())
        }

        /// Exit the liquidity pool for the specified market.
        ///
        /// The LP relinquishes pool shares in exchange for withdrawing outcome tokens from the
        /// pool. The `min_amounts_out` vector specifies the minimum number of each outcome token
        /// that the LP expects to withdraw. These minimum amounts are used to adjust the outcome
        /// balances in the pool, taking into account the reduction in the LP's pool share
        /// ownership.
        ///
        /// The transaction will fail unless the LP withdraws their fees from the pool beforehand. A
        /// batch transaction is very useful here.
        ///
        /// If the LP withdraws all pool shares that exist, then the pool is afterwards destroyed. A
        /// new pool can be deployed at any time, provided that the market is still open.
        ///
        /// The LP is not allowed to leave a positive but small amount liquidity in the pool. If the
        /// liquidity parameter drops below a certain threshold, the transaction will fail. The only
        /// solution is to withdraw _all_ liquidity and let the pool die.
        ///
        /// # Parameters
        ///
        /// - `market_id`: Identifier for the market related to the pool.
        /// - `pool_shares_amount_out`: The number of pool shares the LP will relinquish.
        /// - `min_amounts_out`: Vector of the minimum amounts of each outcome token the LP expects
        ///   to withdraw (with outcomes specified in the order given by `MarketCommonsApi`).
        ///
        /// # Complexity
        ///
        /// `O(n)` where `n` is the number of assets in the pool.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::exit())]
        #[transactional]
        pub fn exit(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] pool_shares_amount_out: BalanceOf<T>,
            min_amounts_out: Vec<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let asset_count = T::MarketCommons::market(&market_id)?.outcomes();
            ensure!(min_amounts_out.len() == asset_count as usize, Error::<T>::IncorrectVecLen);
            Self::do_exit(who, market_id, pool_shares_amount_out, min_amounts_out)?;
            Ok(Some(T::WeightInfo::exit()).into())
        }

        /// Withdraw swap fees from the specified market.
        ///
        /// The transaction will fail if the caller is not a liquidity provider. Should always be
        /// used before calling `exit`.
        ///
        /// # Parameters
        ///
        /// - `market_id`: Identifier for the market related to the pool.
        ///
        /// # Complexity
        ///
        /// `O(1)`.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::withdraw_fees())]
        #[transactional]
        pub fn withdraw_fees(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_withdraw_fees(who, market_id)?;
            Ok(())
        }

        /// Deploy a pool for the specified market and provide liquidity.
        ///
        /// The sender specifies a vector of `spot_prices` for the market's outcomes in the order
        /// given by the `MarketCommonsApi`. The transaction will fail if the spot prices don't add
        /// up to exactly `BASE`.
        ///
        /// Depending on the values in the `spot_prices`, the transaction will transfer different
        /// amounts of each outcome to the pool. The sender specifies a maximum `amount` of outcome
        /// tokens to spend.
        ///
        /// Note that the sender must acquire the outcome tokens in a separate transaction by using
        /// complete set operations. It's therefore convenient to batch this function together with
        /// a `buy_complete_set` with `amount` as amount of complete sets to buy.
        ///
        /// Deploying the pool will cost the signer an additional fee to the tune of the
        /// collateral's existential deposit. This fee is placed in the pool account and ensures
        /// that swap fees can be stored in the pool account without triggering dusting or failed
        /// transfers.
        ///
        /// The operation is currently limited to binary and scalar markets.
        ///
        /// # Complexity
        ///
        /// `O(n)` where `n` is the number of outcomes in the specified market.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::deploy_pool())]
        #[transactional]
        pub fn deploy_pool(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
            spot_prices: Vec<BalanceOf<T>>,
            #[pallet::compact] swap_fee: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let asset_count = T::MarketCommons::market(&market_id)?.outcomes() as u32;
            ensure!(spot_prices.len() == asset_count as usize, Error::<T>::IncorrectVecLen);
            Self::do_deploy_pool(who, market_id, amount, spot_prices, swap_fee)?;
            Ok(Some(T::WeightInfo::deploy_pool()).into())
        }
    }

    impl<T: Config> Pallet<T> {
        #[require_transactional]
        fn do_buy(
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            asset_out: AssetOf<T>,
            amount_in: BalanceOf<T>,
            min_amount_out: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(amount_in != Zero::zero(), Error::<T>::ZeroAmount);
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketNotActive);
            Self::try_mutate_pool(&market_id, |pool| {
                ensure!(pool.contains(&asset_out), Error::<T>::AssetNotFound);
                // Defensive check (shouldn't ever happen)!
                ensure!(
                    pool.calculate_spot_price(asset_out)? <= MAX_SPOT_PRICE.saturated_into(),
                    Error::<T>::Unexpected
                );
                ensure!(amount_in <= pool.calculate_max_amount_in(), Error::<T>::NumericalLimits);
                T::MultiCurrency::transfer(pool.collateral, &who, &pool.account_id, amount_in)?;
                let FeeDistribution {
                    remaining: amount_in_minus_fees,
                    swap_fees: swap_fee_amount,
                    external_fees: external_fee_amount,
                } = Self::distribute_fees(market_id, pool, amount_in)?;
                let swap_amount_out =
                    pool.calculate_swap_amount_out_for_buy(asset_out, amount_in_minus_fees)?;
                let amount_out = swap_amount_out.checked_add_res(&amount_in_minus_fees)?;
                ensure!(amount_out >= min_amount_out, Error::<T>::AmountOutBelowMin);
                // Instead of letting `who` buy the complete sets and then transfer almost all of
                // the outcomes to the pool account, we prevent `(n-1)` storage reads by using the
                // pool account to buy. Note that the fees are already in the pool at this point.
                T::CompleteSetOperations::buy_complete_set(
                    pool.account_id.clone(),
                    market_id,
                    amount_in_minus_fees,
                )?;
                T::MultiCurrency::transfer(asset_out, &pool.account_id, &who, amount_out)?;
                for asset in pool.assets().iter() {
                    pool.increase_reserve(asset, &amount_in_minus_fees)?;
                    if *asset == asset_out {
                        pool.decrease_reserve(asset, &amount_out)?;
                    }
                }
                let new_price = pool.calculate_spot_price(asset_out)?;
                ensure!(
                    new_price <= MAX_SPOT_PRICE.saturated_into(),
                    Error::<T>::SpotPriceAboveMax
                );
                Self::deposit_event(Event::<T>::BuyExecuted {
                    who: who.clone(),
                    market_id,
                    asset_out,
                    amount_in,
                    amount_out,
                    swap_fee_amount,
                    external_fee_amount,
                });
                Ok(())
            })
        }

        #[require_transactional]
        fn do_sell(
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            asset_in: AssetOf<T>,
            amount_in: BalanceOf<T>,
            min_amount_out: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(amount_in != Zero::zero(), Error::<T>::ZeroAmount);
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketNotActive);
            Self::try_mutate_pool(&market_id, |pool| {
                ensure!(pool.contains(&asset_in), Error::<T>::AssetNotFound);
                // Defensive check (shouldn't ever happen)!
                ensure!(
                    pool.calculate_spot_price(asset_in)? >= MIN_SPOT_PRICE.saturated_into(),
                    Error::<T>::Unexpected
                );
                ensure!(amount_in <= pool.calculate_max_amount_in(), Error::<T>::NumericalLimits);
                // Instead of first executing a swap with `(n-1)` transfers from the pool account to
                // `who` and then selling complete sets, we prevent `(n-1)` storage reads: 1)
                // Transfer `amount_in` units of `asset_in` to the pool account, 2) sell
                // `amount_out` complete sets using the pool account, 3) transfer
                // `amount_out_minus_fees` units of collateral to `who`. The fees automatically end
                // up in the pool.
                let amount_out = pool.calculate_swap_amount_out_for_sell(asset_in, amount_in)?;
                // Beware! This transfer happen _after_ calculating `amount_out`:
                T::MultiCurrency::transfer(asset_in, &who, &pool.account_id, amount_in)?;
                T::CompleteSetOperations::sell_complete_set(
                    pool.account_id.clone(),
                    market_id,
                    amount_out,
                )?;
                let FeeDistribution {
                    remaining: amount_out_minus_fees,
                    swap_fees: swap_fee_amount,
                    external_fees: external_fee_amount,
                } = Self::distribute_fees(market_id, pool, amount_out)?;
                ensure!(amount_out_minus_fees >= min_amount_out, Error::<T>::AmountOutBelowMin);
                T::MultiCurrency::transfer(
                    pool.collateral,
                    &pool.account_id,
                    &who,
                    amount_out_minus_fees,
                )?;
                for asset in pool.assets().iter() {
                    if *asset == asset_in {
                        pool.increase_reserve(asset, &amount_in)?;
                    }
                    pool.decrease_reserve(asset, &amount_out)?;
                }
                let new_price = pool.calculate_spot_price(asset_in)?;
                ensure!(
                    new_price >= MIN_SPOT_PRICE.saturated_into(),
                    Error::<T>::SpotPriceBelowMin
                );
                Self::deposit_event(Event::<T>::SellExecuted {
                    who: who.clone(),
                    market_id,
                    asset_in,
                    amount_in,
                    amount_out,
                    swap_fee_amount,
                    external_fee_amount,
                });
                Ok(())
            })
        }

        #[require_transactional]
        fn do_join(
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            pool_shares_amount: BalanceOf<T>,
            max_amounts_in: Vec<BalanceOf<T>>,
        ) -> DispatchResult {
            ensure!(pool_shares_amount != Zero::zero(), Error::<T>::ZeroAmount);
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketNotActive);
            Self::try_mutate_pool(&market_id, |pool| {
                let ratio =
                    pool_shares_amount.bdiv_ceil(pool.liquidity_shares_manager.total_shares()?)?;
                let mut amounts_in = vec![];
                for (&asset, &max_amount_in) in pool.assets().iter().zip(max_amounts_in.iter()) {
                    let balance_in_pool = pool.reserve_of(&asset)?;
                    let amount_in = ratio.bmul_ceil(balance_in_pool)?;
                    amounts_in.push(amount_in);
                    ensure!(amount_in <= max_amount_in, Error::<T>::AmountInAboveMax);
                    T::MultiCurrency::transfer(asset, &who, &pool.account_id, amount_in)?;
                }
                for ((_, balance), amount_in) in pool.reserves.iter_mut().zip(amounts_in.iter()) {
                    *balance = balance.checked_add_res(amount_in)?;
                }
                pool.liquidity_shares_manager.join(&who, pool_shares_amount)?;
                let new_liquidity_parameter = pool
                    .liquidity_parameter
                    .checked_add_res(&ratio.bmul(pool.liquidity_parameter)?)?;
                pool.liquidity_parameter = new_liquidity_parameter;
                Self::deposit_event(Event::<T>::JoinExecuted {
                    who: who.clone(),
                    market_id,
                    pool_shares_amount,
                    amounts_in,
                    new_liquidity_parameter,
                });
                Ok(())
            })
        }

        #[require_transactional]
        fn do_exit(
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            pool_shares_amount: BalanceOf<T>,
            min_amounts_out: Vec<BalanceOf<T>>,
        ) -> DispatchResult {
            ensure!(pool_shares_amount != Zero::zero(), Error::<T>::ZeroAmount);
            let _ = T::MarketCommons::market(&market_id)?;
            Pools::<T>::try_mutate_exists(market_id, |maybe_pool| {
                let pool =
                    maybe_pool.as_mut().ok_or::<DispatchError>(Error::<T>::PoolNotFound.into())?;
                ensure!(
                    pool.liquidity_shares_manager.fees == Zero::zero(),
                    Error::<T>::OutstandingFees
                );
                let ratio =
                    pool_shares_amount.bdiv_floor(pool.liquidity_shares_manager.total_shares()?)?;
                let mut amounts_out = vec![];
                for (&asset, &min_amount_out) in pool.assets().iter().zip(min_amounts_out.iter()) {
                    let balance_in_pool = pool.reserve_of(&asset)?;
                    let amount_out = ratio.bmul_floor(balance_in_pool)?;
                    amounts_out.push(amount_out);
                    ensure!(amount_out >= min_amount_out, Error::<T>::AmountOutBelowMin);
                    T::MultiCurrency::transfer(asset, &pool.account_id, &who, amount_out)?;
                }
                for ((_, balance), amount_out) in pool.reserves.iter_mut().zip(amounts_out.iter()) {
                    *balance = balance.checked_sub_res(amount_out)?;
                }
                pool.liquidity_shares_manager.exit(&who, pool_shares_amount)?;
                if pool.liquidity_shares_manager.total_shares()? == Zero::zero() {
                    // FIXME We will withdraw all remaining funds (the "buffer"). This is an ugly
                    // hack and system should offer the option to whitelist accounts.
                    let remaining =
                        T::MultiCurrency::free_balance(pool.collateral, &pool.account_id);
                    T::MultiCurrency::withdraw(pool.collateral, &pool.account_id, remaining)?;
                    *maybe_pool = None; // Delete the storage map entry.
                    Self::deposit_event(Event::<T>::PoolDestroyed {
                        who: who.clone(),
                        market_id,
                        amounts_out,
                    });
                } else {
                    let liq = pool.liquidity_parameter;
                    let new_liquidity_parameter = liq.checked_sub_res(&ratio.bmul(liq)?)?;
                    ensure!(
                        new_liquidity_parameter >= MIN_LIQUIDITY.saturated_into(),
                        Error::<T>::LiquidityTooLow
                    );
                    pool.liquidity_parameter = new_liquidity_parameter;
                    Self::deposit_event(Event::<T>::ExitExecuted {
                        who: who.clone(),
                        market_id,
                        pool_shares_amount,
                        amounts_out,
                        new_liquidity_parameter,
                    });
                }
                Ok(())
            })
        }

        #[require_transactional]
        fn do_withdraw_fees(who: T::AccountId, market_id: MarketIdOf<T>) -> DispatchResult {
            Self::try_mutate_pool(&market_id, |pool| {
                let amount = pool.liquidity_shares_manager.withdraw_fees(&who)?;
                T::MultiCurrency::transfer(pool.collateral, &pool.account_id, &who, amount)?; // Should never fail.
                Self::deposit_event(Event::<T>::FeesWithdrawn {
                    who: who.clone(),
                    market_id,
                    amount,
                });
                Ok(())
            })
        }

        #[require_transactional]
        fn do_deploy_pool(
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            amount: BalanceOf<T>,
            spot_prices: Vec<BalanceOf<T>>,
            swap_fee: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(!Pools::<T>::contains_key(market_id), Error::<T>::DuplicatePool);
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.creator == who, Error::<T>::NotAllowed);
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketNotActive);
            ensure!(market.scoring_rule == ScoringRule::Lmsr, Error::<T>::InvalidTradingMechanism);
            let asset_count = spot_prices.len();
            ensure!(asset_count as u16 == market.outcomes(), Error::<T>::IncorrectVecLen);
            ensure!(market.outcomes() == 2, Error::<T>::MarketNotBinaryOrScalar);
            ensure!(market.outcomes() <= MAX_ASSETS, Error::<T>::AssetCountAboveMax);
            ensure!(swap_fee >= MIN_SWAP_FEE.saturated_into(), Error::<T>::SwapFeeBelowMin);
            ensure!(swap_fee <= T::MaxSwapFee::get(), Error::<T>::SwapFeeAboveMax);
            ensure!(
                spot_prices
                    .iter()
                    .fold(Zero::zero(), |acc: BalanceOf<T>, &val| acc.saturating_add(val))
                    == BASE.saturated_into(),
                Error::<T>::InvalidSpotPrices
            );
            for &p in spot_prices.iter() {
                ensure!(
                    p.saturated_into::<u128>() >= MIN_SPOT_PRICE,
                    Error::<T>::SpotPriceBelowMin
                );
                ensure!(
                    p.saturated_into::<u128>() <= MAX_SPOT_PRICE,
                    Error::<T>::SpotPriceAboveMax
                );
            }
            let (liquidity_parameter, amounts_in) =
                Math::<T>::calculate_reserves_from_spot_prices(amount, spot_prices)?;
            ensure!(
                liquidity_parameter >= MIN_LIQUIDITY.saturated_into(),
                Error::<T>::LiquidityTooLow
            );
            let pool_account_id = Self::pool_account_id(&market_id);
            let assets = Self::outcomes(market_id)?;
            let mut reserves = BTreeMap::new();
            for (&amount_in, &asset) in amounts_in.iter().zip(assets.iter()) {
                T::MultiCurrency::transfer(asset, &who, &pool_account_id, amount_in)?;
                let _ = reserves.insert(asset, amount_in);
            }
            let collateral = market.base_asset;
            let pool = Pool {
                account_id: pool_account_id.clone(),
                reserves: reserves.clone(),
                collateral,
                liquidity_parameter,
                liquidity_shares_manager: SoloLp::new(who.clone(), amount),
                swap_fee,
            };
            // FIXME Ensure that the existential deposit doesn't kill fees. This is an ugly hack and
            // system should offer the option to whitelist accounts.
            T::MultiCurrency::transfer(
                pool.collateral,
                &who,
                &pool.account_id,
                T::MultiCurrency::minimum_balance(collateral),
            )?;
            Pools::<T>::insert(market_id, pool);
            Self::deposit_event(Event::<T>::PoolDeployed {
                who,
                market_id,
                account_id: pool_account_id,
                reserves,
                collateral,
                liquidity_parameter,
                pool_shares_amount: amount,
                swap_fee,
            });
            Ok(())
        }

        #[inline]
        pub(crate) fn pool_account_id(market_id: &MarketIdOf<T>) -> T::AccountId {
            T::PalletId::get().into_sub_account_truncating((*market_id).saturated_into::<u128>())
        }

        /// Distribute swap fees and external fees and returns the remaining amount.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The ID of the market to which the pool belongs.
        /// - `pool`: The pool on which the trade was executed.
        /// - `amount`: The gross amount from which the fee is deduced.
        ///
        /// Will fail if the total amount of fees is more than the gross amount. In particular, the
        /// function will fail if the external fees exceed the gross amount.
        #[require_transactional]
        fn distribute_fees(
            market_id: MarketIdOf<T>,
            pool: &mut PoolOf<T>,
            amount: BalanceOf<T>,
        ) -> Result<FeeDistribution<T>, DispatchError> {
            let swap_fees = pool.swap_fee.bmul(amount)?;
            pool.liquidity_shares_manager.deposit_fees(swap_fees)?; // Should only error unexpectedly!
            let external_fees =
                T::ExternalFees::distribute(market_id, pool.collateral, &pool.account_id, amount);
            let total_fees = external_fees.saturating_add(swap_fees);
            let remaining = amount.checked_sub(&total_fees).ok_or(Error::<T>::Unexpected)?;
            Ok(FeeDistribution { remaining, swap_fees, external_fees })
        }

        // FIXME Carbon copy of a function in prediction-markets. To be removed later.
        fn outcomes(market_id: MarketIdOf<T>) -> Result<Vec<AssetOf<T>>, DispatchError> {
            let market = T::MarketCommons::market(&market_id)?;
            Ok(match market.market_type {
                MarketType::Categorical(categories) => {
                    let mut assets = Vec::new();
                    for i in 0..categories {
                        assets.push(Asset::CategoricalOutcome(market_id, i));
                    }
                    assets
                }
                MarketType::Scalar(_) => {
                    vec![
                        Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                        Asset::ScalarOutcome(market_id, ScalarPosition::Short),
                    ]
                }
            })
        }

        fn try_mutate_pool<F>(market_id: &MarketIdOf<T>, mutator: F) -> DispatchResult
        where
            F: FnMut(&mut PoolOf<T>) -> DispatchResult,
        {
            Pools::<T>::try_mutate(market_id, |maybe_pool| {
                maybe_pool.as_mut().ok_or(Error::<T>::PoolNotFound.into()).and_then(mutator)
            })
        }
    }

    impl<T: Config> DeployPoolApi for Pallet<T> {
        type AccountId = T::AccountId;
        type Balance = BalanceOf<T>;
        type MarketId = MarketIdOf<T>;

        fn deploy_pool(
            who: Self::AccountId,
            market_id: Self::MarketId,
            amount: Self::Balance,
            spot_prices: Vec<Self::Balance>,
            swap_fee: Self::Balance,
        ) -> DispatchResult {
            Self::do_deploy_pool(who, market_id, amount, spot_prices, swap_fee)
        }
    }
}
