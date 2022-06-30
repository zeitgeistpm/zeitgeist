//! # Global disputes
//!
//! Manages market disputes and resolutions.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod global_disputes_pallet_api;
mod mock;
mod tests;

pub use global_disputes_pallet_api::GlobalDisputesPalletApi;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::GlobalDisputesPalletApi;
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        ensure,
        pallet_prelude::{OptionQuery, StorageDoubleMap, StorageMap, ValueQuery, Weight},
        traits::{Currency, Get, Hooks, IsType, LockIdentifier, LockableCurrency, WithdrawReasons},
        Blake2_128Concat, PalletId,
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

    type BalanceOf<T> =
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
        pub fn vote_on_dispute(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] dispute_index: u32,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            // TODO(#489): Implement global disputes!
            let sender = ensure_signed(origin)?;
            ensure!(
                amount <= CurrencyOf::<T>::free_balance(&sender),
                Error::<T>::InsufficientFundsForVote
            );
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Disputed, Error::<T>::InvalidMarketStatus);

            ensure!(<Whitelist<T>>::get(market_id), Error::<T>::DisputeVoteNotAllowed);

            // dispute vote is already present because of the dispute bond of the disputor
            let dispute_vote = <DisputeVote<T>>::get(market_id, dispute_index)
                .ok_or(Error::<T>::DisputeVoteNotAllowed)?;

            CurrencyOf::<T>::extend_lock(
                T::VoteLockIdentifier::get(),
                &sender,
                amount,
                WithdrawReasons::TRANSFER,
            );

            let dispute_vote = dispute_vote.saturating_add(amount);
            <DisputeVote<T>>::insert(market_id, dispute_index, dispute_vote);

            Self::deposit_event(Event::VotedOnDispute(market_id, dispute_index, amount));
            Ok(())
        }

        /// Unlock the dispute vote value of a global dispute when the 'DisputePeriod' is over.
        #[pallet::weight(10_000_000)]
        pub fn unlock_dispute_vote(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let lock_needed = VotingOf::<T>::mutate(sender, |voting| {
                voting.rejig(frame_system::Pallet::<T>::block_number());
                voting.locked_balance()
            });
            if lock_needed.is_zero() {
                CurrencyOf::<T>::remove_lock(T::VoteLockIdentifier::get(), &sender);
            } else {
                CurrencyOf::<T>::set_lock(
                    T::VoteLockIdentifier::get(),
                    &sender,
                    lock_needed,
                    WithdrawReasons::TRANSFER,
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
        /// The vote on this dispute index is not allowed.
        DisputeVoteNotAllowed,
        /// Sender does not have enough funds for the vote on a dispute.
        InsufficientFundsForVote,
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
            disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
            market_id: &Self::MarketId,
            market: &Market<Self::AccountId, Self::BlockNumber, MomentOf<T>>,
        ) -> DispatchResult {
            if market.mdm != MarketDisputeMechanism::GlobalDisputes {
                return Err(Error::<T>::MarketDoesNotHaveGlobalDisputesMechanism.into());
            }
            // allow voting on dispute when on whitelist
            // on_dispute is called before the push to disputes (pm pallet)
            if !<Whitelist<T>>::get(market_id) && !disputes.len().is_zero() {
                // when the above condition is true, then disputes will have two elements
                // only allow voting with at least two disputes
                <Whitelist<T>>::insert(market_id, true);
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
            let (index, _) = <DisputeVote<T>>::iter_prefix(market_id).fold(
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

    impl<T> GlobalDisputesPalletApi for Pallet<T> where T: Config {}

    impl<T: Config> Pallet<T> {
        /// This is the initial voting balance of the dispute
        pub fn init_dispute_vote(
            market_id: &MarketIdOf<T>,
            dispute_index: u32,
            vote_balance: BalanceOf<T>,
        ) -> Weight {
            if <DisputeVote<T>>::get(market_id, dispute_index).is_none() {
                <DisputeVote<T>>::insert(market_id, dispute_index, vote_balance);
                // TODO storage read and write weight
                return 0;
            }
            // TODO storage read weight
            0
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// Maps the market id to the dispute index and the vote balance.  
    #[pallet::storage]
    pub type DisputeVote<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        Blake2_128Concat,
        u32,
        BalanceOf<T>,
        OptionQuery,
    >;

    #[pallet::storage]
    pub type Whitelist<T: Config> =
        StorageMap<_, Blake2_128Concat, MarketIdOf<T>, bool, ValueQuery>;
}
