use alloc::vec::Vec;
use frame_support::{storage::IterableStorageDoubleMap, StorageDoubleMap, StorageMap};
use orml_tokens::{AccountData, Accounts, TotalIssuance};
use orml_traits::currency::MultiReservableCurrency;
use sp_runtime::traits::Zero;

/// Custom `MultiReservableCurrency` trait.
pub trait ZeitgeistMultiReservableCurrency<AccountId>: MultiReservableCurrency<AccountId> {
    /// Returns all users that holds a given `currency_id`.
    fn accounts_by_currency_id(
        currency_id: Self::CurrencyId,
    ) -> Vec<(AccountId, AccountData<Self::Balance>)>;

    /// Destroys all assets of a given `currency_id`
    fn destroy_all<I>(currency_id: Self::CurrencyId, accounts: I)
    where
        I: Iterator<Item = (AccountId, AccountData<Self::Balance>)>;
}

impl<T> ZeitgeistMultiReservableCurrency<T::AccountId> for orml_tokens::Module<T>
where
    T: orml_tokens::Trait,
{
    fn accounts_by_currency_id(
        currency_id: Self::CurrencyId,
    ) -> Vec<(T::AccountId, AccountData<T::Balance>)> {
        <Accounts<T>>::iter()
            .filter_map(|(k0, k1, v)| {
                if k1 == currency_id {
                    Some((k0, v))
                } else {
                    None
                }
            })
            .collect()
    }

    fn destroy_all<I>(currency_id: Self::CurrencyId, accounts: I)
    where
        I: Iterator<Item = (T::AccountId, AccountData<Self::Balance>)>,
    {
        let mut total = Self::Balance::zero();
        for (k0, v) in accounts {
            <Accounts<T>>::remove(k0, currency_id);
            total += v.free + v.reserved;
        }
        <TotalIssuance<T>>::mutate(currency_id, |v| *v -= total);
    }
}
