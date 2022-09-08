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

mod mock;
mod tests;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use alloc::vec::Vec;
    use core::result;
    use frame_support::pallet_prelude::*;
    use orml_traits::{
        BalanceStatus, MultiCurrency, MultiReservableCurrency, NamedMultiReservableCurrency,
    };
    use sp_runtime::DispatchResult;
    use zeitgeist_primitives::traits::ZeitgeistAssetManager;

    pub(crate) type BalanceOf<T> = <<T as Config>::Currencies as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    pub(crate) type CurrencyIdOf<T> = <<T as Config>::Currencies as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::CurrencyId;

    pub(crate) type ReserveIdentifierOf<T> =
        <<T as Config>::Currencies as NamedMultiReservableCurrency<
            <T as frame_system::Config>::AccountId,
        >>::ReserveIdentifier;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Currencies: ZeitgeistAssetManager<Self::AccountId>;

        #[pallet::constant]
        type GetNativeCurrencyId: Get<CurrencyIdOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Keep track of accounts which ever posessed an asset of given currency_id.
    #[pallet::storage]
    pub type Accounts<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        CurrencyIdOf<T>,
        Blake2_128Concat,
        T::AccountId,
        (),
        OptionQuery,
    >;

    impl<T: Config> NamedMultiReservableCurrency<T::AccountId> for Pallet<T> {
        type ReserveIdentifier = ReserveIdentifierOf<T>;
        fn slash_reserved_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> Self::Balance {
            T::Currencies::slash_reserved_named(id, currency_id, who, value)
        }

        fn reserved_balance_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
        ) -> Self::Balance {
            T::Currencies::reserved_balance_named(id, currency_id, who)
        }

        fn reserve_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> DispatchResult {
            T::Currencies::reserve_named(id, currency_id, who, value)
        }

        fn unreserve_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> Self::Balance {
            T::Currencies::unreserve_named(id, currency_id, who, value)
        }

        fn repatriate_reserved_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            slashed: &T::AccountId,
            beneficiary: &T::AccountId,
            value: Self::Balance,
            status: BalanceStatus,
        ) -> result::Result<Self::Balance, DispatchError> {
            T::Currencies::repatriate_reserved_named(
                id,
                currency_id,
                slashed,
                beneficiary,
                value,
                status,
            )
        }
    }

    impl<T: Config> MultiReservableCurrency<T::AccountId> for Pallet<T> {
        fn can_reserve(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> bool {
            T::Currencies::can_reserve(currency_id, who, value)
        }
        fn slash_reserved(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> Self::Balance {
            T::Currencies::slash_reserved(currency_id, who, value)
        }
        fn reserved_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
            T::Currencies::reserved_balance(currency_id, who)
        }
        fn reserve(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> DispatchResult {
            T::Currencies::reserve(currency_id, who, value)
        }

        fn unreserve(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> Self::Balance {
            T::Currencies::unreserve(currency_id, who, value)
        }

        fn repatriate_reserved(
            currency_id: Self::CurrencyId,
            slashed: &T::AccountId,
            beneficiary: &T::AccountId,
            value: Self::Balance,
            status: BalanceStatus,
        ) -> result::Result<Self::Balance, DispatchError> {
            T::Currencies::repatriate_reserved(currency_id, slashed, beneficiary, value, status)
        }
    }

    impl<T: Config> MultiCurrency<T::AccountId> for Pallet<T> {
        type CurrencyId = CurrencyIdOf<T>;
        type Balance = BalanceOf<T>;
        fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
            T::Currencies::minimum_balance(currency_id)
        }
        fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
            T::Currencies::total_issuance(currency_id)
        }
        fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
            T::Currencies::total_balance(currency_id, who)
        }
        fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
            T::Currencies::free_balance(currency_id, who)
        }
        fn ensure_can_withdraw(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> DispatchResult {
            T::Currencies::ensure_can_withdraw(currency_id, who, amount)
        }
        fn transfer(
            currency_id: Self::CurrencyId,
            from: &T::AccountId,
            to: &T::AccountId,
            amount: Self::Balance,
        ) -> DispatchResult {
            if T::GetNativeCurrencyId::get() != currency_id {
                Accounts::<T>::insert(currency_id, to, ());
            }
            T::Currencies::transfer(currency_id, from, to, amount)
        }
        fn deposit(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> DispatchResult {
            if T::GetNativeCurrencyId::get() != currency_id {
                Accounts::<T>::insert(currency_id, who, ());
            }
            T::Currencies::deposit(currency_id, who, amount)
        }
        fn withdraw(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> DispatchResult {
            T::Currencies::withdraw(currency_id, who, amount)
        }

        fn can_slash(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> bool {
            T::Currencies::can_slash(currency_id, who, amount)
        }
        fn slash(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> Self::Balance {
            T::Currencies::slash(currency_id, who, amount)
        }
    }

    impl<T: Config> ZeitgeistAssetManager<T::AccountId> for Pallet<T> {
        fn accounts_by_currency_id(
            currency_id: Self::CurrencyId,
        ) -> Result<(usize, Vec<T::AccountId>), DispatchError> {
            let accounts: Vec<T::AccountId> =
                Accounts::<T>::iter_prefix(currency_id).map(|(k2, _v)| k2).collect();
            Ok((accounts.len(), accounts))
        }

        fn destroy_all(
            currency_id: Self::CurrencyId, /*_accounts: I*/
        ) -> Result<usize, DispatchError>
// where
        //     I: Iterator<Item = T::AccountId>,
        {
            // Accounts::<T>::remove_prefix(currency_id, None);
            // T::Currencies::destroy_all(currency_id, accounts)
            let mut accounts = 0_usize;
            for (account, _) in Accounts::<T>::drain_prefix(currency_id) {
                accounts += 1;
                T::Currencies::remove(currency_id, account)?;
            }
            T::Currencies::remove_total_issuance(currency_id)?;
            Ok(accounts)
        }

        fn remove(
            currency_id: Self::CurrencyId,
            account: T::AccountId,
        ) -> Result<(), DispatchError> {
            Accounts::<T>::remove(currency_id, account);
            Ok(())
        }

        fn remove_total_issuance(currency_id: Self::CurrencyId) -> Result<(), DispatchError> {
            T::Currencies::remove_total_issuance(currency_id)
        }
    }
}
