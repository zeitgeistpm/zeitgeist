#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::{cmp, convert::TryInto};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, ensure,
    traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
};
use frame_system::ensure_signed;
use sp_runtime::{
    traits::{AccountIdConversion, CheckedAdd, CheckedSub, SaturatedConversion, StaticLookup},
    DispatchError, DispatchResult, FixedPointNumber, FixedU128, ModuleId, RuntimeDebug,
};
use zrml_traits::shares::{ReservableShares, Shares, WrapperShares};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct AccountShares<Balance> {
    pub free: Balance,
    pub reserved: Balance,
}

pub trait Trait: frame_system::Trait {
    type Currency: ReservableCurrency<Self::AccountId>;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type ModuleId: Get<ModuleId>;
}

decl_storage! {
    trait Store for Module<T: Trait> as Shares {
        /// A double map that is keyed by (share_id, account). The reason to make the `share_id` the prefix
        /// key is so that we can efficiently wipe out shares.
        pub Accounts get(fn accounts): double_map hasher (identity) T::Hash, hasher (blake2_128_concat) T::AccountId  => AccountShares<BalanceOf<T>>;

        pub TotalSupply get(fn total_supply): map hasher (identity) T::Hash => BalanceOf<T>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        Hash = <T as frame_system::Trait>::Hash,
    {
        /// Some shares have been transferred. [shares_id, from, to, amount]
        Transferred(Hash, AccountId, AccountId, FixedU128),
        /// Some shares have been reserved. [shares_id, who, amount]
        Reserved(Hash, AccountId, FixedU128),
        /// Shares have been unreserved. [shares_id, who, amount]
        Unreserved(Hash, AccountId, FixedU128),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        TotalIssuanceOverflow,
        BalanceTooLow,
        Underflow,
        Overflow,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        #[weight = 0]
        pub fn transfer(
            origin,
            dest: <T::Lookup as StaticLookup>::Source,
            share_id: T::Hash,
            #[compact] amount: BalanceOf<T>,
        ) {
            let fixed = Self::opaque_to_fixed(amount);
            let from = ensure_signed(origin)?;
            let to = T::Lookup::lookup(dest)?;
            <Self as Shares<T::AccountId, T::Hash>>::transfer(share_id, &from, &to, fixed)?;

            Self::deposit_event(RawEvent::Transferred(share_id, from, to, fixed));
        }

        /// Wraps the native currency into a "share" so that it can be used as if it was part of this
        /// pallet.
        #[weight = 0]
        pub fn wrap_native_currency(origin, amount: FixedU128) {
            let sender = ensure_signed(origin)?;

            Self::do_wrap_native_currency(sender, amount)?;
        }

        #[weight = 0]
        pub fn unwrap_native_currency(origin, amount: FixedU128) {
            let sender = ensure_signed(origin)?;

            Self::do_unwrap_native_currency(sender, amount)?;
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn set_balance(
        share_id: T::Hash,
        who: &T::AccountId,
        balance: FixedU128,
    ) -> DispatchResult {
        let opaque = Self::fixed_to_opaque(balance)?;
        <Accounts<T>>::mutate(share_id, who, |data| data.free = opaque);
        Ok(())
    }

    pub fn set_reserved(
        share_id: T::Hash,
        who: &T::AccountId,
        reserved: FixedU128,
    ) -> DispatchResult {
        let opaque = Self::fixed_to_opaque(reserved)?;
        <Accounts<T>>::mutate(share_id, who, |data| data.reserved = opaque);
        Ok(())
    }

    #[inline]
    fn fixed_to_opaque(fixed: FixedU128) -> Result<BalanceOf<T>, DispatchError> {
        if let Ok(balance) = fixed.into_inner().try_into() {
            Ok(balance)
        } else {
            let msg = "Couldn't convert fixed decimal number into balance storage";
            Err(DispatchError::Other(msg).into())
        }
    }

    #[inline]
    fn get_module_id() -> T::AccountId {
        T::ModuleId::get().into_account()
    }

    #[inline]
    fn opaque_to_fixed(opaque: BalanceOf<T>) -> FixedU128 {
        FixedU128::from_inner(opaque.saturated_into())
    }
}

impl<T: Trait> Shares<T::AccountId, T::Hash> for Module<T> {
    fn free_balance(share_id: T::Hash, who: &T::AccountId) -> FixedU128 {
        Self::opaque_to_fixed(Self::accounts(share_id, who).free)
    }

    fn total_supply(share_id: T::Hash) -> FixedU128 {
        Self::opaque_to_fixed(<TotalSupply<T>>::get(share_id))
    }

    fn destroy(share_id: T::Hash, from: &T::AccountId, amount: FixedU128) -> DispatchResult {
        let amount_opaque = Self::fixed_to_opaque(amount)?;

        if amount.is_zero() {
            return Ok(());
        }

        Self::ensure_can_withdraw(share_id, from, amount)?;

        <TotalSupply<T>>::mutate(share_id, |am| *am -= amount_opaque);
        Self::set_balance(share_id, from, Self::free_balance(share_id, from) - amount)?;

        Ok(())
    }

    fn destroy_all(share_id: T::Hash) -> DispatchResult {
        <Accounts<T>>::remove_prefix(share_id);
        <TotalSupply<T>>::remove(share_id);

        Ok(())
    }

    fn ensure_can_withdraw(
        share_id: T::Hash,
        who: &T::AccountId,
        amount: FixedU128,
    ) -> DispatchResult {
        if amount.is_zero() {
            return Ok(());
        }

        let _new_balance = Self::free_balance(share_id, who)
            .checked_sub(&amount)
            .ok_or(Error::<T>::BalanceTooLow)?;
        Ok(())
    }

    fn generate(share_id: T::Hash, to: &T::AccountId, amount: FixedU128) -> DispatchResult {
        if amount.is_zero() {
            return Ok(());
        }
        let new_total = Self::total_supply(share_id)
            .checked_add(&Self::fixed_to_opaque(amount)?)
            .ok_or(Error::<T>::TotalIssuanceOverflow)?;
        <TotalSupply<T>>::insert(share_id, new_total);
        Self::set_balance(share_id, to, Self::free_balance(share_id, to) + amount)?;

        Ok(())
    }

    fn transfer(
        share_id: T::Hash,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: FixedU128,
    ) -> DispatchResult {
        if amount.is_zero() || from == to {
            return Ok(());
        }

        Self::ensure_can_withdraw(share_id, from, amount)?;

        let from_balance = Self::free_balance(share_id, from);
        let to_balance = Self::free_balance(share_id, to);
        Self::set_balance(share_id, from, from_balance - amount)?;
        Self::set_balance(share_id, to, to_balance + amount)?;

        Ok(())
    }
}

impl<T: Trait> ReservableShares<T::AccountId, T::Hash> for Module<T> {
    fn can_reserve(share_id: T::Hash, who: &T::AccountId, value: FixedU128) -> bool {
        if value.is_zero() {
            return true;
        }

        Self::free_balance(share_id, who)
            .checked_sub(&value)
            .map_or(false, |new_balance| {
                Self::ensure_can_withdraw(share_id, who, new_balance).is_ok()
            })
    }

    fn reserved_balance(share_id: T::Hash, who: &T::AccountId) -> FixedU128 {
        Self::opaque_to_fixed(Self::accounts(share_id, who).reserved)
    }

    fn reserve(share_id: T::Hash, who: &T::AccountId, value: FixedU128) -> DispatchResult {
        if value.is_zero() {
            return Ok(());
        }

        let free = Self::free_balance(share_id, who);
        let reserved = Self::reserved_balance(share_id, who);
        let new_free = free.checked_sub(&value).ok_or(Error::<T>::Underflow)?;
        let new_reserved = reserved.checked_add(&value).ok_or(Error::<T>::Overflow)?;

        Self::set_balance(share_id, who, new_free)?;
        Self::set_reserved(share_id, who, new_reserved)?;
        Self::deposit_event(RawEvent::Reserved(share_id, who.clone(), value));
        Ok(())
    }

    fn unreserve(
        share_id: T::Hash,
        who: &T::AccountId,
        value: FixedU128,
    ) -> Result<FixedU128, DispatchError> {
        if value.is_zero() {
            return Ok(FixedU128::zero());
        }

        let free = Self::free_balance(share_id, who);
        let reserved = Self::reserved_balance(share_id, who);
        let actual = cmp::min(reserved, value);
        let new_free = free + actual;
        let new_reserved = reserved - actual;

        Self::set_balance(share_id, who, new_free)?;
        Self::set_reserved(share_id, who, new_reserved)?;
        Self::deposit_event(RawEvent::Unreserved(share_id, who.clone(), actual));

        Ok(actual)
    }
}

impl<T: Trait> WrapperShares<T::AccountId, T::Hash> for Module<T> {
    fn get_native_currency_id() -> T::Hash {
        let mut h = T::Hash::default();
        h.as_mut().iter_mut().for_each(|byte| *byte = 00);
        h
    }

    fn do_wrap_native_currency(who: T::AccountId, amount: FixedU128) -> DispatchResult {
        ensure!(
            Self::opaque_to_fixed(T::Currency::free_balance(&who)) >= amount,
            Error::<T>::BalanceTooLow
        );

        let id = Self::get_native_currency_id();

        T::Currency::transfer(
            &who,
            &Self::get_module_id(),
            Self::fixed_to_opaque(amount)?,
            ExistenceRequirement::KeepAlive,
        )?;
        Self::generate(id, &who, amount)
    }

    fn do_unwrap_native_currency(who: T::AccountId, amount: FixedU128) -> DispatchResult {
        let id = Self::get_native_currency_id();

        ensure!(
            Self::free_balance(id, &who) >= amount,
            Error::<T>::BalanceTooLow
        );

        Self::destroy(id, &who, amount)?;
        T::Currency::transfer(
            &Self::get_module_id(),
            &who,
            Self::fixed_to_opaque(amount)?,
            ExistenceRequirement::AllowDeath,
        )
    }
}
