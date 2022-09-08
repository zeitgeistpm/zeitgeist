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

#![allow(clippy::type_complexity)]
use alloc::vec::Vec;
use frame_support::traits::Get;
use orml_tokens::{Accounts, TotalIssuance};
use orml_traits::currency::NamedMultiReservableCurrency;
use sp_runtime::DispatchError;

/// Custom `NamedMultiReservableCurrency` trait.
pub trait ZeitgeistAssetManager<AccountId>: NamedMultiReservableCurrency<AccountId> {
    /// Return the total number of accounts that hold _any_ asset (first value) and all accounts
    /// that hold assets of a given `currency_id` (second value).
    /// If the `currency_id` is the native currency, then return None.
    fn accounts_by_currency_id(
        currency_id: Self::CurrencyId,
    ) -> Result<(usize, Vec<AccountId>), DispatchError>;

    /// Destroy all assets of a `currency_id` for the given `accounts`.
    /// If the `currency_id` is the native currency, then return false.
    fn destroy_all(currency_id: Self::CurrencyId) -> Result<usize, DispatchError>;

    /// Remove all assets of a `currency_id` for the given `account`.
    fn remove(currency_id: Self::CurrencyId, account: AccountId) -> Result<(), DispatchError>;

    /// Remove total issuance for given `currency_id`.
    fn remove_total_issuance(currency_id: Self::CurrencyId) -> Result<(), DispatchError>;
}

impl<T> ZeitgeistAssetManager<T::AccountId> for orml_tokens::Pallet<T>
where
    T: orml_tokens::Config,
{
    fn accounts_by_currency_id(
        currency_id: Self::CurrencyId,
    ) -> Result<(usize, Vec<T::AccountId>), DispatchError> {
        let mut total = 0;
        #[allow(
            // Iterator will never yield more than `usize::MAX` elements
            clippy::integer_arithmetic
        )]
        let accounts = <Accounts<T>>::iter()
            .filter_map(|(k0, k1, _v)| {
                total += 1;
                if k1 == currency_id { Some(k0) } else { None }
            })
            .collect();
        Ok((total, accounts))
    }

    fn destroy_all(currency_id: Self::CurrencyId) -> Result<usize, DispatchError> {
        let mut number_of_accounts = 0_usize;
        let (_total_accounts, accounts) = Self::accounts_by_currency_id(currency_id)?;
        for k0 in accounts {
            number_of_accounts += 1;
            <Accounts<T>>::remove(k0, currency_id);
        }
        <TotalIssuance<T>>::remove(currency_id);
        Ok(number_of_accounts)
    }

    fn remove(currency_id: Self::CurrencyId, account: T::AccountId) -> Result<(), DispatchError> {
        <Accounts<T>>::remove(account, currency_id);
        Ok(())
    }
    fn remove_total_issuance(currency_id: Self::CurrencyId) -> Result<(), DispatchError> {
        <TotalIssuance<T>>::remove(currency_id);
        Ok(())
    }
}

// This implementation will only affect the `MultiCurrency` part, i.e., it won't touch
// the native currency
impl<T> ZeitgeistAssetManager<T::AccountId> for orml_currencies::Pallet<T>
where
    T: orml_currencies::Config,
    T::MultiCurrency: ZeitgeistAssetManager<T::AccountId>,
{
    fn accounts_by_currency_id(
        currency_id: Self::CurrencyId,
    ) -> Result<(usize, Vec<T::AccountId>), DispatchError> {
        if currency_id == T::GetNativeCurrencyId::get() {
            Err(DispatchError::Other("NotForNativeCurrency"))
        } else {
            T::MultiCurrency::accounts_by_currency_id(currency_id)
        }
    }

    fn destroy_all(currency_id: Self::CurrencyId) -> Result<usize, DispatchError> {
        if currency_id == T::GetNativeCurrencyId::get() {
            Err(DispatchError::Other("NotForNativeCurrency"))
        } else {
            T::MultiCurrency::destroy_all(currency_id)
        }
    }

    fn remove(currency_id: Self::CurrencyId, account: T::AccountId) -> Result<(), DispatchError> {
        if currency_id == T::GetNativeCurrencyId::get() {
            Err(DispatchError::Other("NotForNativeCurrency"))
        } else {
            T::MultiCurrency::remove(currency_id, account)
        }
    }

    fn remove_total_issuance(currency_id: Self::CurrencyId) -> Result<(), DispatchError> {
        if currency_id == T::GetNativeCurrencyId::get() {
            Err(DispatchError::Other("NotForNativeCurrency"))
        } else {
            T::MultiCurrency::remove_total_issuance(currency_id)
        }
    }
}
