// Copyright 2022-2025 Forecasting Technologies LTD.
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

use crate::weights::*;
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use frame_support::{
    ensure,
    pallet_prelude::{
        DispatchError, DispatchResult, OptionQuery, StorageMap, StorageValue, ValueQuery,
    },
    traits::{IsType, StorageVersion},
    transactional, PalletId, Twox64Concat,
};
use frame_system::{
    ensure_signed,
    pallet_prelude::{BlockNumberFor, OriginFor},
};
use orml_traits::{BalanceStatus, MultiCurrency, NamedMultiReservableCurrency};
pub use pallet::*;
use sp_runtime::traits::{Get, Zero};
use zeitgeist_primitives::{
    hybrid_router_api_types::{ApiError, ExternalFee, OrderbookSoftFail, OrderbookTrade},
    math::{
        checked_ops_res::{CheckedAddRes, CheckedSubRes},
        fixed::FixedMulDiv,
    },
    orderbook::{Order, OrderId},
    traits::{DistributeFees, HybridRouterOrderbookApi, MarketCommonsPalletApi},
    types::{Asset, Market, MarketStatus, MarketType, ScalarPosition, ScoringRule},
};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod migrations;
pub mod mock;
#[cfg(test)]
mod tests;
mod utils;
pub mod weights;

#[frame_support::pallet]
mod pallet {
    use super::*;

