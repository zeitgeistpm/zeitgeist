//! # Global disputes
//!
//! Manages market disputes and resolutions.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod benchmarks;
mod global_disputes_pallet_api;
mod mock;
mod tests;
pub mod weights;

pub use global_disputes_pallet_api::GlobalDisputesPalletApi;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{weights::WeightInfoZeitgeist, GlobalDisputesPalletApi};
    use alloc::vec::Vec;
    use core::marker::PhantomData;
    use frame_support::{
        ensure,
        pallet_prelude::{
            DispatchResultWithPostInfo, OptionQuery, StorageDoubleMap, StorageMap, StorageValue,
            ValueQuery,
        },
        traits::{Currency, Get, Hooks, IsType, LockIdentifier, LockableCurrency, WithdrawReasons},
        Blake2_128Concat, BoundedVec, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::{
        traits::{One, Saturating, Zero},
        DispatchError, SaturatedConversion,
    };
    use zeitgeist_primitives::types::OutcomeReport;

    pub(crate) type VoteId = u128;
    pub(crate) type OutcomeIndex = u128;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Votes on an outcome on a vote identifier with an `amount`.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::vote_on_outcome())]
        pub fn vote_on_outcome(
            origin: OriginFor<T>,
            #[pallet::compact] vote_id: VoteId,
            outcome_index: OutcomeIndex,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(amount <= T::Currency::free_balance(&sender), Error::<T>::InsufficientAmount);
            ensure!(amount >= T::MinOutcomeVoteAmount::get(), Error::<T>::AmountTooLow);

            ensure!(
                <Outcomes<T>>::get(vote_id).len() >= T::MinOutcomes::get() as usize,
                Error::<T>::NotEnoughOutcomes
            );

            let mut outcome_vote_sum = <OutcomeVotes<T>>::get(vote_id, outcome_index)
                .ok_or(Error::<T>::OutcomeDoesNotExist)?;

            <LockInfoOf<T>>::mutate(&sender, vote_id, |lock_info| {
                let mut add_to_outcome_sum = |a| {
                    outcome_vote_sum = outcome_vote_sum.saturating_add(a);
                    <HighestVotes<T>>::mutate(vote_id, |highest| {
                        *highest = Some(highest.map_or(
                            (outcome_index, outcome_vote_sum),
                            |(prev_i, prev_highest_sum)| {
                                if outcome_vote_sum >= prev_highest_sum {
                                    (outcome_index, outcome_vote_sum)
                                } else {
                                    (prev_i, prev_highest_sum)
                                }
                            },
                        ));
                    });
                    <OutcomeVotes<T>>::insert(vote_id, outcome_index, outcome_vote_sum);
                };
                if let Some((prev_index, prev_highest_amount)) = lock_info {
                    if amount >= *prev_highest_amount {
                        if outcome_index == *prev_index {
                            let diff = amount.saturating_sub(*prev_highest_amount);
                            add_to_outcome_sum(diff);
                        } else {
                            add_to_outcome_sum(amount);
                        }
                        *lock_info = Some((outcome_index, amount));
                    }
                } else {
                    add_to_outcome_sum(amount);
                    *lock_info = Some((outcome_index, amount));
                }
            });

            T::Currency::extend_lock(
                T::VoteLockIdentifier::get(),
                &sender,
                amount,
                WithdrawReasons::TRANSFER,
            );

            Self::deposit_event(Event::VotedOnOutcome(vote_id, outcome_index, amount));
            Ok(Some(T::WeightInfo::vote_on_outcome()).into())
        }

        /// Unlock the expired (winner chosen) vote values.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::unlock_vote_balance())]
        pub fn unlock_vote_balance(
            origin: OriginFor<T>,
            voter: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            let mut lock_needed: BalanceOf<T> = Zero::zero();
            let mut resolved_ids = Vec::new();
            for (vote_id, (outcome_index, locked_balance)) in <LockInfoOf<T>>::iter_prefix(&voter) {
                if <HighestVotes<T>>::get(vote_id).is_none() {
                    resolved_ids.push(vote_id);
                    if <OutcomeVotes<T>>::get(vote_id, outcome_index).is_some() {
                        <OutcomeVotes<T>>::remove(vote_id, outcome_index);
                    }
                    continue;
                }
                lock_needed = lock_needed.max(locked_balance);
            }

            for vote_id in resolved_ids {
                <LockInfoOf<T>>::remove(&voter, vote_id);
            }

            if lock_needed.is_zero() {
                T::Currency::remove_lock(T::VoteLockIdentifier::get(), &voter);
            } else {
                T::Currency::set_lock(
                    T::VoteLockIdentifier::get(),
                    &voter,
                    lock_needed,
                    WithdrawReasons::TRANSFER,
                );
            }

            Ok(Some(T::WeightInfo::unlock_vote_balance()).into())
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The currency to allow locking funds for voting.
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

        /// The pallet identifier.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The vote lock identifier for a voting outcome.
        #[pallet::constant]
        type VoteLockIdentifier: Get<LockIdentifier>;

        /// The minimum required amount to vote on an outcome.
        #[pallet::constant]
        type MinOutcomeVoteAmount: Get<BalanceOf<Self>>;

        /// The minimum number of outcomes required to allow voting.
        #[pallet::constant]
        type MinOutcomes: Get<u32>;

        /// The maximum number of possible outcomes.
        #[pallet::constant]
        type MaxOutcomeLimit: Get<u32>;

        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The vote on this outcome index is not allowed, because there are not at least a minimum number of outcomes.
        NotEnoughOutcomes,
        /// The outcome specified with vote id and outcome index is not present.
        OutcomeDoesNotExist,
        /// Sender does not have enough funds for the vote on an outcome.
        InsufficientAmount,
        /// Sender tried to vote with an amount below a defined minium.
        AmountTooLow,
        /// There is no default outcome set in the first place to resolve to.
        NoDefaultOutcome,
        /// The number of maximum outcomes is reached.
        MaxOutcomeLimitReached,
        /// The maximum number of vote id's is reached.
        MaxVoteIds,
        /// There are no votes to determine the winner.
        NoVotesPresent,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A vote happened on an outcome. \[vote_id, outcome_index, vote_amount\]
        VotedOnOutcome(VoteId, u128, BalanceOf<T>),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    impl<T: Config> Pallet<T> {
        fn get_default_outcome(vote_id: VoteId) -> Option<OutcomeReport> {
            // return first element if the BoundedVec is not empty, otherwise None
            <Outcomes<T>>::get(vote_id).get(0usize).map(|o| o.clone())
        }
    }

    impl<T> GlobalDisputesPalletApi for Pallet<T>
    where
        T: Config,
    {
        type Balance = BalanceOf<T>;

        fn get_latest_vote_id() -> VoteId {
            <NextVoteId<T>>::get().saturating_sub(One::one())
        }

        /// For each new voting, this associated function needs to get called to allow pushing outcomes on the new vote id.
        fn get_next_vote_id() -> Result<VoteId, DispatchError> {
            let vote_id = <NextVoteId<T>>::get();
            let new_vote_id = vote_id.checked_add(One::one()).ok_or(Error::<T>::MaxVoteIds)?;
            <NextVoteId<T>>::put(new_vote_id);
            Ok(vote_id)
        }

        // TODO use market id for push as parameter (to push exactly on market) but also keep in mind to allow multiple global disputes on the same market id
        /// Add outcomes (with initial vote balance) to the voting mechanism on the latest vote id.
        fn push_voting_outcome(
            outcome: OutcomeReport,
            vote_balance: Self::Balance,
        ) -> Result<(), DispatchError> {
            let vote_id = Self::get_latest_vote_id();
            let mut outcomes = <Outcomes<T>>::get(vote_id);
            let mut outcome_index: Option<u128> = None;
            for (i, o) in outcomes.iter().enumerate() {
                if *o == outcome {
                    outcome_index = Some(i.saturated_into::<u128>());
                    break;
                }
            }
            match outcome_index {
                None => {
                    ensure!(outcomes.try_push(outcome).is_ok(), Error::<T>::MaxOutcomeLimitReached);
                    let outcome_index =
                        outcomes.len().saturated_into::<u128>().saturating_sub(One::one());
                    <Outcomes<T>>::insert(vote_id, outcomes);
                    <OutcomeVotes<T>>::insert(vote_id, outcome_index, vote_balance);
                }
                Some(i) => {
                    if let Some(prev_vote_balance) = <OutcomeVotes<T>>::get(vote_id, i) {
                        <OutcomeVotes<T>>::insert(
                            vote_id,
                            i,
                            prev_vote_balance.saturating_add(vote_balance),
                        );
                    }
                }
            }
            Ok(())
        }

        /// Determine the outcome with the most amount of tokens.
        fn get_voting_winner(vote_id: VoteId) -> Result<OutcomeReport, DispatchError> {
            let default_outcome =
                Self::get_default_outcome(vote_id).ok_or(<Error<T>>::NoDefaultOutcome)?;
            let (winning_outcome_index, _) =
                <HighestVotes<T>>::get(vote_id).ok_or(<Error<T>>::NoVotesPresent)?;

            let winning_outcome = <Outcomes<T>>::get(vote_id)
                .get(winning_outcome_index as usize)
                .cloned()
                .unwrap_or(default_outcome);

            <Outcomes<T>>::remove(vote_id);
            <HighestVotes<T>>::remove(vote_id);

            Ok(winning_outcome)
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// Use unique identifier to allow multiple global disputes on the same market id.
    #[pallet::storage]
    pub type NextVoteId<T: Config> = StorageValue<_, VoteId, ValueQuery>;

    #[pallet::storage]
    pub type HighestVotes<T: Config> =
        StorageMap<_, Blake2_128Concat, VoteId, (OutcomeIndex, BalanceOf<T>), OptionQuery>;

    /// Maps the vote id to the outcome reports.
    #[pallet::storage]
    pub type Outcomes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        VoteId,
        BoundedVec<OutcomeReport, T::MaxOutcomeLimit>,
        ValueQuery,
    >;

    /// Maps the vote id to the outcome index and the vote balance.  
    #[pallet::storage]
    pub type OutcomeVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        VoteId,
        Blake2_128Concat,
        OutcomeIndex,
        BalanceOf<T>,
        OptionQuery,
    >;

    /// All lock information (vote id, outcome index and locked balance) for a particular voter.
    ///
    /// TWOX-NOTE: SAFE as `AccountId`s are crypto hashes anyway.
    #[pallet::storage]
    pub type LockInfoOf<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::AccountId,
        Blake2_128Concat,
        VoteId,
        (OutcomeIndex, BalanceOf<T>),
        OptionQuery,
    >;
}
