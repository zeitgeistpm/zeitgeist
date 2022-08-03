#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod benchmarks;
mod mock;
mod tests;
pub mod weights;
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

    use crate::weights::WeightInfoZeitgeist;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The origin that is allowed to set the amount burned when crossing Styx.
        type SetBurnAmountOrigin: EnsureOrigin<Self::Origin>;

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Currency: NamedReservableCurrency<Self::AccountId, ReserveIdentifier = [u8; 8]>;

        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// Keep track of crossings. Accounts are only able to cross once.
    #[pallet::storage]
    pub type Crossings<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, ()>;

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
        CrossingFeeChanged(Balance),
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
        /// Burns ZTG(styx.burnAmount()) to cross, granting the ability to claim your zeitgeist avatar.
        #[pallet::weight(T::WeightInfo::cross())]
        pub fn cross(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            if Crossings::<T>::contains_key(&who) {
                Err(Error::<T>::HasAlreadyCrossed)?;
            }

            let amount = BurnAmount::<T>::get().saturated_into();

            if !T::Currency::can_slash(&who, amount) {
                Err(Error::<T>::FundDoesNotHaveEnoughFreeBalance)?;
            }

            T::Currency::slash(&who, amount);
            Crossings::<T>::insert(&who, ());

            Self::deposit_event(Event::AccountCrossed(who, amount.saturated_into()));

            Ok(())
        }

        /// Set the burn amount. Ensures SetBurnAmountOrigin.
        #[pallet::weight(T::WeightInfo::set_burn_amount())]
        pub fn set_burn_amount(
            origin: OriginFor<T>,
            #[pallet::compact] amount: Balance,
        ) -> DispatchResult {
            T::SetBurnAmountOrigin::ensure_origin(origin)?;
            BurnAmount::<T>::put(amount);

            Self::deposit_event(Event::CrossingFeeChanged(amount));

            Ok(())
        }
    }
}
