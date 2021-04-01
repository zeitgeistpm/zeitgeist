use crate::ZeitgeistMultiReservableCurrency;
use frame_support::dispatch::DispatchResult;
use orml_traits::{BasicCurrency, MultiCurrency};

type CurrencyIdOf<T> = <<T as orml_currencies::Trait>::MultiCurrency as MultiCurrency<
    <T as frame_system::Trait>::AccountId,
>>::CurrencyId;

pub trait ZeitgeistCurrenciesExtension: orml_currencies::Trait
where
    Self::MultiCurrency: ZeitgeistMultiReservableCurrency<Self::AccountId>,
{
    fn destroy_all(currency_id: CurrencyIdOf<Self>) -> DispatchResult {
        let accounts = Self::MultiCurrency::accounts_by_currency_id(currency_id);
        // Destroy pool
        Self::MultiCurrency::destroy_all(currency_id, accounts.iter().cloned());
        // Give back to accounts
        for (k0, v) in accounts {
            Self::NativeCurrency::deposit(&k0, v.free + v.reserved)?;
        }
        Ok(())
    }
}
