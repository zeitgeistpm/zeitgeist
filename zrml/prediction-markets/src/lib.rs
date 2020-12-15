//! # Prediction Markets
//!
//! A module for creating, reporting, and disputing prediction markets.
//! 
//! ## Overview
//!
//! TODO
//!
//! ## Interface
//!
//! ### Dispatches
//!
//! #### Public Dispatches 
//!
//! - `create` - Creates a market which then can have its shares be bought or sold.
//! - `buy_complete_set` - Purchases and generates a complete set of outcome shares for a
//!  specific market.
//! - `sell_complete_set` - Sells and destorys a complete set of outcome shares for a market.
//! - `report` -
//! - `dispute` -
//! - `global_dispute` - TODO
//! - `redeem_shares` -
//! 
//! #### `ApprovalOrigin` Dispatches
//!
//! - `approve_market` - Can only be called by the `ApprovalOrigin`. Approves a market
//!  that is waiting for approval from the advisory committee.
//! - `reject_market` - Can only be called by the `ApprovalOrigin`. Rejects a market that
//!  is waiting for approval from the advisory committee.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, 
    ensure, Parameter,
};
use frame_support::traits::{
    Currency, ReservableCurrency, Get, ExistenceRequirement,
    EnsureOrigin, OnUnbalanced, Imbalance,
};
use frame_system::ensure_signed;
use sp_runtime::{ModuleId, SaturatedConversion, DispatchResult};
use sp_runtime::traits::{
    AccountIdConversion, AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, 
    Member, One, Hash, Zero,
};
use sp_std::cmp;
use sp_std::vec::Vec;
use zrml_traits::shares::{Shares, WrapperShares};
use zrml_traits::swaps::Swaps;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod market;
use market::{Market, MarketCreation, MarketDispute, MarketStatus, MarketType};

fn remove_item<I: cmp::PartialEq + Copy>(items: &mut Vec<I>, item: I) {
    let pos = items.iter().position(|&i| i == item).unwrap();
    items.swap_remove(pos);
}

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

type NegativeImbalanceOf<T> =
	<<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

pub trait Trait: frame_system::Trait + pallet_timestamp::Trait  {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    type Currency: ReservableCurrency<Self::AccountId>;

    type Shares: Shares<Self::AccountId, BalanceOf<Self>, Self::Hash> + WrapperShares<Self::AccountId, BalanceOf<Self>, Self::Hash>;

    /// The identifier of individual markets.
    type MarketId: AtLeast32Bit + Parameter + Member + MaybeSerializeDeserialize + Default + Copy;

    /// The module identifier.
    type ModuleId: Get<ModuleId>;

    /// The number of blocks the reporting period remains open.
    type ReportingPeriod: Get<Self::BlockNumber>;

    /// The number of blocks the dispute period remains open.
    type DisputePeriod: Get<Self::BlockNumber>;

    /// The base amount of currency that must be bonded for a market approved by the
    ///  advisory committee.
    type AdvisoryBond: Get<BalanceOf<Self>>;

    /// The base amount of currency that must be bonded in order to create a dispute.
    type DisputeBond: Get<BalanceOf<Self>>;

    /// The additional amount of currency that must be bonded when creating a subsequent
    ///  dispute.
    type DisputeFactor: Get<BalanceOf<Self>>;

    /// The maximum number of disputes allowed on any single market.
    type MaxDisputes: Get<u16>;

    /// The base amount of currency that must be bonded to ensure the oracle reports
    ///  in a timely manner.
    type OracleBond: Get<BalanceOf<Self>>;

    /// The base amount of currency that must be bonded for a permissionless market,
    /// guaranteeing that it will resolve as anything but `Invalid`.
    type ValidityBond: Get<BalanceOf<Self>>;

    type ApprovalOrigin: EnsureOrigin<<Self as frame_system::Trait>::Origin>;

    type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

    type Swap: Swaps<Self::AccountId, BalanceOf<Self>, Self::Hash>;
}

