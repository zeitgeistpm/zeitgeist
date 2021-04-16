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

extern crate alloc;

mod errors;
mod market;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use pallet::{Config, Error, Event, Pallet};

#[frame_support::pallet]
mod pallet {
    use crate::{
        errors::*,
        market::{
            Market, MarketCreation, MarketDispute, MarketEnd, MarketStatus, MarketType, Outcome, Report,
        },
    };
    use alloc::vec::Vec;
    use core::{cmp, marker::PhantomData};
    use frame_support::{
        dispatch, ensure,
        pallet_prelude::{StorageMap, StorageValue, ValueQuery},
        traits::{
            Currency, EnsureOrigin, ExistenceRequirement, Get, Hooks, Imbalance, IsType,
            OnUnbalanced, ReservableCurrency,
        },
        Blake2_128Concat, Parameter,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use orml_traits::MultiCurrency;
    use sp_runtime::{
        traits::{
            AccountIdConversion, AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, Member, One,
            Zero,
        },
        DispatchResult, ModuleId, SaturatedConversion,
    };
    use zeitgeist_primitives::{Asset, ScalarPosition, Swaps, ZeitgeistMultiReservableCurrency};

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Allows the `ApprovalOrigin` to immediately destroy a market.
        ///
        /// todo: this should check if there's any outstanding funds reserved if it stays
        /// in for production
        #[pallet::weight(10_000)]
        pub fn admin_destroy_market(
            origin: OriginFor<T>,
            market_id: T::MarketId,
        ) -> DispatchResult {
            T::ApprovalOrigin::ensure_origin(origin)?;

            let market = Self::market_by_id(&market_id)?;

            Self::clear_auto_resolve(&market_id)?;

            <Markets<T>>::remove(&market_id);

            // Delete of this market's outcome assets.
            for asset in Self::outcome_assets(market_id, market).iter() {
                let accounts = T::Shares::accounts_by_currency_id(*asset);
                T::Shares::destroy_all(*asset, accounts.iter().cloned());
            };

            Ok(())
        }

        /// Allows the `ApprovalOrigin` to immediately move an open market to closed.
        ///
        #[pallet::weight(10_000)]
        pub fn admin_move_market_to_closed(
            origin: OriginFor<T>,
            market_id: T::MarketId,
        ) -> DispatchResult {
            T::ApprovalOrigin::ensure_origin(origin)?;

            let market = Self::market_by_id(&market_id)?;
            let new_end = match market.end {
                MarketEnd::Block(_) => {
                    let current_block = <frame_system::Pallet<T>>::block_number();
                    MarketEnd::Block(current_block)
                }
                MarketEnd::Timestamp(_) => {
                    let now = <pallet_timestamp::Pallet<T>>::get().saturated_into::<u64>();
                    MarketEnd::Timestamp(now)
                }
            };

            <Markets<T>>::mutate(&market_id, |m| {
                m.as_mut().unwrap().end = new_end;
            });
            Ok(())
        }

        /// Allows the `ApprovalOrigin` to immediately move a reported or disputed
        /// market to resolved.
        ////
        #[pallet::weight(10_000)]
        pub fn admin_move_market_to_resolved(
            origin: OriginFor<T>,
            market_id: T::MarketId,
        ) -> DispatchResult {
            T::ApprovalOrigin::ensure_origin(origin)?;

            let market = Self::market_by_id(&market_id)?;
            ensure!(
                market.status == MarketStatus::Reported || market.status == MarketStatus::Disputed,
                "not reported nor disputed"
            );
            Self::clear_auto_resolve(&market_id)?;

            Self::internal_resolve(&market_id)?;
            Ok(())
        }

        /// Approves a market that is waiting for approval from the
        /// advisory committee.
        ///
        /// NOTE: Returns the proposer's bond since the market has been
        /// deemed valid by an advisory committee.
        ///
        /// NOTE: Can only be called by the `ApprovalOrigin`.
        ///
        #[pallet::weight(10_000)]
        pub fn approve_market(origin: OriginFor<T>, market_id: T::MarketId) -> DispatchResult {
            T::ApprovalOrigin::ensure_origin(origin)?;

            let market = Self::market_by_id(&market_id)?;

            let creator = market.creator;

            T::Currency::unreserve(&creator, T::AdvisoryBond::get());
            <Markets<T>>::mutate(&market_id, |m| {
                m.as_mut().unwrap().status = MarketStatus::Active;
            });

            Self::deposit_event(Event::MarketApproved(market_id));
            Ok(())
        }

        /// Generates a complete set of outcome shares for a market.
        ///
        /// NOTE: This is the only way to create new shares.
        ///
        #[pallet::weight(10_000)]
        pub fn buy_complete_set(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            Self::do_buy_complete_set(sender, market_id, amount)?;
            Ok(())
        }

        /// NOTE: Only for PoC probably - should only allow rejections
        /// in a production environment since this better aligns incentives.
        /// See also: Polkadot Treasury
        ///
        #[pallet::weight(10_000)]
        pub fn cancel_pending_market(
            origin: OriginFor<T>,
            market_id: T::MarketId,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let market = Self::market_by_id(&market_id)?;

            let creator = market.creator;
            let status = market.status;
            ensure!(creator == sender, "Canceller must be market creator.");
            ensure!(
                status == MarketStatus::Proposed,
                "Market must be pending approval."
            );
            // The market is being cancelled, return the deposit.
            T::Currency::unreserve(&creator, T::AdvisoryBond::get());
            <Markets<T>>::remove(&market_id);
            Self::deposit_event(Event::MarketCancelled(market_id));
            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn create_categorical_market(
            origin: OriginFor<T>,
            oracle: T::AccountId,
            end: MarketEnd<T::BlockNumber>,
            metadata: Vec<u8>,
            creation: MarketCreation,
            categories: u16,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            Self::ensure_create_market_end(end)?;

            ensure!(
                categories <= T::MaxCategories::get(),
                "Cannot exceed max categories for a new market."
            );

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

            let market_id = Self::get_next_market_id()?;
            let market = Market {
                creator: sender.clone(),
                creation,
                creator_fee: 0,
                oracle,
                end,
                metadata,
                market_type: MarketType::Categorical(categories),
                status,
                report: None,
                resolved_outcome: None,
            };

            <Markets<T>>::insert(market_id.clone(), Some(market));

            Self::deposit_event(Event::MarketCreated(market_id, sender));

            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn create_scalar_market(
            origin: OriginFor<T>,
            oracle: T::AccountId,
            end: MarketEnd<T::BlockNumber>,
            metadata: Vec<u8>,
            creation: MarketCreation,
            outcome_range: (u128, u128)
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            Self::ensure_create_market_end(end)?;

            ensure!(outcome_range.0 < outcome_range.1, "Invalid range provided.");

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

            let market_id = Self::get_next_market_id()?;
            let market = Market {
                creator: sender.clone(),
                creation,
                creator_fee: 0,
                oracle,
                end,
                metadata,
                market_type: MarketType::Scalar(outcome_range),
                status,
                report: None,
                resolved_outcome: None,
            };

            <Markets<T>>::insert(market_id.clone(), Some(market));

            Self::deposit_event(Event::MarketCreated(market_id, sender));

            Ok(())
        }

        /// Deploys a new pool for the market. This pallet keeps track of a single
        /// canonical swap pool for each market in `market_to_swap_pool`.
        ///
        /// The sender should have enough funds to cover all of the required
        /// shares to seed the pool.
        #[pallet::weight(10_000)]
        pub fn deploy_swap_pool_for_market(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            weights: Vec<u128>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let market = Self::market_by_id(&market_id)?;
            // ensure the market is active
            let status = market.status;
            ensure!(status == MarketStatus::Active, Error::<T>::MarketNotActive);

            // ensure a swap pool does not already exist
            ensure!(
                Self::market_to_swap_pool(&market_id).is_none(),
                Error::<T>::SwapPoolExists
            );

            let mut assets = Self::outcome_assets(market_id, market);
            assets.push(Asset::Ztg);

            let pool_id = T::Swap::create_pool(sender, assets, Zero::zero(), weights)?;

            <MarketToSwapPool<T>>::insert(market_id, Some(pool_id));
            Ok(())
        }

        /// Disputes a reported outcome.
        ///
        /// NOTE: Requires a `DisputeBond` + `DisputeFactor` * `num_disputes` amount of currency
        ///  to be reserved.
        ///
        #[pallet::weight(10_000)]
        pub fn dispute(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            outcome: Outcome,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let market = Self::market_by_id(&market_id)?;

            ensure!(market.report.is_some(), Error::<T>::MarketNotReported);

            if let Outcome::Categorical(inner) = outcome {
                if let MarketType::Categorical(categories) = market.market_type {
                    ensure!(inner < categories, Error::<T>::OutcomeOutOfRange);
                } else {
                    return Err(OUTCOME_MISMATCH);
                }
            }
            if let Outcome::Scalar(inner) = outcome {
                if let MarketType::Scalar(outcome_range) = market.market_type {
                    ensure!(inner >= outcome_range.0 && inner <= outcome_range.1, "some");
                } else {
                    return Err(OUTCOME_MISMATCH);
                }
            }

            let disputes = Self::disputes(market_id.clone());
            let num_disputes = disputes.len() as u16;
            ensure!(
                num_disputes < T::MaxDisputes::get(),
                Error::<T>::MaxDisputesReached
            );

            if num_disputes > 0 {
                ensure!(
                    disputes[(num_disputes as usize) - 1].outcome != outcome,
                    Error::<T>::CannotDisputeSameOutcome
                );
            }

            let dispute_bond =
                T::DisputeBond::get() + T::DisputeFactor::get() * num_disputes.into();
            T::Currency::reserve(&sender, dispute_bond)?;

            let current_block = <frame_system::Pallet<T>>::block_number();

            if num_disputes > 0 {
                let prev_dispute = disputes[(num_disputes as usize) - 1].clone();
                let at = prev_dispute.at;
                let mut old_disputes_per_block = Self::market_ids_per_dispute_block(at);
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
                    outcome: outcome.clone(),
                })
            });

            // if not already in dispute
            if market.status != MarketStatus::Disputed {
                <Markets<T>>::mutate(market_id.clone(), |m| {
                    m.as_mut().unwrap().status = MarketStatus::Disputed;
                });
            }

            Self::deposit_event(Event::MarketDisputed(market_id, outcome));
            Ok(())
        }

