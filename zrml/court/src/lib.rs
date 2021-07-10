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
        traits::{Currency, Get, Hooks, IsType, ReservableCurrency},
        Blake2_128Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::{traits::Saturating, ArithmeticError, DispatchError, SaturatedConversion};
    use zeitgeist_primitives::{
        traits::DisputeApi,
        types::{Market, OutcomeReport, ResolutionCounters},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    pub(crate) type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[frame_support::transactional]
        #[pallet::weight(0)]
        pub fn exit_court(origin: OriginFor<T>) -> DispatchResult {
            let account_id = ensure_signed(origin)?;
            let juror = Self::juror(&account_id)?;
            Jurors::<T>::remove(&account_id);
            CurrencyOf::<T>::unreserve(&account_id, juror.staked);
            Ok(())
        }

        #[frame_support::transactional]
        #[pallet::weight(0)]
        pub fn join_court(origin: OriginFor<T>) -> DispatchResult {
            let account_id = ensure_signed(origin)?;
            if Jurors::<T>::get(&account_id).is_some() {
                return Err(Error::<T>::JurorAlreadyExists.into());
            }
            let jurors_num = Jurors::<T>::iter().count();
            let jurors_num_plus_one = jurors_num.checked_add(1).ok_or(ArithmeticError::Overflow)?;
            let stake = Self::current_required_stake(jurors_num_plus_one);
            Jurors::<T>::insert(&account_id, Juror { staked: stake, status: JurorStatus::Ok });
            CurrencyOf::<T>::reserve(&account_id, stake)?;
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

        /// Weight used to calculate the necessary staking amount to become a juror
        type StakeWeight: Get<BalanceOf<Self>>;
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
        // No-one can stake more than BalanceOf::<T>::max(), therefore, this function saturates
        // arithmetic operations.
        fn current_required_stake(jurors_num: usize) -> BalanceOf<T> {
            let jurors_len: BalanceOf<T> = jurors_num.saturated_into();
            T::StakeWeight::get().saturating_mul(jurors_len)
        }

        // Retrieves a juror from the storage
        fn juror(account_id: &T::AccountId) -> Result<Juror<BalanceOf<T>>, DispatchError> {
            Jurors::<T>::get(account_id).ok_or_else(|| Error::<T>::JurorDoesNotExists.into())
        }
    }

    impl<T> DisputeApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type BlockNumber = T::BlockNumber;
        type Origin = T::Origin;
        type MarketId = MarketIdOf<T>;

        fn on_dispute(
            _origin: Self::Origin,
            _market_id: Self::MarketId,
            _outcome: OutcomeReport,
        ) -> Result<[u32; 2], DispatchError> {
            todo!()
        }

        fn on_resolution<F>(_now: Self::BlockNumber, _cb: F) -> DispatchResult
        where
            F: FnMut(&Market<Self::AccountId, Self::BlockNumber>, ResolutionCounters),
        {
            todo!()
        }
    }

    /// Accounts that stake funds to decide outcomes.
    #[pallet::storage]
    pub type Jurors<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Juror<BalanceOf<T>>>;
}
