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
    pallet_prelude::{DispatchResult, OptionQuery, StorageMap, StorageValue, ValueQuery},
    traits::{IsType, StorageVersion},
    transactional, PalletId, Twox64Concat,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use orml_traits::{BalanceStatus, MultiCurrency, NamedMultiReservableCurrency};
pub use pallet::*;
use sp_runtime::{
    traits::{CheckedSub, Get, Zero},
    ArithmeticError, Perquintill, SaturatedConversion, Saturating,
};
use zeitgeist_primitives::{
    traits::{DistributeFees, MarketCommonsPalletApi},
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
                CurrencyId = AssetOf<Self>,
                ReserveIdentifier = [u8; 8],
            >;

        /// The way how fees are taken from the market base asset.
        type ExternalFees: DistributeFees<
                Asset = AssetOf<Self>,
                AccountId = AccountIdOf<Self>,
                Balance = BalanceOf<Self>,
                MarketId = MarketIdOf<Self>,
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
    pub(crate) type AssetOf<T> = Asset<MarketIdOf<T>>;
    pub(crate) type MarketOf<T> = Market<
        AccountIdOf<T>,
        BalanceOf<T>,
        <T as frame_system::Config>::BlockNumber,
        MomentOf<T>,
        AssetOf<T>,
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
            unfilled_maker_amount: BalanceOf<T>,
            unfilled_taker_amount: BalanceOf<T>,
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
        /// The market base asset is not present.
        MarketBaseAssetNotPresent,
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

            T::AssetManager::unreserve_named(
                &Self::reserve_id(),
                order_data.maker_asset,
                maker,
                order_data.maker_amount,
            );

            <Orders<T>>::remove(order_id);

            Self::deposit_event(Event::OrderRemoved { order_id, maker: maker.clone() });

            Ok(Some(T::WeightInfo::remove_order_bid()).into())
        }

        /// Fill an existing order entirely (`maker_partial_fill` = None)
        /// or partially (`maker_partial_fill` = Some(partial_amount)).
        ///
        /// External fees are paid in the base asset.
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

            let maker_fill = maker_partial_fill.unwrap_or(order_data.taker_amount);
            ensure!(!maker_fill.is_zero(), Error::<T>::AmountIsZero);
            ensure!(maker_fill <= order_data.taker_amount, Error::<T>::AmountTooHighForOrder);

            // clone required because of mutable borrow of order_data below
            let maker = order_data.maker.clone();
            let maker_asset = order_data.maker_asset;
            let taker_asset = order_data.taker_asset;

            // the reserve of the maker should always be enough to repatriate successfully, e.g. taker gets a little bit less
            // it should always ensure that the maker's request (maker_fill) is fully filled
            let taker_fill = Self::get_taker_fill(&order_data, maker_fill);

            // if base asset: fund the full amount, but charge base asset fees from taker later
            T::AssetManager::repatriate_reserved_named(
                &Self::reserve_id(),
                maker_asset,
                &maker,
                &taker,
                taker_fill,
                BalanceStatus::Free,
            )?;

            // always charge fees from the base asset
            let maybe_adjusted_maker_fill = if maker_asset == base_asset {
                // charge fees from the taker,
                // who already got the full base asset amount from the maker above (repatriate)
                T::ExternalFees::distribute(
                    order_data.market_id,
                    base_asset,
                    &taker,
                    // taker_fill is the amount what the taker wants to have (base asset from maker)
                    taker_fill,
                );
                // maker_fill is the amount what the maker wants to have (outcome asset from taker)
                // the fees were already charged,
                // because the maker reserved an outcome asset amount minus fees
                maker_fill
            } else {
                // maker_asset is outcome asset here
                debug_assert!(taker_asset == base_asset);

                // charge fees from the taker,
                // who is responsible to pay the base asset minus fees to the maker.
                let external_fees = T::ExternalFees::distribute(
                    order_data.market_id,
                    base_asset,
                    // taker is the one, who pays the base asset to the maker
                    // and the maker gets outcome asset amount minus fees in return
                    &taker,
                    // maker_fill is the amount what the maker wants to have (base asset from taker)
                    maker_fill,
                );
                // base asset amount from taker minus fees to the maker (maker_fill)
                maker_fill.checked_sub(&external_fees).ok_or(ArithmeticError::Underflow)?
            };

            T::AssetManager::transfer(
                order_data.taker_asset,
                &taker,
                &maker,
                // fee was only charged if the taker spends base asset to the maker
                maybe_adjusted_maker_fill,
            )?;

            // the accounting system does not care about, whether fees were charged,
            // it just substracts the total maker_fill and taker_fill (including fees)
            Self::decrease_order_amounts(&mut order_data, maker_fill, taker_fill)?;
            Self::is_ratio_quotient_valid(&order_data)?;

            let unfilled_maker_amount = order_data.maker_amount;
            let unfilled_taker_amount = order_data.taker_amount;
            let total_unfilled = unfilled_maker_amount.saturating_add(unfilled_taker_amount);

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
                unfilled_maker_amount,
                unfilled_taker_amount,
            });

            Ok(Some(T::WeightInfo::fill_order_bid()).into())
        }

        // TODO benchmark is renewed with simplication to have only one benchmark path!
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
            maker_asset: AssetOf<T>,
            #[pallet::compact] maker_amount: BalanceOf<T>,
            taker_asset: AssetOf<T>,
            #[pallet::compact] taker_amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketIsNotActive);
            ensure!(market.scoring_rule == ScoringRule::Orderbook, Error::<T>::InvalidScoringRule);
            let market_assets = Self::outcome_assets(market_id, &market);
            let base_asset = market.base_asset;
            let outcome_asset = if maker_asset == base_asset {
                taker_asset
            } else {
                ensure!(taker_asset == base_asset, Error::<T>::MarketBaseAssetNotPresent);
                maker_asset
            };
            ensure!(
                market_assets.binary_search(&outcome_asset).is_ok(),
                Error::<T>::InvalidOutcomeAsset
            );

            let order_id = <NextOrderId<T>>::get();
            let next_order_id = order_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;

            // The taker gets the maker_asset minus fees, and the maker gets taker_asset minus fees.
            // The outcome asset fee is taken into account here, the base asset fee is paid in fill_order
            let (unfilled_taker_amount, filled_maker_amount) = if maker_asset == base_asset {
                debug_assert!(taker_asset == outcome_asset);
                let taker_amount_minus_fees =
                    taker_amount.saturating_sub(T::ExternalFees::get_fee(market_id, taker_amount));
                // As maker, request taker amount minus fee of outcome asset from the taker(s).
                (taker_amount_minus_fees, maker_amount)
            } else {
                debug_assert!(maker_asset == outcome_asset);
                debug_assert!(taker_asset == base_asset);
                let maker_amount_minus_fees =
                    maker_amount.saturating_sub(T::ExternalFees::get_fee(market_id, maker_amount));
                // As maker, request full amount from the taker(s),
                // so that the taker(s) pays the external fee in base asset.
                (taker_amount, maker_amount_minus_fees)
            };

            // either maker_asset is base asset,
            // then the external fees will be paid in base asset in the fill_order extrinsic
            // so give full maker amount
            // and the taker(s) will get base asset amount minus fees
            // or maker_asset is outcome asset,
            // then the taker will get outcome asset amount minus fees
            T::AssetManager::reserve_named(
                &Self::reserve_id(),
                maker_asset,
                &who,
                filled_maker_amount,
            )?;

            let order = Order {
                market_id,
                maker: who.clone(),
                maker_asset,
                maker_amount: filled_maker_amount,
                taker_asset,
                taker_amount: unfilled_taker_amount,
            };

            <Orders<T>>::insert(order_id, order.clone());
            <NextOrderId<T>>::put(next_order_id);
            Self::deposit_event(Event::OrderPlaced { order_id, order });

            Ok(Some(T::WeightInfo::place_order_bid()).into())
        }
    }

    impl<T: Config> Pallet<T> {
        /// The reserve ID of the orderbook pallet.
        #[inline]
        pub fn reserve_id() -> [u8; 8] {
            T::PalletId::get().0
        }

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

        /// Reduces the reserved maker and requested taker amount
        /// by the amount the maker and taker actually filled.
        fn decrease_order_amounts(
            order_data: &mut OrderOf<T>,
            maker_fill: BalanceOf<T>,
            taker_fill: BalanceOf<T>,
        ) -> DispatchResult {
            order_data.maker_amount = order_data
                .maker_amount
                .checked_sub(&taker_fill)
                .ok_or(ArithmeticError::Underflow)?;
            order_data.taker_amount = order_data
                .taker_amount
                .checked_sub(&maker_fill)
                .ok_or(ArithmeticError::Underflow)?;
            Ok(())
        }

        /// Calculates the amount that the taker is going to get from the maker's amount.
        fn get_taker_fill(order_data: &OrderOf<T>, maker_fill: BalanceOf<T>) -> BalanceOf<T> {
            // the maker_full_fill is the maximum amount of what the maker wants to have
            let maker_full_fill = order_data.taker_amount;
            // the taker_full_fill is the maximum amount of what the taker wants to have
            let taker_full_fill = order_data.maker_amount;
            // Note that this always rounds down, i.e. the taker will always get a little bit less than what they asked for.
            // This ensures that the reserve of the maker is always enough to repatriate successfully!
            let ratio = Perquintill::from_rational(
                maker_fill.saturated_into::<u128>(),
                // this is ensured to be never zero in `is_ratio_quotient_valid`
                maker_full_fill.saturated_into::<u128>(),
            );
            // returns the share (ratio) of what the taker gets from the maker's amount
            // respected the partial fill from the taker of what the maker wants to fill
            ratio
                .mul_floor(taker_full_fill.saturated_into::<u128>())
                .saturated_into::<BalanceOf<T>>()
        }

        fn is_ratio_quotient_valid(order_data: &OrderOf<T>) -> DispatchResult {
            let maker_full_fill = order_data.taker_amount;
            // this ensures that partial fills, which fill nearly the whole order, are not executed
            // this protects the last fill happening without a division by zero for `Perquintill::from_rational`
            let is_ratio_quotient_valid =
                maker_full_fill.is_zero() || maker_full_fill.saturated_into::<u128>() >= 100u128;
            ensure!(is_ratio_quotient_valid, Error::<T>::MakerPartialFillTooLow);
            Ok(())
        }
    }
}
