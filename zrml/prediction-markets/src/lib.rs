// Copyright 2022-2024 Forecasting Technologies LTD.
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
#![recursion_limit = "256"]

extern crate alloc;

mod benchmarks;
pub mod migrations;
pub mod mock;
pub mod orml_asset_registry;
mod tests;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::weights::*;
    use alloc::{format, vec, vec::Vec};
    use core::{cmp, marker::PhantomData};
    use frame_support::{
        dispatch::{DispatchResultWithPostInfo, Pays, Weight},
        ensure, log,
        pallet_prelude::{ConstU32, StorageMap, StorageValue, ValueQuery},
        require_transactional,
        storage::{with_transaction, TransactionOutcome},
        traits::{
            tokens::BalanceStatus, Currency, EnsureOrigin, Get, Hooks, Imbalance, IsType,
            NamedReservableCurrency, OnUnbalanced, StorageVersion,
        },
        transactional, Blake2_128Concat, BoundedVec, PalletId, Twox64Concat,
    };
    use frame_system::{ensure_signed, pallet_prelude::OriginFor};
    use sp_runtime::traits::AccountIdConversion;

    #[cfg(feature = "parachain")]
    use {orml_traits::asset_registry::Inspect, zeitgeist_primitives::types::CustomMetadata};

    use orml_traits::{MultiCurrency, NamedMultiReservableCurrency};
    use sp_arithmetic::per_things::{Perbill, Percent};
    use sp_runtime::{
        traits::{Saturating, Zero},
        DispatchError, DispatchResult, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        constants::MILLISECS_PER_BLOCK,
        traits::{
            CompleteSetOperationsApi, DeployPoolApi, DisputeApi, DisputeMaxWeightApi,
            DisputeResolutionApi, ZeitgeistAssetManager,
        },
        types::{
            Asset, Bond, Deadlines, EarlyClose, EarlyCloseState, GlobalDisputeItem, Market,
            MarketBonds, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus,
            MarketType, MultiHash, OutcomeReport, Report, ResultWithWeightInfo, ScalarPosition,
            ScoringRule,
        },
    };
    use zrml_global_disputes::{types::InitialItem, GlobalDisputesPalletApi};
    use zrml_liquidity_mining::LiquidityMiningPalletApi;
    use zrml_market_commons::MarketCommonsPalletApi;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(8);
    const LOG_TARGET: &str = "runtime::zrml-prediction-markets";
    /// The maximum number of blocks between the [`LastTimeFrame`]
    /// and the current timestamp in block number allowed to recover
    /// the automatic market openings and closings from a chain stall.
    /// Currently 10 blocks is 2 minutes (assuming block time is 12 seconds).
    pub(crate) const MAX_RECOVERY_TIME_FRAMES: TimeFrame = 10;

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type AssetOf<T> = Asset<MarketIdOf<T>>;
    pub(crate) type BalanceOf<T> = <T as zrml_market_commons::Config>::Balance;
    pub(crate) type CacheSize = ConstU32<64>;
    pub(crate) type EditReason<T> = BoundedVec<u8, <T as Config>::MaxEditReasonLen>;
    pub(crate) type InitialItemOf<T> = InitialItem<AccountIdOf<T>, BalanceOf<T>>;
    pub(crate) type MarketBondsOf<T> = MarketBonds<AccountIdOf<T>, BalanceOf<T>>;
    pub(crate) type MarketIdOf<T> = <T as zrml_market_commons::Config>::MarketId;
    pub(crate) type MarketOf<T> = Market<
        AccountIdOf<T>,
        BalanceOf<T>,
        <T as frame_system::Config>::BlockNumber,
        MomentOf<T>,
        AssetOf<T>,
    >;
    pub(crate) type MomentOf<T> =
        <<T as zrml_market_commons::Config>::Timestamp as frame_support::traits::Time>::Moment;
    pub(crate) type NegativeImbalanceOf<T> =
        <<T as Config>::Currency as Currency<AccountIdOf<T>>>::NegativeImbalance;
    pub(crate) type RejectReason<T> = BoundedVec<u8, <T as Config>::MaxRejectReasonLen>;
    pub(crate) type ReportOf<T> = Report<AccountIdOf<T>, <T as frame_system::Config>::BlockNumber>;
    pub(crate) type TimeFrame = u64;

    macro_rules! impl_unreserve_bond {
        ($fn_name:ident, $bond_type:ident) => {
            /// Settle the $bond_type bond by unreserving it.
            ///
            /// This function **should** only be called if the bond is not yet settled, and calling
            /// it if the bond is settled is most likely a logic error. If the bond is already
            /// settled, storage is not changed, a warning is raised and `Ok(())` is returned.
            fn $fn_name(market_id: &MarketIdOf<T>) -> DispatchResult {
                let market = <zrml_market_commons::Pallet<T>>::market(market_id)?;
                let bond = market.bonds.$bond_type.as_ref().ok_or(Error::<T>::MissingBond)?;
                if bond.is_settled {
                    let warning = format!(
                        "Attempting to settle the {} bond of market {:?} multiple times",
                        stringify!($bond_type),
                        market_id,
                    );
                    log::warn!(target: LOG_TARGET, "{}", warning);
                    debug_assert!(false, "{}", warning);
                    return Ok(());
                }
                let missing = T::Currency::unreserve_named(&Self::reserve_id(), &bond.who, bond.value);
                debug_assert!(
                    missing.is_zero(),
                    "Could not unreserve all of the amount. reserve_id: {:?}, who: {:?}, value: {:?}.",
                    &Self::reserve_id(),
                    &bond.who,
                    bond.value,
                );
                <zrml_market_commons::Pallet<T>>::mutate_market(market_id, |m| {
                    m.bonds.$bond_type = Some(Bond { is_settled: true, ..bond.clone() });
                    Ok(())
                })
            }
        };
    }

    macro_rules! impl_slash_bond {
        ($fn_name:ident, $bond_type:ident) => {
            /// Settle the $bond_type bond by slashing and/or unreserving it and return the
            /// resulting imbalance.
            ///
            /// If `slash_percentage` is not specified, then the entire bond is slashed. Otherwise,
            /// only the specified percentage is slashed and the remainder is unreserved.
            ///
            /// This function **should** only be called if the bond is not yet settled, and calling
            /// it if the bond is settled is most likely a logic error. If the bond is already
            /// settled, storage is not changed, a warning is raised and a zero imbalance is
            /// returned.
            fn $fn_name(
                market_id: &MarketIdOf<T>,
                slash_percentage: Option<Percent>,
            ) -> Result<NegativeImbalanceOf<T>, DispatchError> {
                let market = <zrml_market_commons::Pallet<T>>::market(market_id)?;
                let bond = market.bonds.$bond_type.as_ref().ok_or(Error::<T>::MissingBond)?;
                // Trying to settle a bond multiple times is always a logic error, not a runtime
                // error, so we log a warning instead of raising an error.
                if bond.is_settled {
                    let warning = format!(
                        "Attempting to settle the {} bond of market {:?} multiple times",
                        stringify!($bond_type),
                        market_id,
                    );
                    log::warn!(target: LOG_TARGET, "{}", warning);
                    debug_assert!(false, "{}", warning);
                    return Ok(NegativeImbalanceOf::<T>::zero());
                }
                let value = bond.value;
                let (slash_amount, unreserve_amount) = if let Some(percentage) = slash_percentage {
                    let slash_amount = percentage.mul_floor(value);
                    (slash_amount, value.saturating_sub(slash_amount))
                } else {
                    (value, BalanceOf::<T>::zero())
                };
                let (imbalance, excess) =
                    T::Currency::slash_reserved_named(&Self::reserve_id(), &bond.who, slash_amount);
                // If there's excess, there's nothing we can do, so we don't count this as error
                // and log a warning instead.
                if excess != BalanceOf::<T>::zero() {
                    let warning = format!(
                        "Failed to settle the {} bond of market {:?}",
                        stringify!($bond_type),
                        market_id,
                    );
                    log::warn!(target: LOG_TARGET, "{}", warning);
                    debug_assert!(false, "{}", warning);
                }
                if unreserve_amount != BalanceOf::<T>::zero() {
                    let missing = T::Currency::unreserve_named(
                        &Self::reserve_id(),
                        &bond.who,
                        unreserve_amount,
                    );
                    debug_assert!(
                        missing.is_zero(),
                        "Could not unreserve all of the amount. reserve_id: {:?}, \
                         who: {:?}, amount: {:?}, missing: {:?}",
                        Self::reserve_id(),
                        &bond.who,
                        unreserve_amount,
                        missing,
                    );
                }
                <zrml_market_commons::Pallet<T>>::mutate_market(market_id, |m| {
                    m.bonds.$bond_type = Some(Bond { is_settled: true, ..bond.clone() });
                    Ok(())
                })?;
                Ok(imbalance)
            }
        };
    }

    macro_rules! impl_repatriate_bond {
        ($fn_name:ident, $bond_type:ident) => {
            /// Settle the $bond_type bond by repatriating it to free balance of beneficiary.
            ///
            /// This function **should** only be called if the bond is not yet settled, and calling
            /// it if the bond is settled is most likely a logic error. If the bond is already
            /// settled, storage is not changed, a warning is raised and `Ok(())` is returned.
            fn $fn_name(market_id: &MarketIdOf<T>, beneficiary: &T::AccountId) -> DispatchResult {
                let market = <zrml_market_commons::Pallet<T>>::market(market_id)?;
                let bond = market.bonds.$bond_type.as_ref().ok_or(Error::<T>::MissingBond)?;
                if bond.is_settled {
                    let warning = format!(
                        "Attempting to settle the {} bond of market {:?} multiple times",
                        stringify!($bond_type),
                        market_id,
                    );
                    log::warn!(target: LOG_TARGET, "{}", warning);
                    debug_assert!(false, "{}", warning);
                    return Ok(());
                }
                let res = T::Currency::repatriate_reserved_named(
                    &Self::reserve_id(),
                    &bond.who,
                    beneficiary,
                    bond.value,
                    BalanceStatus::Free,
                );
                // If there's an error or missing balance,
                // there's nothing we can do, so we don't count this as error
                // and log a warning instead.
                match res {
                    Ok(missing) if missing != BalanceOf::<T>::zero() => {
                        let warning = format!(
                            "Failed to repatriate all of the {} bond of market {:?} (missing \
                             balance {:?}).",
                            stringify!($bond_type),
                            market_id,
                            missing,
                        );
                        log::warn!(target: LOG_TARGET, "{}", warning);
                        debug_assert!(false, "{}", warning);
                    }
                    Ok(_) => (),
                    Err(_err) => {
                        let warning = format!(
                            "Failed to settle the {} bond of market {:?} (error: {}).",
                            stringify!($bond_type),
                            market_id,
                            stringify!(_err),
                        );
                        log::warn!(target: LOG_TARGET, "{}", warning);
                        debug_assert!(false, "{}", warning);
                    }
                }
                <zrml_market_commons::Pallet<T>>::mutate_market(market_id, |m| {
                    m.bonds.$bond_type = Some(Bond { is_settled: true, ..bond.clone() });
                    Ok(())
                })?;
                Ok(())
            }
        };
    }

    macro_rules! impl_is_bond_pending {
        ($fn_name:ident, $bond_type:ident) => {
            /// Check whether the $bond_type is present (ready to get unreserved or slashed).
            /// Set the flag `with_warning` to `true`, when warnings should be logged
            /// in case the bond is not present or already settled.
            ///
            /// Return `true` if the bond is present and not settled, `false` otherwise.
            #[allow(unused)]
            fn $fn_name(
                market_id: &MarketIdOf<T>,
                market: &MarketOf<T>,
                with_warning: bool,
            ) -> bool {
                if let Some(bond) = &market.bonds.$bond_type {
                    if !bond.is_settled {
                        return true;
                    } else if with_warning {
                        let warning = format!(
                            "[PredictionMarkets] The {} bond is already settled for market {:?}.",
                            stringify!($bond_type),
                            market_id,
                        );
                        log::warn!(target: LOG_TARGET, "{}", warning);
                        debug_assert!(false, "{}", warning);
                    }
                } else if with_warning {
                    let warning = format!(
                        "[PredictionMarkets] The {} bond is not present for market {:?}.",
                        stringify!($bond_type),
                        market_id,
                    );
                    log::warn!(target: LOG_TARGET, "{}", warning);
                    debug_assert!(false, "{}", warning);
                }

                false
            }
        };
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
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
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::admin_move_market_to_closed(CacheSize::get()))]
        #[transactional]
        pub fn admin_move_market_to_closed(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::CloseOrigin::ensure_origin(origin)?;
            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            Self::ensure_market_is_active(&market)?;
            let close_ids_len = Self::clear_auto_close(&market_id)?;
            Self::close_market(&market_id)?;
            Self::set_market_end(&market_id)?;
            // The CloseOrigin should not pay fees for providing this service
            Ok((Some(T::WeightInfo::admin_move_market_to_closed(close_ids_len)), Pays::No).into())
        }

        /// Allows the `ResolveOrigin` to immediately move a reported or disputed
        /// market to resolved.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n + m)`, where `n` is the number of market ids
        /// per dispute / report block, m is the number of disputes.
        #[pallet::call_index(2)]
        #[pallet::weight(
            T::WeightInfo::admin_move_market_to_resolved_scalar_reported(CacheSize::get())
            .max(
                T::WeightInfo::admin_move_market_to_resolved_categorical_reported(CacheSize::get())
            ).max(
                T::WeightInfo::admin_move_market_to_resolved_scalar_disputed(CacheSize::get())
            ).max(
                T::WeightInfo::admin_move_market_to_resolved_categorical_disputed(CacheSize::get())
            )
        )]
        #[transactional]
        pub fn admin_move_market_to_resolved(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::ResolveOrigin::ensure_origin(origin)?;

            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            ensure!(
                market.status == MarketStatus::Reported || market.status == MarketStatus::Disputed,
                Error::<T>::InvalidMarketStatus,
            );
            let (ids_len, _) = Self::clear_auto_resolve(&market_id)?;
            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            let _ = Self::on_resolution(&market_id, &market)?;
            let weight = match market.market_type {
                MarketType::Scalar(_) => match market.status {
                    MarketStatus::Reported => {
                        T::WeightInfo::admin_move_market_to_resolved_scalar_reported(ids_len)
                    }
                    MarketStatus::Disputed => {
                        T::WeightInfo::admin_move_market_to_resolved_scalar_disputed(ids_len)
                    }
                    _ => return Err(Error::<T>::InvalidMarketStatus.into()),
                },
                MarketType::Categorical(_) => match market.status {
                    MarketStatus::Reported => {
                        T::WeightInfo::admin_move_market_to_resolved_categorical_reported(ids_len)
                    }
                    MarketStatus::Disputed => {
                        T::WeightInfo::admin_move_market_to_resolved_categorical_disputed(ids_len)
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
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::approve_market())]
        #[transactional]
        pub fn approve_market(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::ApproveOrigin::ensure_origin(origin)?;
            let new_status = MarketStatus::Active;

            <zrml_market_commons::Pallet<T>>::mutate_market(&market_id, |m| {
                ensure!(m.status == MarketStatus::Proposed, Error::<T>::MarketIsNotProposed);
                ensure!(
                    !MarketIdsForEdit::<T>::contains_key(market_id),
                    Error::<T>::MarketEditRequestAlreadyInProgress
                );
                m.status = new_status;
                Ok(())
            })?;

            Self::unreserve_creation_bond(&market_id)?;

            Self::deposit_event(Event::MarketApproved(market_id, new_status));
            // The ApproveOrigin should not pay fees for providing this service
            let default_weight: Option<Weight> = None;
            Ok((default_weight, Pays::No).into())
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
        #[pallet::call_index(4)]
        #[pallet::weight(
            T::WeightInfo::request_edit(edit_reason.len() as u32)
        )]
        #[transactional]
        pub fn request_edit(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            edit_reason: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            T::RequestEditOrigin::ensure_origin(origin)?;
            let edit_reason: EditReason<T> = edit_reason
                .try_into()
                .map_err(|_| Error::<T>::EditReasonLengthExceedsMaxEditReasonLen)?;
            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
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
            let default_weight: Option<Weight> = None;
            Ok((default_weight, Pays::No).into())
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
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::buy_complete_set(T::MaxCategories::get().into()))]
        #[transactional]
        pub fn buy_complete_set(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            Self::do_buy_complete_set(sender, market_id, amount)?;
            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            let assets = Self::outcome_assets(market_id, &market);
            let assets_len: u32 = assets.len().saturated_into();
            Ok(Some(T::WeightInfo::buy_complete_set(assets_len)).into())
        }

        /// Dispute on a market that has been reported or already disputed.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of outstanding disputes.
        #[pallet::call_index(6)]
        #[pallet::weight(
            T::WeightInfo::dispute_authorized().saturating_add(
                T::Court::on_dispute_max_weight().saturating_add(
                    T::SimpleDisputes::on_dispute_max_weight()
                )
            )
        )]
        #[transactional]
        pub fn dispute(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            ensure!(market.status == MarketStatus::Reported, Error::<T>::InvalidMarketStatus);

            let dispute_mechanism =
                market.dispute_mechanism.as_ref().ok_or(Error::<T>::NoDisputeMechanism)?;
            let weight = match dispute_mechanism {
                MarketDisputeMechanism::Authorized => {
                    T::Authorized::on_dispute(&market_id, &market)?;
                    T::WeightInfo::dispute_authorized()
                }
                MarketDisputeMechanism::Court => {
                    let court_weight = T::Court::on_dispute(&market_id, &market)?.weight;
                    T::WeightInfo::dispute_authorized()
                        .saturating_sub(T::Authorized::on_dispute_max_weight())
                        .saturating_add(court_weight)
                }
                MarketDisputeMechanism::SimpleDisputes => {
                    let sd_weight = T::SimpleDisputes::on_dispute(&market_id, &market)?.weight;
                    T::WeightInfo::dispute_authorized()
                        .saturating_sub(T::Authorized::on_dispute_max_weight())
                        .saturating_add(sd_weight)
                }
            };

            let dispute_bond = T::DisputeBond::get();
            T::AssetManager::reserve_named(&Self::reserve_id(), Asset::Ztg, &who, dispute_bond)?;

            <zrml_market_commons::Pallet<T>>::mutate_market(&market_id, |m| {
                m.status = MarketStatus::Disputed;
                m.bonds.dispute = Some(Bond::new(who.clone(), dispute_bond));
                Ok(())
            })?;

            Self::deposit_event(Event::MarketDisputed(market_id, MarketStatus::Disputed, who));
            Ok((Some(weight)).into())
        }

        /// Creates a market.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of market ids,
        /// which close at the same time as the specified market.
        #[pallet::call_index(8)]
        #[pallet::weight(T::WeightInfo::create_market(CacheSize::get()))]
        #[transactional]
        pub fn create_market(
            origin: OriginFor<T>,
            base_asset: AssetOf<T>,
            creator_fee: Perbill,
            oracle: T::AccountId,
            period: MarketPeriod<T::BlockNumber, MomentOf<T>>,
            deadlines: Deadlines<T::BlockNumber>,
            metadata: MultiHash,
            creation: MarketCreation,
            market_type: MarketType,
            dispute_mechanism: Option<MarketDisputeMechanism>,
            scoring_rule: ScoringRule,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            let (ids_len, _) = Self::do_create_market(
                sender,
                base_asset,
                creator_fee,
                oracle,
                period,
                deadlines,
                metadata,
                creation,
                market_type,
                dispute_mechanism,
                scoring_rule,
            )?;
            Ok(Some(T::WeightInfo::create_market(ids_len)).into())
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
        #[pallet::call_index(9)]
        #[pallet::weight(T::WeightInfo::edit_market(CacheSize::get()))]
        #[transactional]
        pub fn edit_market(
            origin: OriginFor<T>,
            base_asset: AssetOf<T>,
            market_id: MarketIdOf<T>,
            oracle: T::AccountId,
            period: MarketPeriod<T::BlockNumber, MomentOf<T>>,
            deadlines: Deadlines<T::BlockNumber>,
            metadata: MultiHash,
            market_type: MarketType,
            dispute_mechanism: Option<MarketDisputeMechanism>,
            scoring_rule: ScoringRule,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            ensure!(
                MarketIdsForEdit::<T>::contains_key(market_id),
                Error::<T>::MarketEditNotRequested
            );
            let old_market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            ensure!(old_market.creator == sender, Error::<T>::EditorNotCreator);
            ensure!(old_market.status == MarketStatus::Proposed, Error::<T>::InvalidMarketStatus);

            Self::clear_auto_close(&market_id)?;
            let edited_market = Self::construct_market(
                base_asset,
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
                old_market.bonds,
            )?;
            <zrml_market_commons::Pallet<T>>::mutate_market(&market_id, |market| {
                *market = edited_market.clone();
                Ok(())
            })?;

            let ids_amount: u32 = Self::insert_auto_close(&market_id)?;

            MarketIdsForEdit::<T>::remove(market_id);
            Self::deposit_event(Event::MarketEdited(market_id, edited_market));

            Ok(Some(T::WeightInfo::edit_market(ids_amount)).into())
        }

        /// Redeems the winning shares of a prediction market.
        ///
        /// # Weight
        ///
        /// Complexity: `O(1)`
        #[pallet::call_index(12)]
        #[pallet::weight(T::WeightInfo::redeem_shares_categorical()
            .max(T::WeightInfo::redeem_shares_scalar())
        )]
        #[transactional]
        pub fn redeem_shares(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;

            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            let market_account = Self::market_account(market_id);

            ensure!(market.status == MarketStatus::Resolved, Error::<T>::MarketIsNotResolved);
            ensure!(market.is_redeemable(), Error::<T>::InvalidResolutionMechanism);

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
                        T::AssetManager::free_balance(market.base_asset, &market_account)
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
                        T::AssetManager::free_balance(market.base_asset, &market_account)
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
                let missing = T::AssetManager::slash(currency_id, &sender, balance);
                debug_assert!(
                    missing.is_zero(),
                    "Could not slash all of the amount. currency_id {:?}, sender: {:?}, balance: \
                     {:?}.",
                    currency_id,
                    &sender,
                    balance,
                );

                // Pay out the winner.
                let remaining_bal =
                    T::AssetManager::free_balance(market.base_asset, &market_account);
                let actual_payout = payout.min(remaining_bal);

                T::AssetManager::transfer(
                    market.base_asset,
                    &market_account,
                    &sender,
                    actual_payout,
                )?;
                // The if-check prevents scalar markets to emit events even if sender only owns one
                // of the outcome tokens.
                if balance != BalanceOf::<T>::zero() {
                    Self::deposit_event(Event::TokensRedeemed(
                        market_id,
                        currency_id,
                        balance,
                        actual_payout,
                        sender.clone(),
                    ));
                }
            }

            let weight = match resolved_outcome {
                OutcomeReport::Categorical(_) => T::WeightInfo::redeem_shares_categorical(),
                OutcomeReport::Scalar(_) => T::WeightInfo::redeem_shares_scalar(),
            };
            Ok(Some(weight).into())
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
        #[pallet::call_index(13)]
        #[pallet::weight(
            T::WeightInfo::reject_market(CacheSize::get(), reject_reason.len() as u32))]
        #[transactional]
        pub fn reject_market(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            reject_reason: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            T::RejectOrigin::ensure_origin(origin)?;
            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            let close_ids_len = Self::clear_auto_close(&market_id)?;
            let reject_reason: RejectReason<T> = reject_reason
                .try_into()
                .map_err(|_| Error::<T>::RejectReasonLengthExceedsMaxRejectReasonLen)?;
            let reject_reason_len = reject_reason.len() as u32;
            Self::do_reject_market(&market_id, market, reject_reason)?;
            // The RejectOrigin should not pay fees for providing this service
            Ok((Some(T::WeightInfo::reject_market(close_ids_len, reject_reason_len)), Pays::No)
                .into())
        }

        /// Reports the outcome of a market.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of market ids,
        /// which reported at the same time as the specified market.
        #[pallet::call_index(14)]
        #[pallet::weight(
            T::WeightInfo::report_market_with_dispute_mechanism(CacheSize::get())
                .max(T::WeightInfo::report_trusted_market())
        )]
        #[transactional]
        pub fn report(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            outcome: OutcomeReport,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin.clone())?;
            let current_block = <frame_system::Pallet<T>>::block_number();
            let market_report = Report { at: current_block, by: sender.clone(), outcome };
            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            ensure!(market.report.is_none(), Error::<T>::MarketAlreadyReported);
            Self::ensure_market_is_closed(&market)?;
            ensure!(
                market.matches_outcome_report(&market_report.outcome),
                Error::<T>::OutcomeMismatch
            );
            let weight = if market.dispute_mechanism.is_some() {
                Self::report_market_with_dispute_mechanism(
                    origin,
                    market_id,
                    market_report.clone(),
                )?
            } else {
                Self::report_and_resolve_market(origin, market_id, market_report.clone())?
            };
            Self::deposit_event(Event::MarketReported(
                market_id,
                MarketStatus::Reported,
                market_report,
            ));
            Ok(weight)
        }

        /// Sells a complete set of outcomes shares for a market.
        ///
        /// Each complete set is sold for one unit of the market's base asset.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the number of assets for a categorical market.
        #[pallet::call_index(15)]
        #[pallet::weight(T::WeightInfo::sell_complete_set(T::MaxCategories::get().into()))]
        #[transactional]
        pub fn sell_complete_set(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin)?;
            Self::do_sell_complete_set(sender, market_id, amount)?;
            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            let assets = Self::outcome_assets(market_id, &market);
            let assets_len: u32 = assets.len().saturated_into();
            Ok(Some(T::WeightInfo::sell_complete_set(assets_len)).into())
        }

        /// Start a global dispute, if the market dispute mechanism fails.
        ///
        /// # Arguments
        ///
        /// * `market_id`: The identifier of the market.
        ///
        /// NOTE:
        /// The returned outcomes of the market dispute mechanism and the report outcome
        /// are added to the global dispute voting outcomes.
        /// The bond of each dispute is the initial vote amount.
        #[pallet::call_index(16)]
        #[pallet::weight(T::WeightInfo::start_global_dispute(CacheSize::get(), CacheSize::get()))]
        #[transactional]
        pub fn start_global_dispute(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            let dispute_mechanism =
                market.dispute_mechanism.as_ref().ok_or(Error::<T>::NoDisputeMechanism)?;
            ensure!(
                matches!(market.status, MarketStatus::Disputed | MarketStatus::Reported),
                Error::<T>::InvalidMarketStatus
            );

            ensure!(
                matches!(dispute_mechanism, MarketDisputeMechanism::Court),
                Error::<T>::InvalidDisputeMechanism
            );

            ensure!(
                !T::GlobalDisputes::does_exist(&market_id),
                Error::<T>::GlobalDisputeExistsAlready
            );

            let report = market.report.as_ref().ok_or(Error::<T>::MarketIsNotReported)?;

            let res_0 = match dispute_mechanism {
                MarketDisputeMechanism::Authorized => {
                    T::Authorized::has_failed(&market_id, &market)?
                }
                MarketDisputeMechanism::Court => T::Court::has_failed(&market_id, &market)?,
                MarketDisputeMechanism::SimpleDisputes => {
                    T::SimpleDisputes::has_failed(&market_id, &market)?
                }
            };
            let has_failed = res_0.result;
            ensure!(has_failed, Error::<T>::MarketDisputeMechanismNotFailed);

            let res_1 = match dispute_mechanism {
                MarketDisputeMechanism::Authorized => {
                    T::Authorized::on_global_dispute(&market_id, &market)?
                }
                MarketDisputeMechanism::Court => T::Court::on_global_dispute(&market_id, &market)?,
                MarketDisputeMechanism::SimpleDisputes => {
                    T::SimpleDisputes::on_global_dispute(&market_id, &market)?
                }
            };

            let mut initial_items: Vec<InitialItemOf<T>> = Vec::new();

            initial_items.push(InitialItemOf::<T> {
                outcome: report.outcome.clone(),
                owner: report.by.clone(),
                amount: BalanceOf::<T>::zero(),
            });

            let gd_items = res_1.result;

            // push vote outcomes other than the report outcome
            for GlobalDisputeItem { outcome, owner, initial_vote_amount } in gd_items {
                initial_items.push(InitialItemOf::<T> {
                    outcome,
                    owner,
                    amount: initial_vote_amount,
                });
            }

            // ensure, that global disputes controls the resolution now
            // it does not end after the dispute period now, but after the global dispute end

            // ignore first of tuple because we always have max disputes
            let (_, ids_len_2) = Self::clear_auto_resolve(&market_id)?;

            if market.status == MarketStatus::Reported {
                // this is the case that a dispute can not be initiated,
                // because court has not enough juror and delegator stake (dispute errors)
                <zrml_market_commons::Pallet<T>>::mutate_market(&market_id, |m| {
                    m.status = MarketStatus::Disputed;
                    Ok(())
                })?;
            }

            // global disputes uses DisputeResolution API to control its resolution
            let ids_len_1 =
                T::GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice())?;

            Self::deposit_event(Event::GlobalDisputeStarted(market_id));

            Ok(Some(T::WeightInfo::start_global_dispute(ids_len_1, ids_len_2)).into())
        }

        /// Create a market, deploy a LMSR pool, and buy outcome tokens and provide liquidity to the
        /// market.
        ///
        /// # Weight
        ///
        /// `O(n)` where `n` is the number of markets which close on the same block, plus the
        /// resources consumed by `DeployPool::create_pool`. In the standard implementation using
        /// neo-swaps, this is `O(m)` where `m` is the number of assets in the market.
        #[pallet::weight(T::WeightInfo::create_market_and_deploy_pool(
            CacheSize::get(),
            spot_prices.len() as u32,
        ))]
        #[transactional]
        #[pallet::call_index(17)]
        pub fn create_market_and_deploy_pool(
            origin: OriginFor<T>,
            base_asset: AssetOf<T>,
            creator_fee: Perbill,
            oracle: T::AccountId,
            period: MarketPeriod<T::BlockNumber, MomentOf<T>>,
            deadlines: Deadlines<T::BlockNumber>,
            metadata: MultiHash,
            market_type: MarketType,
            dispute_mechanism: Option<MarketDisputeMechanism>,
            #[pallet::compact] amount: BalanceOf<T>,
            spot_prices: Vec<BalanceOf<T>>,
            #[pallet::compact] swap_fee: BalanceOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let (ids_len, market_id) = Self::do_create_market(
                who.clone(),
                base_asset,
                creator_fee,
                oracle,
                period,
                deadlines,
                metadata,
                MarketCreation::Permissionless,
                market_type,
                dispute_mechanism,
                ScoringRule::Lmsr,
            )?;
            Self::do_buy_complete_set(who.clone(), market_id, amount)?;
            let spot_prices_len = spot_prices.len() as u32;
            T::DeployPool::deploy_pool(who, market_id, amount, spot_prices, swap_fee)?;
            Ok(Some(T::WeightInfo::create_market_and_deploy_pool(ids_len, spot_prices_len)).into())
        }

        /// Allows the `CloseMarketsEarlyOrigin` or the market creator to schedule an early close.
        ///
        /// The market creator schedules it `now + EarlyClose...Period` in the future.
        /// This is to allow enough time for a potential dispute.
        /// The market creator reserves a `CloseEarlyDisputeBond`, which is returned,
        /// if the `CloseMarketsEarlyOrigin` decides to accept the early close request
        /// or if it is not disputed.
        /// It is slashed, if the early close request is disputed
        /// and the `CloseMarketsEarlyOrigin` decides to reject the early close.
        /// The `CloseMarketsEarlyOrigin` (or root) can schedule it `now + CloseProtection...Period`
        /// in the future. This is to prevent fat finger mistakes.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the maximum number of market ids
        /// in `MarketIdsPerClose...` either at the old period end or new period end.
        #[pallet::call_index(18)]
        #[pallet::weight(
            T::WeightInfo::schedule_early_close_as_authority(CacheSize::get(), CacheSize::get())
                .max(T::WeightInfo::schedule_early_close_after_dispute(
                    CacheSize::get(),
                    CacheSize::get(),
                ))
                .max(T::WeightInfo::schedule_early_close_as_market_creator(
                    CacheSize::get(),
                    CacheSize::get(),
                ))
        )]
        #[transactional]
        pub fn schedule_early_close(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let is_authorized = T::CloseMarketEarlyOrigin::try_origin(origin.clone()).is_ok();
            let (market, market_creator) = if !is_authorized {
                // check if market creator below
                let who = ensure_signed(origin)?;
                let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
                ensure!(market.creator == who, Error::<T>::RequesterNotCreator);
                (market, Some(who))
            } else {
                let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
                (market, None)
            };

            Self::ensure_market_is_active(&market)?;
            let now_block = <frame_system::Pallet<T>>::block_number();
            let now_time = <zrml_market_commons::Pallet<T>>::now();

            let get_new_period = |block_period,
                                  time_frame_period|
             -> Result<
                MarketPeriod<T::BlockNumber, MomentOf<T>>,
                DispatchError,
            > {
                match &market.period {
                    MarketPeriod::Block(range) => {
                        let close_time = now_block.saturating_add(block_period);
                        ensure!(close_time < range.end, Error::<T>::EarlyCloseRequestTooLate);
                        Ok(MarketPeriod::Block(range.start..close_time))
                    }
                    MarketPeriod::Timestamp(range) => {
                        let close_time = now_time.saturating_add(time_frame_period);
                        ensure!(close_time < range.end, Error::<T>::EarlyCloseRequestTooLate);
                        Ok(MarketPeriod::Timestamp(range.start..close_time))
                    }
                }
            };
            let new_period = if let Some(p) = &market.early_close {
                ensure!(is_authorized, Error::<T>::OnlyAuthorizedCanScheduleEarlyClose);

                match p.state {
                    // in these case the market period got already reset to the old period
                    EarlyCloseState::Disputed => {
                        if Self::is_close_dispute_bond_pending(&market_id, &market, false) {
                            Self::repatriate_close_dispute_bond(&market_id, &market.creator)?;
                        }
                        if Self::is_close_request_bond_pending(&market_id, &market, false) {
                            Self::unreserve_close_request_bond(&market_id)?;
                        }
                    }
                    EarlyCloseState::Rejected => {}
                    EarlyCloseState::ScheduledAsMarketCreator
                    | EarlyCloseState::ScheduledAsOther => {
                        return Err(Error::<T>::InvalidEarlyCloseState.into());
                    }
                }

                get_new_period(
                    T::CloseEarlyProtectionBlockPeriod::get(),
                    T::CloseEarlyProtectionTimeFramePeriod::get(),
                )?
            } else {
                let (block_period, time_frame_period) = if is_authorized {
                    // fat finger protection
                    (
                        T::CloseEarlyProtectionBlockPeriod::get(),
                        T::CloseEarlyProtectionTimeFramePeriod::get(),
                    )
                } else {
                    let market_creator = market_creator.ok_or(Error::<T>::RequesterNotCreator)?;
                    let close_request_bond = T::CloseEarlyRequestBond::get();

                    T::AssetManager::reserve_named(
                        &Self::reserve_id(),
                        Asset::Ztg,
                        &market_creator,
                        close_request_bond,
                    )?;

                    <zrml_market_commons::Pallet<T>>::mutate_market(&market_id, |market| {
                        market.bonds.close_request =
                            Some(Bond::new(market_creator, close_request_bond));
                        Ok(())
                    })?;

                    (T::CloseEarlyBlockPeriod::get(), T::CloseEarlyTimeFramePeriod::get())
                };

                get_new_period(block_period, time_frame_period)?
            };

            let ids_len_0 = Self::clear_auto_close(&market_id)?;

            let state = if is_authorized {
                EarlyCloseState::ScheduledAsOther
            } else {
                EarlyCloseState::ScheduledAsMarketCreator
            };

            <zrml_market_commons::Pallet<T>>::mutate_market(&market_id, |market| {
                let old_market_period = market.period.clone();
                market.period = new_period.clone();
                let early_close = EarlyClose {
                    old: old_market_period,
                    new: new_period.clone(),
                    state: state.clone(),
                };
                market.early_close = Some(early_close);
                Ok(())
            })?;

            // important to do this after the market period is mutated
            let ids_len_1 = Self::insert_auto_close(&market_id)?;

            let weight = match &market.early_close {
                None => {
                    if is_authorized {
                        T::WeightInfo::schedule_early_close_as_authority(ids_len_0, ids_len_1)
                    } else {
                        T::WeightInfo::schedule_early_close_as_market_creator(ids_len_0, ids_len_1)
                    }
                }
                Some(_) => T::WeightInfo::schedule_early_close_after_dispute(ids_len_0, ids_len_1),
            };

            Self::deposit_event(Event::MarketEarlyCloseScheduled { market_id, new_period, state });

            Ok(Some(weight).into())
        }

        /// Allows anyone to dispute a scheduled early close.
        ///
        /// The market period is reset to the original (old) period.
        /// A `CloseEarlyDisputeBond` is reserved, which is returned,
        /// if the `CloseMarketsEarlyOrigin` decides to reject
        /// the early close request of the market creator or
        /// if the `CloseMarketsEarlyOrigin` is inactive.
        /// It is slashed, if the `CloseMarketsEarlyOrigin` decides to schedule the early close.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the maximum number of market ids
        /// in `MarketIdsPerClose...` either at the old period end or new period end.
        #[pallet::call_index(19)]
        #[pallet::weight(T::WeightInfo::dispute_early_close(CacheSize::get(), CacheSize::get()))]
        #[transactional]
        pub fn dispute_early_close(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            Self::ensure_market_is_active(&market)?;
            let mut early_close = market.early_close.ok_or(Error::<T>::NoEarlyCloseScheduled)?;
            match early_close.state {
                EarlyCloseState::ScheduledAsMarketCreator => (),
                EarlyCloseState::ScheduledAsOther
                | EarlyCloseState::Disputed
                | EarlyCloseState::Rejected => {
                    return Err(Error::<T>::InvalidEarlyCloseState.into());
                }
            };

            // ensure we don't dispute if the old market period is already over
            // so that we don't switch back to an invalid market period
            // this should never trigger, because at the time of scheduling a new market period,
            // we ensure the new end is always before the old end
            let is_expired = match early_close.old {
                MarketPeriod::Block(ref range) => {
                    let now_block = <frame_system::Pallet<T>>::block_number();
                    range.end <= now_block
                }
                MarketPeriod::Timestamp(ref range) => {
                    let now_time = <zrml_market_commons::Pallet<T>>::now();
                    range.end <= now_time
                }
            };
            if is_expired {
                log::debug!(
                    "This will never happen, because schedule_early_close ensures that the new \
                     end is always before the old end, otherwise the switch to an old market \
                     period would lead to a never ending market! Market id: {:?}.",
                    market_id
                );
                debug_assert!(false);
            }

            let close_dispute_bond = T::CloseEarlyDisputeBond::get();

            T::AssetManager::reserve_named(
                &Self::reserve_id(),
                Asset::Ztg,
                &who,
                close_dispute_bond,
            )?;

            let ids_len_0 = Self::clear_auto_close(&market_id)?;

            <zrml_market_commons::Pallet<T>>::mutate_market(&market_id, |market| {
                market.period = early_close.old.clone();
                market.bonds.close_dispute = Some(Bond::new(who.clone(), close_dispute_bond));
                early_close.state = EarlyCloseState::Disputed;
                market.early_close = Some(early_close);
                Ok(())
            })?;

            // important to do this after the market period is mutated
            let ids_len_1 = Self::insert_auto_close(&market_id)?;

            Self::deposit_event(Event::MarketEarlyCloseDisputed { market_id });

            Ok(Some(T::WeightInfo::dispute_early_close(ids_len_0, ids_len_1)).into())
        }

        /// Allows the `CloseMarketsEarlyOrigin` to reject a scheduled early close.
        ///
        /// The market period is reset to the original (old) period
        /// in case it was scheduled before (fat-finger protection).
        ///
        /// The disputant gets back the `CloseEarlyDisputeBond`
        /// and receives the market creators `CloseEarlyRequestBond`.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`, where `n` is the maximum number of market ids
        /// in `MarketIdsPerClose...` either at the old period end or new period end.
        #[pallet::call_index(20)]
        #[pallet::weight(
            T::WeightInfo::reject_early_close_after_authority(CacheSize::get(), CacheSize::get())
                .max(T::WeightInfo::reject_early_close_after_dispute())
        )]
        #[transactional]
        pub fn reject_early_close(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            T::CloseMarketEarlyOrigin::ensure_origin(origin)?;
            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            Self::ensure_market_is_active(&market)?;
            let mut early_close = market.early_close.ok_or(Error::<T>::NoEarlyCloseScheduled)?;
            let weight = match early_close.state {
                // market period got already reset inside `dispute_early_close`
                EarlyCloseState::Disputed => T::WeightInfo::reject_early_close_after_dispute(),
                EarlyCloseState::ScheduledAsOther => {
                    // ensure we don't reject if the old market period is already over
                    // so that we don't switch back to an invalid market period
                    let is_expired = match early_close.old {
                        MarketPeriod::Block(ref range) => {
                            let now_block = <frame_system::Pallet<T>>::block_number();
                            range.end <= now_block
                        }
                        MarketPeriod::Timestamp(ref range) => {
                            let now_time = <zrml_market_commons::Pallet<T>>::now();
                            range.end <= now_time
                        }
                    };
                    if is_expired {
                        log::debug!(
                            "This will never happen, because schedule_early_close ensures that \
                             the new end is always before the old end, 
                    otherwise the switch to an old market period would lead to a never ending \
                             market! Market id: {:?}.",
                            market_id
                        );
                        debug_assert!(false);
                    }

                    let ids_len_0 = Self::clear_auto_close(&market_id)?;

                    <zrml_market_commons::Pallet<T>>::mutate_market(&market_id, |market| {
                        market.period = early_close.old.clone();
                        Ok(())
                    })?;

                    // important to do this after the market period is mutated
                    let ids_len_1 = Self::insert_auto_close(&market_id)?;

                    T::WeightInfo::reject_early_close_after_authority(ids_len_0, ids_len_1)
                }
                EarlyCloseState::ScheduledAsMarketCreator | EarlyCloseState::Rejected => {
                    return Err(Error::<T>::InvalidEarlyCloseState.into());
                }
            };

            if let Some(disputor_bond) = market.bonds.close_dispute.as_ref() {
                let close_disputor = &disputor_bond.who;
                Self::repatriate_close_request_bond(&market_id, close_disputor)?;
                Self::unreserve_close_dispute_bond(&market_id)?;
            }

            <zrml_market_commons::Pallet<T>>::mutate_market(&market_id, |market| {
                early_close.state = EarlyCloseState::Rejected;
                market.early_close = Some(early_close);
                Ok(())
            })?;

            Self::deposit_event(Event::MarketEarlyCloseRejected { market_id });

            Ok(Some(weight).into())
        }

        /// Allows the market creator of a trusted market
        /// to immediately move an open market to closed.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n + m)`, where `n` is the number of market ids,
        /// which open at the same time as the specified market,
        /// and `m` is the number of market ids,
        /// which close at the same time as the specified market.
        #[pallet::call_index(21)]
        #[pallet::weight(T::WeightInfo::close_trusted_market(CacheSize::get()))]
        #[transactional]
        pub fn close_trusted_market(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            ensure!(market.creator == who, Error::<T>::CallerNotMarketCreator);
            ensure!(market.dispute_mechanism.is_none(), Error::<T>::MarketIsNotTrusted);
            Self::ensure_market_is_active(&market)?;
            let close_ids_len = Self::clear_auto_close(&market_id)?;
            Self::close_market(&market_id)?;
            Self::set_market_end(&market_id)?;
            Ok(Some(T::WeightInfo::close_trusted_market(close_ids_len)).into())
        }

        /// Allows the manual closing for "broken" markets.
        /// A market is "broken", if an unexpected chain stall happened
        /// and the auto close was scheduled during this time.
        ///
        /// # Weight
        ///
        /// Complexity: `O(n)`,
        /// and `n` is the number of market ids,
        /// which close at the same time as the specified market.
        #[pallet::call_index(22)]
        #[pallet::weight(T::WeightInfo::manually_close_market(CacheSize::get()))]
        #[transactional]
        pub fn manually_close_market(
            origin: OriginFor<T>,
            #[pallet::compact] market_id: MarketIdOf<T>,
        ) -> DispatchResultWithPostInfo {
            ensure_signed(origin)?;

            let market = zrml_market_commons::Pallet::<T>::market(&market_id)?;
            let now = zrml_market_commons::Pallet::<T>::now();
            let range = match &market.period {
                MarketPeriod::Block(_) => {
                    return Err(Error::<T>::NotAllowedForBlockBasedMarkets.into());
                }
                MarketPeriod::Timestamp(ref range) => range,
            };

            let close_ids_len = if range.end <= now {
                let range_end_time_frame = Self::calculate_time_frame_of_moment(range.end);
                let close_ids_len = MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                    range_end_time_frame,
                    |ids| -> Result<u32, DispatchError> {
                        let ids_len = ids.len() as u32;
                        let position = ids
                            .iter()
                            .position(|i| i == &market_id)
                            .ok_or(Error::<T>::MarketNotInCloseTimeFrameList)?;
                        let _ = ids.swap_remove(position);
                        Ok(ids_len)
                    },
                )?;
                Self::on_market_close(&market_id, market)?;
                Self::set_market_end(&market_id)?;
                close_ids_len
            } else {
                return Err(Error::<T>::MarketPeriodEndNotAlreadyReachedYet.into());
            };

            Ok(Some(T::WeightInfo::manually_close_market(close_ids_len)).into())
        }
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + zrml_market_commons::Config {
        /// The base amount of currency that must be bonded for a market approved by the
        ///  advisory committee.
        #[pallet::constant]
        type AdvisoryBond: Get<BalanceOf<Self>>;

        /// The percentage of the advisory bond that gets slashed when a market is rejected.
        #[pallet::constant]
        type AdvisoryBondSlashPercentage: Get<Percent>;

        /// The origin that is allowed to approve / reject pending advised markets.
        type ApproveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Shares of outcome assets and native currency
        type AssetManager: ZeitgeistAssetManager<
                Self::AccountId,
                Balance = BalanceOf<Self>,
                CurrencyId = Asset<MarketIdOf<Self>>,
                ReserveIdentifier = [u8; 8],
            >;

        #[cfg(feature = "parachain")]
        type AssetRegistry: Inspect<
                AssetId = Asset<MarketIdOf<Self>>,
                Balance = BalanceOf<Self>,
                CustomMetadata = CustomMetadata,
            >;

        /// See [`zrml_authorized::AuthorizedPalletApi`].
        type Authorized: zrml_authorized::AuthorizedPalletApi<
                AccountId = Self::AccountId,
                Balance = BalanceOf<Self>,
                NegativeImbalance = NegativeImbalanceOf<Self>,
                BlockNumber = Self::BlockNumber,
                MarketId = MarketIdOf<Self>,
                Moment = MomentOf<Self>,
                Origin = Self::RuntimeOrigin,
            >;

        /// The base amount of currency that must be bonded
        /// by the disputant in order to dispute an early market closure of the market creator.
        #[pallet::constant]
        type CloseEarlyDisputeBond: Get<BalanceOf<Self>>;

        /// The origin that is allowed to close markets early.
        type CloseMarketEarlyOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        type Currency: NamedReservableCurrency<
                Self::AccountId,
                ReserveIdentifier = [u8; 8],
                Balance = BalanceOf<Self>,
            >;

        /// The origin that is allowed to close markets.
        type CloseOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The milliseconds to wait for the `CloseMarketsEarlyOrigin`
        /// before the early market close actually happens (fat-finger protection).
        #[pallet::constant]
        type CloseEarlyProtectionTimeFramePeriod: Get<MomentOf<Self>>;

        /// The block time to wait for the `CloseMarketsEarlyOrigin`
        /// before the early market close actually happens (fat-finger protection).
        #[pallet::constant]
        type CloseEarlyProtectionBlockPeriod: Get<Self::BlockNumber>;

        /// The base amount of currency that must be bonded
        /// by the market creator in order to schedule an early market closure.
        #[pallet::constant]
        type CloseEarlyRequestBond: Get<BalanceOf<Self>>;

        /// See [`zrml_court::CourtPalletApi`].
        type Court: zrml_court::CourtPalletApi<
                AccountId = Self::AccountId,
                Balance = BalanceOf<Self>,
                NegativeImbalance = NegativeImbalanceOf<Self>,
                BlockNumber = Self::BlockNumber,
                MarketId = MarketIdOf<Self>,
                Moment = MomentOf<Self>,
                Origin = Self::RuntimeOrigin,
            >;

        /// Used to deploy neo-swaps pools.
        type DeployPool: DeployPoolApi<
                AccountId = Self::AccountId,
                Balance = BalanceOf<Self>,
                MarketId = MarketIdOf<Self>,
            >;

        /// The origin that is allowed to destroy markets.
        type DestroyOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The base amount of currency that must be bonded in order to create a dispute.
        #[pallet::constant]
        type DisputeBond: Get<BalanceOf<Self>>;

        /// Event
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// See [`GlobalDisputesPalletApi`].
        type GlobalDisputes: GlobalDisputesPalletApi<
                MarketIdOf<Self>,
                Self::AccountId,
                BalanceOf<Self>,
                Self::BlockNumber,
            >;

        type LiquidityMining: LiquidityMiningPalletApi<
                AccountId = Self::AccountId,
                Balance = BalanceOf<Self>,
                BlockNumber = Self::BlockNumber,
                MarketId = MarketIdOf<Self>,
            >;

        /// The maximum number of categories available for categorical markets.
        #[pallet::constant]
        type MaxCategories: Get<u16>;

        /// The minimum number of categories available for categorical markets.
        #[pallet::constant]
        type MinCategories: Get<u16>;

        /// A upper bound for the fee that is charged each trade and given to the market creator.
        #[pallet::constant]
        type MaxCreatorFee: Get<Perbill>;

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

        /// The maximum allowed duration of a market from creation to market close in blocks.
        #[pallet::constant]
        type MaxMarketLifetime: Get<Self::BlockNumber>;

        /// The maximum number of bytes allowed as edit reason.
        #[pallet::constant]
        type MaxEditReasonLen: Get<u32>;

        #[pallet::constant]
        type OutsiderBond: Get<BalanceOf<Self>>;

        /// The module identifier.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The block time to wait for the market creator
        /// before the early market close actually happens.
        #[pallet::constant]
        type CloseEarlyBlockPeriod: Get<Self::BlockNumber>;

        /// The milliseconds to wait for the market creator
        /// before the early market close actually happens.
        #[pallet::constant]
        type CloseEarlyTimeFramePeriod: Get<MomentOf<Self>>;

        /// The origin that is allowed to reject pending advised markets.
        type RejectOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The base amount of currency that must be bonded to ensure the oracle reports
        ///  in a timely manner.
        #[pallet::constant]
        type OracleBond: Get<BalanceOf<Self>>;

        /// The origin that is allowed to request edits in pending advised markets.
        type RequestEditOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// The origin that is allowed to resolve markets.
        type ResolveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// See [`DisputeApi`].
        type SimpleDisputes: zrml_simple_disputes::SimpleDisputesPalletApi<
                AccountId = Self::AccountId,
                Balance = BalanceOf<Self>,
                NegativeImbalance = NegativeImbalanceOf<Self>,
                BlockNumber = Self::BlockNumber,
                MarketId = MarketIdOf<Self>,
                Moment = MomentOf<Self>,
                Origin = Self::RuntimeOrigin,
            >;

        /// Handler for slashed funds.
        type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

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
        /// The market duration is longer than allowed.
        MarketDurationTooLong,
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
        /// The market dispute mechanism has not failed.
        MarketDisputeMechanismNotFailed,
        /// Tried to settle missing bond.
        MissingBond,
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
        /// The action requires another market dispute mechanism.
        InvalidDisputeMechanism,
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
        /// Provided base_asset is not allowed to be used as base_asset.
        InvalidBaseAsset,
        /// A foreign asset in not registered in AssetRegistry.
        UnregisteredForeignAsset,
        /// The start of the global dispute for this market happened already.
        GlobalDisputeExistsAlready,
        /// The market has no dispute mechanism.
        NoDisputeMechanism,
        /// The dispute duration is positive but the market has dispute period.
        NonZeroDisputePeriodOnTrustedMarket,
        /// The fee is too high.
        FeeTooHigh,
        /// The resolution mechanism resulting from the scoring rule is not supported.
        InvalidResolutionMechanism,
        /// The early market close operation was not requested by the market creator.
        RequesterNotCreator,
        /// The early close would be scheduled after the original market period end.
        EarlyCloseRequestTooLate,
        /// This early close state is not valid.
        InvalidEarlyCloseState,
        /// There is no early close scheduled.
        NoEarlyCloseScheduled,
        /// After there was an early close already scheduled,
        /// only the `CloseMarketsEarlyOrigin` can schedule another one.
        OnlyAuthorizedCanScheduleEarlyClose,
        /// The caller is not the market creator.
        CallerNotMarketCreator,
        /// The market is not trusted.
        MarketIsNotTrusted,
        /// The operation is not allowed for market with a block period.
        NotAllowedForBlockBasedMarkets,
        /// The market is not in the close time frame list.
        MarketNotInCloseTimeFrameList,
        /// The market period end was not already reached yet.
        MarketPeriodEndNotAlreadyReachedYet,
    }

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// Custom addition block initialization logic wasn't successful.
        BadOnInitialize,
        /// A complete set of assets has been bought. \[market_id, amount_per_asset, buyer\]
        BoughtCompleteSet(MarketIdOf<T>, BalanceOf<T>, AccountIdOf<T>),
        /// A market has been approved. \[market_id, new_market_status\]
        MarketApproved(MarketIdOf<T>, MarketStatus),
        /// A market has been created. \[market_id, market_account, market\]
        MarketCreated(MarketIdOf<T>, T::AccountId, MarketOf<T>),
        /// A market has been destroyed. \[market_id\]
        MarketDestroyed(MarketIdOf<T>),
        /// A market has been closed. \[market_id\]
        MarketClosed(MarketIdOf<T>),
        /// A market has been scheduled to close early.
        MarketEarlyCloseScheduled {
            market_id: MarketIdOf<T>,
            new_period: MarketPeriod<T::BlockNumber, MomentOf<T>>,
            state: EarlyCloseState,
        },
        /// A market early close request has been disputed.
        MarketEarlyCloseDisputed { market_id: MarketIdOf<T> },
        /// A market early close request has been rejected.
        MarketEarlyCloseRejected { market_id: MarketIdOf<T> },
        /// A market has been disputed \[market_id, new_market_status, disputant\]
        MarketDisputed(MarketIdOf<T>, MarketStatus, AccountIdOf<T>),
        /// An advised market has ended before it was approved or rejected. \[market_id\]
        MarketExpired(MarketIdOf<T>),
        /// A pending market has been rejected as invalid with a reason.
        /// \[market_id, reject_reason\]
        MarketRejected(MarketIdOf<T>, RejectReason<T>),
        /// A market has been reported on. \[market_id, new_market_status, reported_outcome\]
        MarketReported(MarketIdOf<T>, MarketStatus, ReportOf<T>),
        /// A market has been resolved. \[market_id, new_market_status, real_outcome\]
        MarketResolved(MarketIdOf<T>, MarketStatus, OutcomeReport),
        /// A proposed market has been requested edit by advisor. \[market_id, edit_reason\]
        MarketRequestedEdit(MarketIdOf<T>, EditReason<T>),
        /// A proposed market has been edited by the market creator. \[market_id, new_market\]
        MarketEdited(MarketIdOf<T>, MarketOf<T>),
        /// A complete set of assets has been sold. \[market_id, amount_per_asset, seller\]
        SoldCompleteSet(MarketIdOf<T>, BalanceOf<T>, AccountIdOf<T>),
        /// An amount of winning outcomes have been redeemed.
        /// \[market_id, currency_id, amount_redeemed, payout, who\]
        TokensRedeemed(MarketIdOf<T>, AssetOf<T>, BalanceOf<T>, BalanceOf<T>, AccountIdOf<T>),
        /// The global dispute was started. \[market_id\]
        GlobalDisputeStarted(MarketIdOf<T>),
        /// The recovery limit for timestamp based markets was reached due to a prolonged chain stall.
        RecoveryLimitReached { last_time_frame: TimeFrame, limit_time_frame: TimeFrame },
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        // TODO(#792): Remove outcome assets for accounts! Delete "resolved" assets of `orml_tokens` with storage migration.
        fn on_initialize(now: T::BlockNumber) -> Weight {
            let mut total_weight: Weight = Weight::zero();

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
                Self::calculate_time_frame_of_moment(<zrml_market_commons::Pallet<T>>::now())
                    .saturating_add(1);

            // On first pass, we use current_time - 1 to ensure that the chain doesn't try to
            // check all time frames since epoch.
            let last_time_frame =
                LastTimeFrame::<T>::get().unwrap_or_else(|| current_time_frame.saturating_sub(1));

            let _ = with_transaction(|| {
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

                match close.and(resolve) {
                    Err(err) => {
                        Self::deposit_event(Event::BadOnInitialize);
                        log::error!(
                            target: LOG_TARGET,
                            "Block {:?} was not initialized. Error: {:?}",
                            now,
                            err,
                        );
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

    /// Contains market_ids for which advisor has requested edit.
    /// Value for given market_id represents the reason for the edit.
    #[pallet::storage]
    pub type MarketIdsForEdit<T: Config> =
        StorageMap<_, Twox64Concat, MarketIdOf<T>, EditReason<T>>;

    impl<T: Config> Pallet<T> {
        impl_unreserve_bond!(unreserve_creation_bond, creation);
        impl_unreserve_bond!(unreserve_oracle_bond, oracle);
        impl_unreserve_bond!(unreserve_outsider_bond, outsider);
        impl_unreserve_bond!(unreserve_close_request_bond, close_request);
        impl_unreserve_bond!(unreserve_close_dispute_bond, close_dispute);
        impl_unreserve_bond!(unreserve_dispute_bond, dispute);
        impl_slash_bond!(slash_creation_bond, creation);
        impl_slash_bond!(slash_oracle_bond, oracle);
        impl_slash_bond!(slash_outsider_bond, outsider);
        impl_slash_bond!(slash_dispute_bond, dispute);
        impl_repatriate_bond!(repatriate_oracle_bond, oracle);
        impl_repatriate_bond!(repatriate_close_request_bond, close_request);
        impl_repatriate_bond!(repatriate_close_dispute_bond, close_dispute);
        impl_is_bond_pending!(is_creation_bond_pending, creation);
        impl_is_bond_pending!(is_oracle_bond_pending, oracle);
        impl_is_bond_pending!(is_outsider_bond_pending, outsider);
        impl_is_bond_pending!(is_close_dispute_bond_pending, close_dispute);
        impl_is_bond_pending!(is_close_request_bond_pending, close_request);
        impl_is_bond_pending!(is_dispute_bond_pending, dispute);

        #[inline]
        pub(crate) fn market_account(market_id: MarketIdOf<T>) -> AccountIdOf<T> {
            T::PalletId::get().into_sub_account_truncating(market_id.saturated_into::<u128>())
        }

        #[require_transactional]
        fn do_create_market(
            who: T::AccountId,
            base_asset: AssetOf<T>,
            creator_fee: Perbill,
            oracle: T::AccountId,
            period: MarketPeriod<T::BlockNumber, MomentOf<T>>,
            deadlines: Deadlines<T::BlockNumber>,
            metadata: MultiHash,
            creation: MarketCreation,
            market_type: MarketType,
            dispute_mechanism: Option<MarketDisputeMechanism>,
            scoring_rule: ScoringRule,
        ) -> Result<(u32, MarketIdOf<T>), DispatchError> {
            let bonds = match creation {
                MarketCreation::Advised => MarketBonds {
                    creation: Some(Bond::new(who.clone(), T::AdvisoryBond::get())),
                    oracle: Some(Bond::new(who.clone(), T::OracleBond::get())),
                    ..Default::default()
                },
                MarketCreation::Permissionless => MarketBonds {
                    creation: Some(Bond::new(who.clone(), T::ValidityBond::get())),
                    oracle: Some(Bond::new(who.clone(), T::OracleBond::get())),
                    ..Default::default()
                },
            };

            let market = Self::construct_market(
                base_asset,
                who.clone(),
                creator_fee,
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
                bonds.clone(),
            )?;

            T::AssetManager::reserve_named(
                &Self::reserve_id(),
                Asset::Ztg,
                &who,
                bonds.total_amount_bonded(&who),
            )?;

            let market_id = <zrml_market_commons::Pallet<T>>::push_market(market.clone())?;
            let market_account = Self::market_account(market_id);

            let ids_amount: u32 = Self::insert_auto_close(&market_id)?;

            Self::deposit_event(Event::MarketCreated(market_id, market_account, market));

            Ok((ids_amount, market_id))
        }

        pub fn outcome_assets(market_id: MarketIdOf<T>, market: &MarketOf<T>) -> Vec<AssetOf<T>> {
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

        fn insert_auto_close(market_id: &MarketIdOf<T>) -> Result<u32, DispatchError> {
            let market = <zrml_market_commons::Pallet<T>>::market(market_id)?;

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
            let market = <zrml_market_commons::Pallet<T>>::market(market_id)?;

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

        /// Clears this market from being stored for automatic resolution.
        fn clear_auto_resolve(market_id: &MarketIdOf<T>) -> Result<(u32, u32), DispatchError> {
            let market = <zrml_market_commons::Pallet<T>>::market(market_id)?;
            // If there's no dispute mechanism, this function is noop. TODO(#782) This is an
            // anti-pattern, but it makes benchmarking easier.
            let dispute_mechanism = match market.dispute_mechanism {
                Some(ref result) => result,
                None => return Ok((0, 0)),
            };
            let (ids_len, mdm_len) = match market.status {
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
                    // TODO(#782): use multiple benchmarks paths for different dispute mechanisms
                    let ResultWithWeightInfo { result: auto_resolve_block_opt, weight: _ } =
                        match dispute_mechanism {
                            MarketDisputeMechanism::Authorized => {
                                T::Authorized::get_auto_resolve(market_id, &market)
                            }
                            MarketDisputeMechanism::Court => {
                                T::Court::get_auto_resolve(market_id, &market)
                            }
                            MarketDisputeMechanism::SimpleDisputes => {
                                T::SimpleDisputes::get_auto_resolve(market_id, &market)
                            }
                        };
                    if let Some(auto_resolve_block) = auto_resolve_block_opt {
                        let ids_len = remove_auto_resolve::<T>(market_id, auto_resolve_block);
                        (ids_len, 0u32)
                    } else {
                        (0u32, 0u32)
                    }
                }
                _ => (0u32, 0u32),
            };

            Ok((ids_len, mdm_len))
        }

        #[require_transactional]
        pub(crate) fn do_sell_complete_set(
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(amount != BalanceOf::<T>::zero(), Error::<T>::ZeroAmount);

            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            ensure!(market.is_redeemable(), Error::<T>::InvalidScoringRule);

            let market_account = Self::market_account(market_id);
            ensure!(
                T::AssetManager::free_balance(market.base_asset, &market_account) >= amount,
                "Market account does not have sufficient reserves.",
            );

            let assets = Self::outcome_assets(market_id, &market);

            // verify first.
            for asset in assets.iter() {
                // Ensures that the sender has sufficient amount of each
                // share in the set.
                ensure!(
                    T::AssetManager::free_balance(*asset, &who) >= amount,
                    Error::<T>::InsufficientShareBalance,
                );
            }

            // write last.
            for asset in assets.iter() {
                let missing = T::AssetManager::slash(*asset, &who, amount);
                debug_assert!(
                    missing.is_zero(),
                    "Could not slash all of the amount. asset {:?}, who: {:?}, amount: {:?}.",
                    asset,
                    &who,
                    amount,
                );
            }

            T::AssetManager::transfer(market.base_asset, &market_account, &who, amount)?;

            Self::deposit_event(Event::SoldCompleteSet(market_id, amount, who));

            Ok(())
        }

        #[require_transactional]
        pub(crate) fn do_buy_complete_set(
            who: T::AccountId,
            market_id: MarketIdOf<T>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            ensure!(amount != BalanceOf::<T>::zero(), Error::<T>::ZeroAmount);
            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            ensure!(
                T::AssetManager::free_balance(market.base_asset, &who) >= amount,
                Error::<T>::NotEnoughBalance
            );
            ensure!(market.is_redeemable(), Error::<T>::InvalidScoringRule);
            Self::ensure_market_is_active(&market)?;

            let market_account = Self::market_account(market_id);
            T::AssetManager::transfer(market.base_asset, &who, &market_account, amount)?;

            let assets = Self::outcome_assets(market_id, &market);
            for asset in assets.iter() {
                T::AssetManager::deposit(*asset, &who, amount)?;
            }

            Self::deposit_event(Event::BoughtCompleteSet(market_id, amount, who));

            Ok(())
        }

        pub(crate) fn do_reject_market(
            market_id: &MarketIdOf<T>,
            market: MarketOf<T>,
            reject_reason: RejectReason<T>,
        ) -> DispatchResult {
            ensure!(market.status == MarketStatus::Proposed, Error::<T>::InvalidMarketStatus);
            Self::unreserve_oracle_bond(market_id)?;
            let imbalance =
                Self::slash_creation_bond(market_id, Some(T::AdvisoryBondSlashPercentage::get()))?;
            T::Slash::on_unbalanced(imbalance);
            <zrml_market_commons::Pallet<T>>::remove_market(market_id)?;
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
            Self::unreserve_creation_bond(market_id)?;
            Self::unreserve_oracle_bond(market_id)?;
            <zrml_market_commons::Pallet<T>>::remove_market(market_id)?;
            MarketIdsForEdit::<T>::remove(market_id);
            Self::deposit_event(Event::MarketExpired(*market_id));
            Ok(T::WeightInfo::handle_expired_advised_market())
        }

        pub(crate) fn calculate_time_frame_of_moment(time: MomentOf<T>) -> TimeFrame {
            time.saturated_into::<TimeFrame>().saturating_div(MILLISECS_PER_BLOCK.into())
        }

        fn calculate_internal_resolve_weight(market: &MarketOf<T>) -> Weight {
            if let MarketType::Categorical(_) = market.market_type {
                if let MarketStatus::Reported = market.status {
                    T::WeightInfo::internal_resolve_categorical_reported()
                } else {
                    T::WeightInfo::internal_resolve_categorical_disputed()
                }
            } else if let MarketStatus::Reported = market.status {
                T::WeightInfo::internal_resolve_scalar_reported()
            } else {
                T::WeightInfo::internal_resolve_scalar_disputed()
            }
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
                    let now = <frame_system::Pallet<T>>::block_number();
                    ensure!(now < range.end, Error::<T>::InvalidMarketPeriod);
                    ensure!(range.start < range.end, Error::<T>::InvalidMarketPeriod);
                    let lifetime = range.end.saturating_sub(now); // Never saturates!
                    ensure!(
                        lifetime <= T::MaxMarketLifetime::get(),
                        Error::<T>::MarketDurationTooLong,
                    );
                }
                MarketPeriod::Timestamp(ref range) => {
                    // Ensure that the market lasts at least one time frame into the future.
                    let now = <zrml_market_commons::Pallet<T>>::now();
                    let now_frame = Self::calculate_time_frame_of_moment(now);
                    let end_frame = Self::calculate_time_frame_of_moment(range.end);
                    ensure!(now_frame < end_frame, Error::<T>::InvalidMarketPeriod);
                    ensure!(range.start < range.end, Error::<T>::InvalidMarketPeriod);
                    // Verify that the number of frames that the market is open doesn't exceed the
                    // maximum allowed lifetime in blocks.
                    let lifetime = end_frame.saturating_sub(now_frame); // Never saturates!
                    // If this conversion saturates, we're dealing with a market with excessive
                    // lifetime:
                    let lifetime_max: TimeFrame = T::MaxMarketLifetime::get().saturated_into();
                    ensure!(lifetime <= lifetime_max, Error::<T>::MarketDurationTooLong);
                }
            };
            Ok(())
        }

        fn ensure_market_deadlines_are_valid(
            deadlines: &Deadlines<T::BlockNumber>,
            trusted: bool,
        ) -> DispatchResult {
            ensure!(
                deadlines.oracle_duration >= T::MinOracleDuration::get(),
                Error::<T>::OracleDurationSmallerThanMinOracleDuration
            );
            if trusted {
                ensure!(
                    deadlines.dispute_duration == Zero::zero(),
                    Error::<T>::NonZeroDisputePeriodOnTrustedMarket
                );
            } else {
                ensure!(
                    deadlines.dispute_duration >= T::MinDisputeDuration::get(),
                    Error::<T>::DisputeDurationSmallerThanMinDisputeDuration
                );
                ensure!(
                    deadlines.dispute_duration <= T::MaxDisputeDuration::get(),
                    Error::<T>::DisputeDurationGreaterThanMaxDisputeDuration
                );
            }
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

        pub(crate) fn close_market(market_id: &MarketIdOf<T>) -> Result<Weight, DispatchError> {
            <zrml_market_commons::Pallet<T>>::mutate_market(market_id, |market| {
                ensure!(market.status == MarketStatus::Active, Error::<T>::InvalidMarketStatus);

                if let Some(p) = &market.early_close {
                    match p.state {
                        EarlyCloseState::ScheduledAsMarketCreator => {
                            if Self::is_close_request_bond_pending(market_id, market, false) {
                                Self::unreserve_close_request_bond(market_id)?;
                            }
                        }
                        EarlyCloseState::Disputed => {
                            // this is the case that the original close happened,
                            // although requested early close or disputed
                            // there was no decision made via `reject` or `approve`
                            if Self::is_close_dispute_bond_pending(market_id, market, false) {
                                Self::unreserve_close_dispute_bond(market_id)?;
                            }
                            if Self::is_close_request_bond_pending(market_id, market, false) {
                                Self::unreserve_close_request_bond(market_id)?;
                            }
                        }
                        EarlyCloseState::ScheduledAsOther | EarlyCloseState::Rejected => {}
                    }
                }

                market.status = MarketStatus::Closed;
                Ok(())
            })?;
            let mut total_weight = T::DbWeight::get().reads_writes(1, 1);
            Self::deposit_event(Event::MarketClosed(*market_id));
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
            Ok(total_weight)
        }

        pub(crate) fn set_market_end(market_id: &MarketIdOf<T>) -> Result<Weight, DispatchError> {
            <zrml_market_commons::Pallet<T>>::mutate_market(market_id, |market| {
                market.period = match market.period {
                    MarketPeriod::Block(ref range) => {
                        let current_block = <frame_system::Pallet<T>>::block_number();
                        MarketPeriod::Block(range.start..current_block)
                    }
                    MarketPeriod::Timestamp(ref range) => {
                        let now = <zrml_market_commons::Pallet<T>>::now();
                        MarketPeriod::Timestamp(range.start..now)
                    }
                };
                Ok(())
            })?;
            Ok(T::DbWeight::get().reads_writes(1, 1))
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

        /// Handle a market resolution, which is currently in the reported state.
        /// Returns the resolved outcome of a market, which is the reported outcome.
        fn resolve_reported_market(
            market_id: &MarketIdOf<T>,
            market: &MarketOf<T>,
        ) -> Result<OutcomeReport, DispatchError> {
            let report = market.report.as_ref().ok_or(Error::<T>::MarketIsNotReported)?;
            // the oracle bond gets returned if the reporter was the oracle
            if report.by == market.oracle {
                Self::unreserve_oracle_bond(market_id)?;
            } else {
                // reward outsider reporter with oracle bond
                Self::repatriate_oracle_bond(market_id, &report.by)?;

                if Self::is_outsider_bond_pending(market_id, market, true) {
                    Self::unreserve_outsider_bond(market_id)?;
                }
            }

            Ok(report.outcome.clone())
        }

        /// Handle a market resolution, which is currently in the disputed state.
        /// Returns the resolved outcome of a market.
        fn resolve_disputed_market(
            market_id: &MarketIdOf<T>,
            market: &MarketOf<T>,
        ) -> Result<ResultWithWeightInfo<OutcomeReport>, DispatchError> {
            let dispute_mechanism =
                market.dispute_mechanism.as_ref().ok_or(Error::<T>::NoDisputeMechanism)?;
            let report = market.report.as_ref().ok_or(Error::<T>::MarketIsNotReported)?;
            let mut weight = Weight::zero();

            let res: ResultWithWeightInfo<OutcomeReport> =
                Self::get_resolved_outcome(market_id, market, &report.outcome)?;
            let resolved_outcome = res.result;
            weight = weight.saturating_add(res.weight);

            let imbalance_left = Self::settle_bonds(market_id, market, &resolved_outcome, report)?;

            let remainder = match dispute_mechanism {
                MarketDisputeMechanism::Authorized => {
                    let res = T::Authorized::exchange(
                        market_id,
                        market,
                        &resolved_outcome,
                        imbalance_left,
                    )?;
                    let remainder = res.result;
                    weight = weight.saturating_add(res.weight);
                    remainder
                }
                MarketDisputeMechanism::Court => {
                    let res =
                        T::Court::exchange(market_id, market, &resolved_outcome, imbalance_left)?;
                    let remainder = res.result;
                    weight = weight.saturating_add(res.weight);
                    remainder
                }
                MarketDisputeMechanism::SimpleDisputes => {
                    let res = T::SimpleDisputes::exchange(
                        market_id,
                        market,
                        &resolved_outcome,
                        imbalance_left,
                    )?;
                    let remainder = res.result;
                    weight = weight.saturating_add(res.weight);
                    remainder
                }
            };

            T::Slash::on_unbalanced(remainder);

            let res = ResultWithWeightInfo { result: resolved_outcome, weight };

            Ok(res)
        }

        /// Get the outcome the market should resolve to.
        pub(crate) fn get_resolved_outcome(
            market_id: &MarketIdOf<T>,
            market: &MarketOf<T>,
            reported_outcome: &OutcomeReport,
        ) -> Result<ResultWithWeightInfo<OutcomeReport>, DispatchError> {
            let mut resolved_outcome_option = None;
            let mut weight = Weight::zero();

            if let Some(o) = T::GlobalDisputes::determine_voting_winner(market_id) {
                resolved_outcome_option = Some(o);
            }

            // Try to get the outcome of the MDM. If the MDM failed to resolve, default to
            // the oracle's report.
            if resolved_outcome_option.is_none() {
                let dispute_mechanism =
                    market.dispute_mechanism.as_ref().ok_or(Error::<T>::NoDisputeMechanism)?;
                resolved_outcome_option = match dispute_mechanism {
                    MarketDisputeMechanism::Authorized => {
                        let res = T::Authorized::on_resolution(market_id, market)?;
                        weight = weight.saturating_add(res.weight);
                        res.result
                    }
                    MarketDisputeMechanism::Court => {
                        let res = T::Court::on_resolution(market_id, market)?;
                        weight = weight.saturating_add(res.weight);
                        res.result
                    }
                    MarketDisputeMechanism::SimpleDisputes => {
                        let res = T::SimpleDisputes::on_resolution(market_id, market)?;
                        weight = weight.saturating_add(res.weight);
                        res.result
                    }
                };
            }

            let resolved_outcome =
                resolved_outcome_option.unwrap_or_else(|| reported_outcome.clone());

            let res = ResultWithWeightInfo { result: resolved_outcome, weight };

            Ok(res)
        }

        /// Manage the outstanding bonds (oracle, outsider, dispute) of the market.
        fn settle_bonds(
            market_id: &MarketIdOf<T>,
            market: &MarketOf<T>,
            resolved_outcome: &OutcomeReport,
            report: &ReportOf<T>,
        ) -> Result<NegativeImbalanceOf<T>, DispatchError> {
            let mut overall_imbalance = NegativeImbalanceOf::<T>::zero();

            let report_by_oracle = report.by == market.oracle;
            let is_correct = &report.outcome == resolved_outcome;

            let unreserve_outsider = || -> DispatchResult {
                if Self::is_outsider_bond_pending(market_id, market, true) {
                    Self::unreserve_outsider_bond(market_id)?;
                }
                Ok(())
            };

            let slash_outsider = || -> Result<NegativeImbalanceOf<T>, DispatchError> {
                if Self::is_outsider_bond_pending(market_id, market, true) {
                    let imbalance = Self::slash_outsider_bond(market_id, None)?;
                    return Ok(imbalance);
                }
                Ok(NegativeImbalanceOf::<T>::zero())
            };

            if report_by_oracle {
                if is_correct {
                    Self::unreserve_oracle_bond(market_id)?;
                } else {
                    let negative_imbalance = Self::slash_oracle_bond(market_id, None)?;
                    overall_imbalance.subsume(negative_imbalance);
                }
            } else {
                // outsider report
                if is_correct {
                    // reward outsider reporter with oracle bond
                    Self::repatriate_oracle_bond(market_id, &report.by)?;
                    unreserve_outsider()?;
                } else {
                    let oracle_imbalance = Self::slash_oracle_bond(market_id, None)?;
                    let outsider_imbalance = slash_outsider()?;
                    overall_imbalance.subsume(oracle_imbalance);
                    overall_imbalance.subsume(outsider_imbalance);
                }
            }

            if let Some(bond) = &market.bonds.dispute {
                if !bond.is_settled {
                    if is_correct {
                        let imb = Self::slash_dispute_bond(market_id, None)?;
                        overall_imbalance.subsume(imb);
                    } else {
                        // If the report outcome was wrong, the dispute was justified
                        Self::unreserve_dispute_bond(market_id)?;
                        T::Currency::resolve_creating(&bond.who, overall_imbalance);
                        overall_imbalance = NegativeImbalanceOf::<T>::zero();
                    }
                }
            }

            Ok(overall_imbalance)
        }

        pub fn on_resolution(
            market_id: &MarketIdOf<T>,
            market: &MarketOf<T>,
        ) -> Result<Weight, DispatchError> {
            if market.creation == MarketCreation::Permissionless {
                Self::unreserve_creation_bond(market_id)?;
            }

            let mut total_weight: Weight = Weight::zero();

            let resolved_outcome = match market.status {
                MarketStatus::Reported => Self::resolve_reported_market(market_id, market)?,
                MarketStatus::Disputed => {
                    let res = Self::resolve_disputed_market(market_id, market)?;
                    total_weight = total_weight.saturating_add(res.weight);
                    res.result
                }
                _ => return Err(Error::<T>::InvalidMarketStatus.into()),
            };
            // TODO: https://github.com/zeitgeistpm/zeitgeist/issues/815
            // Following call should return weight consumed by it.
            T::LiquidityMining::distribute_market_incentives(market_id)?;

            // NOTE: Currently we don't clean up outcome assets.
            // TODO(#792): Remove outcome assets for accounts! Delete "resolved" assets of `orml_tokens` with storage migration.
            <zrml_market_commons::Pallet<T>>::mutate_market(market_id, |m| {
                m.status = MarketStatus::Resolved;
                m.resolved_outcome = Some(resolved_outcome.clone());
                Ok(())
            })?;

            Self::deposit_event(Event::MarketResolved(
                *market_id,
                MarketStatus::Resolved,
                resolved_outcome,
            ));
            Ok(total_weight.saturating_add(Self::calculate_internal_resolve_weight(market)))
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
                let market = <zrml_market_commons::Pallet<T>>::market(market_id)?;
                mutation(market_id, market)?;
            }
            MarketIdsPerBlock::remove(block_number);

            let mut time_frame_ids_len = 0u32;
            let start = last_time_frame.saturating_add(1);
            let end = current_time_frame;
            let diff = end.saturating_sub(start);
            let start = if diff > MAX_RECOVERY_TIME_FRAMES {
                log::warn!(
                    target: LOG_TARGET,
                    "Could not recover all time frames since the last time frame {:?}.",
                    last_time_frame,
                );
                let limit_time_frame = end.saturating_sub(MAX_RECOVERY_TIME_FRAMES);
                Self::deposit_event(Event::RecoveryLimitReached {
                    last_time_frame,
                    limit_time_frame,
                });
                limit_time_frame
            } else {
                start
            };
            for time_frame in start..=end {
                let market_ids_per_time_frame = MarketIdsPerTimeFrame::get(time_frame);
                time_frame_ids_len =
                    time_frame_ids_len.saturating_add(market_ids_per_time_frame.len() as u32);
                for market_id in market_ids_per_time_frame.iter() {
                    let market = <zrml_market_commons::Pallet<T>>::market(market_id)?;
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
                let market = <zrml_market_commons::Pallet<T>>::market(id)?;
                if let MarketStatus::Reported = market.status {
                    cb(id, &market)?;
                }
            }
            MarketIdsPerReportBlock::<T>::remove(now);

            // Resolve any disputed markets.
            let market_ids_per_dispute_block = MarketIdsPerDisputeBlock::<T>::get(now);
            for id in market_ids_per_dispute_block.iter() {
                let market = <zrml_market_commons::Pallet<T>>::market(id)?;
                cb(id, &market)?;
            }
            MarketIdsPerDisputeBlock::<T>::remove(now);

            Ok(T::WeightInfo::market_resolution_manager(
                market_ids_per_report_block.len() as u32,
                market_ids_per_dispute_block.len() as u32,
            ))
        }

        fn construct_market(
            base_asset: AssetOf<T>,
            creator: T::AccountId,
            creator_fee: Perbill,
            oracle: T::AccountId,
            period: MarketPeriod<T::BlockNumber, MomentOf<T>>,
            deadlines: Deadlines<T::BlockNumber>,
            metadata: MultiHash,
            creation: MarketCreation,
            market_type: MarketType,
            dispute_mechanism: Option<MarketDisputeMechanism>,
            scoring_rule: ScoringRule,
            report: Option<ReportOf<T>>,
            resolved_outcome: Option<OutcomeReport>,
            bonds: MarketBondsOf<T>,
        ) -> Result<MarketOf<T>, DispatchError> {
            let valid_base_asset = match base_asset {
                Asset::Ztg => true,
                #[cfg(feature = "parachain")]
                Asset::ForeignAsset(fa) => {
                    if let Some(metadata) = T::AssetRegistry::metadata(&Asset::ForeignAsset(fa)) {
                        metadata.additional.allow_as_base_asset
                    } else {
                        return Err(Error::<T>::UnregisteredForeignAsset.into());
                    }
                }
                _ => false,
            };

            ensure!(creator_fee <= T::MaxCreatorFee::get(), Error::<T>::FeeTooHigh);
            ensure!(valid_base_asset, Error::<T>::InvalidBaseAsset);
            let MultiHash::Sha3_384(multihash) = metadata;
            ensure!(multihash[0] == 0x15 && multihash[1] == 0x30, <Error<T>>::InvalidMultihash);
            Self::ensure_market_period_is_valid(&period)?;
            Self::ensure_market_deadlines_are_valid(&deadlines, dispute_mechanism.is_none())?;
            Self::ensure_market_type_is_valid(&market_type)?;

            let status: MarketStatus = match creation {
                MarketCreation::Permissionless => MarketStatus::Active,
                MarketCreation::Advised => MarketStatus::Proposed,
            };
            Ok(Market {
                base_asset,
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
                bonds,
                early_close: None,
            })
        }

        fn report_market_with_dispute_mechanism(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
            report: ReportOf<T>,
        ) -> DispatchResultWithPostInfo {
            let sender = ensure_signed(origin.clone())?;
            <zrml_market_commons::Pallet<T>>::mutate_market(&market_id, |market| {
                let mut should_check_origin = false;
                //NOTE: Saturating operation in following block may saturate to u32::MAX value
                //      but that will be the case after thousands of years time. So it is fine.
                match market.period {
                    MarketPeriod::Block(ref range) => {
                        let grace_period_end =
                            range.end.saturating_add(market.deadlines.grace_period);
                        ensure!(grace_period_end <= report.at, Error::<T>::NotAllowedToReportYet);
                        let oracle_duration_end =
                            grace_period_end.saturating_add(market.deadlines.oracle_duration);
                        if report.at <= oracle_duration_end {
                            should_check_origin = true;
                        }
                    }
                    MarketPeriod::Timestamp(ref range) => {
                        let grace_period_in_moments: MomentOf<T> =
                            market.deadlines.grace_period.saturated_into::<u32>().into();
                        let grace_period_in_ms =
                            grace_period_in_moments.saturating_mul(MILLISECS_PER_BLOCK.into());
                        let grace_period_end = range.end.saturating_add(grace_period_in_ms);
                        let now = <zrml_market_commons::Pallet<T>>::now();
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

                let sender_is_oracle = sender == market.oracle;
                let origin_has_permission = T::ResolveOrigin::ensure_origin(origin).is_ok();
                let sender_is_outsider = !sender_is_oracle && !origin_has_permission;

                if should_check_origin {
                    ensure!(
                        sender_is_oracle || origin_has_permission,
                        Error::<T>::ReporterNotOracle
                    );
                } else if sender_is_outsider {
                    let outsider_bond = T::OutsiderBond::get();

                    market.bonds.outsider = Some(Bond::new(sender.clone(), outsider_bond));

                    T::AssetManager::reserve_named(
                        &Self::reserve_id(),
                        Asset::Ztg,
                        &sender,
                        outsider_bond,
                    )?;
                }

                market.report = Some(report.clone());
                market.status = MarketStatus::Reported;

                Ok(())
            })?;

            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            let block_after_dispute_duration =
                report.at.saturating_add(market.deadlines.dispute_duration);
            let ids_len = MarketIdsPerReportBlock::<T>::try_mutate(
                block_after_dispute_duration,
                |ids| -> Result<u32, DispatchError> {
                    ids.try_push(market_id).map_err(|_| <Error<T>>::StorageOverflow)?;
                    Ok(ids.len() as u32)
                },
            )?;

            Ok(Some(T::WeightInfo::report_market_with_dispute_mechanism(ids_len)).into())
        }

        fn report_and_resolve_market(
            origin: OriginFor<T>,
            market_id: MarketIdOf<T>,
            market_report: ReportOf<T>,
        ) -> DispatchResultWithPostInfo {
            <zrml_market_commons::Pallet<T>>::mutate_market(&market_id, |market| {
                let sender = ensure_signed(origin.clone())?;
                let sender_is_oracle = sender == market.oracle;
                let origin_has_permission = T::ResolveOrigin::ensure_origin(origin).is_ok();
                ensure!(sender_is_oracle || origin_has_permission, Error::<T>::ReporterNotOracle);
                market.report = Some(market_report.clone());
                market.status = MarketStatus::Reported;
                Ok(())
            })?;
            let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
            Self::on_resolution(&market_id, &market)?;
            Ok(Some(T::WeightInfo::report_trusted_market()).into())
        }
    }

    fn remove_item<I: cmp::PartialEq, G>(items: &mut BoundedVec<I, G>, item: &I) {
        if let Some(pos) = items.iter().position(|i| i == item) {
            items.swap_remove(pos);
        }
    }

    fn remove_auto_resolve<T: Config>(
        market_id: &MarketIdOf<T>,
        resolve_at: T::BlockNumber,
    ) -> u32 {
        MarketIdsPerDisputeBlock::<T>::mutate(resolve_at, |ids| -> u32 {
            let ids_len = ids.len() as u32;
            remove_item::<MarketIdOf<T>, _>(ids, market_id);
            ids_len
        })
    }

    impl<T> DisputeResolutionApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type Balance = BalanceOf<T>;
        type BlockNumber = T::BlockNumber;
        type MarketId = MarketIdOf<T>;
        type Moment = MomentOf<T>;

        fn resolve(
            market_id: &Self::MarketId,
            market: &MarketOf<T>,
        ) -> Result<Weight, DispatchError> {
            Self::on_resolution(market_id, market)
        }

        fn add_auto_resolve(
            market_id: &Self::MarketId,
            resolve_at: Self::BlockNumber,
        ) -> Result<u32, DispatchError> {
            let ids_len = <MarketIdsPerDisputeBlock<T>>::try_mutate(
                resolve_at,
                |ids| -> Result<u32, DispatchError> {
                    ids.try_push(*market_id).map_err(|_| <Error<T>>::StorageOverflow)?;
                    Ok(ids.len() as u32)
                },
            )?;
            Ok(ids_len)
        }

        fn auto_resolve_exists(market_id: &Self::MarketId, resolve_at: Self::BlockNumber) -> bool {
            <MarketIdsPerDisputeBlock<T>>::get(resolve_at).contains(market_id)
        }

        fn remove_auto_resolve(market_id: &Self::MarketId, resolve_at: Self::BlockNumber) -> u32 {
            remove_auto_resolve::<T>(market_id, resolve_at)
        }
    }

    impl<T> CompleteSetOperationsApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type Balance = BalanceOf<T>;
        type MarketId = MarketIdOf<T>;

        fn buy_complete_set(
            who: Self::AccountId,
            market_id: Self::MarketId,
            amount: Self::Balance,
        ) -> DispatchResult {
            Self::do_buy_complete_set(who, market_id, amount)
        }

        fn sell_complete_set(
            who: Self::AccountId,
            market_id: Self::MarketId,
            amount: Self::Balance,
        ) -> DispatchResult {
            Self::do_sell_complete_set(who, market_id, amount)
        }
    }
}
