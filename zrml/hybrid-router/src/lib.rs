// Copyright 2024 Forecasting Technologies LTD.
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

mod types;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::types::{PendingOrderAmounts, Strategy, TxType};
    use alloc::{vec, vec::Vec};
    use core::marker::PhantomData;
    use frame_support::{
        ensure,
        pallet_prelude::{Decode, DispatchError, Encode, TypeInfo},
        require_transactional,
        traits::{IsType, StorageVersion},
        BoundedVec, RuntimeDebug,
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use orml_traits::MultiCurrency;
    use sp_runtime::{
        traits::{CheckedSub, Get, SaturatedConversion, Zero},
        ArithmeticError, DispatchResult,
    };
    use zeitgeist_primitives::{
        hybrid_router_api_types::{AmmTrade, OrderbookTrade},
        math::{
            checked_ops_res::{CheckedAddRes, CheckedSubRes},
            fixed::{BaseProvider, FixedDiv, FixedMul, ZeitgeistBase},
        },
        orderbook::{Order, OrderId},
        traits::{HybridRouterAmmApi, HybridRouterOrderbookApi},
        types::{Asset, Market, MarketType, ScalarPosition},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The API to handle different asset classes.
        type AssetManager: MultiCurrency<Self::AccountId, CurrencyId = AssetOf<Self>>;

        /// The identifier of individual markets.
        type MarketCommons: MarketCommonsPalletApi<
                AccountId = Self::AccountId,
                BlockNumber = Self::BlockNumber,
                Balance = BalanceOf<Self>,
            >;

        /// The API to handle the Automated Market Maker (AMM).
        type Amm: HybridRouterAmmApi<
                AccountId = AccountIdOf<Self>,
                MarketId = MarketIdOf<Self>,
                Asset = AssetOf<Self>,
                Balance = BalanceOf<Self>,
            >;

        /// The maximum number of orders that can be used to execute a trade.
        #[pallet::constant]
        type MaxOrders: Get<u32>;

        /// The API to handle the order book.
        type OrderBook: HybridRouterOrderbookApi<
                AccountId = AccountIdOf<Self>,
                MarketId = MarketIdOf<Self>,
                Balance = BalanceOf<Self>,
                Asset = AssetOf<Self>,
                Order = OrderOf<Self>,
                OrderId = OrderId,
            >;

        /// The event type for this pallet.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);
    const LOG_TARGET: &str = "runtime::zrml-hybrid-router";

    pub(crate) type AssetOf<T> = Asset<MarketIdOf<T>>;
    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::AssetManager as MultiCurrency<AccountIdOf<T>>>::Balance;
    pub(crate) type OrderOf<T> = Order<AccountIdOf<T>, BalanceOf<T>, MarketIdOf<T>>;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub(crate) type MarketOf<T> =
        Market<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, MomentOf<T>, Asset<MarketIdOf<T>>>;
    pub(crate) type OrdersOf<T> = BoundedVec<OrderId, <T as Config>::MaxOrders>;
    pub(crate) type AmmTradeOf<T> = AmmTrade<BalanceOf<T>>;
    pub(crate) type OrderTradesOf<T> = OrderbookTrade<BalanceOf<T>>;
    pub(crate) type PendingOrderAmountsOf<T> = PendingOrderAmounts<BalanceOf<T>>;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A trade was executed.
        HybridRouterExecuted {
            /// The type of transaction (Buy or Sell).
            tx_type: TxType,
            /// The account ID of the user performing the trade.
            who: AccountIdOf<T>,
            /// The ID of the market.
            market_id: MarketIdOf<T>,
            /// The maximum price limit for buying or the minimum price limit for selling.
            price_limit: BalanceOf<T>,
            /// The asset provided by the trader.
            asset_in: AssetOf<T>,
            /// The amount of the `asset_in` provided by the trader.
            amount_in: BalanceOf<T>,
            /// The asset received by the trader.
            asset_out: AssetOf<T>,
            /// The aggregated amount of the `asset_out` already received
            /// by the trader from AMM and orderbook.
            amount_out: BalanceOf<T>,
            /// The AMM trades that were executed and their information about the amounts.
            amm_trades: Vec<AmmTradeOf<T>>,
            /// The orderbook trades that were executed and their information about the amounts.
            orderbook_trades: Vec<OrderTradesOf<T>>,
            /// The remaining amounts after the trade placed as a limit order.
            pending_order_amounts: Option<PendingOrderAmountsOf<T>>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The specified amount is zero.
        AmountIsZero,
        /// The price limit is too high.
        PriceLimitTooHigh,
        /// The price of an order is above the specified maximum price.
        OrderPriceAboveMaxPrice,
        /// The price of an order is below the specified minimum price.
        OrderPriceBelowMinPrice,
        /// The asset of an order is not equal to the maker asset of the order book.
        AssetNotEqualToOrderBookMakerAsset,
        /// The asset of an order is not equal to the taker asset of the order book.
        AssetNotEqualToOrderBookTakerAsset,
        /// The strategy "immediate or cancel" was applied.
        CancelStrategyApplied,
        /// The asset count does not match the markets asset count.
        AssetCountMismatch,
        /// The maximum number of orders was exceeded.
        MaxOrdersExceeded,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Routes a buy order to AMM and CDA to achieve the best average execution price.
        ///
        /// # Parameters
        ///
        /// * `market_id`: The ID of the market to buy from.
        /// * `asset_count`: The number of assets traded on the market.
        /// * `asset`: The asset to buy.
        /// * `amount_in`: The amount of the market's base asset to sell.
        /// * `max_price`: The maximum price to buy at.
        /// * `orders`: A list of orders from the book to use.
        /// * `strategy`: The strategy to handle the remaining order when the `max_price` is reached.
        ///
        /// The elements of `orders` are the orders that the router may use to execute the order. If any of
        /// these orders are already filled, they are ignored. It is not necessary for the router to use all
        /// specified orders. The smaller the vector, the larger the risk that the AMM is used to fill large
        /// chunks of the order.
        ///
        /// The `orders` vector **must** be sorted in ascending order by the price of their associated
        /// orders. Failing this, the behavior of `buy` is undefined.
        ///
        /// If the maximum price is reached before the entire buy order is filled, the `strategy` parameter
        /// decides if the order is rolled back (`Strategy::ImmediateOrCancel`) or if a limit order for the
        /// remaining amount is placed (`Strategy::LimitOrder`).
        ///
        /// Complexity: `O(n)`
        #[pallet::call_index(0)]
        #[pallet::weight(5000)]
        #[frame_support::transactional]
        pub fn buy(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
            #[pallet::compact] asset_count: u16,
            asset: AssetOf<T>,
            #[pallet::compact] amount_in: BalanceOf<T>,
            #[pallet::compact] max_price: BalanceOf<T>,
            orders: Vec<OrderId>,
            strategy: Strategy,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::do_trade(
                TxType::Buy,
                who,
                market_id,
                asset_count,
                asset,
                amount_in,
                max_price,
                orders,
                strategy,
            )?;

            Ok(())
        }

        /// Routes a sell order to AMM and CDA to achieve the best average execution price.
        ///
        /// # Parameters
        ///
        /// * `market_id`: The ID of the market to sell on.
        /// * `asset_count`: The number of assets traded on the market.
        /// * `asset`: The asset to sell.
        /// * `amount_in`: The amount of `asset` to sell.
        /// * `min_price`: The minimum price to sell at.
        /// * `orders`: A list of orders from the book to use.
        /// * `strategy`: The strategy to handle the remaining order when the `min_price` is reached.
        ///
        /// The elements of `orders` are the orders that the router may use to execute the order. If any of
        /// these orders are already filled, they are ignored. It is not necessary for the router to use all
        /// specified orders. The smaller the vector, the larger the risk that the AMM is used to fill large
        /// chunks of the order.
        ///
        /// The `orders` vector **must** be sorted in ascending order by the price of their associated
        /// orders. Failing this, the behavior of `sell` is undefined.
        ///
        /// If the maximum price is reached before the entire buy order is filled, the `strategy` parameter
        /// decides if the order is rolled back (`Strategy::ImmediateOrCancel`) or if a limit order for the
        /// remaining amount is placed (`Strategy::LimitOrder`).
        ///
        /// Complexity: `O(n)`
        #[pallet::call_index(1)]
        #[pallet::weight(5000)]
        #[frame_support::transactional]
        pub fn sell(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
            #[pallet::compact] asset_count: u16,
            asset: AssetOf<T>,
            #[pallet::compact] amount_in: BalanceOf<T>,
            #[pallet::compact] min_price: BalanceOf<T>,
            orders: Vec<OrderId>,
            strategy: Strategy,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::do_trade(
                TxType::Sell,
                who,
                market_id,
                asset_count,
                asset,
                amount_in,
                min_price,
                orders,
                strategy,
            )?;

            Ok(())
        }
    }

    impl<T> Pallet<T>
    where
        T: Config,
    {
        /// Returns a vector of assets corresponding to the given market ID and market type.
        ///
        /// # Arguments
        ///
        /// * `market_id` - The ID of the market.
        /// * `market` - A reference to the market.
        ///
        /// # Returns
        ///
        /// A vector of assets based on the market type. If the market type is `Categorical`,
        /// the function creates a vector of `CategoricalOutcome` assets with the given market ID
        /// and category index. If the market type is `Scalar`, the function creates a vector
        /// containing `ScalarOutcome` assets with the given market ID and both `Long` and `Short` positions.
        pub fn outcome_assets(market_id: MarketIdOf<T>, market: &MarketOf<T>) -> Vec<AssetOf<T>> {
            match market.market_type {
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
            }
        }

        /// Fills the order from the Automated Market Maker (AMM) if it exists and meets the price conditions.
        ///
        /// # Arguments
        ///
        /// * `tx_type` - The type of transaction (Buy or Sell).
        /// * `who` - The account ID of the user performing the transaction.
        /// * `market_id` - The ID of the market.
        /// * `asset` - The asset to be traded.
        /// * `amount_in` - The amount to be traded.
        /// * `price_limit` - The maximum or minimum price at which the trade can be executed.
        ///
        /// # Returns
        ///
        /// The remaining amount after filling the order from the AMM, or an error if the order cannot be filled.
        /// If the a trade was executed, trade information is returned from the event.
        /// Otherwise `None` is returned for the trade information.
        /// The trade information is useful for event information.
        fn maybe_fill_from_amm(
            tx_type: TxType,
            who: &AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            asset: AssetOf<T>,
            amount_in: BalanceOf<T>,
            price_limit: BalanceOf<T>,
        ) -> Result<(BalanceOf<T>, Option<AmmTradeOf<T>>), DispatchError> {
            if !T::Amm::pool_exists(market_id) {
                return Ok((amount_in, None));
            }

            let spot_price = T::Amm::get_spot_price(market_id, asset)?;

            let amm_amount_in = match tx_type {
                TxType::Buy => {
                    if spot_price >= price_limit {
                        return Ok((amount_in, None));
                    }
                    T::Amm::calculate_buy_amount_until(market_id, asset, price_limit)?
                }
                TxType::Sell => {
                    if spot_price <= price_limit {
                        return Ok((amount_in, None));
                    }
                    T::Amm::calculate_sell_amount_until(market_id, asset, price_limit)?
                }
            };

            let amm_amount_in = amm_amount_in.min(amount_in);

            if amm_amount_in.is_zero() {
                return Ok((amount_in, None));
            }

            let amm_trade_info = match tx_type {
                TxType::Buy => T::Amm::buy(
                    who.clone(),
                    market_id,
                    asset,
                    amm_amount_in,
                    BalanceOf::<T>::zero(),
                )?,
                TxType::Sell => T::Amm::sell(
                    who.clone(),
                    market_id,
                    asset,
                    amm_amount_in,
                    BalanceOf::<T>::zero(),
                )?,
            };

            Ok((amount_in.checked_sub_res(&amm_amount_in)?, Some(amm_trade_info)))
        }

        /// Fills the order from the order book if it exists and meets the price conditions.
        /// If the order is partially filled, the remaining amount is returned.
        ///
        /// # Arguments
        ///
        /// * `tx_type` - The type of transaction (Buy or Sell).
        /// * `orders` - A list of orders from the order book.
        /// * `remaining` - The amount to be traded.
        /// * `who` - The account ID of the user performing the transaction.
        /// * `market_id` - The ID of the market.
        /// * `base_asset` - The base asset of the market.
        /// * `asset` - The asset to be traded.
        /// * `price_limit` - The maximum or minimum price at which the trade can be executed.
        ///
        /// # Returns
        ///
        /// The remaining amount after filling the order, or an error if the order cannot be filled.
        fn maybe_fill_orders(
            tx_type: TxType,
            orders: &[OrderId],
            mut remaining: BalanceOf<T>,
            who: &AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            base_asset: AssetOf<T>,
            asset: AssetOf<T>,
            price_limit: BalanceOf<T>,
        ) -> Result<(BalanceOf<T>, Vec<AmmTradeOf<T>>, Vec<OrderTradesOf<T>>), DispatchError>
        {
            let mut amm_trades = Vec::new();
            let mut order_trades = Vec::new();
            for &order_id in orders {
                if remaining.is_zero() {
                    break;
                }

                let order = match T::OrderBook::order(order_id) {
                    Ok(order) => order,
                    Err(_) => continue,
                };

                let order_price = order.price(base_asset)?;

                match tx_type {
                    TxType::Buy => {
                        // existing order is willing to give the required `asset` as the `maker_asset`
                        ensure!(
                            asset == order.maker_asset,
                            Error::<T>::AssetNotEqualToOrderBookMakerAsset
                        );
                        ensure!(order_price <= price_limit, Error::<T>::OrderPriceAboveMaxPrice);
                    }
                    TxType::Sell => {
                        // existing order is willing to receive the required `asset` as the `taker_asset`
                        ensure!(
                            asset == order.taker_asset,
                            Error::<T>::AssetNotEqualToOrderBookTakerAsset
                        );
                        ensure!(order_price >= price_limit, Error::<T>::OrderPriceBelowMinPrice);
                    }
                }

                let amm_trade_info = Self::maybe_fill_from_amm(
                    tx_type,
                    who,
                    market_id,
                    asset,
                    remaining,
                    order_price,
                )?;
                amm_trade_info.1.map(|t| amm_trades.push(t));
                remaining = amm_trade_info.0;

                if remaining.is_zero() {
                    break;
                }

                // `remaining` is always denominated in the `taker_asset`
                // because this is what the order owner (maker) wants to receive
                let (_taker_fill, maker_fill) =
                    order.taker_and_maker_fill_from_taker_amount(remaining)?;
                // and the `maker_partial_fill` of `fill_order` is specified in `taker_asset`
                let order_trade =
                    T::OrderBook::fill_order(who.clone(), order_id, Some(maker_fill))?;
                order_trades.push(order_trade);
                // `maker_fill` is the amount the order owner (maker) wants to receive
                remaining = remaining.checked_sub_res(&maker_fill)?;
            }

            Ok((remaining, amm_trades, order_trades))
        }

        /// Places a limit order if the strategy is `Strategy::LimitOrder`.
        /// If the strategy is `Strategy::ImmediateOrCancel`, an error is returned.
        ///
        /// # Arguments
        ///
        /// * `strategy` - The strategy to handle the remaining non-zero amount when the `max_price` is reached.
        /// * `who` - The account ID of the user performing the transaction.
        /// * `market_id` - The ID of the market.
        /// * `maker_asset` - The asset to provide.
        /// * `maker_amount` - The amount of the `maker_asset` to be provided.
        /// * `taker_asset` - The asset to be received.
        /// * `taker_amount` - The amount of the `taker_asset` to be received.
        ///
        /// # Returns
        ///
        /// An error if the strategy is `Strategy::ImmediateOrCancel`.
        /// Otherwise, the limit order is placed.
        fn maybe_place_limit_order(
            strategy: Strategy,
            who: &AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            maker_asset: AssetOf<T>,
            maker_amount: BalanceOf<T>,
            taker_asset: AssetOf<T>,
            taker_amount: BalanceOf<T>,
        ) -> DispatchResult {
            match strategy {
                Strategy::ImmediateOrCancel => {
                    return Err(Error::<T>::CancelStrategyApplied.into());
                }
                Strategy::LimitOrder => {
                    T::OrderBook::place_order(
                        who.clone(),
                        market_id,
                        maker_asset,
                        maker_amount,
                        taker_asset,
                        taker_amount,
                    )?;
                }
            }

            Ok(())
        }

        /// Executes a trade by routing the order to the Automated Market Maker (AMM) and the Order Book
        /// to achieve the best average execution price.
        ///
        /// # Arguments
        ///
        /// * `tx_type` - The type of transaction (Buy or Sell).
        /// * `who` - The account ID of the user performing the transaction.
        /// * `market_id` - The ID of the market.
        /// * `asset_count` - The number of assets traded on the market.
        /// * `asset` - The asset to be traded.
        /// * `amount_in` - The amount to be traded.
        /// * `price_limit` - The maximum or minimum price at which the trade can be executed.
        /// * `orders` - A list of orders from the order book.
        /// * `strategy` - The strategy to handle the remaining non-zero amount when the `max_price` is reached.
        ///
        /// # Returns
        ///
        /// An error if the strategy is `Strategy::ImmediateOrCancel` and the full amount cannot be filled.
        /// Otherwise, the trade is executed and maybe places an order,
        /// if the full amount could not be processed at the specified price limit.
        #[require_transactional]
        pub(crate) fn do_trade(
            tx_type: TxType,
            who: AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            asset_count: u16,
            asset: AssetOf<T>,
            amount_in: BalanceOf<T>,
            price_limit: BalanceOf<T>,
            orders: Vec<OrderId>,
            strategy: Strategy,
        ) -> DispatchResult {
            ensure!(amount_in > BalanceOf::<T>::zero(), Error::<T>::AmountIsZero);
            ensure!(
                price_limit <= ZeitgeistBase::<BalanceOf<T>>::get()?,
                Error::<T>::PriceLimitTooHigh
            );
            ensure!(orders.len() as u32 <= T::MaxOrders::get(), Error::<T>::MaxOrdersExceeded);
            let orders: OrdersOf<T> =
                orders.try_into().map_err(|_| Error::<T>::MaxOrdersExceeded)?;
            let market = T::MarketCommons::market(&market_id)?;
            let assets = Self::outcome_assets(market_id, &market);
            ensure!(asset_count as usize == assets.len(), Error::<T>::AssetCountMismatch);

            let (asset_in, asset_out) = match tx_type {
                TxType::Buy => (market.base_asset, asset),
                TxType::Sell => (asset, market.base_asset),
            };
            T::AssetManager::ensure_can_withdraw(asset_in, &who, amount_in)?;

            let mut amm_trades: Vec<AmmTradeOf<T>> = Vec::new();
            let mut remaining = amount_in;

            let order_amm_trades_info = Self::maybe_fill_orders(
                tx_type,
                &orders,
                remaining,
                &who,
                market_id,
                market.base_asset,
                asset,
                price_limit,
            )?;

            remaining = order_amm_trades_info.0;
            amm_trades.extend(order_amm_trades_info.1);
            let orderbook_trades = order_amm_trades_info.2;

            if !remaining.is_zero() {
                let amm_trade_info = Self::maybe_fill_from_amm(
                    tx_type,
                    &who,
                    market_id,
                    asset,
                    remaining,
                    price_limit,
                )?;

                amm_trades.extend(amm_trade_info.1);
                remaining = amm_trade_info.0;
            }

            let pending_order_amounts = if !remaining.is_zero() {
                let (maker_asset, maker_amount, taker_asset, taker_amount) = match tx_type {
                    TxType::Buy => {
                        let maker_asset = market.base_asset;
                        let maker_amount = remaining;
                        let taker_asset = asset;
                        let taker_amount = remaining.bdiv_ceil(price_limit)?;
                        (maker_asset, maker_amount, taker_asset, taker_amount)
                    }
                    TxType::Sell => {
                        let maker_asset = asset;
                        let maker_amount = remaining;
                        let taker_asset = market.base_asset;
                        let taker_amount = price_limit.bmul_floor(remaining)?;
                        (maker_asset, maker_amount, taker_asset, taker_amount)
                    }
                };

                Self::maybe_place_limit_order(
                    strategy,
                    &who,
                    market_id,
                    maker_asset,
                    maker_amount,
                    taker_asset,
                    taker_amount,
                )?;

                Some(PendingOrderAmounts { maker_amount, taker_amount })
            } else {
                None
            };

            let amount_out = orderbook_trades
                .iter()
                .map(|o| o.filled_maker_amount.saturated_into::<u128>())
                .sum::<u128>()
                .checked_add_res(
                    &amm_trades.iter().map(|t| t.amount_out.saturated_into::<u128>()).sum::<u128>(),
                )?
                .saturated_into::<BalanceOf<T>>();

            Self::deposit_event(Event::HybridRouterExecuted {
                tx_type,
                who,
                market_id,
                price_limit,
                asset_in,
                amount_in,
                asset_out,
                amount_out,
                amm_trades,
                orderbook_trades,
                pending_order_amounts,
            });

            Ok(())
        }
    }
}
