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
        weights::WeightInfoZeitgeist, AppealInfo, CourtInfo, CourtPalletApi, CourtStatus, Draw,
        JurorInfo, JurorPoolItem, Periods, Vote,
    };
    use alloc::{
        collections::{BTreeMap, BTreeSet},
        vec::Vec,
    };
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        ensure, log,
        pallet_prelude::{OptionQuery, StorageMap, StorageValue, ValueQuery},
        traits::{
            Currency, Get, Imbalance, IsType, LockIdentifier, LockableCurrency,
            NamedReservableCurrency, Randomness, StorageVersion, WithdrawReasons,
        },
        transactional, Blake2_128Concat, BoundedVec, PalletId,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use rand::{rngs::StdRng, Rng, RngCore, SeedableRng};
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedDiv, Hash, Saturating, StaticLookup, Zero},
        DispatchError, Percent, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        traits::{DisputeApi, DisputeResolutionApi},
        types::{Asset, Market, MarketDisputeMechanism, MarketStatus, OutcomeReport},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The required base bond in order to get an appeal initiated.
        #[pallet::constant]
        type AppealBond: Get<BalanceOf<Self>>;

        /// The additional amount of currency that must be bonded when creating a subsequent
        /// appeal.
        #[pallet::constant]
        type AppealBondFactor: Get<BalanceOf<Self>>;

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

        /// The court lock identifier.
        #[pallet::constant]
        type CourtLockId: Get<LockIdentifier>;

        /// Identifier of this pallet
        #[pallet::constant]
        type CourtPalletId: Get<PalletId>;

        /// The currency implementation used to transfer, lock and reserve tokens.
        type Currency: Currency<Self::AccountId>
            + NamedReservableCurrency<Self::AccountId, ReserveIdentifier = [u8; 8]>
            + LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

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
        type MaxDraws: Get<u32>;

        /// The maximum number of jurors that can be registered.
        #[pallet::constant]
        type MaxJurors: Get<u32>;

        /// The minimum stake a user needs to reserve to become a juror.
        #[pallet::constant]
        type MinJurorStake: Get<BalanceOf<Self>>;

        /// Randomness source
        type Random: Randomness<Self::Hash, Self::BlockNumber>;

        /// The percentage that is slashed if a juror did not vote for the plurality outcome.
        #[pallet::constant]
        type RedistributionPercentage: Get<Percent>;

        /// The percentage that is being slashed from the juror's stake in case she is tardy.
        #[pallet::constant]
        type TardySlashPercentage: Get<Percent>;

        /// Slashed funds are send to the treasury
        #[pallet::constant]
        type TreasuryPalletId: Get<PalletId>;

        /// Weights generated by benchmarks
        type WeightInfo: WeightInfoZeitgeist;
    }

    // Number of jurors for an initial market dispute
    const INITIAL_JURORS_NUM: usize = 5;
    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);
    // Weight used to increase the number of jurors for subsequent appeals
    // of the same market
    const SUBSEQUENT_JURORS_FACTOR: usize = 2;

    pub(crate) type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> = <T as Config>::Currency;
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
    pub(crate) type CourtOf<T> = CourtInfo<<T as frame_system::Config>::BlockNumber, AppealsOf<T>>;
    pub(crate) type JurorInfoOf<T> = JurorInfo<BalanceOf<T>>;
    pub(crate) type JurorPoolItemOf<T> =
        JurorPoolItem<<T as frame_system::Config>::AccountId, BalanceOf<T>>;
    pub(crate) type JurorPoolOf<T> = BoundedVec<JurorPoolItemOf<T>, <T as Config>::MaxJurors>;
    pub(crate) type DrawOf<T> = Draw<
        <T as frame_system::Config>::AccountId,
        BalanceOf<T>,
        <T as frame_system::Config>::Hash,
    >;
    pub(crate) type DrawsOf<T> = BoundedVec<DrawOf<T>, <T as Config>::MaxDraws>;
    pub(crate) type AppealOf<T> = AppealInfo<<T as frame_system::Config>::AccountId, BalanceOf<T>>;
    pub(crate) type AppealsOf<T> = BoundedVec<AppealOf<T>, <T as Config>::MaxAppeals>;

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
    pub type Draws<T: Config> =
        StorageMap<_, Blake2_128Concat, MarketIdOf<T>, DrawsOf<T>, ValueQuery>;

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
        AppealBacked { market_id: MarketIdOf<T> },
        /// A market has been appealed.
        MarketAppealed { market_id: MarketIdOf<T>, appeal_number: u32 },
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
        BackAppealFirst,
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
        /// The report of the market was not found.
        MarketReportNotFound,
        /// The caller has not enough funds to join the court with the specified amount.
        InsufficientAmount,
        /// After the first join of the court the amount has to be higher than the last join.
        AmountBelowLastJoin,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Join to become a juror, who is able to get randomly selected
        /// for court cases according to the provided stake.
        /// The probability to get selected is higher the more funds are staked.
        /// The amount is added to the stake-weighted pool.
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
            let free_balance = T::Currency::free_balance(&who);
            ensure!(amount <= free_balance, Error::<T>::InsufficientAmount);

            let mut jurors = JurorPool::<T>::get();

            if let Some(prev_juror_info) = <Jurors<T>>::get(&who) {
                ensure!(amount > prev_juror_info.stake, Error::<T>::AmountBelowLastJoin);
                if let Ok(i) =
                    jurors.binary_search_by_key(&prev_juror_info.stake, |pool_item| pool_item.stake)
                {
                    jurors.remove(i);
                } else {
                    // this happens if the juror behaved incorrectly
                    // (was denounced, did not reveal, did not vote)
                    // or if `prepare_exit_court` was called
                    return Err(Error::<T>::JurorNeedsToExit.into());
                }
            }

            match jurors.binary_search_by_key(&amount, |pool_item| pool_item.stake) {
                // The reason for this error is that each amount has a clear juror
                // binary_search_by_key could otherwise return an index of an unwanted juror
                // if there are multiple jurors with the same stake
                Ok(_) => return Err(Error::<T>::AmountAlreadyUsed.into()),
                Err(i) => jurors
                    .try_insert(
                        i,
                        JurorPoolItem {
                            stake: amount,
                            juror: who.clone(),
                            slashed: <BalanceOf<T>>::zero(),
                        },
                    )
                    .map_err(|_| Error::<T>::MaxJurorsReached)?,
            };

            T::Currency::set_lock(T::CourtLockId::get(), &who, amount, WithdrawReasons::all());

            JurorPool::<T>::put(jurors);

            let juror_info = JurorInfoOf::<T> { stake: amount };
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

            if let Ok(i) =
                jurors.binary_search_by_key(&prev_juror_info.stake, |pool_item| pool_item.stake)
            {
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
                    .binary_search_by_key(&prev_juror_info.stake, |pool_item| pool_item.stake)
                    .is_err(),
                Error::<T>::JurorNotPreparedToExit
            );

            // ensure not drawn for any market
            for (_, draws) in <Draws<T>>::iter() {
                ensure!(!draws.iter().any(|draw| draw.juror == juror), Error::<T>::JurorStillDrawn);
            }

            Jurors::<T>::remove(&juror);

            T::Currency::remove_lock(T::CourtLockId::get(), &juror);

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
        /// in the list of random selections (draws).
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

            let mut draws = <Draws<T>>::get(market_id);
            let (index, weight, slashable) = match draws.iter().position(|draw| draw.juror == who) {
                Some(index) => {
                    ensure!(
                        matches!(draws[index].vote, Vote::Drawn | Vote::Secret { secret: _ }),
                        Error::<T>::OnlyDrawnJurorsCanVote
                    );
                    (index, draws[index].weight, draws[index].slashable)
                }
                None => return Err(Error::<T>::OnlyDrawnJurorsCanVote.into()),
            };

            let vote = Vote::Secret { secret: secret_vote };
            draws[index] = Draw { juror: who.clone(), weight, vote, slashable };

            <Draws<T>>::insert(market_id, draws);

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
        /// in the list of random selections (draws).
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

            let mut draws = <Draws<T>>::get(market_id);
            let (index, weight, vote, slashable) = match draws
                .iter()
                .position(|draw| draw.juror == juror)
            {
                Some(index) => {
                    (index, draws[index].weight, draws[index].vote.clone(), draws[index].slashable)
                }
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

            let reward_pot = Self::reward_pot(&market_id);
            let slash = T::DenounceSlashPercentage::get() * slashable;
            let (imbalance, missing) = T::Currency::slash(&juror, slash);
            debug_assert!(missing.is_zero(), "Could not slash all of the amount.");
            T::Currency::resolve_creating(&reward_pot, imbalance);

            let mut jurors = JurorPool::<T>::get();
            if let Ok(i) =
                jurors.binary_search_by_key(&prev_juror_info.stake, |pool_item| pool_item.stake)
            {
                // remove from juror list to prevent being drawn
                jurors.remove(i);
                <JurorPool<T>>::put(jurors);
            }

            let raw_vote = Vote::Denounced { secret, outcome: outcome.clone(), salt };
            draws[index] = Draw { juror: juror.clone(), weight, vote: raw_vote, slashable };
            <Draws<T>>::insert(market_id, draws);

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
        /// in the list of random selections (draws).
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

            let mut draws = <Draws<T>>::get(market_id);
            let (index, weight, vote, slashable) = match draws
                .iter()
                .position(|draw| draw.juror == who)
            {
                Some(index) => {
                    (index, draws[index].weight, draws[index].vote.clone(), draws[index].slashable)
                }
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
            draws[index] = Draw { juror: who.clone(), weight, vote: raw_vote, slashable };
            <Draws<T>>::insert(market_id, draws);

            Self::deposit_event(Event::JurorRevealedVote { juror: who, market_id, outcome, salt });
            Ok(())
        }

        /// Back an appeal of a court to get an appeal initiated.
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
        pub fn back_appeal(origin: OriginFor<T>, market_id: MarketIdOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            ensure!(!court.is_appeal_backed, Error::<T>::AppealAlreadyBacked);

            let now = <frame_system::Pallet<T>>::block_number();
            Self::check_appealable_market(&market_id, &court, now)?;

            let draws = Draws::<T>::get(market_id);
            // TODO: shouldn't fail here, especially when there are no votes an appeal should be necessary or start global dispute
            let appealed_outcome = Self::get_winner(draws.as_slice())?;

            let appeal_number = court.appeals.len();
            let bond = default_appeal_bond::<T>(appeal_number);
            let appeal_info = AppealInfo { backer: who.clone(), bond, appealed_outcome };

            court.appeals.try_push(appeal_info).map_err(|_| Error::<T>::MaxAppealsReached)?;

            T::Currency::reserve_named(&Self::reserve_id(), &who, bond)?;

            let last_resolve_at = court.periods.appeal_end;

            court.is_appeal_backed = true;
            <Courts<T>>::insert(market_id, court);

            // we want to avoid the resolution before the full appeal is executed
            // So, the appeal is inevitable after the call to this extrinsic
            // otherwise the market is not going to resolve
            let _ids_len_0 = T::DisputeResolution::remove_auto_resolve(&market_id, last_resolve_at);

            Self::deposit_event(Event::AppealBacked { market_id });

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
            ensure!(!court.is_drawn, Error::<T>::JurorsAlreadyDrawn);
            ensure!(court.is_appeal_backed, Error::<T>::BackAppealFirst);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::check_appealable_market(&market_id, &court, now)?;

            let jurors: JurorPoolOf<T> = JurorPool::<T>::get();
            let appeal_number = court.appeals.len();
            Self::select_jurors(&market_id, jurors.as_slice(), appeal_number)?;

            court.is_drawn = true;
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
            ensure!(court.is_appeal_backed && court.is_drawn, Error::<T>::AppealNotReady);
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
            let appeal_number = court.appeals.len() as u32;
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

            let mut jurors = JurorPool::<T>::get();
            let reward_pot = Self::reward_pot(&market_id);
            let mut slash_and_remove_juror = |ai: &T::AccountId, slashable: BalanceOf<T>| {
                if let Some(prev_juror_info) = <Jurors<T>>::get(ai) {
                    if let Ok(i) = jurors
                        .binary_search_by_key(&prev_juror_info.stake, |pool_item| pool_item.stake)
                    {
                        // remove from juror list to prevent being drawn
                        jurors.remove(i);

                        let slash = T::TardySlashPercentage::get() * slashable;
                        let (imbalance, missing) = T::Currency::slash(ai, slash);
                        debug_assert!(
                            missing.is_zero(),
                            "Could not slash all of the amount for juror {:?}.",
                            ai
                        );

                        T::Currency::resolve_creating(&reward_pot, imbalance);
                    } else {
                        log::warn!(
                            "Juror {:?} not found in JurorPool storage for vote aggregation. \
                             Market id {:?}.",
                            ai,
                            market_id
                        );
                        debug_assert!(false);
                    }
                } else {
                    log::warn!(
                        "Juror {:?} not found in Jurors storage for vote aggregation. Market id \
                         {:?}.",
                        ai,
                        market_id
                    );
                    debug_assert!(false);
                }
            };

            for draw in Draws::<T>::get(market_id).iter() {
                match draw.vote {
                    Vote::Drawn => {
                        slash_and_remove_juror(&draw.juror, draw.slashable);
                    }
                    Vote::Secret { secret: _ } => {
                        slash_and_remove_juror(&draw.juror, draw.slashable);
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

            let draws = Draws::<T>::get(market_id);

            let mut valid_winners_and_losers = Vec::with_capacity(draws.len());

            for draw in draws {
                if let Vote::Revealed { secret: _, outcome, salt: _ } = draw.vote {
                    valid_winners_and_losers.push((draw.juror, outcome, draw.slashable));
                }
            }

            Self::slash_losers_to_award_winners(
                &market_id,
                valid_winners_and_losers.as_slice(),
                &winner,
            );

            court.status = CourtStatus::Closed { winner, punished: true, reassigned: true };
            <Courts<T>>::insert(market_id, court);
            <Draws<T>>::remove(market_id);

            Self::deposit_event(Event::JurorStakesReassigned { market_id });

            Ok(())
        }
    }

    impl<T> Pallet<T>
    where
        T: Config,
    {
        pub(crate) fn choose_multiple_weighted<R: RngCore>(
            jurors: &[JurorPoolItemOf<T>],
            number: usize,
            rng: &mut R,
        ) -> Vec<DrawOf<T>> {
            let total_weight = jurors
                .iter()
                .map(|pool_item| {
                    pool_item.stake.saturating_sub(pool_item.slashed).saturated_into::<u128>()
                })
                .sum::<u128>();

            let mut random_set = BTreeSet::new();
            for _ in 0..number {
                let random_number = rng.gen_range(0u128..total_weight);
                random_set.insert(random_number);
            }

            let mut selections = BTreeMap::<T::AccountId, (u32, BalanceOf<T>)>::new();

            let mut current_weight = 0u128;
            for JurorPoolItem { stake, juror, slashed } in jurors {
                let lower_bound = current_weight;
                let mut remainder = stake.saturating_sub(*slashed);
                let upper_bound = current_weight.saturating_add(remainder.saturated_into::<u128>());

                // this always gets the lowest random number first and maybe removes it
                for random_number in random_set.clone().iter() {
                    if &lower_bound <= random_number && random_number < &upper_bound {
                        let slashable = remainder.min(T::MinJurorStake::get());
                        if let Some((weight, sel_slashable)) = selections.get_mut(juror) {
                            *weight = weight.saturating_add(1);
                            *sel_slashable = sel_slashable.saturating_add(slashable);
                        } else {
                            selections.insert(juror.clone(), (1, slashable));
                        }
                        remainder = remainder.saturating_sub(slashable);
                        random_set.remove(random_number);
                    } else {
                        break;
                    }
                }

                if random_set.is_empty() {
                    break;
                }

                current_weight = upper_bound;
            }

            selections
                .into_iter()
                .map(|(juror, (weight, slashable))| Draw {
                    juror,
                    weight,
                    vote: Vote::Drawn,
                    slashable,
                })
                .collect()
        }

        pub(crate) fn select_jurors(
            market_id: &MarketIdOf<T>,
            jurors: &[JurorPoolItemOf<T>],
            appeal_number: usize,
        ) -> Result<(), DispatchError> {
            let necessary_jurors_number = Self::necessary_jurors_num(appeal_number);
            ensure!(jurors.len() >= necessary_jurors_number, Error::<T>::NotEnoughJurors);

            let mut rng = Self::rng();

            let random_jurors =
                Self::choose_multiple_weighted(jurors, necessary_jurors_number, &mut rng);

            // we allow at most MaxDraws jurors
            // look at `necessary_jurors_num`: MaxAppeals (= 5) example: 2^5 * 5 + 2^5 - 1 = 191
            // MaxDraws should be 191 in this case
            let draws = <DrawsOf<T>>::truncate_from(random_jurors);
            // new appeal round should have a fresh set of draws
            <Draws<T>>::insert(market_id, draws);

            Ok(())
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

            Ok(())
        }

        /// The reserve ID of the court pallet.
        #[inline]
        pub fn reserve_id() -> [u8; 8] {
            T::CourtPalletId::get().0
        }

        #[inline]
        pub fn reward_pot(market_id: &MarketIdOf<T>) -> T::AccountId {
            T::CourtPalletId::get().into_sub_account_truncating(market_id)
        }

        #[inline]
        pub(crate) fn treasury_account_id() -> T::AccountId {
            T::TreasuryPalletId::get().into_account_truncating()
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

        // Calculates the necessary number of jurors depending on the number of market appeals.
        fn necessary_jurors_num(appeals_len: usize) -> usize {
            // 2^(appeals_len) * 5 + 2^(appeals_len) - 1
            // MaxAppeals (= 5) example: 2^5 * 5 + 2^5 - 1 = 191
            SUBSEQUENT_JURORS_FACTOR
                .saturating_pow(appeals_len as u32)
                .saturating_mul(INITIAL_JURORS_NUM)
                .saturating_add(
                    SUBSEQUENT_JURORS_FACTOR.saturating_pow(appeals_len as u32).saturating_sub(1),
                )
        }

        fn slash_losers_to_award_winners(
            market_id: &MarketIdOf<T>,
            valid_winners_and_losers: &[(T::AccountId, OutcomeReport, BalanceOf<T>)],
            winner_outcome: &OutcomeReport,
        ) {
            let mut total_incentives = <NegativeImbalanceOf<T>>::zero();

            let mut jurors = JurorPool::<T>::get();

            let mut winners = Vec::with_capacity(valid_winners_and_losers.len());
            for (juror, outcome, slashable) in valid_winners_and_losers {
                if outcome == winner_outcome {
                    winners.push(juror);
                } else {
                    let slash = T::RedistributionPercentage::get() * *slashable;
                    let (imb, missing) = T::Currency::slash(&juror, slash);
                    debug_assert!(
                        missing.is_zero(),
                        "Could not slash all of the amount for juror {:?}.",
                        juror
                    );
                    total_incentives.subsume(imb);

                    if let Some(juror_info) = <Jurors<T>>::get(juror) {
                        if let Ok(i) = jurors
                            .binary_search_by_key(&juror_info.stake, |pool_item| pool_item.stake)
                        {
                            // remove from juror list to prevent being drawn
                            jurors[i].slashed = jurors[i].slashed.saturating_add(slash);
                        } else {
                            log::warn!("Juror {:?} not found in the pool.", juror);
                            debug_assert!(false);
                        }
                    } else {
                        log::warn!("Juror {:?} not found in Jurors storage.", juror);
                        debug_assert!(false);
                    }
                }
            }

            // reward from denounce slashes and tardy jurors of this market / court
            let reward_pot = Self::reward_pot(market_id);
            let reward = T::Currency::free_balance(&reward_pot);
            let (imb, missing) = T::Currency::slash(&reward_pot, reward);
            debug_assert!(missing.is_zero(), "Could not slash all of the amount for reward pot.");
            total_incentives.subsume(imb);

            if let Some(reward_per_each) =
                total_incentives.peek().checked_div(&winners.len().saturated_into())
            {
                for juror in winners {
                    let (actual_reward, leftover) = total_incentives.split(reward_per_each);
                    total_incentives = leftover;
                    T::Currency::resolve_creating(juror, actual_reward);
                }
            } else {
                // if there are no winners reward the treasury
                let treasury_acc = Self::treasury_account_id();
                T::Currency::resolve_creating(&treasury_acc, total_incentives);
            }

            <JurorPool<T>>::put(jurors);
        }

        fn get_winner(draws: &[DrawOf<T>]) -> Result<OutcomeReport, DispatchError> {
            let mut scores = BTreeMap::<OutcomeReport, u32>::new();

            for draw in draws {
                if let Vote::Revealed { secret: _, outcome, salt: _ } = &draw.vote {
                    if let Some(el) = scores.get_mut(outcome) {
                        *el = el.saturating_add(draw.weight);
                    } else {
                        scores.insert(outcome.clone(), draw.weight);
                    }
                }
            }

            let mut iter = scores.iter();

            let mut best_score = if let Some(first) = iter.next() {
                first
            } else {
                // TODO(#980): Create another voting round or trigger global dispute
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
            let appeal_number = 0usize;
            Self::select_jurors(market_id, jurors.as_slice(), appeal_number)?;

            let now = <frame_system::Pallet<T>>::block_number();

            let periods = Periods {
                backing_end: T::CourtBackingPeriod::get(),
                vote_end: T::CourtVotePeriod::get(),
                aggregation_end: T::CourtAggregationPeriod::get(),
                appeal_end: T::CourtAppealPeriod::get(),
            };

            // sets periods one after the other from now
            let court = CourtInfo::new(now, periods);

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

            let draws = Draws::<T>::get(market_id);
            let winner_outcome = Self::get_winner(draws.as_slice())?;

            court.status = CourtStatus::Closed {
                winner: winner_outcome.clone(),
                punished: false,
                reassigned: false,
            };
            <Courts<T>>::insert(market_id, court);

            Ok(Some(winner_outcome))
        }

        fn maybe_pay(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
            resolved_outcome: &OutcomeReport,
            mut overall_imbalance: NegativeImbalanceOf<T>,
        ) -> Result<NegativeImbalanceOf<T>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            let court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            for AppealInfo { backer, bond, appealed_outcome } in court.appeals {
                if resolved_outcome == &appealed_outcome {
                    let (imb, _) =
                        T::Currency::slash_reserved_named(&Self::reserve_id(), &backer, bond);
                    overall_imbalance.subsume(imb);
                } else {
                    T::Currency::unreserve_named(&Self::reserve_id(), &backer, bond);
                }
            }

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
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<bool, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            let mut has_failed = false;
            let now = <frame_system::Pallet<T>>::block_number();

            // TODO maybe add the case that the voting decision is unclear (or NoVotes case)

            let jurors_len: usize = JurorPool::<T>::decode_len().unwrap_or(0);
            match <Courts<T>>::get(market_id) {
                Some(court) => {
                    let appeals = &court.appeals;
                    let appeal_number = appeals.len();
                    let necessary_jurors_number = Self::necessary_jurors_num(appeal_number);
                    let valid_period =
                        Self::check_appealable_market(market_id, &court, now).is_ok();

                    if valid_period {
                        if jurors_len < necessary_jurors_number {
                            has_failed = true;
                        }

                        if appeals.is_full() {
                            has_failed = true;
                        }
                    }
                }
                None => {
                    let report = market.report.as_ref().ok_or(Error::<T>::MarketReportNotFound)?;
                    let report_block = report.at;
                    let block_after_dispute_duration =
                        report_block.saturating_add(market.deadlines.dispute_duration);
                    let during_dispute_duration =
                        report_block <= now && now < block_after_dispute_duration;

                    let necessary_jurors_number = Self::necessary_jurors_num(0usize);
                    if jurors_len < necessary_jurors_number && during_dispute_duration {
                        has_failed = true;
                    }
                }
            }

            Ok(has_failed)
        }

        fn on_global_dispute(market_id: &Self::MarketId, market: &MarketOf<T>) -> DispatchResult {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            if let Some(court) = <Courts<T>>::get(market_id) {
                let last_resolve_at = court.periods.appeal_end;
                let _ids_len_0 =
                    T::DisputeResolution::remove_auto_resolve(market_id, last_resolve_at);
            }

            Ok(())
        }

        fn clear(market_id: &Self::MarketId, market: &MarketOf<T>) -> DispatchResult {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );
            <Draws<T>>::remove(market_id);
            <Courts<T>>::remove(market_id);
            Ok(())
        }
    }

    impl<T> CourtPalletApi for Pallet<T> where T: Config {}

    // No-one can bound more than BalanceOf<T>, therefore, this functions saturates
    pub fn default_appeal_bond<T>(n: usize) -> BalanceOf<T>
    where
        T: Config,
    {
        T::AppealBond::get().saturating_add(
            T::AppealBondFactor::get().saturating_mul(n.saturated_into::<u32>().into()),
        )
    }
}
