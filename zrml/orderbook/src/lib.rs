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

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use crate::{types::*, weights::*};
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use frame_support::{
    dispatch::DispatchResultWithPostInfo,
    ensure,
    pallet_prelude::{OptionQuery, StorageMap, StorageValue, ValueQuery},
    traits::{IsType, StorageVersion},
    transactional, PalletId, Twox64Concat,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use orml_traits::{BalanceStatus, MultiCurrency, NamedMultiReservableCurrency};
pub use pallet::*;
use sp_runtime::{
    traits::{Get, Zero},
    SaturatedConversion, Saturating,
};
use zeitgeist_primitives::{
    math::{
        checked_ops_res::{CheckedAddRes, CheckedSubRes},
        fixed::{FixedDiv, FixedMul},
    },
    traits::MarketCommonsPalletApi,
    types::{Asset, Market, MarketStatus, MarketType, ScalarPosition, ScoringRule},
};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod mock;
#[cfg(test)]
mod tests;
pub mod types;
mod utils;
pub mod weights;

#[frame_support::pallet]
mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Shares of outcome assets and native currency
        type AssetManager: NamedMultiReservableCurrency<
                Self::AccountId,
                CurrencyId = Asset<MarketIdOf<Self>>,
                ReserveIdentifier = [u8; 8],
            >;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type MarketCommons: MarketCommonsPalletApi<
                AccountId = Self::AccountId,
                BlockNumber = Self::BlockNumber,
                Balance = BalanceOf<Self>,
            >;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        type WeightInfo: WeightInfoZeitgeist;
    }

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    pub(crate) type BalanceOf<T> = <<T as Config>::AssetManager as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type OrderOf<T> = Order<AccountIdOf<T>, BalanceOf<T>, MarketIdOf<T>>;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub(crate) type MarketOf<T> = Market<
        AccountIdOf<T>,
        BalanceOf<T>,
        <T as frame_system::Config>::BlockNumber,
        MomentOf<T>,
        Asset<MarketIdOf<T>>,
    >;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    pub type NextOrderId<T> = StorageValue<_, OrderId, ValueQuery>;

    #[pallet::storage]
    pub type Orders<T: Config> = StorageMap<_, Twox64Concat, OrderId, OrderOf<T>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        OrderFilled {
            order_id: OrderId,
            maker: AccountIdOf<T>,
            taker: AccountIdOf<T>,
            filled: BalanceOf<T>,
            unfilled_outcome_asset_amount: BalanceOf<T>,
            unfilled_base_asset_amount: BalanceOf<T>,
        },
        OrderPlaced {
            order_id: OrderId,
            order: OrderOf<T>,
        },
        OrderRemoved {
            order_id: OrderId,
            maker: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The sender is not the order creator.
        NotOrderCreator,
        /// The order does not exist.
        OrderDoesNotExist,
        /// The market is not active.
        MarketIsNotActive,
        /// The scoring rule is not orderbook.
        InvalidScoringRule,
        /// The specified amount parameter is too high for the order.
        AmountTooHighForOrder,
        /// The specified amount parameter is zero.
        AmountIsZero,
        /// The specified outcome asset is not part of the market.
        InvalidOutcomeAsset,
        /// The maker partial fill leads to a too low quotient for the next order execution.
        MakerPartialFillTooLow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Removes an order.
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(0)]
        #[pallet::weight(
            T::WeightInfo::remove_order_ask().max(T::WeightInfo::remove_order_bid())
        )]
        #[transactional]
        pub fn remove_order(origin: OriginFor<T>, order_id: OrderId) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let order_data = <Orders<T>>::get(order_id).ok_or(Error::<T>::OrderDoesNotExist)?;

            let maker = &order_data.maker;
            ensure!(sender == *maker, Error::<T>::NotOrderCreator);

            match order_data.side {
                OrderSide::Bid => {
                    let missing = T::AssetManager::unreserve_named(
                        &Self::reserve_id(),
                        order_data.base_asset,
                        maker,
                        order_data.base_asset_amount,
                    );
                    debug_assert!(
                        missing.is_zero(),
                        "Could not unreserve all of the amount. reserve_id: {:?}, asset: {:?} \
                         who: {:?}, amount: {:?}, missing: {:?}",
                        Self::reserve_id(),
                        order_data.base_asset,
                        maker,
                        order_data.base_asset_amount,
                        missing,
                    );
                }
                OrderSide::Ask => {
                    let missing = T::AssetManager::unreserve_named(
                        &Self::reserve_id(),
                        order_data.outcome_asset,
                        maker,
                        order_data.outcome_asset_amount,
                    );
                    debug_assert!(
                        missing.is_zero(),
                        "Could not unreserve all of the amount. reserve_id: {:?}, asset: {:?} \
                         who: {:?}, amount: {:?}, missing: {:?}",
                        Self::reserve_id(),
                        order_data.outcome_asset,
                        maker,
                        order_data.outcome_asset_amount,
                        missing,
                    );
                }
            }

            <Orders<T>>::remove(order_id);

            Self::deposit_event(Event::OrderRemoved { order_id, maker: maker.clone() });

            match order_data.side {
                OrderSide::Bid => Ok(Some(T::WeightInfo::remove_order_bid()).into()),
                OrderSide::Ask => Ok(Some(T::WeightInfo::remove_order_ask()).into()),
            }
        }

        /// Fill an existing order entirely (`maker_partial_fill` = None)
        /// or partially (`maker_partial_fill` = Some(partial_amount)).
        ///
        /// NOTE: The `maker_partial_fill` is the partial amount of what the maker wants to fill.
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(1)]
        #[pallet::weight(
            T::WeightInfo::fill_order_ask().max(T::WeightInfo::fill_order_bid())
        )]
        #[transactional]
        pub fn fill_order(
            origin: OriginFor<T>,
            order_id: OrderId,
            maker_partial_fill: Option<BalanceOf<T>>,
        ) -> DispatchResultWithPostInfo {
            let taker = ensure_signed(origin)?;

            let mut order_data = <Orders<T>>::get(order_id).ok_or(Error::<T>::OrderDoesNotExist)?;
            let market = T::MarketCommons::market(&order_data.market_id)?;
            ensure!(market.scoring_rule == ScoringRule::Orderbook, Error::<T>::InvalidScoringRule);
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketIsNotActive);
            let base_asset = market.base_asset;

            let makers_requested_total = match order_data.side {
                OrderSide::Bid => order_data.outcome_asset_amount,
                OrderSide::Ask => order_data.base_asset_amount,
            };
            let maker_fill = maker_partial_fill.unwrap_or(makers_requested_total);
            ensure!(!maker_fill.is_zero(), Error::<T>::AmountIsZero);
            ensure!(maker_fill <= makers_requested_total, Error::<T>::AmountTooHighForOrder);

            let maker = order_data.maker.clone();

            // the reserve of the maker should always be enough to repatriate successfully, e.g. taker gets a little bit less
            // it should always ensure that the maker's request (maker_fill) is fully filled
            match order_data.side {
                OrderSide::Bid => {
                    T::AssetManager::ensure_can_withdraw(
                        order_data.outcome_asset,
                        &taker,
                        maker_fill,
                    )?;

                    // Note that this always rounds down, i.e. the taker will always get a little bit less than what they asked for.
                    // This ensures that the reserve of the maker is always enough to repatriate successfully!
                    let ratio = maker_fill.bdiv_floor(order_data.outcome_asset_amount)?;
                    let taker_fill = ratio.bmul_floor(order_data.base_asset_amount)?;

                    T::AssetManager::repatriate_reserved_named(
                        &Self::reserve_id(),
                        base_asset,
                        &maker,
                        &taker,
                        taker_fill,
                        BalanceStatus::Free,
                    )?;

                    T::AssetManager::transfer(
                        order_data.outcome_asset,
                        &taker,
                        &maker,
                        maker_fill,
                    )?;

                    order_data.base_asset_amount =
                        order_data.base_asset_amount.checked_sub_res(&taker_fill)?;
                    order_data.outcome_asset_amount =
                        order_data.outcome_asset_amount.checked_sub_res(&maker_fill)?;
                    // this ensures that partial fills, which fill nearly the whole order, are not executed
                    // this protects the last fill happening without a division by zero for `Perquintill::from_rational`
                    let is_ratio_quotient_valid = order_data.outcome_asset_amount.is_zero()
                        || order_data.outcome_asset_amount.saturated_into::<u128>() >= 100u128;
                    ensure!(is_ratio_quotient_valid, Error::<T>::MakerPartialFillTooLow);
                }
                OrderSide::Ask => {
                    T::AssetManager::ensure_can_withdraw(base_asset, &taker, maker_fill)?;

                    // Note that this always rounds down, i.e. the taker will always get a little bit less than what they asked for.
                    // This ensures that the reserve of the maker is always enough to repatriate successfully!
                    let ratio = maker_fill.bdiv_floor(order_data.base_asset_amount)?;
                    let taker_fill = ratio.bmul_floor(order_data.outcome_asset_amount)?;

                    T::AssetManager::repatriate_reserved_named(
                        &Self::reserve_id(),
                        order_data.outcome_asset,
                        &maker,
                        &taker,
                        taker_fill,
                        BalanceStatus::Free,
                    )?;

                    T::AssetManager::transfer(base_asset, &taker, &maker, maker_fill)?;

                    order_data.outcome_asset_amount =
                        order_data.outcome_asset_amount.checked_sub_res(&taker_fill)?;
                    order_data.base_asset_amount =
                        order_data.base_asset_amount.checked_sub_res(&maker_fill)?;
                    // this ensures that partial fills, which fill nearly the whole order, are not executed
                    // this protects the last fill happening without a division by zero for `Perquintill::from_rational`
                    let is_ratio_quotient_valid = order_data.base_asset_amount.is_zero()
                        || order_data.base_asset_amount.saturated_into::<u128>() >= 100u128;
                    ensure!(is_ratio_quotient_valid, Error::<T>::MakerPartialFillTooLow);
                }
            };

            let unfilled_outcome_asset_amount = order_data.outcome_asset_amount;
            let unfilled_base_asset_amount = order_data.base_asset_amount;
            let total_unfilled =
                unfilled_outcome_asset_amount.saturating_add(unfilled_base_asset_amount);

            if total_unfilled.is_zero() {
                <Orders<T>>::remove(order_id);
            } else {
                <Orders<T>>::insert(order_id, order_data.clone());
            }

            Self::deposit_event(Event::OrderFilled {
                order_id,
                maker,
                taker: taker.clone(),
                filled: maker_fill,
                unfilled_outcome_asset_amount,
                unfilled_base_asset_amount,
            });

            match order_data.side {
                OrderSide::Bid => Ok(Some(T::WeightInfo::fill_order_bid()).into()),
                OrderSide::Ask => Ok(Some(T::WeightInfo::fill_order_ask()).into()),
            }
        }

        /// Place a new order.
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(2)]
        #[pallet::weight(
            T::WeightInfo::place_order_ask().max(T::WeightInfo::place_order_bid())
        )]
        #[transactional]
        pub fn place_order(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome_asset: Asset<MarketIdOf<T>>,
            side: OrderSide,
            #[pallet::compact] outcome_asset_amount: BalanceOf<T>,
            #[pallet::compact] base_asset_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketIsNotActive);
            ensure!(market.scoring_rule == ScoringRule::Orderbook, Error::<T>::InvalidScoringRule);
            let market_assets = Self::outcome_assets(market_id, &market);
            ensure!(
                market_assets.binary_search(&outcome_asset).is_ok(),
                Error::<T>::InvalidOutcomeAsset
            );
            let base_asset = market.base_asset;

            let order_id = <NextOrderId<T>>::get();
            let next_order_id = order_id.checked_add_res(&1)?;

            let order = Order {
                market_id,
                side: side.clone(),
                maker: who.clone(),
                outcome_asset,
                base_asset,
                outcome_asset_amount,
                base_asset_amount,
            };

            match side {
                OrderSide::Bid => {
                    T::AssetManager::reserve_named(
                        &Self::reserve_id(),
                        base_asset,
                        &who,
                        base_asset_amount,
                    )?;
                }
                OrderSide::Ask => {
                    T::AssetManager::reserve_named(
                        &Self::reserve_id(),
                        outcome_asset,
                        &who,
                        outcome_asset_amount,
                    )?;
                }
            }

            <Orders<T>>::insert(order_id, order.clone());
            <NextOrderId<T>>::put(next_order_id);
            Self::deposit_event(Event::OrderPlaced { order_id, order });

            match side {
                OrderSide::Bid => Ok(Some(T::WeightInfo::place_order_bid()).into()),
                OrderSide::Ask => Ok(Some(T::WeightInfo::place_order_ask()).into()),
            }
        }
    }

    impl<T: Config> Pallet<T> {
        /// The reserve ID of the orderbook pallet.
        #[inline]
        pub fn reserve_id() -> [u8; 8] {
            T::PalletId::get().0
        }

        pub fn outcome_assets(
            market_id: MarketIdOf<T>,
            market: &MarketOf<T>,
        ) -> Vec<Asset<MarketIdOf<T>>> {
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
    }
}
