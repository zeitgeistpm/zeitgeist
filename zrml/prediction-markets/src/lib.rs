// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

extern crate alloc;

mod benchmarks;
pub mod migrations;
pub mod mock;
mod tests;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::weights::*;
    use alloc::{vec, vec::Vec};
    use core::{cmp, marker::PhantomData};
    use frame_support::{
        dispatch::{DispatchResultWithPostInfo, Weight},
        ensure, log,
        pallet_prelude::{ConstU32, StorageMap, StorageValue, ValueQuery},
        storage::{with_transaction, TransactionOutcome},
        traits::{EnsureOrigin, Get, Hooks, IsType, StorageVersion},
        transactional,
        weights::Pays,
        Blake2_128Concat, BoundedVec, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use orml_traits::{MultiCurrency, NamedMultiReservableCurrency};
    use sp_arithmetic::per_things::{Perbill, Percent};
    use sp_runtime::{
        traits::{AccountIdConversion, CheckedDiv, Saturating, Zero},
        DispatchError, DispatchResult, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        constants::MILLISECS_PER_BLOCK,
        traits::{DisputeApi, Swaps, ZeitgeistAssetManager},
        types::{
            Asset, Market, MarketCreation, MarketDispute, MarketDisputeMechanism, MarketPeriod,
            MarketStatus, MarketType, MultiHash, OutcomeReport, Report, ScalarPosition,
            ScoringRule, SubsidyUntil,
        },
    };
    use zrml_liquidity_mining::LiquidityMiningPalletApi;
    use zrml_market_commons::MarketCommonsPalletApi;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

    pub(crate) type BalanceOf<T> = <<T as Config>::AssetManager as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    pub(crate) type TimeFrame = u64;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub type CacheSize = ConstU32<64>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Destroy a market, including its outcome assets, market account and pool account.
        ///
        /// Must be called by `DestroyOrigin`. Bonds (unless already returned) are slashed without
        /// exception. Can currently only be used for destroying CPMM markets.
        #[pallet::weight((
            T::WeightInfo::admin_destroy_reported_market(
                900,
                900,
                T::MaxCategories::get().into()
            ).max(T::WeightInfo::admin_destroy_disputed_market(
                900,
                900,
                T::MaxCategories::get().into()
            )), Pays::No))]
        #[transactional]
        pub fn admin_destroy_market(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            // TODO(#618): Not implemented for Rikiddo!
            T::DestroyOrigin::ensure_origin(origin)?;

            let mut total_accounts = 0usize;
            let mut share_accounts = 0usize;
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.scoring_rule == ScoringRule::CPMM, Error::<T>::InvalidScoringRule);
            let market_status = market.status;
            let outcome_assets = Self::outcome_assets(market_id, &market);
            let outcome_assets_amount = outcome_assets.len();
            let market_account = Self::market_account(market_id);

            // Slash outstanding bonds; see
            // https://github.com/zeitgeistpm/runtime-audit-1/issues/34#issuecomment-1120187097 for
            // details.
            let slash_market_creator = |amount| {
                T::AssetManager::slash_reserved_named(
                    &Self::reserve_id(),
                    Asset::Ztg,
                    &market.creator,
                    amount,
                );
            };
            if market_status == MarketStatus::Proposed {
                slash_market_creator(T::AdvisoryBond::get());
            }
            if market_status != MarketStatus::Resolved
                && market_status != MarketStatus::InsufficientSubsidy
            {
                if market.creation == MarketCreation::Permissionless {
                    slash_market_creator(T::ValidityBond::get());
                }
                slash_market_creator(T::OracleBond::get());
            }

            // Delete market's outcome assets, clear market and delete pool if necessary.
            let mut destroy_asset = |asset: Asset<_>| -> Option<usize> {
                if let Ok((total_accounts, accounts)) =
                    T::AssetManager::accounts_by_currency_id(asset)
                {
                    share_accounts = share_accounts.saturating_add(accounts.len());
                    let _ = T::AssetManager::destroy_all(asset, accounts.iter().cloned());
                    Some(total_accounts)
                } else {
                    // native currency case
                    None
                }
            };
            for asset in outcome_assets.into_iter() {
                if let Some(total) = destroy_asset(asset) {
                    total_accounts = total;
                }
            }
            T::AssetManager::slash(
                Asset::Ztg,
                &market_account,
                T::AssetManager::free_balance(Asset::Ztg, &market_account),
            );
            if let Ok(pool_id) = T::MarketCommons::market_pool(&market_id) {
                T::Swaps::destroy_pool(pool_id)?;
                T::MarketCommons::remove_market_pool(&market_id)?;
            }

            Self::clear_auto_open(&market_id)?;
            Self::clear_auto_close(&market_id)?;
            Self::clear_auto_resolve(&market_id)?;
            T::MarketCommons::remove_market(&market_id)?;
            Disputes::<T>::remove(market_id);

            Self::deposit_event(Event::MarketDestroyed(market_id));

            // Weight correction
            // The DestroyOrigin should not pay fees for providing this service
            if market_status == MarketStatus::Reported {
                Ok((
                    Some(T::WeightInfo::admin_destroy_reported_market(
                        total_accounts.saturated_into(),
                        share_accounts.saturated_into(),
                        outcome_assets_amount.saturated_into(),
                    )),
                    Pays::No,
                )
                    .into())
            } else if market_status == MarketStatus::Disputed {
                Ok((
                    Some(T::WeightInfo::admin_destroy_disputed_market(
                        total_accounts.saturated_into(),
                        share_accounts.saturated_into(),
                        outcome_assets_amount.saturated_into(),
                    )),
                    Pays::No,
                )
                    .into())
            } else {
                Ok((None, Pays::No).into())
            }
        }

        /// Allows the `CloseOrigin` to immediately move an open market to closed.
        //
        // ***** IMPORTANT *****
        //
        // Within the same block, operations that interact with the activeness of the same
        // market will behave differently before and after this call.
        #[pallet::weight((T::WeightInfo::admin_move_market_to_closed(), Pays::No))]
        #[transactional]
        pub fn admin_move_market_to_closed(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            // TODO(#638): Handle Rikiddo markets!
            T::CloseOrigin::ensure_origin(origin)?;
            let market = T::MarketCommons::market(&market_id)?;
            Self::ensure_market_is_active(&market)?;
            Self::clear_auto_open(&market_id)?;
            Self::clear_auto_close(&market_id)?;
            Self::close_market(&market_id)?;
            // The CloseOrigin should not pay fees for providing this service
            Ok((None, Pays::No).into())
        }

        /// Allows the `ResolveOrigin` to immediately move a reported or disputed
        /// market to resolved.
        ////
        #[pallet::weight((T::WeightInfo::admin_move_market_to_resolved_overhead()
            .saturating_add(T::WeightInfo::internal_resolve_categorical_reported(
                4_200,
                4_200,
                T::MaxCategories::get().into()
            ).saturating_sub(T::WeightInfo::internal_resolve_scalar_reported())
        ), Pays::No))]
        #[transactional]
        pub fn admin_move_market_to_resolved(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::ResolveOrigin::ensure_origin(origin)?;

            let market = T::MarketCommons::market(&market_id)?;
            ensure!(
                market.status == MarketStatus::Reported || market.status == MarketStatus::Disputed,
                Error::<T>::InvalidMarketStatus,
            );
            Self::clear_auto_resolve(&market_id)?;
            let market = T::MarketCommons::market(&market_id)?;
            let weight = Self::on_resolution(&market_id, &market)?;
            Ok((Some(weight), Pays::No).into())
        }

        /// Approves a market that is waiting for approval from the
        /// advisory committee.
        ///
        /// NOTE: Returns the proposer's bond since the market has been
        /// deemed valid by an advisory committee.
        ///
        /// NOTE: Can only be called by the `ApproveOrigin`.
        ///
        #[pallet::weight((T::WeightInfo::approve_market(), Pays::No))]
        #[transactional]
        pub fn approve_market(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::ApproveOrigin::ensure_origin(origin)?;
            let mut extra_weight = 0;
            let mut status = MarketStatus::Active;

            T::MarketCommons::mutate_market(&market_id, |m| {
                ensure!(m.status == MarketStatus::Proposed, Error::<T>::MarketIsNotProposed);

                match m.scoring_rule {
                    ScoringRule::CPMM => {
                        m.status = MarketStatus::Active;
                    }
                    ScoringRule::RikiddoSigmoidFeeMarketEma => {
                        m.status = MarketStatus::CollectingSubsidy;
                        status = MarketStatus::CollectingSubsidy;
                        extra_weight = Self::start_subsidy(m, market_id)?;
                    }
                }

                T::AssetManager::unreserve_named(
                    &Self::reserve_id(),
                    Asset::Ztg,
                    &m.creator,
                    T::AdvisoryBond::get(),
                );
                Ok(())
            })?;

            Self::deposit_event(Event::MarketApproved(market_id, status));
            // The ApproveOrigin should not pay fees for providing this service
            Ok((Some(T::WeightInfo::approve_market().saturating_add(extra_weight)), Pays::No)
                .into())
        }

        /// Buy a complete set of outcome shares of a market.
        ///
        /// The cost of a full set is exactly one unit of the market's base asset. For example,
        /// when calling `buy_complete_set(origin, 1, 2)` on a categorical market with five
        /// different outcomes, the caller pays `2` of the base asset and receives `2` of each of
        /// the five outcome tokens.
        ///
        /// NOTE: This is the only way to create new shares of outcome tokens.
        // Note: `buy_complete_set` weight consumption is dependent on how many assets exists.
        // Unfortunately this information can only be retrieved with a storage call, therefore
        // The worst-case scenario is assumed and the correct weight is calculated at the end of this function.
        // This also occurs in numerous other functions.
        #[pallet::weight(
            T::WeightInfo::buy_complete_set(T::MaxCategories::get().into())
        )]
        #[transactional]
        pub fn buy_complete_set(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            Self::do_buy_complete_set(sender, market_id, amount)
        }

        #[pallet::weight(T::WeightInfo::dispute(T::MaxDisputes::get()))]
        #[transactional]
        pub fn dispute(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let disputes = Disputes::<T>::get(market_id);
            let curr_block_num = <frame_system::Pallet<T>>::block_number();
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(
                matches!(market.status, MarketStatus::Reported | MarketStatus::Disputed),
                Error::<T>::InvalidMarketStatus
            );
            let num_disputes: u32 = disputes.len().saturated_into();
            Self::validate_dispute(&disputes, &market, num_disputes, &outcome)?;
            T::AssetManager::reserve_named(
                &Self::reserve_id(),
                Asset::Ztg,
                &who,
                default_dispute_bond::<T>(disputes.len()),
            )?;
            match market.dispute_mechanism {
                MarketDisputeMechanism::Authorized(_) => {
                    T::Authorized::on_dispute(&disputes, &market_id, &market)?
                }
                MarketDisputeMechanism::Court => {
                    T::Court::on_dispute(&disputes, &market_id, &market)?
                }
                MarketDisputeMechanism::SimpleDisputes => {
                    T::SimpleDisputes::on_dispute(&disputes, &market_id, &market)?
                }
            }
            Self::remove_last_dispute_from_market_ids_per_dispute_block(&disputes, &market_id)?;
            Self::set_market_as_disputed(&market, &market_id)?;
            let market_dispute = MarketDispute { at: curr_block_num, by: who, outcome };
            <Disputes<T>>::try_mutate(market_id, |disputes| {
                disputes.try_push(market_dispute.clone()).map_err(|_| <Error<T>>::StorageOverflow)
            })?;
            <MarketIdsPerDisputeBlock<T>>::try_mutate(curr_block_num, |ids| {
                ids.try_push(market_id).map_err(|_| <Error<T>>::StorageOverflow)
            })?;
            Self::deposit_event(Event::MarketDisputed(
                market_id,
                MarketStatus::Disputed,
                market_dispute,
            ));
            Self::calculate_actual_weight(
                T::WeightInfo::dispute,
                num_disputes,
                T::MaxDisputes::get(),
            )
        }

        /// Create a permissionless market, buy complete sets and deploy a pool with specified
        /// liquidity.
        ///
        /// # Arguments
        ///
        /// * `oracle`: The oracle of the market who will report the correct outcome.
        /// * `period`: The active period of the market.
        /// * `metadata`: A hash pointer to the metadata of the market.
        /// * `market_type`: The type of the market.
        /// * `dispute_mechanism`: The market dispute mechanism.
        /// * `swap_fee`: The swap fee, specified as fixed-point ratio (0.1 equals 10% fee)
        /// * `amount`: The amount of each token to add to the pool.
        /// * `weights`: The relative denormalized weight of each asset price.
        #[pallet::weight(
            T::WeightInfo::create_market()
            .saturating_add(T::WeightInfo::buy_complete_set(T::MaxCategories::get().into()))
            .saturating_add(T::WeightInfo::deploy_swap_pool_for_market(
                T::MaxCategories::get().into(),
            ))
            // Overly generous estimation, since we have no access to Swaps WeightInfo
            // (it is loosely coupled to this pallet using a trait). Contains weight for
            // create_pool() and swap_exact_amount_in().
            .saturating_add(5_000_000_000.saturating_mul(T::MaxCategories::get().into()))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
        )]
        #[transactional]
        pub fn create_cpmm_market_and_deploy_assets(
            origin: OriginFor<T>,
            oracle: T::AccountId,
            period: MarketPeriod<T::BlockNumber, MomentOf<T>>,
            metadata: MultiHash,
            market_type: MarketType,
            dispute_mechanism: MarketDisputeMechanism<T::AccountId>,
            #[pallet::compact] swap_fee: BalanceOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
            weights: Vec<u128>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_signed(origin.clone())?;

            let create_market_weight = Self::create_market(
                origin.clone(),
                oracle,
                period,
                metadata,
                MarketCreation::Permissionless,
                market_type.clone(),
                dispute_mechanism,
                ScoringRule::CPMM,
            )?
            .actual_weight
            .unwrap_or_else(T::WeightInfo::create_market);

            // Deploy the swap pool and populate it.
            let asset_count = match market_type {
                MarketType::Categorical(value) => value,
                MarketType::Scalar(_) => 2,
            };
            let market_id = T::MarketCommons::latest_market_id()?;
            let deploy_and_populate_weight = Self::deploy_swap_pool_and_additional_liquidity(
                origin,
                market_id,
                swap_fee,
                amount,
                weights.clone(),
            )?
            .actual_weight
            .unwrap_or_else(|| {
                T::WeightInfo::buy_complete_set(asset_count.into())
                    .saturating_add(T::WeightInfo::deploy_swap_pool_for_market(asset_count.into()))
                    .saturating_add(5_000_000_000.saturating_mul(asset_count.into()))
                    .saturating_add(T::DbWeight::get().reads(2 as Weight))
            });

            Ok(Some(create_market_weight.saturating_add(deploy_and_populate_weight)).into())
        }

        #[pallet::weight(T::WeightInfo::create_market())]
        #[transactional]
        pub fn create_market(
            origin: OriginFor<T>,
            oracle: T::AccountId,
            period: MarketPeriod<T::BlockNumber, MomentOf<T>>,
            metadata: MultiHash,
            creation: MarketCreation,
            market_type: MarketType,
            dispute_mechanism: MarketDisputeMechanism<T::AccountId>,
            scoring_rule: ScoringRule,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            Self::ensure_market_period_is_valid(&period)?;

            match market_type {
                MarketType::Categorical(categories) => {
                    ensure!(categories >= T::MinCategories::get(), <Error<T>>::NotEnoughCategories);
                    ensure!(categories <= T::MaxCategories::get(), <Error<T>>::TooManyCategories);
                }
                MarketType::Scalar(ref outcome_range) => {
                    ensure!(
                        outcome_range.start() < outcome_range.end(),
                        <Error<T>>::InvalidOutcomeRange
                    );
                }
            }
            if scoring_rule == ScoringRule::RikiddoSigmoidFeeMarketEma {
                Self::ensure_market_start_is_in_time(&period)?;
            }

            // Require sha3-384 as multihash. TODO(#608) The irrefutable `if let` is a workaround
            // for a compiler error. Link an issue for this!
            #[allow(irrefutable_let_patterns)]
            let multihash =
                if let MultiHash::Sha3_384(multihash) = metadata { multihash } else { [0u8; 50] };
            ensure!(multihash[0] == 0x15 && multihash[1] == 0x30, <Error<T>>::InvalidMultihash);

            let status: MarketStatus = match creation {
                MarketCreation::Permissionless => {
                    let required_bond = T::ValidityBond::get().saturating_add(T::OracleBond::get());
                    T::AssetManager::reserve_named(
                        &Self::reserve_id(),
                        Asset::Ztg,
                        &sender,
                        required_bond,
                    )?;

                    match scoring_rule {
                        ScoringRule::CPMM => MarketStatus::Active,
                        ScoringRule::RikiddoSigmoidFeeMarketEma => MarketStatus::CollectingSubsidy,
                    }
                }
                MarketCreation::Advised => {
                    let required_bond = T::AdvisoryBond::get().saturating_add(T::OracleBond::get());
                    T::AssetManager::reserve_named(
                        &Self::reserve_id(),
                        Asset::Ztg,
                        &sender,
                        required_bond,
                    )?;
                    MarketStatus::Proposed
                }
            };

            let market = Market {
                creation,
                creator_fee: 0,
                creator: sender,
                market_type,
                dispute_mechanism,
                metadata: Vec::from(multihash),
                oracle,
                period: period.clone(),
                report: None,
                resolved_outcome: None,
                status,
                scoring_rule,
            };
            let market_id = T::MarketCommons::push_market(market.clone())?;
            let market_account = Self::market_account(market_id);
            let mut extra_weight = 0;

            if market.status == MarketStatus::CollectingSubsidy {
                extra_weight = Self::start_subsidy(&market, market_id)?;
            }

            match period {
                MarketPeriod::Block(range) => {
                    MarketIdsPerCloseBlock::<T>::try_mutate(range.end, |ids| {
                        ids.try_push(market_id).map_err(|_| <Error<T>>::StorageOverflow)
                    })?;
                }
                MarketPeriod::Timestamp(range) => {
                    MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                        Self::calculate_time_frame_of_moment(range.end),
                        |ids| ids.try_push(market_id).map_err(|_| <Error<T>>::StorageOverflow),
                    )?;
                }
            }

            Self::deposit_event(Event::MarketCreated(market_id, market_account, market));

            Ok(Some(T::WeightInfo::create_market().saturating_add(extra_weight)).into())
        }

        /// Buy complete sets and deploy a pool with specified liquidity for a market.
        ///
        /// # Arguments
        ///
        /// * `market_id`: The id of the market.
        /// * `swap_fee`: The swap fee, specified as fixed-point ratio (0.1 equals 10% fee)
        /// * `amount`: The amount of each token to add to the pool.
        /// * `weights`: The relative denormalized weight of each outcome asset. The sum of the
        ///     weights must be less or equal to _half_ of the `MaxTotalWeight` constant of the
        ///     swaps pallet.
        #[pallet::weight(
            T::WeightInfo::buy_complete_set(T::MaxCategories::get().into())
            .saturating_add(T::WeightInfo::deploy_swap_pool_for_market(
                T::MaxCategories::get().into(),
            ))
            // Overly generous estimation, since we have no access to Swaps WeightInfo
            // (it is loosely coupled to this pallet using a trait). Contains weight for
            // create_pool() and swap_exact_amount_in()
            .saturating_add(5_000_000_000.saturating_mul(T::MaxCategories::get().into()))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
        )]
        #[transactional]
        pub fn deploy_swap_pool_and_additional_liquidity(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] swap_fee: BalanceOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
            weights: Vec<u128>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin.clone())?;
            let weight_bcs = Self::buy_complete_set(origin.clone(), market_id, amount)?
                .actual_weight
                .unwrap_or_else(|| T::WeightInfo::buy_complete_set(T::MaxCategories::get().into()));
            let weights_len = weights.len();
            Self::deploy_swap_pool_for_market(origin, market_id, swap_fee, amount, weights)?;
            Ok(Some(weight_bcs.saturating_add(T::WeightInfo::deploy_swap_pool_for_market(
                weights_len.saturated_into(),
            )))
            .into())
        }

        /// Deploy a pool with specified liquidity for a market.
        ///
        /// The sender must have enough funds to cover all of the required shares to seed the pool.
        ///
        /// # Arguments
        ///
        /// * `market_id`: The id of the market.
        /// * `swap_fee`: The swap fee, specified as fixed-point ratio (0.1 equals 10% fee)
        /// * `amount`: The amount of each token to add to the pool.
        /// * `weights`: The relative denormalized weight of each outcome asset. The sum of the
        ///     weights must be less or equal to _half_ of the `MaxTotalWeight` constant of the
        ///     swaps pallet.
        #[pallet::weight(
            T::WeightInfo::deploy_swap_pool_for_market(weights.len() as u32)
        )]
        #[transactional]
        pub fn deploy_swap_pool_for_market(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] swap_fee: BalanceOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
            mut weights: Vec<u128>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.scoring_rule == ScoringRule::CPMM, Error::<T>::InvalidScoringRule);
            Self::ensure_market_is_active(&market)?;

            let mut assets = Self::outcome_assets(market_id, &market);
            let base_asset = Asset::Ztg;
            assets.push(base_asset);
            let base_asset_weight = weights.iter().fold(0u128, |acc, val| acc.saturating_add(*val));
            weights.push(base_asset_weight);

            let pool_id = T::Swaps::create_pool(
                sender,
                assets,
                base_asset,
                market_id,
                ScoringRule::CPMM,
                Some(swap_fee),
                Some(amount),
                Some(weights),
            )?;

            // Open the pool now or cache it for later
            match market.period {
                MarketPeriod::Block(ref range) => {
                    let current_block = <frame_system::Pallet<T>>::block_number();
                    let open_block = range.start;
                    if current_block < open_block {
                        MarketIdsPerOpenBlock::<T>::try_mutate(open_block, |ids| {
                            ids.try_push(market_id).map_err(|_| <Error<T>>::StorageOverflow)
                        })?;
                    } else {
                        T::Swaps::open_pool(pool_id)?;
                    }
                }
                MarketPeriod::Timestamp(ref range) => {
                    let current_time_frame =
                        Self::calculate_time_frame_of_moment(T::MarketCommons::now());
                    let open_time_frame = Self::calculate_time_frame_of_moment(range.start);
                    if current_time_frame < open_time_frame {
                        MarketIdsPerOpenTimeFrame::<T>::try_mutate(open_time_frame, |ids| {
                            ids.try_push(market_id).map_err(|_| <Error<T>>::StorageOverflow)
                        })?;
                    } else {
                        T::Swaps::open_pool(pool_id)?;
                    }
                }
            };

            // This errors if a pool already exists!
            T::MarketCommons::insert_market_pool(market_id, pool_id)?;
            Ok(())
        }

        /// Redeems the winning shares of a prediction market.
        ///
        #[pallet::weight(T::WeightInfo::redeem_shares_categorical()
            .max(T::WeightInfo::redeem_shares_scalar())
        )]
        #[transactional]
        pub fn redeem_shares(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let market = T::MarketCommons::market(&market_id)?;
            let market_account = Self::market_account(market_id);

            ensure!(market.status == MarketStatus::Resolved, Error::<T>::MarketIsNotResolved);

            // Check to see if the sender has any winning shares.
            let resolved_outcome =
                market.resolved_outcome.ok_or(Error::<T>::MarketIsNotResolved)?;

            let winning_assets = match resolved_outcome {
                OutcomeReport::Categorical(category_index) => {
                    let winning_currency_id = Asset::CategoricalOutcome(market_id, category_index);
                    let winning_balance =
                        T::AssetManager::free_balance(winning_currency_id, &sender);

                    ensure!(winning_balance > BalanceOf::<T>::zero(), Error::<T>::NoWinningBalance);

                    // Ensure the market account has enough to pay out - if this is
                    // ever not true then we have an accounting problem.
                    ensure!(
                        T::AssetManager::free_balance(Asset::Ztg, &market_account)
                            >= winning_balance,
                        Error::<T>::InsufficientFundsInMarketAccount,
                    );

                    vec![(winning_currency_id, winning_balance, winning_balance)]
                }
                OutcomeReport::Scalar(value) => {
                    let long_currency_id = Asset::ScalarOutcome(market_id, ScalarPosition::Long);
                    let short_currency_id = Asset::ScalarOutcome(market_id, ScalarPosition::Short);
                    let long_balance = T::AssetManager::free_balance(long_currency_id, &sender);
                    let short_balance = T::AssetManager::free_balance(short_currency_id, &sender);

                    ensure!(
                        long_balance > BalanceOf::<T>::zero()
                            || short_balance > BalanceOf::<T>::zero(),
                        Error::<T>::NoWinningBalance
                    );

                    let bound = if let MarketType::Scalar(range) = market.market_type {
                        range
                    } else {
                        return Err(Error::<T>::InvalidMarketType.into());
                    };

                    let calc_payouts = |final_value: u128,
                                        low: u128,
                                        high: u128|
                     -> (Perbill, Perbill) {
                        if final_value <= low {
                            return (Perbill::zero(), Perbill::one());
                        }
                        if final_value >= high {
                            return (Perbill::one(), Perbill::zero());
                        }

                        let payout_long: Perbill = Perbill::from_rational(
                            final_value.saturating_sub(low),
                            high.saturating_sub(low),
                        );
                        let payout_short: Perbill = Perbill::from_parts(
                            Perbill::one().deconstruct().saturating_sub(payout_long.deconstruct()),
                        );
                        (payout_long, payout_short)
                    };

                    let (long_percent, short_percent) =
                        calc_payouts(value, *bound.start(), *bound.end());

                    let long_payout = long_percent.mul_floor(long_balance);
                    let short_payout = short_percent.mul_floor(short_balance);
                    // Ensure the market account has enough to pay out - if this is
                    // ever not true then we have an accounting problem.
                    ensure!(
                        T::AssetManager::free_balance(Asset::Ztg, &market_account)
                            >= long_payout.saturating_add(short_payout),
                        Error::<T>::InsufficientFundsInMarketAccount,
                    );

                    vec![
                        (long_currency_id, long_payout, long_balance),
                        (short_currency_id, short_payout, short_balance),
                    ]
                }
            };

            for (currency_id, payout, balance) in winning_assets {
                // Destroy the shares.
                T::AssetManager::slash(currency_id, &sender, balance);

                // Pay out the winner.
                let remaining_bal = T::AssetManager::free_balance(Asset::Ztg, &market_account);
                let actual_payout = payout.min(remaining_bal);

                T::AssetManager::transfer(Asset::Ztg, &market_account, &sender, actual_payout)?;
                // The if-check prevents scalar markets to emit events even if sender only owns one
                // of the outcome tokens.
                if balance != <BalanceOf<T>>::zero() {
                    Self::deposit_event(Event::TokensRedeemed(
                        market_id,
                        currency_id,
                        balance,
                        actual_payout,
                        sender.clone(),
                    ));
                }
            }

            // Weight correction
            if let OutcomeReport::Categorical(_) = resolved_outcome {
                return Ok(Some(T::WeightInfo::redeem_shares_categorical()).into());
            } else if let OutcomeReport::Scalar(_) = resolved_outcome {
                return Ok(Some(T::WeightInfo::redeem_shares_scalar()).into());
            }

            Ok(None.into())
        }

        /// Rejects a market that is waiting for approval from the advisory committee.
        #[pallet::weight((T::WeightInfo::reject_market(), Pays::No))]
        #[transactional]
        pub fn reject_market(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::RejectOrigin::ensure_origin(origin)?;
            let market = T::MarketCommons::market(&market_id)?;
            Self::clear_auto_open(&market_id)?;
            Self::clear_auto_close(&market_id)?;
            Self::do_reject_market(&market_id, market)?;
            // The RejectOrigin should not pay fees for providing this service
            Ok((None, Pays::No).into())
        }

        /// Reports the outcome of a market.
        ///
        #[pallet::weight(T::WeightInfo::report())]
        #[transactional]
        pub fn report(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
        ) -> DispatchResult {
            let sender = ensure_signed(origin.clone())?;

            let current_block = <frame_system::Pallet<T>>::block_number();
            let market_report = Report { at: current_block, by: sender.clone(), outcome };

            T::MarketCommons::mutate_market(&market_id, |market| {
                ensure!(market.report.is_none(), Error::<T>::MarketAlreadyReported);
                Self::ensure_market_is_closed(market)?;
                ensure!(
                    market.matches_outcome_report(&market_report.outcome),
                    Error::<T>::OutcomeMismatch
                );

                let mut should_check_origin = false;
                match market.period {
                    MarketPeriod::Block(ref range) => {
                        if current_block
                            <= range.end.saturating_add(T::ReportingPeriod::get().into())
                        {
                            should_check_origin = true;
                        }
                    }
                    MarketPeriod::Timestamp(ref range) => {
                        let rp_moment: MomentOf<T> = T::ReportingPeriod::get().into();
                        let reporting_period_in_ms =
                            rp_moment.saturating_mul(MILLISECS_PER_BLOCK.into());
                        if T::MarketCommons::now()
                            <= range.end.saturating_add(reporting_period_in_ms)
                        {
                            should_check_origin = true;
                        }
                    }
                }

                if should_check_origin {
                    let sender_is_oracle = sender == market.oracle;
                    let origin_has_permission = T::ResolveOrigin::ensure_origin(origin).is_ok();
                    ensure!(
                        sender_is_oracle || origin_has_permission,
                        Error::<T>::ReporterNotOracle
                    );
                }

                market.report = Some(market_report.clone());
                market.status = MarketStatus::Reported;

                Ok(())
            })?;

            MarketIdsPerReportBlock::<T>::try_mutate(current_block, |ids| {
                ids.try_push(market_id).map_err(|_| <Error<T>>::StorageOverflow)
            })?;

            Self::deposit_event(Event::MarketReported(
                market_id,
                MarketStatus::Reported,
                market_report,
            ));
            Ok(())
        }

        /// Sells a complete set of outcomes shares for a market.
        ///
        /// Each complete set is sold for one unit of the market's base asset.
        #[pallet::weight(
            T::WeightInfo::sell_complete_set(T::MaxCategories::get().into())
        )]
        #[transactional]
        pub fn sell_complete_set(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(amount != BalanceOf::<T>::zero(), Error::<T>::ZeroAmount);

            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.scoring_rule == ScoringRule::CPMM, Error::<T>::InvalidScoringRule);
            Self::ensure_market_is_active(&market)?;

            let market_account = Self::market_account(market_id);
            ensure!(
                T::AssetManager::free_balance(Asset::Ztg, &market_account) >= amount,
                "Market account does not have sufficient reserves.",
            );

            let assets = Self::outcome_assets(market_id, &market);

            // verify first.
            for asset in assets.iter() {
                // Ensures that the sender has sufficient amount of each
                // share in the set.
                ensure!(
                    T::AssetManager::free_balance(*asset, &sender) >= amount,
                    Error::<T>::InsufficientShareBalance,
                );
            }

            // write last.
            for asset in assets.iter() {
                T::AssetManager::slash(*asset, &sender, amount);
            }

            T::AssetManager::transfer(Asset::Ztg, &market_account, &sender, amount)?;

            Self::deposit_event(Event::SoldCompleteSet(market_id, amount, sender));
            let assets_len: u32 = assets.len().saturated_into();
            let max_cats: u32 = T::MaxCategories::get().into();
            Self::calculate_actual_weight(T::WeightInfo::sell_complete_set, assets_len, max_cats)
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The base amount of currency that must be bonded for a market approved by the
        ///  advisory committee.
        #[pallet::constant]
        type AdvisoryBond: Get<BalanceOf<Self>>;

        /// The percentage of the advisory bond that gets slashed when a market is rejected.
        #[pallet::constant]
        type AdvisoryBondSlashPercentage: Get<Percent>;

        /// The origin that is allowed to approve / reject pending advised markets.
        type ApproveOrigin: EnsureOrigin<Self::Origin>;

        /// Shares of outcome assets and native currency
        type AssetManager: ZeitgeistAssetManager<
            Self::AccountId,
            CurrencyId = Asset<MarketIdOf<Self>>,
            ReserveIdentifier = [u8; 8],
        >;

        /// See [`zrml_authorized::AuthorizedPalletApi`].
        type Authorized: zrml_authorized::AuthorizedPalletApi<
            AccountId = Self::AccountId,
            Balance = BalanceOf<Self>,
            BlockNumber = Self::BlockNumber,
            MarketId = MarketIdOf<Self>,
            Moment = MomentOf<Self>,
            Origin = Self::Origin,
        >;

        /// The origin that is allowed to close markets.
        type CloseOrigin: EnsureOrigin<Self::Origin>;

        /// See [`zrml_court::CourtPalletApi`].
        type Court: zrml_court::CourtPalletApi<
            AccountId = Self::AccountId,
            Balance = BalanceOf<Self>,
            BlockNumber = Self::BlockNumber,
            MarketId = MarketIdOf<Self>,
            Moment = MomentOf<Self>,
            Origin = Self::Origin,
        >;

        /// The origin that is allowed to destroy markets.
        type DestroyOrigin: EnsureOrigin<Self::Origin>;

        /// The base amount of currency that must be bonded in order to create a dispute.
        #[pallet::constant]
        type DisputeBond: Get<BalanceOf<Self>>;

        /// The additional amount of currency that must be bonded when creating a subsequent
        /// dispute.
        #[pallet::constant]
        type DisputeFactor: Get<BalanceOf<Self>>;

        /// The number of blocks the dispute period remains open.
        #[pallet::constant]
        type DisputePeriod: Get<Self::BlockNumber>;

        /// Event
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type LiquidityMining: LiquidityMiningPalletApi<
            AccountId = Self::AccountId,
            Balance = BalanceOf<Self>,
            BlockNumber = Self::BlockNumber,
            MarketId = MarketIdOf<Self>,
        >;

        /// Common market parameters
        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = Self::BlockNumber,
        >;

        /// The maximum number of categories available for categorical markets.
        #[pallet::constant]
        type MaxCategories: Get<u16>;

        /// The shortest period of collecting subsidy for a Rikiddo market.
        #[pallet::constant]
        type MaxSubsidyPeriod: Get<MomentOf<Self>>;

        /// The minimum number of categories available for categorical markets.
        #[pallet::constant]
        type MinCategories: Get<u16>;

        /// The shortest period of collecting subsidy for a Rikiddo market.
        #[pallet::constant]
        type MinSubsidyPeriod: Get<MomentOf<Self>>;

        /// The maximum number of disputes allowed on any single market.
        #[pallet::constant]
        type MaxDisputes: Get<u32>;

        /// The maximum allowed timepoint for the market period (timestamp or blocknumber).
        type MaxMarketPeriod: Get<u64>;

        /// The module identifier.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The origin that is allowed to reject pending advised markets.
        type RejectOrigin: EnsureOrigin<Self::Origin>;

        /// The base amount of currency that must be bonded to ensure the oracle reports
        ///  in a timely manner.
        #[pallet::constant]
        type OracleBond: Get<BalanceOf<Self>>;

        /// The number of blocks the reporting period remains open.
        #[pallet::constant]
        type ReportingPeriod: Get<u32>;

        /// The origin that is allowed to resolve markets.
        type ResolveOrigin: EnsureOrigin<Self::Origin>;

        /// See [`DisputeApi`].
        type SimpleDisputes: DisputeApi<
            AccountId = Self::AccountId,
            Balance = BalanceOf<Self>,
            BlockNumber = Self::BlockNumber,
            MarketId = MarketIdOf<Self>,
            Moment = MomentOf<Self>,
            Origin = Self::Origin,
        >;

        /// Swaps pallet API
        type Swaps: Swaps<Self::AccountId, Balance = BalanceOf<Self>, MarketId = MarketIdOf<Self>>;

        /// The base amount of currency that must be bonded for a permissionless market,
        /// guaranteeing that it will resolve as anything but `Invalid`.
        #[pallet::constant]
        type ValidityBond: Get<BalanceOf<Self>>;

        /// Weights generated by benchmarks
        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Someone is trying to call `dispute` with the same outcome that is currently
        /// registered on-chain.
        CannotDisputeSameOutcome,
        /// Market account does not have enough funds to pay out.
        InsufficientFundsInMarketAccount,
        /// Sender does not have enough share balance.
        InsufficientShareBalance,
        /// An invalid Hash was included in a multihash parameter.
        InvalidMultihash,
        /// An invalid market type was found.
        InvalidMarketType,
        /// An operation is requested that is unsupported for the given scoring rule.
        InvalidScoringRule,
        /// Sender does not have enough balance to buy shares.
        NotEnoughBalance,
        /// Market is already reported on.
        MarketAlreadyReported,
        /// Market was expected to be active.
        MarketIsNotActive,
        /// Market was expected to be closed.
        MarketIsNotClosed,
        /// A market in subsidy collection phase was expected.
        MarketIsNotCollectingSubsidy,
        /// A proposed market was expected.
        MarketIsNotProposed,
        /// A reported market was expected.
        MarketIsNotReported,
        /// A resolved market was expected.
        MarketIsNotResolved,
        /// The point in time when the market becomes active is too soon.
        MarketStartTooSoon,
        /// The point in time when the market becomes active is too late.
        MarketStartTooLate,
        /// The maximum number of disputes has been reached.
        MaxDisputesReached,
        /// The number of categories for a categorical market is too low.
        NotEnoughCategories,
        /// The user has no winning balance.
        NoWinningBalance,
        /// Submitted outcome does not match market type.
        OutcomeMismatch,
        /// The report is not coming from designated oracle.
        ReporterNotOracle,
        /// It was tried to append an item to storage beyond the boundaries.
        StorageOverflow,
        /// Too many categories for a categorical market.
        TooManyCategories,
        /// Catch-all error for invalid market status
        InvalidMarketStatus,
        /// An amount was illegally specified as zero.
        ZeroAmount,
        /// Market period is faulty (too short, outside of limits)
        InvalidMarketPeriod,
        /// The outcome range of the scalar market is invalid.
        InvalidOutcomeRange,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// Custom addition block initialization logic wasn't successful
        BadOnInitialize,
        /// A complete set of assets has been bought \[market_id, amount_per_asset, buyer\]
        BoughtCompleteSet(MarketIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::AccountId),
        /// A market has been approved \[market_id, new_market_status\]
        MarketApproved(MarketIdOf<T>, MarketStatus),
        /// A market has been created \[market_id, market_account, creator\]
        MarketCreated(
            MarketIdOf<T>,
            T::AccountId,
            Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
        ),
        /// A market has been destroyed. \[market_id\]
        MarketDestroyed(MarketIdOf<T>),
        /// A market was started after gathering enough subsidy. \[market_id, new_market_status\]
        MarketStartedWithSubsidy(MarketIdOf<T>, MarketStatus),
        /// A market was discarded after failing to gather enough subsidy. \[market_id, new_market_status\]
        MarketInsufficientSubsidy(MarketIdOf<T>, MarketStatus),
        /// A market has been closed \[market_id\]
        MarketClosed(MarketIdOf<T>),
        /// A market has been disputed \[market_id, new_market_status, new_outcome\]
        MarketDisputed(MarketIdOf<T>, MarketStatus, MarketDispute<T::AccountId, T::BlockNumber>),
        /// An advised market has ended before it was approved or rejected. \[market_id\]
        MarketExpired(MarketIdOf<T>),
        /// A pending market has been rejected as invalid. \[market_id\]
        MarketRejected(MarketIdOf<T>),
        /// A market has been reported on \[market_id, new_market_status, reported_outcome\]
        MarketReported(MarketIdOf<T>, MarketStatus, Report<T::AccountId, T::BlockNumber>),
        /// A market has been resolved \[market_id, new_market_status, real_outcome\]
        MarketResolved(MarketIdOf<T>, MarketStatus, OutcomeReport),
        /// A complete set of assets has been sold \[market_id, amount_per_asset, seller\]
        SoldCompleteSet(MarketIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::AccountId),
        /// An amount of winning outcomes have been redeemed \[market_id, currency_id, amount_redeemed, payout, who\]
        TokensRedeemed(
            MarketIdOf<T>,
            Asset<MarketIdOf<T>>,
            BalanceOf<T>,
            BalanceOf<T>,
            <T as frame_system::Config>::AccountId,
        ),
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_initialize(now: T::BlockNumber) -> Weight {
            let mut total_weight: Weight =
                Self::process_subsidy_collecting_markets(now, T::MarketCommons::now());

            // If we are at genesis or the first block the timestamp is be undefined. No
            // market needs to be opened or closed on blocks #0 or #1, so we skip the
            // evaluation. Without this check, new chains starting from genesis will hang up,
            // since the loops in the `market_status_manager` calls below will run over an interval
            // of 0 to the current time frame.
            if now <= 1u32.into() {
                return total_weight;
            }

            // We add one to the count, because `pallet-timestamp` sets the timestamp _after_
            // `on_initialize` is called, so calling `now()` during `on_initialize` gives us
            // the timestamp of the previous block.
            let current_time_frame =
                Self::calculate_time_frame_of_moment(T::MarketCommons::now()).saturating_add(1);

            // On first pass, we use current_time - 1 to ensure that the chain doesn't try to
            // check all time frames since epoch.
            let last_time_frame =
                LastTimeFrame::<T>::get().unwrap_or_else(|| current_time_frame.saturating_sub(1));

            let _ = with_transaction(|| {
                let open = Self::market_status_manager::<
                    _,
                    MarketIdsPerOpenBlock<T>,
                    MarketIdsPerOpenTimeFrame<T>,
                >(
                    now,
                    last_time_frame,
                    current_time_frame,
                    |market_id, _| {
                        let weight = Self::open_market(market_id)?;
                        total_weight = total_weight.saturating_add(weight);
                        Ok(())
                    },
                );

                let close = Self::market_status_manager::<
                    _,
                    MarketIdsPerCloseBlock<T>,
                    MarketIdsPerCloseTimeFrame<T>,
                >(
                    now,
                    last_time_frame,
                    current_time_frame,
                    |market_id, market| {
                        let weight = Self::on_market_close(market_id, market)?;
                        total_weight = total_weight.saturating_add(weight);
                        Ok(())
                    },
                );

                let resolve = Self::resolution_manager(now, |market_id, market| {
                    let weight = Self::on_resolution(market_id, market)?;
                    total_weight = total_weight.saturating_add(weight);
                    Ok(())
                });

                LastTimeFrame::<T>::set(Some(current_time_frame));

                match open.and(close).and(resolve) {
                    Err(err) => {
                        Self::deposit_event(Event::BadOnInitialize);
                        log::error!("Block {:?} was not initialized. Error: {:?}", now, err);
                        TransactionOutcome::Rollback(err.into())
                    }
                    Ok(_) => TransactionOutcome::Commit(Ok(())),
                }
            });

            total_weight
        }
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    /// For each market, this holds the dispute information for each dispute that's
    /// been issued.
    #[pallet::storage]
    pub type Disputes<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        MarketIdOf<T>,
        BoundedVec<MarketDispute<T::AccountId, T::BlockNumber>, T::MaxDisputes>,
        ValueQuery,
    >;

    #[pallet::storage]
    pub type MarketIdsPerOpenBlock<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::BlockNumber,
        BoundedVec<MarketIdOf<T>, CacheSize>,
        ValueQuery,
    >;

    #[pallet::storage]
    pub type MarketIdsPerOpenTimeFrame<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        TimeFrame,
        BoundedVec<MarketIdOf<T>, CacheSize>,
        ValueQuery,
    >;

    /// A mapping of market identifiers to the block their market ends on.
    #[pallet::storage]
    pub type MarketIdsPerCloseBlock<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::BlockNumber,
        BoundedVec<MarketIdOf<T>, CacheSize>,
        ValueQuery,
    >;

    /// A mapping of market identifiers to the time frame their market ends in.
    #[pallet::storage]
    pub type MarketIdsPerCloseTimeFrame<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        TimeFrame,
        BoundedVec<MarketIdOf<T>, CacheSize>,
        ValueQuery,
    >;

    /// The last time frame that was checked for markets to close.
    #[pallet::storage]
    pub type LastTimeFrame<T: Config> = StorageValue<_, TimeFrame>;

    /// A mapping of market identifiers to the block they were disputed at.
    /// A market only ends up here if it was disputed.
    #[pallet::storage]
    pub type MarketIdsPerDisputeBlock<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::BlockNumber,
        BoundedVec<MarketIdOf<T>, CacheSize>,
        ValueQuery,
    >;

    /// A mapping of market identifiers to the block that they were reported on.
    #[pallet::storage]
    pub type MarketIdsPerReportBlock<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::BlockNumber,
        BoundedVec<MarketIdOf<T>, CacheSize>,
        ValueQuery,
    >;

    /// Contains a list of all markets that are currently collecting subsidy and the deadline.
    // All the values are "cached" here. Results in data duplication, but speeds up the iteration
    // over every market significantly (otherwise 25µs per relevant market per block).
    #[pallet::storage]
    pub type MarketsCollectingSubsidy<T: Config> = StorageValue<
        _,
        BoundedVec<SubsidyUntil<T::BlockNumber, MomentOf<T>, MarketIdOf<T>>, ConstU32<1_048_576>>,
        ValueQuery,
    >;

    impl<T: Config> Pallet<T> {
        pub fn outcome_assets(
            market_id: MarketIdOf<T>,
            market: &Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
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

        #[inline]
        pub(crate) fn market_account(market_id: MarketIdOf<T>) -> T::AccountId {
            T::PalletId::get().into_sub_account_truncating(market_id.saturated_into::<u128>())
        }

        // Manually remove market from cache for auto close.
        fn clear_auto_close(market_id: &MarketIdOf<T>) -> DispatchResult {
            let market = T::MarketCommons::market(market_id)?;

            // No-op if market isn't cached for auto close according to its state.
            match market.status {
                MarketStatus::Active | MarketStatus::Proposed => (),
                _ => return Ok(()),
            };

            match market.period {
                MarketPeriod::Block(range) => {
                    MarketIdsPerCloseBlock::<T>::mutate(range.end, |ids| {
                        remove_item::<MarketIdOf<T>, _>(ids, market_id);
                    });
                }
                MarketPeriod::Timestamp(range) => {
                    let time_frame = Self::calculate_time_frame_of_moment(range.end);
                    MarketIdsPerCloseTimeFrame::<T>::mutate(time_frame, |ids| {
                        remove_item::<MarketIdOf<T>, _>(ids, market_id);
                    });
                }
            };
            Ok(())
        }

        // Manually remove market from cache for auto open.
        fn clear_auto_open(market_id: &MarketIdOf<T>) -> DispatchResult {
            let market = T::MarketCommons::market(market_id)?;

            // No-op if market isn't cached for auto open according to its state.
            match market.status {
                MarketStatus::Active | MarketStatus::Proposed => (),
                _ => return Ok(()),
            };

            match market.period {
                MarketPeriod::Block(range) => {
                    MarketIdsPerOpenBlock::<T>::mutate(range.start, |ids| {
                        remove_item::<MarketIdOf<T>, _>(ids, market_id);
                    });
                }
                MarketPeriod::Timestamp(range) => {
                    let time_frame = Self::calculate_time_frame_of_moment(range.start);
                    MarketIdsPerOpenTimeFrame::<T>::mutate(time_frame, |ids| {
                        remove_item::<MarketIdOf<T>, _>(ids, market_id);
                    });
                }
            };
            Ok(())
        }

        /// Clears this market from being stored for automatic resolution.
        fn clear_auto_resolve(market_id: &MarketIdOf<T>) -> DispatchResult {
            let market = T::MarketCommons::market(market_id)?;
            if market.status == MarketStatus::Reported {
                let report = market.report.ok_or(Error::<T>::MarketIsNotReported)?;
                MarketIdsPerReportBlock::<T>::mutate(report.at, |ids| {
                    remove_item::<MarketIdOf<T>, _>(ids, market_id);
                });
            }
            if market.status == MarketStatus::Disputed {
                let disputes = Disputes::<T>::get(market_id);
                if let Some(last_dispute) = disputes.last() {
                    let at = last_dispute.at;
                    let mut old_disputes_per_block = MarketIdsPerDisputeBlock::<T>::get(at);
                    remove_item::<MarketIdOf<T>, _>(&mut old_disputes_per_block, market_id);
                    MarketIdsPerDisputeBlock::<T>::mutate(at, |ids| {
                        remove_item::<MarketIdOf<T>, _>(ids, market_id);
                    });
                }
            }

            Ok(())
        }

        pub(crate) fn do_buy_complete_set(
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure!(amount != BalanceOf::<T>::zero(), Error::<T>::ZeroAmount);
            ensure!(
                T::AssetManager::free_balance(Asset::Ztg, &who) >= amount,
                Error::<T>::NotEnoughBalance
            );

            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.scoring_rule == ScoringRule::CPMM, Error::<T>::InvalidScoringRule);
            Self::ensure_market_is_active(&market)?;

            let market_account = Self::market_account(market_id);
            T::AssetManager::transfer(Asset::Ztg, &who, &market_account, amount)?;

            let assets = Self::outcome_assets(market_id, &market);
            for asset in assets.iter() {
                T::AssetManager::deposit(*asset, &who, amount)?;
            }

            Self::deposit_event(Event::BoughtCompleteSet(market_id, amount, who));

            let assets_len: u32 = assets.len().saturated_into();
            let max_cats: u32 = T::MaxCategories::get().into();
            Self::calculate_actual_weight(T::WeightInfo::buy_complete_set, assets_len, max_cats)
        }

        pub(crate) fn do_reject_market(
            market_id: &MarketIdOf<T>,
            market: Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
        ) -> Result<Weight, DispatchError> {
            ensure!(market.status == MarketStatus::Proposed, Error::<T>::InvalidMarketStatus);
            let creator = &market.creator;
            let advisory_bond_slash_amount =
                T::AdvisoryBondSlashPercentage::get().mul_floor(T::AdvisoryBond::get());
            let advisory_bond_unreserve_amount =
                T::AdvisoryBond::get().saturating_sub(advisory_bond_slash_amount);
            T::AssetManager::slash_reserved_named(
                &Self::reserve_id(),
                Asset::Ztg,
                creator,
                advisory_bond_slash_amount,
            );
            T::AssetManager::unreserve_named(
                &Self::reserve_id(),
                Asset::Ztg,
                creator,
                T::OracleBond::get().saturating_add(advisory_bond_unreserve_amount),
            );
            T::MarketCommons::remove_market(market_id)?;
            Self::deposit_event(Event::MarketRejected(*market_id));
            Self::deposit_event(Event::MarketDestroyed(*market_id));
            Ok(T::WeightInfo::do_reject_market())
        }

        pub(crate) fn handle_expired_advised_market(
            market_id: &MarketIdOf<T>,
            market: Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
        ) -> Result<Weight, DispatchError> {
            ensure!(market.status == MarketStatus::Proposed, Error::<T>::InvalidMarketStatus);
            let creator = &market.creator;
            T::AssetManager::unreserve_named(
                &Self::reserve_id(),
                Asset::Ztg,
                creator,
                T::AdvisoryBond::get(),
            );
            T::AssetManager::unreserve_named(
                &Self::reserve_id(),
                Asset::Ztg,
                creator,
                T::OracleBond::get(),
            );
            T::MarketCommons::remove_market(market_id)?;
            Self::deposit_event(Event::MarketExpired(*market_id));
            Ok(T::WeightInfo::handle_expired_advised_market())
        }

        pub(crate) fn calculate_time_frame_of_moment(time: MomentOf<T>) -> TimeFrame {
            time.saturated_into::<TimeFrame>().saturating_div(MILLISECS_PER_BLOCK.into())
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

        fn calculate_internal_resolve_weight(
            market: &Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
            total_accounts: u32,
            total_asset_accounts: u32,
            total_categories: u32,
            total_disputes: u32,
        ) -> Weight {
            if let MarketType::Categorical(_) = market.market_type {
                if let MarketStatus::Reported = market.status {
                    T::WeightInfo::internal_resolve_categorical_reported(
                        total_accounts,
                        total_asset_accounts,
                        total_categories,
                    )
                } else {
                    T::WeightInfo::internal_resolve_categorical_disputed(
                        total_accounts,
                        total_asset_accounts,
                        total_categories,
                        total_disputes,
                    )
                }
            } else if let MarketStatus::Reported = market.status {
                T::WeightInfo::internal_resolve_scalar_reported()
            } else {
                T::WeightInfo::internal_resolve_scalar_disputed(total_disputes)
            }
        }

        fn ensure_can_not_dispute_the_same_outcome(
            disputes: &[MarketDispute<T::AccountId, T::BlockNumber>],
            report: &Report<T::AccountId, T::BlockNumber>,
            outcome: &OutcomeReport,
        ) -> DispatchResult {
            if let Some(last_dispute) = disputes.last() {
                ensure!(&last_dispute.outcome != outcome, Error::<T>::CannotDisputeSameOutcome);
            } else {
                ensure!(&report.outcome != outcome, Error::<T>::CannotDisputeSameOutcome);
            }
            Ok(())
        }

        #[inline]
        fn ensure_disputes_does_not_exceed_max_disputes(num_disputes: u32) -> DispatchResult {
            ensure!(num_disputes < T::MaxDisputes::get(), Error::<T>::MaxDisputesReached);
            Ok(())
        }

        fn ensure_market_is_active(
            market: &Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
        ) -> DispatchResult {
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketIsNotActive);
            Ok(())
        }

        fn ensure_market_period_is_valid(
            period: &MarketPeriod<T::BlockNumber, MomentOf<T>>,
        ) -> DispatchResult {
            // The start of the market is allowed to be in the past (this results in the market
            // being active immediately), but the market's end must be at least one block/time
            // frame in the future.
            match period {
                MarketPeriod::Block(ref range) => {
                    ensure!(
                        <frame_system::Pallet<T>>::block_number() < range.end,
                        Error::<T>::InvalidMarketPeriod
                    );
                    ensure!(range.start < range.end, Error::<T>::InvalidMarketPeriod);
                    ensure!(
                        range.end <= T::MaxMarketPeriod::get().saturated_into(),
                        Error::<T>::InvalidMarketPeriod
                    );
                }
                MarketPeriod::Timestamp(ref range) => {
                    // Ensure that the market lasts at least one time frame into the future.
                    let now_frame = Self::calculate_time_frame_of_moment(T::MarketCommons::now());
                    let end_frame = Self::calculate_time_frame_of_moment(range.end);
                    ensure!(now_frame < end_frame, Error::<T>::InvalidMarketPeriod);
                    ensure!(range.start < range.end, Error::<T>::InvalidMarketPeriod);
                    ensure!(
                        range.end <= T::MaxMarketPeriod::get().saturated_into(),
                        Error::<T>::InvalidMarketPeriod
                    );
                }
            };
            Ok(())
        }

        // Check that the market has reached the end of its period.
        fn ensure_market_is_closed(
            market: &Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
        ) -> DispatchResult {
            ensure!(market.status == MarketStatus::Closed, Error::<T>::MarketIsNotClosed);
            Ok(())
        }

        fn ensure_market_start_is_in_time(
            period: &MarketPeriod<T::BlockNumber, MomentOf<T>>,
        ) -> DispatchResult {
            let interval = match period {
                MarketPeriod::Block(range) => {
                    let interval_blocks: u128 = range
                        .start
                        .saturating_sub(<frame_system::Pallet<T>>::block_number())
                        .saturated_into();
                    interval_blocks.saturating_mul(MILLISECS_PER_BLOCK.into())
                }
                MarketPeriod::Timestamp(range) => {
                    range.start.saturating_sub(T::MarketCommons::now()).saturated_into()
                }
            };

            ensure!(
                <MomentOf<T>>::saturated_from(interval) >= T::MinSubsidyPeriod::get(),
                <Error<T>>::MarketStartTooSoon
            );
            ensure!(
                <MomentOf<T>>::saturated_from(interval) <= T::MaxSubsidyPeriod::get(),
                <Error<T>>::MarketStartTooLate
            );
            Ok(())
        }

        // If a market is categorical, destroys all non-winning assets.
        fn manage_resolved_categorical_market(
            market: &Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
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
                                T::AssetManager::accounts_by_currency_id(asset)
                                    .unwrap_or((0usize, vec![]));
                            total_asset_accounts =
                                total_asset_accounts.saturating_add(accounts.len());

                            let _ = T::AssetManager::destroy_all(asset, accounts.iter().cloned());
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

        pub(crate) fn open_market(market_id: &MarketIdOf<T>) -> Result<Weight, DispatchError> {
            // Is no-op if market has no pool. This should never happen, but it's safer to not
            // error in this case.
            let mut total_weight = T::DbWeight::get().reads(1); // (For the `market_pool` read)
            if let Ok(pool_id) = T::MarketCommons::market_pool(market_id) {
                let open_pool_weight = T::Swaps::open_pool(pool_id)?;
                total_weight = total_weight.saturating_add(open_pool_weight);
            }
            Ok(total_weight)
        }

        pub(crate) fn close_market(market_id: &MarketIdOf<T>) -> Result<Weight, DispatchError> {
            T::MarketCommons::mutate_market(market_id, |market| {
                ensure!(market.status == MarketStatus::Active, Error::<T>::InvalidMarketStatus);
                market.status = MarketStatus::Closed;
                Ok(())
            })?;
            let mut total_weight = T::DbWeight::get().reads_writes(1, 1);
            if let Ok(pool_id) = T::MarketCommons::market_pool(market_id) {
                let close_pool_weight = T::Swaps::close_pool(pool_id)?;
                total_weight = total_weight.saturating_add(close_pool_weight);
            };
            Self::deposit_event(Event::MarketClosed(*market_id));
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
            Ok(total_weight)
        }

        /// Handle market state transitions at the end of its active phase.
        fn on_market_close(
            market_id: &MarketIdOf<T>,
            market: Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
        ) -> Result<Weight, DispatchError> {
            match market.status {
                MarketStatus::Active => Self::close_market(market_id),
                MarketStatus::Proposed => Self::handle_expired_advised_market(market_id, market),
                _ => Err(Error::<T>::InvalidMarketStatus.into()), // Should never occur!
            }
        }

        fn on_resolution(
            market_id: &MarketIdOf<T>,
            market: &Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
        ) -> Result<u64, DispatchError> {
            if market.creation == MarketCreation::Permissionless {
                T::AssetManager::unreserve_named(
                    &Self::reserve_id(),
                    Asset::Ztg,
                    &market.creator,
                    T::ValidityBond::get(),
                );
            }

            let mut total_weight = 0;
            let disputes = Disputes::<T>::get(market_id);

            let report = market.report.as_ref().ok_or(Error::<T>::MarketIsNotReported)?;

            let resolved_outcome = match market.status {
                MarketStatus::Reported => {
                    // the oracle bond gets returned if the reporter was the oracle
                    if report.by == market.oracle {
                        T::AssetManager::unreserve_named(
                            &Self::reserve_id(),
                            Asset::Ztg,
                            &market.creator,
                            T::OracleBond::get(),
                        );
                    } else {
                        let excess = T::AssetManager::slash_reserved_named(
                            &Self::reserve_id(),
                            Asset::Ztg,
                            &market.creator,
                            T::OracleBond::get(),
                        );
                        // deposit only to the real reporter what actually was slashed
                        let negative_imbalance = T::OracleBond::get().saturating_sub(excess);

                        if let Err(err) =
                            T::AssetManager::deposit(Asset::Ztg, &report.by, negative_imbalance)
                        {
                            log::warn!(
                                "[PredictionMarkets] Cannot deposit to the reporter. error: {:?}",
                                err
                            );
                        }
                    }

                    report.outcome.clone()
                }
                MarketStatus::Disputed => {
                    // Try to get the outcome of the MDM. If the MDM failed to resolve, default to
                    // the oracle's report.
                    let resolved_outcome_option = match market.dispute_mechanism {
                        MarketDisputeMechanism::Authorized(_) => {
                            T::Authorized::on_resolution(&disputes, market_id, market)?
                        }
                        MarketDisputeMechanism::Court => {
                            T::Court::on_resolution(&disputes, market_id, market)?
                        }
                        MarketDisputeMechanism::SimpleDisputes => {
                            T::SimpleDisputes::on_resolution(&disputes, market_id, market)?
                        }
                    };
                    let resolved_outcome =
                        resolved_outcome_option.unwrap_or_else(|| report.outcome.clone());

                    let mut correct_reporters: Vec<T::AccountId> = Vec::new();

                    let mut overall_imbalance = BalanceOf::<T>::zero();

                    // If the oracle reported right, return the OracleBond, otherwise slash it to
                    // pay the correct reporters.
                    if report.outcome == resolved_outcome {
                        T::AssetManager::unreserve_named(
                            &Self::reserve_id(),
                            Asset::Ztg,
                            &market.creator,
                            T::OracleBond::get(),
                        );
                    } else {
                        let excess = T::AssetManager::slash_reserved_named(
                            &Self::reserve_id(),
                            Asset::Ztg,
                            &market.creator,
                            T::OracleBond::get(),
                        );

                        // negative_imbalance is the actual slash value (excess should be zero)
                        let negative_imbalance = T::OracleBond::get().saturating_sub(excess);
                        overall_imbalance = overall_imbalance.saturating_add(negative_imbalance);
                    }

                    for (i, dispute) in disputes.iter().enumerate() {
                        let actual_bond = default_dispute_bond::<T>(i);
                        if dispute.outcome == resolved_outcome {
                            T::AssetManager::unreserve_named(
                                &Self::reserve_id(),
                                Asset::Ztg,
                                &dispute.by,
                                actual_bond,
                            );

                            correct_reporters.push(dispute.by.clone());
                        } else {
                            let excess = T::AssetManager::slash_reserved_named(
                                &Self::reserve_id(),
                                Asset::Ztg,
                                &dispute.by,
                                actual_bond,
                            );

                            // negative_imbalance is the actual slash value (excess should be zero)
                            let negative_imbalance = actual_bond.saturating_sub(excess);
                            overall_imbalance =
                                overall_imbalance.saturating_add(negative_imbalance);
                        }
                    }

                    // Fold all the imbalances into one and reward the correct reporters. The
                    // number of correct reporters might be zero if the market defaults to the
                    // report after abandoned dispute. In that case, the rewards remain slashed.
                    if let Some(reward_per_each) =
                        overall_imbalance.checked_div(&correct_reporters.len().saturated_into())
                    {
                        for correct_reporter in &correct_reporters {
                            let reward = overall_imbalance.min(reward_per_each); // *Should* always be equal to `reward_per_each`
                            overall_imbalance = overall_imbalance.saturating_sub(reward);

                            if let Err(err) =
                                T::AssetManager::deposit(Asset::Ztg, correct_reporter, reward)
                            {
                                log::warn!(
                                    "[PredictionMarkets] Cannot deposit to the correct reporter. \
                                     error: {:?}",
                                    err
                                );
                            }
                        }
                    }

                    resolved_outcome
                }
                _ => return Err(Error::<T>::InvalidMarketStatus.into()),
            };
            let clean_up_weight = Self::clean_up_pool(market, market_id, &resolved_outcome)?;
            total_weight = total_weight.saturating_add(clean_up_weight);
            T::LiquidityMining::distribute_market_incentives(market_id)?;

            let mut total_accounts = 0u32;
            let mut total_asset_accounts = 0u32;
            let mut total_categories = 0u32;

            if let Ok([local_total_accounts, local_total_asset_accounts, local_total_categories]) =
                Self::manage_resolved_categorical_market(market, market_id, &resolved_outcome)
            {
                total_accounts = local_total_accounts.saturated_into();
                total_asset_accounts = local_total_asset_accounts.saturated_into();
                total_categories = local_total_categories.saturated_into();
            }

            T::MarketCommons::mutate_market(market_id, |m| {
                m.status = MarketStatus::Resolved;
                m.resolved_outcome = Some(resolved_outcome.clone());
                Ok(())
            })?;
            Disputes::<T>::remove(market_id);
            Self::deposit_event(Event::MarketResolved(
                *market_id,
                MarketStatus::Resolved,
                resolved_outcome,
            ));
            Ok(total_weight.saturating_add(Self::calculate_internal_resolve_weight(
                market,
                total_accounts,
                total_asset_accounts,
                total_categories,
                disputes.len().saturated_into(),
            )))
        }

        pub(crate) fn process_subsidy_collecting_markets(
            current_block: T::BlockNumber,
            current_time: MomentOf<T>,
        ) -> Weight {
            let mut total_weight = 0;
            let dbweight = T::DbWeight::get();
            let one_read = T::DbWeight::get().reads(1);
            let one_write = T::DbWeight::get().writes(1);

            let retain_closure = |subsidy_info: &SubsidyUntil<
                T::BlockNumber,
                MomentOf<T>,
                MarketIdOf<T>,
            >| {
                let market_ready = match &subsidy_info.period {
                    MarketPeriod::Block(period) => period.start <= current_block,
                    MarketPeriod::Timestamp(period) => period.start <= current_time,
                };

                if market_ready {
                    let pool_id = T::MarketCommons::market_pool(&subsidy_info.market_id);
                    total_weight.saturating_add(one_read);

                    if let Ok(pool_id) = pool_id {
                        let end_subsidy_result = T::Swaps::end_subsidy_phase(pool_id);

                        if let Ok(result) = end_subsidy_result {
                            total_weight = total_weight.saturating_add(result.weight);

                            if result.result {
                                // Sufficient subsidy, activate market.
                                let mutate_result =
                                    T::MarketCommons::mutate_market(&subsidy_info.market_id, |m| {
                                        m.status = MarketStatus::Active;
                                        Ok(())
                                    });

                                total_weight =
                                    total_weight.saturating_add(one_read).saturating_add(one_write);

                                if let Err(err) = mutate_result {
                                    log::error!(
                                        "[PredictionMarkets] Cannot find market associated to \
                                         market id.
                                    market_id: {:?}, error: {:?}",
                                        subsidy_info.market_id,
                                        err
                                    );
                                    return true;
                                }

                                Self::deposit_event(Event::MarketStartedWithSubsidy(
                                    subsidy_info.market_id,
                                    MarketStatus::Active,
                                ));
                            } else {
                                // Insufficient subsidy, cleanly remove pool and close market.
                                let destroy_result =
                                    T::Swaps::destroy_pool_in_subsidy_phase(pool_id);

                                if let Err(err) = destroy_result {
                                    log::error!(
                                        "[PredictionMarkets] Cannot destroy pool with missing \
                                         subsidy.
                                    market_id: {:?}, error: {:?}",
                                        subsidy_info.market_id,
                                        err
                                    );
                                    return true;
                                } else if let Ok(weight) = destroy_result {
                                    total_weight = total_weight.saturating_add(weight);
                                }

                                let market_result =
                                    T::MarketCommons::mutate_market(&subsidy_info.market_id, |m| {
                                        m.status = MarketStatus::InsufficientSubsidy;

                                        // Unreserve funds reserved during market creation
                                        if m.creation == MarketCreation::Permissionless {
                                            let required_bond = T::ValidityBond::get()
                                                .saturating_add(T::OracleBond::get());
                                            T::AssetManager::unreserve_named(
                                                &Self::reserve_id(),
                                                Asset::Ztg,
                                                &m.creator,
                                                required_bond,
                                            );
                                        } else if m.creation == MarketCreation::Advised {
                                            // AdvisoryBond was already returned when the market
                                            // was approved. Approval is inevitable to reach this.
                                            T::AssetManager::unreserve_named(
                                                &Self::reserve_id(),
                                                Asset::Ztg,
                                                &m.creator,
                                                T::OracleBond::get(),
                                            );
                                        }

                                        total_weight = total_weight
                                            .saturating_add(dbweight.reads(2))
                                            .saturating_add(dbweight.writes(2));
                                        Ok(())
                                    });

                                if let Err(err) = market_result {
                                    log::error!(
                                        "[PredictionMarkets] Cannot find market associated to \
                                         market id.
                                    market_id: {:?}, error: {:?}",
                                        subsidy_info.market_id,
                                        err
                                    );
                                    return true;
                                }

                                // `remove_market_pool` can only error due to missing pool, but
                                // above we ensured that the pool exists.
                                let _ =
                                    T::MarketCommons::remove_market_pool(&subsidy_info.market_id);
                                total_weight =
                                    total_weight.saturating_add(one_read).saturating_add(one_write);
                                Self::deposit_event(Event::MarketInsufficientSubsidy(
                                    subsidy_info.market_id,
                                    MarketStatus::InsufficientSubsidy,
                                ));
                            }

                            return false;
                        } else if let Err(err) = end_subsidy_result {
                            log::error!(
                                "[PredictionMarkets] An error occured during end of subsidy phase.
                        pool_id: {:?}, market_id: {:?}, error: {:?}",
                                pool_id,
                                subsidy_info.market_id,
                                err
                            );
                        }
                    } else if let Err(err) = pool_id {
                        log::error!(
                            "[PredictionMarkets] Cannot find pool associated to market.
                            market_id: {:?}, error: {:?}",
                            subsidy_info.market_id,
                            err
                        );
                        return true;
                    }
                }

                true
            };

            let mut weight_basis = 0;
            <MarketsCollectingSubsidy<T>>::mutate(
                |e: &mut BoundedVec<
                    SubsidyUntil<T::BlockNumber, MomentOf<T>, MarketIdOf<T>>,
                    _,
                >| {
                    weight_basis = T::WeightInfo::process_subsidy_collecting_markets_raw(
                        e.len().saturated_into(),
                    );
                    e.retain(retain_closure);
                },
            );

            weight_basis.saturating_add(total_weight)
        }

        fn remove_last_dispute_from_market_ids_per_dispute_block(
            disputes: &[MarketDispute<T::AccountId, T::BlockNumber>],
            market_id: &MarketIdOf<T>,
        ) -> DispatchResult {
            if let Some(last_dispute) = disputes.last() {
                let at = last_dispute.at;
                MarketIdsPerDisputeBlock::<T>::mutate(at, |ids| {
                    remove_item::<MarketIdOf<T>, _>(ids, market_id);
                });
            }
            Ok(())
        }

        /// The reserve ID of the prediction-markets pallet.
        #[inline]
        pub fn reserve_id() -> [u8; 8] {
            T::PalletId::get().0
        }

        pub(crate) fn market_status_manager<F, MarketIdsPerBlock, MarketIdsPerTimeFrame>(
            block_number: T::BlockNumber,
            last_time_frame: TimeFrame,
            current_time_frame: TimeFrame,
            mut mutation: F,
        ) -> DispatchResult
        where
            F: FnMut(
                &MarketIdOf<T>,
                Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
            ) -> DispatchResult,
            MarketIdsPerBlock: frame_support::StorageMap<
                T::BlockNumber,
                BoundedVec<MarketIdOf<T>, CacheSize>,
                Query = BoundedVec<MarketIdOf<T>, CacheSize>,
            >,
            MarketIdsPerTimeFrame: frame_support::StorageMap<
                TimeFrame,
                BoundedVec<MarketIdOf<T>, CacheSize>,
                Query = BoundedVec<MarketIdOf<T>, CacheSize>,
            >,
        {
            for market_id in MarketIdsPerBlock::get(block_number).iter() {
                let market = T::MarketCommons::market(market_id)?;
                mutation(market_id, market)?;
            }
            MarketIdsPerBlock::remove(block_number);

            for time_frame in last_time_frame.saturating_add(1)..=current_time_frame {
                for market_id in MarketIdsPerTimeFrame::get(time_frame).iter() {
                    let market = T::MarketCommons::market(market_id)?;
                    mutation(market_id, market)?;
                }
                MarketIdsPerTimeFrame::remove(time_frame);
            }

            Ok(())
        }

        fn resolution_manager<F>(now: T::BlockNumber, mut cb: F) -> DispatchResult
        where
            F: FnMut(
                &MarketIdOf<T>,
                &Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
            ) -> DispatchResult,
        {
            let dispute_period = T::DisputePeriod::get();
            if now <= dispute_period {
                return Ok(());
            }

            let block = now.saturating_sub(dispute_period);

            // Resolve all regularly reported markets.
            for id in MarketIdsPerReportBlock::<T>::get(block).iter() {
                let market = T::MarketCommons::market(id)?;
                if let MarketStatus::Reported = market.status {
                    cb(id, &market)?;
                }
            }
            MarketIdsPerReportBlock::<T>::remove(block);

            // Resolve any disputed markets.
            for id in MarketIdsPerDisputeBlock::<T>::get(block).iter() {
                let market = T::MarketCommons::market(id)?;
                cb(id, &market)?;
            }
            MarketIdsPerDisputeBlock::<T>::remove(block);

            Ok(())
        }

        // If the market is already disputed, does nothing.
        fn set_market_as_disputed(
            market: &Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
            market_id: &MarketIdOf<T>,
        ) -> DispatchResult {
            if market.status != MarketStatus::Disputed {
                T::MarketCommons::mutate_market(market_id, |m| {
                    m.status = MarketStatus::Disputed;
                    Ok(())
                })?;
            }
            Ok(())
        }

        // If a market has a pool that is `Active`, then changes from `Active` to `Clean`. If
        // the market does not exist or the market does not have a pool, does nothing.
        fn clean_up_pool(
            market: &Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
            market_id: &MarketIdOf<T>,
            outcome_report: &OutcomeReport,
        ) -> Result<Weight, DispatchError> {
            let pool_id = if let Ok(el) = T::MarketCommons::market_pool(market_id) {
                el
            } else {
                return Ok(T::DbWeight::get().reads(1));
            };
            let market_account = Self::market_account(*market_id);
            let weight = T::Swaps::clean_up_pool(
                &market.market_type,
                pool_id,
                outcome_report,
                &market_account,
            )?;
            Ok(weight.saturating_add(T::DbWeight::get().reads(2)))
        }

        // Creates a pool for the market and registers the market in the list of markets
        // currently collecting subsidy.
        pub(crate) fn start_subsidy(
            market: &Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
            market_id: MarketIdOf<T>,
        ) -> Result<Weight, DispatchError> {
            ensure!(
                market.status == MarketStatus::CollectingSubsidy,
                Error::<T>::MarketIsNotCollectingSubsidy
            );

            let mut assets = Self::outcome_assets(market_id, market);
            let base_asset = Asset::Ztg;
            assets.push(base_asset);
            let total_assets = assets.len();

            let pool_id = T::Swaps::create_pool(
                market.creator.clone(),
                assets,
                base_asset,
                market_id,
                market.scoring_rule,
                None,
                None,
                None,
            )?;

            // This errors if a pool already exists!
            T::MarketCommons::insert_market_pool(market_id, pool_id)?;
            <MarketsCollectingSubsidy<T>>::try_mutate(|markets| {
                markets
                    .try_push(SubsidyUntil { market_id, period: market.period.clone() })
                    .map_err(|_| <Error<T>>::StorageOverflow)
            })?;

            Ok(T::WeightInfo::start_subsidy(total_assets.saturated_into()))
        }

        fn validate_dispute(
            disputes: &[MarketDispute<T::AccountId, T::BlockNumber>],
            market: &Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
            num_disputes: u32,
            outcome_report: &OutcomeReport,
        ) -> DispatchResult {
            let report = market.report.as_ref().ok_or(Error::<T>::MarketIsNotReported)?;
            ensure!(market.matches_outcome_report(outcome_report), Error::<T>::OutcomeMismatch);
            Self::ensure_can_not_dispute_the_same_outcome(disputes, report, outcome_report)?;
            Self::ensure_disputes_does_not_exceed_max_disputes(num_disputes)?;
            Ok(())
        }
    }

    // No-one can bound more than BalanceOf<T>, therefore, this functions saturates
    pub fn default_dispute_bond<T>(n: usize) -> BalanceOf<T>
    where
        T: Config,
    {
        T::DisputeBond::get().saturating_add(
            T::DisputeFactor::get().saturating_mul(n.saturated_into::<u32>().into()),
        )
    }

    fn remove_item<I: cmp::PartialEq, G>(items: &mut BoundedVec<I, G>, item: &I) {
        if let Some(pos) = items.iter().position(|i| i == item) {
            items.swap_remove(pos);
        }
    }
}
