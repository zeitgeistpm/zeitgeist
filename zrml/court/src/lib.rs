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
        pallet_prelude::{Hooks, OptionQuery, StorageMap, StorageValue, ValueQuery, Weight},
        traits::{
            Currency, Get, Imbalance, IsType, LockIdentifier, LockableCurrency,
            NamedReservableCurrency, Randomness, ReservableCurrency, StorageVersion,
            WithdrawReasons,
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

        /// The interval for requesting multiple court votes at once.
        #[pallet::constant]
        type RequestInterval: Get<Self::BlockNumber>;

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

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
    pub(crate) type NegativeImbalanceOf<T> =
        <<T as Config>::Currency as Currency<AccountIdOf<T>>>::NegativeImbalance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub(crate) type MarketOf<T> = Market<
        AccountIdOf<T>,
        BalanceOf<T>,
        <T as frame_system::Config>::BlockNumber,
        MomentOf<T>,
        Asset<MarketIdOf<T>>,
    >;
    pub(crate) type AccountIdLookupOf<T> =
        <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
    pub(crate) type CourtOf<T> = CourtInfo<<T as frame_system::Config>::BlockNumber, AppealsOf<T>>;
    pub(crate) type JurorInfoOf<T> = JurorInfo<BalanceOf<T>>;
    pub(crate) type JurorPoolItemOf<T> = JurorPoolItem<AccountIdOf<T>, BalanceOf<T>>;
    pub(crate) type JurorPoolOf<T> = BoundedVec<JurorPoolItemOf<T>, <T as Config>::MaxJurors>;
    pub(crate) type DrawOf<T> =
        Draw<AccountIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::Hash>;
    pub(crate) type DrawsOf<T> = BoundedVec<DrawOf<T>, <T as Config>::MaxDraws>;
    pub(crate) type AppealOf<T> = AppealInfo<AccountIdOf<T>, BalanceOf<T>>;
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

    /// The block number in the future when jurors should start voting.
    /// This is useful for the user experience of the jurors to vote for multiple courts at once.
    #[pallet::storage]
    pub type RequestBlock<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

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
        /// A market has been appealed.
        MarketAppealed { market_id: MarketIdOf<T>, appeal_number: u32 },
        /// The juror stakes have been reassigned.
        JurorStakesReassigned { market_id: MarketIdOf<T> },
        /// The tardy jurors have been punished.
        TardyJurorsPunished { market_id: MarketIdOf<T> },
        /// The global dispute backing was successful.
        GlobalDisputeBacked { market_id: MarketIdOf<T> },
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
        /// For this appeal round the backing check extrinsic was already called.
        AppealAlreadyBacked,
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
        /// The juror was not found in the pool.
        JurorAlreadyPreparedToExit,
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
        /// The maximum number of normal appeals is reached. So only allow to back a global dispute.
        OnlyGlobalDisputeAppealAllowed,
        /// In order to back a global dispute, it has to be the last appeal (MaxAppeals reached).
        NeedsToBeLastAppeal,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_initialize(now: T::BlockNumber) -> Weight {
            let mut total_weight: Weight = Weight::zero();
            if now >= <RequestBlock<T>>::get() {
                let future_request = now.saturating_add(T::RequestInterval::get());
                <RequestBlock<T>>::put(future_request);
                total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
            }
            total_weight
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Join to become a juror, who is able to get randomly selected
        /// for court cases according to the provided stake.
        /// The probability to get selected is higher the more funds are staked.
        /// The amount is added to the stake-weighted pool.
        /// If the pool is full, the juror with the lowest stake is removed from the juror pool.
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

            if jurors.is_full() {
                // remove the lowest staked juror
                debug_assert!(
                    jurors
                        .first()
                        .map(|pool_item| pool_item.stake)
                        .unwrap_or(<BalanceOf<T>>::zero())
                        <= jurors
                            .last()
                            .map(|pool_item| pool_item.stake)
                            .unwrap_or(<BalanceOf<T>>::zero())
                );
                jurors.remove(0);
            }

            let mut slashed = <BalanceOf<T>>::zero();
            if let Some(prev_juror_info) = <Jurors<T>>::get(&who) {
                ensure!(amount > prev_juror_info.stake, Error::<T>::AmountBelowLastJoin);
                let (index, pool_item) = Self::get_pool_item(&jurors, prev_juror_info.stake)
                    .ok_or(Error::<T>::JurorNeedsToExit)?;
                slashed = pool_item.slashed;
                jurors.remove(index);
            }

            match jurors.binary_search_by_key(&amount, |pool_item| pool_item.stake) {
                // The reason for this error is that each amount has a clear juror
                // binary_search_by_key could otherwise return an index of an unwanted juror
                // if there are multiple jurors with the same stake
                Ok(_) => return Err(Error::<T>::AmountAlreadyUsed.into()),
                Err(i) => jurors
                    .try_insert(i, JurorPoolItem { stake: amount, juror: who.clone(), slashed })
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
        /// Returns an error if the juror is already not part of the pool anymore.
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

            if let Some((index, _)) = Self::get_pool_item(&jurors, prev_juror_info.stake) {
                jurors.remove(index);
                <JurorPool<T>>::put(jurors);
            } else {
                // this error can happen if the lowest bonded juror was removed
                // it can also happen when juror was denounced,
                // did not vote or reveal
                return Err(Error::<T>::JurorAlreadyPreparedToExit.into());
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

            let jurors = JurorPool::<T>::get();
            ensure!(
                Self::get_pool_item(&jurors, prev_juror_info.stake).is_none(),
                Error::<T>::JurorNotPreparedToExit
            );

            // TODO this belongs to prepare_exit_court and this extrinsic should be called whenever a juror is removed from the pool (use state which recognizes that the call to this extrinsic is allowed)
            // TODO unbounded iteration is dangerous here
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
                court.periods.pre_vote_end < now && now <= court.periods.vote_end,
                Error::<T>::NotInVotingPeriod
            );

            let mut draws = <Draws<T>>::get(market_id);
            let (index, draw) = match draws.iter().position(|draw| draw.juror == who) {
                Some(index) => {
                    // allow to override last vote
                    ensure!(
                        matches!(draws[index].vote, Vote::Drawn | Vote::Secret { secret: _ }),
                        Error::<T>::OnlyDrawnJurorsCanVote
                    );
                    (index, draws[index].clone())
                }
                None => return Err(Error::<T>::OnlyDrawnJurorsCanVote.into()),
            };

            let vote = Vote::Secret { secret: secret_vote };
            draws[index] = Draw { juror: who.clone(), vote, ..draw };

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
                court.periods.pre_vote_end < now && now <= court.periods.vote_end,
                Error::<T>::NotInVotingPeriod
            );

            let mut draws = <Draws<T>>::get(market_id);
            let (index, draw) = match draws.iter().position(|draw| draw.juror == juror) {
                Some(index) => (index, draws[index].clone()),
                None => return Err(Error::<T>::JurorNotDrawn.into()),
            };

            let secret = match draw.vote {
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
            let slash = T::DenounceSlashPercentage::get() * draw.slashable;
            let (imbalance, missing) = T::Currency::slash(&juror, slash);
            debug_assert!(missing.is_zero(), "Could not slash all of the amount.");
            T::Currency::resolve_creating(&reward_pot, imbalance);

            let mut jurors = JurorPool::<T>::get();
            if let Some((index, _)) = Self::get_pool_item(&jurors, prev_juror_info.stake) {
                jurors.remove(index);
                <JurorPool<T>>::put(jurors);
            }

            let raw_vote = Vote::Denounced { secret, outcome: outcome.clone(), salt };
            draws[index] = Draw { juror: juror.clone(), vote: raw_vote, ..draw };
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
            let (index, draw) = match draws.iter().position(|draw| draw.juror == who) {
                Some(index) => (index, draws[index].clone()),
                None => return Err(Error::<T>::JurorNotDrawn.into()),
            };

            let secret = match draw.vote {
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
            draws[index] = Draw { juror: who.clone(), vote: raw_vote, ..draw };
            <Draws<T>>::insert(market_id, draws);

            Self::deposit_event(Event::JurorRevealedVote { juror: who, market_id, outcome, salt });
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
        /// Complexity: `O(n)`, where `n` is the number of jurors.
        /// It depends heavily on `choose_multiple_weighted` of `select_jurors`.
        #[pallet::weight(1_000_000_000_000)]
        #[transactional]
        pub fn appeal(origin: OriginFor<T>, market_id: MarketIdOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            let appeal_number = court.appeals.len().saturating_add(1);
            let bond = default_appeal_bond::<T>(appeal_number);
            ensure!(T::Currency::can_reserve(&who, bond), Error::<T>::InsufficientAmount);
            ensure!(
                appeal_number < <AppealsOf<T>>::bound(),
                Error::<T>::OnlyGlobalDisputeAppealAllowed
            );
            let now = <frame_system::Pallet<T>>::block_number();
            Self::check_appealable_market(&market_id, &court, now)?;
            // the outcome which would be resolved on is appealed (including oracle report)
            let appealed_outcome = Self::get_latest_resolved_outcome(&market_id)?;
            let appeal_info = AppealInfo { backer: who.clone(), bond, appealed_outcome };
            court.appeals.try_push(appeal_info).map_err(|_| {
                debug_assert!(false, "Appeal bound is checked above.");
                Error::<T>::MaxAppealsReached
            })?;

            let jurors: JurorPoolOf<T> = JurorPool::<T>::get();
            Self::select_jurors(&market_id, jurors.as_slice(), appeal_number)?;

            let last_resolve_at = court.periods.appeal_end;
            let _ids_len_0 = T::DisputeResolution::remove_auto_resolve(&market_id, last_resolve_at);

            let request_block = <RequestBlock<T>>::get();
            debug_assert!(request_block >= now, "Request block must be greater than now.");
            let pre_vote_end = request_block.saturating_sub(now);
            let periods = Periods {
                pre_vote_end,
                vote_end: T::CourtVotePeriod::get(),
                aggregation_end: T::CourtAggregationPeriod::get(),
                appeal_end: T::CourtAppealPeriod::get(),
            };
            // sets periods one after the other from now
            court.update_periods(periods, now);
            let new_resolve_at = court.periods.appeal_end;
            debug_assert!(new_resolve_at != last_resolve_at);
            let _ids_len_1 =
                T::DisputeResolution::add_auto_resolve(&market_id, court.periods.appeal_end)?;

            T::Currency::reserve_named(&Self::reserve_id(), &who, bond)?;

            <Courts<T>>::insert(market_id, court);

            let appeal_number = appeal_number as u32;
            Self::deposit_event(Event::MarketAppealed { market_id, appeal_number });

            Ok(())
        }

        /// Back the global dispute to allow the market to be resolved
        /// if the last appeal outcome is disagreed on.
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
        pub fn back_global_dispute(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            let appeal_number = court.appeals.len().saturating_add(1);
            ensure!(appeal_number == <AppealsOf<T>>::bound(), Error::<T>::NeedsToBeLastAppeal);

            let now = <frame_system::Pallet<T>>::block_number();
            Self::check_appealable_market(&market_id, &court, now)?;

            // the outcome which would be resolved on is appealed (including oracle report)
            let appealed_outcome = Self::get_latest_resolved_outcome(&market_id)?;

            let bond = default_appeal_bond::<T>(appeal_number);
            let appeal_info = AppealInfo { backer: who.clone(), bond, appealed_outcome };

            court.appeals.try_push(appeal_info).map_err(|_| {
                debug_assert!(false, "Appeal bound is checked above.");
                Error::<T>::MaxAppealsReached
            })?;

            T::Currency::reserve_named(&Self::reserve_id(), &who, bond)?;

            let last_resolve_at = court.periods.appeal_end;
            let _ids_len_0 = T::DisputeResolution::remove_auto_resolve(&market_id, last_resolve_at);

            Self::deposit_event(Event::GlobalDisputeBacked { market_id });

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
                let slash = T::TardySlashPercentage::get() * slashable;
                let (imbalance, missing) = T::Currency::slash(ai, slash);
                debug_assert!(
                    missing.is_zero(),
                    "Could not slash all of the amount for juror {:?}.",
                    ai
                );
                T::Currency::resolve_creating(&reward_pot, imbalance);

                if let Some(prev_juror_info) = <Jurors<T>>::get(ai) {
                    if let Some((index, _)) = Self::get_pool_item(&jurors, prev_juror_info.stake) {
                        jurors.remove(index);
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
                let random_number = rng.gen_range(0u128..=total_weight);
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
                        if let Some((weight, all_slashable)) = selections.get_mut(juror) {
                            *weight = weight.saturating_add(1);
                            *all_slashable = all_slashable.saturating_add(slashable);
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
            debug_assert!(
                if appeal_number > 0 {
                    appeal_number == <Courts<T>>::get(market_id).unwrap().appeals.len()
                } else {
                    // for the first court round, we have no previous winner
                    true
                },
                "Before the draws are overwritten, we need to ensure that the winner of the last \
                 appeal round was determined with 'get_latest_resolved_outcome' (the data of the \
                 last Draws)."
            );
            // new appeal round should have a fresh set of draws
            <Draws<T>>::insert(market_id, draws);

            Ok(())
        }

        // Returns (index, pool_item) if the stake associated with the juror was found.
        // It returns None otherwise.
        pub(crate) fn get_pool_item(
            jurors: &[JurorPoolItemOf<T>],
            stake: BalanceOf<T>,
        ) -> Option<(usize, &JurorPoolItemOf<T>)> {
            if let Ok(i) = jurors.binary_search_by_key(&stake, |pool_item| pool_item.stake) {
                return Some((i, &jurors[i]));
            }
            // this None case can happen when the lowest bonded juror was removed (`join_court`)
            // it can also happen when juror was denounced,
            // did not vote or reveal or when the juror decided to leave the court
            return None;
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
                    let (imb, missing) = T::Currency::slash(juror, slash);
                    debug_assert!(
                        missing.is_zero(),
                        "Could not slash all of the amount for juror {:?}.",
                        juror
                    );
                    total_incentives.subsume(imb);

                    if let Some(juror_info) = <Jurors<T>>::get(juror) {
                        if let Some((index, _)) = Self::get_pool_item(&jurors, juror_info.stake) {
                            jurors[index].slashed = jurors[index].slashed.saturating_add(slash);
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

        // Returns the winner of the current court round.
        // If there is no element inside `draws`, returns `None`.
        // If the best two outcomes have the same score, returns the last court round winner.
        fn get_winner(
            draws: &[DrawOf<T>],
            last_winner: Option<OutcomeReport>,
        ) -> Option<OutcomeReport> {
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
            let mut best_score = iter.next()?;
            let mut second_best_score = if let Some(second) = iter.next() {
                if second.1 > best_score.1 {
                    best_score = second;
                    best_score
                } else {
                    second
                }
            } else {
                return Some(best_score.0.clone());
            };

            for el in iter {
                if el.1 > best_score.1 {
                    best_score = el;
                    second_best_score = best_score;
                } else if el.1 > second_best_score.1 {
                    second_best_score = el;
                }
            }

            if best_score.1 == second_best_score.1 {
                return last_winner;
            }

            Some(best_score.0.clone())
        }

        fn get_latest_resolved_outcome(
            market_id: &MarketIdOf<T>,
        ) -> Result<OutcomeReport, DispatchError> {
            let market = T::MarketCommons::market(market_id)?;
            let court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            let last_winner: Option<OutcomeReport> = court
                .appeals
                .last()
                .map(|appeal_info| Some(appeal_info.appealed_outcome.clone()))
                .unwrap_or(None);
            let report = market.report.as_ref().ok_or(Error::<T>::MarketReportNotFound)?;
            let oracle_outcome = report.outcome.clone();
            let draws = Draws::<T>::get(market_id);
            let resolved_outcome =
                Self::get_winner(draws.as_slice(), last_winner).unwrap_or(oracle_outcome);
            Ok(resolved_outcome)
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
            let request_block = <RequestBlock<T>>::get();
            debug_assert!(request_block >= now, "Request block must be greater than now.");
            let pre_vote_end = request_block.saturating_sub(now);

            let periods = Periods {
                pre_vote_end,
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
            let resolved_outcome = Self::get_latest_resolved_outcome(market_id)?;
            court.status = CourtStatus::Closed {
                winner: resolved_outcome.clone(),
                punished: false,
                reassigned: false,
            };
            <Courts<T>>::insert(market_id, court);

            Ok(Some(resolved_outcome))
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
            for AppealInfo { backer, bond, appealed_outcome } in &court.appeals {
                if resolved_outcome == appealed_outcome {
                    let (imb, _) =
                        T::Currency::slash_reserved_named(&Self::reserve_id(), backer, *bond);
                    overall_imbalance.subsume(imb);
                } else {
                    T::Currency::unreserve_named(&Self::reserve_id(), backer, *bond);
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

            let jurors_len: usize = JurorPool::<T>::decode_len().unwrap_or(0);
            match <Courts<T>>::get(market_id) {
                Some(court) => {
                    let appeals = &court.appeals;
                    let appeal_number = appeals.len().saturating_add(1);
                    let necessary_jurors_number = Self::necessary_jurors_num(appeal_number);
                    let valid_period =
                        Self::check_appealable_market(market_id, &court, now).is_ok();

                    if appeals.is_full() || (valid_period && (jurors_len < necessary_jurors_number))
                    {
                        has_failed = true;
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
                    if during_dispute_duration && jurors_len < necessary_jurors_number {
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

            <Draws<T>>::remove(market_id);
            <Courts<T>>::remove(market_id);

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
