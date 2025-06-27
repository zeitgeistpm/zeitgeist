// Copyright 2023-2025 Forecasting Technologies LTD.
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
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

extern crate alloc;

mod benchmarking;
mod consts;
mod helpers;
mod liquidity_tree;
mod macros;
mod math;
pub mod migration;
pub mod mock;
mod pool_storage;
mod tests;
pub mod traits;
pub mod types;
mod utility;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{
        consts::LN_NUMERICAL_LIMIT,
        liquidity_tree::types::{BenchmarkInfo, LiquidityTree, LiquidityTreeError},
        math::{traits::MathOps, types::Math},
        traits::{LiquiditySharesManager, PoolOperations, PoolStorage},
        types::{FeeDistribution, MaxAssets, Pool, PoolType},
        utility::LogCeil,
        weights::*,
    };
    use alloc::{
        collections::{BTreeMap, BTreeSet},
        vec,
        vec::Vec,
    };
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        ensure,
        pallet_prelude::{StorageMap, StorageValue, ValueQuery},
        require_transactional,
        traits::{Get, IsType, StorageVersion},
        transactional, PalletError, PalletId, Parameter, Twox64Concat,
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use orml_traits::MultiCurrency;
    use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;
    use sp_runtime::{
        traits::{
            AccountIdConversion, AtLeast32Bit, CheckedSub, MaybeSerializeDeserialize, Member,
            Saturating, Zero,
        },
        DispatchError, DispatchResult, Perbill, RuntimeDebug, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        constants::{BASE, CENT},
        hybrid_router_api_types::{AmmSoftFail, AmmTrade, ApiError},
        math::{
            checked_ops_res::{CheckedAddRes, CheckedMulRes, CheckedSubRes},
            fixed::{BaseProvider, FixedDiv, FixedMul, ZeitgeistBase},
        },
        traits::{
            CombinatorialTokensApi, CombinatorialTokensFuel, CombinatorialTokensUnsafeApi,
            CompleteSetOperationsApi, DeployPoolApi, DistributeFees, HybridRouterAmmApi,
        },
        types::{Asset, MarketStatus, ScoringRule},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion::new(3);

    // These should not be config parameters to avoid misconfigurations.
    pub(crate) const EXIT_FEE: u128 = CENT / 10;
    /// The minimum allowed swap fee. Hardcoded to avoid misconfigurations which may lead to
    /// exploits.
    pub const MIN_SWAP_FEE: u128 = BASE / 1_000; // 0.1%.
    /// The maximum allowed spot price when creating a pool.
    pub const MAX_SPOT_PRICE: u128 = BASE - CENT / 2;
    /// The minimum allowed spot price when creating a pool.
    pub const MIN_SPOT_PRICE: u128 = CENT / 2;
    /// The maximum value the spot price is allowed to take in a combinatorial market.
    pub const COMBO_MAX_SPOT_PRICE: u128 = BASE - CENT / 10;
    /// The minimum value the spot price is allowed to take in a combinatorial market.
    pub const COMBO_MIN_SPOT_PRICE: u128 = CENT / 10;
    /// The minimum vallowed value of a pool's liquidity parameter.
    pub(crate) const MIN_LIQUIDITY: u128 = BASE;
    /// The minimum percentage each new LP position must increase the liquidity by, represented as
    /// fractional (0.0139098411 represents 1.39098411%).
    pub(crate) const MIN_RELATIVE_LP_POSITION_VALUE: u128 = 139098411; // 1.39098411%

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub type AssetOf<T> = Asset<MarketIdOf<T>>;
    pub type BalanceOf<T> =
        <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
    pub type MarketIdOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub type FuelOf<T> = <<T as Config>::CombinatorialTokens as CombinatorialTokensApi>::Fuel;
    pub(crate) type AssetIndexType = u16;
    pub(crate) type LiquidityTreeOf<T> = LiquidityTree<T, <T as Config>::MaxLiquidityTreeDepth>;
    pub(crate) type PoolOf<T> = Pool<T, LiquidityTreeOf<T>, MaxAssets>;
    pub(crate) type AmmTradeOf<T> = AmmTrade<BalanceOf<T>>;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type of combinatorial ID used by the combinatorial tokens APIs.
        type CombinatorialId: Clone;

        /// API used for calculating splits of tokens when creating combinatorial pools.
        type CombinatorialTokens: CombinatorialTokensApi<
            AccountId = Self::AccountId,
            Balance = BalanceOf<Self>,
            CombinatorialId = Self::CombinatorialId,
            MarketId = MarketIdOf<Self>,
        >;

        /// API for fast creation of tokens when buying or selling combinatorial tokens.
        type CombinatorialTokensUnsafe: CombinatorialTokensUnsafeApi<
            AccountId = Self::AccountId,
            Balance = BalanceOf<Self>,
            MarketId = MarketIdOf<Self>,
        >;

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

        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = BlockNumberFor<Self>,
            Balance = BalanceOf<Self>,
            MarketId = Self::PoolId,
        >;

        type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = AssetOf<Self>>;

        type PoolId: AtLeast32Bit
            + Copy
            + Default
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + Member
            + Parameter;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type WeightInfo: WeightInfoZeitgeist;

        /// The maximum allowed liquidity tree depth per pool. Each pool can support
        /// `2^(depth + 1) - 1` liquidity providers. **Must** be less than 16.
        #[pallet::constant]
        type MaxLiquidityTreeDepth: Get<u32>;

        /// The maximum number of splits allowed when creating a combinatorial pool.
        #[pallet::constant]
        type MaxSplits: Get<u16>;

        #[pallet::constant]
        type MaxSwapFee: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    pub(crate) type Pools<T: Config> = StorageMap<_, Twox64Concat, T::PoolId, PoolOf<T>>;

    #[pallet::storage]
    pub(crate) type PoolCount<T: Config> = StorageValue<_, T::PoolId, ValueQuery>;

    #[pallet::storage]
    pub(crate) type MarketIdToPoolId<T: Config> =
        StorageMap<_, Twox64Concat, MarketIdOf<T>, T::PoolId>;

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
            pool_id: T::PoolId,
            asset_out: AssetOf<T>,
            amount_in: BalanceOf<T>,
            amount_out: BalanceOf<T>,
            swap_fee_amount: BalanceOf<T>,
            external_fee_amount: BalanceOf<T>,
        },
        /// Informant sold a position. `amount_out` is the amount of collateral received by `who`,
        /// with swap and external fees already deducted.
        SellExecuted {
            who: T::AccountId,
            pool_id: T::PoolId,
            asset_in: AssetOf<T>,
            amount_in: BalanceOf<T>,
            amount_out: BalanceOf<T>,
            swap_fee_amount: BalanceOf<T>,
            external_fee_amount: BalanceOf<T>,
        },
        /// Liquidity provider withdrew fees.
        FeesWithdrawn { who: T::AccountId, pool_id: T::PoolId, amount: BalanceOf<T> },
        /// Liquidity provider joined the pool.
        JoinExecuted {
            who: T::AccountId,
            pool_id: T::PoolId,
            pool_shares_amount: BalanceOf<T>,
            amounts_in: Vec<BalanceOf<T>>,
            new_liquidity_parameter: BalanceOf<T>,
        },
        /// Liquidity provider left the pool.
        ExitExecuted {
            who: T::AccountId,
            pool_id: T::PoolId,
            pool_shares_amount: BalanceOf<T>,
            amounts_out: Vec<BalanceOf<T>>,
            new_liquidity_parameter: BalanceOf<T>,
        },
        /// Pool was createed.
        PoolDeployed {
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            pool_id: T::PoolId,
            account_id: T::AccountId,
            reserves: BTreeMap<AssetOf<T>, BalanceOf<T>>,
            collateral: AssetOf<T>,
            liquidity_parameter: BalanceOf<T>,
            pool_shares_amount: BalanceOf<T>,
            swap_fee: BalanceOf<T>,
        },
        /// Pool was destroyed.
        PoolDestroyed { who: T::AccountId, pool_id: T::PoolId, amounts_out: Vec<BalanceOf<T>> },
        /// A combinatorial position was opened.
        ComboBuyExecuted {
            who: AccountIdOf<T>,
            pool_id: T::PoolId,
            buy: Vec<AssetOf<T>>,
            sell: Vec<AssetOf<T>>,
            amount_in: BalanceOf<T>,
            amount_out: BalanceOf<T>,
            swap_fee_amount: BalanceOf<T>,
            external_fee_amount: BalanceOf<T>,
        },
        /// A combinatorial position was closed.
        ComboSellExecuted {
            who: AccountIdOf<T>,
            pool_id: T::PoolId,
            buy: Vec<AssetOf<T>>,
            keep: Vec<AssetOf<T>>,
            sell: Vec<AssetOf<T>>,
            amount_buy: BalanceOf<T>,
            amount_keep: BalanceOf<T>,
            amount_out: BalanceOf<T>,
            swap_fee_amount: BalanceOf<T>,
            external_fee_amount: BalanceOf<T>,
        },
        /// Pool was createed.
        CombinatorialPoolDeployed {
            who: T::AccountId,
            market_ids: Vec<MarketIdOf<T>>,
            pool_id: T::PoolId,
            account_id: T::AccountId,
            reserves: BTreeMap<AssetOf<T>, BalanceOf<T>>,
            collateral: AssetOf<T>,
            liquidity_parameter: BalanceOf<T>,
            pool_shares_amount: BalanceOf<T>,
            swap_fee: BalanceOf<T>,
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
        /// Some calculation failed. This shouldn't happen.
        MathError,
        /// The user is not allowed to execute this command.
        NotAllowed,
        /// This feature is not yet implemented.
        NotImplemented,
        /// Some value in the operation is too large or small.
        NumericalLimits(NumericalLimitsError),
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
        /// An error occurred when handling the liquidty tree.
        LiquidityTreeError(LiquidityTreeError),
        /// The relative value of a new LP position is too low.
        MinRelativeLiquidityThresholdViolated,
        /// Narrowing type conversion occurred.
        NarrowingConversion,

        /// The buy/sell/keep partition specified is empty, or contains overlaps or assets that don't
        /// belong to the market.
        InvalidPartition,

        /// The `amount_keep` parameter must be zero if `keep` is empty and less than `amount_buy`
        /// if `keep` is not empty.
        InvalidAmountKeep,

        /// The number of market IDs specified must be greater than two and no more than the
        /// maximum.
        InvalidMarketCount,

        /// Creating a combinatorial pool for these markets will require more splits than allowed.
        MaxSplitsExceeded,

        /// The specified markets do not all use the same collateral.
        CollateralMismatch,

        /// This function is not allowed to be called for this type of pool.
        InvalidPoolType,
    }

    #[derive(Decode, Encode, Eq, PartialEq, PalletError, RuntimeDebug, TypeInfo)]
    pub enum NumericalLimitsError {
        /// Selling is not allowed at prices this low.
        SpotPriceTooLow,
        /// Interactions which move the price below a particular threshold are not allowed.
        SpotPriceSlippedTooLow,
        /// The maximum buy or sell amount was exceeded.
        MaxAmountExceeded,
        /// The minimum buy or sell amount was exceeded.
        MinAmountNotMet,
        /// Interactions which move the price above a particular threshold are not allowed.
        SpotPriceSlippedTooHigh,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Buy outcome tokens from the specified market.
        ///
        /// The `amount_in` is paid in collateral. The transaction fails if the amount of outcome
        /// tokens received is smaller than `min_amount_out`. The user must correctly specify the
        /// number of outcomes for benchmarking reasons.
        ///
        /// The `amount_in` parameter must also satisfy lower and upper limits due to numerical
        /// constraints. In fact, after `amount_in` has been adjusted for fees, the following must
        /// hold:
        ///
        /// - `amount_in_minus_fees <= EXP_NUMERICAL_LIMIT * pool.liquidity_parameter`.
        /// - `exp(amount_in_minus_fees/pool.liquidity_parameter) - 1 + p <= LN_NUMERICAL_LIMIT`,
        ///   where `p` is the spot price of `asset_out`.
        ///
        /// # Parameters
        ///
        /// - `origin`: The origin account making the purchase.
        /// - `pool_id`: Identifier for the pool used to trade on.
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
        #[pallet::weight(T::WeightInfo::buy((*asset_count).into()))]
        #[transactional]
        pub fn buy(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: T::PoolId,
            asset_count: AssetIndexType,
            asset_out: AssetOf<T>,
            #[pallet::compact] amount_in: BalanceOf<T>,
            #[pallet::compact] min_amount_out: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let pool = <Self as PoolStorage>::get(pool_id)?;
            let asset_count_real = pool.assets().len();
            let asset_count_real_u16: u16 =
                asset_count_real.try_into().map_err(|_| Error::<T>::NarrowingConversion)?;
            ensure!(asset_count == asset_count_real_u16, Error::<T>::IncorrectAssetCount);

            let _ = Self::do_buy(who, pool_id, asset_out, amount_in, min_amount_out)?;

            Ok(Some(T::WeightInfo::buy(asset_count.into())).into())
        }

        /// Sell outcome tokens to the specified market.
        ///
        /// The `amount_in` is paid in outcome tokens. The transaction fails if the amount of outcome
        /// tokens received is smaller than `min_amount_out`. The user must correctly specify the
        /// number of outcomes for benchmarking reasons.
        ///
        /// The `amount_in` parameter must also satisfy lower and upper limits due to numerical
        /// constraints. In fact, the following must hold:
        ///
        /// - `amount_in <= EXP_NUMERICAL_LIMIT * pool.liquidity_parameter`.
        /// - The spot price of `asset_in` is greater than `exp(-EXP_NUMERICAL_LIMIT)` before and
        ///   after execution
        ///
        /// # Parameters
        ///
        /// - `origin`: The origin account making the sale.
        /// - `pool_id`: Identifier for the pool used to trade on.
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
        #[pallet::weight(T::WeightInfo::sell((*asset_count).into()))]
        #[transactional]
        pub fn sell(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: T::PoolId,
            asset_count: AssetIndexType,
            asset_in: AssetOf<T>,
            #[pallet::compact] amount_in: BalanceOf<T>,
            #[pallet::compact] min_amount_out: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let pool = <Self as PoolStorage>::get(pool_id)?;
            let asset_count_real = pool.assets().len();
            let asset_count_real_u16: u16 =
                asset_count_real.try_into().map_err(|_| Error::<T>::NarrowingConversion)?;
            ensure!(asset_count == asset_count_real_u16, Error::<T>::IncorrectAssetCount);

            let _ = Self::do_sell(who, pool_id, asset_in, amount_in, min_amount_out)?;

            Ok(Some(T::WeightInfo::sell(asset_count_real_u16.into())).into())
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
        /// - `pool_id`: Identifier for the pool to add liquidity to.
        /// - `pool_shares_amount`: The number of new pool shares the LP will receive.
        /// - `max_amounts_in`: Vector of the maximum amounts of each outcome token the LP is
        ///   willing to deposit (with outcomes specified in the order of `MarketCommonsApi`).
        ///
        /// # Complexity
        ///
        /// `O(n + d)` where `n` is the number of assets in the pool and `d` is the depth of the
        /// pool's liquidity tree, or, equivalently, `log_2(m)` where `m` is the number of liquidity
        /// providers in the pool.
        #[pallet::call_index(2)]
        #[pallet::weight(
            T::WeightInfo::join_in_place(max_amounts_in.len().saturated_into())
                .max(T::WeightInfo::join_reassigned(max_amounts_in.len().saturated_into()))
                .max(T::WeightInfo::join_leaf(max_amounts_in.len().saturated_into()))
        )]
        #[transactional]
        pub fn join(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: T::PoolId,
            #[pallet::compact] pool_shares_amount: BalanceOf<T>,
            max_amounts_in: Vec<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // Ensure that the conversion in the weight calculation doesn't saturate.
            let _: u32 =
                max_amounts_in.len().try_into().map_err(|_| Error::<T>::NarrowingConversion)?;

            let pool = <Self as PoolStorage>::get(pool_id)?;
            let asset_count_real = pool.assets().len();
            ensure!(max_amounts_in.len() == asset_count_real, Error::<T>::IncorrectVecLen);

            Self::do_join(who, pool_id, pool_shares_amount, max_amounts_in)
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
        /// new pool can be deployed at any time, provided that the market is still open. If there
        /// are funds left in the pool account (this can happen due to exit fees), the remaining
        /// funds are destroyed.
        ///
        /// The LP is not allowed to leave a positive but small amount liquidity in the pool. If the
        /// liquidity parameter drops below a certain threshold, the transaction will fail. The only
        /// solution is to withdraw _all_ liquidity and let the pool die.
        ///
        /// # Parameters
        ///
        /// - `pool_id`: Identifier for the pool to withdraw liquidity from.
        /// - `pool_shares_amount_out`: The number of pool shares the LP will relinquish.
        /// - `min_amounts_out`: Vector of the minimum amounts of each outcome token the LP expects
        ///   to withdraw (with outcomes specified in the order given by `MarketCommonsApi`).
        ///
        /// # Complexity
        ///
        /// `O(n + d)` where `n` is the number of assets in the pool and `d` is the depth of the
        /// pool's liquidity tree, or, equivalently, `log_2(m)` where `m` is the number of liquidity
        /// providers in the pool.
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::exit(min_amounts_out.len().saturated_into()))]
        #[transactional]
        pub fn exit(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: T::PoolId,
            #[pallet::compact] pool_shares_amount_out: BalanceOf<T>,
            min_amounts_out: Vec<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let pool = <Self as PoolStorage>::get(pool_id)?;
            let asset_count_real = pool.assets().len();
            let min_amounts_out_len = min_amounts_out.len();
            ensure!(min_amounts_out_len == asset_count_real, Error::<T>::IncorrectVecLen);

            // Ensure that the conversion in the weight calculation doesn't saturate.
            let min_amounts_out_len_u32: u32 =
                min_amounts_out_len.try_into().map_err(|_| Error::<T>::NarrowingConversion)?;

            Self::do_exit(who, pool_id, pool_shares_amount_out, min_amounts_out)?;

            Ok(Some(T::WeightInfo::exit(min_amounts_out_len_u32)).into())
        }

        /// Withdraw swap fees from the specified market.
        ///
        /// The transaction will fail if the caller is not a liquidity provider. Should always be
        /// used before calling `exit`.
        ///
        /// # Parameters
        ///
        /// - `pool_id`: Identifier for the market related to the pool.
        ///
        /// # Complexity
        ///
        /// `O(1)`.
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::withdraw_fees())]
        #[transactional]
        pub fn withdraw_fees(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: T::PoolId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::do_withdraw_fees(who, pool_id)?;

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
        /// `O(n)` where `n` is the number of assets in the pool.
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::deploy_pool(spot_prices.len().saturated_into()))]
        #[transactional]
        pub fn deploy_pool(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
            spot_prices: Vec<BalanceOf<T>>,
            #[pallet::compact] swap_fee: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let asset_count = T::MarketCommons::market(&market_id)?.outcomes();
            let asset_count_u32: u32 = asset_count.into();
            let spot_prices_len: u32 =
                spot_prices.len().try_into().map_err(|_| Error::<T>::NarrowingConversion)?;
            ensure!(spot_prices_len == asset_count_u32, Error::<T>::IncorrectVecLen);

            Self::do_deploy_pool(who, market_id, amount, spot_prices, swap_fee)?;

            Ok(Some(T::WeightInfo::deploy_pool(spot_prices_len)).into())
        }

        /// Make a combinatorial bet on the specified pool.
        ///
        /// The `amount_in` is paid in collateral. The transaction fails if the amount of outcome
        /// tokens received is smaller than `min_amount_out`. The user must correctly specify the
        /// number of outcomes for benchmarking reasons.
        ///
        /// The user's collateral is used to mint complete sets of the combinatorial tokens in the
        /// pool. The parameters `buy` and `sell` are used to specify which of these tokens the user
        /// wants and doesn't want: The assets in `sell` are sold to buy more of `buy` from the
        /// pool. The assets not contained in either of these will remain in the users wallet
        /// unchanged.
        ///
        /// The function will error if certain numerical constraints are violated.
        ///
        /// # Parameters
        ///
        /// - `origin`: The origin account making the purchase.
        /// - `pool_id`: Identifier for the pool used to trade on.
        /// - `asset_count`: Number of assets in the pool.
        /// - `buy`: The assets that the user want to have more of. Must not be empty.
        /// - `sell`: The assets that the user doesn't want any of. Must not be empty.
        /// - `amount_in`: Amount of collateral paid by the user.
        /// - `min_amount_out`: Minimum number of outcome tokens the user expects to receive.
        ///
        /// # Complexity
        ///
        /// Depends on the implementation of `CombinatorialTokensUnsafeApi` and `ExternalFees`; when
        /// using the canonical implementations, the runtime complexity is `O(asset_count)`.
        #[allow(clippy::too_many_arguments)]
        #[pallet::call_index(6)]
        #[pallet::weight(T::WeightInfo::combo_buy(asset_count.log_ceil().into()))]
        #[transactional]
        pub fn combo_buy(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: T::PoolId,
            asset_count: AssetIndexType,
            buy: Vec<AssetOf<T>>,
            sell: Vec<AssetOf<T>>,
            #[pallet::compact] amount_in: BalanceOf<T>,
            #[pallet::compact] min_amount_out: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let pool = <Self as PoolStorage>::get(pool_id)?;
            let asset_count_real = pool.assets().len();
            let asset_count_real_u16: u16 =
                asset_count_real.try_into().map_err(|_| Error::<T>::NarrowingConversion)?;
            ensure!(asset_count == asset_count_real_u16, Error::<T>::IncorrectAssetCount);

            Self::do_combo_buy(who, pool_id, buy, sell, amount_in, min_amount_out)
        }

        /// Cancel a combinatorial bet on the specified pool.
        ///
        /// The `buy`, `keep` and `sell` parameters are used to specify the amounts of the bet the
        /// user wishes to cancel. The user must hold `amount_buy` units of each asset in `buy` and
        /// `amount_keep` of each asset in `keep` in their wallet. If `keep` is empty, then
        /// `amount_keep` must be zero.
        ///
        /// The transaction fails if the amount of outcome tokens received is smaller than
        /// `min_amount_out`. The user must correctly specify the number of outcomes for
        /// benchmarking reasons.
        ///
        /// The function will error if certain numerical constraints are violated.
        ///
        /// # Parameters
        ///
        /// - `origin`: The origin account making the purchase.
        /// - `pool_id`: Identifier for the pool used to trade on.
        /// - `asset_count`: Number of assets in the pool.
        /// - `buy`: The `buy` of the bet that the user wishes to cancel. Must not be empty.
        /// - `keep`: The tokens not contained in `buy` or `sell` of the bet that the user wishes to
        ///   cancel. May be empty.
        /// - `sell`: The `sell` of the bet that the user wishes to cancel. Must not be empty.
        /// - `amount_buy`: Amount of tokens in `buy` the user wishes to let go.
        /// - `amount_keep`: Amount of tokens in `keep` the user wishes to let go.
        /// - `min_amount_out`: Minimum number of outcome tokens the user expects to receive.
        ///
        /// # Complexity
        ///
        /// Depends on the implementation of `CombinatorialTokensUnsafeApi` and `ExternalFees`; when
        /// using the canonical implementations, the runtime complexity is `O(asset_count)`.
        #[allow(clippy::too_many_arguments)]
        #[pallet::call_index(7)]
        #[pallet::weight(T::WeightInfo::combo_sell(asset_count.log_ceil().into()))]
        #[transactional]
        pub fn combo_sell(
            origin: OriginFor<T>,
            #[pallet::compact] pool_id: T::PoolId,
            asset_count: AssetIndexType,
            buy: Vec<AssetOf<T>>,
            keep: Vec<AssetOf<T>>,
            sell: Vec<AssetOf<T>>,
            #[pallet::compact] amount_buy: BalanceOf<T>,
            #[pallet::compact] amount_keep: BalanceOf<T>,
            #[pallet::compact] min_amount_out: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let pool = <Self as PoolStorage>::get(pool_id)?;
            let asset_count_real = pool.assets().len();
            let asset_count_real_u16: u16 =
                asset_count_real.try_into().map_err(|_| Error::<T>::NarrowingConversion)?;
            ensure!(asset_count == asset_count_real_u16, Error::<T>::IncorrectAssetCount);

            Self::do_combo_sell(
                who,
                pool_id,
                buy,
                keep,
                sell,
                amount_buy,
                amount_keep,
                min_amount_out,
            )
        }

        /// Deploy a combinatorial pool for the specified markets and provide liquidity.
        ///
        /// The tokens of each of the markets specified by `market_ids` are split into atoms. For
        /// each combination of outcome tokens `x, ..., z` from the markets, there is one
        /// combinatorial token `x & ... & z` in the pool.
        ///
        /// The pool's assets are ordered by lexicographical order, using the ordering of tokens of
        /// each individual market provided by the `MarketCommonsApi`. For example, if three markets
        /// with outcomes `x_1, x_2`, `y_1, y_2` and `z_1, z_2` are involved, the outcomes of the
        /// pool are (in order):
        ///
        /// x_1 & y_1 & z_1
        /// x_1 & y_1 & z_2
        /// x_1 & y_2 & z_1
        /// x_1 & y_2 & z_2
        /// x_2 & y_1 & z_1
        /// x_2 & y_1 & z_2
        /// x_2 & y_2 & z_1
        /// x_2 & y_2 & z_2
        ///
        /// The sender specifies a vector of `spot_prices` for the assets of the new pool, in the
        /// order as described above.
        ///
        /// Depending on the values in the `spot_prices`, the transaction will transfer different
        /// amounts of each outcome to the pool. The sender specifies a maximum `amount` of outcome
        /// tokens to spend.
        ///
        /// Unlike in the `deploy_pool` extrinsic, the sender need not acquire the outcome tokens
        /// themselves. Instead, all they need is `amount` units of collateral.
        ///
        /// Deploying the pool will cost the signer an additional fee to the tune of the
        /// collateral's existential deposit. This fee is placed in the pool account and ensures
        /// that swap fees can be stored in the pool account without triggering dusting or failed
        /// transfers.
        ///
        /// The `fuel` parameter specifies how much work the cryptographic id manager will do
        /// and can be used for benchmarking purposes.
        ///
        /// # Complexity
        ///
        /// `O(n)` where `n` is the number of splits required to create the pool.
        /// The `fuel` parameter specifies how much work the cryptographic id manager will do
        /// and can be used for benchmarking purposes.
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::deploy_combinatorial_pool(
            asset_count.log_ceil().into(),
            fuel.total(),
        ))]
        #[transactional]
        pub fn deploy_combinatorial_pool(
            origin: OriginFor<T>,
            asset_count: AssetIndexType,
            market_ids: Vec<MarketIdOf<T>>,
            amount: BalanceOf<T>,
            spot_prices: Vec<BalanceOf<T>>,
            swap_fee: BalanceOf<T>,
            fuel: FuelOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut real_asset_count = 1u16;
            for market_id in market_ids.iter() {
                let market = T::MarketCommons::market(market_id)?;
                real_asset_count = real_asset_count.saturating_mul(market.outcomes());
            }
            ensure!(asset_count == real_asset_count, Error::<T>::IncorrectAssetCount);

            Self::do_deploy_combinatorial_pool(who, market_ids, amount, spot_prices, swap_fee, fuel)
        }
    }

    impl<T: Config> Pallet<T> {
        #[require_transactional]
        pub(crate) fn do_buy(
            who: T::AccountId,
            pool_id: T::PoolId,
            asset_out: AssetOf<T>,
            amount_in: BalanceOf<T>,
            min_amount_out: BalanceOf<T>,
        ) -> Result<AmmTradeOf<T>, DispatchError> {
            ensure!(amount_in != Zero::zero(), Error::<T>::ZeroAmount);

            <Self as PoolStorage>::try_mutate_pool(&pool_id, |pool| {
                ensure!(pool.is_active()?, Error::<T>::MarketNotActive);
                ensure!(
                    matches!(pool.pool_type, PoolType::Standard(_)),
                    Error::<T>::InvalidPoolType
                );
                ensure!(pool.contains(&asset_out), Error::<T>::AssetNotFound);
                T::MultiCurrency::transfer(pool.collateral, &who, &pool.account_id, amount_in)?;
                let FeeDistribution {
                    remaining: amount_in_minus_fees,
                    swap_fees: swap_fee_amount,
                    external_fees: external_fee_amount,
                } = Self::distribute_fees(pool, &pool.account_id.clone(), amount_in)?;
                ensure!(
                    amount_in_minus_fees <= pool.calculate_numerical_threshold(),
                    Error::<T>::NumericalLimits(NumericalLimitsError::MaxAmountExceeded),
                );
                ensure!(
                    pool.calculate_buy_ln_argument(asset_out, amount_in_minus_fees)?
                        >= LN_NUMERICAL_LIMIT.saturated_into(),
                    Error::<T>::NumericalLimits(NumericalLimitsError::MinAmountNotMet),
                );
                let buy = vec![asset_out];
                let sell = pool.assets_complement(&buy);
                // `swap_amount_out` is the amount of assets in sell (S) that are sold for more
                // assets of buy (B). In the reference documentation it's called `y(x)`
                let swap_amount_out =
                    pool.calculate_swap_amount_out_for_buy(buy, sell, amount_in_minus_fees)?;
                // The following is the buy complete set amount plus the additional amount
                // that was received through the sale of the unwanted outcomes in the sell.
                let amount_out = swap_amount_out.checked_add_res(&amount_in_minus_fees)?;
                ensure!(amount_out >= min_amount_out, Error::<T>::AmountOutBelowMin);
                // Instead of letting `who` buy the complete sets and then transfer almost all of
                // the outcomes to the pool account, we prevent `(n-1)` storage reads by using the
                // pool account to buy. Note that the fees are already in the pool at this point.
                let PoolType::Standard(market_id) = pool.pool_type else {
                    return Err(Error::<T>::Unexpected.into());
                };
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
                Self::deposit_event(Event::<T>::BuyExecuted {
                    who: who.clone(),
                    pool_id,
                    asset_out,
                    amount_in,
                    amount_out,
                    swap_fee_amount,
                    external_fee_amount,
                });
                Ok(AmmTrade { amount_in, amount_out, swap_fee_amount, external_fee_amount })
            })
        }

        #[require_transactional]
        pub(crate) fn do_sell(
            who: T::AccountId,
            pool_id: T::PoolId,
            asset_in: AssetOf<T>,
            amount_in: BalanceOf<T>,
            min_amount_out: BalanceOf<T>,
        ) -> Result<AmmTradeOf<T>, DispatchError> {
            ensure!(amount_in != Zero::zero(), Error::<T>::ZeroAmount);

            <Self as PoolStorage>::try_mutate_pool(&pool_id, |pool| {
                ensure!(pool.is_active()?, Error::<T>::MarketNotActive);
                ensure!(
                    matches!(pool.pool_type, PoolType::Standard(_)),
                    Error::<T>::InvalidPoolType
                );
                ensure!(pool.contains(&asset_in), Error::<T>::AssetNotFound);
                // Ensure that the price of `asset_in` is at least `exp(-EXP_NUMERICAL_LIMITS) =
                // 4.5399...e-05`.
                ensure!(
                    pool.reserve_of(&asset_in)? <= pool.calculate_numerical_threshold(),
                    Error::<T>::NumericalLimits(NumericalLimitsError::SpotPriceTooLow),
                );
                ensure!(
                    amount_in <= pool.calculate_numerical_threshold(),
                    Error::<T>::NumericalLimits(NumericalLimitsError::MaxAmountExceeded),
                );

                // `asset_in` is sold in order to get the amount of full sets of all possible
                // outcomes, the `amount_out` is calculated in which all other assets are sold to
                // get an equal amount of each possible asset back,
                // which can then be burned for collateral
                let buy = vec![asset_in];
                let keep = vec![];
                let sell = pool.assets_complement(&buy);
                let amount_out = pool.calculate_swap_amount_out_for_sell(
                    buy,
                    keep,
                    sell,
                    amount_in,
                    Zero::zero(),
                )?;

                // Instead of first executing a swap with `(n-1)` transfers from the pool account to
                // `who` and then selling complete sets, we prevent `(n-1)` storage reads: 1)
                // Transfer `amount_in` units of `asset_in` to the pool account, 2) sell
                // `amount_out` complete sets using the pool account, 3) transfer
                // `amount_out_minus_fees` units of collateral to `who`. The fees automatically end
                // up in the pool.
                T::MultiCurrency::transfer(asset_in, &who, &pool.account_id, amount_in)?;
                let PoolType::Standard(market_id) = pool.pool_type else {
                    return Err(Error::<T>::Unexpected.into());
                };
                T::CompleteSetOperations::sell_complete_set(
                    pool.account_id.clone(),
                    market_id,
                    amount_out,
                )?;
                let FeeDistribution {
                    remaining: amount_out_minus_fees,
                    swap_fees: swap_fee_amount,
                    external_fees: external_fee_amount,
                } = Self::distribute_fees(pool, &pool.account_id.clone(), amount_out)?;
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
                // Ensure that the sell doesn't move the price below the minimum defined by
                // `EXP_NUMERICAL_LIMITS` (see comment above).
                ensure!(
                    pool.reserve_of(&asset_in)? <= pool.calculate_numerical_threshold(),
                    Error::<T>::NumericalLimits(NumericalLimitsError::SpotPriceSlippedTooLow),
                );
                Self::deposit_event(Event::<T>::SellExecuted {
                    who: who.clone(),
                    pool_id,
                    asset_in,
                    amount_in,
                    amount_out: amount_out_minus_fees,
                    swap_fee_amount,
                    external_fee_amount,
                });
                Ok(AmmTrade {
                    amount_in,
                    amount_out: amount_out_minus_fees,
                    swap_fee_amount,
                    external_fee_amount,
                })
            })
        }

        #[require_transactional]
        pub(crate) fn do_join(
            who: T::AccountId,
            pool_id: T::PoolId,
            pool_shares_amount: BalanceOf<T>,
            max_amounts_in: Vec<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            ensure!(pool_shares_amount != Zero::zero(), Error::<T>::ZeroAmount);

            let weight = <Self as PoolStorage>::try_mutate_pool(&pool_id, |pool| {
                ensure!(pool.is_active()?, Error::<T>::MarketNotActive);
                ensure!(
                    max_amounts_in.len() == pool.assets().len(),
                    Error::<T>::IncorrectAssetCount
                );
                let asset_count_u32 =
                    max_amounts_in.len().try_into().map_err(|_| Error::<T>::NarrowingConversion)?;
                let ratio =
                    pool_shares_amount.bdiv_ceil(pool.liquidity_shares_manager.total_shares()?)?;
                // Ensure that new LPs contribute at least MIN_RELATIVE_LP_POSITION_VALUE. Note that
                // this ensures that the ratio can never be zero.
                if pool.liquidity_shares_manager.shares_of(&who).is_err() {
                    ensure!(
                        ratio >= MIN_RELATIVE_LP_POSITION_VALUE.saturated_into(),
                        Error::<T>::MinRelativeLiquidityThresholdViolated,
                    );
                }
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
                let benchmark_info =
                    pool.liquidity_shares_manager.join(&who, pool_shares_amount)?;
                let new_liquidity_parameter = pool
                    .liquidity_parameter
                    .checked_add_res(&ratio.bmul(pool.liquidity_parameter)?)?;
                pool.liquidity_parameter = new_liquidity_parameter;
                Self::deposit_event(Event::<T>::JoinExecuted {
                    who: who.clone(),
                    pool_id,
                    pool_shares_amount,
                    amounts_in,
                    new_liquidity_parameter,
                });
                let weight = match benchmark_info {
                    BenchmarkInfo::InPlace => T::WeightInfo::join_in_place(asset_count_u32),
                    BenchmarkInfo::Reassigned => T::WeightInfo::join_reassigned(asset_count_u32),
                    BenchmarkInfo::Leaf => T::WeightInfo::join_leaf(asset_count_u32),
                };
                Ok(weight)
            })?;
            Ok((Some(weight)).into())
        }

        #[require_transactional]
        pub(crate) fn do_exit(
            who: T::AccountId,
            pool_id: T::PoolId,
            pool_shares_amount: BalanceOf<T>,
            min_amounts_out: Vec<BalanceOf<T>>,
        ) -> DispatchResult {
            ensure!(pool_shares_amount != Zero::zero(), Error::<T>::ZeroAmount);

            <Self as PoolStorage>::try_mutate_exists(&pool_id, |pool| {
                let ratio = {
                    let mut ratio = pool_shares_amount
                        .bdiv_floor(pool.liquidity_shares_manager.total_shares()?)?;
                    if pool.is_active()? {
                        let multiplier = ZeitgeistBase::<BalanceOf<T>>::get()?
                            .checked_sub_res(&EXIT_FEE.saturated_into())?;
                        ratio = ratio.bmul_floor(multiplier)?;
                    }
                    ratio
                };
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
                    let withdraw_remaining = |&asset| -> DispatchResult {
                        let remaining = T::MultiCurrency::free_balance(asset, &pool.account_id);
                        T::MultiCurrency::withdraw(asset, &pool.account_id, remaining)?;
                        Ok(())
                    };
                    // TODO(#1220): We will withdraw all remaining funds (the "buffer"). This is an
                    // ugly hack and frame_system should offer the option to whitelist accounts.
                    withdraw_remaining(&pool.collateral)?;
                    // Clear left-over tokens. These naturally occur in the form of exit fees.
                    for asset in pool.assets().iter() {
                        withdraw_remaining(asset)?;
                    }
                    Self::deposit_event(Event::<T>::PoolDestroyed {
                        who: who.clone(),
                        pool_id,
                        amounts_out,
                    });

                    // Delete the pool. No need to clear `MarketIdToPoolId`.
                    Ok(((), true))
                } else {
                    let old_liquidity_parameter = pool.liquidity_parameter;
                    let new_liquidity_parameter = old_liquidity_parameter
                        .checked_sub_res(&ratio.bmul(old_liquidity_parameter)?)?;
                    // If `who` still holds pool shares, check that their position has at least
                    // minimum size.
                    if let Ok(remaining_pool_shares_amount) =
                        pool.liquidity_shares_manager.shares_of(&who)
                    {
                        ensure!(
                            new_liquidity_parameter >= MIN_LIQUIDITY.saturated_into(),
                            Error::<T>::LiquidityTooLow
                        );
                        let remaining_pool_shares_ratio = remaining_pool_shares_amount
                            .bdiv_floor(pool.liquidity_shares_manager.total_shares()?)?;
                        ensure!(
                            remaining_pool_shares_ratio
                                >= MIN_RELATIVE_LP_POSITION_VALUE.saturated_into(),
                            Error::<T>::MinRelativeLiquidityThresholdViolated
                        );
                    }
                    pool.liquidity_parameter = new_liquidity_parameter;
                    Self::deposit_event(Event::<T>::ExitExecuted {
                        who: who.clone(),
                        pool_id,
                        pool_shares_amount,
                        amounts_out,
                        new_liquidity_parameter,
                    });

                    Ok(((), false))
                }
            })
        }

        #[require_transactional]
        pub(crate) fn do_withdraw_fees(who: T::AccountId, pool_id: T::PoolId) -> DispatchResult {
            <Self as PoolStorage>::try_mutate_pool(&pool_id, |pool| {
                let amount = pool.liquidity_shares_manager.withdraw_fees(&who)?;
                T::MultiCurrency::transfer(pool.collateral, &pool.account_id, &who, amount)?; // Should never fail.
                Self::deposit_event(Event::<T>::FeesWithdrawn {
                    who: who.clone(),
                    pool_id,
                    amount,
                });
                Ok(())
            })
        }

        #[require_transactional]
        pub(crate) fn do_deploy_pool(
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            amount: BalanceOf<T>,
            spot_prices: Vec<BalanceOf<T>>,
            swap_fee: BalanceOf<T>,
        ) -> DispatchResult {
            // MarketIdToPoolId is not cleared when a pool is destroyed, so checking if
            // `MarketIdToPoolId` holds a key is not enough.
            if let Some(pool_id) = MarketIdToPoolId::<T>::get(market_id) {
                ensure!(!Pools::<T>::contains_key(pool_id), Error::<T>::DuplicatePool);
            }

            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketNotActive);
            ensure!(
                market.scoring_rule == ScoringRule::AmmCdaHybrid,
                Error::<T>::InvalidTradingMechanism
            );
            let asset_count_u16: u16 =
                spot_prices.len().try_into().map_err(|_| Error::<T>::NarrowingConversion)?;
            let asset_count_u32: u32 = asset_count_u16.into();
            ensure!(asset_count_u16 == market.outcomes(), Error::<T>::IncorrectVecLen);
            ensure!(asset_count_u32 <= MaxAssets::get(), Error::<T>::AssetCountAboveMax);
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
            let pool_id = <Self as PoolStorage>::next_pool_id();
            let pool_account_id = Self::pool_account_id(&pool_id);
            let mut reserves = BTreeMap::new();
            for (&amount_in, &asset) in amounts_in.iter().zip(market.outcome_assets().iter()) {
                T::MultiCurrency::transfer(asset, &who, &pool_account_id, amount_in)?;
                let _ = reserves.insert(asset, amount_in);
            }
            let collateral = market.base_asset;
            let pool = Pool {
                account_id: pool_account_id.clone(),
                assets: market.outcome_assets().try_into().map_err(|_| Error::<T>::Unexpected)?,
                reserves: reserves.clone().try_into().map_err(|_| Error::<T>::Unexpected)?,
                collateral,
                liquidity_parameter,
                liquidity_shares_manager: LiquidityTree::new(who.clone(), amount)?,
                swap_fee,
                pool_type: PoolType::Standard(market_id),
            };
            // TODO(#1220): Ensure that the existential deposit doesn't kill fees. This is an ugly
            // hack and system should offer the option to whitelist accounts.
            T::MultiCurrency::transfer(
                pool.collateral,
                &who,
                &pool.account_id,
                T::MultiCurrency::minimum_balance(collateral),
            )?;
            let _ = <Self as PoolStorage>::add(pool)?;
            MarketIdToPoolId::<T>::insert(market_id, pool_id);
            Self::deposit_event(Event::<T>::PoolDeployed {
                who,
                market_id,
                pool_id,
                account_id: pool_account_id,
                reserves,
                collateral,
                liquidity_parameter,
                pool_shares_amount: amount,
                swap_fee,
            });
            Ok(())
        }

        #[require_transactional]
        pub(crate) fn do_deploy_combinatorial_pool(
            who: T::AccountId,
            market_ids: Vec<MarketIdOf<T>>,
            amount: BalanceOf<T>,
            spot_prices: Vec<BalanceOf<T>>,
            swap_fee: BalanceOf<T>,
            fuel: FuelOf<T>,
        ) -> DispatchResult {
            ensure!(swap_fee >= MIN_SWAP_FEE.saturated_into(), Error::<T>::SwapFeeBelowMin);
            ensure!(swap_fee <= T::MaxSwapFee::get(), Error::<T>::SwapFeeAboveMax);

            let (collection_ids, position_ids, collateral) =
                Self::split_markets(who.clone(), market_ids.clone(), amount, fuel)?;

            ensure!(spot_prices.len() == collection_ids.len(), Error::<T>::IncorrectVecLen);
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
            let pool_id = <Self as PoolStorage>::next_pool_id();
            let pool_account_id = Self::pool_account_id(&pool_id);
            let mut reserves = BTreeMap::new();
            for (&amount_in, &asset) in amounts_in.iter().zip(position_ids.iter()) {
                T::MultiCurrency::transfer(asset, &who, &pool_account_id, amount_in)?;
                let _ = reserves.insert(asset, amount_in);
            }
            let pool = Pool {
                account_id: pool_account_id.clone(),
                assets: position_ids.try_into().map_err(|_| Error::<T>::Unexpected)?,
                reserves: reserves.clone().try_into().map_err(|_| Error::<T>::Unexpected)?,
                collateral,
                liquidity_parameter,
                liquidity_shares_manager: LiquidityTree::new(who.clone(), amount)?,
                swap_fee,
                pool_type: PoolType::Combinatorial(
                    market_ids.clone().try_into().map_err(|_| Error::<T>::Unexpected)?,
                ),
            };

            ensure!(pool.is_active()?, Error::<T>::MarketNotActive);

            // TODO(#1220): Ensure that the existential deposit doesn't kill fees. This is an ugly
            // hack and system should offer the option to whitelist accounts.
            T::MultiCurrency::transfer(
                pool.collateral,
                &who,
                &pool.account_id,
                T::MultiCurrency::minimum_balance(collateral),
            )?;
            let _ = <Self as PoolStorage>::add(pool);
            Self::deposit_event(Event::<T>::CombinatorialPoolDeployed {
                who,
                market_ids,
                pool_id,
                account_id: pool_account_id,
                reserves,
                collateral,
                liquidity_parameter,
                pool_shares_amount: amount,
                swap_fee,
            });
            Ok(())
        }

        #[allow(clippy::too_many_arguments)]
        #[require_transactional]
        pub(crate) fn do_combo_buy(
            who: T::AccountId,
            pool_id: T::PoolId,
            buy: Vec<AssetOf<T>>,
            sell: Vec<AssetOf<T>>,
            amount_in: BalanceOf<T>,
            min_amount_out: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(amount_in != Zero::zero(), Error::<T>::ZeroAmount);

            <Self as PoolStorage>::try_mutate_pool(&pool_id, |pool| {
                ensure!(pool.is_active()?, Error::<T>::MarketNotActive);
                ensure!(
                    matches!(pool.pool_type, PoolType::Combinatorial(_)),
                    Error::<T>::InvalidPoolType
                );

                // Ensure that `buy` and `sell` partition are disjoint, only contain assets from
                // the market and don't contain dupliates.
                ensure!(!buy.is_empty(), Error::<T>::InvalidPartition);
                ensure!(!sell.is_empty(), Error::<T>::InvalidPartition);
                for asset in buy.iter() {
                    ensure!(!sell.contains(asset), Error::<T>::InvalidPartition);
                    ensure!(pool.assets().contains(asset), Error::<T>::InvalidPartition);
                }
                for asset in sell.iter() {
                    ensure!(pool.assets().contains(asset), Error::<T>::InvalidPartition);
                }
                let buy_set = buy.iter().collect::<BTreeSet<_>>();
                let sell_set = sell.iter().collect::<BTreeSet<_>>();
                ensure!(buy_set.len() == buy.len(), Error::<T>::InvalidPartition);
                ensure!(sell_set.len() == sell.len(), Error::<T>::InvalidPartition);

                let FeeDistribution {
                    remaining: amount_in_minus_fees,
                    swap_fees: swap_fee_amount,
                    external_fees: external_fee_amount,
                } = Self::distribute_fees(pool, &who, amount_in)?;
                // `swap_amount_out` is the amount of assets in sell (S) that are sold for more
                // assets of buy (B). In the reference documentation it's called `y(x)`
                let swap_amount_out = pool.calculate_swap_amount_out_for_buy(
                    buy.clone(),
                    sell.clone(),
                    amount_in_minus_fees,
                )?;
                // The following is the buy complete set amount plus the additional amount
                // that was received through the sale of the unwanted outcomes in the sell.
                let amount_out = swap_amount_out.checked_add_res(&amount_in_minus_fees)?;
                ensure!(amount_out >= min_amount_out, Error::<T>::AmountOutBelowMin);

                // Using unsafe API to avoid doing work. This is perfectly safe as long as
                // `pool.assets()` returns a "full set" of split tokens.
                T::CombinatorialTokensUnsafe::split_position_unsafe(
                    who.clone(),
                    pool.collateral,
                    pool.assets(),
                    amount_in_minus_fees,
                )?;

                for &asset in buy.iter() {
                    T::MultiCurrency::transfer(asset, &pool.account_id, &who, swap_amount_out)?;
                    pool.decrease_reserve(&asset, &swap_amount_out)?;
                }
                for &asset in sell.iter() {
                    T::MultiCurrency::transfer(
                        asset,
                        &who,
                        &pool.account_id,
                        amount_in_minus_fees,
                    )?;
                    pool.increase_reserve(&asset, &amount_in_minus_fees)?;
                }

                // Ensure that numerical limits of all prices are respected.
                for &asset in pool.assets().iter() {
                    let spot_price = pool.calculate_spot_price(asset)?;
                    ensure!(
                        spot_price >= COMBO_MIN_SPOT_PRICE.saturated_into(),
                        Error::<T>::NumericalLimits(NumericalLimitsError::SpotPriceSlippedTooLow)
                    );
                    ensure!(
                        spot_price <= COMBO_MAX_SPOT_PRICE.saturated_into(),
                        Error::<T>::NumericalLimits(NumericalLimitsError::SpotPriceSlippedTooHigh)
                    );
                }

                Self::deposit_event(Event::<T>::ComboBuyExecuted {
                    who: who.clone(),
                    pool_id,
                    buy: buy.clone(),
                    sell: sell.clone(),
                    amount_in,
                    amount_out,
                    swap_fee_amount,
                    external_fee_amount,
                });

                Ok(())
            })
        }

        #[allow(clippy::too_many_arguments)]
        #[require_transactional]
        pub(crate) fn do_combo_sell(
            who: T::AccountId,
            pool_id: T::PoolId,
            buy: Vec<AssetOf<T>>,
            keep: Vec<AssetOf<T>>,
            sell: Vec<AssetOf<T>>,
            amount_buy: BalanceOf<T>,
            amount_keep: BalanceOf<T>,
            min_amount_out: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(amount_buy != Zero::zero(), Error::<T>::ZeroAmount);

            if keep.is_empty() {
                ensure!(amount_keep.is_zero(), Error::<T>::InvalidAmountKeep);
            } else {
                ensure!(amount_keep < amount_buy, Error::<T>::InvalidAmountKeep);
            }

            <Self as PoolStorage>::try_mutate_pool(&pool_id, |pool| {
                ensure!(pool.is_active()?, Error::<T>::MarketNotActive);
                ensure!(
                    matches!(pool.pool_type, PoolType::Combinatorial(_)),
                    Error::<T>::InvalidPoolType
                );

                // Ensure that `buy` and `sell` partition are disjoint and only contain assets from
                // the market.
                ensure!(!buy.is_empty(), Error::<T>::InvalidPartition);
                ensure!(!sell.is_empty(), Error::<T>::InvalidPartition);
                for asset in buy.iter() {
                    ensure!(!keep.contains(asset), Error::<T>::InvalidPartition);
                    ensure!(!sell.contains(asset), Error::<T>::InvalidPartition);
                    ensure!(pool.assets().contains(asset), Error::<T>::InvalidPartition);
                }
                for asset in sell.iter() {
                    ensure!(!keep.contains(asset), Error::<T>::InvalidPartition);
                    ensure!(pool.assets().contains(asset), Error::<T>::InvalidPartition);
                }
                for asset in keep.iter() {
                    ensure!(pool.assets().contains(asset), Error::<T>::InvalidPartition);
                }
                let buy_set = buy.iter().collect::<BTreeSet<_>>();
                let keep_set = keep.iter().collect::<BTreeSet<_>>();
                let sell_set = sell.iter().collect::<BTreeSet<_>>();
                ensure!(buy_set.len() == buy.len(), Error::<T>::InvalidPartition);
                ensure!(keep_set.len() == keep.len(), Error::<T>::InvalidPartition);
                ensure!(sell_set.len() == sell.len(), Error::<T>::InvalidPartition);
                let total_assets = buy.len().saturating_add(keep.len()).saturating_add(sell.len());
                ensure!(total_assets == pool.assets().len(), Error::<T>::InvalidPartition);

                // This is the amount of collateral the user will receive in the end, or,
                // equivalently, the amount of each asset in `sell` that the user intermittently
                // receives from the pool (before selling complete sets).
                let amount_out = pool.calculate_swap_amount_out_for_sell(
                    buy.clone(),
                    keep.clone(),
                    sell.clone(),
                    amount_buy,
                    amount_keep,
                )?;
                ensure!(amount_out >= min_amount_out, Error::<T>::AmountOutBelowMin);

                // The deal is that the user gives up all of the assets specified in the function
                // parameters and receives `amount_out` (minus fees) units of collateral. To create
                // the collateral, the pool has to call `sell_complete_set`. This approach is more
                // stable than letting the user call `sell_complete_set` after equalizing their
                // assets, as doing so may lead to `sell_complete_set` failing due to rounding
                // errors.

                for &asset in buy.iter() {
                    T::MultiCurrency::transfer(asset, &who, &pool.account_id, amount_buy)?;
                    pool.increase_reserve(&asset, &amount_buy)?;
                }

                for &asset in keep.iter() {
                    T::MultiCurrency::transfer(asset, &who, &pool.account_id, amount_keep)?;
                    pool.increase_reserve(&asset, &amount_keep)?;
                }

                // Using unsafe API to avoid doing work. This is perfectly safe as long as
                // `pool.assets()` returns a "full set" of split tokens.
                T::CombinatorialTokensUnsafe::merge_position_unsafe(
                    pool.account_id.clone(),
                    pool.collateral,
                    pool.assets(),
                    amount_out,
                )?;

                for &asset in pool.assets().iter() {
                    pool.decrease_reserve(&asset, &amount_out)?;
                }

                let FeeDistribution {
                    remaining: amount_out_minus_fees,
                    swap_fees: swap_fee_amount,
                    external_fees: external_fee_amount,
                } = Self::distribute_fees(pool, &pool.account_id.clone(), amount_out)?;

                T::MultiCurrency::transfer(
                    pool.collateral,
                    &pool.account_id,
                    &who,
                    amount_out_minus_fees,
                )?;

                // Ensure that numerical limits of all prices are respected.
                for &asset in pool.assets().iter() {
                    let spot_price = pool.calculate_spot_price(asset)?;
                    ensure!(
                        spot_price >= COMBO_MIN_SPOT_PRICE.saturated_into(),
                        Error::<T>::NumericalLimits(NumericalLimitsError::SpotPriceSlippedTooLow)
                    );
                    ensure!(
                        spot_price <= COMBO_MAX_SPOT_PRICE.saturated_into(),
                        Error::<T>::NumericalLimits(NumericalLimitsError::SpotPriceSlippedTooHigh)
                    );
                }

                Self::deposit_event(Event::<T>::ComboSellExecuted {
                    who: who.clone(),
                    pool_id,
                    buy: buy.clone(),
                    keep: keep.clone(),
                    sell: sell.clone(),
                    amount_buy,
                    amount_keep,
                    amount_out: amount_out_minus_fees,
                    swap_fee_amount,
                    external_fee_amount,
                });

                Ok(())
            })
        }

        #[inline]
        pub(crate) fn pool_account_id(pool_id: &T::PoolId) -> T::AccountId {
            T::PalletId::get().into_sub_account_truncating((*pool_id).saturated_into::<u128>())
        }

        /// Returns the assets contained in the pool given by `pool_id`.
        pub fn assets(pool_id: T::PoolId) -> Result<Vec<AssetOf<T>>, DispatchError> {
            let pool = <Self as PoolStorage>::get(pool_id)?;

            Ok(pool.assets.into_inner())
        }

        /// Distribute swap fees and external fees and returns the remaining amount.
        ///
        /// # Arguments
        ///
        /// - `pool_id`: The ID of the pool on which the trade was executed.
        /// - `pool`: The pool on which the trade was executed.
        /// - `account`: The account that the fee is deducted from.
        /// - `amount`: The gross amount from which the fee is deduced.
        ///
        /// Will fail if the total amount of fees is more than the gross amount. In particular, the
        /// function will fail if the external fees exceed the gross amount.
        #[require_transactional]
        fn distribute_fees(
            pool: &mut PoolOf<T>,
            account: &AccountIdOf<T>,
            amount: BalanceOf<T>,
        ) -> Result<FeeDistribution<T>, DispatchError> {
            let swap_fees = pool.swap_fee.bmul(amount)?;
            T::MultiCurrency::transfer(pool.collateral, account, &pool.account_id, swap_fees)?;
            pool.liquidity_shares_manager.deposit_fees(swap_fees)?; // Should only error unexpectedly!

            let mut external_fees: BalanceOf<T> = Zero::zero();
            for &market_id in pool.pool_type.iter_market_ids() {
                let f = T::ExternalFees::distribute(market_id, pool.collateral, account, amount);
                external_fees = external_fees.saturating_add(f);
            }

            let total_fees = external_fees.saturating_add(swap_fees);
            let remaining = amount.checked_sub(&total_fees).ok_or(Error::<T>::Unexpected)?;
            Ok(FeeDistribution { remaining, swap_fees, external_fees })
        }

        /// Takes `amount` units of collateral and splits these tokens into the elementary outcome
        /// tokens of the combinatorial market comprised of the specified markets (all specified
        /// markets must have the same collateral). Returns the collateral token type and a list of
        /// outcome tokens.
        pub(crate) fn split_markets(
            who: T::AccountId,
            market_ids: Vec<MarketIdOf<T>>,
            amount: BalanceOf<T>,
            fuel: FuelOf<T>,
        ) -> Result<(Vec<T::CombinatorialId>, Vec<AssetOf<T>>, AssetOf<T>), DispatchError> {
            let markets =
                market_ids.iter().map(T::MarketCommons::market).collect::<Result<Vec<_>, _>>()?;

            // Calculate the total amount of split operations required. One split for splitting
            // collateral into the positions of the first market, and then it's one split for each
            // position created in the previous step.
            let mut total_splits = 0u16;
            let mut prev_positions = 0u16;
            for market in markets.iter() {
                ensure!(
                    market.scoring_rule == ScoringRule::AmmCdaHybrid,
                    Error::<T>::InvalidTradingMechanism
                );

                if total_splits == 0u16 {
                    total_splits = 1u16;
                    prev_positions = market.outcomes();
                } else {
                    total_splits = total_splits.checked_add_res(&prev_positions)?;
                    prev_positions = prev_positions.checked_mul_res(&market.outcomes())?;
                }
            }
            ensure!(total_splits <= T::MaxSplits::get(), Error::<T>::MaxSplitsExceeded);

            let collateral = Self::try_common_collateral(market_ids.clone())?;

            let mut split_count = 0u16;
            let mut collection_ids: Vec<T::CombinatorialId> = vec![];
            let mut position_ids = vec![];
            for market_id in market_ids.iter() {
                let asset_count = T::MarketCommons::market(market_id)?.outcomes() as usize;
                let partition: Vec<Vec<bool>> = (0..asset_count)
                    .map(|index| {
                        let mut index_set = vec![false; asset_count];
                        if let Some(value) = index_set.get_mut(index) {
                            *value = true;
                        }

                        index_set
                    })
                    .collect();

                if split_count == 0 {
                    let split_position_info = T::CombinatorialTokens::split_position(
                        who.clone(),
                        None,
                        *market_id,
                        partition.clone(),
                        amount,
                        fuel.clone(),
                    )?;

                    collection_ids.extend_from_slice(&split_position_info.collection_ids);
                    position_ids.extend_from_slice(&split_position_info.position_ids);

                    split_count.saturating_inc();
                } else {
                    let mut new_collection_ids = vec![];
                    let mut new_position_ids = vec![];

                    for parent_collection_id in collection_ids.iter() {
                        if split_count > total_splits {
                            return Err(Error::<T>::Unexpected.into());
                        }

                        let split_position_info = T::CombinatorialTokens::split_position(
                            who.clone(),
                            Some(parent_collection_id.clone()),
                            *market_id,
                            partition.clone(),
                            amount,
                            fuel.clone(),
                        )?;

                        new_collection_ids.extend_from_slice(&split_position_info.collection_ids);
                        new_position_ids.extend_from_slice(&split_position_info.position_ids);

                        split_count.saturating_inc();
                    }

                    collection_ids = new_collection_ids;
                    position_ids = new_position_ids;
                }
            }

            let result = (collection_ids, position_ids, collateral);

            Ok(result)
        }

        pub(crate) fn try_common_collateral(
            market_ids: Vec<MarketIdOf<T>>,
        ) -> Result<AssetOf<T>, DispatchError> {
            let first_market_id = market_ids.first().ok_or(Error::<T>::InvalidMarketCount)?;
            let first_market = T::MarketCommons::market(first_market_id)?;
            let collateral = first_market.base_asset;

            for market_id in market_ids.iter() {
                let market = T::MarketCommons::market(market_id)?;
                ensure!(market.base_asset == collateral, Error::<T>::CollateralMismatch);
            }

            Ok(collateral)
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

    impl<T: Config> Pallet<T> {
        fn amount_including_fee_surplus(
            amount: BalanceOf<T>,
            fee_fractional: BalanceOf<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let fee_divisor = ZeitgeistBase::<BalanceOf<T>>::get()?
                .checked_sub(&fee_fractional)
                .ok_or(Error::<T>::Unexpected)?;
            amount.bdiv(fee_divisor)
        }

        fn total_fee_fractional(
            swap_fee: BalanceOf<T>,
            external_fee_percentage: Perbill,
        ) -> Result<BalanceOf<T>, DispatchError> {
            let external_fee_fractional =
                external_fee_percentage.mul_floor(ZeitgeistBase::<BalanceOf<T>>::get()?);
            swap_fee.checked_add_res(&external_fee_fractional)
        }

        fn match_failure(error: DispatchError) -> ApiError<AmmSoftFail> {
            let spot_price_too_low: DispatchError =
                Error::<T>::NumericalLimits(NumericalLimitsError::SpotPriceTooLow).into();
            let spot_price_slipped_too_low: DispatchError =
                Error::<T>::NumericalLimits(NumericalLimitsError::SpotPriceSlippedTooLow).into();
            let max_amount_exceeded: DispatchError =
                Error::<T>::NumericalLimits(NumericalLimitsError::MaxAmountExceeded).into();
            let min_amount_not_met: DispatchError =
                Error::<T>::NumericalLimits(NumericalLimitsError::MinAmountNotMet).into();
            if spot_price_too_low == error
                || spot_price_slipped_too_low == error
                || max_amount_exceeded == error
                || min_amount_not_met == error
            {
                ApiError::SoftFailure(AmmSoftFail::Numerical)
            } else {
                ApiError::HardFailure(error)
            }
        }
    }

    impl<T: Config> HybridRouterAmmApi for Pallet<T> {
        type AccountId = T::AccountId;
        type MarketId = MarketIdOf<T>;
        type Balance = BalanceOf<T>;
        type Asset = AssetOf<T>;

        fn pool_exists(market_id: Self::MarketId) -> bool {
            let Some(pool_id) = MarketIdToPoolId::<T>::get(market_id) else {
                return false;
            };
            Pools::<T>::contains_key(pool_id)
        }

        fn get_spot_price(
            market_id: Self::MarketId,
            asset: Self::Asset,
        ) -> Result<Self::Balance, DispatchError> {
            let pool = <Self as PoolStorage>::get(market_id)?;
            pool.calculate_spot_price(asset)
        }

        fn calculate_buy_amount_until(
            market_id: Self::MarketId,
            asset: Self::Asset,
            until: Self::Balance,
        ) -> Result<Self::Balance, DispatchError> {
            let pool = <Self as PoolStorage>::get(market_id)?;
            let buy_amount = pool.calculate_buy_amount_until(asset, until)?;
            let total_fee_fractional = Self::total_fee_fractional(
                pool.swap_fee,
                T::ExternalFees::fee_percentage(market_id),
            )?;
            let buy_amount_plus_fees =
                Self::amount_including_fee_surplus(buy_amount, total_fee_fractional)?;
            Ok(buy_amount_plus_fees)
        }

        fn buy(
            who: Self::AccountId,
            market_id: Self::MarketId,
            asset_out: Self::Asset,
            amount_in: Self::Balance,
            min_amount_out: Self::Balance,
        ) -> Result<AmmTradeOf<T>, ApiError<AmmSoftFail>> {
            Self::do_buy(who, market_id, asset_out, amount_in, min_amount_out)
                .map_err(Self::match_failure)
        }

        fn calculate_sell_amount_until(
            market_id: Self::MarketId,
            asset: Self::Asset,
            until: Self::Balance,
        ) -> Result<Self::Balance, DispatchError> {
            let pool = <Self as PoolStorage>::get(market_id)?;
            pool.calculate_sell_amount_until(asset, until)
        }

        fn sell(
            who: Self::AccountId,
            market_id: Self::MarketId,
            asset_out: Self::Asset,
            amount_in: Self::Balance,
            min_amount_out: Self::Balance,
        ) -> Result<AmmTradeOf<T>, ApiError<AmmSoftFail>> {
            Self::do_sell(who, market_id, asset_out, amount_in, min_amount_out)
                .map_err(Self::match_failure)
        }
    }
}
