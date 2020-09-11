//! # Bonding Curves
//!
//! TODO

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    ensure,
    traits::Get,
};
use frame_system::ensure_signed;
use orml_traits::{
    MultiCurrency, MultiReservableCurrency,
};
use sp_runtime::ModuleId;
use sp_runtime::traits::{AccountIdConversion, SaturatedConversion};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[derive(Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BondingCurve<AccountId, CurrencyId> {
    // The module-owned account for this bonding curve.
    // account: AccountId,
    /// The creator of the bonding curve.
    creator: AccountId,
    /// The currency id of the bonding curve token.
    currency_id: CurrencyId,
    /// The exponent of the curve.
    exponent: u32,
    /// The slope of the curve.
    slope: u128,
    /// The maximum supply that can be minted from the curve.
    max_supply: u128,
}

impl<
    AccountId,
    CurrencyId,
> BondingCurve<AccountId, CurrencyId> {
    /// Integral when the curve is at point `x`.
    pub fn integral(&self, x: u128) -> u128 {
        let nexp = self.exponent + 1;
        x.pow(nexp) * self.slope / nexp as u128
    }
}

type BalanceOf<T> = <<T as Trait>::Currency as MultiCurrency<<T as frame_system::Trait>::AccountId>>::Balance;
type CurrencyIdOf<T> =
    <<T as Trait>::Currency as MultiCurrency<<T as frame_system::Trait>::AccountId>>::CurrencyId;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: MultiReservableCurrency<Self::AccountId>;

	/// The native currency.
	type GetNativeCurrencyId: Get<CurrencyIdOf<Self>>;

    /// The deposit required for creating a new bonding curve.
    type CurveDeposit: Get<BalanceOf<Self>>;

    /// The module identifier.
    type ModuleId: Get<ModuleId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as BondingCurve {
        Curves get(fn curves):
            map hasher(blake2_128_concat) u64 => Option<BondingCurve<T::AccountId, CurrencyIdOf<T>>>;

        NextCurveId: u64;
	}
}

decl_event!(
    pub enum Event<T> where 
        AccountId = <T as frame_system::Trait>::AccountId,
        BalanceOf = BalanceOf<T>
    {
        /// (CurveId, Creator)
        NewCurve(u64, AccountId),
        /// (Buyer, CurveId, Amount, Cost)
        CurveBuy(AccountId, u64, BalanceOf, BalanceOf),
        /// (Seller, CurveId, Amount, Return)
        CurveSell(AccountId, u64, BalanceOf, BalanceOf),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
        /// Sender does not have enough base currency to reserve for a new curve.
        InsufficientBalanceToReserve,
        /// A curve does not exist for this curve id.
        CurveDoesNotExist,
        /// Sender does not have enough base currency to make a purchase.
        InsufficentBalanceForPurchase,
        /// The currency that is trying to be created already exists.
        CurrencyAlreadyExists,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

        fn deposit_event() = default;
        
        const ModuleId: ModuleId = T::ModuleId::get();

        /// Native currency
		const GetNativeCurrencyId: CurrencyIdOf<T> = T::GetNativeCurrencyId::get();

        /// Creates a new bonding curve.
        ///
        #[weight = 0]
        pub fn create(origin, currency_id: CurrencyIdOf<T>, exponent: u32, slope: u128, max_supply: u128) {
            let sender = ensure_signed(origin)?;

            // Requires an amount to be reserved.
            ensure!(
                T::Currency::can_reserve(T::GetNativeCurrencyId::get(), &sender, T::CurveDeposit::get()),
                Error::<T>::InsufficientBalanceToReserve,
            );

            // Ensure that a curve with this id does not already exist.
            ensure!(
                T::Currency::total_issuance(currency_id) == 0.into(),
                Error::<T>::CurrencyAlreadyExists,
            );

            // Adds 1 of the token to the module account.
            T::Currency::deposit(currency_id, &Self::module_account(), 1.saturated_into())?;

            let new_curve = BondingCurve {
                creator: sender.clone(),
                currency_id,
                exponent,
                slope,
                max_supply,
            };

            // Mutations start here
            let curve_id = Self::next_id();
            <Curves<T>>::insert(curve_id, new_curve);

            Self::deposit_event(RawEvent::NewCurve(curve_id, sender));
        }

        /// Buys from a bonding curve.
        ///
        #[weight = 0]
        pub fn buy(origin, curve_id: u64, amount: BalanceOf<T>) {
            let sender = ensure_signed(origin)?;

            if let Some(curve) = Self::curves(curve_id) {
                let currency_id = curve.currency_id;
                let total_issuance = T::Currency::total_issuance(currency_id).saturated_into::<u128>();
                let issuance_after = total_issuance + amount.saturated_into::<u128>();

                ensure!(
                    issuance_after <= curve.max_supply,
                    "Exceeded max supply.",
                );

                let integral_after: BalanceOf<T> = curve.integral(issuance_after).saturated_into();
                
                let cost = integral_after - integral_before;
                ensure!(
                    T::Currency::free_balance(T::GetNativeCurrencyId::get(), &sender) >= cost.into(),
                    Error::<T>::InsufficentBalanceForPurchase,
                );

                let curve_account = Self::get_module_sub_account(curve_id);

                T::Currency::transfer(T::GetNativeCurrencyId::get(), &sender, &curve_account, cost).ok(); // <- Why does the `?` operator not work?

                T::Currency::deposit(currency_id, &sender, amount).ok();

                Self::deposit_event(RawEvent::CurveBuy(sender, curve_id, amount, cost));
            } else {
                Err(Error::<T>::CurveDoesNotExist)?;
            }
        }

        /// Sells into a bonding curve.
        ///
        #[weight = 0]
        pub fn sell(origin, curve_id: u64, amount: BalanceOf<T>) {
            let sender = ensure_signed(origin)?;

            if let Some(curve) = Self::curves(curve_id) {
                let currency_id = curve.currency_id;

                T::Currency::ensure_can_withdraw(currency_id, &sender, amount)?;
                
                let total_issuance = T::Currency::total_issuance(currency_id);
                let issuance_after = total_issuance - amount;

                let integral_before: BalanceOf<T> = curve.integral(total_issuance.saturated_into::<u128>()).saturated_into();
                let integral_after: BalanceOf<T> = curve.integral(issuance_after.saturated_into::<u128>()).saturated_into();

                let return_amount = integral_before - integral_after;

                let curve_account = Self::get_module_sub_account(curve_id);

                T::Currency::withdraw(currency_id, &sender, amount).ok();

                T::Currency::transfer(T::GetNativeCurrencyId::get(), &curve_account, &sender, return_amount).ok();

                Self::deposit_event(RawEvent::CurveSell(sender, curve_id, amount, return_amount));
            } else {
                Err(Error::<T>::CurveDoesNotExist)?;
            }
        }
	}
}

impl<T: Trait> Module<T> {
    fn module_account() -> T::AccountId {
        T::ModuleId::get().into_account()
    }

    fn get_module_sub_account(id: u64) -> T::AccountId {
        T::ModuleId::get().into_sub_account(id)
    }

    /// DANGER - Mutates storage
    fn next_id() -> u64 {
        let id = NextCurveId::get();
        NextCurveId::mutate(|n| *n += 1);
        id
    }
}