        /// Starts a global dispute.
        ///
        /// NOTE: Requires the market to be already disputed `MaxDisputes` amount of times.
        ///
        #[pallet::weight(10_000)]
        pub fn global_dispute(origin: OriginFor<T>, market_id: T::MarketId) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            let _market = Self::market_by_id(&market_id)?;
            // TODO: implement global disputes
            Ok(())
        }

        /// Redeems the winning shares of a prediction market.
        ///
        #[pallet::weight(10_000)]
        pub fn redeem_shares(origin: OriginFor<T>, market_id: T::MarketId) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let market = Self::market_by_id(&market_id)?;
            let market_account = Self::market_account(market_id);

            ensure!(
                market.status == MarketStatus::Resolved,
                Error::<T>::MarketNotResolved,
            );

            // Check to see if the sender has any winning shares.
            let resolved_outcome = market.resolved_outcome.ok_or_else(|| NOT_RESOLVED)?;

            let winning_assets = match resolved_outcome {
                Outcome::Categorical(category_index) => {
                    let winning_currency_id = Asset::CategoricalOutcome(market_id, category_index);
                    let winning_balance = T::Shares::free_balance(winning_currency_id, &sender);
                    ensure!(winning_balance >= BalanceOf::<T>::zero(), Error::<T>::NoWinningBalance);

                    // Ensure the market account has enough to pay out - if this is
                    // ever not true then we have an accounting problem.
                    ensure!(
                        T::Currency::free_balance(&market_account) >= winning_balance,
                        Error::<T>::InsufficientFundsInMarketAccount,
                    );

                    vec![(winning_currency_id, winning_balance)]
                }
                Outcome::Scalar(value) => {
                    let long_currency_id = Asset::ScalarOutcome(market_id, ScalarPosition::Long);
                    let short_currency_id = Asset::ScalarOutcome(market_id, ScalarPosition::Short);
                    let long_balance = T::Shares::free_balance(long_currency_id, &sender);
                    let short_balance = T::Shares::free_balance(short_currency_id, &sender);
                    let zero = BalanceOf::<T>::zero();
                    let one = BalanceOf::<T>::one();

                    ensure!(long_balance >= zero || short_balance >= zero, Error::<T>::NoWinningBalance);

                    if let MarketType::Scalar((bound_low, bound_high)) = market.market_type {
                        let calc_payouts = |final_value, low, high| -> (BalanceOf<T>, BalanceOf<T>) {
                            if final_value < low {
                                return (zero, one);
                            }
                            if final_value > high {
                                return (one, zero);
                            }

                            let payout_long: u128 = (final_value - low) / (high - low);
                            // let payout_long = one;
                            (payout_long.saturated_into(), one - payout_long.saturated_into()) 
                        };

                        let (long_payout, short_payout) = calc_payouts(value, bound_low, bound_high);

                        // Ensure the market account has enough to pay out - if this is
                        // ever not true then we have an accounting problem.
                        ensure!(
                            T::Currency::free_balance(&market_account) >= long_payout + short_payout,
                            Error::<T>::InsufficientFundsInMarketAccount,
                        );

                        vec![(long_currency_id, long_payout), (short_currency_id, short_payout)]
                    } else {
                        panic!("should never happen");
                    }
                }
            };

            for (currency_id, amount) in winning_assets {
                // Destory the shares.
                T::Shares::slash(currency_id, &sender, amount);

                // Pay out the winner. One full unit of currency per winning share.
                T::Currency::transfer(
                    &market_account,
                    &sender,
                    amount,
                    ExistenceRequirement::AllowDeath,
                )?;
            }

            Ok(())
        }

        /// Rejects a market that is waiting for approval from the advisory
        /// committee.
        ///
        /// NOTE: Will slash the reserved `AdvisoryBond` from the market creator.
        ///
        #[pallet::weight(10_000)]
        pub fn reject_market(origin: OriginFor<T>, market_id: T::MarketId) -> DispatchResult {
            T::ApprovalOrigin::ensure_origin(origin)?;

            let market = Self::market_by_id(&market_id)?;
            let creator = market.creator;
            let (imbalance, _) = T::Currency::slash_reserved(&creator, T::AdvisoryBond::get());
            // Slashes the imbalance.
            T::Slash::on_unbalanced(imbalance);
            <Markets<T>>::remove(&market_id);
            Self::deposit_event(Event::MarketRejected(market_id));
            Ok(())
        }

        /// Reports the outcome of a market.
        ///
        #[pallet::weight(10_000)]
        pub fn report(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            outcome: Outcome,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let mut market = Self::market_by_id(&market_id)?;

            // TODO make this a conditional check
            // ensure!(outcome <= market.outcomes(), Error::<T>::OutcomeOutOfRange);
            ensure!(market.report.is_none(), Error::<T>::MarketAlreadyReported);

            // ensure market is not active
            ensure!(
                !Self::is_market_active(market.end),
                Error::<T>::MarketNotClosed
            );

            let current_block = <frame_system::Pallet<T>>::block_number();

            match market.end {
                MarketEnd::Block(block) => {
                    // blocks
                    if current_block <= block + T::ReportingPeriod::get() {
                        ensure!(sender == market.oracle, Error::<T>::ReporterNotOracle);
                    } // otherwise anyone can be the reporter
                }
                MarketEnd::Timestamp(timestamp) => {
                    // unix timestamp
                    let now = <pallet_timestamp::Pallet<T>>::get().saturated_into::<u64>();
                    let reporting_period_in_ms =
                        T::ReportingPeriod::get().saturated_into::<u64>() * 6000;
                    if now <= timestamp + reporting_period_in_ms {
                        ensure!(sender == market.oracle, Error::<T>::ReporterNotOracle);
                    } // otherwise anyone can be the reporter
                }
            }

            market.report = Some(Report {
                at: current_block,
                by: sender.clone(),
                outcome: outcome.clone(),
            });
            market.status = MarketStatus::Reported;
            <Markets<T>>::insert(market_id.clone(), Some(market));

            <MarketIdsPerReportBlock<T>>::mutate(current_block, |v| {
                v.push(market_id.clone());
            });

            Self::deposit_event(Event::MarketReported(market_id, outcome));
            Ok(())
        }

        /// Destroys a complete set of outcomes shares for a market.
        ///
        #[pallet::weight(10_000)]
        pub fn sell_complete_set(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let market = Self::market_by_id(&market_id)?;
            ensure!(
                Self::is_market_active(market.end),
                Error::<T>::MarketNotActive
            );

            let market_account = Self::market_account(market_id.clone());
            ensure!(
                T::Currency::free_balance(&market_account) >= amount,
                "Market account does not have sufficient reserves.",
            );

            for asset in Self::outcome_assets(market_id, market).iter() {
                // Ensures that the sender has sufficient amount of each
                // share in the set.
                ensure!(
                    T::Shares::free_balance(*asset, &sender) >= amount,
                    Error::<T>::InsufficientShareBalance,
                );

                T::Shares::slash(*asset, &sender, amount);
            }

            T::Currency::transfer(
                &market_account,
                &sender,
                amount,
                ExistenceRequirement::AllowDeath,
            )?;

            Self::deposit_event(Event::SoldCompleteSet(market_id, sender));

            Ok(())
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// The base amount of currency that must be bonded for a market approved by the
        ///  advisory committee.
        type AdvisoryBond: Get<BalanceOf<Self>>;

        type ApprovalOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

        type Currency: ReservableCurrency<Self::AccountId>;

        /// The base amount of currency that must be bonded in order to create a dispute.
        type DisputeBond: Get<BalanceOf<Self>>;

        /// The additional amount of currency that must be bonded when creating a subsequent
        ///  dispute.
        type DisputeFactor: Get<BalanceOf<Self>>;

        /// The number of blocks the dispute period remains open.
        type DisputePeriod: Get<Self::BlockNumber>;

        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type Shares: ZeitgeistMultiReservableCurrency<
            Self::AccountId,
            Balance = BalanceOf<Self>,
            CurrencyId = Asset<Self::MarketId>,
        >;

        /// The identifier of individual markets.
        type MarketId: AtLeast32Bit
            + Copy
            + Default
            + MaybeSerializeDeserialize
            + Member
            + Parameter;

        /// The maximum number of categories available for categorical markets.
        type MaxCategories: Get<u16>;

        /// The maximum number of disputes allowed on any single market.
        type MaxDisputes: Get<u16>;

        /// The module identifier.
        type ModuleId: Get<ModuleId>;

        /// The base amount of currency that must be bonded to ensure the oracle reports
        ///  in a timely manner.
        type OracleBond: Get<BalanceOf<Self>>;

        /// The number of blocks the reporting period remains open.
        type ReportingPeriod: Get<Self::BlockNumber>;

        type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

        type Swap: Swaps<
            Self::AccountId,
            Balance = BalanceOf<Self>,
            Hash = Self::Hash,
            MarketId = Self::MarketId,
        >;

        /// The base amount of currency that must be bonded for a permissionless market,
        /// guaranteeing that it will resolve as anything but `Invalid`.
        type ValidityBond: Get<BalanceOf<Self>>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Someone is trying to call `dispute` with the same outcome that is currently
        ///  registered on-chain.
        CannotDisputeSameOutcome,
        /// The sender's balance is too low to take this order.
        CurrencyBalanceTooLow,
        /// End block is too soon.
        EndBlockTooSoon,
        /// End timestamp is too soon.
        EndTimestampTooSoon,
        /// Market account does not have enough funds to pay out.
        InsufficientFundsInMarketAccount,
        /// Sender does not have enough share balance.
        InsufficientShareBalance,
        /// A market with the provided ID does not exist.
        MarketDoesNotExist,
        /// The market status is something other than active.
        MarketNotActive,
        /// Sender does not have enough balance to buy shares.
        NotEnoughBalance,
        /// The order has already been taken.
        OrderAlreadyTaken,
        /// The order hash was not found in the pallet.
        OrderDoesNotExist,
        /// The outcome being reported is out of range.
        OutcomeOutOfRange,
        /// Market is already reported on.
        MarketAlreadyReported,
        /// The market is not closed.
        MarketNotClosed,
        /// The market is not overdue.
        MarketNotOverdue,
        /// The market is not reported on.
        MarketNotReported,
        /// The market is not resolved.
        MarketNotResolved,
        /// The maximum number of disputes has been reached.
        MaxDisputesReached,
        /// The user has no winning balance.
        NoWinningBalance,
        /// The report is not coming from designated oracle.
        ReporterNotOracle,
        /// The user has a share balance that is too low.
        ShareBalanceTooLow,
        /// A swap pool already exists for this market.
        SwapPoolExists,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A complete set of shares has been bought [market_id, buyer]
        BoughtCompleteSet(
            <T as Config>::MarketId,
            <T as frame_system::Config>::AccountId,
        ),
        /// A market has been approved [market_id]
        MarketApproved(<T as Config>::MarketId),
        /// A market has been created [market_id, creator]
        MarketCreated(
            <T as Config>::MarketId,
            <T as frame_system::Config>::AccountId,
        ),
        /// A pending market has been cancelled. [market_id, creator]
        MarketCancelled(<T as Config>::MarketId),
        /// A market has been disputed [market_id, new_outcome]
        MarketDisputed(<T as Config>::MarketId, Outcome),
        /// NOTE: Maybe we should only allow rejections.
        /// A pending market has been rejected as invalid. [market_id]
        MarketRejected(<T as Config>::MarketId),
        /// A market has been reported on [market_id, reported_outcome]
        MarketReported(<T as Config>::MarketId, Outcome),
        /// A market has been resolved [market_id, real_outcome]
        MarketResolved(<T as Config>::MarketId, u16),
        /// A complete set of shares has been sold [market_id, seller]
        SoldCompleteSet(
            <T as Config>::MarketId,
            <T as frame_system::Config>::AccountId,
        ),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        /// The finalize function will move all reported markets to resolved.
        ///
        /// Disputed markets need to be resolved manually.
        fn on_finalize(now: T::BlockNumber) {
            let dispute_period = T::DisputePeriod::get();
            if now <= dispute_period {
                return;
            }

            // Resolve all regularly reported markets.
            let market_ids = Self::market_ids_per_report_block(now - dispute_period);
            if !market_ids.is_empty() {
                market_ids.iter().for_each(|id| {
                    let market =
                        Self::markets(id).expect("Market stored in report block does not exist");
                    if market.status != MarketStatus::Reported {
                    } else {
                        Self::internal_resolve(id).expect("Internal respolve failed");
                    }
                });
            }

            // Resolve any disputed markets.
            let disputed = Self::market_ids_per_dispute_block(now - dispute_period);
            if !disputed.is_empty() {
                disputed.iter().for_each(|id| {
                    Self::internal_resolve(id).expect("Internal resolve failed");
                });
            }
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// For each market, this holds the dispute information for each dispute that's
    /// been issued.
    #[pallet::storage]
    #[pallet::getter(fn disputes)]
    pub type Disputes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::MarketId,
        Vec<MarketDispute<T::AccountId, T::BlockNumber>>,
        ValueQuery,
    >;

    /// For each market, this holds the dispute information for each dispute that's
    /// been issued.
    #[pallet::storage]
    #[pallet::getter(fn markets)]
    pub type Markets<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::MarketId,
        Option<Market<T::AccountId, T::BlockNumber>>,
        ValueQuery,
    >;

    /// A mapping of market identifiers to the block they were disputed at.
    /// A market only ends up here if it was disputed.
    #[pallet::storage]
    #[pallet::getter(fn market_ids_per_dispute_block)]
    pub type MarketIdsPerDisputeBlock<T: Config> =
        StorageMap<_, Blake2_128Concat, T::BlockNumber, Vec<T::MarketId>, ValueQuery>;

    /// A mapping of market identifiers to the block that they were reported on.
    #[pallet::storage]
    #[pallet::getter(fn market_ids_per_report_block)]
    pub type MarketIdsPerReportBlock<T: Config> =
        StorageMap<_, Blake2_128Concat, T::BlockNumber, Vec<T::MarketId>, ValueQuery>;

    /// The number of markets that have been created and the next identifier
    /// for a created market.
    #[pallet::storage]
    #[pallet::getter(fn market_count)]
    pub type MarketCount<T: Config> = StorageValue<_, T::MarketId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn market_to_swap_pool)]
    pub type MarketToSwapPool<T: Config> =
        StorageMap<_, Blake2_128Concat, T::MarketId, Option<u128>, ValueQuery>;

    impl<T: Config> Pallet<T> {

        pub fn outcome_assets(
            market_id: T::MarketId,
            market: Market<T::AccountId, T::BlockNumber>,
        ) -> Vec<Asset<T::MarketId>> {
            match market.market_type {
                MarketType::Categorical(categories) => {
                    let mut assets = Vec::new();
                    for i in 0..categories {
                        assets.push(Asset::CategoricalOutcome(market_id, i));
                    }
                    assets
                }
                MarketType::Scalar(_) => {
                    vec![Asset::ScalarOutcome(market_id, ScalarPosition::Long), Asset::ScalarOutcome(market_id, ScalarPosition::Short)]
                }
            }
        }

        pub(crate) fn market_account(market_id: T::MarketId) -> T::AccountId {
            T::ModuleId::get().into_sub_account(market_id)
        }

        /// Clears this market from being stored for automatic resolution.
        fn clear_auto_resolve(market_id: &T::MarketId) -> Result<(), dispatch::DispatchError> {
            let market = Self::market_by_id(&market_id)?;
            if market.status == MarketStatus::Reported {
                let report = market.report.ok_or_else(|| NO_REPORT)?;
                let mut old_reports_per_block = Self::market_ids_per_report_block(report.at);
                remove_item::<T::MarketId>(&mut old_reports_per_block, market_id.clone());
                <MarketIdsPerReportBlock<T>>::insert(report.at, old_reports_per_block);
            }
            if market.status == MarketStatus::Disputed {
                let disputes = Self::disputes(market_id.clone());
                let num_disputes = disputes.len() as u16;
                let prev_dispute = disputes[(num_disputes as usize) - 1].clone();
                let at = prev_dispute.at;
                let mut old_disputes_per_block = Self::market_ids_per_dispute_block(at);
                remove_item::<T::MarketId>(&mut old_disputes_per_block, market_id.clone());
                <MarketIdsPerDisputeBlock<T>>::insert(at, old_disputes_per_block);
            }

            Ok(())
        }

        fn do_buy_complete_set(
            who: T::AccountId,
            market_id: T::MarketId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(
                T::Currency::free_balance(&who) >= amount.into(),
                Error::<T>::NotEnoughBalance,
            );

            let market = Self::market_by_id(&market_id)?;
            ensure!(
                Self::is_market_active(market.end),
                Error::<T>::MarketNotActive
            );

            let market_account = Self::market_account(market_id.clone());
            T::Currency::transfer(
                &who,
                &market_account,
                amount,
                ExistenceRequirement::KeepAlive,
            )?;

            for asset in Self::outcome_assets(market_id, market).iter() {
                T::Shares::deposit(*asset, &who, amount)?;
            };

            Self::deposit_event(Event::BoughtCompleteSet(market_id, who));

            Ok(())
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

        /// Performs the logic for resolving a market, including slashing and distributing
        /// funds.
        ///
        /// NOTE: This function does not perform any checks on the market that is being given.
        /// In the function calling this you should that the market is already in a reported or
        /// disputed state.
        ///
        fn internal_resolve(market_id: &T::MarketId) -> DispatchResult {
            let market = Self::market_by_id(market_id)?;
            let report = market.report.clone().ok_or_else(|| NO_REPORT)?;

            // if the market was permissionless and not invalid, return `ValidityBond`.
            // if market.creation == MarketCreation::Permissionless {
            //     if report.outcome != 0 {
            //         T::Currency::unreserve(&market.creator, T::ValidityBond::get());
            //     } else {
            //         // Give it to the treasury instead.
            //         let (imbalance, _) =
            //             T::Currency::slash_reserved(&market.creator, T::ValidityBond::get());
            //         T::Slash::on_unbalanced(imbalance);
            //     }
            // }
            T::Currency::unreserve(&market.creator, T::ValidityBond::get());

            let resolved_outcome = match market.status {
                MarketStatus::Reported => report.clone().outcome,
                MarketStatus::Disputed => {
                    let disputes = Self::disputes(market_id.clone());
                    let num_disputes = disputes.len() as u16;
                    // count the last dispute's outcome as the winning one
                    let last_dispute = disputes[(num_disputes as usize) - 1].clone();
                    last_dispute.outcome
                }
                _ => panic!("Cannot happen"),
            };

            match market.status {
                MarketStatus::Reported => {
                    // the oracle bond gets returned if the reporter was the oracle
                    if report.by == market.oracle {
                        T::Currency::unreserve(&market.creator, T::OracleBond::get());
                    } else {
                        let (imbalance, _) =
                            T::Currency::slash_reserved(&market.creator, T::OracleBond::get());

                        // give it to the real reporter
                        T::Currency::resolve_creating(&report.by, imbalance);
                    }
                }
                MarketStatus::Disputed => {
                    let disputes = Self::disputes(market_id.clone());
                    let num_disputes = disputes.len() as u16;

                    let mut correct_reporters: Vec<T::AccountId> = Vec::new();

                    let mut overall_imbalance = NegativeImbalanceOf::<T>::zero();

                    // if the reporter reported right, return the OracleBond, otherwise
                    // slash it to pay the correct reporters
                    if report.outcome == resolved_outcome {
                        T::Currency::unreserve(&market.creator, T::OracleBond::get());
                    } else {
                        let (imbalance, _) =
                            T::Currency::slash_reserved(&market.creator, T::OracleBond::get());

                        overall_imbalance.subsume(imbalance);
                    }

                    for i in 0..num_disputes {
                        let dispute = &disputes[i as usize];
                        let dispute_bond =
                            T::DisputeBond::get() + T::DisputeFactor::get() * i.into();
                        if dispute.outcome == resolved_outcome {
                            T::Currency::unreserve(&dispute.by, dispute_bond);

                            correct_reporters.push(dispute.by.clone());
                        } else {
                            let (imbalance, _) =
                                T::Currency::slash_reserved(&dispute.by, dispute_bond);
                            overall_imbalance.subsume(imbalance);
                        }
                    }

                    // fold all the imbalances into one and reward the correct reporters.
                    let reward_per_each =
                        overall_imbalance.peek() / (correct_reporters.len() as u32).into();
                    for i in 0..correct_reporters.len() {
                        let (amount, leftover) = overall_imbalance.split(reward_per_each);
                        T::Currency::resolve_creating(&correct_reporters[i], amount);
                        overall_imbalance = leftover;
                    }
                }
                _ => (),
            };

            if let MarketType::Categorical(_) = market.market_type {
                if let Outcome::Categorical(index) = resolved_outcome {
                    let assets = Self::outcome_assets(*market_id, market);
                    for asset in assets.iter() {
                        if let Asset::CategoricalOutcome(_, inner_index) = asset {
                            if index == *inner_index { continue; }
                            let accounts = T::Shares::accounts_by_currency_id(*asset);
                            T::Shares::destroy_all(*asset, accounts.iter().cloned());
                        }
                    }
                }
            }

            <Markets<T>>::mutate(&market_id, |m| {
                m.as_mut().unwrap().status = MarketStatus::Resolved;
                m.as_mut().unwrap().resolved_outcome = Some(resolved_outcome);
            });

            Ok(())
        }

        fn is_market_active(end: MarketEnd<T::BlockNumber>) -> bool {
            match end {
                MarketEnd::Block(block) => {
                    let current_block = <frame_system::Pallet<T>>::block_number();
                    return current_block < block;
                }
                MarketEnd::Timestamp(timestamp) => {
                    let now = <pallet_timestamp::Pallet<T>>::get().saturated_into::<u64>();
                    return now < timestamp;
                }
            }
        }

        fn market_by_id(
            market_id: &T::MarketId,
        ) -> Result<Market<T::AccountId, T::BlockNumber>, Error<T>>
        where
            T: Config,
        {
            Self::markets(market_id).ok_or(Error::<T>::MarketDoesNotExist.into())
        }

        fn ensure_create_market_end(end: MarketEnd<T::BlockNumber>) -> DispatchResult {
            match end {
                MarketEnd::Block(block) => {
                    let current_block = <frame_system::Pallet<T>>::block_number();
                    ensure!(current_block < block, Error::<T>::EndBlockTooSoon);
                }
                MarketEnd::Timestamp(timestamp) => {
                    let now = <pallet_timestamp::Pallet<T>>::get();
                    ensure!(
                        now < timestamp.saturated_into(),
                        Error::<T>::EndTimestampTooSoon
                    );
                }
            };
    
            Ok(())
        }    
    }

    fn remove_item<I: cmp::PartialEq + Copy>(items: &mut Vec<I>, item: I) {
        let pos = items.iter().position(|&i| i == item).unwrap();
        items.swap_remove(pos);
    }
}
