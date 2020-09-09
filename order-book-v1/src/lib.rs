//! # Orderbook
//!
//! A module to trade shares using a 0x-style off-chain orderbook.
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

use codec::Encode;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, 
    ensure, Parameter,
};
use frame_support::traits::{
    Currency, ReservableCurrency, Get, ExistenceRequirement,
    EnsureOrigin,
};
use frame_support::weights::Weight;
use frame_system::{self as system, ensure_signed};
use sp_runtime::ModuleId;
use sp_runtime::traits::{
    AccountIdConversion, AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, 
    Member, One, Hash, Zero,
};
use xrml_traits::shares::{ReservableShares, Shares};

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

fn remove_item<I>(items: &mut Vec<I>, item: I) {
    let pos = items.iter().position(|&i| i == item).unwrap();
    items.swap_remove(pos);
}

#[derive(Clone, Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Order<AccountId, Balance, Hash> {
    side: OrderSide,
    maker: AccountId,
    taker: Option<AccountId>,
    share_id: Hash,
    total: Balance,
    price: Balance,
    filled: Balance,
}

impl<
    AccountId,
    Balance,
    Hash,
> Order<AccountId, Balance, Hash> {
    pub fn cost(&self) -> Balance {
        self.total
            .checked_sub(self.filled)
            .checked_mul(self.price)
    }
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: ReservableCurrency<Self::AccountId>;

    type Shares: Shares<Self::AccountId, BalanceOf<T>, Self::Hash> + ReservableShares<Self::AccountId, BalanceOf<T>, Self::Hash>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Orderbook {
        /// Stores the order hash mapping to the amount of the taker asset already bought by maker.
        // Filled get(fn filled): map hasher(blake2_128_concat) T::Hash => u128;
        // Cancelled get(fn cancelled): map hasher(blake2_128_concat) T::Hash => bool;

        Bids get(fn bids): map hasher(blake2_128_concat) T::Hash => Vec<T::Hash>;
        Asks get(fn asks): map hasher(blake2_128_concat) T::Hash => Vec<T::Hash>;

        OrderData get(fn order_data): map hasher(blake2_128_concat) T::Hash => 
            Option<Order<T::AccountId, BalanceOf<T>, T::Hash>>;

        Nonce get(fn nonce): u64;

    }
}

decl_event! {
    pub enum Event<T> {
        /// [maker, order_hash]
        OrderMade(AccountId, Hash),
        /// [taker, order_hash]
        OrderTaken(Accountid, Hash),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        OrderDoesNotExist,
        OrderAlreadyTaken,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 0]
        fn make_order(
            origin,
            share_id: T::Hash,
            side: OrderSide,
            amount: BalanceOf<T>,
            price: BalanceOf<T>,
        ) {
            let sender = ensure_signed(origin)?;

            // Only store nonce in memory for now.
            let nonce = Nonce<T>::get();
            let hash = (sender.clone(), share_id, nonce)
                .using_encoded(<T as frame_system::Trait>::Hashing::hash);

            // Love the smell of fresh orders in the morning.
            let order = Order {
                side: side.clone(),
                maker: sender.clone(),
                taker: None,
                share_id,
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

                    <Bids<T>>::mutate(share_id, |b| {
                        b.as_mut().unwrap().push(hash);
                    });

                    T::Currency::reserve(&sender, cost)?;
                }
                OrderSide::Ask => {
                    ensure!(
                        T::Shares::can_reserve(share_id, &sender, amount),
                        Error::<T>::InsufficientBalance,
                    );

                    <Asks<T>>::mutate(share_id, |a| {
                        a.as_mut().unwrap().push(hash);
                    });

                    T::Shares::reserve(share_id, &sender, amount)?;
                }

                <OrderData<T>>::insert(hash, order);
                Self::deposit_event(RawEvent::OrderMade(sender, hash))
            }

        }

        #[weoght = 0]
        fn take_order(origin, order_hash: T::Hash) {
            let sender = ensure_signed(origin)?;

            if let Some(order_data) = Self::order_data(order_hash) {
                ensure!(order_data.taker.is_none(), Error::<T>::OrderAlreadyTaken);

                match order_data.side {
                    let cost = order.cost();
                    let share_id = order_data.share_id;
                    let maker = order_data.maker;

                    OrderSide::Bid => {
                        T::Shares::ensure_can_withdraw(share_id, &sender, order_data.total)?;

                        T::Currency::unreserve(&maker, cost);
                        T::Currency::transfer(&maker, &sender, cost, ExistenceRequirement::AllowDeath)?;

                        T::Shares::transfer(share_id, &sender, &maker, order_data.total)?;

                        // Self::deposit_event(RawEvent::OrderFilled());
                    }
                    OrderSide::Ask => {
                        T::Currency::ensure_can_withdraw(&sender, cost)?;

                        T::Shares::unreserve(share_id, &maker, order_data.total)?;
                        T::Shares::transfer(share_id, &maker, &sender, order_data.total)?;

                        T::Currency::transfer(&sender, &maker, cost, ExistenceRequirement::AllowDeath)?;

                    }
                }

            } else {
                Err(Error::<T>::OrderDoesNotExist)?;
            }
        }

        #[weight = 0]
        fn cancel_order(origin, share_id: T::Hash, order_hash: T::Hash) {
            let sender = ensure_signed(origin)?;

            if let Some(order_data) = Self::order_data(order_hash) {
                ensure!(
                    sender == order_data.maker,
                    "Canceller must be order maker."
                );

                match order_data.side {
                    OrderSide::Bid => {
                        let mut bids = Self::bids(share_id);
                        remove_item::<T::Hash>(&bids, order_hash);
                        <Bids<T>>::insert(share_id, bids);
                    }   
                    OrderSide::Ask => {
                        let mut asks = Self::asks(share_id);
                        remove_item::<T::Hash>(&asks, order_hash);
                        <Asks<T>>::insert(share_id, asks);
                    }
                }

                <OrderData<T>>::remove_item(order_hash);
            } else {
                Err(Error::<T>::OrderDoesNotExist)?;
            }
        }
    }
}

impl<T: Trait> Module<T> {
    
}
