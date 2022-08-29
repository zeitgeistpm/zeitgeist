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
            Decode, DispatchResult, DispatchResultWithPostInfo, Encode, MaxEncodedLen, OptionQuery,
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
    use sp_runtime::traits::{AccountIdConversion, CheckedDiv, Saturating, Zero};
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
    pub struct LockInfo<MarketId, Balance>(Vec<(MarketId, Balance)>);

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

    #[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
    pub struct OutcomeInfo<Balance, OwnerInfo> {
        pub outcome_sum: Balance,
        pub owners: OwnerInfo,
    }

    #[derive(TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
    pub struct WinnerInfo<Balance, OwnerInfo> {
        pub outcome: OutcomeReport,
        pub vote_sum: Balance,
        pub is_finished: bool,
        pub owners: OwnerInfo,
    }

    impl<Balance: Saturating, OwnerInfo: Default> WinnerInfo<Balance, OwnerInfo> {
        pub fn new(outcome: OutcomeReport, vote_sum: Balance) -> Self {
            WinnerInfo { outcome, vote_sum, is_finished: false, owners: Default::default() }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[frame_support::transactional]
        #[pallet::weight(5000)]
        pub fn add_vote_outcome(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let winner_info =
                <Winners<T>>::get(market_id).ok_or(Error::<T>::NoGlobalDisputeStarted)?;
            ensure!(!winner_info.is_finished, Error::<T>::GlobalDisputeAlreadyFinished);

            ensure!(
                <Outcomes<T>>::get(market_id, outcome.clone()).is_none(),
                Error::<T>::OutcomeAlreadyExists
            );

            let voting_outcome_fee = T::VotingOutcomeFee::get();

            let reward_account = Self::reward_account(&market_id);

            T::Currency::transfer(
                &sender,
                &reward_account,
                voting_outcome_fee,
                ExistenceRequirement::AllowDeath,
            )?;

            Self::push_voting_outcome(&market_id, outcome.clone(), &sender, voting_outcome_fee);

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

            if let Some(mut winner_info) = <Winners<T>>::get(market_id) {
                ensure!(winner_info.is_finished, Error::<T>::UnfinishedGlobalDispute);
                if let Some(outcome_info) =
                    <Outcomes<T>>::get(market_id, winner_info.clone().outcome)
                {
                    winner_info.owners = outcome_info.owners;
                    <Winners<T>>::insert(market_id, winner_info.clone());
                }
                match <Outcomes<T>>::remove_prefix(market_id, Some(T::RemoveKeysLimit::get())) {
                    KillStorageResult::AllRemoved(_) => {
                        Self::deposit_event(Event::OutcomesFullyCleaned(market_id));
                        let reward_account = Self::reward_account(&market_id);
                        let reward_account_free_balance =
                            T::Currency::free_balance(&reward_account);
                        if !reward_account_free_balance.is_zero() {
                            let mut remainder = reward_account_free_balance;
                            let owners_len: usize = winner_info.owners.len();
                            let owners_len_in_balance: BalanceOf<T> =
                                <BalanceOf<T>>::from(owners_len as u32);
                            if let Some(reward_per_each) =
                                reward_account_free_balance.checked_div(&owners_len_in_balance)
                            {
                                for winner in winner_info.owners.iter() {
                                    let reward = remainder.min(reward_per_each); // *Should* always be equal to `reward_per_each`
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
                            }
                            Self::deposit_event(Event::OutcomeOwnerRewarded(market_id));
                        }
                    }
                    KillStorageResult::SomeRemaining(_) => {
                        Self::deposit_event(Event::OutcomesPartiallyCleaned(market_id));
                    }
                }
            }

            Ok(().into())
        }

        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::vote_on_outcome(T::MaxOwners::get(), T::MaxOwners::get()))]
        pub fn vote_on_outcome(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(amount <= T::Currency::free_balance(&sender), Error::<T>::InsufficientAmount);
            ensure!(amount >= T::MinOutcomeVoteAmount::get(), Error::<T>::AmountTooLow);

            let winner_info =
                <Winners<T>>::get(market_id).ok_or(Error::<T>::NoGlobalDisputeStarted)?;
            ensure!(!winner_info.is_finished, Error::<T>::GlobalDisputeAlreadyFinished);

            let mut outcome_info =
                <Outcomes<T>>::get(market_id, &outcome).ok_or(Error::<T>::OutcomeDoesNotExist)?;
            let winner_owners_len = winner_info.owners.len() as u32;
            let outcome_owners_len = outcome_info.owners.len() as u32;

            // The `outcome_sum` never decreases (only increases) to allow
            // caching the outcome with the highest `outcome_sum`.
            // If the `outcome_sum` decreases, it would lead to more storage,
            // because the winning outcome could have a smaller `outcome_sum`
            // than the second highest `outcome_sum`.
            let add_to_outcome_sum = |a| {
                outcome_info.outcome_sum = outcome_info.outcome_sum.saturating_add(a);
                <Winners<T>>::mutate(market_id, |highest| {
                    *highest = Some(highest.clone().map_or(
                        WinnerInfo::new(outcome.clone(), outcome_info.outcome_sum),
                        |prev_winner_info| {
                            if outcome_info.outcome_sum >= prev_winner_info.vote_sum {
                                WinnerInfo::new(outcome.clone(), outcome_info.outcome_sum)
                            } else {
                                prev_winner_info
                            }
                        },
                    ));
                });
                <Outcomes<T>>::insert(market_id, &outcome, outcome_info);
            };

            <Locks<T>>::try_mutate(&sender, |LockInfo(ref mut lock_info)| -> DispatchResult {
                match lock_info.binary_search_by_key(&market_id, |i| i.0) {
                    Ok(i) => {
                        let prev_highest_amount: BalanceOf<T> = lock_info[i].1;
                        if amount > prev_highest_amount {
                            let diff = amount.saturating_sub(prev_highest_amount);
                            add_to_outcome_sum(diff);
                            lock_info[i].1 = amount;
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
                Ok(())
            })?;

            T::Currency::extend_lock(
                T::VoteLockIdentifier::get(),
                &sender,
                amount,
                WithdrawReasons::TRANSFER,
            );

            Self::deposit_event(Event::VotedOnOutcome(market_id, outcome, amount));
            Ok(Some(T::WeightInfo::vote_on_outcome(outcome_owners_len, winner_owners_len)).into())
        }

        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::unlock_vote_balance())]
        pub fn unlock_vote_balance(
            origin: OriginFor<T>,
            voter: AccountIdLookupOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;
            let voter = T::Lookup::lookup(voter)?;

            let mut lock_needed: BalanceOf<T> = Zero::zero();
            <Locks<T>>::mutate(&voter, |LockInfo(ref mut lock_info)| {
                lock_info.retain(|(market_id, locked_balance)| {
                    match <Winners<T>>::get(market_id) {
                        Some(winner_info) if winner_info.is_finished => false,
                        _ => {
                            lock_needed = lock_needed.max(*locked_balance);
                            true
                        }
                    }
                });
            });

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
        /// The currency to allow locking native token for voting.
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The pallet identifier.
        #[pallet::constant]
        type GlobalDisputesPalletId: Get<PalletId>;

        /// To reference the market id type.
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The maximum number of market ids (participate in multiple different global disputes at the same time) for one account to vote on outcomes.
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

        /// The vote lock identifier.
        #[pallet::constant]
        type VoteLockIdentifier: Get<LockIdentifier>;

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
        /// A new voting outcome has been added. \[market_id, outcome_report\]
        AddedVotingOutcome(MarketIdOf<T>, OutcomeReport),
        /// The winner of the global dispute system is determined. \[market_id\]
        GlobalDisputeWinnerDetermined(MarketIdOf<T>),
        /// The outcome owner has been rewarded. \[market_id\]
        OutcomeOwnerRewarded(MarketIdOf<T>),
        /// The outcomes storage item is partially cleaned. \[market_id\]
        OutcomesPartiallyCleaned(MarketIdOf<T>),
        /// The outcomes storage item is fully cleaned. \[market_id\]
        OutcomesFullyCleaned(MarketIdOf<T>),
        /// A vote happened on an outcome. \[market_id, outcome, vote_amount\]
        VotedOnOutcome(MarketIdOf<T>, OutcomeReport, BalanceOf<T>),
    }

    impl<T: Config> Pallet<T> {
        pub fn reward_account(market_id: &MarketIdOf<T>) -> T::AccountId {
            T::GlobalDisputesPalletId::get().into_sub_account(market_id)
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
        ) {
            match <Winners<T>>::get(market_id) {
                Some(winner_info) if winner_info.is_finished => return,
                _ => (),
            }
            let update_winner = |b| {
                <Winners<T>>::mutate(market_id, |highest| {
                    *highest = Some(highest.clone().map_or(
                        WinnerInfo::new(outcome.clone(), b),
                        |prev_winner_info| {
                            if b >= prev_winner_info.vote_sum {
                                WinnerInfo::new(outcome.clone(), b)
                            } else {
                                prev_winner_info
                            }
                        },
                    ));
                });
            };
            match <Outcomes<T>>::get(market_id, outcome.clone()) {
                Some(mut outcome_info) => {
                    let outcome_sum = outcome_info.outcome_sum.saturating_add(vote_balance);
                    update_winner(outcome_sum);
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
                        update_winner(vote_balance);
                        let outcome_info = OutcomeInfo { outcome_sum: vote_balance, owners };
                        <Outcomes<T>>::insert(market_id, outcome, outcome_info);
                    }
                }
            }
        }

        fn get_voting_winner(market_id: &MarketIdOf<T>) -> Option<OutcomeReport> {
            let winner_info_opt = <Winners<T>>::get(market_id);

            if let Some(mut winner_info) = winner_info_opt.clone() {
                winner_info.is_finished = true;
                <Winners<T>>::insert(market_id, winner_info);
            }

            Self::deposit_event(Event::GlobalDisputeWinnerDetermined(*market_id));

            winner_info_opt.map(|winner_info| winner_info.outcome)
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
