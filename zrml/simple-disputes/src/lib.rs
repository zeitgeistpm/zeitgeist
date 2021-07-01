//! # Simple disputes
//!
//! Manages market disputes and resolutions.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod mock;
mod resolution_counters;
mod simple_disputes_pallet_api;
mod tests;

pub use pallet::*;
pub use resolution_counters::ResolutionCounters;
pub use simple_disputes_pallet_api::DisputeApi;

#[frame_support::pallet]
mod pallet {
    use crate::{DisputeApi, ResolutionCounters};
    use alloc::{vec, vec::Vec};
    use core::{cmp, marker::PhantomData};
    use frame_support::{
        dispatch::DispatchResult,
        ensure,
        pallet_prelude::StorageMap,
        traits::{Currency, Get, Hooks, Imbalance, IsType, ReservableCurrency},
        Blake2_128Concat, PalletId,
    };
    use frame_system::ensure_signed;
    use sp_runtime::{DispatchError, SaturatedConversion};
    use zeitgeist_primitives::{
        traits::{Swaps, ZeitgeistMultiReservableCurrency},
        types::{
            Asset, Market, MarketDispute, MarketStatus, MarketType, OutcomeReport, ScalarPosition,
        },
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    pub(crate) type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
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

        /// Common market parameters
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The maximum number of disputes allowed on any single market.
        type MaxDisputes: Get<u32>;

        /// The base amount of currency that must be bonded to ensure the oracle reports
        ///  in a timely manner.
        type OracleBond: Get<BalanceOf<Self>>;

        /// The pallet identifier.
        type PalletId: Get<PalletId>;

        /// Swap shares
        type Shares: ZeitgeistMultiReservableCurrency<
            Self::AccountId,
            Balance = BalanceOf<Self>,
            CurrencyId = Asset<<Self::MarketCommons as MarketCommonsPalletApi>::MarketId>,
        >;

        /// Swaps pallet
        type Swaps: Swaps<Self::AccountId, Balance = BalanceOf<Self>, MarketId = MarketIdOf<Self>>;

        /// The base amount of currency that must be bonded for a permissionless market,
        /// guaranteeing that it will resolve as anything but `Invalid`.
        type ValidityBond: Get<BalanceOf<Self>>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Block does not exists
        BlockDoesNotExist,
        /// Someone is trying to call `dispute` with the same outcome that is currently
        /// registered on-chain.
        CannotDisputeSameOutcome,
        /// A market with the provided ID does not exist.
        MarketDoesNotExist,
        /// The market is not reported on.
        MarketNotReported,
        /// The maximum number of disputes has been reached.
        MaxDisputesReached,
        /// Market does not have a report
        NoReport,
        /// Submitted outcome does not match market type
        OutcomeMismatch,
        /// The outcome being reported is out of range.
        OutcomeOutOfRange,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A market has been disputed [market_id, new_outcome]
        MarketDisputed(MarketIdOf<T>, OutcomeReport),
        /// A complete set of shares has been sold [market_id, seller]
        SoldCompleteSet(MarketIdOf<T>, <T as frame_system::Config>::AccountId),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    impl<T> DisputeApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type BlockNumber = T::BlockNumber;
        type Origin = T::Origin;
        type MarketId = MarketIdOf<T>;

        // MarketIdPerDisputeBlock

        fn insert_market_id_per_dispute_block(
            block: Self::BlockNumber,
            market_ids: Vec<Self::MarketId>,
        ) {
            MarketIdsPerDisputeBlock::<T>::insert(block, market_ids)
        }

        fn market_ids_per_dispute_block(
            block: &Self::BlockNumber,
        ) -> Result<Vec<Self::MarketId>, DispatchError> {
            MarketIdsPerDisputeBlock::<T>::try_get(block)
                .map_err(|_err| Error::<T>::BlockDoesNotExist.into())
        }

        fn mutate_market_ids_per_report_block<F>(
            block: &Self::BlockNumber,
            cb: F,
        ) -> Result<(), DispatchError>
        where
            F: FnOnce(&mut Vec<Self::MarketId>),
        {
            <MarketIdsPerReportBlock<T>>::try_mutate(block, |opt| {
                if let Some(vec) = opt {
                    cb(vec);
                    return Ok(());
                }
                Err(Error::<T>::BlockDoesNotExist.into())
            })
        }

        // MarketIdPerReportBlock

        fn insert_market_id_per_report_block(
            block: Self::BlockNumber,
            market_ids: Vec<Self::MarketId>,
        ) {
            MarketIdsPerReportBlock::<T>::insert(block, market_ids)
        }

        fn market_ids_per_report_block(
            block: &Self::BlockNumber,
        ) -> Result<Vec<Self::MarketId>, DispatchError> {
            MarketIdsPerReportBlock::<T>::try_get(block)
                .map_err(|_err| Error::<T>::BlockDoesNotExist.into())
        }

        // Misc

        fn disputes(
            market_id: &Self::MarketId,
        ) -> Result<Vec<MarketDispute<Self::AccountId, Self::BlockNumber>>, DispatchError> {
            Disputes::<T>::get(market_id).ok_or_else(|| Error::<T>::MarketDoesNotExist.into())
        }

        fn dispute_period() -> Self::BlockNumber {
            T::DisputePeriod::get()
        }

        fn max_disputes() -> u32 {
            T::MaxDisputes::get()
        }

        fn internal_resolve(
            market_id: &Self::MarketId,
            market: &Market<Self::AccountId, Self::BlockNumber>,
        ) -> Result<ResolutionCounters, DispatchError> {
            let report = market.report.clone().ok_or(Error::<T>::NoReport)?;
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
                    let disputes = <Disputes<T>>::get(market_id).unwrap_or_default();
                    let num_disputes = disputes.len() as u32;
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
                    let disputes = <Disputes<T>>::get(market_id).unwrap_or_default();
                    total_disputes = disputes.len() as _;

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

                    for (i, dispute) in disputes.iter().enumerate() {
                        let dispute_bond =
                            T::DisputeBond::get() + T::DisputeFactor::get() * (i as u32).into();
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

            T::MarketCommons::mutate_market(&market_id, |m| {
                m.status = MarketStatus::Resolved;
                m.resolved_outcome = Some(resolved_outcome);
                Ok(())
            })?;

            Ok(ResolutionCounters {
                total_accounts,
                total_asset_accounts,
                total_categories,
                total_disputes,
            })
        }

        fn on_dispute(
            origin: Self::Origin,
            market_id: Self::MarketId,
            outcome: OutcomeReport,
        ) -> Result<[u32; 2], DispatchError> {
            let sender = ensure_signed(origin)?;

            let market = T::MarketCommons::market(&market_id)?;

            ensure!(market.report.is_some(), Error::<T>::MarketNotReported);

            if let OutcomeReport::Categorical(inner) = outcome {
                if let MarketType::Categorical(categories) = market.market_type {
                    ensure!(inner < categories, Error::<T>::OutcomeOutOfRange);
                } else {
                    return Err(Error::<T>::OutcomeMismatch.into());
                }
            }
            if let OutcomeReport::Scalar(inner) = outcome {
                if let MarketType::Scalar(outcome_range) = market.market_type {
                    ensure!(
                        inner >= outcome_range.0 && inner <= outcome_range.1,
                        Error::<T>::OutcomeOutOfRange
                    );
                } else {
                    return Err(Error::<T>::OutcomeMismatch.into());
                }
            }

            let disputes = <Disputes<T>>::get(market_id).unwrap_or_default();
            let num_disputes = disputes.len() as u32;
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
                let mut old_disputes_per_block =
                    Self::market_ids_per_dispute_block(&at).unwrap_or_default();
                Self::remove_item::<MarketIdOf<T>>(&mut old_disputes_per_block, market_id);
                <MarketIdsPerDisputeBlock<T>>::insert(at, old_disputes_per_block);
            }

            let does_not_exist = <MarketIdsPerDisputeBlock<T>>::mutate(current_block, |ids_opt| {
                if let Some(ids) = ids_opt {
                    ids.push(market_id);
                    false
                } else {
                    true
                }
            });
            if does_not_exist {
                <MarketIdsPerDisputeBlock<T>>::insert(current_block, vec![market_id]);
            }

            let does_not_exist = <Disputes<T>>::mutate(market_id, |disputes_opt| {
                if let Some(disputes) = disputes_opt {
                    disputes.push(MarketDispute {
                        at: current_block,
                        by: sender.clone(),
                        outcome: outcome.clone(),
                    });
                    false
                } else {
                    true
                }
            });
            if does_not_exist {
                <Disputes<T>>::insert(
                    market_id,
                    vec![MarketDispute { at: current_block, by: sender, outcome: outcome.clone() }],
                );
            }

            // if not already in dispute
            if market.status != MarketStatus::Disputed {
                T::MarketCommons::mutate_market(&market_id, |m| {
                    m.status = MarketStatus::Disputed;
                    Ok(())
                })?;
            }

            Self::deposit_event(Event::MarketDisputed(market_id, outcome));
            Ok([num_disputes as u32, max_disputes as u32])
        }

        fn on_resolution<F>(now: Self::BlockNumber, mut cb: F) -> DispatchResult
        where
            F: FnMut(&Market<Self::AccountId, Self::BlockNumber>, ResolutionCounters),
        {
            let dispute_period = T::DisputePeriod::get();
            if now <= dispute_period {
                return Ok(());
            }

            // Resolve all regularly reported markets.
            let report_block = now - dispute_period;
            let market_ids = Self::market_ids_per_report_block(&report_block).unwrap_or_default();
            for id in &market_ids {
                let market = T::MarketCommons::market(id)?;
                if let MarketStatus::Reported = market.status {
                    let rc = Self::internal_resolve(id, &market)?;
                    cb(&market, rc);
                }
            }

            // Resolve any disputed markets.
            let dispute_block = now - dispute_period;
            let disputed = Self::market_ids_per_dispute_block(&dispute_block).unwrap_or_default();
            for id in &disputed {
                let market = T::MarketCommons::market(id)?;
                let rc = Self::internal_resolve(id, &market)?;
                cb(&market, rc);
            }

            Ok(())
        }

        // Migrations (Temporary)

        fn dispute(
            market_id: &Self::MarketId,
        ) -> Option<Vec<MarketDispute<Self::AccountId, Self::BlockNumber>>> {
            Disputes::<T>::get(market_id)
        }

        fn insert_dispute(
            market_id: Self::MarketId,
            dispute: Vec<MarketDispute<Self::AccountId, Self::BlockNumber>>,
        ) {
            Disputes::<T>::insert(market_id, dispute);
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// For each market, this holds the dispute information for each dispute that's
    /// been issued.
    #[pallet::storage]
    pub type Disputes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        Vec<MarketDispute<T::AccountId, T::BlockNumber>>,
    >;

    /// A mapping of market identifiers to the block they were disputed at.
    /// A market only ends up here if it was disputed.
    #[pallet::storage]
    pub type MarketIdsPerDisputeBlock<T: Config> =
        StorageMap<_, Blake2_128Concat, T::BlockNumber, Vec<MarketIdOf<T>>>;

    /// A mapping of market identifiers to the block that they were reported on.
    #[pallet::storage]
    pub type MarketIdsPerReportBlock<T: Config> =
        StorageMap<_, Blake2_128Concat, T::BlockNumber, Vec<MarketIdOf<T>>>;

    impl<T: Config> Pallet<T> {
        // If a market has a pool that is `Active`, then changes from `Active` to `Stale`.
        fn manage_pool_staleness(
            market: &Market<T::AccountId, T::BlockNumber>,
            market_id: &MarketIdOf<T>,
            outcome_report: &OutcomeReport,
        ) -> DispatchResult {
            let pool_id = T::MarketCommons::market_pool(market_id)?;

            T::Swaps::set_pool_as_stale(&market.market_type, pool_id, outcome_report)?;

            Ok(())
        }

        // If a market is categorical, destroys all non-winning assets.
        fn manage_resolved_categorical_market(
            market: &Market<T::AccountId, T::BlockNumber>,
            market_id: &MarketIdOf<T>,
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

        fn outcome_assets(
            market_id: MarketIdOf<T>,
            market: &Market<T::AccountId, T::BlockNumber>,
        ) -> Vec<Asset<MarketIdOf<T>>> {
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
