// Copyright 2022-2023 Forecasting Technologies LTD.
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

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity)]

extern crate alloc;

use crate::{
    weights::WeightInfoZeitgeist, AppealInfo, CommitmentMatcher, CourtInfo, CourtStatus, Draw,
    JurorInfo, JurorPoolItem, JurorVoteWithStakes, RawCommitment, RoundTiming, SelectionAdd,
    SelectionError, SelectionValue, SelfInfo, Vote,
};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    vec::Vec,
};
use core::marker::PhantomData;
use frame_support::{
    dispatch::DispatchResult,
    ensure, log,
    pallet_prelude::{
        ConstU32, DispatchResultWithPostInfo, EnsureOrigin, Hooks, OptionQuery, StorageMap,
        StorageValue, ValueQuery, Weight,
    },
    traits::{
        Currency, Get, Imbalance, IsType, LockIdentifier, LockableCurrency,
        NamedReservableCurrency, OnUnbalanced, Randomness, ReservableCurrency, StorageVersion,
        WithdrawReasons,
    },
    transactional, Blake2_128Concat, BoundedVec, PalletId,
};
use frame_system::{
    ensure_signed,
    pallet_prelude::{BlockNumberFor, OriginFor},
};
use rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use sp_arithmetic::{per_things::Perquintill, traits::One};
use sp_runtime::{
    traits::{AccountIdConversion, Hash, Saturating, StaticLookup, Zero},
    DispatchError, Perbill, SaturatedConversion,
};
use zeitgeist_primitives::{
    traits::{DisputeApi, DisputeResolutionApi},
    types::{
        Asset, GlobalDisputeItem, Market, MarketDisputeMechanism, MarketStatus, OutcomeReport,
    },
};
use zrml_market_commons::MarketCommonsPalletApi;

mod benchmarks;
mod court_pallet_api;
pub mod migrations;
mod mock;
mod mock_storage;
mod tests;
pub mod types;
pub mod weights;

pub use court_pallet_api::CourtPalletApi;
pub use pallet::*;
pub use types::*;

#[frame_support::pallet]
mod pallet {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The required base bond in order to get an appeal initiated.
        /// This bond increases exponentially with the number of appeals.
        #[pallet::constant]
        type AppealBond: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type BlocksPerYear: Get<Self::BlockNumber>;

        /// The time in which the jurors can cast their commitment vote.
        #[pallet::constant]
        type CourtVotePeriod: Get<Self::BlockNumber>;

        /// The time in which the jurors should reveal their commitment vote.
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

        /// The functionality to allow controlling the markets resolution time.
        type DisputeResolution: DisputeResolutionApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
            MarketId = MarketIdOf<Self>,
            Moment = MomentOf<Self>,
        >;

        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        #[pallet::constant]
        type InflationPeriod: Get<Self::BlockNumber>;

        /// Market commons
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
            Currency = Self::Currency,
        >;

        /// The maximum number of appeals until the court fails.
        #[pallet::constant]
        type MaxAppeals: Get<u32>;

        /// The maximum number of randomly selected jurors for a dispute.
        #[pallet::constant]
        type MaxSelectedDraws: Get<u32>;

        #[pallet::constant]
        type MaxDelegations: Get<u32>;

        /// The maximum number of jurors that can be registered.
        #[pallet::constant]
        type MaxJurors: Get<u32>;

        /// The minimum stake a user needs to reserve to become a juror.
        #[pallet::constant]
        type MinJurorStake: Get<BalanceOf<Self>>;

        /// The origin for monetary governance
        type MonetaryGovernanceOrigin: EnsureOrigin<Self::Origin>;

        /// Randomness source
        type Random: Randomness<Self::Hash, Self::BlockNumber>;

        /// The global interval which schedules the start of new court vote periods.
        #[pallet::constant]
        type RequestInterval: Get<Self::BlockNumber>;

        /// Handler for slashed funds.
        type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

        /// Slashed funds are send to the treasury
        #[pallet::constant]
        type TreasuryPalletId: Get<PalletId>;

