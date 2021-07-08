//! # Court

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod juror;
mod juror_status;
mod mock;
mod tests;

pub use juror::Juror;
pub use juror_status::JurorStatus;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{Juror, JurorStatus};
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::StorageMap,
        traits::{Currency, Hooks, IsType},
        Blake2_128Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::DispatchError;
    use zrml_market_commons::MarketCommonsPalletApi;

    pub(crate) type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn exit_court(origin: OriginFor<T>) -> DispatchResult {
            let account_id = ensure_signed(origin)?;
            let _ = Self::juror(&account_id)?;
            Jurors::<T>::remove(account_id);
            Ok(())
        }

        #[pallet::weight(0)]
        pub fn join_court(origin: OriginFor<T>) -> DispatchResult {
            let account_id = ensure_signed(origin)?;
            if Jurors::<T>::get(&account_id).is_some() {
                return Err(Error::<T>::JurorAlreadyExists.into());
            }
            Jurors::<T>::insert(
                account_id,
                Juror { staked: Default::default(), status: JurorStatus::Ok },
            );
            Ok(())
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Market commons
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// It is not possible to insert a Juror that is already stored
        JurorAlreadyExists,
        /// An account id does not exist on the jurors storage.
        JurorDoesNotExists,
    }

    #[pallet::event]
    pub enum Event<T>
    where
        T: Config, {}

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    impl<T> Pallet<T>
    where
        T: Config,
    {
        // Retrieves a juror from the storage
        fn juror(account_id: &T::AccountId) -> Result<Juror<BalanceOf<T>>, DispatchError> {
            Jurors::<T>::get(account_id).ok_or(Error::<T>::JurorDoesNotExists.into())
        }
    }

    /// Accounts that stake funds to decide outcomes.
    #[pallet::storage]
    pub type Jurors<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Juror<BalanceOf<T>>>;
}
