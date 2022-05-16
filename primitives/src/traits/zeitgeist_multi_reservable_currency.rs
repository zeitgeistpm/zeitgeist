use alloc::vec::Vec;
use orml_tokens::{AccountData, Accounts, TotalIssuance};
use orml_traits::currency::NamedMultiReservableCurrency;

/// Custom `MultiReservableCurrency` trait.
pub trait ZeitgeistMultiReservableCurrency<AccountId>:
    NamedMultiReservableCurrency<AccountId>
{
    /// Return the total number of accounts that hold _any_ asset (first value) and all accounts
    /// that hold assets of a given `currency_id` (second value).
    fn accounts_by_currency_id(
        currency_id: Self::CurrencyId,
    ) -> (usize, Vec<(AccountId, AccountData<Self::Balance>)>);

    /// Destroy all assets of a `currency_id` for the given `accounts`.
    fn destroy_all<I>(currency_id: Self::CurrencyId, accounts: I)
    where
        I: Iterator<Item = (AccountId, AccountData<Self::Balance>)>;
}

impl<T> ZeitgeistMultiReservableCurrency<T::AccountId> for orml_tokens::Pallet<T>
where
    T: orml_tokens::Config,
{
    fn accounts_by_currency_id(
        currency_id: Self::CurrencyId,
    ) -> (usize, Vec<(T::AccountId, AccountData<T::Balance>)>) {
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
        (total, accounts)
    }

    fn destroy_all<I>(currency_id: Self::CurrencyId, accounts: I)
    where
        I: Iterator<Item = (T::AccountId, AccountData<Self::Balance>)>,
    {
        for (k0, _) in accounts {
            <Accounts<T>>::remove(k0, currency_id);
        }
        <TotalIssuance<T>>::remove(currency_id);
    }
}

// This implementation will only affect the `MultiCurrency` part, i.e., it won't touch
// the native currency
// TODO what about this?
/*
impl<T> ZeitgeistMultiReservableCurrency<T::AccountId> for orml_currencies::Pallet<T>
where
    T: orml_currencies::Config,
    T::MultiCurrency: ZeitgeistMultiReservableCurrency<T::AccountId>,
{
    fn accounts_by_currency_id(
        currency_id: Self::CurrencyId,
    ) -> (usize, Vec<(T::AccountId, AccountData<Self::Balance>)>) {
        T::MultiCurrency::accounts_by_currency_id(currency_id)
    }

    fn destroy_all<I>(currency_id: Self::CurrencyId, accounts: I)
    where
        I: Iterator<Item = (T::AccountId, AccountData<Self::Balance>)>,
    {
        T::MultiCurrency::destroy_all(currency_id, accounts)
    }
}
*/
