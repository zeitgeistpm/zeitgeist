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
    use core::marker::PhantomData;
    use frame_support::{
        ensure, log,
        pallet_prelude::{
            Decode, DispatchError, DispatchResultWithPostInfo, Encode, MaxEncodedLen, OptionQuery,
            StorageDoubleMap, StorageMap, TypeInfo, ValueQuery,
        },
        sp_runtime::{traits::StaticLookup, RuntimeDebug},
        storage::child::KillStorageResult,
        traits::{
            Currency, ExistenceRequirement, Get, IsType, LockIdentifier, LockableCurrency,
            WithdrawReasons,
        },
        Blake2_128Concat, BoundedVec, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedDiv, Saturating, Zero},
        DispatchResult,
    };
    use sp_std::{vec, vec::Vec};
    use zeitgeist_primitives::types::OutcomeReport;
    use zrml_market_commons::MarketCommonsPalletApi;

    pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    pub type OutcomeInfoOf<T> = OutcomeInfo<BalanceOf<T>, OwnerInfoOf<T>>;
    pub type WinnerInfoOf<T> = WinnerInfo<BalanceOf<T>, OwnerInfoOf<T>>;
    type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
    type OwnerInfoOf<T> = BoundedVec<AccountIdOf<T>, <T as Config>::MaxOwners>;
    type LockInfoOf<T> = LockInfo<MarketIdOf<T>, BalanceOf<T>>;

    #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
    pub struct LockInfo<MarketId, Balance>(pub Vec<(MarketId, Balance)>);

    impl<MarketId, Balance> Default for LockInfo<MarketId, Balance> {
        fn default() -> Self {
            Self(Vec::new())
        }
    }

    impl<MarketId, Balance> MaxEncodedLen for LockInfo<MarketId, Balance>
    where
        Balance: MaxEncodedLen,
        MarketId: MaxEncodedLen,
    {
        fn max_encoded_len() -> usize {
            MarketId::max_encoded_len().saturating_add(Balance::max_encoded_len())
        }
    }

    /// The information about a voting outcome of a global dispute.
    #[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
    pub struct OutcomeInfo<Balance, OwnerInfo> {
        /// The current sum of all locks on this outcome.
        pub outcome_sum: Balance,
        /// The vector of owners of the outcome.
        pub owners: OwnerInfo,
    }

    /// The information about the current highest winning outcome.
    #[derive(TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
    pub struct WinnerInfo<Balance, OwnerInfo> {
        /// The outcome, which is in the lead.
        pub outcome: OutcomeReport,
        /// The information about the winning outcome.
        pub outcome_info: OutcomeInfo<Balance, OwnerInfo>,
        /// Check, if the global dispute is finished.
        pub is_finished: bool,
    }

    impl<Balance: Saturating, OwnerInfo: Default> WinnerInfo<Balance, OwnerInfo> {
        pub fn new(outcome: OutcomeReport, vote_sum: Balance) -> Self {
            let outcome_info = OutcomeInfo { outcome_sum: vote_sum, owners: Default::default() };
            WinnerInfo { outcome, is_finished: false, outcome_info }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add voting outcome to a global dispute in exchange for a constant fee.
        /// Errors if the voting outcome already exists or
        /// if the global dispute has not started or has already finished.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of owner(s) of the outcome
        /// in the case that this gets called for an already existing outcome.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::add_vote_outcome(T::MaxOwners::get()))]
        pub fn add_vote_outcome(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
        ) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(origin)?;

            let winner_info =
                <Winners<T>>::get(market_id).ok_or(Error::<T>::NoGlobalDisputeStarted)?;
            ensure!(!winner_info.is_finished, Error::<T>::GlobalDisputeAlreadyFinished);

            ensure!(
                <Outcomes<T>>::get(market_id, &outcome).is_none(),
                Error::<T>::OutcomeAlreadyExists
            );

            let voting_outcome_fee = T::VotingOutcomeFee::get();

            Self::push_voting_outcome(&market_id, outcome.clone(), &owner, voting_outcome_fee)?;

            let reward_account = Self::reward_account(&market_id);

            T::Currency::transfer(
                &owner,
                &reward_account,
                voting_outcome_fee,
                ExistenceRequirement::AllowDeath,
            )?;

            Self::deposit_event(Event::AddedVotingOutcome { market_id, owner, outcome });
            // charge weight for successfully have no owners in Winners and no owners in empty Outcomes
            Ok((Some(T::WeightInfo::add_vote_outcome(0u32))).into())
        }

        /// Reward the collected fees to the owner(s) of a voting outcome.
        /// Fails if the global dispute is not concluded yet.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n + m)`, where `n` is the number of all existing outcomes for a global dispute,
        /// and `m` is the number of owners for the winning outcome.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::reward_outcome_owner(
            T::MaxOwners::get(),
            T::RemoveKeysLimit::get()
        ))]
        pub fn reward_outcome_owner(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            let (owners_len, removed_keys_amount) = <Winners<T>>::try_mutate(
                market_id,
                |winner_info: &mut Option<WinnerInfoOf<T>>| -> Result<(u32, u32), DispatchError> {
                    let mut winner_info =
                        winner_info.as_mut().ok_or(Error::<T>::NoGlobalDisputeStarted)?;
                    ensure!(winner_info.is_finished, Error::<T>::UnfinishedGlobalDispute);

                    // Outcome can be None when the second call happens in case RemoveKeysLimit is not reached
                    if let Some(outcome_info) = <Outcomes<T>>::get(market_id, &winner_info.outcome)
                    {
                        // storage write is needed here in case, that the first call of reward_outcome_owner doesn't reward the owners
                        // this can happen if there are more than RemoveKeysLimit keys (for KillStorageResult::SomeRemaining)
                        winner_info.outcome_info = outcome_info;
                    }

                    match <Outcomes<T>>::remove_prefix(market_id, Some(T::RemoveKeysLimit::get())) {
                        KillStorageResult::AllRemoved(removed_keys_amount) => {
                            let reward_account = Self::reward_account(&market_id);
                            let reward_account_free_balance =
                                T::Currency::free_balance(&reward_account);
                            let owners_len = winner_info.outcome_info.owners.len() as u32;
                            let at_least_one_owner_str = "Global Disputes: There should be always \
                                                          at least one owner for a voting outcome.";
                            debug_assert!(owners_len != 0u32, "{}", at_least_one_owner_str);
                            if reward_account_free_balance.is_zero() {
                                Self::deposit_event(Event::OutcomesFullyCleaned { market_id });
                                return Ok((owners_len, removed_keys_amount));
                            }
                            let mut remainder = reward_account_free_balance;
                            let owners_len_in_balance: BalanceOf<T> =
                                <BalanceOf<T>>::from(owners_len);
                            if let Some(reward_per_each) =
                                reward_account_free_balance.checked_div(&owners_len_in_balance)
                            {
                                for winner in winner_info.outcome_info.owners.iter() {
                                    // *Should* always be equal to `reward_per_each`
                                    let reward = remainder.min(reward_per_each);
                                    remainder = remainder.saturating_sub(reward);
                                    // Reward the loosing funds to the winners without charging a transfer fee
                                    let _ = T::Currency::resolve_into_existing(
                                        winner,
                                        T::Currency::withdraw(
                                            &reward_account,
                                            reward,
                                            WithdrawReasons::TRANSFER,
                                            ExistenceRequirement::AllowDeath,
                                        )?,
                                    );
                                }
                            } else {
                                log::error!("{}", at_least_one_owner_str);
                            }
                            if !remainder.is_zero() {
                                log::error!(
                                    "Global Disputes: The reward remainder for the market id {:?} 
                                    should be zero after the reward process. Reward remainder \
                                     amount: {:?}",
                                    &market_id,
                                    remainder
                                );
                                debug_assert!(false);
                            }
                            Self::deposit_event(Event::OutcomeOwnersRewarded {
                                market_id,
                                owners: winner_info.outcome_info.owners.to_vec(),
                            });
                            Self::deposit_event(Event::OutcomesFullyCleaned { market_id });
                            Ok((owners_len, removed_keys_amount))
                        }
                        KillStorageResult::SomeRemaining(removed_keys_amount) => {
                            Self::deposit_event(Event::OutcomesPartiallyCleaned { market_id });
                            Ok((0u32, removed_keys_amount))
                        }
                    }
                },
            )?;

            Ok((Some(T::WeightInfo::reward_outcome_owner(owners_len, removed_keys_amount))).into())
        }

        /// Vote on existing voting outcomes by locking native tokens.
        /// Fails if the global dispute has not started or has already finished.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n + m)`, where `n` is the number of all current votes on global disputes,
        /// and `m` is the number of owners for the specified outcome.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::vote_on_outcome(
            T::MaxOwners::get(),
            T::MaxGlobalDisputeVotes::get()
        ))]
        pub fn vote_on_outcome(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let voter = ensure_signed(origin)?;
            ensure!(amount <= T::Currency::free_balance(&voter), Error::<T>::InsufficientAmount);
            ensure!(amount >= T::MinOutcomeVoteAmount::get(), Error::<T>::AmountTooLow);

            let winner_info =
                <Winners<T>>::get(market_id).ok_or(Error::<T>::NoGlobalDisputeStarted)?;
            ensure!(!winner_info.is_finished, Error::<T>::GlobalDisputeAlreadyFinished);

            let mut outcome_info =
                <Outcomes<T>>::get(market_id, &outcome).ok_or(Error::<T>::OutcomeDoesNotExist)?;
            let outcome_owners_len = outcome_info.owners.len() as u32;

            // The `outcome_sum` never decreases (only increases) to allow
            // caching the outcome with the highest `outcome_sum`.
            // If the `outcome_sum` decreases, it would lead to more storage,
            // because the winning outcome could have a smaller `outcome_sum`
            // than the second highest `outcome_sum`.
            let add_to_outcome_sum = |a| {
                outcome_info.outcome_sum = outcome_info.outcome_sum.saturating_add(a);
                Self::update_winner(&market_id, &outcome, outcome_info.outcome_sum);
                <Outcomes<T>>::insert(market_id, &outcome, outcome_info);
            };

            let LockInfo(ref mut lock_info) = <Locks<T>>::get(&voter);

            let vote_lock_counter = lock_info.len() as u32;

            match lock_info.binary_search_by_key(&market_id, |i| i.0) {
                Ok(i) => {
                    let prev_highest_amount: BalanceOf<T> = lock_info[i].1;
                    if amount > prev_highest_amount {
                        let diff = amount.saturating_sub(prev_highest_amount);
                        add_to_outcome_sum(diff);
                        lock_info[i].1 = amount;
                    } else {
                        // concious decision here to not throw an error when the new amount is not higher
                        // because the user would have to pay worst case fees then
                        return Ok(Some(T::WeightInfo::vote_on_outcome(outcome_owners_len, vote_lock_counter)).into());
                    }
                }
                Err(i) => {
                    ensure!(
                        lock_info.len() as u32 <= T::MaxGlobalDisputeVotes::get(),
                        Error::<T>::MaxVotesReached
                    );
                    add_to_outcome_sum(amount);
                    lock_info.insert(i, (market_id, amount));
                }
            }

            T::Currency::extend_lock(
                T::GlobalDisputeLockId::get(),
                &voter,
                amount,
                WithdrawReasons::TRANSFER,
            );

            <Locks<T>>::insert(&voter, LockInfo(lock_info.to_vec()));

            Self::deposit_event(Event::VotedOnOutcome {
                market_id,
                voter,
                outcome,
                vote_amount: amount,
            });
            Ok(Some(T::WeightInfo::vote_on_outcome(outcome_owners_len, vote_lock_counter)).into())
        }

        /// Return all locked native tokens in a global dispute.
        /// Fails if the global dispute is not concluded yet.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n + m)`, where `n` is the number of all current votes on global disputes,
        /// and `m` is the number of owners for the winning outcome.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::unlock_vote_balance(
            T::MaxGlobalDisputeVotes::get(),
            T::MaxOwners::get()
        ))]
        pub fn unlock_vote_balance(
            origin: OriginFor<T>,
            voter: AccountIdLookupOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;
            let voter = T::Lookup::lookup(voter)?;

            let mut lock_needed: BalanceOf<T> = Zero::zero();
            let vote_lock_counter =
                <Locks<T>>::mutate(&voter, |LockInfo(ref mut lock_info)| -> u32 {
                    let vote_lock_counter = lock_info.len() as u32;
                    lock_info.retain(|&(market_id, locked_balance)| {
                        // weight component MaxOwners comes from querying the winner information
                        match <Winners<T>>::get(market_id) {
                            Some(winner_info) if winner_info.is_finished => false,
                            _ => {
                                lock_needed = lock_needed.max(locked_balance);
                                true
                            }
                        }
                    });
                    vote_lock_counter
                });

            if lock_needed.is_zero() {
                T::Currency::remove_lock(T::GlobalDisputeLockId::get(), &voter);
            } else {
                T::Currency::set_lock(
                    T::GlobalDisputeLockId::get(),
                    &voter,
                    lock_needed,
                    WithdrawReasons::TRANSFER,
                );
            }

            // use the worst case for owners length, because otherwise we would have to count each in Locks
            Ok(Some(T::WeightInfo::unlock_vote_balance(vote_lock_counter, T::MaxOwners::get()))
                .into())
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The currency implementation used to lock tokens for voting.
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The vote lock identifier.
        #[pallet::constant]
        type GlobalDisputeLockId: Get<LockIdentifier>;

        /// The pallet identifier.
        #[pallet::constant]
        type GlobalDisputesPalletId: Get<PalletId>;

        /// To reference the market id type.
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The maximum numbers of distinct markets on which one account can simultaneously vote on outcomes.
        #[pallet::constant]
        type MaxGlobalDisputeVotes: Get<u32>;

        /// The maximum number of owners for a voting outcome for private API calls of `push_voting_outcome`.
        #[pallet::constant]
        type MaxOwners: Get<u32>;

        /// The minimum required amount to vote on an outcome.
        #[pallet::constant]
        type MinOutcomeVoteAmount: Get<BalanceOf<Self>>;

        /// The maximum number of keys to remove from a storage map.
        #[pallet::constant]
        type RemoveKeysLimit: Get<u32>;

        /// The fee required to add a voting outcome.
        #[pallet::constant]
        type VotingOutcomeFee: Get<BalanceOf<Self>>;

        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Sender tried to vote with an amount below a defined minium.
        AmountTooLow,
        /// The global dispute period is already over and the winner is determined.
        GlobalDisputeAlreadyFinished,
        /// Sender does not have enough funds for the vote on an outcome.
        InsufficientAmount,
        /// No global dispute present at the moment.
        NoGlobalDisputeStarted,
        /// The voting outcome has been already added.
        OutcomeAlreadyExists,
        /// The outcome specified is not present in the voting outcomes.
        OutcomeDoesNotExist,
        /// The global dispute period is not over yet. The winner is not yet determined.
        UnfinishedGlobalDispute,
        /// The maximum number of votes for this account is reached.
        MaxVotesReached,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A new voting outcome has been added.
        AddedVotingOutcome {
            market_id: MarketIdOf<T>,
            owner: AccountIdOf<T>,
            outcome: OutcomeReport,
        },
        /// The winner of the global dispute system is determined.
        GlobalDisputeWinnerDetermined { market_id: MarketIdOf<T> },
        /// The outcome owner has been rewarded.
        OutcomeOwnersRewarded { market_id: MarketIdOf<T>, owners: Vec<AccountIdOf<T>> },
        /// The outcomes storage item is partially cleaned.
        OutcomesPartiallyCleaned { market_id: MarketIdOf<T> },
        /// The outcomes storage item is fully cleaned.
        OutcomesFullyCleaned { market_id: MarketIdOf<T> },
        /// A vote happened on an outcome.
        VotedOnOutcome {
            voter: AccountIdOf<T>,
            market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
            vote_amount: BalanceOf<T>,
        },
    }

    impl<T: Config> Pallet<T> {
        pub fn reward_account(market_id: &MarketIdOf<T>) -> T::AccountId {
            T::GlobalDisputesPalletId::get().into_sub_account(market_id)
        }

        fn update_winner(market_id: &MarketIdOf<T>, outcome: &OutcomeReport, amount: BalanceOf<T>) {
            <Winners<T>>::mutate(market_id, |highest: &mut Option<WinnerInfoOf<T>>| {
                *highest = Some(highest.clone().map_or(
                    WinnerInfo::new(outcome.clone(), amount),
                    |prev_winner_info| {
                        if amount >= prev_winner_info.outcome_info.outcome_sum {
                            WinnerInfo::new(outcome.clone(), amount)
                        } else {
                            prev_winner_info
                        }
                    },
                ));
            });
        }
    }

    impl<T> GlobalDisputesPalletApi<MarketIdOf<T>, AccountIdOf<T>, BalanceOf<T>> for Pallet<T>
    where
        T: Config,
    {
        fn push_voting_outcome(
            market_id: &MarketIdOf<T>,
            outcome: OutcomeReport,
            owner: &T::AccountId,
            vote_balance: BalanceOf<T>,
        ) -> DispatchResult {
            match <Winners<T>>::get(market_id) {
                Some(winner_info) if winner_info.is_finished => {
                    return Err(Error::<T>::GlobalDisputeAlreadyFinished.into());
                }
                _ => (),
            }
            match <Outcomes<T>>::get(market_id, &outcome) {
                Some(mut outcome_info) => {
                    let outcome_sum = outcome_info.outcome_sum.saturating_add(vote_balance);
                    Self::update_winner(market_id, &outcome, outcome_sum);
                    outcome_info.outcome_sum = outcome_sum;
                    // there can not be more than MaxDisputes owners
                    if outcome_info.owners.try_push(owner.clone()).is_ok() {
                        <Outcomes<T>>::insert(market_id, outcome, outcome_info);
                    } else {
                        log::warn!(
                            "Warning: The voting outcome was not added.  This happens because \
                             there are too many voting outcome owners (length is {:?}).",
                            &outcome_info.owners.len()
                        );
                    }
                }
                None => {
                    // adding one item to BoundedVec can not fail
                    if let Ok(owners) = BoundedVec::try_from(vec![owner.clone()]) {
                        Self::update_winner(market_id, &outcome, vote_balance);
                        let outcome_info = OutcomeInfo { outcome_sum: vote_balance, owners };
                        <Outcomes<T>>::insert(market_id, outcome, outcome_info);
                    }
                }
            }
            Ok(())
        }

        fn determine_voting_winner(market_id: &MarketIdOf<T>) -> Option<OutcomeReport> {
            match <Winners<T>>::get(market_id) {
                Some(mut winner_info) => {
                    winner_info.is_finished = true;
                    let winner_outcome = winner_info.outcome.clone();
                    <Winners<T>>::insert(market_id, winner_info);
                    Self::deposit_event(Event::GlobalDisputeWinnerDetermined {
                        market_id: *market_id,
                    });
                    Some(winner_outcome)
                }
                _ => None,
            }
        }

        fn is_started(market_id: &MarketIdOf<T>) -> bool {
            <Winners<T>>::get(market_id).is_some()
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// All highest lock information (vote id, outcome index and locked balance) for a particular voter.
    ///
    /// TWOX-NOTE: SAFE as `AccountId`s are crypto hashes anyway.
    #[pallet::storage]
    pub type Locks<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, LockInfoOf<T>, ValueQuery>;

    /// Maps the market id to the outcome and providing information about the outcome.
    #[pallet::storage]
    pub type Outcomes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        Blake2_128Concat,
        OutcomeReport,
        OutcomeInfoOf<T>,
        OptionQuery,
    >;

    /// Maps the market id to all information about the winner outcome and if the global dispute is finished.
    #[pallet::storage]
    pub type Winners<T: Config> =
        StorageMap<_, Blake2_128Concat, MarketIdOf<T>, WinnerInfoOf<T>, OptionQuery>;
}
