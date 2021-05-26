//! # Court
//!
//! Manages market disputes

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod court_pallet_api;
mod mock;
mod tests;

pub use court_pallet_api::CourtPalletApi;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::CourtPalletApi;
    use alloc::vec::Vec;
    use core::{cmp, marker::PhantomData};
    use frame_support::{
        dispatch::{DispatchResult, DispatchResultWithPostInfo, Weight},
        ensure,
        pallet_prelude::{StorageMap, ValueQuery},
        traits::{Currency, Get, Hooks, Imbalance, IsType, ReservableCurrency},
        Blake2_128Concat, PalletId, Parameter,
    };
    use frame_system::ensure_signed;
    use sp_runtime::{
        traits::{AtLeast32Bit, MaybeSerializeDeserialize, Member},
        DispatchError, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        traits::{Swaps, ZeitgeistMultiReservableCurrency},
        types::{
            Asset, Market, MarketDispute, MarketStatus, MarketType, OutcomeReport, PoolId,
            ScalarPosition,
        },
    };

    pub const NO_REPORT: DispatchError = DispatchError::Other("Report does not exist");
    pub const NOT_RESOLVED: DispatchError = DispatchError::Other("Resolved outcome does not exist");
    pub const OUTCOME_MISMATCH: DispatchError =
        DispatchError::Other("Submitted outcome does not match market type");

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::NegativeImbalance;

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Native currency
        type Currency: ReservableCurrency<Self::AccountId>;

        /// The base amount of currency that must be bonded in order to create a dispute.
        type DisputeBond: Get<BalanceOf<Self>>;

        /// The additional amount of currency that must be bonded when creating a subsequent
        ///  dispute.
        type DisputeFactor: Get<BalanceOf<Self>>;

        /// The number of blocks the dispute period remains open.
        type DisputePeriod: Get<Self::BlockNumber>;

        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The identifier of individual markets.
        type MarketId: AtLeast32Bit
            + Copy
            + Default
            + MaybeSerializeDeserialize
            + Member
            + Parameter;

        /// The maximum number of disputes allowed on any single market.
        type MaxDisputes: Get<u16>;

        /// The base amount of currency that must be bonded to ensure the oracle reports
        ///  in a timely manner.
        type OracleBond: Get<BalanceOf<Self>>;

        /// The pallet identifier.
        type PalletId: Get<PalletId>;

        /// Swap shares
        type Shares: ZeitgeistMultiReservableCurrency<
            Self::AccountId,
            Balance = BalanceOf<Self>,
            CurrencyId = Asset<Self::MarketId>,
        >;

        /// Swap pallet
        type Swap: Swaps<Self::AccountId, Balance = BalanceOf<Self>, MarketId = Self::MarketId>;

        /// The base amount of currency that must be bonded for a permissionless market,
        /// guaranteeing that it will resolve as anything but `Invalid`.
        type ValidityBond: Get<BalanceOf<Self>>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Someone is trying to call `dispute` with the same outcome that is currently
        /// registered on-chain.
        CannotDisputeSameOutcome,
        /// A market with the provided ID does not exist.
        MarketDoesNotExist,
        /// The market is not reported on.
        MarketNotReported,
        /// The maximum number of disputes has been reached.
        MaxDisputesReached,
        /// The outcome being reported is out of range.
        OutcomeOutOfRange,
        /// A pool of a market does not exist
        PoolDoesNotExist,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A market has been disputed [market_id, new_outcome]
        MarketDisputed(<T as Config>::MarketId, OutcomeReport),
        /// A complete set of shares has been sold [market_id, seller]
        SoldCompleteSet(
            <T as Config>::MarketId,
            <T as frame_system::Config>::AccountId,
        ),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    impl<T> CourtPalletApi for Pallet<T>
    where
        T: Config,
    {
        type BlockNumber = T::BlockNumber;
        type MarketId = T::MarketId;
        type Origin = T::Origin;

        fn on_dispute(
            origin: Self::Origin,
            market_id: Self::MarketId,
            outcome: OutcomeReport,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let market = Self::market_by_id(&market_id)?;

            ensure!(market.report.is_some(), Error::<T>::MarketNotReported);

            if let OutcomeReport::Categorical(inner) = outcome {
                if let MarketType::Categorical(categories) = market.market_type {
                    ensure!(inner < categories, Error::<T>::OutcomeOutOfRange);
                } else {
                    return Err(OUTCOME_MISMATCH.into());
                }
            }
            if let OutcomeReport::Scalar(inner) = outcome {
                if let MarketType::Scalar(outcome_range) = market.market_type {
                    ensure!(
                        inner >= outcome_range.0 && inner <= outcome_range.1,
                        Error::<T>::OutcomeOutOfRange
                    );
                } else {
                    return Err(OUTCOME_MISMATCH.into());
                }
            }

            let disputes = Self::disputes(market_id);
            let num_disputes = disputes.len() as u16;
            let max_disputes = T::MaxDisputes::get();
            ensure!(num_disputes < max_disputes, Error::<T>::MaxDisputesReached);

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
                Self::remove_item::<T::MarketId>(&mut old_disputes_per_block, market_id);
                <MarketIdsPerDisputeBlock<T>>::insert(at, old_disputes_per_block);
            }

            <MarketIdsPerDisputeBlock<T>>::mutate(current_block, |ids| {
                ids.push(market_id);
            });

            <Disputes<T>>::mutate(market_id, |disputes| {
                disputes.push(MarketDispute {
                    at: current_block,
                    by: sender,
                    outcome: outcome.clone(),
                })
            });

            // if not already in dispute
            if market.status != MarketStatus::Disputed {
                <Markets<T>>::mutate(market_id, |m| {
                    m.as_mut().unwrap().status = MarketStatus::Disputed;
                });
            }

            Self::deposit_event(Event::MarketDisputed(market_id, outcome));
            Self::calculate_actual_weight(|_| 0, num_disputes as u32, max_disputes as u32)
        }

        fn on_resolution(now: Self::BlockNumber) -> Result<Weight, DispatchError> {
            let dispute_period = T::DisputePeriod::get();
            if now <= dispute_period {
                return Ok(1_000_000);
            }

            // Resolve all regularly reported markets.
            let mut total_weight: Weight = 0;
            let market_ids = Self::market_ids_per_report_block(now - dispute_period);
            market_ids.iter().for_each(|id| {
                let market =
                    Self::markets(id).expect("Market stored in report block does not exist");
                if let MarketStatus::Reported = market.status {
                    let weight = Self::internal_resolve(id).expect("Internal resolve failed");
                    total_weight = total_weight.saturating_add(weight);
                }
            });

            // Resolve any disputed markets.
            let disputed = Self::market_ids_per_dispute_block(now - dispute_period);
            disputed.iter().for_each(|id| {
                let weight = Self::internal_resolve(id).expect("Internal resolve failed");
                total_weight = total_weight.saturating_add(weight);
            });

            Ok(total_weight)
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

    #[pallet::storage]
    #[pallet::getter(fn market_to_swap_pool)]
    pub type MarketToSwapPool<T: Config> =
        StorageMap<_, Blake2_128Concat, T::MarketId, Option<PoolId>, ValueQuery>;

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

    impl<T: Config> Pallet<T> {
        /// Performs the logic for resolving a market, including slashing and distributing
        /// funds.
        ///
        /// NOTE: This function does not perform any checks on the market that is being given.
        /// In the function calling this you should that the market is already in a reported or
        /// disputed state.
        #[allow(unused_assignments, unused_variables)]
        pub(crate) fn internal_resolve(market_id: &T::MarketId) -> Result<Weight, DispatchError> {
            let market = Self::market_by_id(market_id)?;
            let report = market.report.clone().ok_or(NO_REPORT)?;
            let mut total_accounts = 0u32;
            let mut total_asset_accounts = 0u32;
            let mut total_categories = 0u32;
            let mut total_disputes = 0u32;

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
                    let disputes = Self::disputes(market_id);
                    let num_disputes = disputes.len() as u16;
                    // count the last dispute's outcome as the winning one
                    let last_dispute = disputes[(num_disputes as usize) - 1].clone();
                    last_dispute.outcome
                }
                _ => panic!("Cannot happen"),
            };

            let market_status = market.status;
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
                    let disputes = Self::disputes(market_id);
                    let num_disputes = disputes.len() as u16;
                    total_disputes = num_disputes.into();

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
                    for correct_reporter in &correct_reporters {
                        let (amount, leftover) = overall_imbalance.split(reward_per_each);
                        T::Currency::resolve_creating(correct_reporter, amount);
                        overall_imbalance = leftover;
                    }
                }
                _ => (),
            };

            let _ = Self::manage_pool_staleness(&market, market_id, &resolved_outcome);
            if let Ok([local_total_accounts, local_total_asset_accounts, local_total_categories]) =
                Self::manage_resolved_categorical_market(&market, market_id, &resolved_outcome)
            {
                total_accounts = local_total_accounts.saturated_into();
                total_asset_accounts = local_total_asset_accounts.saturated_into();
                total_categories = local_total_categories.saturated_into();
            }

            <Markets<T>>::mutate(&market_id, |m| {
                m.as_mut().unwrap().status = MarketStatus::Resolved;
                m.as_mut().unwrap().resolved_outcome = Some(resolved_outcome);
            });

            // Calculate required weight
            // MUST be updated when new market types are added.
            if let MarketType::Categorical(_) = market.market_type {
                if let MarketStatus::Reported = market_status {
                    Ok(0)
                } else {
                    Ok(0)
                }
            } else if let MarketStatus::Reported = market_status {
                Ok(0)
            } else {
                Ok(0)
            }
        }

        fn calculate_actual_weight<F>(
            func: F,
            weight_parameter: u32,
            max_weight_parameter: u32,
        ) -> DispatchResultWithPostInfo
        where
            F: Fn(u32) -> Weight,
        {
            if weight_parameter == max_weight_parameter {
                Ok(None.into())
            } else {
                Ok(Some(func(weight_parameter)).into())
            }
        }

        // If a market has a pool that is `Active`, then changes from `Active` to `Stale`.
        fn manage_pool_staleness(
            market: &Market<T::AccountId, T::BlockNumber>,
            market_id: &T::MarketId,
            outcome_report: &OutcomeReport,
        ) -> DispatchResult {
            let pool_id = Self::market_pool_id(market_id)?;

            T::Swap::set_pool_as_stale(&market.market_type, pool_id, outcome_report)?;

            Ok(())
        }

        // If a market is categorical, destroys all non-winning assets.
        fn manage_resolved_categorical_market(
            market: &Market<T::AccountId, T::BlockNumber>,
            market_id: &T::MarketId,
            outcome_report: &OutcomeReport,
        ) -> Result<[usize; 3], DispatchError> {
            let mut total_accounts: usize = 0;
            let mut total_asset_accounts: usize = 0;
            let mut total_categories: usize = 0;

            if let MarketType::Categorical(_) = market.market_type {
                if let OutcomeReport::Categorical(winning_asset_idx) = *outcome_report {
                    let assets = Self::outcome_assets(*market_id, market);
                    total_categories = assets.len().saturated_into();

                    let mut assets_iter = assets.iter().cloned();
                    let mut manage_asset = |asset: Asset<_>, winning_asset_idx| {
                        if let Asset::CategoricalOutcome(_, idx) = asset {
                            if idx == winning_asset_idx {
                                return 0;
                            }
                            let (total_accounts, accounts) =
                                T::Shares::accounts_by_currency_id(asset);
                            total_asset_accounts =
                                total_asset_accounts.saturating_add(accounts.len());
                            T::Shares::destroy_all(asset, accounts.iter().cloned());
                            total_accounts
                        } else {
                            0
                        }
                    };

                    if let Some(first_asset) = assets_iter.next() {
                        total_accounts = manage_asset(first_asset, winning_asset_idx);
                    }
                    for asset in assets_iter {
                        let _ = manage_asset(asset, winning_asset_idx);
                    }
                }
            }

            Ok([total_accounts, total_asset_accounts, total_categories])
        }

        fn market_by_id(
            market_id: &T::MarketId,
        ) -> Result<Market<T::AccountId, T::BlockNumber>, Error<T>>
        where
            T: Config,
        {
            Self::markets(market_id).ok_or(Error::<T>::MarketDoesNotExist)
        }

        // Returns the corresponding **stored** pool id of a market id
        fn market_pool_id(market_id: &T::MarketId) -> Result<u128, DispatchError> {
            if let Ok(Some(el)) = <MarketToSwapPool<T>>::try_get(market_id) {
                Ok(el)
            } else {
                Err(Error::<T>::PoolDoesNotExist.into())
            }
        }

        fn outcome_assets(
            market_id: T::MarketId,
            market: &Market<T::AccountId, T::BlockNumber>,
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
                    vec![
                        Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                        Asset::ScalarOutcome(market_id, ScalarPosition::Short),
                    ]
                }
            }
        }

        fn remove_item<I>(items: &mut Vec<I>, item: I)
        where
            I: cmp::PartialEq + Copy,
        {
            let pos = items.iter().position(|&i| i == item).unwrap();
            items.swap_remove(pos);
        }
    }
}
