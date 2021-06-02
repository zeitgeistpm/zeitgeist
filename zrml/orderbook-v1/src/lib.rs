//! # Orderbook
//!
//! A module to trade shares using a naive on-chain orderbook.
//!
//! ## Overview
//!
//! TODO
//!
//! ## Interface
//!
//! ### Dispatches
//!

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{
    traits::{CheckedMul, CheckedSub},
    RuntimeDebug,
};
use zeitgeist_primitives::types::Asset;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub mod weights;

#[frame_support::pallet]
mod pallet {
    use crate::{weights::*, Order, OrderSide};
    use alloc::vec::Vec;
    use core::{cmp, marker::PhantomData};
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        ensure,
        pallet_prelude::{StorageMap, StorageValue, ValueQuery},
        traits::{
            Currency, ExistenceRequirement, Hooks, IsType, ReservableCurrency, WithdrawReasons,
        },
        Blake2_128Concat, Parameter,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use orml_traits::{MultiCurrency, MultiReservableCurrency};
    use parity_scale_codec::Encode;
    use sp_runtime::traits::{AtLeast32Bit, Hash, MaybeSerializeDeserialize, Member, Zero};
    use zeitgeist_primitives::types::Asset;

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(
            T::WeightInfo::cancel_order_ask().max(T::WeightInfo::cancel_order_bid())
        )]
        pub fn cancel_order(
            origin: OriginFor<T>,
            asset: Asset<T::MarketId>,
            order_hash: T::Hash,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let mut bid = true;

            if let Some(order_data) = Self::order_data(order_hash) {
                let maker = order_data.maker.clone();
                ensure!(sender == maker, Error::<T>::NotOrderCreator);

                match order_data.side {
                    OrderSide::Bid => {
                        T::Currency::unreserve(&maker, order_data.cost());
                        let mut bids = Self::bids(asset);
                        remove_item::<T::Hash>(&mut bids, order_hash);
                        <Bids<T>>::insert(asset, bids);
                    }
                    OrderSide::Ask => {
                        T::Shares::unreserve(order_data.asset, &maker, order_data.total);
                        let mut asks = Self::asks(asset);
                        remove_item::<T::Hash>(&mut asks, order_hash);
                        <Asks<T>>::insert(asset, asks);
                        bid = false;
                    }
                }

                <OrderData<T>>::remove(order_hash);
            } else {
                return Err(Error::<T>::OrderDoesNotExist.into());
            }

            if bid {
                Ok(Some(T::WeightInfo::cancel_order_bid()).into())
            } else {
                Ok(Some(T::WeightInfo::cancel_order_ask()).into())
            }
        }

        #[pallet::weight(
            T::WeightInfo::fill_order_ask().max(T::WeightInfo::fill_order_bid())
        )]
        pub fn fill_order(origin: OriginFor<T>, order_hash: T::Hash) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let mut bid = true;

            if let Some(order_data) = Self::order_data(order_hash) {
                ensure!(order_data.taker.is_none(), Error::<T>::OrderAlreadyTaken);

                let cost = order_data.cost();
                let maker = order_data.maker;

                match order_data.side {
                    OrderSide::Bid => {
                        T::Shares::ensure_can_withdraw(
                            order_data.asset,
                            &sender,
                            order_data.total,
                        )?;

                        T::Currency::unreserve(&maker, cost);
                        T::Currency::transfer(
                            &maker,
                            &sender,
                            cost,
                            ExistenceRequirement::AllowDeath,
                        )?;

                        T::Shares::transfer(order_data.asset, &sender, &maker, order_data.total)?;
                    }
                    OrderSide::Ask => {
                        T::Currency::ensure_can_withdraw(
                            &sender,
                            cost,
                            WithdrawReasons::all(),
                            Zero::zero(),
                        )?;

                        T::Shares::unreserve(order_data.asset, &maker, order_data.total);
                        T::Shares::transfer(order_data.asset, &maker, &sender, order_data.total)?;

                        T::Currency::transfer(
                            &sender,
                            &maker,
                            cost,
                            ExistenceRequirement::AllowDeath,
                        )?;
                        bid = false;
                    }
                }

                Self::deposit_event(Event::OrderFilled(sender, order_hash));
            } else {
                return Err(Error::<T>::OrderDoesNotExist.into());
            }

            if bid {
                Ok(Some(T::WeightInfo::fill_order_bid()).into())
            } else {
                Ok(Some(T::WeightInfo::fill_order_ask()).into())
            }
        }

        #[pallet::weight(
            T::WeightInfo::make_order_ask().max(T::WeightInfo::make_order_bid())
        )]
        pub fn make_order(
            origin: OriginFor<T>,
            asset: Asset<T::MarketId>,
            side: OrderSide,
            amount: BalanceOf<T>,
            price: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            // Only store nonce in memory for now.
            let nonce = <Nonce<T>>::get();
            let hash = Self::order_hash(&sender, asset, nonce);
            let mut bid = true;

            // Love the smell of fresh orders in the morning.
            let order = Order {
                side: side.clone(),
                maker: sender.clone(),
                taker: None,
                asset,
                total: amount,
                price,
                filled: Zero::zero(),
            };

            let cost = order.cost();

            match side {
                OrderSide::Bid => {
                    ensure!(
                        T::Currency::can_reserve(&sender, cost),
                        Error::<T>::InsufficientBalance,
                    );

                    <Bids<T>>::mutate(asset, |b: &mut Vec<T::Hash>| {
                        b.push(hash);
                    });

                    T::Currency::reserve(&sender, cost)?;
                }
                OrderSide::Ask => {
                    ensure!(
                        T::Shares::can_reserve(asset, &sender, amount),
                        Error::<T>::InsufficientBalance,
                    );

                    <Asks<T>>::mutate(asset, |a| {
                        a.push(hash);
                    });

                    T::Shares::reserve(asset, &sender, amount)?;
                    bid = false;
                }
            }

            <OrderData<T>>::insert(hash, Some(order));
            <Nonce<T>>::mutate(|n| *n += 1);
            Self::deposit_event(Event::OrderMade(sender, hash));

            if bid {
                Ok(Some(T::WeightInfo::make_order_bid()).into())
            } else {
                Ok(Some(T::WeightInfo::make_order_ask()).into())
            }
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currency: ReservableCurrency<Self::AccountId>;

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type MarketId: AtLeast32Bit
            + Copy
            + Default
            + MaybeSerializeDeserialize
            + Member
            + Parameter;

        type Shares: MultiReservableCurrency<
            Self::AccountId,
            Balance = BalanceOf<Self>,
            CurrencyId = Asset<Self::MarketId>,
        >;

        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
        InsufficientBalance,
        NotOrderCreator,
        OrderAlreadyTaken,
        OrderDoesNotExist,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// [taker, order_hash]
        OrderFilled(
            <T as frame_system::Config>::AccountId,
            <T as frame_system::Config>::Hash,
        ),
        /// [maker, order_hash]
        OrderMade(
            <T as frame_system::Config>::AccountId,
            <T as frame_system::Config>::Hash,
        ),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    #[pallet::getter(fn asks)]
    pub type Asks<T: Config> =
        StorageMap<_, Blake2_128Concat, Asset<T::MarketId>, Vec<T::Hash>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn bids)]
    pub type Bids<T: Config> =
        StorageMap<_, Blake2_128Concat, Asset<T::MarketId>, Vec<T::Hash>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn nonce)]
    pub type Nonce<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn order_data)]
    pub type OrderData<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        Option<Order<T::AccountId, BalanceOf<T>, T::MarketId>>,
        ValueQuery,
    >;

    impl<T: Config> Pallet<T> {
        pub fn order_hash(
            creator: &T::AccountId,
            asset: Asset<T::MarketId>,
            nonce: u64,
        ) -> T::Hash {
            (&creator, asset, nonce).using_encoded(T::Hashing::hash)
        }
    }

    fn remove_item<I: cmp::PartialEq + Copy>(items: &mut Vec<I>, item: I) {
        let pos = items.iter().position(|&i| i == item).unwrap();
        items.swap_remove(pos);
    }
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
pub struct Order<AccountId, Balance, MarketId> {
    side: OrderSide,
    maker: AccountId,
    taker: Option<AccountId>,
    asset: Asset<MarketId>,
    total: Balance,
    price: Balance,
    filled: Balance,
}

impl<AccountId, Balance: CheckedSub + CheckedMul, MarketId> Order<AccountId, Balance, MarketId> {
    pub fn cost(&self) -> Balance {
        self.total
            .checked_sub(&self.filled)
            .unwrap()
            .checked_mul(&self.price)
            .unwrap()
    }
}
