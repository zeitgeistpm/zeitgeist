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
use zrml_traits::shares::Shares;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

}

decl_storage! {
    trait Store for Module<T: Trait> as Orderbook {
        /// Stores the order hash mapping to the amount of the taker asset already bought by maker.
        Filled get(fn filled): map hasher(blake2_128_concat) T::Hash => u128;
        Cancelled get(fn cancelled): map hasher(blake2_128_concat) T::Hash => bool;
    }
}

decl_event! {
    pub enum Event<T> {

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
        fn fill_order(
            origin,
            order: Order,
            taker_asset_fill_amount: u128,
            signature: [u8;64],
        ) {

        }

        #[weight = 0]
        fn fill_or_kill_order() {

        }

        #[weight = 0]
        fn match_orders() {

        }

        #[weight = 0]
        fn cancel_order() {

        }
    }
}

impl<T: Trait> Module<T> {

}