        /// Weights generated by benchmarks
        type WeightInfo: WeightInfoZeitgeist;
    }

    // Number of draws for an initial market dispute.
    const INITIAL_DRAWS_NUM: usize = 5;
    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(2);
    // Weight used to increase the number of jurors for subsequent appeals
    // of the same market
    const APPEAL_BASIS: usize = 2;
    // Basis used to increase the bond for subsequent appeals of the same market
    const APPEAL_BOND_BASIS: u32 = 2;

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
    pub(crate) type HashOf<T> = <T as frame_system::Config>::Hash;
    pub(crate) type AccountIdLookupOf<T> =
        <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
    pub(crate) type CourtOf<T> = CourtInfo<<T as frame_system::Config>::BlockNumber, AppealsOf<T>>;
    pub(crate) type DelegatedStakesOf<T> =
        BoundedVec<(AccountIdOf<T>, BalanceOf<T>), <T as Config>::MaxDelegations>;
    pub(crate) type SelectionValueOf<T> = SelectionValue<BalanceOf<T>, DelegatedStakesOf<T>>;
    pub(crate) type DelegationsOf<T> = BoundedVec<AccountIdOf<T>, <T as Config>::MaxDelegations>;
    pub(crate) type VoteOf<T> = Vote<HashOf<T>, DelegatedStakesOf<T>>;
    pub(crate) type JurorVoteWithStakesOf<T> = JurorVoteWithStakes<AccountIdOf<T>, BalanceOf<T>>;
    pub(crate) type JurorInfoOf<T> = JurorInfo<BalanceOf<T>, BlockNumberFor<T>, DelegationsOf<T>>;
    pub(crate) type JurorPoolItemOf<T> = JurorPoolItem<AccountIdOf<T>, BalanceOf<T>>;
    pub(crate) type JurorPoolOf<T> = BoundedVec<JurorPoolItemOf<T>, <T as Config>::MaxJurors>;
    pub(crate) type DrawOf<T> = Draw<AccountIdOf<T>, BalanceOf<T>, HashOf<T>, DelegatedStakesOf<T>>;
    pub(crate) type SelectedDrawsOf<T> = BoundedVec<DrawOf<T>, <T as Config>::MaxSelectedDraws>;
    pub(crate) type AppealOf<T> = AppealInfo<AccountIdOf<T>, BalanceOf<T>>;
    pub(crate) type AppealsOf<T> = BoundedVec<AppealOf<T>, <T as Config>::MaxAppeals>;
    pub(crate) type CommitmentMatcherOf<T> = CommitmentMatcher<AccountIdOf<T>, HashOf<T>>;
    pub(crate) type RawCommitmentOf<T> = RawCommitment<AccountIdOf<T>, HashOf<T>>;
    pub type CacheSize = ConstU32<64>;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    /// The pool of jurors who can get randomly selected according to their stake.
    /// The pool is sorted by stake in ascending order [min, ..., max].
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
    pub type SelectedDraws<T: Config> =
        StorageMap<_, Blake2_128Concat, MarketIdOf<T>, SelectedDrawsOf<T>, ValueQuery>;

    /// The general information about each court.
    #[pallet::storage]
    pub type Courts<T: Config> =
        StorageMap<_, Blake2_128Concat, MarketIdOf<T>, CourtOf<T>, OptionQuery>;

    /// The block number in the future when jurors should start voting.
    /// This is useful for the user experience of the jurors to vote for multiple courts at once.
    #[pallet::storage]
    pub type RequestBlock<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    #[pallet::type_value]
    pub fn DefaultYearlyInflation<T: Config>() -> Perbill {
        Perbill::from_perthousand(20u32)
    }

    /// The current inflation rate.
    #[pallet::storage]
    pub type YearlyInflation<T: Config> =
        StorageValue<_, Perbill, ValueQuery, DefaultYearlyInflation<T>>;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A juror has been added to the court.
        JurorJoined {
            juror: T::AccountId,
            stake: BalanceOf<T>,
        },
        /// A juror prepared to exit the court.
        JurorPreparedExit {
            juror: T::AccountId,
        },
        /// A juror has been removed from the court.
        JurorExited {
            juror: T::AccountId,
            exit_amount: BalanceOf<T>,
            active_lock: BalanceOf<T>,
        },
        /// A juror has voted in a court.
        JurorVoted {
            market_id: MarketIdOf<T>,
            juror: T::AccountId,
            commitment: T::Hash,
        },
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
        /// A delegator has delegated their stake to jurors.
        DelegatedToJurors {
            delegator: T::AccountId,
            amount: BalanceOf<T>,
            delegated_jurors: Vec<T::AccountId>,
        },
        /// A market has been appealed.
        MarketAppealed {
            market_id: MarketIdOf<T>,
            appeal_number: u32,
        },
        MintedInCourt {
            juror: T::AccountId,
            amount: BalanceOf<T>,
        },
        /// The juror stakes have been reassigned. The losing jurors have been slashed.
        /// The winning jurors have been rewarded by the losers.
        /// The losing jurors are those, who did not vote,
        /// were denounced or did not reveal their vote.
        JurorStakesReassigned {
            market_id: MarketIdOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// An account id does not exist on the jurors storage.
        JurorDoesNotExist,
        /// On dispute or resolution, someone tried to pass a non-court market type.
        MarketDoesNotHaveCourtMechanism,
        /// The market is not in a state where it can be disputed.
        MarketIsNotDisputed,
        /// Only jurors can reveal their votes.
        OnlyJurorsCanReveal,
        /// The vote is not commitment.
        VoteAlreadyRevealed,
        /// The outcome and salt reveal do not match the commitment vote.
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
        /// The caller of this extrinsic needs to be drawn or in the commitment vote state.
        InvalidVoteState,
        /// The amount is below the minimum required stake.
        BelowMinJurorStake,
        /// The maximum number of possible jurors has been reached.
        MaxJurorsReached,
        /// In order to exit the court the juror has to exit
        /// the pool first with `prepare_exit_court`.
        JurorAlreadyPreparedExit,
        /// The juror needs to exit the court and then rejoin.
        JurorNeedsToExit,
        /// The juror was not randomly selected for the court.
        JurorNotDrawn,
        /// The juror was drawn but did not manage to commitmently vote within the court.
        JurorNotVoted,
        /// The juror was already denounced.
        VoteAlreadyDenounced,
        /// A juror tried to denounce herself.
        SelfDenounceDisallowed,
        /// The court is not in the closed state.
        CourtNotClosed,
        /// The juror stakes of the court already got reassigned.
        CourtAlreadyReassigned,
        /// There are not enough jurors in the pool.
        NotEnoughJurors,
        /// The report of the market was not found.
        MarketReportNotFound,
        /// The caller has not enough funds to join the court with the specified amount.
        AmountExceedsBalance,
        /// After the first join of the court the amount has to be higher than the current stake.
        AmountBelowLastJoin,
        /// The random number generation failed.
        RandNumGenFailed,
        /// The amount is too low to kick the lowest juror out of the stake-weighted pool.
        AmountBelowLowestJuror,
        /// This should not happen, because the juror account should only be once in a pool.
        JurorTwiceInPool,
        /// The caller of this function is not part of the juror draws.
        CallerNotInSelectedDraws,
        /// The callers balance is lower than the appeal bond.
        AppealBondExceedsBalance,
        /// The outcome does not match the market outcome type.
        OutcomeMismatch,
        /// The juror should at least wait one inflation period after the funds can be unstaked.
        /// Otherwise hopping in and out for inflation rewards is possible.
        WaitFullInflationPeriod,
        /// The `prepare_exit_at` field is not present.
        PrepareExitAtNotPresent,
        MaxDelegationsReached,
        JurorDelegated,
        SelfDelegationNotAllowed,
        IdenticalDelegationsNotAllowed,
        NoDelegations,
        DelegatedToInvalidJuror,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_initialize(now: T::BlockNumber) -> Weight {
            let mut total_weight: Weight = Weight::zero();
            total_weight = total_weight.saturating_add(Self::handle_inflation(now));
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
            if now >= <RequestBlock<T>>::get() {
                let future_request = now.saturating_add(T::RequestInterval::get());
                <RequestBlock<T>>::put(future_request);
                total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
            }
            total_weight
        }

        fn integrity_test() {
            assert!(!T::BlocksPerYear::get().is_zero(), "Blocks per year assumption changed.");
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Join to become a juror, who is able to get randomly selected
        /// for court cases according to the provided stake.
        /// The probability to get selected is higher the more funds are staked.
        /// The `amount` of this call represents the total stake of the juror.
        /// If the pool is full, the lowest staked juror is removed from the juror pool.
        ///
        /// # Arguments
        ///
        /// - `amount`: The total stake associated with the joining juror.
        ///
        /// # Weight
        ///
        /// Complexity: `O(log(n))`, where `n` is the number of jurors in the stake-weighted pool.
        #[pallet::weight(T::WeightInfo::join_court(T::MaxJurors::get()))]
        #[transactional]
        pub fn join_court(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let jurors_len = Self::do_join_court(&who, amount, None)?;

            Self::deposit_event(Event::JurorJoined { juror: who, stake: amount });

            Ok(Some(T::WeightInfo::join_court(jurors_len)).into())
        }

        // TODO benchmark
        #[pallet::weight(T::WeightInfo::join_court(T::MaxJurors::get()))]
        #[transactional]
        pub fn delegate(
            origin: OriginFor<T>,
            amount: BalanceOf<T>,
            delegations: Vec<T::AccountId>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(!delegations.is_empty(), Error::<T>::NoDelegations);
            let mut sorted_delegations: DelegationsOf<T> =
                delegations.clone().try_into().map_err(|_| Error::<T>::MaxDelegationsReached)?;

            let jurors = JurorPool::<T>::get();
            let is_valid_set = sorted_delegations.iter().all(|pretended_juror| {
                <Jurors<T>>::get(pretended_juror).map_or(false, |pretended_juror_info| {
                    Self::get_pool_item(&jurors, pretended_juror_info.stake, pretended_juror)
                        .is_some()
                        && pretended_juror_info.delegations.is_none()
                })
            });
            ensure!(is_valid_set, Error::<T>::DelegatedToInvalidJuror);
            // ensure all elements are different
            sorted_delegations.sort();
            let has_duplicates = sorted_delegations
                .iter()
                .zip(sorted_delegations.iter().skip(1))
                .any(|(x, y)| x == y);
            ensure!(!has_duplicates, Error::<T>::IdenticalDelegationsNotAllowed);
            ensure!(!sorted_delegations.contains(&who), Error::<T>::SelfDelegationNotAllowed);

            let jurors_len = Self::do_join_court(&who, amount, Some(sorted_delegations))?;

            Self::deposit_event(Event::DelegatedToJurors {
                delegator: who,
                amount,
                delegated_jurors: delegations,
            });

            Ok(Some(T::WeightInfo::join_court(jurors_len)).into())
        }

        /// Prepare as a juror to exit the court.
        /// When this is called the juror is not anymore able to get drawn for new cases.
        /// The juror gets removed from the stake-weighted pool.
        /// After that the juror can exit the court.
        ///
        /// # Weight
        ///
        /// Complexity: `O(log(n))`, where `n` is the number of jurors in the stake-weighted pool.
        #[pallet::weight(T::WeightInfo::prepare_exit_court(T::MaxJurors::get()))]
        #[transactional]
        pub fn prepare_exit_court(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let mut prev_juror_info =
                <Jurors<T>>::get(&who).ok_or(Error::<T>::JurorDoesNotExist)?;
            ensure!(
                prev_juror_info.prepare_exit_at.is_none(),
                Error::<T>::JurorAlreadyPreparedExit
            );

            let mut jurors = JurorPool::<T>::get();
            let jurors_len = jurors.len() as u32;

            // do not error in the else case
            // because the juror might have been already removed from the pool
            if let Some((index, _)) = Self::get_pool_item(&jurors, prev_juror_info.stake, &who) {
                jurors.remove(index);
                <JurorPool<T>>::put(jurors);
            }

            let now = <frame_system::Pallet<T>>::block_number();
            prev_juror_info.prepare_exit_at = Some(now);
            <Jurors<T>>::insert(&who, prev_juror_info);

            Self::deposit_event(Event::JurorPreparedExit { juror: who });

            Ok(Some(T::WeightInfo::prepare_exit_court(jurors_len)).into())
        }

        /// Exit the court.
        /// The stake which is not locked by any court case is unlocked.
        /// `prepare_exit_court` must be called before
        /// to remove the juror from the stake-weighted pool.
        ///
        /// # Arguments
        ///
        /// - `juror`: The juror, who is assumed to be not be part of the pool anymore.
        ///
        /// # Weight
        ///
        /// Complexity: `O(log(n))`, where `n` is the number of jurors in the stake-weighted pool.
        #[pallet::weight(T::WeightInfo::exit_court_set().max(T::WeightInfo::exit_court_remove()))]
        #[transactional]
        pub fn exit_court(
            origin: OriginFor<T>,
            juror: AccountIdLookupOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            let juror = T::Lookup::lookup(juror)?;

            let mut prev_juror_info =
                <Jurors<T>>::get(&juror).ok_or(Error::<T>::JurorDoesNotExist)?;

            let now = <frame_system::Pallet<T>>::block_number();
            let prepare_exit_at =
                prev_juror_info.prepare_exit_at.ok_or(Error::<T>::PrepareExitAtNotPresent)?;
            ensure!(
                now.saturating_sub(prepare_exit_at) >= T::InflationPeriod::get(),
                Error::<T>::WaitFullInflationPeriod
            );

            let (exit_amount, active_lock, weight) = if prev_juror_info.active_lock.is_zero() {
                T::Currency::remove_lock(T::CourtLockId::get(), &juror);
                Jurors::<T>::remove(&juror);
                (prev_juror_info.stake, <BalanceOf<T>>::zero(), T::WeightInfo::exit_court_remove())
            } else {
                let active_lock = prev_juror_info.active_lock;
                let exit_amount = prev_juror_info.stake.saturating_sub(active_lock);
                T::Currency::set_lock(
                    T::CourtLockId::get(),
                    &juror,
                    active_lock,
                    WithdrawReasons::all(),
                );

                prev_juror_info.stake = active_lock;
                Jurors::<T>::insert(&juror, prev_juror_info);

                (exit_amount, active_lock, T::WeightInfo::exit_court_set())
            };

            Self::deposit_event(Event::JurorExited { juror, exit_amount, active_lock });

            Ok(Some(weight).into())
        }

        /// Vote as a randomly selected juror for a specific court case.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The identifier of the court.
        /// - `commitment_vote`: A hash which consists of `juror ++ outcome ++ salt`.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of jurors
        /// in the list of random selections (draws).
        #[pallet::weight(T::WeightInfo::vote(T::MaxSelectedDraws::get()))]
        #[transactional]
        pub fn vote(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            commitment_vote: T::Hash,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(
                court.cycle_ends.pre_vote < now && now <= court.cycle_ends.vote,
                Error::<T>::NotInVotingPeriod
            );

            let mut draws = <SelectedDraws<T>>::get(market_id);

            match draws.binary_search_by_key(&who, |draw| draw.juror.clone()) {
                Ok(index) => {
                    let draw = draws[index].clone();

                    // allow to override last vote
                    ensure!(
                        matches!(draws[index].vote, Vote::Drawn | Vote::Secret { commitment: _ }),
                        Error::<T>::InvalidVoteState
                    );

                    let vote = Vote::Secret { commitment: commitment_vote };
                    draws[index] = Draw { vote, ..draw };
                }
                Err(_) => return Err(Error::<T>::CallerNotInSelectedDraws.into()),
            }

            let draws_len = draws.len() as u32;

            <SelectedDraws<T>>::insert(market_id, draws);

            Self::deposit_event(Event::JurorVoted {
                juror: who,
                market_id,
                commitment: commitment_vote,
            });

            Ok(Some(T::WeightInfo::vote(draws_len)).into())
        }

        /// Denounce a juror during the voting period for which the commitment vote is known.
        /// This is useful to punish the behaviour that jurors reveal
        /// their commitments before the voting period ends.
        /// A check of `commitment_hash == hash(juror ++ outcome ++ salt)` is performed for validation.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The identifier of the court.
        /// - `juror`: The juror whose commitment vote might be known.
        /// - `outcome`: The raw vote outcome which should match with the commitment of the juror.
        /// - `salt`: The hash which is used to proof that the juror did reveal
        /// her vote during the voting period.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of jurors
        /// in the list of random selections (draws).
        #[pallet::weight(T::WeightInfo::denounce_vote(T::MaxSelectedDraws::get()))]
        #[transactional]
        pub fn denounce_vote(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            juror: AccountIdLookupOf<T>,
            outcome: OutcomeReport,
            salt: T::Hash,
        ) -> DispatchResultWithPostInfo {
            let denouncer = ensure_signed(origin)?;

            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.matches_outcome_report(&outcome), Error::<T>::OutcomeMismatch);

            let juror = T::Lookup::lookup(juror)?;

            ensure!(denouncer != juror, Error::<T>::SelfDenounceDisallowed);

            ensure!(<Jurors<T>>::contains_key(&juror), Error::<T>::JurorDoesNotExist);

            let court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            let now = <frame_system::Pallet<T>>::block_number();
            // ensure in vote period
            ensure!(
                court.cycle_ends.pre_vote < now && now <= court.cycle_ends.vote,
                Error::<T>::NotInVotingPeriod
            );

            let mut draws = <SelectedDraws<T>>::get(market_id);
            match draws.binary_search_by_key(&juror, |draw| draw.juror.clone()) {
                Ok(index) => {
                    let draw = draws[index].clone();

                    let raw_commmitment =
                        RawCommitment { juror: juror.clone(), outcome: outcome.clone(), salt };

                    let commitment = Self::get_hashed_commitment(draw.vote, raw_commmitment)?;

                    // slash for the misbehaviour happens in reassign_juror_stakes
                    let raw_vote = Vote::Denounced { commitment, outcome: outcome.clone(), salt };
                    draws[index] = Draw { vote: raw_vote, ..draw };
                }
                Err(_) => return Err(Error::<T>::JurorNotDrawn.into()),
            }

            let draws_len = draws.len() as u32;

            <SelectedDraws<T>>::insert(market_id, draws);

            Self::deposit_event(Event::DenouncedJurorVote {
                denouncer,
                juror,
                market_id,
                outcome,
                salt,
            });

            Ok(Some(T::WeightInfo::denounce_vote(draws_len)).into())
        }

        /// Reveal the commitment vote of the caller juror.
        /// A check of `commitment_hash == hash(juror ++ outcome ++ salt)` is performed for validation.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The identifier of the court.
        /// - `outcome`: The raw vote outcome which should match with the commitment of the juror.
        /// - `salt`: The hash which is used for the validation.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of jurors
        /// in the list of random selections (draws).
        #[pallet::weight(T::WeightInfo::reveal_vote(T::MaxSelectedDraws::get()))]
        #[transactional]
        pub fn reveal_vote(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
            salt: T::Hash,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.matches_outcome_report(&outcome), Error::<T>::OutcomeMismatch);

            ensure!(<Jurors<T>>::get(&who).is_some(), Error::<T>::OnlyJurorsCanReveal);
            let court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            let now = <frame_system::Pallet<T>>::block_number();
            ensure!(
                court.cycle_ends.vote < now && now <= court.cycle_ends.aggregation,
                Error::<T>::NotInAggregationPeriod
            );

            let mut draws = <SelectedDraws<T>>::get(market_id);
            match draws.binary_search_by_key(&who, |draw| draw.juror.clone()) {
                Ok(index) => {
                    let draw = draws[index].clone();

                    let raw_commitment =
                        RawCommitment { juror: who.clone(), outcome: outcome.clone(), salt };

                    let commitment = Self::get_hashed_commitment(draw.vote, raw_commitment)?;

                    let raw_vote = Vote::Revealed { commitment, outcome: outcome.clone(), salt };
                    draws[index] = Draw { juror: who.clone(), vote: raw_vote, ..draw };
                }
                Err(_) => return Err(Error::<T>::JurorNotDrawn.into()),
            }

            let draws_len = draws.len() as u32;

            <SelectedDraws<T>>::insert(market_id, draws);

            Self::deposit_event(Event::JurorRevealedVote { juror: who, market_id, outcome, salt });

            Ok(Some(T::WeightInfo::reveal_vote(draws_len)).into())
        }

        /// Trigger an appeal for a court. The last appeal does not trigger a new court round
        /// but instead it marks the court mechanism for this market as failed.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The identifier of the court.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of jurors.
        /// It depends heavily on `choose_multiple_weighted` of `select_jurors`.
        #[pallet::weight(T::WeightInfo::appeal(
            T::MaxJurors::get(),
            T::MaxAppeals::get(),
            CacheSize::get(),
            CacheSize::get(),
        ))]
        #[transactional]
        pub fn appeal(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let mut court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            let appeal_number = court.appeals.len().saturating_add(1);
            ensure!(appeal_number <= T::MaxAppeals::get() as usize, Error::<T>::MaxAppealsReached);
            let bond = get_appeal_bond::<T>(appeal_number);
            ensure!(T::Currency::can_reserve(&who, bond), Error::<T>::AppealBondExceedsBalance);
            let now = <frame_system::Pallet<T>>::block_number();
            Self::check_appealable_market(&market_id, &court, now)?;

            // the outcome which would be resolved on is appealed (including oracle report)
            let old_draws = SelectedDraws::<T>::get(market_id);
            let appealed_outcome =
                Self::get_latest_resolved_outcome(&market_id, old_draws.as_slice())?;
            let appeal_info = AppealInfo { backer: who.clone(), bond, appealed_outcome };
            court.appeals.try_push(appeal_info).map_err(|_| {
                debug_assert!(false, "Appeal bound is checked above.");
                Error::<T>::MaxAppealsReached
            })?;

            let last_resolve_at = court.cycle_ends.appeal;

            // used for benchmarking, juror pool is queried inside `select_jurors`
            let jurors_len = <JurorPool<T>>::decode_len().unwrap_or(0) as u32;

            let mut ids_len_1 = 0u32;

            // if appeal_number == MaxAppeals, then don't start a new appeal round
            if appeal_number < T::MaxAppeals::get() as usize {
                let new_draws = Self::select_jurors(appeal_number)?;
                let request_block = <RequestBlock<T>>::get();
                debug_assert!(request_block >= now, "Request block must be greater than now.");
                let round_timing = RoundTiming {
                    pre_vote_end: request_block,
                    vote_period: T::CourtVotePeriod::get(),
                    aggregation_period: T::CourtAggregationPeriod::get(),
                    appeal_period: T::CourtAppealPeriod::get(),
                };
                // sets cycle_ends one after the other from now
                court.update_lifecycle(round_timing);
                let new_resolve_at = court.cycle_ends.appeal;
                debug_assert!(new_resolve_at != last_resolve_at);
                ids_len_1 = T::DisputeResolution::add_auto_resolve(&market_id, new_resolve_at)?;
                <SelectedDraws<T>>::insert(market_id, new_draws);
                Self::unlock_jurors_from_last_draw(&market_id, old_draws);
            }

            let ids_len_0 = T::DisputeResolution::remove_auto_resolve(&market_id, last_resolve_at);

            T::Currency::reserve_named(&Self::reserve_id(), &who, bond)?;

            <Courts<T>>::insert(market_id, court);

            let appeal_number = appeal_number as u32;
            Self::deposit_event(Event::MarketAppealed { market_id, appeal_number });

            Ok(Some(T::WeightInfo::appeal(jurors_len, appeal_number, ids_len_0, ids_len_1)).into())
        }

        /// The juror stakes get reassigned according to the plurality decision of the jurors.
        /// The losing jurors get slashed and pay for the winning jurors.
        /// The tardy or denounced jurors get slashed.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The identifier of the court.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of randomly selected jurors for this court.
        #[pallet::weight(T::WeightInfo::reassign_juror_stakes(T::MaxSelectedDraws::get()))]
        #[transactional]
        pub fn reassign_juror_stakes(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            let mut court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            let winner = match court.status {
                CourtStatus::Closed { winner } => winner,
                CourtStatus::Reassigned => return Err(Error::<T>::CourtAlreadyReassigned.into()),
                _ => return Err(Error::<T>::CourtNotClosed.into()),
            };

            let draws = SelectedDraws::<T>::get(market_id);
            let draws_len = draws.len() as u32;

            let reward_pot = Self::reward_pot(&market_id);
            let slash_juror = |ai: &T::AccountId, slashable: BalanceOf<T>| {
                let (imbalance, missing) = T::Currency::slash(ai, slashable);
                debug_assert!(
                    missing.is_zero(),
                    "Could not slash all of the amount for juror {:?}.",
                    ai
                );
                T::Currency::resolve_creating(&reward_pot, imbalance);
            };

            // map delegated jurors to own_slashable, outcome and Vec<(delegator, delegator_stake)>
            let mut jurors_to_stakes = BTreeMap::<T::AccountId, JurorVoteWithStakesOf<T>>::new();

            for draw in draws {
                if let Some(mut juror_info) = <Jurors<T>>::get(&draw.juror) {
                    juror_info.active_lock = juror_info.active_lock.saturating_sub(draw.slashable);
                    <Jurors<T>>::insert(&draw.juror, juror_info);
                } else {
                    log::warn!(
                        "Juror {:?} not found in Jurors storage (reassign_juror_stakes). Market \
                         id {:?}.",
                        draw.juror,
                        market_id
                    );
                    debug_assert!(false);
                }

                match draw.vote {
                    Vote::Drawn
                    | Vote::Secret { commitment: _ }
                    | Vote::Denounced { commitment: _, outcome: _, salt: _ } => {
                        slash_juror(&draw.juror, draw.slashable);
                    }
                    Vote::Revealed { commitment: _, outcome, salt: _ } => {
                        jurors_to_stakes.entry(draw.juror).or_default().self_info =
                            Some(SelfInfo { slashable: draw.slashable, outcome });
                    }
                    Vote::Delegated { delegated_stakes } => {
                        let delegator = draw.juror;
                        for (j, delegated_stake) in delegated_stakes {
                            // fill the delegations for each juror
                            // [(juror_0, [(delegator_0, delegator_stake_0), ...]),
                            // (juror_1, [(delegator_42, delegator_stake_42), ...]), ...]
                            let delegations =
                                &mut jurors_to_stakes.entry(j).or_default().delegations;
                            // future-proof binary search by key
                            // because many delegators can back one juror
                            // we might want to fastly find elements later on
                            match delegations.binary_search_by_key(&delegator, |(d, _)| d.clone()) {
                                Ok(i) => {
                                    debug_assert!(
                                        false,
                                        "Delegator {:?} already in delegations.",
                                        delegator
                                    );
                                    delegations[i].1.saturating_add(delegated_stake);
                                }
                                Err(i) => {
                                    delegations.insert(i, (delegator.clone(), delegated_stake));
                                }
                            }
                        }
                    }
                }
            }

            Self::slash_losers_to_award_winners(&market_id, jurors_to_stakes, &winner);

            court.status = CourtStatus::Reassigned;
            <Courts<T>>::insert(market_id, court);

            <SelectedDraws<T>>::remove(market_id);

            Self::deposit_event(Event::JurorStakesReassigned { market_id });

            Ok(Some(T::WeightInfo::reassign_juror_stakes(draws_len)).into())
        }

        #[pallet::weight(T::WeightInfo::set_inflation())]
        #[transactional]
        pub fn set_inflation(origin: OriginFor<T>, inflation: Perbill) -> DispatchResult {
            T::MonetaryGovernanceOrigin::ensure_origin(origin)?;

            <YearlyInflation<T>>::put(inflation);

            Ok(())
        }
    }

    impl<T> Pallet<T>
    where
        T: Config,
    {
        // Handle the external incentivisation of the court system.
        pub(crate) fn handle_inflation(now: T::BlockNumber) -> Weight {
            let inflation_period = T::InflationPeriod::get();
            if (now % inflation_period).is_zero() {
                let yearly_inflation_rate = <YearlyInflation<T>>::get();
                let yearly_inflation_amount = yearly_inflation_rate * T::Currency::total_issuance();
                let blocks_per_year = T::BlocksPerYear::get()
                    .saturated_into::<u128>()
                    .saturated_into::<BalanceOf<T>>();
                debug_assert!(!T::BlocksPerYear::get().is_zero());
                let issue_per_block = yearly_inflation_amount / blocks_per_year.max(One::one());

                let inflation_period_mint = issue_per_block.saturating_mul(
                    inflation_period.saturated_into::<u128>().saturated_into::<BalanceOf<T>>(),
                );

                let jurors = <JurorPool<T>>::get();
                let jurors_len = jurors.len() as u32;
                let total_stake = jurors.iter().fold(0u128, |acc, pool_item| {
                    acc.saturating_add(pool_item.stake.saturated_into::<u128>())
                });
                for JurorPoolItem { stake, juror, .. } in jurors {
                    let share =
                        Perquintill::from_rational(stake.saturated_into::<u128>(), total_stake);
                    let mint = share * inflation_period_mint.saturated_into::<u128>();
                    if let Ok(imb) = T::Currency::deposit_into_existing(
                        &juror,
                        mint.saturated_into::<BalanceOf<T>>(),
                    ) {
                        Self::deposit_event(Event::MintedInCourt {
                            juror: juror.clone(),
                            amount: imb.peek(),
                        });
                    }
                }

                return T::WeightInfo::handle_inflation(jurors_len);
            }

            Weight::zero()
        }

        // Get `n` unique and ordered random numbers from the random number generator.
        // If the generator returns three times the same number in a row, an error is returned.
        pub(crate) fn get_n_random_numbers(
            n: usize,
            max: u128,
        ) -> Result<BTreeSet<u128>, DispatchError> {
            let mut rng = Self::rng();

            let mut random_set = BTreeSet::new();
            let mut insert_unused_random_number = || -> DispatchResult {
                let mut count = 0u8;
                // this loop is to make sure we don't insert the same random number twice
                while !random_set.insert(rng.gen_range(0u128..=max)) {
                    count = count.saturating_add(1u8);
                    if count >= 3u8 {
                        return Err(Error::<T>::RandNumGenFailed.into());
                    }
                }
                Ok(())
            };

            for _ in 0..n {
                insert_unused_random_number()?;
            }

            debug_assert!(random_set.len() == n);

            Ok(random_set)
        }

        // Get the sum of the unconsumed stake from all jurors in the pool.
        // The unconsumed stake is the stake that was not already locked in previous courts.
        fn get_unconsumed_stake_sum(jurors: &[JurorPoolItemOf<T>]) -> u128 {
            jurors
                .iter()
                .map(|pool_item| {
                    pool_item
                        .stake
                        .saturating_sub(pool_item.consumed_stake)
                        .saturated_into::<u128>()
                })
                .sum::<u128>()
        }

        // Returns the added active lock amount.
        // The added active lock amount is noted in the Jurors map.
        fn update_active_lock(
            juror: &T::AccountId,
            selections: &BTreeMap<T::AccountId, SelectionValueOf<T>>,
        ) -> BalanceOf<T> {
            if let Some(SelectionValue {
                weight: _,
                slashable: total_lock_added,
                delegated_stakes: _,
            }) = selections.get(juror)
            {
                if let Some(mut juror_info) = <Jurors<T>>::get(juror) {
                    juror_info.active_lock =
                        juror_info.active_lock.saturating_add(*total_lock_added);
                    <Jurors<T>>::insert(juror, juror_info);
                } else {
                    debug_assert!(false, "Juror should exist in the Jurors map");
                }
                return *total_lock_added;
            }

            <BalanceOf<T>>::zero()
        }

        // Returns true, if `n` is greater or equal to `lower_bound` and less than `upper_bound`.
        // Returns false, otherwise.
        fn in_range(n: u128, lower_bound: u128, upper_bound: u128) -> bool {
            debug_assert!(lower_bound <= upper_bound);
            if lower_bound <= n && n < upper_bound {
                return true;
            }
            false
        }

        fn add_delegated_juror(
            mut delegated_stakes: DelegatedStakesOf<T>,
            delegated_juror: &T::AccountId,
            amount: BalanceOf<T>,
        ) -> DelegatedStakesOf<T> {
            match delegated_stakes.binary_search_by_key(&delegated_juror, |(j, _)| j) {
                Ok(index) => {
                    delegated_stakes[index].1 = delegated_stakes[index].1.saturating_add(amount);
                }
                Err(index) => {
                    let _ = delegated_stakes
                        .try_insert(index, (delegated_juror.clone(), amount))
                        .map_err(|_| {
                            debug_assert!(
                                false,
                                "BoundedVec insertion should not fail, because the length of \
                                 jurors is ensured for delegations."
                            );
                        });
                }
            }

            delegated_stakes
        }

        // Updates the `selections` map for the juror and the lock amount.
        // If `juror` does not already exist in `selections`,
        // the vote weight is set to 1 and the lock amount is initially set.
        // For each call on the same juror, the vote weight is incremented by one
        // and the lock amount is added to the previous amount.
        fn update_selections(
            selections: &mut BTreeMap<T::AccountId, SelectionValueOf<T>>,
            juror: &T::AccountId,
            sel_add: SelectionAdd<AccountIdOf<T>, BalanceOf<T>>,
        ) {
            if let Some(SelectionValue { weight, slashable, delegated_stakes }) =
                selections.get_mut(juror)
            {
                match sel_add {
                    SelectionAdd::SelfStake { lock } => {
                        *weight = weight.saturating_add(1);
                        *slashable = slashable.saturating_add(lock);
                    }
                    SelectionAdd::DelegationStake { delegated_juror, lock } => {
                        *slashable = slashable.saturating_add(lock);
                        *delegated_stakes = Self::add_delegated_juror(
                            delegated_stakes.clone(),
                            &delegated_juror,
                            lock,
                        );
                    }
                    SelectionAdd::DelegationWeight => {
                        *weight = weight.saturating_add(1);
                    }
                };
            } else {
                match sel_add {
                    SelectionAdd::SelfStake { lock } => {
                        selections.insert(
                            juror.clone(),
                            SelectionValue {
                                weight: 1,
                                slashable: lock,
                                delegated_stakes: Default::default(),
                            },
                        );
                    }
                    SelectionAdd::DelegationStake { delegated_juror, lock } => {
                        let delegated_stakes = Self::add_delegated_juror(
                            DelegatedStakesOf::<T>::default(),
                            &delegated_juror,
                            lock,
                        );
                        selections.insert(
                            juror.clone(),
                            SelectionValue { weight: 0, slashable: lock, delegated_stakes },
                        );
                    }
                    SelectionAdd::DelegationWeight => {
                        selections.insert(
                            juror.clone(),
                            SelectionValue {
                                weight: 1,
                                slashable: <BalanceOf<T>>::zero(),
                                delegated_stakes: Default::default(),
                            },
                        );
                    }
                };
            }
        }

        fn get_valid_delegated_juror(
            delegations: &[T::AccountId],
            random_number: u128,
        ) -> Option<T::AccountId> {
            let jurors: JurorPoolOf<T> = JurorPool::<T>::get();
            let mut delegated_juror = None;

            for count in 0..delegations.len() {
                let delegation_index = (random_number.saturating_add(count as u128)
                    % delegations.len() as u128) as usize;
                delegated_juror = match delegations.get(delegation_index) {
                    Some(del_j) => Some(del_j.clone()),
                    None => {
                        log::error!("Delegation with modulo index should exist!");
                        debug_assert!(false);
                        None
                    }
                };

                if let Some(del_j) = &delegated_juror {
                    if let Some(delegated_juror_info) = <Jurors<T>>::get(del_j) {
                        if delegated_juror_info.delegations.is_some() {
                            // skip if delegated juror is delegator herself
                            continue;
                        }
                        if Self::get_pool_item(&jurors, delegated_juror_info.stake, del_j).is_some()
                        {
                            delegated_juror = Some(del_j.clone());
                            break;
                        }
                    }
                }
            }

            delegated_juror
        }

        fn add_to_selections(
            selections: &mut BTreeMap<T::AccountId, SelectionValueOf<T>>,
            juror: &T::AccountId,
            lock_added: BalanceOf<T>,
            random_number: u128,
        ) -> Result<(), SelectionError> {
            let delegations_opt =
                <Jurors<T>>::get(juror.clone()).and_then(|juror_info| juror_info.delegations);
            match delegations_opt {
                Some(delegations) => {
                    let delegated_juror =
                        Self::get_valid_delegated_juror(delegations.as_slice(), random_number)
                            .ok_or(SelectionError::NoValidDelegatedJuror)?;

                    // delegated juror gets the vote weight
                    let sel_add = SelectionAdd::DelegationWeight;
                    Self::update_selections(selections, &delegated_juror, sel_add);

                    let sel_add = SelectionAdd::DelegationStake {
                        delegated_juror: delegated_juror.clone(),
                        lock: lock_added,
                    };
                    // delegator risks his stake (to delegated juror), but gets no vote weight
                    Self::update_selections(selections, juror, sel_add);
                }
                None => {
                    let sel_add = SelectionAdd::SelfStake { lock: lock_added };
                    Self::update_selections(selections, juror, sel_add);
                }
            }

            Ok(())
        }

        // Match the random numbers to select some jurors from the pool.
        // The active lock (and consumed stake) of the selected jurors
        // is increased by the random selection weight.
        fn get_selections(
            jurors: &mut JurorPoolOf<T>,
            mut random_set: BTreeSet<u128>,
        ) -> BTreeMap<T::AccountId, SelectionValueOf<T>> {
            let mut selections = BTreeMap::<T::AccountId, SelectionValueOf<T>>::new();

            let mut invalid_juror_indices = Vec::<usize>::new();

            let mut current_weight = 0u128;
            for (i, pool_item) in jurors.iter_mut().enumerate() {
                let lower_bound = current_weight;
                let mut unconsumed = pool_item.stake.saturating_sub(pool_item.consumed_stake);
                let upper_bound =
                    current_weight.saturating_add(unconsumed.saturated_into::<u128>());

                // this always gets the lowest random number first and maybe removes it
                let unchangable_random_numbers = random_set.clone();
                for random_number in unchangable_random_numbers.iter() {
                    if Self::in_range(*random_number, lower_bound, upper_bound) {
                        let lock_added = unconsumed.min(T::MinJurorStake::get());
                        unconsumed = unconsumed.saturating_sub(lock_added);

                        match Self::add_to_selections(
                            &mut selections,
                            &pool_item.juror,
                            lock_added,
                            *random_number,
                        ) {
                            Ok(()) => {}
                            Err(SelectionError::NoValidDelegatedJuror) => {
                                // it would be pretty expensive to request another selection
                                // so just ignore this missing MinJurorStake
                                // I mean we also miss MinJurorStake in the case
                                // if the juror fails to vote or reveal or gets denounced
                                invalid_juror_indices.push(i);
                            }
                        }

                        random_set.remove(random_number);
                    } else {
                        break;
                    }
                }

                let total_lock_added = Self::update_active_lock(&pool_item.juror, &selections);
                pool_item.consumed_stake =
                    pool_item.consumed_stake.saturating_add(total_lock_added);

                if random_set.is_empty() {
                    break;
                }

                current_weight = upper_bound;
            }

            for i in invalid_juror_indices {
                jurors.remove(i);
            }

            selections
        }

        // Converts the `selections` map into a vector of `Draw` structs.
        fn convert_selections_to_draws(
            selections: BTreeMap<T::AccountId, SelectionValueOf<T>>,
        ) -> Vec<DrawOf<T>> {
            selections
                .into_iter()
                .map(|(juror, SelectionValue { weight, slashable, delegated_stakes })| Draw {
                    juror,
                    weight,
                    vote: if !delegated_stakes.is_empty() {
                        debug_assert!(
                            weight.is_zero(),
                            "Jurors who delegated shouldn't have voting weight."
                        );
                        Vote::Delegated { delegated_stakes }
                    } else {
                        Vote::Drawn
                    },
                    slashable,
                })
                .collect()
        }

        // Choose `number` of jurors from the pool randomly
        // according to the weighted stake of the jurors.
        // Return the random draws.
        pub(crate) fn choose_multiple_weighted(
            number: usize,
        ) -> Result<Vec<DrawOf<T>>, DispatchError> {
            let mut jurors = <JurorPool<T>>::get();
            let total_weight = Self::get_unconsumed_stake_sum(jurors.as_slice());
            let random_set = Self::get_n_random_numbers(number, total_weight)?;
            let selections = Self::get_selections(&mut jurors, random_set);
            <JurorPool<T>>::put(jurors);

            Ok(Self::convert_selections_to_draws(selections))
        }

        // Reduce the active lock of the jurors from the last draw.
        // This is useful so that the jurors can thaw their non-locked stake.
        fn unlock_jurors_from_last_draw(market_id: &MarketIdOf<T>, last_draws: SelectedDrawsOf<T>) {
            // keep in mind that the old draw likely contains different jurors
            for old_draw in last_draws {
                if let Some(mut juror_info) = <Jurors<T>>::get(&old_draw.juror) {
                    juror_info.active_lock =
                        juror_info.active_lock.saturating_sub(old_draw.slashable);
                    <Jurors<T>>::insert(&old_draw.juror, juror_info);
                } else {
                    log::warn!(
                        "Juror {:?} not found in Jurors storage (unlock_jurors_from_last_draw). \
                         Market id {:?}.",
                        old_draw.juror,
                        market_id
                    );
                    debug_assert!(false);
                }
            }
        }

        // TODO benchmark select_jurors explicitly for future comparisons
        // Selects the jurors for the next round.
        // The `consumed_stake` in `JurorPool` and `active_lock` in `Jurors` is increased
        // equally according to the weight inside the `new_draws`.
        // With increasing `consumed_stake` the probability to get selected
        // in further court rounds shrinks.
        //
        // Returns the new draws.
        pub(crate) fn select_jurors(
            appeal_number: usize,
        ) -> Result<SelectedDrawsOf<T>, DispatchError> {
            let jurors_len = <JurorPool<T>>::decode_len().unwrap_or(0);
            let necessary_jurors_weight = Self::necessary_jurors_weight(appeal_number);
            ensure!(jurors_len >= necessary_jurors_weight, Error::<T>::NotEnoughJurors);

            let random_jurors = Self::choose_multiple_weighted(necessary_jurors_weight)?;
            debug_assert!(
                random_jurors.windows(2).all(|window| window[0].juror <= window[1].juror),
                "The vector is not sorted by the juror field"
            );

            debug_assert!(
                random_jurors.len() <= T::MaxSelectedDraws::get() as usize,
                "The number of randomly selected jurors should be less than or equal to \
                 `MaxSelectedDraws`."
            );

            // TODO what is the maximum number of draws with delegations? It is using necessary_jurors_weight (the number of atoms / draw weight) for the last round
            // new appeal round should have a fresh set of draws
            Ok(<SelectedDrawsOf<T>>::truncate_from(random_jurors))
        }

        // Returns (index, pool_item) if the pool item is part of the juror pool.
        // It returns None otherwise.
        pub(crate) fn get_pool_item<'a>(
            jurors: &'a [JurorPoolItemOf<T>],
            stake: BalanceOf<T>,
            juror: &T::AccountId,
        ) -> Option<(usize, &'a JurorPoolItemOf<T>)> {
            if let Ok(i) = jurors.binary_search_by_key(&(stake, juror), |pool_item| {
                (pool_item.stake, &pool_item.juror)
            }) {
                return Some((i, &jurors[i]));
            }
            // this None case can happen whenever the juror decided to leave the court
            // or was kicked out of the juror pool because of the lowest stake
            None
        }

        // Returns OK if the market is in a valid state to be appealed.
        // Returns an error otherwise.
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
                court.cycle_ends.aggregation < now && now < court.cycle_ends.appeal,
                Error::<T>::NotInAppealPeriod
            );

            Ok(())
        }

        /// The reserve ID of the court pallet.
        #[inline]
        pub fn reserve_id() -> [u8; 8] {
            T::CourtPalletId::get().0
        }

        /// The account ID which is used to reward the correct jurors.
        #[inline]
        pub fn reward_pot(market_id: &MarketIdOf<T>) -> T::AccountId {
            T::CourtPalletId::get().into_sub_account_truncating(market_id)
        }

        /// The account ID of the treasury.
        #[inline]
        pub(crate) fn treasury_account_id() -> T::AccountId {
            T::TreasuryPalletId::get().into_account_truncating()
        }

        // Get a random seed based on a nonce.
        pub(crate) fn get_random_seed(nonce: u64) -> [u8; 32] {
            debug_assert!(
                !<frame_system::Pallet<T>>::block_number().is_zero(),
                "When testing with the randomness of the collective flip pallet it produces a \
                 underflow (block number substraction by one) panic if the block number is zero."
            );
            let mut seed = [0; 32];
            let (random_hash, _) = T::Random::random(&nonce.to_le_bytes());
            seed.copy_from_slice(&random_hash.as_ref()[..32]);
            seed
        }

        // Returns a cryptographically secure random number generator
        // implementation based on the seed provided by the `Config::Random` type
        // and the `JurorsSelectionNonce` storage.
        pub(crate) fn rng() -> impl RngCore {
            let nonce = <JurorsSelectionNonce<T>>::mutate(|n| {
                let rslt = *n;
                *n = n.wrapping_add(1);
                rslt
            });
            let random_seed = Self::get_random_seed(nonce);
            ChaCha20Rng::from_seed(random_seed)
        }

        // Calculates the necessary number of draws depending on the number of market appeals.
        pub(crate) fn necessary_jurors_weight(appeals_len: usize) -> usize {
            // 2^(appeals_len) * 5 + 2^(appeals_len) - 1
            // MaxAppeals - 1 (= 3) example: 2^3 * 5 + 2^3 - 1 = 47
            APPEAL_BASIS
                .saturating_pow(appeals_len as u32)
                .saturating_mul(INITIAL_DRAWS_NUM)
                .saturating_add(APPEAL_BASIS.saturating_pow(appeals_len as u32).saturating_sub(1))
        }

        // Slash the losers and use the slashed amount plus the reward pot to reward the winners.
        fn slash_losers_to_award_winners(
            market_id: &MarketIdOf<T>,
            jurors_to_stakes: BTreeMap<T::AccountId, JurorVoteWithStakesOf<T>>,
            winner_outcome: &OutcomeReport,
        ) {
            let mut total_incentives = <NegativeImbalanceOf<T>>::zero();

            let slash_all_delegators =
                |delegations: &[(T::AccountId, BalanceOf<T>)]| -> NegativeImbalanceOf<T> {
                    let mut total_imb = <NegativeImbalanceOf<T>>::zero();
                    for (delegator, d_slashable) in delegations.iter() {
                        let (imb, missing) = T::Currency::slash(delegator, *d_slashable);
                        total_imb.subsume(imb);
                        debug_assert!(
                            missing.is_zero(),
                            "Could not slash all of the amount for delegator {:?}.",
                            delegator
                        );
                    }
                    total_imb
                };

            let mut total_winner_stake = BalanceOf::<T>::zero();
            let mut winners = Vec::<(T::AccountId, BalanceOf<T>)>::new();
            for (juror, JurorVoteWithStakes { self_info, delegations }) in jurors_to_stakes.iter() {
                match self_info {
                    Some(SelfInfo { slashable, outcome }) => {
                        if outcome == winner_outcome {
                            winners.push((juror.clone(), *slashable));
                            total_winner_stake = total_winner_stake.saturating_add(*slashable);

                            winners.extend(delegations.clone());
                            let total_delegation_stake = delegations
                                .iter()
                                .fold(BalanceOf::<T>::zero(), |acc, (_, delegator_stake)| {
                                    acc.saturating_add(*delegator_stake)
                                });
                            total_winner_stake =
                                total_winner_stake.saturating_add(total_delegation_stake);
                        } else {
                            let (imb, missing) = T::Currency::slash(juror, *slashable);
                            total_incentives.subsume(imb);
                            debug_assert!(
                                missing.is_zero(),
                                "Could not slash all of the amount for juror {:?}.",
                                juror
                            );

                            let imb = slash_all_delegators(delegations.as_slice());
                            total_incentives.subsume(imb);
                        }
                    }
                    None => {
                        // in this case the delegators have delegated their vote
                        // to a tardy or denounced juror
                        let imb = slash_all_delegators(delegations.as_slice());
                        total_incentives.subsume(imb);
                    }
                }
            }

            // reward from denounce slashes and tardy jurors of this market / court
            let reward_pot = Self::reward_pot(market_id);
            let reward = T::Currency::free_balance(&reward_pot);
            let (imb, missing) = T::Currency::slash(&reward_pot, reward);
            debug_assert!(missing.is_zero(), "Could not slash all of the amount for reward pot.");
            total_incentives.subsume(imb);

            let total_reward = total_incentives.peek();
            for (winner, risked_amount) in winners {
                let r = risked_amount.saturated_into::<u128>();
                let t = total_winner_stake.saturated_into::<u128>();
                let share = Perquintill::from_rational(r, t);
                let reward_per_each = (share * total_reward.saturated_into::<u128>())
                    .saturated_into::<BalanceOf<T>>();
                let (actual_reward, leftover) = total_incentives.split(reward_per_each);
                total_incentives = leftover;
                T::Currency::resolve_creating(&winner, actual_reward);
            }

            if !total_incentives.peek().is_zero() {
                // if there are no winners reward the treasury
                T::Slash::on_unbalanced(total_incentives);
            }
        }

        // Returns the winner of the current court round.
        // If there is no element inside `draws`, returns `None`.
        // If the best two outcomes have the same score, returns the last court round winner.
        pub(crate) fn get_winner(
            draws: &[DrawOf<T>],
            last_winner: Option<OutcomeReport>,
        ) -> Option<OutcomeReport> {
            let mut scores = BTreeMap::<OutcomeReport, u32>::new();

            for draw in draws {
                if let Vote::Revealed { commitment: _, outcome, salt: _ } = &draw.vote {
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
                    let new_second = best_score;
                    best_score = second;
                    new_second
                } else {
                    second
                }
            } else {
                return Some(best_score.0.clone());
            };

            for el in iter {
                if el.1 > best_score.1 {
                    second_best_score = best_score;
                    best_score = el;
                } else if el.1 > second_best_score.1 {
                    second_best_score = el;
                }
            }

            if best_score.1 == second_best_score.1 {
                return last_winner;
            }

            Some(best_score.0.clone())
        }

        // Returns the outcome, on which the market would resolve
        // if the current court round is the final (not appealed) court round.
        pub(crate) fn get_latest_resolved_outcome(
            market_id: &MarketIdOf<T>,
            last_draws: &[DrawOf<T>],
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
            let resolved_outcome =
                Self::get_winner(last_draws, last_winner).unwrap_or(oracle_outcome);
            Ok(resolved_outcome)
        }

        // Check if the (juror, outcome, salt) combination matches the secret hash of the vote.
        pub(crate) fn is_valid(commitment_matcher: CommitmentMatcherOf<T>) -> DispatchResult {
            // market id and current appeal number is part of salt generation
            // salt should be signed by the juror (market_id ++ appeal number)
            // salt can be reproduced only be the juror address
            // with knowing market_id and appeal number
            // so even if the salt is forgotten it can be reproduced only by the juror
            let CommitmentMatcher {
                hashed: commitment,
                raw: RawCommitment { juror, outcome, salt },
            } = commitment_matcher;

            ensure!(
                commitment == T::Hashing::hash_of(&(juror, outcome, salt)),
                Error::<T>::InvalidReveal
            );

            Ok(())
        }

        // Convert the raw commitment to a hashed commitment,
        // and check if it matches with the secret hash of the vote.
        // Otherwise return an error.
        pub(crate) fn get_hashed_commitment(
            vote: VoteOf<T>,
            raw_commitment: RawCommitmentOf<T>,
        ) -> Result<T::Hash, DispatchError> {
            match vote {
                Vote::Secret { commitment } => {
                    let commitment_matcher =
                        CommitmentMatcher { hashed: commitment, raw: raw_commitment };
                    Self::is_valid(commitment_matcher)?;
                    Ok(commitment)
                }
                Vote::Drawn => Err(Error::<T>::JurorNotVoted.into()),
                Vote::Delegated { delegated_stakes: _ } => Err(Error::<T>::JurorDelegated.into()),
                Vote::Revealed { commitment: _, outcome: _, salt: _ } => {
                    Err(Error::<T>::VoteAlreadyRevealed.into())
                }
                Vote::Denounced { commitment: _, outcome: _, salt: _ } => {
                    Err(Error::<T>::VoteAlreadyDenounced.into())
                }
            }
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

            let appeal_number = 0usize;
            let new_draws = Self::select_jurors(appeal_number)?;

            let now = <frame_system::Pallet<T>>::block_number();
            let request_block = <RequestBlock<T>>::get();
            debug_assert!(request_block >= now, "Request block must be greater than now.");
            let round_timing = RoundTiming {
                pre_vote_end: request_block,
                vote_period: T::CourtVotePeriod::get(),
                aggregation_period: T::CourtAggregationPeriod::get(),
                appeal_period: T::CourtAppealPeriod::get(),
            };

            // sets cycle_ends one after the other from now
            let court = CourtInfo::new(round_timing);

            let _ids_len =
                T::DisputeResolution::add_auto_resolve(market_id, court.cycle_ends.appeal)?;

            <SelectedDraws<T>>::insert(market_id, new_draws);
            <Courts<T>>::insert(market_id, court);

            Ok(())
        }

        fn on_resolution(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Option<OutcomeReport>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            let mut court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;
            let draws = SelectedDraws::<T>::get(market_id);
            let resolved_outcome = Self::get_latest_resolved_outcome(market_id, draws.as_slice())?;
            Self::unlock_jurors_from_last_draw(market_id, draws);
            court.status = CourtStatus::Closed { winner: resolved_outcome.clone() };
            <Courts<T>>::insert(market_id, court);

            Ok(Some(resolved_outcome))
        }

        fn exchange(
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
                    let (imb, missing) =
                        T::Currency::slash_reserved_named(&Self::reserve_id(), backer, *bond);
                    debug_assert!(missing.is_zero());
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

            Ok(<Courts<T>>::get(market_id).map(|court| court.cycle_ends.appeal))
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
                    let necessary_jurors_weight = Self::necessary_jurors_weight(appeal_number);
                    let valid_period =
                        Self::check_appealable_market(market_id, &court, now).is_ok();

                    if appeals.is_full() || (valid_period && (jurors_len < necessary_jurors_weight))
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

                    let necessary_jurors_weight = Self::necessary_jurors_weight(0usize);
                    if during_dispute_duration && jurors_len < necessary_jurors_weight {
                        has_failed = true;
                    }
                }
            }

            Ok(has_failed)
        }

        fn on_global_dispute(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Vec<GlobalDisputeItem<Self::AccountId, Self::Balance>>, DispatchError> {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            let court = <Courts<T>>::get(market_id).ok_or(Error::<T>::CourtNotFound)?;

            let report = market.report.as_ref().ok_or(Error::<T>::MarketReportNotFound)?;
            let oracle_outcome = &report.outcome;

            let gd_outcomes = court
                .appeals
                .iter()
                // oracle outcome is added by pm pallet
                .filter(|a| &a.appealed_outcome != oracle_outcome)
                .map(|a| {
                    GlobalDisputeItem {
                        outcome: a.appealed_outcome.clone(),
                        // we have no better global dispute outcome owner
                        owner: Self::treasury_account_id(),
                        // initial vote amount
                        initial_vote_amount: <BalanceOf<T>>::zero(),
                    }
                })
                .collect::<Vec<GlobalDisputeItem<Self::AccountId, Self::Balance>>>();

            let old_draws = SelectedDraws::<T>::get(market_id);
            Self::unlock_jurors_from_last_draw(market_id, old_draws);
            <SelectedDraws<T>>::remove(market_id);
            <Courts<T>>::remove(market_id);

            Ok(gd_outcomes)
        }

        fn clear(market_id: &Self::MarketId, market: &MarketOf<T>) -> DispatchResult {
            ensure!(
                market.dispute_mechanism == MarketDisputeMechanism::Court,
                Error::<T>::MarketDoesNotHaveCourtMechanism
            );

            let old_draws = SelectedDraws::<T>::get(market_id);
            Self::unlock_jurors_from_last_draw(market_id, old_draws);
            <SelectedDraws<T>>::remove(market_id);
            <Courts<T>>::remove(market_id);

            Ok(())
        }
    }

    impl<T> CourtPalletApi for Pallet<T> where T: Config {}

    // No-one can bound more than BalanceOf<T>, therefore, this functions saturates
    pub fn get_appeal_bond<T>(n: usize) -> BalanceOf<T>
    where
        T: Config,
    {
        T::AppealBond::get().saturating_mul(
            (APPEAL_BOND_BASIS.saturating_pow(n as u32)).saturated_into::<BalanceOf<T>>(),
        )
    }
}

