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
        dispatch::DispatchResult,
        ensure,
        pallet_prelude::{DispatchResultWithPostInfo, OptionQuery, StorageDoubleMap, Weight},
        traits::{Currency, Get, Hooks, IsType, LockIdentifier, LockableCurrency, WithdrawReasons},
        Blake2_128Concat, PalletId, Twox64Concat,
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
        #[frame_support::transactional]
        #[pallet::weight(T::WeightInfo::vote_on_dispute())]
        pub fn vote_on_dispute(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] dispute_index: u32,
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

            // for voting there must be at least two disputes
            let mut iter = <DisputeVotes<T>>::iter_prefix(market_id).take(2);
            ensure!(iter.next().is_some() && iter.next().is_some(), Error::<T>::NotEnoughDisputes);

            // dispute vote is already present because of the dispute bond of the disputor
            let vote_balance = <DisputeVotes<T>>::get(market_id, dispute_index)
                .ok_or(Error::<T>::DisputeDoesNotExist)?;

            <LockInfoOf<T>>::mutate(&sender, market_id, |locked_balance| {
                *locked_balance = Some(locked_balance.map_or(amount, |x| x.max(amount)));
            });

            CurrencyOf::<T>::extend_lock(
                T::VoteLockIdentifier::get(),
                &sender,
                amount,
                WithdrawReasons::TRANSFER,
            );

            <DisputeVotes<T>>::insert(
                market_id,
                dispute_index,
                vote_balance.saturating_add(amount),
            );

            Self::deposit_event(Event::VotedOnDispute(market_id, dispute_index, amount));
            Ok(Some(T::WeightInfo::vote_on_dispute()).into())
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
                if <DisputeVotes<T>>::iter_prefix(market_id).take(1).next().is_none() {
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

        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 1. Any resolution must either have a `Disputed` or `Reported` market status
        /// 2. If status is `Disputed`, then at least one dispute must exist
        InvalidMarketStatus,
        /// On dispute or resolution, someone tried to pass a non-global-disputes market type
        MarketDoesNotHaveGlobalDisputesMechanism,
        /// The vote on this dispute index is not allowed, because there are not at least two disputes.
        NotEnoughDisputes,
        /// The dispute specified with market id and dispute index is not present.
        DisputeDoesNotExist,
        /// Sender does not have enough funds for the vote on a dispute.
        InsufficientAmount,
        /// Sender tried to vote with an amount below a defined minium.
        AmountTooLow,
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
            if market.dispute_mechanism != MarketDisputeMechanism::GlobalDisputes {
                return Err(Error::<T>::MarketDoesNotHaveGlobalDisputesMechanism.into());
            }

            Ok(())
        }

        fn on_resolution(
            disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: &Self::MarketId,
            market: &Market<Self::AccountId, Self::BlockNumber, MomentOf<T>>,
        ) -> Result<Option<OutcomeReport>, DispatchError> {
            if market.dispute_mechanism != MarketDisputeMechanism::GlobalDisputes {
                return Err(Error::<T>::MarketDoesNotHaveGlobalDisputesMechanism.into());
            }
            if market.status != MarketStatus::Disputed {
                return Err(Error::<T>::InvalidMarketStatus.into());
            }

            let (index, _) = <DisputeVotes<T>>::drain_prefix(market_id).fold(
                (0u32, <BalanceOf<T>>::zero()),
                |(i0, b0), (i1, b1)| {
                    match b0.cmp(&b1) {
                        Ordering::Greater => (i0, b0),
                        Ordering::Less => (i1, b1),
                        // if the vote balance is the same on multiple outcomes, the in time last should be taken, because it's the one with the most dispute bond and less voting time
                        Ordering::Equal => (i0.max(i1), b0),
                    }
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
            // TODO(#603) fix weight calc
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
pub(crate) fn market_mock<T>()
-> zeitgeist_primitives::types::Market<T::AccountId, T::BlockNumber, MomentOf<T>>
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