decl_storage! {
    trait Store for Module<T: Trait> as PredictionMarkets {
        /// Stores all of the actual market data.
        Markets get(fn markets):
            map hasher(blake2_128_concat) T::MarketId => 
                Option<Market<T::AccountId>>;

        /// The number of markets that have been created and the next identifier
        /// for a created market.
        MarketCount get(fn market_count): T::MarketId;

        /// A mapping of market identifiers to the block that they were reported on.
        MarketIdsPerReportBlock get(fn market_ids_per_report_block):
            map hasher(blake2_128_concat) T::BlockNumber => Vec<T::MarketId>;

        /// A mapping of market identifiers to the block they were disputed at.
        ///  A market only ends up here if it was disputed.
        MarketIdsPerDisputeBlock get(fn market_ids_per_dispute_block):
            map hasher(blake2_128_concat) T::BlockNumber => Vec<T::MarketId>;

        /// For each market, this holds the dispute information for each dispute that's
        ///  been issued.
        Disputes get(fn disputes):
            map hasher(blake2_128_concat) T::MarketId =>
                Vec<MarketDispute<T::AccountId, T::BlockNumber>>;

        MarketToSwapPool get(fn market_to_swap_pool):
            map hasher(blake2_128_concat) T::MarketId =>
                Option<u128>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        MarketId = <T as Trait>::MarketId,
    {
        /// A market has been created [market_id, creator]
        MarketCreated(MarketId, AccountId),
        /// A pending market has been cancelled. [market_id, creator]
        MarketCancelled(MarketId), /// NOTE: Maybe we should only allow rejections.
        /// A pending market has been rejected as invalid. [market_id]
        MarketRejected(MarketId),
        /// A market has been approved [market_id]
        MarketApproved(MarketId),
        /// A complete set of shares has been bought [market_id, buyer]
        BoughtCompleteSet(MarketId, AccountId),
        /// A complete set of shares has been sold [market_id, seller]
        SoldCompleteSet(MarketId, AccountId),
        /// A market has been reported on [market_id, reported_outcome]
        MarketReported(MarketId, u16),
        /// A market has been disputed [market_id, actual_outcome]
        MarketDisputed(MarketId, u16),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// A market with the provided ID does not exist.
        MarketDoesNotExist,
        /// Sender does not have enough balance to buy shares.
        NotEnoughBalance,
        /// The market status is something other than active.
        MarketNotActive,
        /// Sender does not have enough share balance.
        InsufficientShareBalance,
        /// The order hash was not found in the pallet.
        OrderDoesntExist,
        /// The user has a share balance that is too low.
        ShareBalanceTooLow,
        /// The order has already been taken.
        OrderAlreadyTaken,
        /// The sender's balance is too low to take this order.
        CurrencyBalanceTooLow,
        /// The market identity was not found in the pallet.
        MarketDoesntExist,
        /// The market is not resolved.
        MarketNotResolved,
        /// The user has no winning balance.
        NoWinningBalance,
        /// Market account does not have enough funds to pay out.
        InsufficientFundsInMarketAccount,
        /// The report is not coming from designated oracle.
        ReporterNotOracle,
        /// The market is not closed.
        MarketNotClosed,
        /// The market is not overdue.
        MarketNotOverdue,
        /// The market is not reported on.
        MarketNotReported,
        /// The maximum number of disputes has been reached.
        MaxDisputesReached,
        /// Someone is trying to call `dispute` with the same outcome that is currently
        ///  registered on-chain.
        CannotDisputeSameOutcome,
        /// The outcome being reported is out of range.
        OutcomeOutOfRange,
        /// Market is already reported on.
        MarketAlreadyReported,
        /// A swap pool already exists for this market.
        SwapPoolExists,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
 
        const AdvisoryBond: BalanceOf<T> = T::AdvisoryBond::get();

        const DisputeBond: BalanceOf<T> = T::DisputeBond::get();

        const DisputeFactor: BalanceOf<T> = T::DisputeFactor::get();

        const DisputePeriod: T::BlockNumber = T::DisputePeriod::get();

        const OracleBond: BalanceOf<T> = T::OracleBond::get();

        const ValidityBond: BalanceOf<T> = T::ValidityBond::get();

        type Error = Error<T>;

        fn deposit_event() = default;

        /// The finalize function will move all reported markets to resolved.
        ///
        /// Disputed markets need to be resolved manually.
        ///
        fn on_finalize(now: T::BlockNumber) {
            let dispute_period = T::DisputePeriod::get();
            if now <= dispute_period { return; }

            // Resolve all regularly reported markets.
            let market_ids = Self::market_ids_per_report_block(now - dispute_period);
            if !market_ids.is_empty() {
                market_ids.iter().for_each(|id| {
                    let market = Self::markets(id).unwrap();
                    if market.status != MarketStatus::Reported { }
                     else { Self::internal_resolve(id); }                    
                });
            }            

            // Resolve any disputed markets.
            let disputed = Self::market_ids_per_dispute_block(now - dispute_period);
            if !disputed.is_empty() {
                disputed.iter().for_each(|id| {
                    Self::internal_resolve(id);
                });
            }
        }

        /// Allows Root to destroy markets.
        ///
        /// todo: this should check if there's any outstanding funds reserved if it stays
        /// in for production
        #[weight = 0]
        pub fn destroy_market(origin, market_id: T::MarketId) {
            T::ApprovalOrigin::ensure_origin(origin)?;

            if let Some(market) = Self::markets(&market_id) {

                <Markets<T>>::remove(&market_id);

                // delete all the shares if any exist
                for i in 0..=market.outcomes() {
                    let share_id = Self::market_outcome_share_id(market_id.clone(), i);
                    T::Shares::destroy_all(share_id).unwrap();
                }  
            } else {
                Err(Error::<T>::MarketDoesNotExist)?;
            }
        }

        /// Creates a new prediction market, seeded with the intial values.
        ///
        #[weight = 0]
        pub fn create(
            origin,
            oracle: T::AccountId,
            market_type: MarketType,
            end: u64,
            metadata: Vec<u8>,
            creation: MarketCreation,
        ) {
            let sender = ensure_signed(origin)?;

            // PoC - Only binary markets are currently supported.
            ensure!(market_type == MarketType::Binary, "Only binary markets are currently supported.");

            // Check the end is in the future.
            if end > 1_000_000_000 {
                // unix timestamp
                let now = <pallet_timestamp::Module<T>>::get();
                ensure!(now < end.saturated_into(), "End timestamp must be before now.");
            } else {
                // block
                let current_block = <frame_system::Module<T>>::block_number();
                ensure!(current_block < end.saturated_into(), "End block must be in the future.");
            }

            // This will check if the length is correct for an IPFS CID
            // ensure!(metadata.length == 46, "Incorrect metadata length");

            let status: MarketStatus = match creation {
                MarketCreation::Permissionless => {
                    let required_bond = T::ValidityBond::get() + T::OracleBond::get();
                    T::Currency::reserve(&sender, required_bond)?;
                    MarketStatus::Active
                }
                MarketCreation::Advised => {
                    let required_bond = T::AdvisoryBond::get() + T::OracleBond::get();
                    T::Currency::reserve(&sender, required_bond)?;
                    MarketStatus::Proposed
                }
            };

            let new_market_id = Self::get_next_market_id()?;
            let new_market = Market {
                creator: sender.clone(),
                creation,
                creator_fee: 0,
                oracle,
                end,
                metadata,
                market_type,
                status,
                reported_outcome: None,
                reporter: None,
                categories: None,
            };

            <Markets<T>>::insert(new_market_id.clone(), new_market);

            Self::deposit_event(RawEvent::MarketCreated(new_market_id, sender));
        }

        /// Approves a market that is waiting for approval from the
        /// advisory committee.
        ///
        /// NOTE: Returns the proposer's bond since the market has been
        /// deemed valid by an advisory committee.
        ///
        /// NOTE: Can only be called by the `ApprovalOrigin`.
        ///
        #[weight = 0]
        pub fn approve_market(origin, market_id: T::MarketId) {
            T::ApprovalOrigin::ensure_origin(origin)?;

            if let Some(market) = Self::markets(&market_id) {
                let creator = market.creator;
                
                T::Currency::unreserve(&creator, T::AdvisoryBond::get());
                <Markets<T>>::mutate(&market_id, |m| {
                    m.as_mut().unwrap().status = MarketStatus::Active;
                });
    
                Self::deposit_event(RawEvent::MarketApproved(market_id));
            } else {
                Err(Error::<T>::MarketDoesNotExist)?;
            }
        }


        /// Rejects a market that is waiting for approval from the advisory
        /// committee.
        ///
        /// NOTE: Will slash the reserved `AdvisoryBond` from the market creator.
        ///
        #[weight = 0]
        pub fn reject_market(origin, market_id: T::MarketId) {
            T::ApprovalOrigin::ensure_origin(origin)?;

            if let Some(market) = Self::markets(&market_id) {
                let creator = market.creator;
                let (imbalance, _) =  T::Currency::slash_reserved(&creator, T::AdvisoryBond::get());
                // Slashes the imbalance.
                T::Slash::on_unbalanced(imbalance);
                <Markets<T>>::remove(&market_id);
                Self::deposit_event(RawEvent::MarketRejected(market_id));
            } else {
                Err(Error::<T>::MarketDoesNotExist)?;
            }
        }

        /// NOTE: Only for PoC probably - should only allow rejections
        /// in a production environment since this better aligns incentives.
        /// See also: Polkadot Treasury
        ///
        #[weight = 0]
        pub fn cancel_pending_market(origin, market_id: T::MarketId) {
            let sender = ensure_signed(origin)?;

            if let Some(market) = <Markets<T>>::get(&market_id) {
                let creator = market.creator;
                let status = market.status;
                ensure!(creator == sender, "Canceller must be market creator.");
                ensure!(status == MarketStatus::Proposed, "Market must be pending approval.");
                // The market is being cancelled, return the deposit.
                T::Currency::unreserve(&creator, T::AdvisoryBond::get());
                <Markets<T>>::remove(&market_id);
                Self::deposit_event(RawEvent::MarketCancelled(market_id));
            } else {
                Err(Error::<T>::MarketDoesNotExist)?;
            }
        }

        /// Deploys a new pool for the market. This pallet keeps track of a single
        /// canonical swap pool for each market in `market_to_swap_pool`.
        ///
        /// The sender should have enough funds to cover all of the required
        /// shares to seed the pool.
        #[weight = 0]
        pub fn deploy_swap_pool_for_market(origin, market_id: T::MarketId) {
            let sender = ensure_signed(origin)?;

            if let Some(market) = <Markets<T>>::get(&market_id) {
                // ensure the market is active
                let status = market.status;
                ensure!(status == MarketStatus::Active, Error::<T>::MarketNotActive);

                // ensure a swap pool does not already exist
                ensure!(Self::market_to_swap_pool(&market_id).is_none(), Error::<T>::SwapPoolExists);

                let mut assets = Vec::new();
                let mut weights = Vec::new();
                assets.push(T::Shares::get_native_currency_id());
                weights.push(10_000_000_000_u128);
                for i in 0..=market.outcomes() {
                    assets.push(
                        Self::market_outcome_share_id(market_id, i)
                    );
                    weights.push(10_000_000_000_u128);
                }

                let pool_id = T::Swap::do_create_pool(sender, assets, Zero::zero(), weights)?;

                <MarketToSwapPool<T>>::insert(market_id, pool_id);
            } else {
                Err(Error::<T>::MarketDoesNotExist)?;
            }
        }

        /// Generates a complete set of outcome shares for a market.
        ///
        /// NOTE: This is the only way to create new shares.
        ///
        #[weight = 0]
        pub fn buy_complete_set(
            origin,
            market_id: T::MarketId,
            #[compact] amount: BalanceOf<T>, 
        ) {
            let sender = ensure_signed(origin)?;

            Self::do_buy_complete_set(sender, market_id, amount)?;
        }

        /// Destroys a complete set of outcomes shares for a market.
        ///
        #[weight = 0]
        pub fn sell_complete_set(
            origin,
            market_id: T::MarketId,
            #[compact] amount: BalanceOf<T>,
        ) {
            let sender = ensure_signed(origin)?;

            if let Some(market) = Self::markets(market_id.clone()) {
                ensure!(Self::is_market_active(market.end), Error::<T>::MarketNotActive);

                let market_account = Self::market_account(market_id.clone());
                ensure!(
                    T::Currency::free_balance(&market_account) >= amount,
                    "Market account does not have sufficient reserves.",
                );

                for i in 0..=market.outcomes() {
                    let share_id = Self::market_outcome_share_id(market_id.clone(), i);

                    // Ensures that the sender has sufficient amount of each
                    // share in the set.
                    ensure!(
                        T::Shares::free_balance(share_id, &sender) >= amount,
                        Error::<T>::InsufficientShareBalance,
                    );
                }

                // This loop must be done twice because we check the entire
                // set of shares before making any mutations to storage.
                for i in 0..=market.outcomes() {
                    let share_id = Self::market_outcome_share_id(market_id.clone(), i);

                    T::Shares::destroy(share_id, &sender, amount)?;
                }

                T::Currency::transfer(&market_account, &sender, amount, ExistenceRequirement::AllowDeath)?;

                Self::deposit_event(RawEvent::SoldCompleteSet(market_id, sender));
            } else {
                Err(Error::<T>::MarketDoesNotExist)?;
            }
        }

        /// Reports the outcome of a market.
        ///
        #[weight = 0]
        pub fn report(origin, market_id: T::MarketId, reported_outcome: u16) {
            let sender = ensure_signed(origin)?;

            if let Some(mut market) = Self::markets(market_id.clone()) {
                ensure!(reported_outcome <= market.outcomes(), Error::<T>::OutcomeOutOfRange);

                ensure!(market.status != MarketStatus::Reported, Error::<T>::MarketAlreadyReported);

                // ensure market is not active
                ensure!(!Self::is_market_active(market.end.clone()), Error::<T>::MarketNotClosed);

                let current_block = <frame_system::Module<T>>::block_number();

                if market.end > 1_000_000_000 {
                    // unix timestamp
                    let now = <pallet_timestamp::Module<T>>::get().saturated_into::<u64>();
                    let reporting_period_in_ms = T::ReportingPeriod::get().saturated_into::<u64>() * 6000;
                    if now <= market.end + reporting_period_in_ms {
                        ensure!(sender == market.oracle, Error::<T>::ReporterNotOracle);
                    } // otherwise anyone can be the reporter
                } else {
                    // blocks
                    let end: T::BlockNumber = market.end.saturated_into();
                    if current_block <= end + T::ReportingPeriod::get() {
                        ensure!(sender == market.oracle, Error::<T>::ReporterNotOracle);
                    } // otherwise anyone can be the reporter
                }

                market.reported_outcome = Some(reported_outcome);
                market.status = MarketStatus::Reported;
                market.reporter = Some(sender);
                <Markets<T>>::insert(market_id.clone(), market);

                <MarketIdsPerReportBlock<T>>::mutate(current_block, |v| {
                    v.push(market_id.clone());
                });

                Self::deposit_event(RawEvent::MarketReported(market_id, reported_outcome));
            } else {
                Err(Error::<T>::MarketDoesNotExist)?;
            }
        }

        /// Disputes a reported outcome.
        ///
        /// NOTE: Requires a `DisputeBond` + `DisputeFactor` * `num_disputes` amount of currency
        ///  to be reserved.
        ///
        #[weight = 0]
        pub fn dispute(origin, market_id: T::MarketId, outcome: u16) {
            let sender = ensure_signed(origin)?;

            if let Some(market) = Self::markets(market_id.clone()) {
                ensure!(market.status == MarketStatus::Reported || market.status == MarketStatus::Disputed, Error::<T>::MarketNotReported);
                ensure!(outcome <= market.outcomes(), Error::<T>::OutcomeOutOfRange);

                let disputes = Self::disputes(market_id.clone());
                let num_disputes = disputes.len() as u16;
                ensure!(num_disputes < T::MaxDisputes::get(), Error::<T>::MaxDisputesReached);

                if num_disputes > 0 {
                    ensure!(disputes[(num_disputes as usize) - 1].outcome != outcome, Error::<T>::CannotDisputeSameOutcome);
                }

                let dispute_bond = T::DisputeBond::get() + T::DisputeFactor::get() * num_disputes.into();
                T::Currency::reserve(&sender, dispute_bond)?;

                let current_block = <frame_system::Module<T>>::block_number();

                if num_disputes > 0 {
                    let prev_dispute = disputes[(num_disputes as usize) - 1].clone();
                    let at = prev_dispute.at;
                    let mut old_disputes_per_block = <MarketIdsPerDisputeBlock<T>>::get(at);
                    remove_item::<T::MarketId>(&mut old_disputes_per_block, market_id.clone());
                    <MarketIdsPerDisputeBlock<T>>::insert(at, old_disputes_per_block);
                }

                <MarketIdsPerDisputeBlock<T>>::mutate(current_block, |ids| {
                    ids.push(market_id.clone());
                });

                <Disputes<T>>::mutate(market_id.clone(), |disputes| {
                    disputes.push(MarketDispute {
                        at: current_block,
                        by: sender,
                        outcome,
                    })
                });

                // if not already in dispute
                if market.status != MarketStatus::Disputed {
                    <Markets<T>>::mutate(market_id.clone(), |m| {
                        m.as_mut().unwrap().status = MarketStatus::Disputed;
                    });
                }

                Self::deposit_event(RawEvent::MarketDisputed(market_id, outcome));
            } else {
                Err(Error::<T>::MarketDoesNotExist)?;
            }
        }

        /// Starts a global dispute.
        ///
        /// NOTE: Requires the market to be already disputed `MaxDisputes` amount of times.
        ///
        #[weight = 0]
        pub fn global_dispute(origin, market_id: T::MarketId) {
            let _sender = ensure_signed(origin)?;
            if let Some(_market) = Self::markets(market_id.clone()) {
                // TODO: implement global disputes
            } else {
                Err(Error::<T>::MarketDoesNotExist)?;
            }

        }

        /// Redeems the winning shares of a prediction market.
        ///
        #[weight = 0]
        pub fn redeem_shares(origin, market_id: T::MarketId) {
            let sender = ensure_signed(origin)?;

            if let Some(market) = Self::markets(market_id.clone()) {
                ensure!(
                    market.status == MarketStatus::Resolved,
                    Error::<T>::MarketNotResolved,
                );

                // Check to see if the sender has any winning shares.
                let reported_outcome = market.reported_outcome.unwrap();
                let winning_shares_id = Self::market_outcome_share_id(market_id.clone(), reported_outcome);
                let winning_balance = T::Shares::free_balance(winning_shares_id, &sender);

                ensure!(
                    winning_balance >= 0.into(),
                    Error::<T>::NoWinningBalance,
                );

                // Ensure the market account has enough to pay out - if this is
                // ever not true then we have an accounting problem.
                let market_account = Self::market_account(market_id);
                ensure!(
                    T::Currency::free_balance(&market_account) >= winning_balance,
                    Error::<T>::InsufficientFundsInMarketAccount,
                );

                // Destory the shares.
                T::Shares::destroy(winning_shares_id, &sender, winning_balance)?;

                // Pay out the winner. One full unit of currency per winning share.
                T::Currency::transfer(&market_account, &sender, winning_balance, ExistenceRequirement::AllowDeath)?;
            } else {
                Err(Error::<T>::MarketDoesNotExist)?;
            }
        }

    }
}

impl<T: Trait> Module<T> {

    pub fn market_account(market_id: T::MarketId) -> T::AccountId {
        T::ModuleId::get().into_sub_account(market_id)
    }

    pub fn market_outcome_share_id(market_id: T::MarketId, outcome: u16) -> T::Hash {
        ("zge/pm", market_id, outcome).using_encoded(<T as frame_system::Trait>::Hashing::hash)
    }

    fn is_market_active(end: u64) -> bool {
        if end > 1_000_000_000 {
            // unix timestamp
            let now = <pallet_timestamp::Module<T>>::get().saturated_into::<u64>();
            return now < end;
        } else {
            // block number
            let current_block = <frame_system::Module<T>>::block_number();
            return current_block < end.saturated_into();
        }
    }

    /// DANGEROUS - MUTATES PALLET STORAGE
    ///
    fn get_next_market_id() -> Result<T::MarketId, dispatch::DispatchError> {
        let next = Self::market_count();
        let inc = next
            .checked_add(&One::one())
            .ok_or("Overflow when incrementing market count.")?;
        <MarketCount<T>>::put(inc);
        Ok(next)
    }

    fn do_buy_complete_set(who: T::AccountId, market_id: T::MarketId, amount: BalanceOf<T>) -> DispatchResult {
        ensure!(
            T::Currency::free_balance(&who) >= amount.into(), 
            Error::<T>::NotEnoughBalance,
        );

        if let Some(market) = Self::markets(market_id.clone()) {
            ensure!(Self::is_market_active(market.end), Error::<T>::MarketNotActive);

            let market_account = Self::market_account(market_id.clone());
            T::Currency::transfer(&who, &market_account, amount, ExistenceRequirement::KeepAlive)?;

            for i in 0..=market.outcomes() {
                let share_id = Self::market_outcome_share_id(market_id.clone(), i);

                T::Shares::generate(share_id, &who, amount)?;
            }

            Self::deposit_event(RawEvent::BoughtCompleteSet(market_id, who));

            Ok(())
        } else {
            Err(Error::<T>::MarketDoesNotExist)?
        }
    }

    fn internal_resolve(market_id: &T::MarketId) {
        let market = Self::markets(market_id).unwrap();
        let reporter = market.reporter.clone().unwrap();
        let reported_outcome = market.reported_outcome.unwrap();

        // if the market was permissionless and not invalid, return `ValidityBond`.
        if market.creation == MarketCreation::Permissionless {
            if reported_outcome != 0 {
                T::Currency::unreserve(&market.creator, T::ValidityBond::get());
            } else {
                // Give it to the treasury instead.
                let (imbalance, _) = T::Currency::slash_reserved(&market.creator, T::ValidityBond::get());
                T::Slash::on_unbalanced(imbalance);
            }
        }

        match market.status {
            MarketStatus::Reported => {
                // the oracle bond gets returned if the reporter was the oracle
                if reporter == market.oracle {
                    T::Currency::unreserve(&market.creator, T::OracleBond::get());
                } else {
                    let (imbalance, _) = T::Currency::slash_reserved(&market.creator, T::OracleBond::get());

                    // give it to the real reporter
                    T::Currency::resolve_creating(&reporter, imbalance);
                }
            }
            MarketStatus::Disputed => {
                let disputes = Self::disputes(market_id.clone());
                let num_disputes= disputes.len() as u16;
                // count the last dispute's outcome as the winning one
                let last_dispute = disputes[(num_disputes as usize) - 1].clone();
                let last_outcome = last_dispute.outcome;
                let mut correct_reporters: Vec<T::AccountId> = Vec::new();

                let mut overall_imbalance = NegativeImbalanceOf::<T>::zero();

                // if the reporter reported right, return the OracleBond, otherwise
                // slash it to pay the correct reporters
                if reported_outcome == last_outcome {
                    T::Currency::unreserve(&market.creator, T::OracleBond::get());
                } else {
                    let (imbalance, _) = T::Currency::slash_reserved(&market.creator, T::OracleBond::get());

                    overall_imbalance.subsume(imbalance);
                }

                for i in 0..num_disputes {
                    let dispute = &disputes[i as usize];
                    let dispute_bond = T::DisputeBond::get() + T::DisputeFactor::get() * i.into();
                    if dispute.outcome == last_outcome {
                        T::Currency::unreserve(&dispute.by, dispute_bond);

                        correct_reporters.push(dispute.by.clone());
                    } else {
                        let (imbalance, _) = T::Currency::slash_reserved(&dispute.by, dispute_bond);
                        overall_imbalance.subsume(imbalance);
                    }
                }

                // fold all the imbalances into one and reward the correct reporters.
                let reward_per_each = overall_imbalance.peek() / (correct_reporters.len() as u32).into();
                for i in 0..correct_reporters.len() {
                    let (amount, leftover) = overall_imbalance.split(reward_per_each);
                    T::Currency::resolve_creating(&correct_reporters[i], amount);
                    overall_imbalance = leftover;
                }
            }
            _ => panic!("Should never happen"), //TODO remove after testing
        };

        for i in 0..=market.outcomes() {
            // don't delete the winning outcome...
            if i == reported_outcome { continue; }
            // ... but delete the rest
            let share_id = Self::market_outcome_share_id(market_id.clone(), i);
            T::Shares::destroy_all(share_id).unwrap();
        }  

        <Markets<T>>::mutate(&market_id, |m| {
            m.as_mut().unwrap().status = MarketStatus::Resolved;
        });
    }
}