impl<T: Config> Pallet<T> {
    fn do_join_court(
        who: &T::AccountId,
        amount: BalanceOf<T>,
        delegations: Option<DelegationsOf<T>>,
    ) -> Result<u32, DispatchError> {
        ensure!(amount >= T::MinJurorStake::get(), Error::<T>::BelowMinJurorStake);
        let free_balance = T::Currency::free_balance(who);
        ensure!(amount <= free_balance, Error::<T>::AmountExceedsBalance);

        let mut jurors = JurorPool::<T>::get();

        let (active_lock, consumed_stake) = if let Some(prev_juror_info) = <Jurors<T>>::get(who) {
            ensure!(amount > prev_juror_info.stake, Error::<T>::AmountBelowLastJoin);
            let (index, pool_item) = Self::get_pool_item(&jurors, prev_juror_info.stake, who)
                .ok_or(Error::<T>::JurorNeedsToExit)?;
            debug_assert!(
                prev_juror_info.prepare_exit_at.is_none(),
                "If the pool item is found, the prepare_exit_at could have never been written."
            );
            let consumed_stake = pool_item.consumed_stake;
            jurors.remove(index);
            (prev_juror_info.active_lock, consumed_stake)
        } else {
            if jurors.is_full() {
                let lowest_item = jurors.first();
                let lowest_stake = lowest_item
                    .map(|pool_item| pool_item.stake)
                    .unwrap_or_else(<BalanceOf<T>>::zero);
                debug_assert!({
                    let mut sorted = jurors.clone();
                    sorted.sort_by_key(|pool_item| (pool_item.stake, pool_item.juror.clone()));
                    jurors.len() == sorted.len()
                        && jurors
                            .iter()
                            .zip(sorted.iter())
                            .all(|(a, b)| lowest_stake <= a.stake && a == b)
                });
                ensure!(amount > lowest_stake, Error::<T>::AmountBelowLowestJuror);
                // remove the lowest staked juror
                jurors.remove(0);
            }
            (<BalanceOf<T>>::zero(), <BalanceOf<T>>::zero())
        };

        match jurors
            .binary_search_by_key(&(amount, who), |pool_item| (pool_item.stake, &pool_item.juror))
        {
            Ok(_) => {
                debug_assert!(
                    false,
                    "This should never happen, because we are removing the juror above."
                );
                return Err(Error::<T>::JurorTwiceInPool.into());
            }
            Err(i) => jurors
                .try_insert(i, JurorPoolItem { stake: amount, juror: who.clone(), consumed_stake })
                .map_err(|_| {
                    debug_assert!(
                        false,
                        "This should never happen, because we are removing the lowest staked \
                         juror above."
                    );
                    Error::<T>::MaxJurorsReached
                })?,
        };

        T::Currency::set_lock(T::CourtLockId::get(), who, amount, WithdrawReasons::all());

        let jurors_len = jurors.len() as u32;
        JurorPool::<T>::put(jurors);

        let juror_info =
            JurorInfoOf::<T> { stake: amount, active_lock, prepare_exit_at: None, delegations };
        <Jurors<T>>::insert(who, juror_info);

        Ok(jurors_len)
    }
}
