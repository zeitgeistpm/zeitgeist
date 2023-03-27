// Copyright 2022-2023 Forecasting Technologies LTD.
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
pub mod migrations;
mod mock;
mod tests;
pub mod types;
mod utils;
pub mod weights;

pub use global_disputes_pallet_api::GlobalDisputesPalletApi;
pub use pallet::*;

pub type PossessionOf<T> = crate::types::Possession<AccountIdOf<T>, BalanceOf<T>, OwnerInfoOf<T>>;
pub type InitialItemOf<T> = crate::types::InitialItem<AccountIdOf<T>, BalanceOf<T>>;

#[frame_support::pallet]
mod pallet {
    use crate::{types::*, weights::WeightInfoZeitgeist, GlobalDisputesPalletApi, InitialItemOf};
    use core::marker::PhantomData;
    use frame_support::{
        ensure, log,
        pallet_prelude::{
            DispatchResultWithPostInfo, OptionQuery, StorageDoubleMap, StorageMap, ValueQuery,
        },
        sp_runtime::traits::StaticLookup,
        traits::{
            Currency, ExistenceRequirement, Get, IsType, LockIdentifier, LockableCurrency,
            StorageVersion, WithdrawReasons,
        },
        Blake2_128Concat, BoundedVec, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedDiv, Saturating, Zero},
        DispatchError, DispatchResult,
    };
    use sp_std::{vec, vec::Vec};
    use zeitgeist_primitives::{traits::DisputeResolutionApi, types::OutcomeReport};
    use zrml_market_commons::MarketCommonsPalletApi;

    pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    pub(crate) type OwnerInfoOf<T> = BoundedVec<AccountIdOf<T>, <T as Config>::MaxOwners>;
    pub type OutcomeInfoOf<T> = OutcomeInfo<AccountIdOf<T>, BalanceOf<T>, OwnerInfoOf<T>>;
    pub type GlobalDisputeInfoOf<T> = GlobalDisputeInfo<
        AccountIdOf<T>,
        BalanceOf<T>,
        OwnerInfoOf<T>,
        <T as frame_system::Config>::BlockNumber,
    >;

    type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
    pub type LockInfoOf<T> =
        BoundedVec<(MarketIdOf<T>, BalanceOf<T>), <T as Config>::MaxGlobalDisputeVotes>;

    // TODO(#968): to remove after the storage migration
    pub type WinnerInfoOf<T> = OldWinnerInfo<BalanceOf<T>, OwnerInfoOf<T>>;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The time period in which the addition of new outcomes are allowed.
        #[pallet::constant]
        type AddOutcomePeriod: Get<Self::BlockNumber>;

        /// The currency implementation used to lock tokens for voting.
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type DisputeResolution: DisputeResolutionApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
            MarketId = MarketIdOf<Self>,
            Moment = MomentOf<Self>,
        >;

        /// The vote lock identifier.
        #[pallet::constant]
        type GlobalDisputeLockId: Get<LockIdentifier>;

        /// The pallet identifier.
        #[pallet::constant]
        type GlobalDisputesPalletId: Get<PalletId>;

        /// To reference the market id type.
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            Currency = Self::Currency,
            BlockNumber = Self::BlockNumber,
        >;

        /// The maximum numbers of distinct markets
        /// on which one account can simultaneously vote on outcomes.
        /// When the user unlocks, the user has again `MaxGlobalDisputeVotes` number of votes.
        /// This constant is useful to limit the number of for-loop iterations (weight constraints).
        #[pallet::constant]
        type MaxGlobalDisputeVotes: Get<u32>;

        /// The maximum number of owners
        /// for a voting outcome for private API calls of `push_vote_outcome`.
        #[pallet::constant]
        type MaxOwners: Get<u32>;

        /// The minimum required amount to vote on an outcome.
        #[pallet::constant]
        type MinOutcomeVoteAmount: Get<BalanceOf<Self>>;

        /// The maximum number of keys to remove from a storage map.
        #[pallet::constant]
        type RemoveKeysLimit: Get<u32>;

        /// The time period in which votes are allowed.
        #[pallet::constant]
        type GdVotingPeriod: Get<Self::BlockNumber>;

        /// The fee required to add a voting outcome.
        #[pallet::constant]
        type VotingOutcomeFee: Get<BalanceOf<Self>>;

        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    /// All highest lock information (vote id, outcome index and locked balance)
    /// for a particular voter.
    ///
    /// TWOX-NOTE: SAFE as `AccountId`s are crypto hashes anyway.
    #[pallet::storage]
    pub type Locks<T: Config> =
        StorageMap<_, Twox64Concat, T::AccountId, LockInfoOf<T>, ValueQuery>;

    /// Maps the market id to the outcome and providing information about the outcome.
    #[pallet::storage]
    pub type Outcomes<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        MarketIdOf<T>,
        Blake2_128Concat,
        OutcomeReport,
        OutcomeInfoOf<T>,
        OptionQuery,
    >;

    /// Maps the market id to all information
    /// about the global dispute.
    #[pallet::storage]
    pub type GlobalDisputesInfo<T: Config> =
        StorageMap<_, Twox64Concat, MarketIdOf<T>, GlobalDisputeInfoOf<T>, OptionQuery>;

    // TODO(#986): to remove after the storage migration
    #[pallet::storage]
    pub type Winners<T: Config> =
        StorageMap<_, Twox64Concat, MarketIdOf<T>, WinnerInfoOf<T>, OptionQuery>;

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
        /// The outcome owners have been rewarded.
        OutcomeOwnersRewarded { market_id: MarketIdOf<T>, owners: Vec<AccountIdOf<T>> },
        /// The outcome owner has been rewarded.
        OutcomeOwnerRewarded { market_id: MarketIdOf<T>, owner: AccountIdOf<T> },
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

    #[pallet::error]
    pub enum Error<T> {
        /// Sender tried to vote with an amount below a defined minimum.
        AmountTooLow,
        /// To start a global dispute, at least two outcomes are required.
        AtLeastTwoOutcomesRequired,
        /// The global dispute status is invalid for this operation.
        InvalidGlobalDisputeStatus,
        /// Sender does not have enough funds for the vote on an outcome.
        InsufficientAmount,
        /// The maximum amount of owners is reached.
        MaxOwnersReached,
        /// The maximum number of votes for this account is reached.
        MaxVotesReached,
        /// The amount in the reward pot is zero.
        NoFundsToReward,
        /// No global dispute present at the moment.
        GlobalDisputeNotFound,
        /// The voting outcome has been already added.
        OutcomeAlreadyExists,
        /// The outcome specified is not present in the voting outcomes.
        OutcomeDoesNotExist,
        /// Submitted outcome does not match market type.
        OutcomeMismatch,
        /// The outcomes are not fully cleaned yet.
        OutcomesNotFullyCleaned,
        /// Only a shared possession is allowed.
        SharedPossessionRequired,
        /// The global dispute period is not over yet. The winner is not yet determined.
        UnfinishedGlobalDispute,
        /// The period in which outcomes can be added is over.
        AddOutcomePeriodIsOver,
        /// It is not inside the period in which votes are allowed.
        NotInGdVotingPeriod,
        /// The operation requires a global dispute in a destroyed state.
        GlobalDisputeNotDestroyed,
        /// The global dispute was already started.
        GlobalDisputeAlreadyExists,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add voting outcome to a global dispute in exchange for a constant fee.
        /// Errors if the voting outcome already exists or
        /// if the global dispute has not started or has already finished.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The id of the market.
        /// - `outcome`: The outcome report to add.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of owner(s) of the winner outcome
        /// in the case that this gets called for an already finished global dispute.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::add_vote_outcome(T::MaxOwners::get()))]
        pub fn add_vote_outcome(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
        ) -> DispatchResultWithPostInfo {
            let owner = ensure_signed(origin)?;

            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.matches_outcome_report(&outcome), Error::<T>::OutcomeMismatch);

            let gd_info =
                <GlobalDisputesInfo<T>>::get(market_id).ok_or(Error::<T>::GlobalDisputeNotFound)?;
            let now = <frame_system::Pallet<T>>::block_number();
            if let GdStatus::Active { add_outcome_end, vote_end: _ } = gd_info.status {
                ensure!(now <= add_outcome_end, Error::<T>::AddOutcomePeriodIsOver);
            } else {
                return Err(Error::<T>::InvalidGlobalDisputeStatus.into());
            }

            ensure!(
                !<Outcomes<T>>::contains_key(market_id, &outcome),
                Error::<T>::OutcomeAlreadyExists
            );

            let voting_outcome_fee = T::VotingOutcomeFee::get();

            let reward_account = Self::reward_account(&market_id);

            T::Currency::transfer(
                &owner,
                &reward_account,
                voting_outcome_fee,
                ExistenceRequirement::KeepAlive,
            )?;

            let possession = Possession::Paid { owner: owner.clone(), fee: voting_outcome_fee };
            let outcome_info = OutcomeInfo { outcome_sum: voting_outcome_fee, possession };
            Self::update_winner(&market_id, &outcome, outcome_info.clone());
            <Outcomes<T>>::insert(market_id, outcome.clone(), outcome_info);

            Self::deposit_event(Event::AddedVotingOutcome { market_id, owner, outcome });
            // charge weight for successfully have no owners in Winners
            // this is the case, because owners are not inserted
            // as long as the global dispute is not finished
            Ok((Some(T::WeightInfo::add_vote_outcome(0u32))).into())
        }

        /// Return the voting outcome fees in case the global dispute was destroyed.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The id of the market.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`,
        /// where `n` is the number of all existing outcomes for a global dispute.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::refund_vote_fees(
            T::RemoveKeysLimit::get(),
            T::MaxOwners::get(),
        ))]
        pub fn refund_vote_fees(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            let gd_info =
                <GlobalDisputesInfo<T>>::get(market_id).ok_or(Error::<T>::GlobalDisputeNotFound)?;
            ensure!(gd_info.status == GdStatus::Destroyed, Error::<T>::GlobalDisputeNotDestroyed);

            let mut owners_len = 0u32;
            let mut removed_keys_amount = 0u32;
            for (_, outcome_info) in
                <Outcomes<T>>::drain_prefix(market_id).take(T::RemoveKeysLimit::get() as usize)
            {
                match outcome_info.possession {
                    Possession::Paid { owner, fee } => {
                        let res = T::Currency::transfer(
                            &Self::reward_account(&market_id),
                            &owner,
                            fee,
                            ExistenceRequirement::AllowDeath,
                        );
                        debug_assert!(res.is_ok());
                    }
                    Possession::Shared { owners } => {
                        owners_len = owners_len.saturating_add(owners.len() as u32);
                    }
                }
                removed_keys_amount = removed_keys_amount.saturating_add(1u32);
            }

            if <Outcomes<T>>::iter_prefix(market_id).next().is_none() {
                Self::deposit_event(Event::OutcomesFullyCleaned { market_id });
            } else {
                Self::deposit_event(Event::OutcomesPartiallyCleaned { market_id });
            }

            // weight for max owners, because we don't know
            Ok((Some(T::WeightInfo::refund_vote_fees(removed_keys_amount, owners_len))).into())
        }

        /// Purge all outcomes to allow the winning outcome owner(s) to get their reward.
        /// Fails if the global dispute is not concluded yet.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The id of the market.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`,
        /// where `n` is the number of all existing outcomes for a global dispute.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::purge_outcomes(
            T::RemoveKeysLimit::get(),
            T::MaxOwners::get(),
        ))]
        pub fn purge_outcomes(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            let mut gd_info =
                <GlobalDisputesInfo<T>>::get(market_id).ok_or(Error::<T>::GlobalDisputeNotFound)?;
            ensure!(gd_info.status == GdStatus::Finished, Error::<T>::UnfinishedGlobalDispute);

            let winning_outcome: Option<OutcomeInfoOf<T>> =
                <Outcomes<T>>::take(market_id, &gd_info.winner_outcome);
            let mut owners_len = 0u32;
            // move the winning outcome info to GlobalDisputesInfo before it gets drained
            if let Some(outcome_info) = winning_outcome {
                if let Possession::Shared { owners } = &outcome_info.possession {
                    owners_len = owners.len() as u32;
                }
                // storage write is needed in case to save the owners
                // of the winning outcome before they are drained
                gd_info.outcome_info = outcome_info;
                <GlobalDisputesInfo<T>>::insert(market_id, gd_info);
            }

            let mut removed_keys_amount = 0u32;
            for (_, outcome_info) in
                <Outcomes<T>>::drain_prefix(market_id).take(T::RemoveKeysLimit::get() as usize)
            {
                if let Possession::Shared { owners } = outcome_info.possession {
                    owners_len = owners_len.saturating_add(owners.len() as u32);
                }
                removed_keys_amount = removed_keys_amount.saturating_add(1u32);
            }

            if <Outcomes<T>>::iter_prefix(market_id).next().is_none() {
                Self::deposit_event(Event::OutcomesFullyCleaned { market_id });
            } else {
                Self::deposit_event(Event::OutcomesPartiallyCleaned { market_id });
            }

            // weight for max owners, because we don't know
            Ok((Some(T::WeightInfo::purge_outcomes(removed_keys_amount, owners_len))).into())
        }

        /// Reward the collected fees to the owner(s) of a voting outcome.
        /// Fails if outcomes is not already purged.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The id of the market.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of owners for the winning outcome.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::reward_outcome_owner_paid_possession()
            .max(T::WeightInfo::reward_outcome_owner_shared_possession(T::MaxOwners::get())))]
        pub fn reward_outcome_owner(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            ensure!(
                <Outcomes<T>>::iter_prefix(market_id).next().is_none(),
                <Error<T>>::OutcomesNotFullyCleaned
            );

            let gd_info =
                <GlobalDisputesInfo<T>>::get(market_id).ok_or(Error::<T>::GlobalDisputeNotFound)?;
            ensure!(gd_info.status == GdStatus::Finished, Error::<T>::UnfinishedGlobalDispute);

            let reward_account = Self::reward_account(&market_id);
            let reward_account_free_balance = T::Currency::free_balance(&reward_account);
            ensure!(!reward_account_free_balance.is_zero(), Error::<T>::NoFundsToReward);

            match gd_info.outcome_info.possession {
                Possession::Shared { owners } => Self::reward_shared_possession(
                    market_id,
                    reward_account,
                    reward_account_free_balance,
                    owners,
                ),
                Possession::Paid { owner, fee: _ } => Self::reward_paid_possession(
                    market_id,
                    reward_account,
                    reward_account_free_balance,
                    owner,
                ),
            }
        }

        /// Vote on existing voting outcomes by locking native tokens.
        /// Fails if the global dispute has not started or has already finished.
        ///
        /// # Arguments
        ///
        /// - `market_id`: The id of the market.
        /// - `outcome`: The existing outcome report to vote on.
        /// - `amount`: The amount to vote with.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n + m)`, where `n` is the number of all current votes on global disputes,
        /// and `m` is the number of owners for the specified outcome.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::vote_on_outcome(
            T::MaxOwners::get(),
            T::MaxGlobalDisputeVotes::get(),
        ))]
        pub fn vote_on_outcome(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let voter = ensure_signed(origin)?;
            let voter_free_balance = T::Currency::free_balance(&voter);
            ensure!(amount <= voter_free_balance, Error::<T>::InsufficientAmount);
            ensure!(amount >= T::MinOutcomeVoteAmount::get(), Error::<T>::AmountTooLow);

            let gd_info =
                <GlobalDisputesInfo<T>>::get(market_id).ok_or(Error::<T>::GlobalDisputeNotFound)?;
            let now = <frame_system::Pallet<T>>::block_number();
            if let GdStatus::Active { add_outcome_end, vote_end } = gd_info.status {
                ensure!(add_outcome_end < now && now <= vote_end, Error::<T>::NotInGdVotingPeriod);
            } else {
                return Err(Error::<T>::InvalidGlobalDisputeStatus.into());
            }

            let mut outcome_info =
                <Outcomes<T>>::get(market_id, &outcome).ok_or(Error::<T>::OutcomeDoesNotExist)?;
            let outcome_owners_len = match outcome_info.possession {
                Possession::Shared { ref owners } => owners.len() as u32,
                Possession::Paid { .. } => 1u32,
            };

            // The `outcome_sum` never decreases (only increases) to allow
            // caching the outcome with the highest `outcome_sum`.
            // If the `outcome_sum` decreases, it would lead to more storage,
            // because the winning outcome could have a smaller `outcome_sum`
            // than the second highest `outcome_sum`.
            let add_to_outcome_sum = |a| {
                outcome_info.outcome_sum = outcome_info.outcome_sum.saturating_add(a);
                Self::update_winner(&market_id, &outcome, outcome_info.clone());
                <Outcomes<T>>::insert(market_id, &outcome, outcome_info);
            };

            let mut lock_info = <Locks<T>>::get(&voter);

            let vote_lock_counter = lock_info.len() as u32;

            let lock_amount = match lock_info.binary_search_by_key(&market_id, |i| i.0) {
                Ok(i) => {
                    let prev_amount_acc: BalanceOf<T> = lock_info[i].1;
                    let new_amount_acc = amount.saturating_add(prev_amount_acc);
                    ensure!(new_amount_acc <= voter_free_balance, Error::<T>::InsufficientAmount);
                    add_to_outcome_sum(amount);
                    lock_info[i].1 = new_amount_acc;
                    new_amount_acc
                }
                Err(i) => {
                    lock_info
                        .try_insert(i, (market_id, amount))
                        .map_err(|_| Error::<T>::MaxVotesReached)?;
                    add_to_outcome_sum(amount);
                    amount
                }
            };

            T::Currency::extend_lock(
                T::GlobalDisputeLockId::get(),
                &voter,
                lock_amount,
                WithdrawReasons::TRANSFER,
            );

            <Locks<T>>::insert(&voter, lock_info);

            Self::deposit_event(Event::VotedOnOutcome {
                market_id,
                voter,
                outcome,
                vote_amount: amount,
            });
            Ok(Some(T::WeightInfo::vote_on_outcome(outcome_owners_len, vote_lock_counter)).into())
        }

        /// Return all locked native tokens from a finished or destroyed global dispute.
        /// Fails if the global dispute is not concluded yet.
        ///
        /// # Arguments
        ///
        /// - `voter`: The account id lookup to unlock funds for.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n + m)`, where `n` is the number of all current votes on global disputes,
        /// and `m` is the number of owners for the winning outcome.
        #[frame_support::transactional]
        #[pallet::weight(
            T::WeightInfo::unlock_vote_balance_set(
                T::MaxGlobalDisputeVotes::get(),
                T::MaxOwners::get(),
            )
            .max(T::WeightInfo::unlock_vote_balance_remove(
                T::MaxGlobalDisputeVotes::get(),
                T::MaxOwners::get(),
            ))
        )]
        pub fn unlock_vote_balance(
            origin: OriginFor<T>,
            voter: AccountIdLookupOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;
            let voter = T::Lookup::lookup(voter)?;

            let mut lock_needed: BalanceOf<T> = Zero::zero();
            let mut lock_info = <Locks<T>>::get(&voter);
            let vote_lock_counter = lock_info.len() as u32;
            // Inside retain we follow these rules:
            // 1. Remove all locks from resolved (/ finished / destroyed) global disputes.
            // 2. Then find the maximum lock from all unresolved global disputes.
            lock_info.retain(|&(market_id, locked_balance)| {
                // weight component MaxOwners comes from querying the winner information
                match <GlobalDisputesInfo<T>>::get(market_id) {
                    Some(gd_info) => {
                        if matches!(gd_info.status, GdStatus::Finished | GdStatus::Destroyed) {
                            false
                        } else {
                            lock_needed = lock_needed.max(locked_balance);
                            true
                        }
                    }
                    None => {
                        log::warn!(
                            "Global Disputes: Winner info is not found for market with id {:?}.",
                            market_id
                        );
                        debug_assert!(false);
                        // unlock these funds
                        false
                    }
                }
            });

            <Locks<T>>::insert(&voter, lock_info);

            // use the worst case for owners length,
            // because otherwise we would have to count each in Locks
            if lock_needed.is_zero() {
                T::Currency::remove_lock(T::GlobalDisputeLockId::get(), &voter);

                Ok(Some(T::WeightInfo::unlock_vote_balance_remove(
                    vote_lock_counter,
                    T::MaxOwners::get(),
                ))
                .into())
            } else {
                T::Currency::set_lock(
                    T::GlobalDisputeLockId::get(),
                    &voter,
                    lock_needed,
                    WithdrawReasons::TRANSFER,
                );
                Ok(Some(T::WeightInfo::unlock_vote_balance_set(
                    vote_lock_counter,
                    T::MaxOwners::get(),
                ))
                .into())
            }
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn reward_account(market_id: &MarketIdOf<T>) -> T::AccountId {
            T::GlobalDisputesPalletId::get().into_sub_account_truncating(market_id)
        }

        fn update_winner(
            market_id: &MarketIdOf<T>,
            outcome: &OutcomeReport,
            outcome_info: OutcomeInfoOf<T>,
        ) {
            let amount = outcome_info.outcome_sum;
            <GlobalDisputesInfo<T>>::mutate(
                market_id,
                |highest: &mut Option<GlobalDisputeInfoOf<T>>| {
                    *highest = Some(highest.clone().map_or(
                        // if never a highest was present set the first here
                        GlobalDisputeInfo::new(outcome.clone(), outcome_info.possession, amount),
                        |mut prev_gd_info| {
                            if amount >= prev_gd_info.outcome_info.outcome_sum {
                                prev_gd_info.update_winner(outcome.clone(), amount);
                                prev_gd_info
                            } else {
                                prev_gd_info
                            }
                        },
                    ));
                },
            );
        }

        fn reward_shared_possession(
            market_id: MarketIdOf<T>,
            reward_account: AccountIdOf<T>,
            reward: BalanceOf<T>,
            owners: OwnerInfoOf<T>,
        ) -> DispatchResultWithPostInfo {
            let mut remainder = reward;
            let owners_len = owners.len() as u32;
            let owners_len_in_balance: BalanceOf<T> = <BalanceOf<T>>::from(owners_len);
            if let Some(reward_per_each) = reward.checked_div(&owners_len_in_balance) {
                for winner in owners.iter() {
                    // *Should* always be equal to `reward_per_each`
                    let reward = remainder.min(reward_per_each);
                    remainder = remainder.saturating_sub(reward);
                    // Reward the losing funds to the winners
                    let res = T::Currency::transfer(
                        &reward_account,
                        winner,
                        reward,
                        ExistenceRequirement::AllowDeath,
                    );
                    // not really much we can do if it fails
                    debug_assert!(
                        res.is_ok(),
                        "Global Disputes: Rewarding a outcome owner failed."
                    );
                }
            } else {
                log::error!(
                    "Global Disputes: There should be always at least one owner for a voting \
                     outcome. This can also happen if reward is smaller than owners_len."
                );
                debug_assert!(false);
            }
            Self::deposit_event(Event::OutcomeOwnersRewarded {
                market_id,
                owners: owners.into_inner(),
            });
            Ok((Some(T::WeightInfo::reward_outcome_owner_shared_possession(owners_len))).into())
        }

        fn reward_paid_possession(
            market_id: MarketIdOf<T>,
            reward_account: AccountIdOf<T>,
            reward: BalanceOf<T>,
            owner: AccountIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let res = T::Currency::transfer(
                &reward_account,
                &owner,
                reward,
                ExistenceRequirement::AllowDeath,
            );
            // not really much we can do if it fails
            debug_assert!(res.is_ok(), "Global Disputes: Rewarding a outcome owner failed.");
            Self::deposit_event(Event::OutcomeOwnerRewarded { market_id, owner });
            Ok((Some(T::WeightInfo::reward_outcome_owner_paid_possession())).into())
        }
    }

    impl<T> GlobalDisputesPalletApi<MarketIdOf<T>, AccountIdOf<T>, BalanceOf<T>, T::BlockNumber>
        for Pallet<T>
    where
        T: Config,
    {
        fn get_add_outcome_period() -> T::BlockNumber {
            T::AddOutcomePeriod::get()
        }

        fn get_vote_period() -> T::BlockNumber {
            T::GdVotingPeriod::get()
        }

        fn determine_voting_winner(market_id: &MarketIdOf<T>) -> Option<OutcomeReport> {
            match <GlobalDisputesInfo<T>>::get(market_id) {
                Some(mut gd_info) => {
                    gd_info.status = GdStatus::Finished;
                    let winner_outcome = gd_info.winner_outcome.clone();
                    <GlobalDisputesInfo<T>>::insert(market_id, gd_info);
                    Self::deposit_event(Event::GlobalDisputeWinnerDetermined {
                        market_id: *market_id,
                    });
                    Some(winner_outcome)
                }
                _ => None,
            }
        }

        fn does_exist(market_id: &MarketIdOf<T>) -> bool {
            <GlobalDisputesInfo<T>>::get(market_id).is_some()
        }

        fn is_active(market_id: &MarketIdOf<T>) -> bool {
            if let Some(gd_info) = <GlobalDisputesInfo<T>>::get(market_id) {
                if let GdStatus::Active { add_outcome_end: _, vote_end: _ } = gd_info.status {
                    return true;
                }
            }
            false
        }

        fn start_global_dispute(
            market_id: &MarketIdOf<T>,
            initial_items: &[InitialItemOf<T>],
        ) -> Result<u32, DispatchError> {
            let market = T::MarketCommons::market(market_id)?;

            ensure!(
                <GlobalDisputesInfo<T>>::get(market_id).is_none(),
                Error::<T>::GlobalDisputeAlreadyExists
            );

            ensure!(initial_items.len() >= 2, Error::<T>::AtLeastTwoOutcomesRequired);

            for InitialItem { outcome, owner, amount } in initial_items {
                ensure!(market.matches_outcome_report(outcome), Error::<T>::OutcomeMismatch);

                match <Outcomes<T>>::get(market_id, outcome) {
                    Some(mut outcome_info) => {
                        let outcome_sum = outcome_info.outcome_sum.saturating_add(*amount);
                        outcome_info.outcome_sum = outcome_sum;
                        let mut owners = outcome_info
                            .possession
                            .get_shared_owners()
                            .ok_or(Error::<T>::SharedPossessionRequired)?;
                        owners.try_push(owner.clone()).map_err(|_| Error::<T>::MaxOwnersReached)?;
                        outcome_info.possession = Possession::Shared { owners };
                        Self::update_winner(market_id, outcome, outcome_info.clone());
                        <Outcomes<T>>::insert(market_id, outcome, outcome_info);
                    }
                    None => {
                        // adding one item to BoundedVec can not fail
                        if let Ok(owners) = BoundedVec::try_from(vec![owner.clone()]) {
                            let possession = Possession::Shared { owners };
                            let outcome_info = OutcomeInfo { outcome_sum: *amount, possession };
                            Self::update_winner(market_id, outcome, outcome_info.clone());
                            <Outcomes<T>>::insert(market_id, outcome, outcome_info);
                        } else {
                            log::error!("Global Disputes: Could not construct a bounded vector.");
                            debug_assert!(false);
                        }
                    }
                }
            }

            let now = <frame_system::Pallet<T>>::block_number();
            let add_outcome_end = now.saturating_add(T::AddOutcomePeriod::get());
            let vote_end = add_outcome_end.saturating_add(T::GdVotingPeriod::get());
            let ids_len = T::DisputeResolution::add_auto_resolve(market_id, vote_end)?;

            <GlobalDisputesInfo<T>>::try_mutate(market_id, |gd_info| -> DispatchResult {
                let mut raw_gd_info = gd_info.as_mut().ok_or(Error::<T>::GlobalDisputeNotFound)?;
                raw_gd_info.status = GdStatus::Active { add_outcome_end, vote_end };
                *gd_info = Some(raw_gd_info.clone());
                Ok(())
            })?;

            Ok(ids_len)
        }

        fn destroy_global_dispute(market_id: &MarketIdOf<T>) -> Result<(), DispatchError> {
            <GlobalDisputesInfo<T>>::try_mutate(market_id, |gd_info| {
                let mut raw_gd_info = gd_info.as_mut().ok_or(Error::<T>::GlobalDisputeNotFound)?;

                // in case the global dispute is already finished nothing needs to be done
                if let GdStatus::Active { add_outcome_end: _, vote_end } = raw_gd_info.status {
                    T::DisputeResolution::remove_auto_resolve(market_id, vote_end);

                    raw_gd_info.status = GdStatus::Destroyed;
                    *gd_info = Some(raw_gd_info.clone());
                }

                Ok(())
            })
        }
    }
}
