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

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
mod tests;
mod types;
mod utils;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{
        types::{Strategy, TxType},
        weights::WeightInfoZeitgeist,
    };
    use alloc::{vec, vec::Vec};
    use core::marker::PhantomData;
    use frame_support::{
        ensure,
        pallet_prelude::{Decode, DispatchError, Encode, TypeInfo},
        require_transactional,
        traits::{IsType, StorageVersion},
        RuntimeDebug,
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use orml_traits::MultiCurrency;
    use sp_runtime::{
        traits::{CheckedSub, SaturatedConversion, Zero},
        ArithmeticError, DispatchResult,
    };
    use zeitgeist_primitives::{
        math::fixed::{BaseProvider, FixedDiv, FixedMul, ZeitgeistBase},
        order_book::{Order, OrderId},
        traits::{HybridRouterAmmApi, HybridRouterOrderBookApi},
        types::{Asset, Market, MarketType, ScalarPosition},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The api to handle different asset classes.
        type AssetManager: MultiCurrency<Self::AccountId, CurrencyId = AssetOf<Self>>;

        type MarketCommons: MarketCommonsPalletApi<
                AccountId = Self::AccountId,
                BlockNumber = Self::BlockNumber,
                Balance = BalanceOf<Self>,
            >;

        type Amm: HybridRouterAmmApi<
                AccountId = AccountIdOf<Self>,
                MarketId = MarketIdOf<Self>,
                Asset = AssetOf<Self>,
                Balance = BalanceOf<Self>,
            >;

        type OrderBook: HybridRouterOrderBookApi<
                AccountId = AccountIdOf<Self>,
                MarketId = MarketIdOf<Self>,
                Balance = BalanceOf<Self>,
                Asset = AssetOf<Self>,
                Order = OrderOf<Self>,
                OrderId = OrderId,
            >;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Weights generated by benchmarks.
        type WeightInfo: WeightInfoZeitgeist;
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

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A buy order was executed.
        HybridRouterBuyExecuted {
            who: AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            asset: AssetOf<T>,
            amount_in: BalanceOf<T>,
            max_price: BalanceOf<T>,
        },
        /// A sell order was executed.
        HybridRouterSellExecuted {
            who: AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            asset: AssetOf<T>,
            amount_in: BalanceOf<T>,
            min_price: BalanceOf<T>,
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
        /// Action cannot be completed because an unexpected error has occurred. This should be
        /// reported to protocol maintainers.
        InconsistentState(InconsistentStateError),
    }

    // NOTE: these errors should never happen.
    #[derive(Encode, Decode, Eq, PartialEq, TypeInfo, frame_support::PalletError, RuntimeDebug)]
    pub enum InconsistentStateError {}

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
        #[pallet::weight(T::WeightInfo::buy())]
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
        #[pallet::weight(T::WeightInfo::buy())]
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

        fn maybe_fill_from_amm(
            tx_type: TxType,
            who: &AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            asset: AssetOf<T>,
            amount_in: BalanceOf<T>,
            price_limit: BalanceOf<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            if !T::Amm::pool_exists(market_id) {
                return Ok(amount_in);
            }

            let spot_price = T::Amm::get_spot_price(market_id, asset)?;

            let mut remaining = amount_in;
            let amm_amount_in = match tx_type {
                TxType::Buy => {
                    if spot_price >= price_limit {
                        return Ok(amount_in);
                    }
                    T::Amm::calculate_buy_amount_until(market_id, asset, price_limit)?
                }
                TxType::Sell => {
                    if spot_price <= price_limit {
                        return Ok(amount_in);
                    }
                    T::Amm::calculate_sell_amount_until(market_id, asset, price_limit)?
                }
            };

            let amm_amount_in = amm_amount_in.min(remaining);

            if !amm_amount_in.is_zero() {
                match tx_type {
                    TxType::Buy => {
                        T::Amm::buy(&who, market_id, asset, amm_amount_in, BalanceOf::<T>::zero())?;
                    }
                    TxType::Sell => {
                        T::Amm::sell(
                            &who,
                            market_id,
                            asset,
                            amm_amount_in,
                            BalanceOf::<T>::zero(),
                        )?;
                    }
                }
                remaining = remaining
                    .checked_sub(&amm_amount_in)
                    .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))?;
            }

            Ok(remaining)
        }

        fn maybe_fill_orders(
            tx_type: TxType,
            orders: &[OrderId],
            mut remaining: BalanceOf<T>,
            who: &AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            base_asset: AssetOf<T>,
            asset: AssetOf<T>,
            price_limit: BalanceOf<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
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

                remaining = Self::maybe_fill_from_amm(
                    tx_type,
                    who,
                    market_id,
                    asset,
                    remaining,
                    order_price,
                )?;

                if remaining.is_zero() {
                    break;
                }

                // `remaining` is always denominated in the `taker_asset`
                // because this is what the order owner (maker) wants to receive
                let (_taker_fill, maker_fill) =
                    order.taker_and_maker_fill_from_taker_amount(remaining)?;
                // and the `maker_partial_fill` of `fill_order` is specified in `taker_asset`
                T::OrderBook::fill_order(who, order_id, Some(maker_fill))?;
                // `maker_fill` is the amount the order owner (maker) wants to receive
                remaining = remaining
                    .checked_sub(&maker_fill)
                    .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))?;
            }

            Ok(remaining)
        }

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
                        who,
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
            let market = T::MarketCommons::market(&market_id)?;
            let assets = Self::outcome_assets(market_id, &market);
            ensure!(asset_count as usize == assets.len(), Error::<T>::AssetCountMismatch);

            let asset_in = match tx_type {
                TxType::Buy => market.base_asset,
                TxType::Sell => asset,
            };
            T::AssetManager::ensure_can_withdraw(asset_in, &who, amount_in)?;

            let mut remaining = amount_in;

            remaining = Self::maybe_fill_orders(
                tx_type,
                &orders,
                remaining,
                &who,
                market_id,
                market.base_asset,
                asset,
                price_limit,
            )?;

            if !remaining.is_zero() {
                remaining = Self::maybe_fill_from_amm(
                    tx_type,
                    &who,
                    market_id,
                    asset,
                    remaining,
                    price_limit,
                )?;
            }

            if !remaining.is_zero() {
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
            }

            Self::deposit_event(Event::HybridRouterBuyExecuted {
                who: who.clone(),
                market_id,
                asset,
                amount_in,
                max_price: price_limit,
            });

            Ok(())
        }
    }
}
