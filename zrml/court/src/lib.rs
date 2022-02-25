//! # Court

// It is important to note that if a categorical market has only two outcomes, then winners
// won't receive any rewards because accounts of the most voted outcome on the loser side are
// simply registered as `JurorStatus::Tardy`.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod benchmarks;
mod court_pallet_api;
mod juror;
mod juror_status;
mod mock;
mod tests;
pub mod weights;

pub use court_pallet_api::CourtPalletApi;
pub use juror::Juror;
pub use juror_status::JurorStatus;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{weights::WeightInfoZeitgeist, CourtPalletApi, Juror, JurorStatus};
    use alloc::{
        collections::{BTreeMap, BTreeSet},
        vec::Vec,
    };
    use arrayvec::ArrayVec;
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::{StorageDoubleMap, StorageMap, StorageValue, ValueQuery},
        traits::{
            BalanceStatus, Currency, Get, Hooks, IsType, NamedReservableCurrency, Randomness,
            StorageVersion,
        },
        Blake2_128Concat, PalletId,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use rand::{rngs::StdRng, seq::SliceRandom, RngCore, SeedableRng};
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedDiv, Saturating},
        ArithmeticError, DispatchError, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        constants::CourtPalletId,
        traits::DisputeApi,
        types::{Market, MarketDispute, MarketDisputeMechanism, OutcomeReport},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    pub(crate) const RESERVE_ID: [u8; 8] = CourtPalletId::get().0;

    // Number of jurors for an initial market dispute
    const INITIAL_JURORS_NUM: usize = 3;
    const MAX_RANDOM_JURORS: usize = 13;
    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);
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
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // `transactional` attribute is not used simply because
        // `remove_juror_from_all_courts_of_all_markets` is infallible.
        #[pallet::weight(T::WeightInfo::exit_court())]
        pub fn exit_court(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let juror = Self::juror(&who)?;
            Self::remove_juror_from_all_courts_of_all_markets(&who);
            Self::deposit_event(Event::ExitedJuror(who, juror));
            Ok(())
        }

        // `transactional` attribute is not used here because once `reserve_named` is
        // successful, `insert` won't fail.
        #[pallet::weight(T::WeightInfo::join_court())]
        pub fn join_court(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            if Jurors::<T>::get(&who).is_some() {
                return Err(Error::<T>::JurorAlreadyExists.into());
            }
            let jurors_num = Jurors::<T>::iter().count();
            let jurors_num_plus_one = jurors_num.checked_add(1).ok_or(ArithmeticError::Overflow)?;
            let stake = Self::current_required_stake(jurors_num_plus_one);
            CurrencyOf::<T>::reserve_named(&RESERVE_ID, &who, stake)?;
            let juror = Juror { status: JurorStatus::Ok };
            Jurors::<T>::insert(&who, juror.clone());
            Self::deposit_event(Event::JoinedJuror(who, juror));
            Ok(())
        }

        // `transactional` attribute is not used here because no fallible storage operation
        // is performed.
        #[pallet::weight(T::WeightInfo::vote())]
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
        #[pallet::constant]
        type CourtCaseDuration: Get<Self::BlockNumber>;

        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Market commons
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// Identifier of this pallet
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Randomness source
        type Random: Randomness<Self::Hash, Self::BlockNumber>;

        /// Weight used to calculate the necessary staking amount to become a juror
        #[pallet::constant]
        type StakeWeight: Get<BalanceOf<Self>>;

        /// Slashed funds are send to the treasury
        #[pallet::constant]
        type TreasuryPalletId: Get<PalletId>;

        /// Weights generated by benchmarks
        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// It is not possible to insert a Juror that is already stored
        JurorAlreadyExists,
        /// An account id does not exist on the jurors storage.
        JurorDoesNotExists,
        /// On dispute or resolution, someone tried to pass a non-court market type
        MarketDoesNotHaveCourtMechanism,
        /// No-one voted on an outcome to resolve a market
        NoVotes,
        /// Forbids voting of unknown accounts
        OnlyJurorsCanVote,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        ExitedJuror(T::AccountId, Juror),
        JoinedJuror(T::AccountId, Juror),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    impl<T> Pallet<T>
    where
        T: Config,
    {
        // Returns an unique random subset of `jurors` with length `len`.
        //
        // If `len` is greater than the length of `jurors`, then `len` will be capped.
        pub(crate) fn random_jurors<'a, 'b, R>(
            jurors: &'a [(T::AccountId, Juror)],
            len: usize,
            rng: &mut R,
        ) -> ArrayVec<&'b (T::AccountId, Juror), MAX_RANDOM_JURORS>
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

        // Used to avoid code duplications.
        pub(crate) fn set_stored_juror_as_tardy(account_id: &T::AccountId) -> DispatchResult {
            Self::mutate_juror(account_id, |juror| {
                juror.status = JurorStatus::Tardy;
                Ok(())
            })
        }

        pub(crate) fn treasury_account_id() -> T::AccountId {
            T::TreasuryPalletId::get().into_account()
        }

        // No-one can stake more than BalanceOf::<T>::max(), therefore, this function saturates
        // arithmetic operations.
        fn current_required_stake(jurors_num: usize) -> BalanceOf<T> {
            let jurors_len: BalanceOf<T> = jurors_num.saturated_into();
            T::StakeWeight::get().saturating_mul(jurors_len)
        }

        // Retrieves a juror from the storage
        fn juror(account_id: &T::AccountId) -> Result<Juror, DispatchError> {
            Jurors::<T>::get(account_id).ok_or_else(|| Error::<T>::JurorDoesNotExists.into())
        }

        // # Manages tardy jurors and returns valid winners and valid losers.
        //
        // ## Management
        //
        // * Jurors that didn't vote within `CourtCaseDuration` or didn't vote at all are
        // placed as tardy.
        //
        // * Slashes 20% of staked funds and removes tardy jurors that didn't vote or voted
        // after the maximum allowed block.
        //
        // ## Returned list of accounts
        //
        // All new and old tardy jurors, excluding the ones that voted within `CourtCaseDuration`,
        // are removed from the list of accounts that will be slashed to reward winners. Already
        // tardy jurors that voted again on the second most voted outcome are also removed from the
        // same list.
        //
        // In other words, does not yield slashed accounts, winners of the losing side,
        // accounts that didn't vote or accounts that voted after the maximum allowed block
        fn manage_tardy_jurors<'a, 'b, F>(
            requested_jurors: &'a [(
                T::AccountId,
                Juror,
                T::BlockNumber,
                Option<&(T::BlockNumber, OutcomeReport)>,
            )],
            mut cb: F,
        ) -> Result<Vec<(&'b T::AccountId, &'b OutcomeReport)>, DispatchError>
        where
            F: FnMut(&OutcomeReport) -> bool,
            'a: 'b,
        {
            let mut valid_winners_and_losers = Vec::with_capacity(requested_jurors.len());
            let treasury_account_id = Self::treasury_account_id();

            let slash_and_remove_juror = |ai: &T::AccountId| {
                let all_reserved = CurrencyOf::<T>::reserved_balance_named(&RESERVE_ID, ai);
                // Division will never overflow
                let slash = all_reserved / BalanceOf::<T>::from(TARDY_PUNISHMENT_DIVISOR);
                let _ = CurrencyOf::<T>::repatriate_reserved_named(
                    &RESERVE_ID,
                    ai,
                    &treasury_account_id,
                    slash,
                    BalanceStatus::Free,
                )?;
                Self::remove_juror_from_all_courts_of_all_markets(ai);
                Ok::<_, DispatchError>(())
            };

            for (ai, juror, max_block, vote_opt) in requested_jurors {
                if let Some((block, outcome)) = vote_opt {
                    let vote_is_expired = block > max_block;
                    if vote_is_expired {
                        // Tardy juror voted after maximum allowed block. Slash
                        if let JurorStatus::Tardy = juror.status {
                            slash_and_remove_juror(ai)?;
                        }
                        // Ordinary juror voted after maximum allowed block. Set as tardy
                        else {
                            Self::set_stored_juror_as_tardy(ai)?;
                        }
                    } else {
                        let has_voted_on_the_second_most_outcome = cb(outcome);
                        if has_voted_on_the_second_most_outcome {
                            // Don't set already tardy juror as tardy again
                            if JurorStatus::Tardy != juror.status {
                                Self::set_stored_juror_as_tardy(ai)?;
                            }
                        } else {
                            valid_winners_and_losers.push((ai, outcome));
                        }
                    }
                // Tardy juror didn't vote. Slash
                } else if let JurorStatus::Tardy = juror.status {
                    slash_and_remove_juror(ai)?;
                }
                // Ordinary juror didn't vote. Set as tardy
                else {
                    Self::set_stored_juror_as_tardy(ai)?;
                }
            }

            Ok(valid_winners_and_losers)
        }

        // Modifies a stored juror.
        fn mutate_juror<F>(account_id: &T::AccountId, mut cb: F) -> DispatchResult
        where
            F: FnMut(&mut Juror) -> DispatchResult,
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

        // Calculates the necessary number of jurors depending on the number of market disputes.
        //
        // Result is capped to `usize::MAX` or in other words, capped to a very, very, very
        // high number of jurors.
        fn necessary_jurors_num(disputes: &[MarketDispute<T::AccountId, T::BlockNumber>]) -> usize {
            let len = disputes.len();
            INITIAL_JURORS_NUM.saturating_add(SUBSEQUENT_JURORS_FACTOR.saturating_mul(len))
        }

        // Every juror that not voted on the first or second most voted outcome are slashed.
        fn slash_losers_to_award_winners(
            valid_winners_and_losers: &[(&T::AccountId, &OutcomeReport)],
            winner_outcome: &OutcomeReport,
        ) -> DispatchResult {
            let mut total_incentives = BalanceOf::<T>::from(0u8);
            let mut total_winners = BalanceOf::<T>::from(0u8);

            for (jai, outcome) in valid_winners_and_losers {
                if outcome == &winner_outcome {
                    total_winners = total_winners.saturating_add(BalanceOf::<T>::from(1u8));
                } else {
                    let all_reserved = CurrencyOf::<T>::reserved_balance_named(&RESERVE_ID, jai);
                    // Division will never overflow
                    let slash = all_reserved / BalanceOf::<T>::from(2u8);
                    CurrencyOf::<T>::slash_reserved_named(&RESERVE_ID, jai, slash);
                    total_incentives = total_incentives.saturating_add(slash);
                }
            }

            let individual_winner_incentive =
                if let Some(e) = total_incentives.checked_div(&total_winners) {
                    e
                } else {
                    // No winners
                    return Ok(());
                };

            for (jai, outcome) in valid_winners_and_losers {
                if outcome == &winner_outcome {
                    CurrencyOf::<T>::deposit_into_existing(jai, individual_winner_incentive)?;
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

            let mut iter = scores.iter();

            let mut best_score = if let Some(first) = iter.next() {
                first
            } else {
                return Err(Error::<T>::NoVotes.into());
            };

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

        // Obliterates all stored references of a juror un-reserving balances.
        fn remove_juror_from_all_courts_of_all_markets(ai: &T::AccountId) {
            CurrencyOf::<T>::unreserve_all_named(&RESERVE_ID, ai);
            Jurors::<T>::remove(ai);
            let mut market_ids = BTreeSet::new();
            market_ids.extend(RequestedJurors::<T>::iter().map(|el| el.0));
            for market_id in &market_ids {
                RequestedJurors::<T>::remove(market_id, ai);
            }
            market_ids.clear();
            market_ids.extend(Votes::<T>::iter().map(|el| el.0));
            for market_id in &market_ids {
                Votes::<T>::remove(market_id, ai);
            }
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
            disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: &Self::MarketId,
            market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
        ) -> DispatchResult {
            if market.mdm != MarketDisputeMechanism::Court {
                return Err(Error::<T>::MarketDoesNotHaveCourtMechanism.into());
            }
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

        // Set jurors that sided on the second most voted outcome as tardy. Jurors are only
        // rewarded if sided on the most voted outcome but jurors that voted second most
        // voted outcome (winner of the losing majority) are placed as tardy instead of
        // being slashed.
        fn on_resolution(
            _: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: &Self::MarketId,
            market: &Market<Self::AccountId, Self::BlockNumber, MomentOf<T>>,
        ) -> Result<OutcomeReport, DispatchError> {
            if market.mdm != MarketDisputeMechanism::Court {
                return Err(Error::<T>::MarketDoesNotHaveCourtMechanism.into());
            }
            let votes: Vec<_> = Votes::<T>::iter_prefix(market_id).collect();
            let requested_jurors: Vec<_> = RequestedJurors::<T>::iter_prefix(market_id)
                .map(|(juror_id, max_allowed_block)| {
                    let juror = Self::juror(&juror_id)?;
                    let vote_opt = votes.iter().find(|el| el.0 == juror_id).map(|el| &el.1);
                    Ok((juror_id, juror, max_allowed_block, vote_opt))
                })
                .collect::<Result<_, DispatchError>>()?;
            let (first, second_opt) = Self::two_best_outcomes(&votes)?;
            let valid_winners_and_losers = if let Some(second) = second_opt {
                Self::manage_tardy_jurors(&requested_jurors, |outcome| outcome == &second)?
            } else {
                Self::manage_tardy_jurors(&requested_jurors, |_| false)?
            };
            Self::slash_losers_to_award_winners(&valid_winners_and_losers, &first)?;
            Votes::<T>::remove_prefix(market_id, None);
            RequestedJurors::<T>::remove_prefix(market_id, None);
            Ok(first)
        }
    }

    impl<T> CourtPalletApi for Pallet<T> where T: Config {}

    /// Accounts that stake funds to decide outcomes.
    #[pallet::storage]
    pub type Jurors<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Juror>;

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
