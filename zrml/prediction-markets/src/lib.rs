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
        traits::{
            Currency, EnsureOrigin, Get, Hooks, Imbalance, IsType, NamedReservableCurrency,
            OnUnbalanced, StorageVersion,
        },
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
            Asset, Deadlines, Market, MarketCreation, MarketDispute, MarketDisputeMechanism,
            MarketPeriod, MarketStatus, MarketType, MultiHash, OutcomeReport, Report,
            ScalarPosition, ScoringRule, SubsidyUntil,
        },
    };
    use zrml_liquidity_mining::LiquidityMiningPalletApi;
    use zrml_market_commons::MarketCommonsPalletApi;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(6);

    pub(crate) type BalanceOf<T> = <<T as Config>::AssetManager as MultiCurrency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;
    pub(crate) type CurrencyOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Currency;
    pub(crate) type NegativeImbalanceOf<T> =
        <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::NegativeImbalance;
    pub(crate) type TimeFrame = u64;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub(crate) type MomentOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
    pub type MarketOf<T> = Market<
        <T as frame_system::Config>::AccountId,
        <T as frame_system::Config>::BlockNumber,
        MomentOf<T>,
    >;
    pub type CacheSize = ConstU32<64>;
    pub type EditReason<T> = BoundedVec<u8, <T as Config>::MaxEditReasonLen>;
    pub type RejectReason<T> = BoundedVec<u8, <T as Config>::MaxRejectReasonLen>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Destroy a market, including its outcome assets, market account and pool account.
        ///
        /// Must be called by `DestroyOrigin`. Bonds (unless already returned) are slashed without
        /// exception. Can currently only be used for destroying CPMM markets.
        #[pallet::weight((
            T::WeightInfo::admin_destroy_reported_market(
                T::MaxCategories::get().into(),
                CacheSize::get(),
                CacheSize::get(),
                CacheSize::get(),
            )
            .max(T::WeightInfo::admin_destroy_disputed_market(
                T::MaxCategories::get().into(),
                T::MaxDisputes::get(),
                CacheSize::get(),
                CacheSize::get(),
                CacheSize::get(),
            )),
            Pays::No,
        ))]
        #[transactional]
        pub fn admin_destroy_market(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            // TODO(#618): Not implemented for Rikiddo!
            T::DestroyOrigin::ensure_origin(origin)?;

            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.scoring_rule == ScoringRule::CPMM, Error::<T>::InvalidScoringRule);
            let market_status = market.status;
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
                MarketIdsForEdit::<T>::remove(market_id);
            }
            if market_status != MarketStatus::Resolved
                && market_status != MarketStatus::InsufficientSubsidy
            {
                if market.creation == MarketCreation::Permissionless {
                    slash_market_creator(T::ValidityBond::get());
                }
                slash_market_creator(T::OracleBond::get());
            }

            // NOTE: Currently we don't clean up outcome assets.
            // TODO(#792): Remove outcome assets for accounts! Delete "resolved" assets of `orml_tokens` with storage migration.
            T::AssetManager::slash(
                Asset::Ztg,
                &market_account,
                T::AssetManager::free_balance(Asset::Ztg, &market_account),
            );
            let mut category_count = 0u32;
            if let Ok(pool_id) = T::MarketCommons::market_pool(&market_id) {
                let pool = T::Swaps::pool(pool_id)?;
                category_count = pool.assets.len().saturated_into();
                let _ = T::Swaps::destroy_pool(pool_id)?;
                T::MarketCommons::remove_market_pool(&market_id)?;
            }

            let open_ids_len = Self::clear_auto_open(&market_id)?;
            let close_ids_len = Self::clear_auto_close(&market_id)?;
            let (ids_len, disputes_len) = Self::clear_auto_resolve(&market_id)?;
            T::MarketCommons::remove_market(&market_id)?;
            Disputes::<T>::remove(market_id);

            Self::deposit_event(Event::MarketDestroyed(market_id));

            // Weight correction
            // The DestroyOrigin should not pay fees for providing this service
            if market_status == MarketStatus::Reported {
                Ok((
                    Some(T::WeightInfo::admin_destroy_reported_market(
                        category_count,
                        open_ids_len,
                        close_ids_len,
                        ids_len,
                    )),
                    Pays::No,
                )
                    .into())
            } else if market_status == MarketStatus::Disputed {
                Ok((
                    Some(T::WeightInfo::admin_destroy_disputed_market(
                        category_count,
                        disputes_len,
                        open_ids_len,
                        close_ids_len,
                        ids_len,
                    )),
                    Pays::No,
                )
                    .into())
            } else {
                Ok((None, Pays::No).into())
            }
        }

        /// Allows the `CloseOrigin` to immediately move an open market to closed.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n + m)`, where `n` is the number of market ids,
        /// which open at the same time as the specified market,
        /// and `m` is the number of market ids,
        /// which close at the same time as the specified market.
        //
        // ***** IMPORTANT *****
        //
        // Within the same block, operations that interact with the activeness of the same
        // market will behave differently before and after this call.
        #[pallet::weight((
            T::WeightInfo::admin_move_market_to_closed(
                CacheSize::get(), CacheSize::get()), Pays::No
            )
        )]
        #[transactional]
        pub fn admin_move_market_to_closed(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            // TODO(#638): Handle Rikiddo markets!
            T::CloseOrigin::ensure_origin(origin)?;
            let market = T::MarketCommons::market(&market_id)?;
            Self::ensure_market_is_active(&market)?;
            let open_ids_len = Self::clear_auto_open(&market_id)?;
            let close_ids_len = Self::clear_auto_close(&market_id)?;
            Self::close_market(&market_id)?;
            // The CloseOrigin should not pay fees for providing this service
            Ok((
                Some(T::WeightInfo::admin_move_market_to_closed(open_ids_len, close_ids_len)),
                Pays::No,
            )
                .into())
        }

        /// Allows the `ResolveOrigin` to immediately move a reported or disputed
        /// market to resolved.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n + m)`, where `n` is the number of market ids
        /// per dispute / report block, m is the number of disputes.
        #[pallet::weight((
            T::WeightInfo::admin_move_market_to_resolved_scalar_reported(CacheSize::get())
            .max(
                T::WeightInfo::admin_move_market_to_resolved_categorical_reported(CacheSize::get())
            ).max(
                T::WeightInfo::admin_move_market_to_resolved_scalar_disputed(
                    CacheSize::get(),
                    T::MaxDisputes::get()
                )
            ).max(
                T::WeightInfo::admin_move_market_to_resolved_categorical_disputed(
                    CacheSize::get(),
                    T::MaxDisputes::get()
                )
            ),
            Pays::No,
        ))]
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
            let (ids_len, disputes_len) = Self::clear_auto_resolve(&market_id)?;
            let market = T::MarketCommons::market(&market_id)?;
            let _ = Self::on_resolution(&market_id, &market)?;
            let weight = match market.market_type {
                MarketType::Scalar(_) => match market.status {
                    MarketStatus::Reported => {
                        T::WeightInfo::admin_move_market_to_resolved_scalar_reported(ids_len)
                    }
                    MarketStatus::Disputed => {
                        T::WeightInfo::admin_move_market_to_resolved_scalar_disputed(
                            ids_len,
                            disputes_len,
                        )
                    }
                    _ => return Err(Error::<T>::InvalidMarketStatus.into()),
                },
                MarketType::Categorical(_) => match market.status {
                    MarketStatus::Reported => {
                        T::WeightInfo::admin_move_market_to_resolved_categorical_reported(ids_len)
                    }
                    MarketStatus::Disputed => {
                        T::WeightInfo::admin_move_market_to_resolved_categorical_disputed(
                            ids_len,
                            disputes_len,
                        )
                    }
                    _ => return Err(Error::<T>::InvalidMarketStatus.into()),
                },
            };
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
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::weight((T::WeightInfo::approve_market(), Pays::No))]
        #[transactional]
        pub fn approve_market(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            // TODO(#787): Handle Rikiddo benchmarks!
            T::ApproveOrigin::ensure_origin(origin)?;
            let mut extra_weight = 0;
            let mut status = MarketStatus::Active;

            T::MarketCommons::mutate_market(&market_id, |m| {
                ensure!(m.status == MarketStatus::Proposed, Error::<T>::MarketIsNotProposed);
                ensure!(
                    !MarketIdsForEdit::<T>::contains_key(market_id),
                    Error::<T>::MarketEditRequestAlreadyInProgress
                );

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

        /// Request an edit to a proposed market.
        ///
        /// Can only be called by the `RequestEditOrigin`.
        ///
        /// # Arguments
        ///
        /// * `market_id`: The id of the market to edit.
        /// * `edit_reason`: An short record of what needs to be changed.
        ///
        /// # Weight
        ///
        /// Complexity: `O(edit_reason.len())`
        #[pallet::weight((
            T::WeightInfo::request_edit(edit_reason.len() as u32),
            Pays::No,
        ))]
        #[transactional]
        pub fn request_edit(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            edit_reason: Vec<u8>,
        ) -> DispatchResult {
            T::RequestEditOrigin::ensure_origin(origin)?;
            let edit_reason: EditReason<T> = edit_reason
                .try_into()
                .map_err(|_| Error::<T>::EditReasonLengthExceedsMaxEditReasonLen)?;
            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.status == MarketStatus::Proposed, Error::<T>::MarketIsNotProposed);
            MarketIdsForEdit::<T>::try_mutate(market_id, |reason| {
                if reason.is_some() {
                    Err(Error::<T>::MarketEditRequestAlreadyInProgress)
                } else {
                    *reason = Some(edit_reason.clone());
                    Ok(())
                }
            })?;
            Self::deposit_event(Event::MarketRequestedEdit(market_id, edit_reason));
            Ok(())
        }

        /// Buy a complete set of outcome shares of a market.
        ///
        /// The cost of a full set is exactly one unit of the market's base asset. For example,
        /// when calling `buy_complete_set(origin, 1, 2)` on a categorical market with five
        /// different outcomes, the caller pays `2` of the base asset and receives `2` of each of
        /// the five outcome tokens.
        ///
        /// NOTE: This is the only way to create new shares of outcome tokens.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of outcome assets in the market.
        // Note: `buy_complete_set` weight consumption is dependent on how many assets exists.
        // Unfortunately this information can only be retrieved with a storage call, therefore
        // The worst-case scenario is assumed
        // and the correct weight is calculated at the end of this function.
        // This also occurs in numerous other functions.
        #[pallet::weight(T::WeightInfo::buy_complete_set(T::MaxCategories::get().into()))]
        #[transactional]
        pub fn buy_complete_set(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            Self::do_buy_complete_set(sender, market_id, amount)
        }

        /// Dispute on a market that has been reported or already disputed.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of outstanding disputes.
        #[pallet::weight(T::WeightInfo::dispute_authorized(
            T::MaxDisputes::get(),
            CacheSize::get()
        ))]
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
                MarketDisputeMechanism::Authorized => {
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
            // each dispute resets dispute_duration
            let dispute_duration_ends_at_block =
                curr_block_num.saturating_add(market.deadlines.dispute_duration);
            <MarketIdsPerDisputeBlock<T>>::try_mutate(dispute_duration_ends_at_block, |ids| {
                ids.try_push(market_id).map_err(|_| <Error<T>>::StorageOverflow)
            })?;
            Self::deposit_event(Event::MarketDisputed(
                market_id,
                MarketStatus::Disputed,
                market_dispute,
            ));
            // TODO(#782): add court benchmark
            Ok((Some(T::WeightInfo::dispute_authorized(num_disputes, CacheSize::get()))).into())
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
        ///
        /// # Weight
        ///
        /// Complexity:
        /// - create_market: `O(n)`, where `n` is the number of market ids,
        /// which close at the same time as the specified market.
        /// - buy_complete_set: `O(n)`, where `n` is the number of outcome assets
        /// for the categorical market.
        /// - deploy_swap_pool_for_market_open_pool: `O(n)`,
        /// where n is the number of outcome assets for the categorical market.
        /// - deploy_swap_pool_for_market_future_pool: `O(n + m)`,
        /// where `n` is the number of outcome assets for the categorical market
        /// and `m` is the number of market ids,
        /// which open at the same time as the specified market.
        #[pallet::weight(
            T::WeightInfo::create_market(CacheSize::get())
            .saturating_add(T::WeightInfo::buy_complete_set(T::MaxCategories::get().into()))
            .saturating_add(
                T::WeightInfo::deploy_swap_pool_for_market_open_pool(weights.len() as u32)
                .max(T::WeightInfo::deploy_swap_pool_for_market_future_pool(
                    weights.len() as u32, CacheSize::get()
                )
            ))
        )]
        #[transactional]
        pub fn create_cpmm_market_and_deploy_assets(
            origin: OriginFor<T>,
            oracle: T::AccountId,
            period: MarketPeriod<T::BlockNumber, MomentOf<T>>,
            deadlines: Deadlines<T::BlockNumber>,
            metadata: MultiHash,
            market_type: MarketType,
            dispute_mechanism: MarketDisputeMechanism,
            #[pallet::compact] swap_fee: BalanceOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
            weights: Vec<u128>,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_signed(origin.clone())?;

            let create_market_weight = Self::create_market(
                origin.clone(),
                oracle,
                period,
                deadlines,
                metadata,
                MarketCreation::Permissionless,
                market_type.clone(),
                dispute_mechanism,
                ScoringRule::CPMM,
            )?
            .actual_weight
            .ok_or(Error::<T>::UnexpectedNoneInPostInfo)?;

            // Deploy the swap pool and populate it.
            let market_id = T::MarketCommons::latest_market_id()?;
            let deploy_and_populate_weight = Self::deploy_swap_pool_and_additional_liquidity(
                origin,
                market_id,
                swap_fee,
                amount,
                weights.clone(),
            )?
            .actual_weight
            .ok_or(Error::<T>::UnexpectedNoneInPostInfo)?;

            Ok(Some(create_market_weight.saturating_add(deploy_and_populate_weight)).into())
        }

        /// Creates a market.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of market ids,
        /// which close at the same time as the specified market.
        #[pallet::weight(T::WeightInfo::create_market(CacheSize::get()))]
        #[transactional]
        pub fn create_market(
            origin: OriginFor<T>,
            oracle: T::AccountId,
            period: MarketPeriod<T::BlockNumber, MomentOf<T>>,
            deadlines: Deadlines<T::BlockNumber>,
            metadata: MultiHash,
            creation: MarketCreation,
            market_type: MarketType,
            dispute_mechanism: MarketDisputeMechanism,
            scoring_rule: ScoringRule,
        ) -> DispatchResultWithPostInfo {
            // TODO(#787): Handle Rikiddo benchmarks!
            let sender = ensure_signed(origin)?;

            let market = Self::construct_market(
                sender.clone(),
                0_u8,
                oracle,
                period,
                deadlines,
                metadata,
                creation.clone(),
                market_type,
                dispute_mechanism,
                scoring_rule,
                None,
                None,
            )?;

            match creation {
                MarketCreation::Permissionless => {
                    let required_bond = T::ValidityBond::get().saturating_add(T::OracleBond::get());
                    T::AssetManager::reserve_named(
                        &Self::reserve_id(),
                        Asset::Ztg,
                        &sender,
                        required_bond,
                    )?;
                }
                MarketCreation::Advised => {
                    let required_bond = T::AdvisoryBond::get().saturating_add(T::OracleBond::get());
                    T::AssetManager::reserve_named(
                        &Self::reserve_id(),
                        Asset::Ztg,
                        &sender,
                        required_bond,
                    )?;
                }
            }

            let market_id = T::MarketCommons::push_market(market.clone())?;
            let market_account = Self::market_account(market_id);
            let mut extra_weight = 0;

            if market.status == MarketStatus::CollectingSubsidy {
                extra_weight = Self::start_subsidy(&market, market_id)?;
            }

            let ids_amount: u32 = Self::insert_auto_close(&market_id)?;

            Self::deposit_event(Event::MarketCreated(market_id, market_account, market));

            Ok(Some(T::WeightInfo::create_market(ids_amount).saturating_add(extra_weight)).into())
        }

        /// Edit a proposed market for which request is made.
        ///
        /// Edit can only be made by the creator of the market.
        ///
        /// # Arguments
        ///
        /// * `market_id`: The id of the market to edit.
        /// * `oracle`: Oracle to edit market.
        /// * `period`: MarketPeriod to edit market.
        /// * `deadlines`: Deadlines to edit market.
        /// * `metadata`: MultiHash metadata to edit market.
        /// * `market_type`: MarketType to edit market.
        /// * `dispute_mechanism`: MarketDisputeMechanism to edit market.
        /// * `scoring_rule`: ScoringRule to edit market.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of markets
        /// which end at the same time as the market before the edit.
        #[pallet::weight(T::WeightInfo::edit_market(CacheSize::get()))]
        #[transactional]
        pub fn edit_market(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
            oracle: T::AccountId,
            period: MarketPeriod<T::BlockNumber, MomentOf<T>>,
            deadlines: Deadlines<T::BlockNumber>,
            metadata: MultiHash,
            market_type: MarketType,
            dispute_mechanism: MarketDisputeMechanism,
            scoring_rule: ScoringRule,
        ) -> DispatchResultWithPostInfo {
            // TODO(#787): Handle Rikiddo benchmarks!
            let sender = ensure_signed(origin)?;
            ensure!(
                MarketIdsForEdit::<T>::contains_key(market_id),
                Error::<T>::MarketEditNotRequested
            );
            let old_market = T::MarketCommons::market(&market_id)?;
            ensure!(old_market.creator == sender, Error::<T>::EditorNotCreator);
            ensure!(old_market.status == MarketStatus::Proposed, Error::<T>::InvalidMarketStatus);

            Self::clear_auto_close(&market_id)?;
            let edited_market = Self::construct_market(
                old_market.creator,
                old_market.creator_fee,
                oracle,
                period,
                deadlines,
                metadata,
                old_market.creation,
                market_type,
                dispute_mechanism,
                scoring_rule,
                old_market.report,
                old_market.resolved_outcome,
            )?;
            T::MarketCommons::mutate_market(&market_id, |market| {
                *market = edited_market.clone();
                Ok(())
            })?;

            let ids_amount: u32 = Self::insert_auto_close(&market_id)?;

            MarketIdsForEdit::<T>::remove(market_id);
            Self::deposit_event(Event::MarketEdited(market_id, edited_market));

            Ok(Some(T::WeightInfo::edit_market(ids_amount)).into())
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
        ///
        /// # Weight
        ///
        /// Complexity:
        /// - buy_complete_set: `O(n)`,
        /// where `n` is the number of outcome assets for the categorical market.
        /// - deploy_swap_pool_for_market_open_pool: `O(n)`,
        /// where `n` is the number of outcome assets for the categorical market.
        /// - deploy_swap_pool_for_market_future_pool: `O(n + m)`,
        /// where `n` is the number of outcome assets for the categorical market,
        /// and `m` is the number of market ids,
        /// which open at the same time as the specified market.
        #[pallet::weight(
            T::WeightInfo::buy_complete_set(T::MaxCategories::get().into())
            .saturating_add(
                T::WeightInfo::deploy_swap_pool_for_market_open_pool(weights.len() as u32)
            .max(
                T::WeightInfo::deploy_swap_pool_for_market_future_pool(
                    weights.len() as u32, CacheSize::get()
                ))
            )
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
                .ok_or(Error::<T>::UnexpectedNoneInPostInfo)?;
            let weight_deploy =
                Self::deploy_swap_pool_for_market(origin, market_id, swap_fee, amount, weights)?
                    .actual_weight
                    .ok_or(Error::<T>::UnexpectedNoneInPostInfo)?;
            Ok(Some(weight_bcs.saturating_add(weight_deploy)).into())
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
        ///
        /// # Weight
        ///
        /// Complexity:
        /// - deploy_swap_pool_for_market_open_pool: `O(n)`,
        /// where `n` is the number of outcome assets for the categorical market.
        /// - deploy_swap_pool_for_market_future_pool: `O(n + m)`,
        /// where `n` is the number of outcome assets for the categorical market,
        /// and `m` is the number of market ids,
        /// which open at the same time as the specified market.
        #[pallet::weight(
            T::WeightInfo::deploy_swap_pool_for_market_open_pool(weights.len() as u32)
            .max(
                T::WeightInfo::deploy_swap_pool_for_market_future_pool(
                    weights.len() as u32, CacheSize::get()
                )
            )
        )]
        #[transactional]
        pub fn deploy_swap_pool_for_market(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] swap_fee: BalanceOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
            mut weights: Vec<u128>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let market = T::MarketCommons::market(&market_id)?;
            ensure!(market.scoring_rule == ScoringRule::CPMM, Error::<T>::InvalidScoringRule);
            Self::ensure_market_is_active(&market)?;

            let mut assets = Self::outcome_assets(market_id, &market);
            let weights_len = weights.len() as u32;
            // although this extrinsic is transactional and this check is inside Swaps::create_pool
            // the iteration over weights happens still before the check in Swaps::create_pool
            // this could stall the chain, because a malicious user puts a large vector in
            ensure!(weights.len() == assets.len(), Error::<T>::WeightsLenMustEqualAssetsLen);

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
            let ids_len: Option<u32> = match market.period {
                MarketPeriod::Block(ref range) => {
                    let current_block = <frame_system::Pallet<T>>::block_number();
                    let open_block = range.start;
                    if current_block < open_block {
                        let ids_len = MarketIdsPerOpenBlock::<T>::try_mutate(
                            open_block,
                            |ids| -> Result<u32, DispatchError> {
                                ids.try_push(market_id).map_err(|_| <Error<T>>::StorageOverflow)?;
                                Ok(ids.len() as u32)
                            },
                        )?;
                        Some(ids_len)
                    } else {
                        T::Swaps::open_pool(pool_id)?;
                        None
                    }
                }
                MarketPeriod::Timestamp(ref range) => {
                    let current_time_frame =
                        Self::calculate_time_frame_of_moment(T::MarketCommons::now());
                    let open_time_frame = Self::calculate_time_frame_of_moment(range.start);
                    if current_time_frame < open_time_frame {
                        let ids_len = MarketIdsPerOpenTimeFrame::<T>::try_mutate(
                            open_time_frame,
                            |ids| -> Result<u32, DispatchError> {
                                ids.try_push(market_id).map_err(|_| <Error<T>>::StorageOverflow)?;
                                Ok(ids.len() as u32)
                            },
                        )?;
                        Some(ids_len)
                    } else {
                        T::Swaps::open_pool(pool_id)?;
                        None
                    }
                }
            };

            // This errors if a pool already exists!
            T::MarketCommons::insert_market_pool(market_id, pool_id)?;
            match ids_len {
                Some(market_ids_len) => {
                    Ok(Some(T::WeightInfo::deploy_swap_pool_for_market_future_pool(
                        weights_len,
                        market_ids_len,
                    ))
                    .into())
                }
                None => {
                    Ok(Some(T::WeightInfo::deploy_swap_pool_for_market_open_pool(weights_len))
                        .into())
                }
            }
        }

        /// Redeems the winning shares of a prediction market.
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
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
        ///
        /// # Weight
        ///
        /// Complexity: `O(n + m)`,
        /// where `n` is the number of market ids,
        /// which open at the same time as the specified market,
        /// and `m` is the number of market ids,
        /// which close at the same time as the specified market.
        #[pallet::weight((
            T::WeightInfo::reject_market(
                CacheSize::get(),
                CacheSize::get(),
                reject_reason.len() as u32,
            ),
            Pays::No,
        ))]
        #[transactional]
        pub fn reject_market(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            reject_reason: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            T::RejectOrigin::ensure_origin(origin)?;
            let market = T::MarketCommons::market(&market_id)?;
            let open_ids_len = Self::clear_auto_open(&market_id)?;
            let close_ids_len = Self::clear_auto_close(&market_id)?;
            let reject_reason: RejectReason<T> = reject_reason
                .try_into()
                .map_err(|_| Error::<T>::RejectReasonLengthExceedsMaxRejectReasonLen)?;
            let reject_reason_len = reject_reason.len() as u32;
            Self::do_reject_market(&market_id, market, reject_reason)?;
            // The RejectOrigin should not pay fees for providing this service
            Ok((
                Some(T::WeightInfo::reject_market(close_ids_len, open_ids_len, reject_reason_len)),
                Pays::No,
            )
                .into())
        }

        /// Reports the outcome of a market.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of market ids,
        /// which reported at the same time as the specified market.
        #[pallet::weight(T::WeightInfo::report(CacheSize::get()))]
        #[transactional]
        pub fn report(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
        ) -> DispatchResultWithPostInfo {
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
                //NOTE: Saturating operation in following block may saturate to u32::MAX value
                //      but that will be the case after thousands of years time. So it is fine.
                match market.period {
                    MarketPeriod::Block(ref range) => {
                        let grace_period_end =
                            range.end.saturating_add(market.deadlines.grace_period);
                        ensure!(
                            grace_period_end <= current_block,
                            Error::<T>::NotAllowedToReportYet
                        );
                        let oracle_duration_end =
                            grace_period_end.saturating_add(market.deadlines.oracle_duration);
                        if current_block <= oracle_duration_end {
                            should_check_origin = true;
                        }
                    }
                    MarketPeriod::Timestamp(ref range) => {
                        let grace_period_in_moments: MomentOf<T> =
                            market.deadlines.grace_period.saturated_into::<u32>().into();
                        let grace_period_in_ms =
                            grace_period_in_moments.saturating_mul(MILLISECS_PER_BLOCK.into());
                        let grace_period_end = range.end.saturating_add(grace_period_in_ms);
                        let now = T::MarketCommons::now();
                        ensure!(grace_period_end <= now, Error::<T>::NotAllowedToReportYet);
                        let oracle_duration_in_moments: MomentOf<T> =
                            market.deadlines.oracle_duration.saturated_into::<u32>().into();
                        let oracle_duration_in_ms =
                            oracle_duration_in_moments.saturating_mul(MILLISECS_PER_BLOCK.into());
                        let oracle_duration_end =
                            grace_period_end.saturating_add(oracle_duration_in_ms);
                        if now <= oracle_duration_end {
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

            let market = T::MarketCommons::market(&market_id)?;
            let block_after_dispute_duration =
                current_block.saturating_add(market.deadlines.dispute_duration);
            let ids_len = MarketIdsPerReportBlock::<T>::try_mutate(
                block_after_dispute_duration,
                |ids| -> Result<u32, DispatchError> {
                    ids.try_push(market_id).map_err(|_| <Error<T>>::StorageOverflow)?;
                    Ok(ids.len() as u32)
                },
            )?;

            Self::deposit_event(Event::MarketReported(
                market_id,
                MarketStatus::Reported,
                market_report,
            ));
            Ok(Some(T::WeightInfo::report(ids_len)).into())
        }

        /// Sells a complete set of outcomes shares for a market.
        ///
        /// Each complete set is sold for one unit of the market's base asset.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of assets for a categorical market.
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
            Ok(Some(T::WeightInfo::sell_complete_set(assets_len)).into())
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

        /// The minimum number of blocks allowed to be specified as dispute_duration
        /// in create_market.
        #[pallet::constant]
        type MinDisputeDuration: Get<Self::BlockNumber>;

        /// The minimum number of blocks allowed to be specified as oracle_duration
        /// in create_market.
        #[pallet::constant]
        type MinOracleDuration: Get<Self::BlockNumber>;

        /// The maximum number of blocks allowed to be specified as grace_period
        /// in create_market.
        #[pallet::constant]
        type MaxGracePeriod: Get<Self::BlockNumber>;

        /// The maximum number of blocks allowed to be specified as oracle_duration
        /// in create_market.
        #[pallet::constant]
        type MaxOracleDuration: Get<Self::BlockNumber>;

        /// The maximum number of blocks allowed to be specified as dispute_duration
        /// in create_market.
        #[pallet::constant]
        type MaxDisputeDuration: Get<Self::BlockNumber>;

        /// The maximum length of reject reason string.
        #[pallet::constant]
        type MaxRejectReasonLen: Get<u32>;

        //NOTE: DisputePeriod will be removed once relevant migrations are executed.
        /// The number of blocks the dispute period remains open.
        #[pallet::constant]
        type DisputePeriod: Get<Self::BlockNumber>;

        /// The maximum allowed timepoint for the market period (timestamp or blocknumber).
        type MaxMarketPeriod: Get<u64>;

        /// The maximum number of bytes allowed as edit reason.
        #[pallet::constant]
        type MaxEditReasonLen: Get<u32>;

        /// The module identifier.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The origin that is allowed to reject pending advised markets.
        type RejectOrigin: EnsureOrigin<Self::Origin>;

        /// The base amount of currency that must be bonded to ensure the oracle reports
        ///  in a timely manner.
        #[pallet::constant]
        type OracleBond: Get<BalanceOf<Self>>;

        //NOTE: ReportingPeriod will be removed once relevant migrations are executed.
        /// The number of blocks the reporting period remains open.
        #[pallet::constant]
        type ReportingPeriod: Get<Self::BlockNumber>;

        /// The origin that is allowed to request edits in pending advised markets.
        type RequestEditOrigin: EnsureOrigin<Self::Origin>;

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

        /// Handler for slashed funds.
        type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

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
        /// Only creator is able to edit the market.
        EditorNotCreator,
        /// EditReason's length greater than MaxEditReasonLen.
        EditReasonLengthExceedsMaxEditReasonLen,
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
        /// Market edit request is already in progress.
        MarketEditRequestAlreadyInProgress,
        /// Market is not requested for edit.
        MarketEditNotRequested,
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
        /// A disputed market was expected.
        MarketIsNotDisputed,
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
        /// RejectReason's length greater than MaxRejectReasonLen.
        RejectReasonLengthExceedsMaxRejectReasonLen,
        /// The report is not coming from designated oracle.
        ReporterNotOracle,
        /// It was tried to append an item to storage beyond the boundaries.
        StorageOverflow,
        /// Too many categories for a categorical market.
        TooManyCategories,
        /// Catch-all error for invalid market status.
        InvalidMarketStatus,
        /// The post dispatch should never be None.
        UnexpectedNoneInPostInfo,
        /// An amount was illegally specified as zero.
        ZeroAmount,
        /// Market period is faulty (too short, outside of limits)
        InvalidMarketPeriod,
        /// The outcome range of the scalar market is invalid.
        InvalidOutcomeRange,
        /// Can not report before market.deadlines.grace_period is ended.
        NotAllowedToReportYet,
        /// Specified dispute_duration is smaller than MinDisputeDuration.
        DisputeDurationSmallerThanMinDisputeDuration,
        /// Specified oracle_duration is smaller than MinOracleDuration.
        OracleDurationSmallerThanMinOracleDuration,
        /// Specified dispute_duration is greater than MaxDisputeDuration.
        DisputeDurationGreaterThanMaxDisputeDuration,
        /// Specified grace_period is greater than MaxGracePeriod.
        GracePeriodGreaterThanMaxGracePeriod,
        /// Specified oracle_duration is greater than MaxOracleDuration.
        OracleDurationGreaterThanMaxOracleDuration,
        /// The weights length has to be equal to the assets length.
        WeightsLenMustEqualAssetsLen,
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
        /// A market has been created \[market_id, market_account, market\]
        MarketCreated(MarketIdOf<T>, T::AccountId, MarketOf<T>),
        /// A market has been destroyed. \[market_id\]
        MarketDestroyed(MarketIdOf<T>),
        /// A market was started after gathering enough subsidy. \[market_id, new_market_status\]
        MarketStartedWithSubsidy(MarketIdOf<T>, MarketStatus),
        /// A market was discarded after failing to gather enough subsidy.
        /// \[market_id, new_market_status\]
        MarketInsufficientSubsidy(MarketIdOf<T>, MarketStatus),
        /// A market has been closed \[market_id\]
        MarketClosed(MarketIdOf<T>),
        /// A market has been disputed \[market_id, new_market_status, new_outcome\]
        MarketDisputed(MarketIdOf<T>, MarketStatus, MarketDispute<T::AccountId, T::BlockNumber>),
        /// An advised market has ended before it was approved or rejected. \[market_id\]
        MarketExpired(MarketIdOf<T>),
        /// A pending market has been rejected as invalid with a reason. \[market_id, reject_reason\]
        MarketRejected(MarketIdOf<T>, RejectReason<T>),
        /// A market has been reported on \[market_id, new_market_status, reported_outcome\]
        MarketReported(MarketIdOf<T>, MarketStatus, Report<T::AccountId, T::BlockNumber>),
        /// A market has been resolved \[market_id, new_market_status, real_outcome\]
        MarketResolved(MarketIdOf<T>, MarketStatus, OutcomeReport),
        /// A proposed market has been requested edit by advisor. \[market_id, edit_reason\]
        MarketRequestedEdit(MarketIdOf<T>, EditReason<T>),
        /// A proposed market has been edited by the market creator \[market_id\]
        MarketEdited(MarketIdOf<T>, MarketOf<T>),
        /// A complete set of assets has been sold \[market_id, amount_per_asset, seller\]
        SoldCompleteSet(MarketIdOf<T>, BalanceOf<T>, <T as frame_system::Config>::AccountId),
        /// An amount of winning outcomes have been redeemed
        /// \[market_id, currency_id, amount_redeemed, payout, who\]
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
        // TODO(#792): Remove outcome assets for accounts! Delete "resolved" assets of `orml_tokens` with storage migration.
        fn on_initialize(now: T::BlockNumber) -> Weight {
            let mut total_weight: Weight = 0u64;

            // TODO(#808): Use weight when Rikiddo is ready
            let _ = Self::process_subsidy_collecting_markets(now, T::MarketCommons::now());
            total_weight = total_weight
                .saturating_add(T::WeightInfo::process_subsidy_collecting_markets_dummy());

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

                total_weight = total_weight.saturating_add(open.unwrap_or_else(|_| {
                    T::WeightInfo::market_status_manager(CacheSize::get(), CacheSize::get())
                }));

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

                if let Ok(weight) = close {
                    total_weight = total_weight.saturating_add(weight);
                } else {
                    // charge weight for the worst case
                    total_weight = total_weight.saturating_add(
                        T::WeightInfo::market_status_manager(CacheSize::get(), CacheSize::get()),
                    );
                }

                let resolve = Self::resolution_manager(now, |market_id, market| {
                    let weight = Self::on_resolution(market_id, market)?;
                    total_weight = total_weight.saturating_add(weight);
                    Ok(())
                });

                if let Ok(weight) = resolve {
                    total_weight = total_weight.saturating_add(weight);
                } else {
                    // charge weight for the worst case
                    total_weight =
                        total_weight.saturating_add(T::WeightInfo::market_resolution_manager(
                            CacheSize::get(),
                            CacheSize::get(),
                        ));
                }

                LastTimeFrame::<T>::set(Some(current_time_frame));
                total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

                match open.and(close).and(resolve) {
                    Err(err) => {
                        Self::deposit_event(Event::BadOnInitialize);
                        log::error!("Block {:?} was not initialized. Error: {:?}", now, err);
                        TransactionOutcome::Rollback(err.into())
                    }
                    Ok(_) => TransactionOutcome::Commit(Ok(())),
                }
            });

            total_weight.saturating_add(T::WeightInfo::on_initialize_resolve_overhead())
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

    /// Contains market_ids for which advisor has requested edit.
    /// Value for given market_id represents the reason for the edit.
    #[pallet::storage]
    pub type MarketIdsForEdit<T: Config> =
        StorageMap<_, Twox64Concat, MarketIdOf<T>, EditReason<T>>;

    impl<T: Config> Pallet<T> {
        pub fn outcome_assets(
            market_id: MarketIdOf<T>,
            market: &MarketOf<T>,
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

        fn insert_auto_close(market_id: &MarketIdOf<T>) -> Result<u32, DispatchError> {
            let market = T::MarketCommons::market(market_id)?;

            match market.period {
                MarketPeriod::Block(range) => MarketIdsPerCloseBlock::<T>::try_mutate(
                    range.end,
                    |ids| -> Result<u32, DispatchError> {
                        ids.try_push(*market_id).map_err(|_| <Error<T>>::StorageOverflow)?;
                        Ok(ids.len() as u32)
                    },
                ),
                MarketPeriod::Timestamp(range) => MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                    Self::calculate_time_frame_of_moment(range.end),
                    |ids| -> Result<u32, DispatchError> {
                        ids.try_push(*market_id).map_err(|_| <Error<T>>::StorageOverflow)?;
                        Ok(ids.len() as u32)
                    },
                ),
            }
        }

        // Manually remove market from cache for auto close.
        fn clear_auto_close(market_id: &MarketIdOf<T>) -> Result<u32, DispatchError> {
            let market = T::MarketCommons::market(market_id)?;

            // No-op if market isn't cached for auto close according to its state.
            match market.status {
                MarketStatus::Active | MarketStatus::Proposed => (),
                _ => return Ok(0u32),
            };

            let close_ids_len = match market.period {
                MarketPeriod::Block(range) => {
                    MarketIdsPerCloseBlock::<T>::mutate(range.end, |ids| -> u32 {
                        let ids_len = ids.len() as u32;
                        remove_item::<MarketIdOf<T>, _>(ids, market_id);
                        ids_len
                    })
                }
                MarketPeriod::Timestamp(range) => {
                    let time_frame = Self::calculate_time_frame_of_moment(range.end);
                    MarketIdsPerCloseTimeFrame::<T>::mutate(time_frame, |ids| -> u32 {
                        let ids_len = ids.len() as u32;
                        remove_item::<MarketIdOf<T>, _>(ids, market_id);
                        ids_len
                    })
                }
            };
            Ok(close_ids_len)
        }

        // Manually remove market from cache for auto open.
        fn clear_auto_open(market_id: &MarketIdOf<T>) -> Result<u32, DispatchError> {
            let market = T::MarketCommons::market(market_id)?;

            // No-op if market isn't cached for auto open according to its state.
            match market.status {
                MarketStatus::Active | MarketStatus::Proposed => (),
                _ => return Ok(0u32),
            };

            let open_ids_len = match market.period {
                MarketPeriod::Block(range) => {
                    MarketIdsPerOpenBlock::<T>::mutate(range.start, |ids| -> u32 {
                        let ids_len = ids.len() as u32;
                        remove_item::<MarketIdOf<T>, _>(ids, market_id);
                        ids_len
                    })
                }
                MarketPeriod::Timestamp(range) => {
                    let time_frame = Self::calculate_time_frame_of_moment(range.start);
                    MarketIdsPerOpenTimeFrame::<T>::mutate(time_frame, |ids| -> u32 {
                        let ids_len = ids.len() as u32;
                        remove_item::<MarketIdOf<T>, _>(ids, market_id);
                        ids_len
                    })
                }
            };
            Ok(open_ids_len)
        }

        /// Clears this market from being stored for automatic resolution.
        fn clear_auto_resolve(market_id: &MarketIdOf<T>) -> Result<(u32, u32), DispatchError> {
            let market = T::MarketCommons::market(market_id)?;
            let (ids_len, disputes_len) = match market.status {
                MarketStatus::Reported => {
                    let report = market.report.ok_or(Error::<T>::MarketIsNotReported)?;
                    let dispute_duration_ends_at_block =
                        report.at.saturating_add(market.deadlines.dispute_duration);
                    MarketIdsPerReportBlock::<T>::mutate(
                        dispute_duration_ends_at_block,
                        |ids| -> (u32, u32) {
                            let ids_len = ids.len() as u32;
                            remove_item::<MarketIdOf<T>, _>(ids, market_id);
                            (ids_len, 0u32)
                        },
                    )
                }
                MarketStatus::Disputed => {
                    let disputes = Disputes::<T>::get(market_id);
                    let last_dispute = disputes.last().ok_or(Error::<T>::MarketIsNotDisputed)?;
                    let dispute_duration_ends_at_block =
                        last_dispute.at.saturating_add(market.deadlines.dispute_duration);
                    MarketIdsPerDisputeBlock::<T>::mutate(
                        dispute_duration_ends_at_block,
                        |ids| -> (u32, u32) {
                            let ids_len = ids.len() as u32;
                            remove_item::<MarketIdOf<T>, _>(ids, market_id);
                            (ids_len, disputes.len() as u32)
                        },
                    )
                }
                _ => (0u32, 0u32),
            };

            Ok((ids_len, disputes_len))
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
            Ok(Some(T::WeightInfo::buy_complete_set(assets_len)).into())
        }

        pub(crate) fn do_reject_market(
            market_id: &MarketIdOf<T>,
            market: MarketOf<T>,
            reject_reason: RejectReason<T>,
        ) -> DispatchResult {
            ensure!(market.status == MarketStatus::Proposed, Error::<T>::InvalidMarketStatus);
            let creator = &market.creator;
            let advisory_bond_slash_amount =
                T::AdvisoryBondSlashPercentage::get().mul_floor(T::AdvisoryBond::get());
            let advisory_bond_unreserve_amount =
                T::AdvisoryBond::get().saturating_sub(advisory_bond_slash_amount);
            let (imbalance, _) = CurrencyOf::<T>::slash_reserved_named(
                &Self::reserve_id(),
                creator,
                advisory_bond_slash_amount.saturated_into::<u128>().saturated_into(),
            );
            T::Slash::on_unbalanced(imbalance);
            T::AssetManager::unreserve_named(
                &Self::reserve_id(),
                Asset::Ztg,
                creator,
                T::OracleBond::get().saturating_add(advisory_bond_unreserve_amount),
            );
            T::MarketCommons::remove_market(market_id)?;
            MarketIdsForEdit::<T>::remove(market_id);
            Self::deposit_event(Event::MarketRejected(*market_id, reject_reason));
            Self::deposit_event(Event::MarketDestroyed(*market_id));
            Ok(())
        }

        pub(crate) fn handle_expired_advised_market(
            market_id: &MarketIdOf<T>,
            market: MarketOf<T>,
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
            MarketIdsForEdit::<T>::remove(market_id);
            Self::deposit_event(Event::MarketExpired(*market_id));
            Ok(T::WeightInfo::handle_expired_advised_market())
        }

        pub(crate) fn calculate_time_frame_of_moment(time: MomentOf<T>) -> TimeFrame {
            time.saturated_into::<TimeFrame>().saturating_div(MILLISECS_PER_BLOCK.into())
        }

        fn calculate_internal_resolve_weight(market: &MarketOf<T>, total_disputes: u32) -> Weight {
            if let MarketType::Categorical(_) = market.market_type {
                if let MarketStatus::Reported = market.status {
                    T::WeightInfo::internal_resolve_categorical_reported()
                } else {
                    T::WeightInfo::internal_resolve_categorical_disputed(total_disputes)
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

        fn ensure_market_is_active(market: &MarketOf<T>) -> DispatchResult {
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

        fn ensure_market_deadlines_are_valid(
            deadlines: &Deadlines<T::BlockNumber>,
        ) -> DispatchResult {
            ensure!(
                deadlines.oracle_duration >= T::MinOracleDuration::get(),
                Error::<T>::OracleDurationSmallerThanMinOracleDuration
            );
            ensure!(
                deadlines.dispute_duration >= T::MinDisputeDuration::get(),
                Error::<T>::DisputeDurationSmallerThanMinDisputeDuration
            );
            ensure!(
                deadlines.dispute_duration <= T::MaxDisputeDuration::get(),
                Error::<T>::DisputeDurationGreaterThanMaxDisputeDuration
            );
            ensure!(
                deadlines.grace_period <= T::MaxGracePeriod::get(),
                Error::<T>::GracePeriodGreaterThanMaxGracePeriod
            );
            ensure!(
                deadlines.oracle_duration <= T::MaxOracleDuration::get(),
                Error::<T>::OracleDurationGreaterThanMaxOracleDuration
            );
            Ok(())
        }

        fn ensure_market_type_is_valid(market_type: &MarketType) -> DispatchResult {
            match market_type {
                MarketType::Categorical(categories) => {
                    ensure!(
                        *categories >= T::MinCategories::get(),
                        <Error<T>>::NotEnoughCategories
                    );
                    ensure!(*categories <= T::MaxCategories::get(), <Error<T>>::TooManyCategories);
                }
                MarketType::Scalar(ref outcome_range) => {
                    ensure!(
                        outcome_range.start() < outcome_range.end(),
                        <Error<T>>::InvalidOutcomeRange
                    );
                }
            }
            Ok(())
        }

        // Check that the market has reached the end of its period.
        fn ensure_market_is_closed(market: &MarketOf<T>) -> DispatchResult {
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
            market: MarketOf<T>,
        ) -> Result<Weight, DispatchError> {
            match market.status {
                MarketStatus::Active => Self::close_market(market_id),
                MarketStatus::Proposed => Self::handle_expired_advised_market(market_id, market),
                _ => Err(Error::<T>::InvalidMarketStatus.into()), // Should never occur!
            }
        }

        fn resolve_reported_market(market: &MarketOf<T>) -> Result<OutcomeReport, DispatchError> {
            let report = market.report.as_ref().ok_or(Error::<T>::MarketIsNotReported)?;
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

            Ok(report.outcome.clone())
        }

        fn resolve_disputed_market(
            market_id: &MarketIdOf<T>,
            market: &MarketOf<T>,
        ) -> Result<OutcomeReport, DispatchError> {
            let report = market.report.as_ref().ok_or(Error::<T>::MarketIsNotReported)?;
            let disputes = Disputes::<T>::get(market_id);
            // Try to get the outcome of the MDM. If the MDM failed to resolve, default to
            // the oracle's report.
            let resolved_outcome_option = match market.dispute_mechanism {
                MarketDisputeMechanism::Authorized => {
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

            // If the oracle reported right, return the OracleBond, otherwise slash it to
            // pay the correct reporters.
            let mut overall_imbalance = NegativeImbalanceOf::<T>::zero();
            if report.by == market.oracle && report.outcome == resolved_outcome {
                T::AssetManager::unreserve_named(
                    &Self::reserve_id(),
                    Asset::Ztg,
                    &market.creator,
                    T::OracleBond::get(),
                );
            } else {
                let (imbalance, _) = CurrencyOf::<T>::slash_reserved_named(
                    &Self::reserve_id(),
                    &market.creator,
                    T::OracleBond::get().saturated_into::<u128>().saturated_into(),
                );
                overall_imbalance.subsume(imbalance);
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
                    let (imbalance, _) = CurrencyOf::<T>::slash_reserved_named(
                        &Self::reserve_id(),
                        &dispute.by,
                        actual_bond.saturated_into::<u128>().saturated_into(),
                    );
                    overall_imbalance.subsume(imbalance);
                }
            }

            // Fold all the imbalances into one and reward the correct reporters. The
            // number of correct reporters might be zero if the market defaults to the
            // report after abandoned dispute. In that case, the rewards remain slashed.
            if let Some(reward_per_each) =
                overall_imbalance.peek().checked_div(&correct_reporters.len().saturated_into())
            {
                for correct_reporter in &correct_reporters {
                    let (actual_reward, leftover) = overall_imbalance.split(reward_per_each);
                    overall_imbalance = leftover;
                    CurrencyOf::<T>::resolve_creating(correct_reporter, actual_reward);
                }
            }
            T::Slash::on_unbalanced(overall_imbalance);

            Ok(resolved_outcome)
        }

        pub fn on_resolution(
            market_id: &MarketIdOf<T>,
            market: &MarketOf<T>,
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

            let resolved_outcome = match market.status {
                MarketStatus::Reported => Self::resolve_reported_market(market)?,
                MarketStatus::Disputed => Self::resolve_disputed_market(market_id, market)?,
                _ => return Err(Error::<T>::InvalidMarketStatus.into()),
            };
            let clean_up_weight = Self::clean_up_pool(market, market_id, &resolved_outcome)?;
            total_weight = total_weight.saturating_add(clean_up_weight);
            // TODO: https://github.com/zeitgeistpm/zeitgeist/issues/815
            // Following call should return weight consumed by it.
            T::LiquidityMining::distribute_market_incentives(market_id)?;

            // NOTE: Currently we don't clean up outcome assets.
            // TODO(#792): Remove outcome assets for accounts! Delete "resolved" assets of `orml_tokens` with storage migration.
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
                let market = T::MarketCommons::market(market_id)?;
                let dispute_duration_ends_at_block =
                    last_dispute.at.saturating_add(market.deadlines.dispute_duration);
                MarketIdsPerDisputeBlock::<T>::mutate(dispute_duration_ends_at_block, |ids| {
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
        ) -> Result<Weight, DispatchError>
        where
            F: FnMut(&MarketIdOf<T>, MarketOf<T>) -> DispatchResult,
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
            let market_ids_per_block = MarketIdsPerBlock::get(block_number);
            for market_id in market_ids_per_block.iter() {
                let market = T::MarketCommons::market(market_id)?;
                mutation(market_id, market)?;
            }
            MarketIdsPerBlock::remove(block_number);

            let mut time_frame_ids_len = 0u32;
            for time_frame in last_time_frame.saturating_add(1)..=current_time_frame {
                let market_ids_per_time_frame = MarketIdsPerTimeFrame::get(time_frame);
                time_frame_ids_len =
                    time_frame_ids_len.saturating_add(market_ids_per_time_frame.len() as u32);
                for market_id in market_ids_per_time_frame.iter() {
                    let market = T::MarketCommons::market(market_id)?;
                    mutation(market_id, market)?;
                }
                MarketIdsPerTimeFrame::remove(time_frame);
            }

            Ok(T::WeightInfo::market_status_manager(
                market_ids_per_block.len() as u32,
                time_frame_ids_len,
            ))
        }

        pub(crate) fn resolution_manager<F>(
            now: T::BlockNumber,
            mut cb: F,
        ) -> Result<Weight, DispatchError>
        where
            F: FnMut(&MarketIdOf<T>, &MarketOf<T>) -> DispatchResult,
        {
            // Resolve all regularly reported markets.
            let market_ids_per_report_block = MarketIdsPerReportBlock::<T>::get(now);
            for id in market_ids_per_report_block.iter() {
                let market = T::MarketCommons::market(id)?;
                if let MarketStatus::Reported = market.status {
                    cb(id, &market)?;
                }
            }
            MarketIdsPerReportBlock::<T>::remove(now);

            // Resolve any disputed markets.
            let market_ids_per_dispute_block = MarketIdsPerDisputeBlock::<T>::get(now);
            for id in market_ids_per_dispute_block.iter() {
                let market = T::MarketCommons::market(id)?;
                cb(id, &market)?;
            }
            MarketIdsPerDisputeBlock::<T>::remove(now);

            Ok(T::WeightInfo::market_resolution_manager(
                market_ids_per_report_block.len() as u32,
                market_ids_per_dispute_block.len() as u32,
            ))
        }

        // If the market is already disputed, does nothing.
        fn set_market_as_disputed(
            market: &MarketOf<T>,
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
            market: &MarketOf<T>,
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
            market: &MarketOf<T>,
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
            market: &MarketOf<T>,
            num_disputes: u32,
            outcome_report: &OutcomeReport,
        ) -> DispatchResult {
            let report = market.report.as_ref().ok_or(Error::<T>::MarketIsNotReported)?;
            ensure!(market.matches_outcome_report(outcome_report), Error::<T>::OutcomeMismatch);
            Self::ensure_can_not_dispute_the_same_outcome(disputes, report, outcome_report)?;
            Self::ensure_disputes_does_not_exceed_max_disputes(num_disputes)?;
            Ok(())
        }

        fn construct_market(
            creator: T::AccountId,
            creator_fee: u8,
            oracle: T::AccountId,
            period: MarketPeriod<T::BlockNumber, MomentOf<T>>,
            deadlines: Deadlines<T::BlockNumber>,
            metadata: MultiHash,
            creation: MarketCreation,
            market_type: MarketType,
            dispute_mechanism: MarketDisputeMechanism,
            scoring_rule: ScoringRule,
            report: Option<Report<T::AccountId, T::BlockNumber>>,
            resolved_outcome: Option<OutcomeReport>,
        ) -> Result<MarketOf<T>, DispatchError> {
            let MultiHash::Sha3_384(multihash) = metadata;
            ensure!(multihash[0] == 0x15 && multihash[1] == 0x30, <Error<T>>::InvalidMultihash);
            Self::ensure_market_period_is_valid(&period)?;
            Self::ensure_market_deadlines_are_valid(&deadlines)?;
            Self::ensure_market_type_is_valid(&market_type)?;

            if scoring_rule == ScoringRule::RikiddoSigmoidFeeMarketEma {
                Self::ensure_market_start_is_in_time(&period)?;
            }
            let status: MarketStatus = match creation {
                MarketCreation::Permissionless => match scoring_rule {
                    ScoringRule::CPMM => MarketStatus::Active,
                    ScoringRule::RikiddoSigmoidFeeMarketEma => MarketStatus::CollectingSubsidy,
                },
                MarketCreation::Advised => MarketStatus::Proposed,
            };
            Ok(Market {
                creation,
                creator_fee,
                creator,
                market_type,
                dispute_mechanism,
                metadata: Vec::from(multihash),
                oracle,
                period,
                deadlines,
                report,
                resolved_outcome,
                status,
                scoring_rule,
            })
        }
    }

    // No-one can bound more than BalanceOf<T>, therefore, this functions saturates
    pub(crate) fn default_dispute_bond<T>(n: usize) -> BalanceOf<T>
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
