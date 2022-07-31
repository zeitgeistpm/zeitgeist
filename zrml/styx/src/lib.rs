#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

extern crate alloc;

mod mock;
mod tests;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        pallet_prelude::*,
        traits::{Currency, NamedReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::SaturatedConversion;
    use zeitgeist_primitives::types::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The origin that is allowed to destroy markets.
        type SetBurnAmountOrigin: EnsureOrigin<Self::Origin>;

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: NamedReservableCurrency<Self::AccountId, ReserveIdentifier = [u8; 8]>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Keep track of crossings. Accounts are only able to cross once.
    #[pallet::storage]
    pub type Crossings<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool>;

    #[pallet::type_value]
    pub fn DefaultBurnAmount<T: Config>() -> Balance {
        (zeitgeist_primitives::constants::BASE * 100).saturated_into()
    }

    /// An extra layer of pseudo randomness.
    #[pallet::storage]
    pub type BurnAmount<T: Config> = StorageValue<_, Balance, ValueQuery, DefaultBurnAmount<T>>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A account crossed and claimed their right to create their avatar.
        AccountCrossed(T::AccountId, Balance),
        /// The crossing fee was changed.
        CrossingFeeChanged(T::AccountId, Balance),
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Account does not have enough balance to cross.
        FundDoesNotHaveEnoughFreeBalance,
        /// Account has already crossed.
        HasAlreadyCrossed,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Burns 200 ZTG to cross, granting the ability to claim your zeitgeist avatar.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn cross(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            if Crossings::<T>::contains_key(&who) {
                Err(Error::<T>::HasAlreadyCrossed)?;
            }

            let amount = BurnAmount::<T>::get().saturated_into();
            let free = T::Currency::free_balance(&who);

            if free < amount {
                Err(Error::<T>::FundDoesNotHaveEnoughFreeBalance)?;
            }

            T::Currency::slash(&who, amount);
            Crossings::<T>::insert(&who, true);

            Self::deposit_event(Event::AccountCrossed(who, amount.saturated_into()));

            Ok(())
        }

        /// Set the burn amount. Needs 50% council vote.
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
        pub fn set_burn_amount(origin: OriginFor<T>, amount: Balance) -> DispatchResult {
            let who = ensure_signed(origin.clone())?;
            T::SetBurnAmountOrigin::ensure_origin(origin)?;

            BurnAmount::<T>::put(amount);

            Self::deposit_event(Event::CrossingFeeChanged(who, amount));

            Ok(())
        }
    }
}
