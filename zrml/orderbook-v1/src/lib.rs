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
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{CheckedMul, CheckedSub},
    ArithmeticError, DispatchError, RuntimeDebug,
};
use zeitgeist_primitives::types::Asset;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod mock;
#[cfg(test)]
mod tests;
pub mod weights;

#[frame_support::pallet]
mod pallet {
    use crate::{weights::*, Order, OrderSide};
    use core::{cmp, marker::PhantomData};
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        ensure,
        pallet_prelude::{ConstU32, StorageMap, StorageValue, ValueQuery},
        traits::{
            Currency, ExistenceRequirement, Hooks, IsType, ReservableCurrency, StorageVersion,
            WithdrawReasons,
        },
        transactional, Blake2_128Concat, BoundedVec, Identity,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use orml_traits::{MultiCurrency, MultiReservableCurrency};
    use parity_scale_codec::Encode;
    use sp_runtime::{
        traits::{Hash, Zero},
        ArithmeticError, DispatchError,
    };
    use zeitgeist_primitives::{traits::MarketId, types::Asset};

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

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
                        let cost = order_data.cost()?;
                        T::Currency::unreserve(&maker, cost);
                        let mut bids = Self::bids(asset);
                        remove_item::<T::Hash, _>(&mut bids, order_hash);
                        <Bids<T>>::insert(asset, bids);
                    }
                    OrderSide::Ask => {
                        T::Shares::unreserve(order_data.asset, &maker, order_data.total);
                        let mut asks = Self::asks(asset);
                        remove_item::<T::Hash, _>(&mut asks, order_hash);
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

                let cost = order_data.cost()?;

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
        #[transactional]
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

            let cost = order.cost()?;

            match side {
                OrderSide::Bid => {
                    ensure!(
                        T::Currency::can_reserve(&sender, cost),
                        Error::<T>::InsufficientBalance,
                    );

                    <Bids<T>>::try_mutate(asset, |b: &mut BoundedVec<T::Hash, _>| {
                        b.try_push(hash).map_err(|_| <Error<T>>::StorageOverflow)
                    })?;

                    T::Currency::reserve(&sender, cost)?;
                }
                OrderSide::Ask => {
                    ensure!(
                        T::Shares::can_reserve(asset, &sender, amount),
                        Error::<T>::InsufficientBalance,
                    );

                    <Asks<T>>::try_mutate(asset, |a| {
                        a.try_push(hash).map_err(|_| <Error<T>>::StorageOverflow)
                    })?;

                    T::Shares::reserve(asset, &sender, amount)?;
                    bid = false;
                }
            }

            <OrderData<T>>::insert(hash, Some(order.clone()));
            <Nonce<T>>::try_mutate(|n| {
                *n = n.checked_add(1).ok_or(ArithmeticError::Overflow)?;
                Ok::<_, DispatchError>(())
            })?;
            Self::deposit_event(Event::OrderMade(sender, hash, order));

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

        type MarketId: MarketId;

        type Shares: MultiReservableCurrency<
            Self::AccountId,
            Balance = BalanceOf<Self>,
            CurrencyId = Asset<Self::MarketId>,
        >;

        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Insufficient balance.
        InsufficientBalance,
        NotOrderCreator,
        /// The order was already taken.
        OrderAlreadyTaken,
        /// The order does not exist.
        OrderDoesNotExist,
        /// It was tried to append an item to storage beyond the boundaries.
        StorageOverflow,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// [taker, order_hash]
        OrderFilled(<T as frame_system::Config>::AccountId, <T as frame_system::Config>::Hash),
        /// [maker, order_hash, order_data]
        OrderMade(
            <T as frame_system::Config>::AccountId,
            <T as frame_system::Config>::Hash,
            Order<T::AccountId, BalanceOf<T>, T::MarketId>,
        ),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    #[pallet::getter(fn asks)]
    pub type Asks<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Asset<T::MarketId>,
        BoundedVec<T::Hash, ConstU32<1_048_576>>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn bids)]
    pub type Bids<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        Asset<T::MarketId>,
        BoundedVec<T::Hash, ConstU32<1_048_576>>,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn nonce)]
    pub type Nonce<T> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn order_data)]
    pub type OrderData<T: Config> = StorageMap<
        _,
        Identity,
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

    fn remove_item<I: cmp::PartialEq + Copy, G>(items: &mut BoundedVec<I, G>, item: I) {
        let pos = items.iter().position(|&i| i == item).unwrap();
        items.swap_remove(pos);
    }
}

#[derive(Clone, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Clone, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Order<AccountId, Balance, MarketId: MaxEncodedLen> {
    side: OrderSide,
    maker: AccountId,
    taker: Option<AccountId>,
    asset: Asset<MarketId>,
    total: Balance,
    price: Balance,
    filled: Balance,
}

impl<AccountId, Balance: CheckedSub + CheckedMul, MarketId> Order<AccountId, Balance, MarketId>
where
    Balance: CheckedSub + CheckedMul,
    MarketId: MaxEncodedLen,
{
    pub fn cost(&self) -> Result<Balance, DispatchError> {
        match self.total.checked_sub(&self.filled) {
            Some(subtotal) => match subtotal.checked_mul(&self.price) {
                Some(cost) => Ok(cost),
                _ => Err(DispatchError::Arithmetic(ArithmeticError::Overflow)),
            },
            _ => Err(DispatchError::Arithmetic(ArithmeticError::Overflow)),
        }
    }
}
