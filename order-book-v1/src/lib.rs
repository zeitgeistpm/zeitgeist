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
    Member, One, Hash,
};
use xrml_traits::shares::Shares;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

#[derive(Clone, Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Order<AccountId, Balance> {
    side: OrderSide,
    maker: AccountId,
    taker: Option<AccountId>,
    total: Balance,
    price: Balance,
    filled: Balance,
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: ReservableCurrency<Self::AccountId>;

    type Shares: Shares<Self::AccountId, BalanceOf<T>, Self::Hash>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Orderbook {
        /// Stores the order hash mapping to the amount of the taker asset already bought by maker.
        // Filled get(fn filled): map hasher(blake2_128_concat) T::Hash => u128;
        // Cancelled get(fn cancelled): map hasher(blake2_128_concat) T::Hash => bool;

        Bids get(fn bids): map hasher(blake2_128_concat) T::Hash => Vec<T::Hash>;
        Asks get(fn asks): map hasher(blake2_128_concat) T::Hash => Vec<T::Hash>;

        OrderData get(fn order_data): map hasher(blake2_128_concat) T::Hash => 
            Option<Order<T::AccountId, BalanceOf<T>>>;

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
        One,
        Two,
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
                total: amount,
                price,
                filled: 0,
            };

            let cost = amount * price;

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
        fn take_order(
            origin,
        ) {

        }

        #[weight = 0]
        fn cancel_order(origin) {

        }
    }
}

impl<T: Trait> Module<T> {

}
