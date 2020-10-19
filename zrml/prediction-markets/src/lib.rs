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
//! - `create` - Creates a market which then can have its shares be bought or sold.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, 
    ensure, Parameter,
};
use frame_support::traits::{
    Currency, ReservableCurrency, Get, ExistenceRequirement,
    EnsureOrigin,
};
use frame_support::weights::Weight;
use frame_system::{self as system, ensure_signed};
use sp_runtime::ModuleId;
use sp_runtime::traits::{
    AccountIdConversion, AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, 
    Member, One, Hash,
};
use sp_std::vec::Vec;
use zrml_traits::shares::Shares;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod market;

use market::{Market, MarketCreation, MarketStatus, MarketType};

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type Currency: ReservableCurrency<Self::AccountId>;

    type Shares: Shares<Self::AccountId, BalanceOf<Self>, Self::Hash>;

    /// The identifier of individual markets.
    type MarketId: AtLeast32Bit + Parameter + Member + MaybeSerializeDeserialize + Default;

    /// The module identifier.
    type ModuleId: Get<ModuleId>;

    /// The number of blocks the reporting period remains open.
    type ReportingPeriod: Get<Self::BlockNumber>;
    /// The number of blocks the dispute period remains open.
    type DisputePeriod: Get<Self::BlockNumber>;
    /// The base amount of currency that must be bonded in order to create a dispute.
    type DisputeBond: Get<BalanceOf<Self>>;
    /// The base amount of currency that must be bonded for a permissionless market,
    /// guaranteeing that it will resolve as anything but `Invalid`.
    type ValidityBond: Get<BalanceOf<Self>>;
    /// The base amount of currency that must be bonded for a market approved by the
    ///  advisory committee.
    type AdvisoryBond: Get<BalanceOf<Self>>;
    /// The base amount of currency that must be bonded to ensure the oracle reports
    ///  in a timely manner.
    type OracleBond: Get<BalanceOf<Self>>;

    type ApprovalOrigin: EnsureOrigin<<Self as system::Trait>::Origin>;
}

