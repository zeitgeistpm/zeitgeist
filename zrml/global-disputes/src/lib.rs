#![doc = include_str!("../README.md")]
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
    use alloc::{vec, vec::Vec};
    use core::marker::PhantomData;
    use frame_support::{
        ensure,
        pallet_prelude::{
            ConstU32, Decode, DispatchResultWithPostInfo, Encode, MaxEncodedLen, OptionQuery,
            StorageDoubleMap, StorageMap, TypeInfo,
        },
        storage::child::KillStorageResult,
        traits::{
            Currency, ExistenceRequirement, Get, IsType, LockIdentifier, LockableCurrency,
            WithdrawReasons,
        },
        Blake2_128Concat, BoundedVec, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::traits::{AccountIdConversion, CheckedDiv, Saturating, Zero};
    use zeitgeist_primitives::types::OutcomeReport;
    use zrml_market_commons::MarketCommonsPalletApi;

    pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

    pub type OutcomeInfoOf<T> = OutcomeInfo<BalanceOf<T>, AccountIdOf<T>>;
    pub type WinnerInfoOf<T> = WinnerInfo<BalanceOf<T>, AccountIdOf<T>>;

    #[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialOrd, PartialEq)]
    pub struct OutcomeInfo<Balance, AccountId> {
        pub outcome_sum: Balance,
        pub owners: BoundedVec<AccountId, ConstU32<10>>,
    }

    #[derive(TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialOrd, PartialEq)]
    pub struct WinnerInfo<Balance, AccountId> {
        pub outcome: OutcomeReport,
        pub vote_sum: Balance,
        pub is_finished: bool,
        pub owners: BoundedVec<AccountId, ConstU32<10>>,
    }

    impl<Balance: Saturating, AccountId> WinnerInfo<Balance, AccountId> {
        fn new(outcome: OutcomeReport, vote_sum: Balance) -> Self {
            WinnerInfo { outcome, vote_sum, is_finished: false, owners: Default::default() }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add an outcome to the voting outcomes to allow anybody to lock native tokens on it.
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

        /// Reward the owners of the winning outcome with the `VotingOutcomeFee`'s of the looser outcome owners.
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
                            if let Some(reward_per_each) = reward_account_free_balance
                                .checked_div(&<BalanceOf<T>>::from(winner_info.owners.len() as u32))
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

        /// Votes on an outcome with an `amount`.
        ///
        /// NOTE:
        /// As long as the voting amount increases for multiple calls of this function,
        /// only the increased difference amount is included in the `outcome_sum`.
        /// The reason for this is to prevent the possible attack vector
        /// of increasing the `outcome_sum` with the exact same locked balance.
        /// The `outcome_sum` never decreases (only increases) to allow
        /// caching the outcome with the highest `outcome_sum`.
        /// If the `outcome_sum` decreases, it would lead to more storage,
        /// because the winning outcome could have a smaller `outcome_sum`
        /// than the second highest `outcome_sum`.
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

            let winner_info =
                <Winners<T>>::get(market_id).ok_or(Error::<T>::NoGlobalDisputeStarted)?;
            ensure!(!winner_info.is_finished, Error::<T>::GlobalDisputeAlreadyFinished);

            let mut outcome_info =
                <Outcomes<T>>::get(market_id, &outcome).ok_or(Error::<T>::OutcomeDoesNotExist)?;

            <LockInfoOf<T>>::mutate(&sender, market_id, |lock_info| {
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
                if let Some(prev_highest_amount) = lock_info {
                    if amount > *prev_highest_amount {
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

        /// Unlock the voters locked token amounts from the finished global disputes.
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
                match <Winners<T>>::get(market_id) {
                    Some(winner_info) if winner_info.is_finished => {
                        resolved_ids.push(market_id);
                        continue;
                    }
                    _ => (),
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
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// To reference the market id type.
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The currency to allow locking native token for voting.
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

        /// The pallet identifier.
        #[pallet::constant]
        type GlobalDisputesPalletId: Get<PalletId>;

        /// The vote lock identifier.
        #[pallet::constant]
        type VoteLockIdentifier: Get<LockIdentifier>;

        /// The minimum required amount to vote on an outcome.
        #[pallet::constant]
        type MinOutcomeVoteAmount: Get<BalanceOf<Self>>;

        /// The fee required to add a voting outcome.
        #[pallet::constant]
        type VotingOutcomeFee: Get<BalanceOf<Self>>;

        /// The maximum number of keys to remove from a storage map.
        #[pallet::constant]
        type RemoveKeysLimit: Get<u32>;

        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The outcome specified is not present in the voting outcomes.
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
        /// The global dispute period is already over and the winner is determined.
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
        /// A new voting outcome has been added. \[market_id, outcome_report\]
        AddedVotingOutcome(MarketIdOf<T>, OutcomeReport),
        /// The outcome owner has been rewarded. \[market_id\]
        OutcomeOwnerRewarded(MarketIdOf<T>),
        /// The outcomes storage item is partially cleaned. \[market_id\]
        OutcomesPartiallyCleaned(MarketIdOf<T>),
        /// The outcomes storage item is fully cleaned. \[market_id\]
        OutcomesFullyCleaned(MarketIdOf<T>),
    }

    impl<T: Config> Pallet<T> {
        fn reward_account(market_id: &MarketIdOf<T>) -> T::AccountId {
            T::GlobalDisputesPalletId::get().into_sub_account(market_id)
        }
    }

    impl<T> GlobalDisputesPalletApi<MarketIdOf<T>, AccountIdOf<T>, BalanceOf<T>> for Pallet<T>
    where
        T: Config,
    {
        /// Add an outcome (with initial vote balance) to the voting outcomes. 
        /// The `outcome_sum` of exact same outcomes is added.
        /// There can be multiple owners of one same outcome.
        /// 
        /// NOTE:
        /// This is meant to be called to start a global dispute.
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
                    let _ = outcome_info.owners.try_push(owner.clone());
                    <Outcomes<T>>::insert(market_id, outcome, outcome_info);
                }
                None => {
                    if let Ok(owners) = BoundedVec::try_from(vec![owner.clone()]) {
                        update_winner(vote_balance);
                        let outcome_info = OutcomeInfo { outcome_sum: vote_balance, owners };
                        <Outcomes<T>>::insert(market_id, outcome, outcome_info);
                    }
                }
            }
        }

        /// Determine the outcome with the highest `outcome_sum` as the winner.
        ///
        /// NOTE:
        /// This is meant to be called to finish a global dispute.
        fn get_voting_winner(market_id: &MarketIdOf<T>) -> Option<OutcomeReport> {
            let winner_info_opt = <Winners<T>>::get(market_id);

            if let Some(mut winner_info) = winner_info_opt.clone() {
                winner_info.is_finished = true;
                <Winners<T>>::insert(market_id, winner_info);
            }

            winner_info_opt.map(|winner_info| winner_info.outcome)
        }

        /// Check if the global dispute started already.
        fn is_started(market_id: &MarketIdOf<T>) -> bool {
            <Winners<T>>::get(market_id).is_some()
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// Maps the market id to all information about the winner outcome and if the global dispute is finished.
    #[pallet::storage]
    pub type Winners<T: Config> =
        StorageMap<_, Blake2_128Concat, MarketIdOf<T>, WinnerInfoOf<T>, OptionQuery>;

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
}
