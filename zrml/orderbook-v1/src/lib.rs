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
use core::marker::PhantomData;
use frame_support::{
    dispatch::DispatchResultWithPostInfo,
    ensure,
    pallet_prelude::{OptionQuery, StorageMap, StorageValue, ValueQuery},
    traits::{Currency, IsType, StorageVersion},
    transactional, PalletId, Twox64Concat,
};
use frame_system::{ensure_signed, pallet_prelude::OriginFor};
use orml_traits::{BalanceStatus, MultiCurrency, NamedMultiReservableCurrency};
pub use pallet::*;
use parity_scale_codec::Encode;
use sp_runtime::{
    traits::{CheckedMul, CheckedSub, Get, Hash, Zero},
    ArithmeticError, DispatchError,
};
use zeitgeist_primitives::{
    traits::{MarketCommonsPalletApi, ZeitgeistAssetManager},
    types::{Asset, Market, MarketStatus, MarketType, ScalarPosition, ScoringRule},
};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod mock;
#[cfg(test)]
mod tests;
mod types;
pub mod weights;

#[frame_support::pallet]
mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Shares of outcome assets and native currency
        type AssetManager: ZeitgeistAssetManager<
                Self::AccountId,
                CurrencyId = Asset<MarketIdOf<Self>>,
                ReserveIdentifier = [u8; 8],
            >;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type MarketCommons: MarketCommonsPalletApi<AccountId = Self::AccountId, BlockNumber = Self::BlockNumber>;

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
    pub(crate) type HashOf<T> = <T as frame_system::Config>::Hash;
    pub(crate) type OrderOf<T> = Order<AccountIdOf<T>, BalanceOf<T>, MarketIdOf<T>>;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub(crate) type MarketCommonsBalanceOf<T> =
        <<<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency as Currency<
            AccountIdOf<T>,
        >>::Balance;
    pub(crate) type MarketOf<T> = Market<
        AccountIdOf<T>,
        MarketCommonsBalanceOf<T>,
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
    pub type Orders<T: Config> = StorageMap<_, Twox64Concat, T::Hash, OrderOf<T>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        OrderPartiallyFilled {
            order_hash: HashOf<T>,
            maker: AccountIdOf<T>,
            taker: AccountIdOf<T>,
            filled: BalanceOf<T>,
        },
        OrderFullyFilled {
            order_hash: HashOf<T>,
            maker: AccountIdOf<T>,
            taker: AccountIdOf<T>,
            filled: BalanceOf<T>,
        },
        OrderPlaced {
            order_hash: HashOf<T>,
            order_id: OrderId,
            maker: T::AccountId,
            order: OrderOf<T>,
        },
        OrderCancelled {
            order_hash: HashOf<T>,
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
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(
            T::WeightInfo::cancel_order_ask().max(T::WeightInfo::cancel_order_bid())
        )]
        #[transactional]
        pub fn cancel_order(
            origin: OriginFor<T>,
            order_hash: T::Hash,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let order_data = <Orders<T>>::get(order_hash).ok_or(Error::<T>::OrderDoesNotExist)?;

            let maker = &order_data.maker;
            ensure!(sender == *maker, Error::<T>::NotOrderCreator);

            match order_data.side {
                OrderSide::Bid => {
                    let cost = order_data
                        .amount
                        .checked_mul(&order_data.price)
                        .ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;
                    T::AssetManager::unreserve_named(
                        &Self::reserve_id(),
                        order_data.base_asset,
                        maker,
                        cost,
                    );
                }
                OrderSide::Ask => {
                    T::AssetManager::unreserve_named(
                        &Self::reserve_id(),
                        order_data.outcome_asset,
                        maker,
                        order_data.amount,
                    );
                }
            }

            <Orders<T>>::remove(order_hash);

            Self::deposit_event(Event::OrderCancelled { order_hash, maker: maker.clone() });

            match order_data.side {
                OrderSide::Bid => Ok(Some(T::WeightInfo::cancel_order_bid()).into()),
                OrderSide::Ask => Ok(Some(T::WeightInfo::cancel_order_ask()).into()),
            }
        }

        #[pallet::call_index(1)]
        #[pallet::weight(
            T::WeightInfo::fill_order_ask().max(T::WeightInfo::fill_order_bid())
        )]
        #[transactional]
        pub fn fill_order(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            order_hash: T::Hash,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(!amount.is_zero(), Error::<T>::AmountIsZero);

            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketIsNotActive);
            let base_asset = market.base_asset;

            let mut order_data =
                <Orders<T>>::get(order_hash).ok_or(Error::<T>::OrderDoesNotExist)?;
            ensure!(amount <= order_data.amount, Error::<T>::AmountTooHighForOrder);
            let cost = amount
                .checked_mul(&order_data.price)
                .ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;
            order_data.amount = order_data
                .amount
                .checked_sub(&amount)
                .ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))?;
            let maker = order_data.maker.clone();

            match order_data.side {
                OrderSide::Bid => {
                    T::AssetManager::ensure_can_withdraw(order_data.outcome_asset, &who, amount)?;

                    T::AssetManager::repatriate_reserved_named(
                        &Self::reserve_id(),
                        base_asset,
                        &maker,
                        &who,
                        cost,
                        BalanceStatus::Free,
                    )?;

                    T::AssetManager::transfer(order_data.outcome_asset, &who, &maker, amount)?;
                }
                OrderSide::Ask => {
                    T::AssetManager::ensure_can_withdraw(base_asset, &who, cost)?;

                    T::AssetManager::repatriate_reserved_named(
                        &Self::reserve_id(),
                        order_data.outcome_asset,
                        &maker,
                        &who,
                        order_data.amount,
                        BalanceStatus::Free,
                    )?;

                    T::AssetManager::transfer(base_asset, &who, &maker, cost)?;
                }
            }

            if !order_data.amount.is_zero() {
                <Orders<T>>::insert(order_hash, order_data.clone());
                Self::deposit_event(Event::OrderPartiallyFilled {
                    order_hash,
                    maker,
                    taker: who.clone(),
                    filled: amount,
                });
            } else {
                <Orders<T>>::remove(order_hash);
                Self::deposit_event(Event::OrderFullyFilled {
                    order_hash,
                    maker: maker.clone(),
                    taker: who.clone(),
                    filled: amount,
                });
            }

            match order_data.side {
                OrderSide::Bid => Ok(Some(T::WeightInfo::fill_order_bid()).into()),
                OrderSide::Ask => Ok(Some(T::WeightInfo::fill_order_ask()).into()),
            }
        }

        #[pallet::call_index(2)]
        #[pallet::weight(
            T::WeightInfo::make_order_ask().max(T::WeightInfo::make_order_bid())
        )]
        #[transactional]
        pub fn place_order(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome_asset: Asset<MarketIdOf<T>>,
            side: OrderSide,
            #[pallet::compact] amount: BalanceOf<T>,
            #[pallet::compact] price: BalanceOf<T>,
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
            let next_order_id = order_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;

            let order_hash = Self::order_hash(&who, market_id, order_id);
            let order = Order {
                market_id,
                side: side.clone(),
                maker: who.clone(),
                outcome_asset,
                base_asset,
                amount,
                price,
            };

            match side {
                OrderSide::Bid => {
                    let cost = amount
                        .checked_mul(&price)
                        .ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))?;

                    T::AssetManager::reserve_named(&Self::reserve_id(), base_asset, &who, cost)?;
                }
                OrderSide::Ask => {
                    T::AssetManager::reserve_named(
                        &Self::reserve_id(),
                        outcome_asset,
                        &who,
                        amount,
                    )?;
                }
            }

            <Orders<T>>::insert(order_hash, order.clone());
            <NextOrderId<T>>::put(next_order_id);
            Self::deposit_event(Event::OrderPlaced {
                order_hash,
                order_id,
                maker: who.clone(),
                order,
            });

            match side {
                OrderSide::Bid => Ok(Some(T::WeightInfo::make_order_bid()).into()),
                OrderSide::Ask => Ok(Some(T::WeightInfo::make_order_ask()).into()),
            }
        }
    }

    impl<T: Config> Pallet<T> {
        /// The reserve ID of the orderbook pallet.
        #[inline]
        pub fn reserve_id() -> [u8; 8] {
            T::PalletId::get().0
        }

        pub fn order_hash(
            creator: &T::AccountId,
            market_id: MarketIdOf<T>,
            order_id: OrderId,
        ) -> T::Hash {
            (&creator, market_id, order_id).using_encoded(T::Hashing::hash)
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