decl_storage! {
    trait Store for Module<T: Trait> as PredictionMarkets {
        Markets get(fn markets):
            map hasher(blake2_128_concat) T::MarketId => 
                Option<Market<T::AccountId, T::BlockNumber>>;

        MarketCount get(fn market_count): T::MarketId;

        MarketIdsPerEndBlock get(fn market_ids_per_end_block):
            map hasher(blake2_128_concat) T::BlockNumber => Vec<T::MarketId>;
        
        MarketIdsPerReportBlock get(fn market_ids_per_report_block):
            map hasher(blake2_128_concat) T::BlockNumber => Vec<T::MarketId>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
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
        /// A market has been reported on [market_id, winning_outcome]
        MarketReported(MarketId, u16),
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
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
 
        const AdvisoryBond: BalanceOf<T> = T::AdvisoryBond::get();
        const DisputeBond: BalanceOf<T> = T::DisputeBond::get();
        const OracleBond: BalanceOf<T> = T::OracleBond::get();
        // TODO: Rename validity bond?
        const ValidityBond: BalanceOf<T> = T::ValidityBond::get();

        type Error = Error<T>;

        fn deposit_event() = default;

        /// The initializer will automatically close any markets that are
        /// slated to be closed at the beginning of the block.
        ///
        fn on_initialize(now: T::BlockNumber) -> Weight {
            let market_ids = Self::market_ids_per_end_block(now);
            if !market_ids.is_empty() {
                market_ids.iter().for_each(|id| {
                    <Markets<T>>::mutate(id, |m| {
                        m.as_mut().unwrap().status = MarketStatus::Closed;
                    });
                });
            }

            0
        }

        /// The finalize function will move all reported markets to finalized.
        ///
        fn on_finalize(now: T::BlockNumber) {
            let reporting_period = T::ReportingPeriod::get();
            if now <= reporting_period { return; }

            let market_ids = Self::market_ids_per_end_block(now - T::ReportingPeriod::get());
            if !market_ids.is_empty() {
                market_ids.iter().for_each(|id| {
                    let market = Self::markets(id).unwrap();
                    if market.status == MarketStatus::Reported {
                        <Markets<T>>::mutate(id, |m| {
                            m.as_mut().unwrap().status = MarketStatus::Resolved;
                        });

                        for i in 0..market.outcomes {
                            // skip deleting the winning outcome
                            if i == market.winning_outcome.unwrap() { continue; }
                            // ...but delete all others
                            let share_id = Self::market_outcome_share_id(id.clone(), i);
                            T::Shares::destroy_all(share_id);
                        }
                    } else if market.status == MarketStatus::Closed {
                        // TODO: determine what to do with markets that were not reported on
                        // they should move into an overdue queue of some type
                        // slash the reserved amount for the oracle not reporting
                        let (neg_imbal, _) = T::Currency::slash_reserved(&market.creator, T::OracleBond::get());

                    }
                })
            }
        }

        /// Creates a new prediction market, seeded with the intial values.
        ///
        #[weight = 0]
        pub fn create(
            origin,
            oracle: T::AccountId,
            market_type: MarketType,
            outcomes: u16,
            end_block: T::BlockNumber,
            metadata: Vec<u8>,
            creation: MarketCreation,
        ) {
            let sender = ensure_signed(origin)?;

            // PoC - Only binary markets are currently supported.
            ensure!(market_type == MarketType::Binary, "Only binary markets are currently supported.");
            ensure!(outcomes == 3, "Only binary markets are currently supported.");

            // Check the end_block is in the future.
            let current_block = <frame_system::Module<T>>::block_number();
            ensure!(current_block < end_block, "End block must be in the future.");

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
                creator_fee: 0,
                oracle,
                outcomes,
                end_block,
                metadata,
                market_type,
                status,
                winning_outcome: None,
            };

            <Markets<T>>::insert(new_market_id.clone(), new_market);
            <MarketIdsPerEndBlock<T>>::mutate(end_block, |v| v.push(new_market_id.clone()));

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

        #[weight = 0]
        pub fn reject_market(origin, market_id: T::MarketId) {
            T::ApprovalOrigin::ensure_origin(origin)?;

            if let Some(market) = Self::markets(&market_id) {
                let creator = market.creator;
                let (_imbalance, _) =  T::Currency::slash_reserved(&creator, T::AdvisoryBond::get());
                // TODO: Handle the imbalance by moving it to the treasury.
                <Markets<T>>::remove(&market_id);
                Self::deposit_event(RawEvent::MarketRejected(market_id));
            } else {
                Err(Error::<T>::MarketDoesNotExist)?;
            }
        }

        /// NOTE: Only for PoC probably - should only allow rejections
        /// in a production environment since this better aligns incentives.
        /// See also: Polkadot Treasury
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

            ensure!(
                T::Currency::free_balance(&sender) >= amount.into(),
                Error::<T>::NotEnoughBalance,
            );

            if let Some(market) = Self::markets(market_id.clone()) {
                ensure!(market.status == MarketStatus::Active, Error::<T>::MarketNotActive);

                let market_account = Self::market_account(market_id.clone());
                T::Currency::transfer(&sender, &market_account, amount, ExistenceRequirement::KeepAlive)?;

                let outcomes = market.outcomes;
                for i in 0..outcomes {
                    let share_id = Self::market_outcome_share_id(market_id.clone(), i);

                    T::Shares::generate(share_id, &sender, amount)?;
                }

                Self::deposit_event(RawEvent::BoughtCompleteSet(market_id, sender));
            } else {
                Err(Error::<T>::MarketDoesNotExist)?;
            }
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
                ensure!(
                    market.status == MarketStatus::Active,
                    Error::<T>::MarketNotActive,
                );

                let market_account = Self::market_account(market_id.clone());
                ensure!(
                    T::Currency::free_balance(&market_account) >= amount,
                    "Market account does not have sufficient reserves.",
                );

                let outcomes = market.outcomes;
                for i in 0..outcomes {
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
                for i in 0..outcomes {
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
        /// NOTE: Only callable by the designated oracle of a market.
        ///
        #[weight = 0]
        pub fn report(origin, market_id: T::MarketId, winning_outcome: u16) {
            let sender = ensure_signed(origin)?;

            if let Some(mut market) = Self::markets(market_id.clone()) {
                let oracle = market.oracle.clone();
                ensure!(sender == oracle, Error::<T>::ReporterNotOracle);
                
                // Make sure the market is closed and in reporting period.
                ensure!(market.status == MarketStatus::Closed, Error::<T>::MarketNotClosed);

                market.winning_outcome = Some(winning_outcome);
                market.status = MarketStatus::Reported;
                <Markets<T>>::insert(market_id.clone(), market);

                Self::deposit_event(RawEvent::MarketReported(market_id, winning_outcome));
            } else {
                Err(Error::<T>::MarketDoesNotExist)?;
            }
        }

        #[weight = 0]
        pub fn report_overdue(origin, market_id: T::MarketId, winning_outcome: u16) {

        }

        /// Disputes a reported outcome.
        ///
        /// NOTE: Requires a `DisputeBond` amount of currency to be locked.
        ///
        #[weight = 0]
        pub fn dispute(origin, market_id: T::MarketId) {
            let _sender = ensure_signed(origin)?;
            if let Some(_market) = Self::markets(market_id) {
                // TODO
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
                let winning_outcome = market.winning_outcome.unwrap();
                let winning_shares_id = Self::market_outcome_share_id(market_id.clone(), winning_outcome);
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
        (market_id, outcome).using_encoded(<T as system::Trait>::Hashing::hash)
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
}
