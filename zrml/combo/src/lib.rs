// Copyright 2024 Forecasting Technologies LTD.
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

mod traits;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use core::marker::PhantomData;
    use frame_support::{
        pallet_prelude::{IsType, StorageVersion},
        require_transactional, transactional,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::DispatchResult;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    // TODO Types
    pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    // TODO Storage Items

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config, {}

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(0)] // TODO
        #[transactional]
        pub fn split_position(origin: OriginFor<T>) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            Self::do_split_position()
        }

        #[pallet::call_index(1)]
        #[pallet::weight(0)] // TODO
        #[transactional]
        pub fn merge_position(origin: OriginFor<T>) -> DispatchResult {
            let _ = ensure_signed(origin)?;
            Self::do_merge_position()
        }
    }

    impl<T: Config> Pallet<T> {
        #[require_transactional]
        fn do_split_position() -> DispatchResult {
            Ok(())
        }

        #[require_transactional]
        fn do_merge_position() -> DispatchResult {
            Ok(())
        }
    }
}
