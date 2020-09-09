#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, Parameter,
};
use frame_system::ensure_signed;
use sp_runtime::{
    traits::{AtLeast32Bit, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, StaticLookup, Zero},
    DispatchResult, RuntimeDebug,
};
use sp_std::{
    cmp,
	prelude::*,
};
use xrml_traits::shares::{ReservableShares, Shares};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct AccountData<Balance> {
    pub free: Balance,
    pub reserved: Balance,
}

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Balance: Parameter + Member + Copy + MaybeSerializeDeserialize + AtLeast32Bit + Default;
}

decl_storage! {
    trait Store for Module<T: Trait> as Shares {
        pub Accounts get(fn accounts):
            double_map hasher (blake2_128_concat) T::AccountId, hasher (identity) T::Hash =>
                AccountData<T::Balance>;

        pub TotalSupply get(fn total_supply): map hasher (identity) T::Hash => T::Balance;
    }
}

decl_event!(
    pub enum Event<T>
        where
            AccountId = <T as frame_system::Trait>::AccountId,
            Hash = <T as frame_system::Trait>::Hash,
            Balance = <T as Trait>::Balance,
    {
        /// Some shares have been transferred. [shares_id, from, to, amount]
        Transferred(Hash, AccountId, AccountId, Balance),
        /// Some shares have been reserved. [shares_id, who, amount]
        Reserved(Hash, AccountId, Balance),
        /// Shares have been unreserved. [shares_id, who, amount]
        Unreserved(Hash, AccountId, Balance),
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
            #[compact] amount: T::Balance,
        ) {
            let from = ensure_signed(origin)?;
            let to = T::Lookup::lookup(dest)?;
            <Self as Shares<T::AccountId, T::Balance, T::Hash>>::transfer(share_id, &from, &to, amount)?;

            Self::deposit_event(RawEvent::Transferred(share_id, from, to, amount));
        }
    }
}

impl<T: Trait> Module<T> {
    fn set_balance(share_id: T::Hash, who: &T::AccountId, balance: T::Balance) {
        <Accounts<T>>::mutate(who, share_id, |data| data.free = balance);
    }

    fn set_reserved(share_id: T::Hash, who: &T::AccountId, reserved: T::Balance) {
        <Accounts<T>>::mutate(who, share_id, |data| data.reserved = reserved);
    }

}

impl<T: Trait> Shares<T::AccountId, T::Balance, T::Hash> for Module<T> {
    type Balance = T::Balance;

    fn free_balance(share_id: T::Hash, who: &T::AccountId) -> Self::Balance {
        Self::accounts(who, share_id).free
    }

    fn total_supply(share_id: T::Hash) -> Self::Balance {
        <TotalSupply<T>>::get(share_id)
    }

    fn destroy(share_id: T::Hash, from: &T::AccountId, amount: Self::Balance) -> DispatchResult {
        if amount.is_zero() {
            return Ok(());
        }

        Self::ensure_can_withdraw(share_id, from, amount)?;

        <TotalSupply<T>>::mutate(share_id, |am| *am -= amount);
        Self::set_balance(share_id, from, Self::free_balance(share_id, from) - amount);

        Ok(())
    }

    fn ensure_can_withdraw(share_id: T::Hash, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
        if amount.is_zero() {
            return Ok(());
        }

        let _new_balance = Self::free_balance(share_id, who)
            .checked_sub(&amount)
            .ok_or(Error::<T>::BalanceTooLow)?;
        Ok(())
    }

    fn generate(share_id: T::Hash, to: &T::AccountId, amount: Self::Balance) -> DispatchResult {
        if amount.is_zero() {
            return Ok(());
        }

        let new_total = Self::total_supply(share_id)
            .checked_add(&amount)
            .ok_or(Error::<T>::TotalIssuanceOverflow)?;
        <TotalSupply<T>>::insert(share_id, new_total);
        Self::set_balance(share_id, to, Self::free_balance(share_id, to) + amount);

        Ok(())
    }

    fn transfer(share_id: T::Hash, from: &T::AccountId, to: &T::AccountId, amount: Self::Balance) -> DispatchResult {
        if amount.is_zero() || from == to {
            return Ok(());
        }

        Self::ensure_can_withdraw(share_id, from, amount)?;

        let from_balance = Self::free_balance(share_id, from);
        let to_balance = Self::free_balance(share_id, to);
        Self::set_balance(share_id, from, from_balance - amount);
        Self::set_balance(share_id, to, to_balance + amount);

        Ok(())
    }
}

impl<T: Trait> ReservableShares<T::AccountId, T::Balance, T::Hash> for Module<T> {
    fn can_reserve(share_id: T::Hash, who: &T::AccountId, value: T::Balance) -> bool {
        if value.is_zero() { return true }

        Self::free_balance(share_id, who)
            .checked_sub(&value)
            .map_or(false, |new_balance|
                Self::ensure_can_withdraw(share_id, who, new_balance).is_ok()
            )
    }

    fn reserved_balance(share_id: T::Hash, who: &T::AccountId) -> T::Balance {
        Self::accounts(who, share_id).reserved
    }

    fn reserve(share_id: T::Hash, who: &T::AccountId, value: T::Balance) -> DispatchResult {
        if value.is_zero() { return Ok(()) }

        let free = Self::free_balance(share_id, who);
        let reserved = Self::reserved_balance(share_id, who);
        let new_free = free.checked_sub(&value).ok_or(Error::<T>::Underflow)?;
        let new_reserved = reserved.checked_add(&value).ok_or(Error::<T>::Overflow)?;

        Self::set_balance(share_id, who, new_free);
        Self::set_reserved(share_id, who, new_reserved);
        Self::deposit_event(RawEvent::Reserved(share_id, who.clone(), value));
        Ok(())
    }

    fn unreserve(share_id: T::Hash, who: &T::AccountId, value: T::Balance) -> T::Balance {
        if value.is_zero() { return Zero::zero() }

        let free = Self::free_balance(share_id, who);
        let reserved = Self::reserved_balance(share_id, who);
        let actual = cmp::min(reserved, value);
        let new_free = free + actual;
        let new_reserved = reserved - actual;

        Self::set_balance(share_id, who, new_free);
        Self::set_reserved(share_id, who, new_reserved);
        Self::deposit_event(RawEvent::Unreserved(share_id, who.clone(), actual));

        actual
    }
}
