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
pub use zrml_market_commons::MarketCommonsPalletApi;

#[frame_support::pallet]
mod pallet {
    use super::MarketCommonsPalletApi;
    use crate::{weights::WeightInfoZeitgeist, GlobalDisputesPalletApi};
    use alloc::vec::Vec;
    use core::{cmp::Ordering, marker::PhantomData};
    use frame_support::{
        ensure,
        pallet_prelude::{
            DispatchResultWithPostInfo, OptionQuery, StorageDoubleMap, StorageMap, ValueQuery,
        },
        traits::{Currency, Get, Hooks, IsType, LockIdentifier, LockableCurrency, WithdrawReasons},
        Blake2_128Concat, BoundedVec, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::{
        traits::{One, Saturating, Zero},
        DispatchError, SaturatedConversion,
    };
    use zeitgeist_primitives::types::{MarketStatus, OutcomeReport};

    pub(crate) type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Votes on a dispute after there are already two disputes and the 'DisputePeriod' is not over.
        /// NOTE: In the 'DisputePeriod' voting on a dispute is allowed.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::vote_on_outcome())]
        pub fn vote_on_outcome(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome_index: u32,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                amount <= CurrencyOf::<T>::free_balance(&sender),
                Error::<T>::InsufficientAmount
            );
            ensure!(amount >= T::MinDisputeVoteAmount::get(), Error::<T>::AmountTooLow);

            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Disputed, Error::<T>::InvalidMarketStatus);

            ensure!(
                <Outcomes<T>>::get(market_id).len() >= T::MinOutcomes::get() as usize,
                Error::<T>::NotEnoughOutcomes
            );

            // dispute vote is already present because of the dispute bond of the disputor
            let vote_balance = <OutcomeVotes<T>>::get(market_id, outcome_index)
                .ok_or(Error::<T>::OutcomeDoesNotExist)?;

            <LockInfoOf<T>>::mutate(&sender, market_id, |locked_balance| {
                *locked_balance = Some(locked_balance.map_or(amount, |x| x.max(amount)));
            });

            CurrencyOf::<T>::extend_lock(
                T::VoteLockIdentifier::get(),
                &sender,
                amount,
                WithdrawReasons::TRANSFER,
            );

            <OutcomeVotes<T>>::insert(
                market_id,
                outcome_index,
                vote_balance.saturating_add(amount),
            );

