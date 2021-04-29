use alloc::vec::Vec;
use orml_tokens::{AccountData, Accounts, TotalIssuance};
use orml_traits::currency::MultiReservableCurrency;

/// Custom `MultiReservableCurrency` trait.
pub trait ZeitgeistMultiReservableCurrency<AccountId>: MultiReservableCurrency<AccountId> {
    /// Returns the total number of accounts (1st value) and all users that hold a given `currency_id` (2nd value)
    fn accounts_by_currency_id(
        currency_id: Self::CurrencyId,
    ) -> (u32, Vec<(AccountId, AccountData<Self::Balance>)>);

    /// Destroys all assets of a given `currency_id`
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
    ) -> (u32, Vec<(T::AccountId, AccountData<T::Balance>)>) {
        let mut total: u32 = 0;
        (
            total,
            <Accounts<T>>::iter()
                .filter_map(|(k0, k1, v)| {
                    if total < u32::MAX {
                        total += 1;
                    }

                    if k1 == currency_id {
                        Some((k0, v))
                    } else {
                        None
                    }
                })
                .collect(),
        )
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
