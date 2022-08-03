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
        pallet_prelude::{DispatchResultWithPostInfo, OptionQuery, StorageDoubleMap, StorageMap},
        traits::{
            Currency, ExistenceRequirement, Get, IsType, LockIdentifier, LockableCurrency,
            WithdrawReasons,
        },
        storage::child::KillStorageResult,
        Blake2_128Concat, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::traits::{AccountIdConversion, Saturating, Zero};
    use zeitgeist_primitives::types::OutcomeReport;
    use zrml_market_commons::MarketCommonsPalletApi;

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
            let sender = ensure_signed(origin)?;

            ensure!(Self::is_started(&market_id), Error::<T>::NoGlobalDisputeStarted);
            if let Some((_winner_outcome, _winner_vote_balance, is_finished)) = <Winners<T>>::get(market_id) {
                ensure!(!is_finished, Error::<T>::GlobalDisputeAlreadyFinished);
            }

            ensure!(
                <OutcomeVotes<T>>::get(market_id, outcome.clone()).is_none(),
                Error::<T>::OutcomeAlreadyExists
            );

            let voting_outcome_fee = T::VotingOutcomeFee::get();

            let reward_account = T::GlobalDisputesPalletId::get().into_sub_account(market_id);

            T::Currency::transfer(
                &sender,
                &reward_account,
                voting_outcome_fee,
                ExistenceRequirement::AllowDeath,
            )?;

            Self::push_voting_outcome(&market_id, outcome.clone(), voting_outcome_fee);

            <OutcomeOwner<T>>::insert(market_id, outcome.clone(), sender);

            Self::deposit_event(Event::AddedVotingOutcome(market_id, outcome));
            Ok(().into())
        }

        #[frame_support::transactional]
        #[pallet::weight(5000)]
        pub fn reward_outcome_owner(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            if let Some((winner_outcome, _winner_vote_balance, is_finished)) =
                <Winners<T>>::get(market_id)
            {
                ensure!(is_finished, Error::<T>::UnfinishedGlobalDispute);
                let reward_account = T::GlobalDisputesPalletId::get().into_sub_account(market_id);
                for (outcome, account) in <OutcomeOwner<T>>::drain_prefix(market_id)
                    .take(T::RemoveKeysLimit::get() as usize)
                {
                    if outcome == winner_outcome {
                        let reward_account_free_balance =
                            T::Currency::free_balance(&reward_account);
                        // Reward the loosing funds to the winner without charging a transfer fee
                        if !reward_account_free_balance.is_zero() {
                            let _ = T::Currency::resolve_into_existing(
                                &account,
                                T::Currency::withdraw(
                                    &reward_account,
                                    reward_account_free_balance,
                                    WithdrawReasons::TRANSFER,
                                    ExistenceRequirement::AllowDeath,
                                )?,
                            );
                            Self::deposit_event(Event::OutcomeOwnerRewarded(market_id));
                        }
                    }
                }
                if <OutcomeOwner<T>>::iter_prefix(market_id).take(1).next().is_some() {
                    Self::deposit_event(Event::OutcomeOwnersPartiallyCleaned(market_id));
                } else {
                    Self::deposit_event(Event::OutcomeOwnersFullyCleaned(market_id));
                }
                match <OutcomeVotes<T>>::remove_prefix(market_id, Some(T::RemoveKeysLimit::get())) {
                    KillStorageResult::AllRemoved(_) => Self::deposit_event(Event::OutcomeVotesFullyCleaned(market_id)),
                    KillStorageResult::SomeRemaining(_) => Self::deposit_event(Event::OutcomeVotesPartiallyCleaned(market_id)),
                }
            }

            Ok(().into())
        }

        /// Votes on an outcome on a vote identifier with an `amount`.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::vote_on_outcome())]
        pub fn vote_on_outcome(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(amount <= T::Currency::free_balance(&sender), Error::<T>::InsufficientAmount);
            ensure!(amount >= T::MinOutcomeVoteAmount::get(), Error::<T>::AmountTooLow);

            ensure!(Self::is_started(&market_id), Error::<T>::NoGlobalDisputeStarted);

            let mut outcome_vote_sum = <OutcomeVotes<T>>::get(market_id, &outcome)
                .ok_or(Error::<T>::OutcomeDoesNotExist)?;

            <LockInfoOf<T>>::mutate(&sender, market_id, |lock_info| {
                let mut add_to_outcome_sum = |a| {
                    outcome_vote_sum = outcome_vote_sum.saturating_add(a);
                    <Winners<T>>::mutate(market_id, |highest| {
                        *highest = Some(highest.clone().map_or(
                            (outcome.clone(), outcome_vote_sum, false),
                            |(prev_outcome, prev_highest_sum, _is_finished)| {
                                if outcome_vote_sum >= prev_highest_sum {
                                    (outcome.clone(), outcome_vote_sum, false)
                                } else {
                                    (prev_outcome, prev_highest_sum, false)
                                }
                            },
                        ));
                    });
                    <OutcomeVotes<T>>::insert(market_id, &outcome, outcome_vote_sum);
                };
                if let Some(prev_highest_amount) = lock_info {
                    if amount >= *prev_highest_amount {
                        let diff = amount.saturating_sub(*prev_highest_amount);
                        add_to_outcome_sum(diff);
                        *lock_info = Some(amount);
                    }
                } else {
                    add_to_outcome_sum(amount);
                    *lock_info = Some(amount);
                }
            });

            T::Currency::extend_lock(
                T::VoteLockIdentifier::get(),
                &sender,
                amount,
                WithdrawReasons::TRANSFER,
            );

            Self::deposit_event(Event::VotedOnOutcome(market_id, outcome, amount));
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
            for (market_id, locked_balance) in <LockInfoOf<T>>::iter_prefix(&voter) {
                // true is pattern matching for is_finished
                if let Some((_winner_outcome, _winner_vote_balance, true)) =
                    <Winners<T>>::get(market_id)
                {
                    resolved_ids.push(market_id);
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
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

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

        /// The maximum number of keys to remove from a storage map.
        #[pallet::constant]
        type RemoveKeysLimit: Get<u32>;

        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The outcome specified with vote id and outcome index is not present.
        OutcomeDoesNotExist,
        /// Sender does not have enough funds for the vote on an outcome.
        InsufficientAmount,
        /// The global dispute period is not over yet. The winner is not yet determined.
        UnfinishedGlobalDispute,
        /// Sender tried to vote with an amount below a defined minium.
        AmountTooLow,
        /// No global dispute present at the moment.
        NoGlobalDisputeStarted,
        /// The voting outcome has been already added.
        OutcomeAlreadyExists,
        /// The global dispute is already over.
        GlobalDisputeAlreadyFinished,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A vote happened on an outcome. \[market_id, outcome, vote_amount\]
        VotedOnOutcome(MarketIdOf<T>, OutcomeReport, BalanceOf<T>),
        /// A new outcome has been pushed. \[market_id, outcome_report\]
        AddedVotingOutcome(MarketIdOf<T>, OutcomeReport),
        /// The outcome owner has been rewarded. \[market_id\]
        OutcomeOwnerRewarded(MarketIdOf<T>),
        /// The outcome votes storage item is partially cleaned. So there are some missing. \[market_id\]
        OutcomeVotesPartiallyCleaned(MarketIdOf<T>),
        /// The outcome votes storage item is fully cleaned. \[market_id\]
        OutcomeVotesFullyCleaned(MarketIdOf<T>),
        /// The outcome owners storage item is fully cleaned. \[market_id\]
        OutcomeOwnersPartiallyCleaned(MarketIdOf<T>),
        /// The outcome owners storage item is fully cleaned. \[market_id\]
        OutcomeOwnersFullyCleaned(MarketIdOf<T>),
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
        ) {
            let update_winner = |b| {
                <Winners<T>>::mutate(market_id, |highest| {
                    *highest = Some(highest.clone().map_or(
                        (outcome.clone(), b, false),
                        |(prev_outcome, prev_highest_sum, _is_finished)| {
                            if b >= prev_highest_sum {
                                (outcome.clone(), b, false)
                            } else {
                                (prev_outcome, prev_highest_sum, false)
                            }
                        },
                    ));
                });
            };
            match <OutcomeVotes<T>>::get(market_id, outcome.clone()) {
                Some(b) => {
                    let outcome_sum = b.saturating_add(vote_balance);
                    update_winner(outcome_sum);
                    <OutcomeVotes<T>>::insert(market_id, outcome, outcome_sum);
                }
                None => {
                    update_winner(vote_balance);
                    <OutcomeVotes<T>>::insert(market_id, outcome, vote_balance);
                }
            }
        }

        /// Determine the outcome with the most amount of tokens.
        fn get_voting_winner(market_id: &MarketIdOf<T>) -> Option<OutcomeReport> {
            let winner = <Winners<T>>::get(market_id);

            if let Some((outcome, vote_balance, _is_finished)) = &winner {
                <Winners<T>>::insert(market_id, (outcome, vote_balance, true));
            }

            winner.map(|(outcome, _, _)| outcome)
        }

        fn is_started(market_id: &MarketIdOf<T>) -> bool {
            <Winners<T>>::get(market_id).is_some()
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    pub type Winners<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        (OutcomeReport, BalanceOf<T>, bool),
        OptionQuery,
    >;

    /// Maps the vote id to the outcome index and the vote balance.  
    #[pallet::storage]
    pub type OutcomeVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        Blake2_128Concat,
        OutcomeReport,
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
        BalanceOf<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type OutcomeOwner<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        Blake2_128Concat,
        OutcomeReport,
        T::AccountId,
        OptionQuery,
    >;
}
