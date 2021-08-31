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
    use alloc::collections::BTreeMap;
    use arrayvec::ArrayVec;
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::{StorageDoubleMap, StorageMap, StorageValue, ValueQuery},
        traits::{
            BalanceStatus, Currency, Get, Hooks, IsType, NamedReservableCurrency, Randomness,
            ReservableCurrency,
        },
        Blake2_128Concat, PalletId,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use rand::{rngs::StdRng, seq::SliceRandom, RngCore, SeedableRng};
    use sp_runtime::{
        traits::{AccountIdConversion, Saturating},
        ArithmeticError, DispatchError, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        constants::CourtPalletId,
        traits::DisputeApi,
        types::{Market, MarketDispute, OutcomeReport},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    // Number of jurors for an initial market dispute
    const INITIAL_JURORS_NUM: usize = 3;
    const MAX_RANDOM_JURORS: usize = 13;
    const RESERVE_ID: [u8; 8] = CourtPalletId::get().0;
    // Weight used to increase the number of jurors for subsequent disputes
    // of the same market
    const SUBSEQUENT_JURORS_FACTOR: usize = 2;
    // Divides the reserved juror balance to calculate the slash amount. `5` here
    // means that the output value will be 20% of the dividend.
    const TARDY_PUNISHMENT_DIVISOR: u8 = 5;

    pub(crate) type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn exit_court(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let juror = Self::juror(&who)?;
            Jurors::<T>::remove(&who);
            CurrencyOf::<T>::unreserve_named(&RESERVE_ID, &who, juror.staked);
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
            CurrencyOf::<T>::reserve_named(&RESERVE_ID, &who, stake)?;
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
                market_id,
                who,
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

        /// Identifier of this pallet
        type PalletId: Get<PalletId>;

        /// Randomness source
        type Random: Randomness<Self::Hash, Self::BlockNumber>;

        /// Weight used to calculate the necessary staking amount to become a juror
        type StakeWeight: Get<BalanceOf<Self>>;

        /// Slashed funds are send to the treasury
        type TreasuryId: Get<PalletId>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// It is not possible to insert a Juror that is already stored
        JurorAlreadyExists,
        /// An account id does not exist on the jurors storage.
        JurorDoesNotExists,
        /// No-one voted on an outcome to resolve a market
        NoVotes,
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
            'a: 'b,
            R: RngCore,
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

        pub(crate) fn set_juror_as_tardy(account_id: &T::AccountId) -> DispatchResult {
            Self::mutate_juror(account_id, |juror| {
                juror.status = JurorStatus::Tardy;
                Ok(())
            })
        }

        pub(crate) fn treasury_account_id() -> T::AccountId {
            T::TreasuryId::get().into_account()
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

        // Calculates the necessary number of jurors depending on the number of market disputes.
        //
        // Result is capped to `usize::MAX` or in other words, capped to a very, very, very
        // high number of jurors.
        fn necessary_jurors_num(disputes: &[MarketDispute<T::AccountId, T::BlockNumber>]) -> usize {
            let len = disputes.len();
            INITIAL_JURORS_NUM.saturating_add(SUBSEQUENT_JURORS_FACTOR.saturating_mul(len))
        }

        // * Jurors that didn't vote within `CourtCaseDuration` or didn't vote at all are
        // placed as tardy.
        //
        // * Slashes 20% of staked funds and removes tardy jurors that didn't vote a second time.
        fn manage_tardy_jurors(
            requested_jurors: &[(T::AccountId, T::BlockNumber)],
            votes: &[(T::AccountId, (T::BlockNumber, OutcomeReport))],
        ) -> DispatchResult {
            let treasury_account_id = Self::treasury_account_id();

            for (ai, max_block) in requested_jurors {
                if let Some((_, (block, _))) = votes.iter().find(|el| &el.0 == ai) {
                    if block > max_block {
                        Self::set_juror_as_tardy(ai)?;
                    }
                } else {
                    let juror = Self::juror(ai)?;
                    if let JurorStatus::Tardy = juror.status {
                        let reserved = CurrencyOf::<T>::reserved_balance_named(&RESERVE_ID, ai);
                        // Division will never overflow
                        let slash = reserved / BalanceOf::<T>::from(TARDY_PUNISHMENT_DIVISOR);
                        CurrencyOf::<T>::repatriate_reserved_named(
                            &RESERVE_ID,
                            ai,
                            &treasury_account_id,
                            slash,
                            BalanceStatus::Free,
                        )?;
                        CurrencyOf::<T>::unreserve_named(&RESERVE_ID, ai, reserved);
                        Jurors::<T>::remove(ai);
                    } else {
                        Self::set_juror_as_tardy(ai)?;
                    }
                }
            }

            Ok(())
        }

        // Retrieves a juror from the storage
        fn mutate_juror<F>(account_id: &T::AccountId, mut cb: F) -> DispatchResult
        where
            F: FnMut(&mut Juror<BalanceOf<T>>) -> DispatchResult,
        {
            Jurors::<T>::try_mutate(account_id, |opt| {
                if let Some(el) = opt {
                    cb(el)?;
                } else {
                    return Err(Error::<T>::JurorDoesNotExists.into());
                }
                Ok(())
            })
        }

        // Jurors are only rewarded if sided on the most voted outcome but jurors that voted
        // second most voted outcome (winner of the losing majority) are placed as tardy instead
        // of being slashed
        fn set_jurors_that_sided_on_the_second_most_voted_outcome_as_tardy(
            second_most_voted_outcome: &Option<OutcomeReport>,
            votes: &[(T::AccountId, (T::BlockNumber, OutcomeReport))],
        ) -> DispatchResult {
            if let Some(el) = second_most_voted_outcome {
                for (ai, (_, outcome_report)) in votes {
                    if outcome_report == el {
                        Self::set_juror_as_tardy(ai)?;
                    }
                }
            }
            Ok(())
        }

        // For market resolution based on the votes of a market
        fn two_best_outcomes(
            votes: &[(T::AccountId, (T::BlockNumber, OutcomeReport))],
        ) -> Result<(OutcomeReport, Option<OutcomeReport>), DispatchError> {
            let mut scores = BTreeMap::<OutcomeReport, u32>::new();

            for (_, (_, outcome_report)) in votes {
                if let Some(el) = scores.get_mut(outcome_report) {
                    *el = el.saturating_add(1);
                } else {
                    scores.insert(outcome_report.clone(), 1);
                }
            }

            let mut best_score;
            let mut iter = scores.iter();

            if let Some(first) = iter.next() {
                best_score = first;
            } else {
                return Err(Error::<T>::NoVotes.into());
            }

            let mut second_best_score = if let Some(second) = iter.next() {
                if second.1 > best_score.1 {
                    best_score = second;
                    best_score
                } else {
                    second
                }
            } else {
                return Ok((best_score.0.clone(), None));
            };

            for el in iter {
                if el.1 > best_score.1 {
                    best_score = el;
                    second_best_score = best_score;
                } else if el.1 > second_best_score.1 {
                    second_best_score = el;
                }
            }

            Ok((best_score.0.clone(), Some(second_best_score.0.clone())))
        }
    }

    impl<T> DisputeApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type Balance = BalanceOf<T>;
        type BlockNumber = T::BlockNumber;
        type MarketId = MarketIdOf<T>;
        type Moment = MomentOf<T>;
        type Origin = T::Origin;

        fn on_dispute(
            bond: Self::Balance,
            disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: &Self::MarketId,
            who: &Self::AccountId,
        ) -> DispatchResult {
            CurrencyOf::<T>::reserve(who, bond)?;
            let jurors: Vec<_> = Jurors::<T>::iter().collect();
            let necessary_jurors_num = Self::necessary_jurors_num(disputes);
            let mut rng = Self::rng();
            let random_jurors = Self::random_jurors(&jurors, necessary_jurors_num, &mut rng);
            let curr_block_num = <frame_system::Pallet<T>>::block_number();
            let block_limit = curr_block_num.saturating_add(T::CourtCaseDuration::get());
            for (ai, _) in random_jurors {
                RequestedJurors::<T>::insert(market_id, ai, block_limit);
            }
            Ok(())
        }

        fn on_resolution<D>(
            _: &D,
            _: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: &Self::MarketId,
            _: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
        ) -> Result<OutcomeReport, DispatchError>
        where
            D: Fn(usize) -> Self::Balance,
        {
            let requested_jurors: Vec<_> = RequestedJurors::<T>::iter_prefix(market_id).collect();
            let votes: Vec<_> = Votes::<T>::iter_prefix(market_id).collect();
            let (first, second) = Self::two_best_outcomes(&votes)?;
            Self::manage_tardy_jurors(&requested_jurors, &votes)?;
            Self::set_jurors_that_sided_on_the_second_most_voted_outcome_as_tardy(&second, &votes)?;
            Votes::<T>::remove_prefix(market_id, None);
            RequestedJurors::<T>::remove_prefix(market_id, None);
            Ok(first)
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
        MarketIdOf<T>,
        Blake2_128Concat,
        T::AccountId,
        T::BlockNumber,
    >;

    /// Votes of market outcomes for disputes
    ///
    /// Stores the vote block number and the submitted outcome.
    #[pallet::storage]
    pub type Votes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        Blake2_128Concat,
        T::AccountId,
        (T::BlockNumber, OutcomeReport),
    >;
}
