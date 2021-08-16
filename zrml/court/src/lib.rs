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
        pallet_prelude::{StorageDoubleMap, StorageMap, StorageValue, ValueQuery},
        traits::{Currency, Get, Hooks, IsType, Randomness, ReservableCurrency},
        Blake2_128Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use rand::{rngs::StdRng, seq::SliceRandom, RngCore, SeedableRng};
    use sp_runtime::{traits::Saturating, ArithmeticError, DispatchError, SaturatedConversion};
    use zeitgeist_primitives::{
        traits::DisputeApi,
        types::{Market, MarketDispute, OutcomeReport},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    // Number of jurors for an initial market dispute
    const INITIAL_JURORS_NUM: usize = 3;
    const MAX_RANDOM_JURORS: usize = 13;
    // Weight used to increase the number of jurors for subsequent disputes
    // of the same market
    const SUBSEQUENT_JURORS_FACTOR: usize = 2;

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
            let who = ensure_signed(origin)?;
            let juror = Self::juror(&who)?;
            Jurors::<T>::remove(&who);
            CurrencyOf::<T>::unreserve(&who, juror.staked);
            Ok(())
        }

        // `transactional` attribute is not used here because once `reserve` is successful, `insert`
        // won't fail.
        #[pallet::weight(0)]
        pub fn join_court(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            if Jurors::<T>::get(&who).is_some() {
                return Err(Error::<T>::JurorAlreadyExists.into());
            }
            let jurors_num = Jurors::<T>::iter().count();
            let jurors_num_plus_one = jurors_num.checked_add(1).ok_or(ArithmeticError::Overflow)?;
            let stake = Self::current_required_stake(jurors_num_plus_one);
            CurrencyOf::<T>::reserve(&who, stake)?;
            Jurors::<T>::insert(&who, Juror { staked: stake, status: JurorStatus::Ok });
            Ok(())
        }

        // `transactional` attribute is not used here because no fallible storage operation
        // is performed
        #[pallet::weight(0)]
        pub fn vote(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            if Jurors::<T>::get(&who).is_none() {
                return Err(Error::<T>::OnlyJurorsCanVote.into());
            }
            Votes::<T>::insert(
                who,
                market_id,
                (<frame_system::Pallet<T>>::block_number(), outcome),
            );
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
        /// Forbids voting of unknown accounts
        OnlyJurorsCanVote,
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
        pub(crate) fn random_jurors<'a, 'b, R>(
            jurors: &'a [(T::AccountId, Juror<BalanceOf<T>>)],
            len: usize,
            rng: &mut R,
        ) -> ArrayVec<&'b (T::AccountId, Juror<BalanceOf<T>>), MAX_RANDOM_JURORS>
        where
            R: RngCore,
            'a: 'b,
        {
            let actual_len = jurors.len().min(len);
            jurors.choose_multiple(rng, actual_len).collect()
        }

        // Returns a pseudo random number generator implementation based on the seed
        // provided by the `Config::Random` type and the `JurorsSelectionNonce` storage.
        pub(crate) fn rng() -> impl RngCore {
            let nonce = <JurorsSelectionNonce<T>>::mutate(|n| {
                let rslt = *n;
                *n = n.wrapping_add(1);
                rslt
            });
            let mut seed = [0; 32];
            let (random_hash, _) = T::Random::random(&nonce.to_le_bytes());
            for (byte, el) in random_hash.as_ref().iter().copied().zip(seed.iter_mut()) {
                *el = byte
            }
            StdRng::from_seed(seed)
        }

        // No-one can stake more than BalanceOf::<T>::max(), therefore, this function saturates
        // arithmetic operations.
        fn current_required_stake(jurors_num: usize) -> BalanceOf<T> {
            let jurors_len: BalanceOf<T> = jurors_num.saturated_into();
            T::StakeWeight::get().saturating_mul(jurors_len)
        }

        // Calculates the necessary number of jurors depending on the number of market disputes.
        //
        // Result is capped to `usize::MAX` or in other words, capped to a very, very, very
        // high number of jurors.
        fn necessary_jurors_num(disputes: &[MarketDispute<T::AccountId, T::BlockNumber>]) -> usize {
            let len = disputes.len();
            INITIAL_JURORS_NUM.saturating_add(SUBSEQUENT_JURORS_FACTOR.saturating_mul(len))
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
            disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: Self::MarketId,
            who: Self::AccountId,
        ) -> DispatchResult
        where
            D: Fn(usize) -> Self::Balance,
        {
            CurrencyOf::<T>::reserve(&who, dispute_bond(disputes.len()))?;
            let jurors: Vec<_> = Jurors::<T>::iter().collect();
            let necessary_jurors_num = Self::necessary_jurors_num(disputes);
            let mut rng = Self::rng();
            let random_jurors = Self::random_jurors(&jurors, necessary_jurors_num, &mut rng);
            for (ai, _) in random_jurors {
                RequestedJurors::<T>::insert(ai, market_id, T::CourtCaseDuration::get());
            }
            Ok(())
        }

        fn on_resolution<F>(_now: Self::BlockNumber, _cb: F) -> DispatchResult
        where
            F: FnMut(
                &Self::MarketId,
                &Market<Self::AccountId, Self::BlockNumber>,
            ) -> DispatchResult,
        {
            Ok(())
        }
    }

    /// Accounts that stake funds to decide outcomes.
    #[pallet::storage]
    pub type Jurors<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Juror<BalanceOf<T>>>;

    /// An extra layer of pseudo randomness.
    #[pallet::storage]
    pub type JurorsSelectionNonce<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Selected jurors that should vote a market outcome until a certain block number
    #[pallet::storage]
    pub type RequestedJurors<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        MarketIdOf<T>,
        T::BlockNumber,
    >;

    /// Votes of market outcomes for disputes
    #[pallet::storage]
    pub type Votes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        MarketIdOf<T>,
        (T::BlockNumber, OutcomeReport),
    >;
}
