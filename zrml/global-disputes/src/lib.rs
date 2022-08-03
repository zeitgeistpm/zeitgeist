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
            DispatchResultWithPostInfo, OptionQuery, StorageDoubleMap, StorageMap, ValueQuery,
        },
        traits::{
            Currency, Get, IsType, LockIdentifier, LockableCurrency, NamedReservableCurrency,
            WithdrawReasons,
        },
        Blake2_128Concat, BoundedVec, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::{
        traits::{One, Saturating, Zero},
        DispatchError, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        constants::GlobalDisputesPalletId,
        types::{OutcomeIndex, OutcomeReport},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    pub(crate) const RESERVE_ID: [u8; 8] = GlobalDisputesPalletId::get().0;

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Push an outcome to the global dispute system to allow anybody to vote on.
        #[frame_support::transactional]
        #[pallet::weight(5000)]
        pub fn add_vote_outcome(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let voting_outcome_fee = T::VotingOutcomeFee::get();
            ensure!(
                voting_outcome_fee >= T::Currency::free_balance(&who),
                Error::<T>::InsufficientAmount
            );

            ensure!(
                <Outcomes<T>>::get(market_id).len() >= One::one(),
                Error::<T>::NoGlobalDisputeStarted
            );

            Self::push_voting_outcome(&market_id, outcome.clone(), voting_outcome_fee)?;

            // save the reserve and the account with this market id
            T::Currency::reserve_named(&RESERVE_ID, &who, voting_outcome_fee)?;

            Self::deposit_event(Event::PushedVotingOutcome(market_id, outcome));
            Ok(().into())
        }

        /// Votes on an outcome on a vote identifier with an `amount`.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::vote_on_outcome())]
        pub fn vote_on_outcome(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] outcome_index: OutcomeIndex,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(amount <= T::Currency::free_balance(&sender), Error::<T>::InsufficientAmount);
            ensure!(amount >= T::MinOutcomeVoteAmount::get(), Error::<T>::AmountTooLow);

            let outcomes = <Outcomes<T>>::get(market_id);
            let outcome_number = outcomes.len();
            ensure!(outcome_number >= One::one(), Error::<T>::NoGlobalDisputeStarted);

            ensure!(
                outcome_number >= T::MinOutcomes::get() as usize,
                Error::<T>::NotEnoughOutcomes
            );

            let mut outcome_vote_sum = <OutcomeVotes<T>>::get(market_id, outcome_index).unwrap_or(Zero::zero());

            let outcome: &OutcomeReport = outcomes.get(outcome_index as usize).ok_or(Error::<T>::OutcomeDoesNotExist)?;

            <LockInfoOf<T>>::mutate(&sender, market_id, |lock_info| {
                let mut add_to_outcome_sum = |a| {
                    outcome_vote_sum = outcome_vote_sum.saturating_add(a);
                    <Winners<T>>::mutate(market_id, |highest| {
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
                    <OutcomeVotes<T>>::insert(market_id, outcome_index, outcome_vote_sum);
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

            Self::deposit_event(Event::VotedOnOutcome(market_id, outcome_index, amount));
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
            for (market_id, (outcome_index, locked_balance)) in <LockInfoOf<T>>::iter_prefix(&voter)
            {
                if <Winners<T>>::get(market_id).is_none() {
                    resolved_ids.push(market_id);
                    if <OutcomeVotes<T>>::get(market_id, outcome_index).is_some() {
                        // TODO if there is no lock for the outcome index, then the storage for this is never removed
                        // TODO maybe think about removing prefix with a limit of 5
                        <OutcomeVotes<T>>::remove(market_id, outcome_index);
                    }
                    continue;
                }
                lock_needed = lock_needed.max(locked_balance);
            }

            for market_id in resolved_ids {
                <LockInfoOf<T>>::remove(&voter, market_id);
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

        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The currency to allow locking funds for voting.
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>
            + NamedReservableCurrency<Self::AccountId, ReserveIdentifier = [u8; 8]>;

        /// The pallet identifier.
        #[pallet::constant]
        type GlobalDisputesPalletId: Get<PalletId>;

        /// The vote lock identifier for a voting outcome.
        #[pallet::constant]
        type VoteLockIdentifier: Get<LockIdentifier>;

        /// The minimum required amount to vote on an outcome.
        #[pallet::constant]
        type MinOutcomeVoteAmount: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type VotingOutcomeFee: Get<BalanceOf<Self>>;

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
        /// The number of maximum outcomes is reached.
        MaxOutcomeLimitReached,
        /// No global dispute present at the moment.
        NoGlobalDisputeStarted,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A vote happened on an outcome. \[market_id, outcome_index, vote_amount\]
        VotedOnOutcome(MarketIdOf<T>, u32, BalanceOf<T>),
        /// A new outcome has been pushed. \[market_id, outcome_report\]
        PushedVotingOutcome(MarketIdOf<T>, OutcomeReport),
    }

    impl<T> GlobalDisputesPalletApi<MarketIdOf<T>, BalanceOf<T>> for Pallet<T>
    where
        T: Config,
    {
        /// Add outcomes (with initial vote balance) to the voting mechanism.
        fn push_voting_outcome(
            market_id: &MarketIdOf<T>,
            outcome: OutcomeReport,
            vote_balance: BalanceOf<T>,
        ) -> Result<(), DispatchError> {
            let mut outcomes = <Outcomes<T>>::get(market_id);
            ensure!(outcomes.try_push(outcome).is_ok(), Error::<T>::MaxOutcomeLimitReached);
            let outcome_index = outcomes.len().saturated_into::<u32>().saturating_sub(One::one());
            <Winners<T>>::mutate(market_id, |highest| {
                *highest = Some(highest.map_or(
                    (outcome_index, vote_balance),
                    |(prev_i, prev_highest_sum)| {
                        if vote_balance >= prev_highest_sum {
                            (outcome_index, vote_balance)
                        } else {
                            (prev_i, prev_highest_sum)
                        }
                    },
                ));
            });
            <Outcomes<T>>::insert(market_id, outcomes);
            <OutcomeVotes<T>>::insert(market_id, outcome_index, vote_balance);
            Ok(())
        }

        /// Determine the outcome with the most amount of tokens.
        fn get_voting_winner(market_id: &MarketIdOf<T>) -> Option<OutcomeReport> {
            let winning_outcome_index =
                <Winners<T>>::get(market_id).map(|(i, _)| i as usize).unwrap_or(0usize);

            let winning_outcome = <Outcomes<T>>::get(market_id).get(winning_outcome_index).cloned();

            <Outcomes<T>>::remove(market_id);
            <Winners<T>>::remove(market_id);

            winning_outcome
        }

        fn is_started(market_id: &MarketIdOf<T>) -> bool {
            <Outcomes<T>>::get(market_id).len() >= One::one()
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    pub type Winners<T: Config> =
        StorageMap<_, Blake2_128Concat, MarketIdOf<T>, (OutcomeIndex, BalanceOf<T>), OptionQuery>;

    /// Maps the vote id to the outcome reports.
    #[pallet::storage]
    pub type Outcomes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        BoundedVec<OutcomeReport, T::MaxOutcomeLimit>,
        ValueQuery,
    >;

    /// Maps the vote id to the outcome index and the vote balance.  
    #[pallet::storage]
    pub type OutcomeVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        Blake2_128Concat,
        OutcomeIndex,
        BalanceOf<T>,
        OptionQuery,
    >;

    /// All highest lock information (vote id, outcome index and locked balance) for a particular voter.
    ///
    /// TWOX-NOTE: SAFE as `AccountId`s are crypto hashes anyway.
    #[pallet::storage]
    pub type LockInfoOf<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::AccountId,
        Blake2_128Concat,
        MarketIdOf<T>,
        (OutcomeIndex, BalanceOf<T>),
        OptionQuery,
    >;
}
