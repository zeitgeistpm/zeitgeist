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
    use crate::{types::Strategy, weights::WeightInfoZeitgeist};
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
        math::fixed::FixedMul,
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
            amount: BalanceOf<T>,
            max_price: BalanceOf<T>,
        },
        /// A sell order was executed.
        HybridRouterSellExecuted {
            who: AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            asset: AssetOf<T>,
            amount: BalanceOf<T>,
            min_price: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
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
        /// * `amount`: The amount of `asset` to buy.
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
            #[pallet::compact] asset_count: u32,
            asset: AssetOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
            #[pallet::compact] max_price: BalanceOf<T>,
            orders: Vec<OrderId>,
            strategy: Strategy,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::do_buy(who, market_id, asset_count, asset, amount, max_price, orders, strategy)?;

            Ok(())
        }

        /// Routes a sell order to AMM and CDA to achieve the best average execution price.
        ///
        /// # Parameters
        ///
        /// * `market_id`: The ID of the market to sell on.
        /// * `asset_count`: The number of assets traded on the market.
        /// * `asset`: The asset to sell.
        /// * `amount`: The amount of `asset` to sell.
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
            #[pallet::compact] asset_count: u32,
            asset: AssetOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
            #[pallet::compact] min_price: BalanceOf<T>,
            orders: Vec<OrderId>,
            strategy: Strategy,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::do_sell(who, market_id, asset_count, asset, amount, min_price, orders, strategy)?;

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

        fn maybe_buy_from_amm(
            who: &AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            asset: AssetOf<T>,
            amount: BalanceOf<T>,
            max_price: BalanceOf<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            if !T::Amm::pool_exists(market_id) {
                return Ok(amount);
            }

            let mut remaining = amount;
            let amm_amount = T::Amm::calculate_buy_amount_until(market_id, asset, max_price)?;
            let amm_amount = amm_amount.min(remaining);

            if !amm_amount.is_zero() {
                T::Amm::buy(&who, market_id, asset, amm_amount, BalanceOf::<T>::zero())?;
                remaining = remaining
                    .checked_sub(&amm_amount)
                    .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))?;
            }

            Ok(remaining)
        }

        #[require_transactional]
        fn do_buy(
            who: AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            asset_count: u32,
            asset: AssetOf<T>,
            amount: BalanceOf<T>,
            max_price: BalanceOf<T>,
            orders: Vec<OrderId>,
            strategy: Strategy,
        ) -> DispatchResult {
            let market = T::MarketCommons::market(&market_id)?;
            let assets = Self::outcome_assets(market_id, &market);
            let assets_len: u32 = assets.len().saturated_into();
            ensure!(asset_count == assets_len, Error::<T>::AssetCountMismatch);

            let required_base_asset_amount = amount.bmul_ceil(max_price)?;
            T::AssetManager::ensure_can_withdraw(
                market.base_asset,
                &who,
                required_base_asset_amount,
            )?;

            let mut remaining = amount;

            for order_id in orders {
                if remaining.is_zero() {
                    break;
                }

                let order = match T::OrderBook::order(order_id) {
                    Ok(order) => order,
                    Err(_) => continue,
                };

                // existing order is willing to give the required `asset` as the `maker_asset`
                ensure!(asset == order.maker_asset, Error::<T>::AssetNotEqualToOrderBookMakerAsset);

                let order_price = order.price(market.base_asset)?;
                ensure!(order_price <= max_price, Error::<T>::OrderPriceAboveMaxPrice);

                remaining =
                    Self::maybe_buy_from_amm(&who, market_id, asset, remaining, order_price)?;

                if remaining.is_zero() {
                    break;
                }

                // `remaining` is denominated in `asset`, so the `maker_asset`
                // but `maker_partial_fill` of `fill_order` is specified in `taker_asset`
                let (taker_fill, maker_fill) =
                    order.taker_and_maker_fill_from_maker_amount(remaining)?;
                T::OrderBook::fill_order(&who, order_id, Some(maker_fill))?;
                remaining = remaining
                    .checked_sub(&taker_fill)
                    .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))?;
            }

            if !remaining.is_zero() {
                remaining = Self::maybe_buy_from_amm(&who, market_id, asset, remaining, max_price)?;
            }

            if !remaining.is_zero() {
                match strategy {
                    Strategy::ImmediateOrCancel => {
                        return Err(Error::<T>::CancelStrategyApplied.into());
                    }
                    Strategy::LimitOrder => {
                        let maker_amount = max_price.bmul_floor(remaining)?;
                        T::OrderBook::place_order(
                            &who,
                            market_id,
                            market.base_asset,
                            maker_amount,
                            asset,
                            remaining,
                        )?;
                    }
                }
            }

            // TODO: Do we want to emit an event for the Hybrid Router at all if Order Book and AMM do already?
            Self::deposit_event(Event::HybridRouterBuyExecuted {
                who: who.clone(),
                market_id,
                asset,
                amount,
                max_price,
            });

            Ok(())
        }

        fn maybe_sell_from_amm(
            who: &AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            asset: AssetOf<T>,
            amount: BalanceOf<T>,
            min_price: BalanceOf<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            if !T::Amm::pool_exists(market_id) {
                return Ok(amount);
            }

            let mut remaining = amount;
            let amm_amount = T::Amm::calculate_sell_amount_until(market_id, asset, min_price)?;
            let amm_amount = amm_amount.min(remaining);

            if !amm_amount.is_zero() {
                T::Amm::sell(&who, market_id, asset, amm_amount, BalanceOf::<T>::zero())?;
                remaining = remaining
                    .checked_sub(&amm_amount)
                    .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))?;
            }

            Ok(remaining)
        }

        #[require_transactional]
        fn do_sell(
            who: AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            asset_count: u32,
            asset: AssetOf<T>,
            amount: BalanceOf<T>,
            min_price: BalanceOf<T>,
            orders: Vec<OrderId>,
            strategy: Strategy,
        ) -> DispatchResult {
            let market = T::MarketCommons::market(&market_id)?;
            let assets = Self::outcome_assets(market_id, &market);
            let assets_len: u32 = assets.len().saturated_into();
            ensure!(asset_count == assets_len, Error::<T>::AssetCountMismatch);

            T::AssetManager::ensure_can_withdraw(asset, &who, amount)?;

            let mut remaining = amount;

            for order_id in orders {
                if remaining.is_zero() {
                    break;
                }

                let order = match T::OrderBook::order(order_id) {
                    Ok(order) => order,
                    Err(_) => continue,
                };

                ensure!(asset == order.taker_asset, Error::<T>::AssetNotEqualToOrderBookTakerAsset);

                let order_price = order.price(market.base_asset)?;
                ensure!(order_price >= min_price, Error::<T>::OrderPriceBelowMinPrice);

                remaining =
                    Self::maybe_sell_from_amm(&who, market_id, asset, remaining, order_price)?;

                if remaining.is_zero() {
                    break;
                }

                // `remaining` is denominated in `asset`, so the `taker_asset`
                // and the `maker_partial_fill` of `fill_order` is specified in `taker_asset`
                let (taker_fill, maker_fill) =
                    order.taker_and_maker_fill_from_taker_amount(remaining)?;
                T::OrderBook::fill_order(&who, order_id, Some(maker_fill))?;
                remaining = remaining
                    .checked_sub(&taker_fill)
                    .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))?;
            }

            if !remaining.is_zero() {
                remaining =
                    Self::maybe_sell_from_amm(&who, market_id, asset, remaining, min_price)?;
            }

            if !remaining.is_zero() {
                match strategy {
                    Strategy::ImmediateOrCancel => {
                        return Err(Error::<T>::CancelStrategyApplied.into());
                    }
                    Strategy::LimitOrder => {
                        let maker_amount = min_price.bmul_floor(remaining)?;
                        T::OrderBook::place_order(
                            &who,
                            market_id,
                            asset,
                            maker_amount,
                            market.base_asset,
                            remaining,
                        )?;
                    }
                }
            }

            Self::deposit_event(Event::HybridRouterSellExecuted {
                who: who.clone(),
                market_id,
                asset,
                amount,
                min_price,
            });

            Ok(())
        }
    }
}
