// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

// It is important to note that if a categorical market has only two outcomes, then winners
// won't receive any rewards because accounts of the most voted outcome on the loser side are
// simply registered as `JurorStatus::Tardy`.

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]

extern crate alloc;

mod benchmarks;
mod court_pallet_api;
pub mod migrations;
mod mock;
mod tests;
mod types;
pub mod weights;

pub use court_pallet_api::CourtPalletApi;
pub use pallet::*;
pub use types::*;

#[frame_support::pallet]
mod pallet {
    use crate::{
        weights::WeightInfoZeitgeist, CourtInfo, CourtPalletApi, CourtStatus, JurorInfo, Periods,
        Vote,
    };
    use alloc::{collections::BTreeMap, vec::Vec};
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        ensure, log,
        pallet_prelude::{EnsureOrigin, OptionQuery, StorageMap, StorageValue, ValueQuery},
        traits::{
            BalanceStatus, Currency, Get, Imbalance, IsType, NamedReservableCurrency, Randomness,
            StorageVersion,
        },
        transactional, Blake2_128Concat, BoundedVec, PalletId,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use rand::{rngs::StdRng, RngCore, SeedableRng};
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedDiv, Hash, Saturating, StaticLookup},
        DispatchError, Percent, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        traits::{DisputeApi, DisputeResolutionApi},
        types::{Asset, Market, MarketDisputeMechanism, MarketStatus, OutcomeReport},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The origin which may start appeals.
        type AppealOrigin: EnsureOrigin<Self::Origin>;

        /// The time to wait before jurors can start voting.
        /// The intention is to use this period as preparation time
        /// (for example vote outcome addition through crowdfunding)
        #[pallet::constant]
        type CourtBackingPeriod: Get<Self::BlockNumber>;

        /// The time in which the jurors can cast their secret vote.
        #[pallet::constant]
        type CourtVotePeriod: Get<Self::BlockNumber>;

        /// The time in which the jurors should reveal their secret vote.
        #[pallet::constant]
        type CourtAggregationPeriod: Get<Self::BlockNumber>;

        /// The time in which a court case can get appealed.
        #[pallet::constant]
        type CourtAppealPeriod: Get<Self::BlockNumber>;

        /// The slash percentage if a secret vote gets revealed during the voting period.
        #[pallet::constant]
        type DenounceSlashPercentage: Get<Percent>;

        /// The functionality to allow controlling the markets resolution time.
        type DisputeResolution: DisputeResolutionApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
            MarketId = MarketIdOf<Self>,
            Moment = MomentOf<Self>,
        >;

        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Market commons
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The maximum number of appeals until the court fails.
        #[pallet::constant]
        type MaxAppeals: Get<u32>;

        /// The maximum number of randomly selected jurors for a dispute.
        #[pallet::constant]
        type MaxDrawings: Get<u32>;

        /// The maximum number of jurors that can be registered.
        #[pallet::constant]
        type MaxJurors: Get<u32>;

        /// The minimum stake a user needs to reserve to become a juror.
        #[pallet::constant]
        type MinJurorStake: Get<BalanceOf<Self>>;

        /// Identifier of this pallet
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Randomness source
        type Random: Randomness<Self::Hash, Self::BlockNumber>;

        /// The percentage that is slashed if a juror did not vote for the plurality outcome.
        #[pallet::constant]
        type RedistributionPercentage: Get<Percent>;

        /// The percentage that is being slashed from the juror's stake.
        #[pallet::constant]
        type SlashPercentage: Get<Percent>;

        /// Slashed funds are send to the treasury
        #[pallet::constant]
        type TreasuryPalletId: Get<PalletId>;

        /// Weights generated by benchmarks
        type WeightInfo: WeightInfoZeitgeist;
    }

    // Number of jurors for an initial market dispute
    const INITIAL_JURORS_NUM: usize = 3;
    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);
    // Weight used to increase the number of jurors for subsequent appeals
    // of the same market
    const SUBSEQUENT_JURORS_FACTOR: usize = 2;

    pub(crate) type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;
    pub(crate) type NegativeImbalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub(crate) type MarketOf<T> = Market<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        <T as frame_system::Config>::BlockNumber,
        MomentOf<T>,
        Asset<MarketIdOf<T>>,
    >;
    pub(crate) type AccountIdLookupOf<T> =
        <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
    pub(crate) type CourtOf<T> = CourtInfo<<T as frame_system::Config>::BlockNumber>;
    pub(crate) type JurorInfoOf<T> = JurorInfo<BalanceOf<T>>;
    pub(crate) type JurorPoolOf<T> = BoundedVec<
        (BalanceOf<T>, <T as frame_system::Config>::AccountId),
        <T as Config>::MaxJurors,
    >;
    pub(crate) type DrawingsOf<T> = BoundedVec<
        (<T as frame_system::Config>::AccountId, Vote<<T as frame_system::Config>::Hash>),
        <T as Config>::MaxDrawings,
    >;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    /// The pool of jurors who can get randomly selected according to their stake.
    #[pallet::storage]
    pub type JurorPool<T: Config> = StorageValue<_, JurorPoolOf<T>, ValueQuery>;

    /// The general information about each juror.
    #[pallet::storage]
    pub type Jurors<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, JurorInfoOf<T>, OptionQuery>;

    /// An extra layer of pseudo randomness.
    #[pallet::storage]
    pub type JurorsSelectionNonce<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// The randomly selected jurors with their vote.
    #[pallet::storage]
    pub type Drawings<T: Config> =
        StorageMap<_, Blake2_128Concat, MarketIdOf<T>, DrawingsOf<T>, ValueQuery>;

    /// The general information about each court.
    #[pallet::storage]
    pub type Courts<T: Config> =
        StorageMap<_, Blake2_128Concat, MarketIdOf<T>, CourtOf<T>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A juror has been added to the court.
        JoinedJuror { juror: T::AccountId },
        /// A juror prepared to exit the court.
        JurorPreparedExit { juror: T::AccountId },
        /// A juror has been removed from the court.
        ExitedJuror { juror: T::AccountId },
        /// A juror has voted in a court.
        JurorVoted { market_id: MarketIdOf<T>, juror: T::AccountId, secret: T::Hash },
        /// A juror has revealed their vote.
        JurorRevealedVote {
            juror: T::AccountId,
            market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
            salt: T::Hash,
        },
        /// A juror vote has been denounced.
        DenouncedJurorVote {
            denouncer: T::AccountId,
            juror: T::AccountId,
            market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
            salt: T::Hash,
        },
        /// The jurors for an appeal have been drawn.
        AppealJurorsDrawn { market_id: MarketIdOf<T> },
        /// The backing for an appeal has been checked.
        AppealBackingChecked { market_id: MarketIdOf<T> },
        /// A market has been appealed.
        MarketAppealed { market_id: MarketIdOf<T>, appeal_number: u8 },
        /// The juror stakes have been reassigned.
        JurorStakesReassigned { market_id: MarketIdOf<T> },
        /// The tardy jurors have been punished.
        TardyJurorsPunished { market_id: MarketIdOf<T> },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// An account id does not exist on the jurors storage.
        JurorDoesNotExists,
        /// On dispute or resolution, someone tried to pass a non-court market type
        MarketDoesNotHaveCourtMechanism,
        /// No-one voted on an outcome to resolve a market
        NoVotes,
        /// The market is not in a state where it can be disputed.
        MarketIsNotDisputed,
        /// Only jurors can reveal their votes.
        OnlyJurorsCanReveal,
        /// The vote is not secret.
        VoteAlreadyRevealed,
        /// The outcome and salt reveal do not match the secret vote.
        InvalidReveal,
        /// No court for this market id was found.
        CourtNotFound,
        /// This operation is only allowed in the voting period.
        NotInVotingPeriod,
        /// This operation is only allowed in the aggregation period.
        NotInAggregationPeriod,
        /// The maximum number of appeals has been reached.
        MaxAppealsReached,
        /// This operation is only allowed in the appeal period.
        NotInAppealPeriod,
        /// The court is already present for this market.
        CourtAlreadyExists,
        /// For this appeal round the random juror selection extrinsic was already called.
        JurorsAlreadyDrawn,
        /// For this appeal round the backing check extrinsic was already called.
        AppealAlreadyBacked,
        /// In order to start an appeal the backing check extrinsic must be called first.
        CheckBackingFirst,
        /// The final appeal extrinsic can only be called after the backing check extrinsic
        /// and random selection of jurors for this appeal.
        AppealNotReady,
        /// The caller of this extrinsic must be a randomly selected juror.
        OnlyDrawnJurorsCanVote,
        /// The amount is below the minimum required stake.
        BelowMinJurorStake,
        /// The maximum number of possible jurors has been reached.
        MaxJurorsReached,
        /// In order to exit the court the juror must not be randomly selected in an active appeal.
        JurorStillDrawn,
        /// In order to exit the court the juror has to exit
        /// the pool first with `prepare_exit_court`.
        JurorNotPreparedToExit,
        /// The juror was not found in the pool. This happens after `prepare_exit_court`
        /// or if the juror did not vote for plurality decisions.
        /// The juror needs to exit the court and then rejoin.
        JurorNeedsToExit,
        /// The juror was not randomly selected for the court.
        JurorNotDrawn,
        /// The juror was drawn but did not manage to secretly vote within the court.
        JurorNotVoted,
        /// The juror was already denounced. This action can only happen once.
        VoteAlreadyDenounced,
        /// A juror tried to denounce herself.
        SelfDenounceDisallowed,
        /// In order to execute the binary search efficiently
        /// the join amount must be unqiue for each juror.
        AmountAlreadyUsed,
        /// The court is not in the closed state.
        CourtNotClosed,
        /// The jurors were already reassigned.
        JurorsAlreadyReassigned,
        /// The tardy jurors were already punished.
        TardyJurorsAlreadyPunished,
        /// Punish the tardy jurors first.
        PunishTardyJurorsFirst,
        /// There are not enough jurors in the pool.
        NotEnoughJurors,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Join to become a juror, who is able to get randomly selected
        /// for court cases according to the provided stake.
        /// The probability to get selected is higher the more funds are staked.
        ///
        /// # Arguments
        ///
        /// - `amount`: The amount associated with the joining juror.
        ///
        /// # Weight
        ///
        /// Complexity: `O(log(n))`, where `n` is the number of jurors in the stake-weighted pool.
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn join_court(origin: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            ensure!(amount >= T::MinJurorStake::get(), Error::<T>::BelowMinJurorStake);

            let mut jurors = JurorPool::<T>::get();

            let mut juror_info = JurorInfoOf::<T> { stake: amount };

            if let Some(prev_juror_info) = <Jurors<T>>::get(&who) {
                if let Ok(i) = jurors.binary_search_by_key(&prev_juror_info.stake, |tuple| tuple.0)
                {
                    jurors.remove(i);
                } else {
                    // this happens if the juror behaved incorrectly
                    // (was denounced, did not reveal, did not vote)
                    // or if `prepare_exit_court` was called
                    return Err(Error::<T>::JurorNeedsToExit.into());
                }

                juror_info.stake = juror_info.stake.saturating_add(prev_juror_info.stake);
            }

            match jurors.binary_search_by_key(&juror_info.stake, |tuple| tuple.0) {
                // The reason for this error is that each amount has a clear juror
                // binary_search_by_key could otherwise return an index of an unwanted juror
                // if there are multiple jurors with the same stake
                Ok(_) => return Err(Error::<T>::AmountAlreadyUsed.into()),
                Err(i) => jurors
                    .try_insert(i, (juror_info.stake, who.clone()))
                    .map_err(|_| Error::<T>::MaxJurorsReached)?,
            };

            // full reserve = prev_juror_info.stake (already reserved) + amount
            CurrencyOf::<T>::reserve_named(&Self::reserve_id(), &who, amount)?;

            JurorPool::<T>::put(jurors);

            <Jurors<T>>::insert(&who, juror_info);

            Self::deposit_event(Event::JoinedJuror { juror: who });
            Ok(())
        }

        /// Prepare as a juror to exit the court.
        /// For this the juror has to be removed from the stake weighted pool first before the exit.
        ///
        /// # Weight
        ///
        /// Complexity: `O(log(n))`, where `n` is the number of jurors in the stake-weighted pool.
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn prepare_exit_court(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let prev_juror_info = <Jurors<T>>::get(&who).ok_or(Error::<T>::JurorDoesNotExists)?;

            let mut jurors = JurorPool::<T>::get();

            if let Ok(i) = jurors.binary_search_by_key(&prev_juror_info.stake, |tuple| tuple.0) {
                // remove from juror list to prevent being drawn
                jurors.remove(i);
                <JurorPool<T>>::put(jurors);
            } else {
                // this happens if the juror was slashed by the vote aggregation
                return Err(Error::<T>::JurorNeedsToExit.into());
            }

            Self::deposit_event(Event::JurorPreparedExit { juror: who });
            Ok(())
        }

        /// Remove a juror from all courts.
        /// This is only possible if the juror is not part of the pool anymore
        /// (with `prepare_exit_court` or was denounced, did not reveal, did not vote)
        /// and the juror is not bonded in active courts anymore.
        ///
        /// # Arguments
        ///
        /// - `juror`: The juror, who is assumed to be not be part of the pool anymore.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n * m)`, where `n` is the number of markets
        /// which have active random selections in place, and `m` is the number of jurors
        /// randomly selected for each market.
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn exit_court(origin: OriginFor<T>, juror: AccountIdLookupOf<T>) -> DispatchResult {
            ensure_signed(origin)?;

            let juror = T::Lookup::lookup(juror)?;

            let prev_juror_info = <Jurors<T>>::get(&juror).ok_or(Error::<T>::JurorDoesNotExists)?;

            ensure!(
                JurorPool::<T>::get()
                    .binary_search_by_key(&prev_juror_info.stake, |tuple| tuple.0)
                    .is_err(),
                Error::<T>::JurorNotPreparedToExit
            );

            // ensure not drawn for any market
            for (_, drawings) in <Drawings<T>>::iter() {
                ensure!(!drawings.iter().any(|(j, _)| j == &juror), Error::<T>::JurorStillDrawn);
            }

            Jurors::<T>::remove(&juror);

            CurrencyOf::<T>::unreserve_all_named(&Self::reserve_id(), &juror);

            Self::deposit_event(Event::ExitedJuror { juror });
            Ok(())
        }

        /// Vote as a randomly selected juror for a specific court case.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The identifier of the court.
        /// - `secret_vote`: A hash which consists of `juror ++ outcome ++ salt`.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of jurors
        /// in the list of random selections (drawings).
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn vote(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            secret_vote: T::Hash,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(
                court.periods.backing_end < now && now <= court.periods.vote_end,
                Error::<T>::NotInVotingPeriod
            );

            let mut drawings = <Drawings<T>>::get(market_id);
            match drawings.iter().position(|(juror, _)| juror == &who) {
                Some(index) => {
                    let vote = Vote::Secret { secret: secret_vote };
                    drawings[index] = (who.clone(), vote);
                }
                None => return Err(Error::<T>::OnlyDrawnJurorsCanVote.into()),
            }

            <Drawings<T>>::insert(market_id, drawings);

            Self::deposit_event(Event::JurorVoted { juror: who, market_id, secret: secret_vote });
            Ok(())
        }

        /// Denounce a juror during the voting period for which the secret vote is known.
        /// This is useful to punish the behaviour that jurors reveal
        /// their secrets before the voting period ends.
        /// A check of `secret_hash == hash(juror ++ outcome ++ salt)` is performed for validation.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The identifier of the court.
        /// - `juror`: The juror whose secret vote might be known.
        /// - `outcome`: The raw vote outcome which should match with the secret of the juror.
        /// - `salt`: The hash which is used to proof that the juror did reveal
        /// her vote during the voting period.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of jurors
        /// in the list of random selections (drawings).
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn denounce_vote(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            juror: AccountIdLookupOf<T>,
            outcome: OutcomeReport,
            salt: T::Hash,
        ) -> DispatchResult {
            let denouncer = ensure_signed(origin)?;

            let juror = T::Lookup::lookup(juror)?;

            ensure!(denouncer != juror, Error::<T>::SelfDenounceDisallowed);

            let prev_juror_info = <Jurors<T>>::get(&juror).ok_or(Error::<T>::JurorDoesNotExists)?;

            let court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            let now = <frame_system::Pallet<T>>::block_number();
            // ensure in vote period
            ensure!(
                court.periods.backing_end < now && now <= court.periods.vote_end,
                Error::<T>::NotInVotingPeriod
            );

            let mut drawings = <Drawings<T>>::get(market_id);
            let (index, vote) = match drawings.iter().position(|(j, _)| j == &juror) {
                Some(index) => (index, drawings[index].1.clone()),
                None => return Err(Error::<T>::JurorNotDrawn.into()),
            };

            let secret = match vote {
                Vote::Secret { secret } => {
                    ensure!(
                        secret == T::Hashing::hash_of(&(juror.clone(), outcome.clone(), salt)),
                        Error::<T>::InvalidReveal
                    );
                    secret
                }
                Vote::Drawn => return Err(Error::<T>::JurorNotVoted.into()),
                Vote::Revealed { secret: _, outcome: _, salt: _ } => {
                    return Err(Error::<T>::VoteAlreadyRevealed.into());
                }
                Vote::Denounced { secret: _, outcome: _, salt: _ } => {
                    return Err(Error::<T>::VoteAlreadyDenounced.into());
                }
            };

            let treasury_account_id = Self::treasury_account_id();
            let all_reserved = CurrencyOf::<T>::reserved_balance_named(&Self::reserve_id(), &juror);
            let slash = T::DenounceSlashPercentage::get() * all_reserved;
            let _ = CurrencyOf::<T>::repatriate_reserved_named(
                &Self::reserve_id(),
                &juror,
                &treasury_account_id,
                slash,
                BalanceStatus::Free,
            )?;

            let mut jurors = JurorPool::<T>::get();
            if let Ok(i) = jurors.binary_search_by_key(&prev_juror_info.stake, |tuple| tuple.0) {
                // remove from juror list to prevent being drawn
                jurors.remove(i);
                <JurorPool<T>>::put(jurors);
            }

            let raw_vote = Vote::Denounced { secret, outcome: outcome.clone(), salt };
            drawings[index] = (juror.clone(), raw_vote);
            <Drawings<T>>::insert(market_id, drawings);

            Self::deposit_event(Event::DenouncedJurorVote {
                denouncer,
                juror,
                market_id,
                outcome,
                salt,
            });
            Ok(())
        }

        /// Reveal the secret vote of the caller juror.
        /// A check of `secret_hash == hash(juror ++ outcome ++ salt)` is performed for validation.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The identifier of the court.
        /// - `outcome`: The raw vote outcome which should match with the secret of the juror.
        /// - `salt`: The hash which is used for the validation.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of jurors
        /// in the list of random selections (drawings).
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn reveal_vote(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
            salt: T::Hash,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(<Jurors<T>>::get(&who).is_some(), Error::<T>::OnlyJurorsCanReveal);
            let court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(
                court.periods.vote_end < now && now <= court.periods.aggregation_end,
                Error::<T>::NotInAggregationPeriod
            );

            let mut drawings = <Drawings<T>>::get(market_id);
            let (index, vote) = match drawings.iter().position(|(juror, _)| juror == &who) {
                Some(index) => (index, drawings[index].1.clone()),
                None => return Err(Error::<T>::JurorNotDrawn.into()),
            };

            let secret = match vote {
                Vote::Secret { secret } => {
                    // market id and current appeal number is part of salt generation
                    // salt should be signed by the juror (market_id ++ appeal number)
                    // salt can be reproduced only be the juror address
                    // with knowing market_id and appeal number
                    // so even if the salt is forgotten it can be reproduced only by the juror
                    ensure!(
                        secret == T::Hashing::hash_of(&(who.clone(), outcome.clone(), salt)),
                        Error::<T>::InvalidReveal
                    );
                    secret
                }
                Vote::Drawn => return Err(Error::<T>::JurorNotVoted.into()),
                Vote::Revealed { secret: _, outcome: _, salt: _ } => {
                    return Err(Error::<T>::VoteAlreadyRevealed.into());
                }
                Vote::Denounced { secret: _, outcome: _, salt: _ } => {
                    return Err(Error::<T>::VoteAlreadyDenounced.into());
                }
            };

            let raw_vote = Vote::Revealed { secret, outcome: outcome.clone(), salt };
            drawings[index] = (who.clone(), raw_vote);
            <Drawings<T>>::insert(market_id, drawings);

            Self::deposit_event(Event::JurorRevealedVote { juror: who, market_id, outcome, salt });
            Ok(())
        }

        /// Check if the appeal of a court is allowed to get initiated.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The identifier of the court.
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn check_appeal_backing(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
        ) -> DispatchResult {
            T::AppealOrigin::ensure_origin(origin)?;

            let mut court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            ensure!(!court.appeal_info.is_backed, Error::<T>::AppealAlreadyBacked);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::check_appealable_market(&market_id, &court, now)?;

            court.appeal_info.is_backed = true;
            <Courts<T>>::insert(market_id, court);

            Self::deposit_event(Event::AppealBackingChecked { market_id });

            Ok(())
        }

        /// Randomly select jurors from the pool according to their stake
        /// for the coming appeal round.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The identifier of the court.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` depends on `choose_multiple_weighted` of `select_jurors`.
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn draw_appeal_jurors(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let mut court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            ensure!(!court.appeal_info.is_drawn, Error::<T>::JurorsAlreadyDrawn);
            ensure!(court.appeal_info.is_backed, Error::<T>::CheckBackingFirst);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::check_appealable_market(&market_id, &court, now)?;

            let last_resolve_at = court.periods.appeal_end;

            let appeal_number = court.appeal_info.current as usize;
            let jurors: JurorPoolOf<T> = JurorPool::<T>::get();
            ensure!(jurors.len() >= INITIAL_JURORS_NUM, Error::<T>::NotEnoughJurors);
            Self::select_jurors(&market_id, jurors.as_slice(), appeal_number);
            // at the time of flushing the last drawings in `select_jurors`
            // we want to avoid the resolution before the full appeal is executed
            // otherwise `draw_appeal_jurors` would replace all votes with `Vote::Drawn`
            // So, the appeal is inevitable after the call to this extrinsic
            // otherwise the market is not going to resolve
            let _ids_len_0 = T::DisputeResolution::remove_auto_resolve(&market_id, last_resolve_at);

            court.appeal_info.is_drawn = true;
            <Courts<T>>::insert(market_id, court);

            Self::deposit_event(Event::AppealJurorsDrawn { market_id });

            Ok(())
        }

        /// Trigger an appeal for a court.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The identifier of the court.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of market ids
        /// inside the dispute resolution list.
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn appeal(origin: OriginFor<T>, market_id: MarketIdOf<T>) -> DispatchResult {
            ensure_signed(origin)?;

            let mut court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            ensure!(court.appeal_info.is_appeal_ready(), Error::<T>::AppealNotReady);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::check_appealable_market(&market_id, &court, now)?;

            let periods = Periods {
                backing_end: T::CourtBackingPeriod::get(),
                vote_end: T::CourtVotePeriod::get(),
                aggregation_end: T::CourtAggregationPeriod::get(),
                appeal_end: T::CourtAppealPeriod::get(),
            };
            // sets periods one after the other from now
            court.update_periods(periods, now);
            let appeal_number = court.appeal_info.current;
            court.appeal_info.current = court.appeal_info.current.saturating_add(1);

            let _ids_len_1 =
                T::DisputeResolution::add_auto_resolve(&market_id, court.periods.appeal_end)?;

            <Courts<T>>::insert(market_id, court);

            Self::deposit_event(Event::MarketAppealed { market_id, appeal_number });

            Ok(())
        }

        /// After the court is closed (resolution happened), the tardy jurors can get punished.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The identifier of the court.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of randomly selected jurors for this court.
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn punish_tardy_jurors(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let mut court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            let winner = match court.status {
                CourtStatus::Closed { winner, punished, reassigned: _ } => {
                    ensure!(!punished, Error::<T>::TardyJurorsAlreadyPunished);
                    winner
                }
                _ => return Err(Error::<T>::CourtNotClosed.into()),
            };

            let treasury_account_id = Self::treasury_account_id();

            let mut jurors = JurorPool::<T>::get();
            let mut slash_and_remove_juror = |ai: &T::AccountId| {
                let all_reserved = CurrencyOf::<T>::reserved_balance_named(&Self::reserve_id(), ai);
                let slash = T::SlashPercentage::get() * all_reserved;
                let res = CurrencyOf::<T>::repatriate_reserved_named(
                    &Self::reserve_id(),
                    ai,
                    &treasury_account_id,
                    slash,
                    BalanceStatus::Free,
                );
                if let Err(e) = res {
                    log::warn!(
                        "Failed to slash juror {:?} for market {:?}: {:?}",
                        ai,
                        market_id,
                        e
                    );
                    debug_assert!(false);
                }

                if let Some(prev_juror_info) = <Jurors<T>>::get(ai) {
                    if let Ok(i) =
                        jurors.binary_search_by_key(&prev_juror_info.stake, |tuple| tuple.0)
                    {
                        // remove from juror list to prevent being drawn
                        jurors.remove(i);
                    }
                } else {
                    log::warn!("Juror {:?} not found in Jurors storage for vote aggregation.", ai);
                    debug_assert!(false);
                }
            };

            for (juror, vote) in Drawings::<T>::get(market_id).iter() {
                match vote {
                    Vote::Drawn => {
                        slash_and_remove_juror(juror);
                    }
                    Vote::Secret { secret: _ } => {
                        slash_and_remove_juror(juror);
                    }
                    // denounce extrinsic already punished the juror
                    Vote::Denounced { secret: _, outcome: _, salt: _ } => (),
                    Vote::Revealed { secret: _, outcome: _, salt: _ } => (),
                }
            }

            court.status = CourtStatus::Closed { winner, reassigned: false, punished: true };
            <Courts<T>>::insert(market_id, court);
            <JurorPool<T>>::put(jurors);

            Self::deposit_event(Event::TardyJurorsPunished { market_id });

            Ok(())
        }

        /// After the court is closed (resolution happened) and the tardy jurors have been punished,
        /// the juror stakes can get reassigned.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The identifier of the court.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of randomly selected jurors for this court.
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn reassign_juror_stakes(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let mut court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            let winner = match court.status {
                CourtStatus::Closed { winner, punished, reassigned } => {
                    ensure!(!reassigned, Error::<T>::JurorsAlreadyReassigned);
                    ensure!(punished, Error::<T>::PunishTardyJurorsFirst);
                    winner
                }
                _ => return Err(Error::<T>::CourtNotClosed.into()),
            };

            let drawings = Drawings::<T>::get(market_id);

            let mut valid_winners_and_losers = Vec::with_capacity(drawings.len());

            for (juror, vote) in drawings {
                if let Vote::Revealed { secret: _, outcome, salt: _ } = vote {
                    valid_winners_and_losers.push((juror, outcome));
                }
            }

            Self::slash_losers_to_award_winners(valid_winners_and_losers.as_slice(), &winner)?;

            court.status = CourtStatus::Closed { winner, punished: true, reassigned: true };
            <Courts<T>>::insert(market_id, court);
            <Drawings<T>>::remove(market_id);

            Self::deposit_event(Event::JurorStakesReassigned { market_id });

            Ok(())
        }
    }

    impl<T> Pallet<T>
    where
        T: Config,
    {
        pub(crate) fn choose_multiple_weighted<R: RngCore>(
            market_id: &MarketIdOf<T>,
            jurors: &[(BalanceOf<T>, T::AccountId)],
            number: usize,
            rng: &mut R,
        ) -> Vec<(T::AccountId, Vote<T::Hash>)> {
            use rand::{
                distributions::{Distribution, WeightedIndex},
                seq::SliceRandom,
            };

            let mut selected = Vec::with_capacity(number);

            let res = WeightedIndex::new(jurors.iter().map(|item| item.0.saturated_into::<u128>()));

            match res {
                Ok(distribution) => {
                    for _ in 0..number {
                        selected.push((jurors[distribution.sample(rng)].1.clone(), Vote::Drawn));
                    }
                }
                Err(err) => {
                    // this can also happen when there are no jurors
                    log::warn!(
                        "Court: weighted selection failed, falling back to random selection for \
                         market {:?} with error: {:?}.",
                        market_id,
                        err
                    );
                    // fallback to random selection if weighted selection fails
                    jurors.choose_multiple(rng, number).for_each(|item| {
                        selected.push((item.1.clone(), Vote::Drawn));
                    });
                }
            }

            selected
        }

        pub(crate) fn select_jurors(
            market_id: &MarketIdOf<T>,
            jurors: &[(BalanceOf<T>, T::AccountId)],
            appeal_number: usize,
        ) {
            let necessary_jurors_num = Self::necessary_jurors_num(appeal_number);
            let mut rng = Self::rng();
            let actual_len = jurors.len().min(necessary_jurors_num);

            let random_jurors =
                Self::choose_multiple_weighted(market_id, jurors, actual_len, &mut rng);

            // we allow at most MaxDrawings jurors
            // look at `necessary_jurors_num`: MaxAppeals (= 5) example: 2^5 * 3 + 2^5 - 1 = 127
            // MaxDrawings should be 127 in this case
            let drawings = <DrawingsOf<T>>::truncate_from(random_jurors);
            // new appeal round should have a fresh set of drawings
            <Drawings<T>>::insert(market_id, drawings);
        }

        pub(crate) fn check_appealable_market(
            market_id: &MarketIdOf<T>,
            court: &CourtOf<T>,
            now: T::BlockNumber,
        ) -> Result<(), DispatchError> {
            let market = T::MarketCommons::market(market_id)?;
            ensure!(market.status == MarketStatus::Disputed, Error::<T>::MarketIsNotDisputed);
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            ensure!(
                court.periods.aggregation_end < now && now <= court.periods.appeal_end,
                Error::<T>::NotInAppealPeriod
            );

            ensure!(
                court.appeal_info.current < court.appeal_info.max,
                Error::<T>::MaxAppealsReached
            );

            Ok(())
        }

        /// The reserve ID of the court pallet.
        #[inline]
        pub fn reserve_id() -> [u8; 8] {
            T::PalletId::get().0
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

        #[inline]
        pub(crate) fn treasury_account_id() -> T::AccountId {
            T::TreasuryPalletId::get().into_account_truncating()
        }

        // Calculates the necessary number of jurors depending on the number of market appeals.
        fn necessary_jurors_num(appeals_len: usize) -> usize {
            // 2^(appeals_len) * 3 + 2^(appeals_len) - 1
            // MaxAppeals (= 5) example: 2^5 * 3 + 2^5 - 1 = 127
            SUBSEQUENT_JURORS_FACTOR
                .saturating_pow(appeals_len as u32)
                .saturating_mul(INITIAL_JURORS_NUM)
                .saturating_add(
                    SUBSEQUENT_JURORS_FACTOR.saturating_pow(appeals_len as u32).saturating_sub(1),
                )
        }

        fn slash_losers_to_award_winners(
            valid_winners_and_losers: &[(T::AccountId, OutcomeReport)],
            winner_outcome: &OutcomeReport,
        ) -> DispatchResult {
            let mut total_incentives = <NegativeImbalanceOf<T>>::zero();

            let mut winners = Vec::with_capacity(valid_winners_and_losers.len());
            for (juror, outcome) in valid_winners_and_losers {
                if outcome == winner_outcome {
                    winners.push(juror);
                } else {
                    let all_reserved =
                        CurrencyOf::<T>::reserved_balance_named(&Self::reserve_id(), juror);
                    let slash = T::RedistributionPercentage::get() * all_reserved;
                    let (imb, _excess) =
                        CurrencyOf::<T>::slash_reserved_named(&Self::reserve_id(), juror, slash);
                    total_incentives.subsume(imb);
                }
            }

            if let Some(reward_per_each) =
                total_incentives.peek().checked_div(&winners.len().saturated_into())
            {
                for juror in winners {
                    let (actual_reward, leftover) = total_incentives.split(reward_per_each);
                    total_incentives = leftover;
                    CurrencyOf::<T>::resolve_creating(juror, actual_reward);
                }
            } else {
                // if there are no winners reward the treasury
                let treasury_acc = Self::treasury_account_id();
                CurrencyOf::<T>::resolve_creating(&treasury_acc, total_incentives);
            }

            Ok(())
        }

        fn get_winner(
            votes: &[(T::AccountId, Vote<T::Hash>)],
        ) -> Result<OutcomeReport, DispatchError> {
            let mut scores = BTreeMap::<OutcomeReport, u32>::new();

            for (_juror, vote) in votes {
                if let Vote::Revealed { secret: _, outcome, salt: _ } = vote {
                    if let Some(el) = scores.get_mut(outcome) {
                        *el = el.saturating_add(1);
                    } else {
                        scores.insert(outcome.clone(), 1);
                    }
                }
            }

            let mut iter = scores.iter();

            let mut best_score = if let Some(first) = iter.next() {
                first
            } else {
                // TODO(#980): Create another voting round
                return Err(Error::<T>::NoVotes.into());
            };

            for el in iter {
                if el.1 > best_score.1 {
                    best_score = el;
                }
            }

            Ok(best_score.0.clone())
        }
    }

    impl<T> DisputeApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type Balance = BalanceOf<T>;
        type NegativeImbalance = NegativeImbalanceOf<T>;
        type BlockNumber = T::BlockNumber;
        type MarketId = MarketIdOf<T>;
        type Moment = MomentOf<T>;
        type Origin = T::Origin;

        fn on_dispute(market_id: &Self::MarketId, market: &MarketOf<T>) -> DispatchResult {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            ensure!(!<Courts<T>>::contains_key(market_id), Error::<T>::CourtAlreadyExists);

            let jurors: JurorPoolOf<T> = JurorPool::<T>::get();
            ensure!(jurors.len() >= INITIAL_JURORS_NUM, Error::<T>::NotEnoughJurors);
            Self::select_jurors(market_id, jurors.as_slice(), 0usize);

            let now = <frame_system::Pallet<T>>::block_number();

            let periods = Periods {
                backing_end: T::CourtBackingPeriod::get(),
                vote_end: T::CourtVotePeriod::get(),
                aggregation_end: T::CourtAggregationPeriod::get(),
                appeal_end: T::CourtAppealPeriod::get(),
            };

            // sets periods one after the other from now
            let court = CourtInfo::new(now, periods, T::MaxAppeals::get() as u8);

            let _ids_len =
                T::DisputeResolution::add_auto_resolve(market_id, court.periods.appeal_end)?;

            <Courts<T>>::insert(market_id, court);

            Ok(())
        }

        fn get_resolution_outcome(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Option<OutcomeReport>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            let mut court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;

            let drawings = Drawings::<T>::get(market_id);
            let winner_outcome = Self::get_winner(drawings.as_slice())?;

            court.status = CourtStatus::Closed {
                winner: winner_outcome.clone(),
                punished: false,
                reassigned: false,
            };
            <Courts<T>>::insert(market_id, court);

            Ok(Some(winner_outcome))
        }

        fn maybe_pay(
            _: &Self::MarketId,
            market: &MarketOf<T>,
            _: &OutcomeReport,
            overall_imbalance: NegativeImbalanceOf<T>,
        ) -> Result<NegativeImbalanceOf<T>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );
            // TODO all funds to treasury?
            Ok(overall_imbalance)
        }

        fn get_auto_resolve(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Option<Self::BlockNumber>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            let court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            Ok(Some(court.periods.appeal_end))
        }

        fn has_failed(
            _market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<bool, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            // TODO: for now disallow global dispute for court, later use max appeals check
            Ok(false)
        }

        fn on_global_dispute(_: &Self::MarketId, market: &MarketOf<T>) -> DispatchResult {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );
            Ok(())
        }

        fn clear(market_id: &Self::MarketId, market: &MarketOf<T>) -> DispatchResult {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );
            <Drawings<T>>::remove(market_id);
            <Courts<T>>::remove(market_id);
            Ok(())
        }
    }

    impl<T> CourtPalletApi for Pallet<T> where T: Config {}
}