    #[allow(dead_code)]
    const LOG_TARGET: &str = "runtime::zrml-orderbook";

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
            BlockNumber = BlockNumberFor<Self>,
            Balance = BalanceOf<Self>,
        >;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        type WeightInfo: WeightInfoZeitgeist;
    }

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type AssetOf<T> = Asset<MarketIdOf<T>>;
    pub(crate) type BalanceOf<T> = <<T as Config>::AssetManager as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    pub(crate) type ExternalFeeOf<T> = ExternalFee<AccountIdOf<T>, BalanceOf<T>>;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MarketOf<T> =
        Market<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, MomentOf<T>, MarketIdOf<T>>;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub(crate) type OrderOf<T> = Order<AccountIdOf<T>, BalanceOf<T>, MarketIdOf<T>>;
    pub(crate) type OrderbookTradeOf<T> = OrderbookTrade<AccountIdOf<T>, BalanceOf<T>>;

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
            filled_maker_amount: BalanceOf<T>,
            filled_taker_amount: BalanceOf<T>,
            unfilled_maker_amount: BalanceOf<T>,
            unfilled_taker_amount: BalanceOf<T>,
            external_fee: ExternalFeeOf<T>,
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
        /// The scoring rule is not order book.
        InvalidScoringRule,
        /// The specified amount parameter is too high for the order.
        AmountTooHighForOrder,
        /// The specified outcome asset is not part of the market.
        InvalidOutcomeAsset,
        /// The maker partial fill leads to a too low quotient for the next order execution.
        PartialFillNearFullFillNotAllowed,
        /// The market base asset is not present.
        MarketBaseAssetNotPresent,
        /// The specified amount is below the minimum balance.
        BelowMinimumBalance,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Removes an order.
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::remove_order())]
        #[transactional]
        pub fn remove_order(origin: OriginFor<T>, order_id: OrderId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::do_remove_order(order_id, who)?;

            Ok(())
        }

        /// Fill an existing order entirely (`maker_partial_fill` = None)
        /// or partially (`maker_partial_fill` = Some(partial_amount)).
        ///
        /// External fees are paid in the base asset.
        ///
        /// NOTE: The `maker_partial_fill` is the partial amount
        /// of what the maker wants to get filled.
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::fill_order())]
        #[transactional]
        pub fn fill_order(
            origin: OriginFor<T>,
            order_id: OrderId,
            maker_partial_fill: Option<BalanceOf<T>>,
        ) -> DispatchResult {
            let taker = ensure_signed(origin)?;

            let _ = Self::do_fill_order(order_id, taker, maker_partial_fill)?;

            Ok(())
        }

        /// Place a new order.
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::place_order())]
        #[transactional]
        pub fn place_order(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            maker_asset: AssetOf<T>,
            #[pallet::compact] maker_amount: BalanceOf<T>,
            taker_asset: AssetOf<T>,
            #[pallet::compact] taker_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Self::do_place_order(
                who,
                market_id,
                maker_asset,
                maker_amount,
                taker_asset,
                taker_amount,
            )?;

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// The reserve ID of the order book pallet.
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
            order_data.maker_amount = order_data.maker_amount.checked_sub_res(&taker_fill)?;
            order_data.taker_amount = order_data.taker_amount.checked_sub_res(&maker_fill)?;
            Ok(())
        }

        /// Calculates the amount that the taker is going to get from the maker's amount.
        fn get_taker_fill(
            order_data: &OrderOf<T>,
            maker_fill: BalanceOf<T>,
        ) -> Result<BalanceOf<T>, DispatchError> {
            // the maker_full_fill is the maximum amount of what the maker wants to have
            let maker_full_fill = order_data.taker_amount;
            // the taker_full_fill is the maximum amount of what the taker wants to have
            let taker_full_fill = order_data.maker_amount;
            // rounding down: the taker will always get a little bit less than what they asked for.
            // This ensures that the reserve of the maker
            // is always enough to repatriate successfully!
            // `maker_full_fill` is ensured to be never zero in `ensure_ratio_quotient_valid`
            maker_fill.bmul_bdiv_floor(taker_full_fill, maker_full_fill)
        }

        fn ensure_ratio_quotient_valid(order_data: &OrderOf<T>) -> DispatchResult {
            let maker_full_fill = order_data.taker_amount;
            // this ensures that partial fills, which fill nearly the whole order, are not executed
            // this protects the last fill happening
            // without a division by zero for `Perquintill::from_rational`
            let is_ratio_quotient_valid = maker_full_fill.is_zero()
                || maker_full_fill >= T::AssetManager::minimum_balance(order_data.taker_asset);
            ensure!(is_ratio_quotient_valid, Error::<T>::PartialFillNearFullFillNotAllowed);
            Ok(())
        }

        fn do_remove_order(order_id: OrderId, who: AccountIdOf<T>) -> DispatchResult {
            let order_data = <Orders<T>>::get(order_id).ok_or(Error::<T>::OrderDoesNotExist)?;

            let maker = &order_data.maker;
            ensure!(who == *maker, Error::<T>::NotOrderCreator);

            let missing = T::AssetManager::unreserve_named(
                &Self::reserve_id(),
                order_data.maker_asset,
                maker,
                order_data.maker_amount,
            );

            debug_assert!(
                missing.is_zero(),
                "Could not unreserve all of the amount. reserve_id: {:?}, asset: {:?} who: {:?}, \
                 amount: {:?}, missing: {:?}",
                Self::reserve_id(),
                order_data.maker_asset,
                maker,
                order_data.maker_amount,
                missing,
            );

            <Orders<T>>::remove(order_id);

            Self::deposit_event(Event::OrderRemoved { order_id, maker: maker.clone() });

            Ok(())
        }

        /// Charge the external fees in base asset and return the adjusted maker fill.
        ///
        /// `maker_fill` is the amount that the maker wants to have.
        /// `taker_fill` is the amount that the taker wants to have.
        /// It does not charge fees from the outcome asset.
        ///
        /// Returns the adjusted maker fill and the external fee.
        fn charge_external_fees(
            order_data: &OrderOf<T>,
            base_asset: AssetOf<T>,
            maker_fill: BalanceOf<T>,
            taker: &AccountIdOf<T>,
            taker_fill: BalanceOf<T>,
        ) -> Result<(BalanceOf<T>, ExternalFeeOf<T>), DispatchError> {
            let maker_asset_is_base = order_data.maker_asset == base_asset;
            let base_asset_fill = if maker_asset_is_base {
                taker_fill
            } else {
                debug_assert!(order_data.taker_asset == base_asset);
                maker_fill
            };
            let fee_amount = T::ExternalFees::distribute(
                order_data.market_id,
                base_asset,
                taker,
                base_asset_fill,
            );
            if maker_asset_is_base {
                Ok((maker_fill, ExternalFeeOf::<T> { account: taker.clone(), amount: fee_amount }))
            } else {
                Ok((
                    // maker gets less base asset, so the maker paid the fees
                    maker_fill.checked_sub_res(&fee_amount)?,
                    ExternalFeeOf::<T> { account: order_data.maker.clone(), amount: fee_amount },
                ))
            }
        }

        fn do_fill_order(
            order_id: OrderId,
            taker: AccountIdOf<T>,
            maker_partial_fill: Option<BalanceOf<T>>,
        ) -> Result<OrderbookTradeOf<T>, DispatchError> {
            let mut order_data = <Orders<T>>::get(order_id).ok_or(Error::<T>::OrderDoesNotExist)?;
            let market = T::MarketCommons::market(&order_data.market_id)?;
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketIsNotActive);
            let base_asset = market.base_asset;

            let maker_fill = maker_partial_fill.unwrap_or(order_data.taker_amount);
            ensure!(
                maker_fill >= T::AssetManager::minimum_balance(order_data.taker_asset),
                Error::<T>::BelowMinimumBalance
            );
            ensure!(maker_fill <= order_data.taker_amount, Error::<T>::AmountTooHighForOrder);

            let maker = order_data.maker.clone();
            let maker_asset = order_data.maker_asset;
            let taker_asset = order_data.taker_asset;

            let taker_fill = Self::get_taker_fill(&order_data, maker_fill)?;

            // if base asset: fund the full amount, but charge base asset fees from taker later
            T::AssetManager::repatriate_reserved_named(
                &Self::reserve_id(),
                maker_asset,
                &maker,
                &taker,
                taker_fill,
                BalanceStatus::Free,
            )?;

            // always charge fees from the base asset and not the outcome asset
            let (maybe_adjusted_maker_fill, external_fee) = Self::charge_external_fees(
                &order_data,
                base_asset,
                maker_fill,
                &taker,
                taker_fill,
            )?;

            T::AssetManager::transfer(
                taker_asset,
                &taker,
                &maker,
                // fee was only charged if the taker spends base asset to the maker
                maybe_adjusted_maker_fill,
            )?;

            // the accounting system does not care about, whether fees were charged,
            // it just substracts the total maker_fill and taker_fill (including fees)
            Self::decrease_order_amounts(&mut order_data, maker_fill, taker_fill)?;
            Self::ensure_ratio_quotient_valid(&order_data)?;

            if order_data.maker_amount.is_zero() {
                <Orders<T>>::remove(order_id);
            } else {
                <Orders<T>>::insert(order_id, order_data.clone());
            }

            Self::deposit_event(Event::OrderFilled {
                order_id,
                maker,
                taker: taker.clone(),
                filled_maker_amount: taker_fill,
                filled_taker_amount: maker_fill,
                unfilled_maker_amount: order_data.maker_amount,
                unfilled_taker_amount: order_data.taker_amount,
                external_fee: external_fee.clone(),
            });

            Ok(OrderbookTrade {
                filled_maker_amount: taker_fill,
                filled_taker_amount: maker_fill,
                external_fee,
            })
        }

        fn do_place_order(
            who: AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            maker_asset: AssetOf<T>,
            maker_amount: BalanceOf<T>,
            taker_asset: AssetOf<T>,
            taker_amount: BalanceOf<T>,
        ) -> DispatchResult {
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketIsNotActive);
            ensure!(
                market.scoring_rule == ScoringRule::AmmCdaHybrid,
                Error::<T>::InvalidScoringRule
            );

            let base_asset = market.base_asset;
            let outcome_asset = if maker_asset == base_asset {
                taker_asset
            } else {
                ensure!(taker_asset == base_asset, Error::<T>::MarketBaseAssetNotPresent);
                maker_asset
            };
            let market_assets = market.outcome_assets();
            market_assets
                .binary_search(&outcome_asset)
                .map_err(|_| Error::<T>::InvalidOutcomeAsset)?;

            ensure!(
                maker_amount >= T::AssetManager::minimum_balance(maker_asset),
                Error::<T>::BelowMinimumBalance
            );
            ensure!(
                taker_amount >= T::AssetManager::minimum_balance(taker_asset),
                Error::<T>::BelowMinimumBalance
            );

            let order_id = <NextOrderId<T>>::get();
            let next_order_id = order_id.checked_add_res(&1)?;

            // fees are always only charged in the base asset in fill_order
            T::AssetManager::reserve_named(&Self::reserve_id(), maker_asset, &who, maker_amount)?;

            let order = Order {
                market_id,
                maker: who,
                maker_asset,
                maker_amount,
                taker_asset,
                taker_amount,
            };

            <Orders<T>>::insert(order_id, order.clone());
            <NextOrderId<T>>::put(next_order_id);
            Self::deposit_event(Event::OrderPlaced { order_id, order });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn match_failure(error: DispatchError) -> ApiError<OrderbookSoftFail> {
            let below_minimum_balance: DispatchError = Error::<T>::BelowMinimumBalance.into();
            let partial_fill_near_full_fill_not_allowed: DispatchError =
                Error::<T>::PartialFillNearFullFillNotAllowed.into();
            if error == below_minimum_balance {
                ApiError::SoftFailure(OrderbookSoftFail::BelowMinimumBalance)
            } else if error == partial_fill_near_full_fill_not_allowed {
                ApiError::SoftFailure(OrderbookSoftFail::PartialFillNearFullFillNotAllowed)
            } else {
                ApiError::HardFailure(error)
            }
        }
    }

    impl<T: Config> HybridRouterOrderbookApi for Pallet<T> {
        type AccountId = AccountIdOf<T>;
        type MarketId = MarketIdOf<T>;
        type Balance = BalanceOf<T>;
        type Asset = AssetOf<T>;
        type Order = OrderOf<T>;
        type OrderId = OrderId;

        fn order(order_id: Self::OrderId) -> Result<Self::Order, DispatchError> {
            <Orders<T>>::get(order_id).ok_or(Error::<T>::OrderDoesNotExist.into())
        }

        fn fill_order(
            who: Self::AccountId,
            order_id: Self::OrderId,
            maker_partial_fill: Option<Self::Balance>,
        ) -> Result<OrderbookTradeOf<T>, ApiError<OrderbookSoftFail>> {
            Self::do_fill_order(order_id, who, maker_partial_fill).map_err(Self::match_failure)
        }

        fn place_order(
            who: Self::AccountId,
            market_id: Self::MarketId,
            maker_asset: Self::Asset,
            maker_amount: Self::Balance,
            taker_asset: Self::Asset,
            taker_amount: Self::Balance,
        ) -> Result<(), ApiError<OrderbookSoftFail>> {
            Self::do_place_order(
                who,
                market_id,
                maker_asset,
                maker_amount,
                taker_asset,
                taker_amount,
            )
            .map_err(Self::match_failure)
        }
    }
}
