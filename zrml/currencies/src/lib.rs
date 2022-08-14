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

//! # Currencies
//!
//! A module for provide currency_id to account_id(s) mapping.
//!
//! ## Overview
//!
//! The pallet is wrapper over orml_currencies to provide fast access by keeping a map of currency_id to account_id(s).
//! It implements same traits as implemented by orml_currencies, prediction-market pallet uses this
//! pallet instead of orml_currencies. All calls are redirected to orml_currencies after updating
//! the above mapping.
//! 
//!
//! ## Interface
//!
//! ### Dispatches
//!
//! #### Public Dispatches
//!
//! #### Admin Dispatches
//!

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod benchmarks;
mod mock;
mod tests;
pub mod weights;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Currency};
    use frame_system::pallet_prelude::*;
    use sp_runtime::SaturatedConversion;
    use zeitgeist_primitives::types::Balance;
    use orml_traits::MultiReservableCurrency;

    use crate::weights::WeightInfoZeitgeist;

    #[pallet::config]
    pub trait Config: frame_system::Config {

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type WeightInfo: WeightInfoZeitgeist;

        type Currencies: MultiReservableCurrency<<Self as frame_system::Config>::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    // /// Keep track of crossings. Accounts are only able to cross once.
    // #[pallet::storage]
    // pub type Crossings<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>;

    // #[pallet::type_value]
    // pub fn DefaultBurnAmount<T: Config>() -> Balance {
    //     (zeitgeist_primitives::constants::BASE * 200).saturated_into()
    // }

    // /// An extra layer of pseudo randomness.
    // #[pallet::storage]
    // pub type BurnAmount<T: Config> = StorageValue<_, Balance, ValueQuery, DefaultBurnAmount<T>>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
    }

    #[pallet::error]
    pub enum Error<T> {
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
    }
}
