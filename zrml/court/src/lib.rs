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
    use arrayvec::ArrayVec;
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::StorageMap,
        traits::{Currency, Get, Hooks, IsType, Randomness, ReservableCurrency},
        Blake2_128Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::{traits::Saturating, ArithmeticError, DispatchError, SaturatedConversion};
    use zeitgeist_primitives::{
        traits::DisputeApi,
        types::{Market, OutcomeReport, ResolutionCounters},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    const MAX_RANDOM_JURORS: usize = 13;

    pub(crate) type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
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
        /// Block duration to cast a vote on an outcome.
        type CourtCaseDuration: Get<Self::BlockNumber>;

        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Market commons
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// Randomness source
        type Random: Randomness<Self::Hash, Self::BlockNumber>;

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
        // Returns an unique random subset of `jurors` with length `len`.
        //
        // If `len` is greater than the length of `jurors`, then `len` will be capped.
        pub(crate) fn random_jurors<'a, 'b>(
            jurors: &'a [(T::AccountId, Juror<BalanceOf<T>>)],
            len: usize,
        ) -> ArrayVec<&'b Juror<BalanceOf<T>>, MAX_RANDOM_JURORS>
        where
            'a: 'b,
        {
            let actual_len = jurors.len().min(len);
            let mut subset = ArrayVec::new();
            if actual_len == 0 {
                return subset;
            }
            // https://github.com/paritytech/substrate/issues/8312
            let (random_hash, _) = T::Random::random(b"zrml-court");
            for byte in random_hash.as_ref().iter().copied() {
                if subset.len() == MAX_RANDOM_JURORS {
                    break;
                }
                // `actual_len` will never be 0
                let idx = Into::<usize>::into(byte) % actual_len;
                // `idx` will always be within the length of `jurors`
                let (_, juror) = jurors.get(idx).unwrap();
                let juror_is_not_included = subset.iter().all(|&el| el != juror);
                if juror_is_not_included {
                    // `push` will never overflow the internal capacity of `MAX_RANDOM_JURORS` jurors
                    subset.push(juror);
                }
            }
            subset
        }

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
        type Balance = BalanceOf<T>;
        type BlockNumber = T::BlockNumber;
        type Origin = T::Origin;
        type MarketId = MarketIdOf<T>;

        fn on_dispute<D>(
            dispute_bond: D,
            _market_id: Self::MarketId,
            _outcome: OutcomeReport,
            who: Self::AccountId,
        ) -> DispatchResult
        where
            D: Fn(usize) -> Self::Balance,
        {
            CurrencyOf::<T>::reserve(&who, dispute_bond(1))?;
            let jurors: Vec<_> = Jurors::<T>::iter().collect();
            let _ = Self::random_jurors(&jurors, 3);
            Ok(())
        }

        fn on_resolution<D, F>(_dispute_bond: D, _now: Self::BlockNumber, _cb: F) -> DispatchResult
        where
            D: Fn(usize) -> Self::Balance,
            F: FnMut(&Market<Self::AccountId, Self::BlockNumber>, ResolutionCounters),
        {
            todo!()
        }
    }

    /// Accounts that stake funds to decide outcomes.
    #[pallet::storage]
    pub type Jurors<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Juror<BalanceOf<T>>>;
}