            Self::deposit_event(Event::VotedOnOutcome(market_id, outcome_index, amount));
            Ok(Some(T::WeightInfo::vote_on_outcome()).into())
        }

        /// Unlock the dispute vote value of a global dispute when the 'DisputePeriod' is over.
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::unlock_vote_balance())]
        pub fn unlock_vote_balance(
            origin: OriginFor<T>,
            voter: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            let mut lock_needed: BalanceOf<T> = Zero::zero();
            let mut resolved_markets = Vec::new();
            for (market_id, locked_balance) in <LockInfoOf<T>>::iter_prefix(&voter) {
                if <OutcomeVotes<T>>::iter_prefix(market_id).take(1).next().is_none() {
                    resolved_markets.push(market_id);
                    continue;
                }
                lock_needed = lock_needed.max(locked_balance);
            }

            for market_id in resolved_markets {
                <LockInfoOf<T>>::remove(&voter, market_id);
            }

            if lock_needed.is_zero() {
                CurrencyOf::<T>::remove_lock(T::VoteLockIdentifier::get(), &voter);
            } else {
                CurrencyOf::<T>::set_lock(
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

        /// The identifier of individual markets.
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The pallet identifier.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The vote lock identifier for disputes
        #[pallet::constant]
        type VoteLockIdentifier: Get<LockIdentifier>;

        /// The minimum required amount to vote on a dispute.
        #[pallet::constant]
        type MinDisputeVoteAmount: Get<BalanceOf<Self>>;

        /// The minimum number of outcomes required to allow voting.
        #[pallet::constant]
        type MinOutcomes: Get<u32>;

        #[pallet::constant]
        type MaxOutcomeLimit: Get<u32>;

        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 1. Any resolution must either have a `Disputed` or `Reported` market status
        /// 2. If status is `Disputed`, then at least one dispute must exist
        InvalidMarketStatus,
        /// On dispute or resolution, someone tried to pass a non-global-disputes market type.
        MarketDoesNotHaveGlobalDisputesMechanism,
        /// The vote on this outcome index is not allowed, because there are not at least a minimum number of outcomes.
        NotEnoughOutcomes,
        /// The dispute specified with market id and outcome index is not present.
        OutcomeDoesNotExist,
        /// Sender does not have enough funds for the vote on an outcome.
        InsufficientAmount,
        /// Sender tried to vote with an amount below a defined minium.
        AmountTooLow,
        /// The vote outcome is already present.
        VoteOutcomeAlreadyExists,
        /// There is no default outcome set in the first place to resolve to.
        NoDefaultOutcome,
        /// The number of maximum outcomes is reached.
        MaxOutcomeLimitReached,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A vote happened on an Outcome. \[market_id, outcome_index, vote_amount\]
        VotedOnOutcome(MarketIdOf<T>, u32, BalanceOf<T>),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    impl<T: Config> Pallet<T> {
        fn get_default_outcome_and_index(
            market_id: &MarketIdOf<T>,
        ) -> Option<(u32, OutcomeReport)> {
            // return first element if the BoundedVec is not empty, otherwise None
            <Outcomes<T>>::get(market_id).get(0usize).map(|o| (0u32, o.clone()))
        }

        fn get_more_recent_outcome_index(x: u32, y: u32) -> u32 {
            // return more recent element => is last added, so the higher index
            x.max(y)
        }
    }

    impl<T> GlobalDisputesPalletApi for Pallet<T>
    where
        T: Config,
    {
        type Balance = BalanceOf<T>;
        type MarketId = MarketIdOf<T>;

        /// This is the initial voting balance of the outcome
        fn push_voting_outcome(
            market_id: &MarketIdOf<T>,
            outcome: OutcomeReport,
            vote_balance: BalanceOf<T>,
        ) -> Result<(), DispatchError> {
            let mut outcomes = <Outcomes<T>>::get(market_id);
            ensure!(!outcomes.iter().any(|o| *o == outcome), Error::<T>::VoteOutcomeAlreadyExists);
            ensure!(outcomes.try_push(outcome).is_ok(), Error::<T>::MaxOutcomeLimitReached);
            let outcome_index = outcomes.len().saturated_into::<u32>().saturating_sub(One::one());
            <Outcomes<T>>::insert(market_id, outcomes);
            <OutcomeVotes<T>>::insert(market_id, outcome_index, vote_balance);
            Ok(())
        }

        fn get_voting_winner(market_id: &Self::MarketId) -> Result<OutcomeReport, DispatchError> {
            let (default_outcome_index, default_outcome) =
                Self::get_default_outcome_and_index(market_id)
                    .ok_or(<Error<T>>::NoDefaultOutcome)?;
            let (winning_outcome_index, _) = <OutcomeVotes<T>>::drain_prefix(market_id).fold(
                (default_outcome_index, <BalanceOf<T>>::zero()),
                |(o0, b0), (o1, b1)| match b0.cmp(&b1) {
                    Ordering::Greater => (o0, b0),
                    Ordering::Less => (o1, b1),
                    Ordering::Equal => (Self::get_more_recent_outcome_index(o0, o1), b0),
                },
            );

            let winning_outcome = <Outcomes<T>>::get(market_id)
                .get(winning_outcome_index as usize)
                .map(|o| o.clone())
                .unwrap_or(default_outcome);
            Ok(winning_outcome)
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::storage]
    pub type Outcomes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        BoundedVec<OutcomeReport, T::MaxOutcomeLimit>,
        ValueQuery,
    >;

    /// Maps the market id to the dispute index and the vote balance.  
    #[pallet::storage]
    pub type OutcomeVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        Blake2_128Concat,
        u32,
        BalanceOf<T>,
        OptionQuery,
    >;

    /// All lock information (market id and locked balance) for a particular voter.
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

#[cfg(any(feature = "runtime-benchmarks", test))]
pub(crate) fn market_mock<T>() -> zeitgeist_primitives::types::Market<
    T::AccountId,
    T::BlockNumber,
    <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment,
>
where
    T: crate::Config,
{
    use frame_support::traits::Get;
    use sp_runtime::traits::AccountIdConversion;
    use zeitgeist_primitives::types::ScoringRule;

    zeitgeist_primitives::types::Market {
        creation: zeitgeist_primitives::types::MarketCreation::Permissionless,
        creator_fee: 0,
        creator: T::PalletId::get().into_account(),
        market_type: zeitgeist_primitives::types::MarketType::Scalar(0..=100),
        dispute_mechanism: zeitgeist_primitives::types::MarketDisputeMechanism::GlobalDisputes,
        metadata: Default::default(),
        oracle: T::PalletId::get().into_account(),
        period: zeitgeist_primitives::types::MarketPeriod::Block(Default::default()),
        report: None,
        resolved_outcome: None,
        scoring_rule: ScoringRule::CPMM,
        status: zeitgeist_primitives::types::MarketStatus::Disputed,
    }
}
