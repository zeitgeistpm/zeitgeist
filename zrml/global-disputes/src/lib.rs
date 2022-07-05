//! # Global disputes
//!
//! Manages market disputes and resolutions.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod global_disputes_pallet_api;
mod mock;
mod tests;
mod benchmarks;

pub use global_disputes_pallet_api::GlobalDisputesPalletApi;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::GlobalDisputesPalletApi;
    use alloc::vec::Vec;
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        ensure,
        pallet_prelude::{OptionQuery, StorageDoubleMap, StorageMap, ValueQuery, Weight},
        traits::{Currency, Get, Hooks, IsType, LockIdentifier, LockableCurrency, WithdrawReasons},
        Blake2_128Concat, BoundedVec, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::{
        traits::{Saturating, Zero},
        DispatchError,
    };
    use zeitgeist_primitives::{
        traits::DisputeApi,
        types::{Market, MarketDispute, MarketDisputeMechanism, MarketStatus, OutcomeReport},
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    pub(crate) type BalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Votes on a dispute after there are already two disputes and the 'DisputePeriod' is not over.
        /// NOTE: In the 'DisputePeriod' voting on a dispute is allowed.
        #[pallet::weight(10_000_000)]
        pub fn vote(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] dispute_index: u32,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            ensure!(
                amount <= CurrencyOf::<T>::free_balance(&sender),
                Error::<T>::InsufficientFundsForVote
            );
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Disputed, Error::<T>::InvalidMarketStatus);

            // for voting there must be at least two disputes
            let mut iter = <DisputeVotes<T>>::iter_prefix(market_id).take(2);
            ensure!(iter.next() != None && iter.next() != None, Error::<T>::NotEnoughDisputes);

            // dispute vote is already present because of the dispute bond of the disputor
            let dispute_vote = <DisputeVotes<T>>::get(market_id, dispute_index)
                .ok_or(Error::<T>::DisputeDoesNotExist)?;

            <LockInfoOf<T>>::try_mutate(&sender, |locks_info| {
                locks_info.try_push((market_id, amount)).map_err(|_| <Error<T>>::StorageOverflow)
            })?;

            CurrencyOf::<T>::extend_lock(
                T::VoteLockIdentifier::get(),
                &sender,
                amount,
                WithdrawReasons::all(),
            );

            let vote_balance = dispute_vote.saturating_add(amount);
            <DisputeVotes<T>>::insert(market_id, dispute_index, vote_balance);

            Self::deposit_event(Event::VotedOnDispute(market_id, dispute_index, amount));
            Ok(())
        }

        /// Unlock the dispute vote value of a global dispute when the 'DisputePeriod' is over.
        #[pallet::weight(10_000_000)]
        pub fn unlock(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let mut locks_info = <LockInfoOf<T>>::get(&sender);

            let mut disputed_markets = Vec::new();
            // find all disputes which are not resolved yet
            for (market_id, _) in <DisputeVotes<T>>::iter_keys() {
                disputed_markets.push(market_id);
            }

            // remove all items which are resolved
            locks_info.retain(|(market_id, _)| disputed_markets.contains(market_id));

            let lock_needed: BalanceOf<T> = locks_info
                .clone()
                .into_inner()
                .iter()
                .map(|(_, locked_balance)| locked_balance)
                .fold(Zero::zero(), |b0, b1| b0.max(*b1));

            if !locks_info.len().is_zero() {
                <LockInfoOf<T>>::insert(&sender, locks_info);
            } else {
                <LockInfoOf<T>>::remove(&sender);
            }

            if lock_needed.is_zero() {
                CurrencyOf::<T>::remove_lock(T::VoteLockIdentifier::get(), &sender);
            } else {
                CurrencyOf::<T>::set_lock(
                    T::VoteLockIdentifier::get(),
                    &sender,
                    lock_needed,
                    WithdrawReasons::all(),
                );
            }

            Ok(())
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

        #[pallet::constant]
        type MaxDisputeLocks: Get<u32>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 1. Any resolution must either have a `Disputed` or `Reported` market status
        /// 2. If status is `Disputed`, then at least one dispute must exist
        InvalidMarketStatus,
        /// On dispute or resolution, someone tried to pass a non-global-disputes market type
        MarketDoesNotHaveGlobalDisputesMechanism,
        /// An initial vote balance was already made for this dispute.
        DisputeVoteAlreadyPresent,
        /// The vote on this dispute index is not allowed, because there are not at least two disputes.
        NotEnoughDisputes,
        /// The dispute specified with market id and dispute index is not present.
        DisputeDoesNotExist,
        /// Sender does not have enough funds for the vote on a dispute.
        InsufficientFundsForVote,
        /// The storage has overflown.
        StorageOverflow,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A vote happened on a dispute. \[market_id, dispute_index, vote_amount\]
        VotedOnDispute(MarketIdOf<T>, u32, BalanceOf<T>),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    impl<T> DisputeApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type Balance = BalanceOf<T>;
        type BlockNumber = T::BlockNumber;
        type MarketId = MarketIdOf<T>;
        type Moment = MomentOf<T>;
        type Origin = T::Origin;

        fn on_dispute(
            _disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            _market_id: &Self::MarketId,
            market: &Market<Self::AccountId, Self::BlockNumber, MomentOf<T>>,
        ) -> DispatchResult {
            if market.mdm != MarketDisputeMechanism::GlobalDisputes {
                return Err(Error::<T>::MarketDoesNotHaveGlobalDisputesMechanism.into());
            }

            Ok(())
        }

        fn on_resolution(
            disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: &Self::MarketId,
            market: &Market<Self::AccountId, Self::BlockNumber, MomentOf<T>>,
        ) -> Result<Option<OutcomeReport>, DispatchError> {
            if market.mdm != MarketDisputeMechanism::GlobalDisputes {
                return Err(Error::<T>::MarketDoesNotHaveGlobalDisputesMechanism.into());
            }
            if market.status != MarketStatus::Disputed {
                return Err(Error::<T>::InvalidMarketStatus.into());
            }

            let (index, _) = <DisputeVotes<T>>::drain_prefix(market_id).fold(
                (0u32, <BalanceOf<T>>::zero()),
                |(i0, b0), (i1, b1)| {
                    if b0 > b1 { (i0, b0) } else { (i1, b1) }
                },
            );

            if let Some(winning_dispute) = disputes.get(index as usize) {
                Ok(Some(winning_dispute.outcome.clone()))
            } else {
                Err(Error::<T>::InvalidMarketStatus.into())
            }
        }
    }

    impl<T> GlobalDisputesPalletApi for Pallet<T>
    where
        T: Config,
    {
        /// This is the initial voting balance of the dispute
        fn init_dispute_vote(
            market_id: &MarketIdOf<T>,
            dispute_index: u32,
            vote_balance: BalanceOf<T>,
        ) -> Weight {
            if <DisputeVotes<T>>::get(market_id, dispute_index).is_none() {
                <DisputeVotes<T>>::insert(market_id, dispute_index, vote_balance);
                return T::DbWeight::get()
                    .writes(1 as Weight)
                    .saturating_add(T::DbWeight::get().reads(1 as Weight));
            }
            T::DbWeight::get().reads(1 as Weight)
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// Maps the market id to the dispute index and the vote balance.  
    #[pallet::storage]
    pub type DisputeVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        Blake2_128Concat,
        u32,
        BalanceOf<T>,
        OptionQuery,
    >;

    /// All lock information (market_id, end_block, balance) for a particular voter.
    ///
    /// TWOX-NOTE: SAFE as `AccountId`s are crypto hashes anyway.
    #[pallet::storage]
    pub type LockInfoOf<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AccountId,
        BoundedVec<(MarketIdOf<T>, BalanceOf<T>), T::MaxDisputeLocks>,
        ValueQuery,
    >;
}

#[cfg(any(feature = "runtime-benchmarks", test))]
pub(crate) fn market_mock<T>(
    ai: T::AccountId,
) -> zeitgeist_primitives::types::Market<T::AccountId, T::BlockNumber, MomentOf<T>>
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
        mdm: zeitgeist_primitives::types::MarketDisputeMechanism::GlobalDisputes,
        metadata: Default::default(),
        oracle: T::PalletId::get().into_account(),
        period: zeitgeist_primitives::types::MarketPeriod::Block(Default::default()),
        report: None,
        resolved_outcome: None,
        scoring_rule: ScoringRule::CPMM,
        status: zeitgeist_primitives::types::MarketStatus::Disputed,
    }
}

