#![cfg_attr(not(feature = "std"), no_std)]

use xrml_traits::shares::Shares;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

}

decl_storage! {
    trait Store for Module<T: Trait> as Shares {

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
    }
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;
    }
}
