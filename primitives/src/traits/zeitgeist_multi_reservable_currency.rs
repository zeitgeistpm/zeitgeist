#![allow(clippy::type_complexity)]
use alloc::vec::Vec;
use frame_support::traits::Get;
use orml_tokens::{AccountData, Accounts, TotalIssuance};
use orml_traits::currency::NamedMultiReservableCurrency;
use sp_runtime::DispatchError;

/// Custom `NamedMultiReservableCurrency` trait.
pub trait ZeitgeistAssetManager<AccountId>: NamedMultiReservableCurrency<AccountId> {
    /// Return the total number of accounts that hold _any_ asset (first value) and all accounts
    /// that hold assets of a given `currency_id` (second value).
    /// If the `currency_id` is the native currency, then return None.
    fn accounts_by_currency_id(
        currency_id: Self::CurrencyId,
    ) -> Result<(usize, Vec<(AccountId, AccountData<Self::Balance>)>), DispatchError>;

    /// Destroy all assets of a `currency_id` for the given `accounts`.
    /// If the `currency_id` is the native currency, then return false.
    fn destroy_all<I>(currency_id: Self::CurrencyId, accounts: I) -> Result<(), DispatchError>
    where
        I: Iterator<Item = (AccountId, AccountData<Self::Balance>)>;
}

impl<T> ZeitgeistAssetManager<T::AccountId> for orml_tokens::Pallet<T>
where
    T: orml_tokens::Config,
{
    fn accounts_by_currency_id(
        currency_id: Self::CurrencyId,
    ) -> Result<(usize, Vec<(T::AccountId, AccountData<Self::Balance>)>), DispatchError> {
        let mut total = 0;
        #[allow(
            // Iterator will never yield more than `usize::MAX` elements
            clippy::integer_arithmetic
        )]
        let accounts = <Accounts<T>>::iter()
            .filter_map(|(k0, k1, v)| {
                total += 1;
                if k1 == currency_id { Some((k0, v)) } else { None }
            })
            .collect();
        Ok((total, accounts))
    }

    fn destroy_all<I>(currency_id: Self::CurrencyId, accounts: I) -> Result<(), DispatchError>
    where
        I: Iterator<Item = (T::AccountId, AccountData<Self::Balance>)>,
    {
        for (k0, _) in accounts {
            <Accounts<T>>::remove(k0, currency_id);
        }
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
    ) -> Result<(usize, Vec<(T::AccountId, AccountData<Self::Balance>)>), DispatchError> {
        if currency_id == T::GetNativeCurrencyId::get() {
            Err(DispatchError::Other("NotForNativeCurrency"))
        } else {
            T::MultiCurrency::accounts_by_currency_id(currency_id)
        }
    }

    fn destroy_all<I>(currency_id: Self::CurrencyId, accounts: I) -> Result<(), DispatchError>
    where
        I: Iterator<Item = (T::AccountId, AccountData<Self::Balance>)>,
    {
        if currency_id == T::GetNativeCurrencyId::get() {
            Err(DispatchError::Other("NotForNativeCurrency"))
        } else {
            T::MultiCurrency::destroy_all(currency_id, accounts)
        }
    }
}
