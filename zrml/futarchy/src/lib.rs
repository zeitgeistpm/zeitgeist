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

pub mod mock;
mod tests;
mod traits;
pub mod types;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use core::marker::PhantomData;
    use frame_support::{
        pallet_prelude::{EnsureOrigin, IsType, StorageVersion},
        require_transactional,
        traits::{QueryPreimage, StorePreimage},
        transactional,
    };
    use frame_system::pallet_prelude::OriginFor;
    use orml_traits::MultiCurrency;
    use sp_runtime::DispatchResult;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type MultiCurrency: MultiCurrency<Self::AccountId>;

        // Preimage interface for acquiring call data.
        type Preimages: QueryPreimage + StorePreimage;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type SubmitOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        // // TODO
        // // The origin from which proposals may be whitelisted.
        // type WhitelistOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        // TODO Scheduler, EnactmentPeriod
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;

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
        #[transactional]
        #[pallet::weight({0})]
        pub fn submit(origin: OriginFor<T>) -> DispatchResult {
            T::SubmitOrigin::ensure_origin(origin)?;
            Self::do_submit()
        }
    }

    impl<T: Config> Pallet<T> {
        #[require_transactional]
        fn do_submit() -> DispatchResult {
            Ok(())
        }
    }
}
