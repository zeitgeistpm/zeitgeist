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
    use core::{cmp::Ordering, marker::PhantomData};
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
    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Votes on an outcome on a vote identifier with an `amount`.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::vote_on_dispute())]
        pub fn vote_on_outcome(
            origin: OriginFor<T>,
            #[pallet::compact] vote_id: VoteId,
            outcome_index: u32,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(amount <= T::Currency::free_balance(&sender), Error::<T>::InsufficientAmount);
            ensure!(amount >= T::MinOutcomeVoteAmount::get(), Error::<T>::AmountTooLow);

            ensure!(
                <Outcomes<T>>::get(vote_id).len() >= T::MinOutcomes::get() as usize,
                Error::<T>::NotEnoughOutcomes
            );

            let vote_balance = <OutcomeVotes<T>>::get(vote_id, outcome_index)
                .ok_or(Error::<T>::OutcomeDoesNotExist)?;

            <LockInfoOf<T>>::mutate(&sender, vote_id, |lock_info| {
                let sub_vote_balance = |i, a| {
                    Self::update_vote_balance(
                        vote_id,
                        vote_balance,
                        i,
                        a,
                        <BalanceOf<T>>::saturating_sub,
                    );
                };
                let add_vote_balance = |i, a| {
                    Self::update_vote_balance(
                        vote_id,
                        vote_balance,
                        i,
                        a,
                        <BalanceOf<T>>::saturating_add,
                    );
                };
                if let Some((prev_index, prev_amount)) = lock_info {
                    if outcome_index != *prev_index {
                        sub_vote_balance(*prev_index, amount);
                        add_vote_balance(outcome_index, amount);
                    } else {
                        match amount.cmp(prev_amount) {
                            Ordering::Greater => {
                                let diff = amount.saturating_sub(*prev_amount);
                                add_vote_balance(outcome_index, diff);
                            }
                            Ordering::Less => {
                                let diff = prev_amount.saturating_sub(amount);
                                <OutcomeVotes<T>>::insert(
                                    vote_id,
                                    outcome_index,
                                    vote_balance.saturating_sub(diff),
                                );
                                sub_vote_balance(outcome_index, diff);
                            }
                            Ordering::Equal => (),
                        }
                    }
                } else {
                    add_vote_balance(outcome_index, amount);
                }
                *lock_info = Some((outcome_index, amount));
            });

            T::Currency::extend_lock(
                T::VoteLockIdentifier::get(),
                &sender,
                amount,
                WithdrawReasons::TRANSFER,
            );

            Self::deposit_event(Event::VotedOnOutcome(vote_id, outcome_index, amount));
            Ok(Some(T::WeightInfo::vote_on_dispute()).into())
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
            for (vote_id, (_, locked_balance)) in <LockInfoOf<T>>::iter_prefix(&voter) {
                if <OutcomeVotes<T>>::iter_prefix(vote_id).take(1).next().is_none() {
                    resolved_ids.push(vote_id);
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
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A vote happened on an outcome. \[vote_id, outcome_index, vote_amount\]
        VotedOnOutcome(VoteId, u32, BalanceOf<T>),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    impl<T: Config> Pallet<T> {
        fn get_default_outcome_and_index(vote_id: VoteId) -> Option<(u32, OutcomeReport)> {
            // return first element if the BoundedVec is not empty, otherwise None
            <Outcomes<T>>::get(vote_id).get(0usize).map(|o| (0u32, o.clone()))
        }

        fn get_outcome_index_for_same_balance(x: u32, y: u32) -> u32 {
            // return more recent element => is last added, so the higher index
            x.max(y)
        }

        fn update_vote_balance<F>(
            vote_id: VoteId,
            vote_balance: BalanceOf<T>,
            outcome_index: u32,
            amount: BalanceOf<T>,
            saturating_f: F,
        ) where
            F: FnOnce(BalanceOf<T>, BalanceOf<T>) -> BalanceOf<T>,
        {
            <OutcomeVotes<T>>::insert(vote_id, outcome_index, saturating_f(vote_balance, amount));
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

        /// Add outcomes (with initial vote balance) to the voting mechanism on the latest vote id.
        fn push_voting_outcome(
            outcome: OutcomeReport,
            vote_balance: Self::Balance,
        ) -> Result<(), DispatchError> {
            let vote_id = Self::get_latest_vote_id();
            let mut outcomes = <Outcomes<T>>::get(vote_id);
            let mut outcome_index: Option<u32> = None;
            for (i, o) in outcomes.iter().enumerate() {
                if *o == outcome {
                    outcome_index = Some(i.saturated_into::<u32>());
                    break;
                }
            }
            match outcome_index {
                None => {
                    ensure!(outcomes.try_push(outcome).is_ok(), Error::<T>::MaxOutcomeLimitReached);
                    let outcome_index =
                        outcomes.len().saturated_into::<u32>().saturating_sub(One::one());
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
            let (default_outcome_index, default_outcome) =
                Self::get_default_outcome_and_index(vote_id).ok_or(<Error<T>>::NoDefaultOutcome)?;
            let (winning_outcome_index, _) = <OutcomeVotes<T>>::drain_prefix(vote_id).fold(
                (default_outcome_index, <BalanceOf<T>>::zero()),
                |(o0, b0), (o1, b1)| match b0.cmp(&b1) {
                    Ordering::Greater => (o0, b0),
                    Ordering::Less => (o1, b1),
                    Ordering::Equal => (Self::get_outcome_index_for_same_balance(o0, o1), b0),
                },
            );

            let winning_outcome = <Outcomes<T>>::get(vote_id)
                .get(winning_outcome_index as usize)
                .cloned()
                .unwrap_or(default_outcome);

            <Outcomes<T>>::remove(vote_id);

            Ok(winning_outcome)
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// Use unique identifier to allow multiple global disputes on the same market id.
    #[pallet::storage]
    pub type NextVoteId<T: Config> = StorageValue<_, VoteId, ValueQuery>;

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
        u32,
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
        (u32, BalanceOf<T>),
        OptionQuery,
    >;
}
