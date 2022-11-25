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

use crate::{CacheSize, Config, Disputes, MarketIdOf, Pallet};
use alloc::{vec, vec::Vec};
use frame_support::{
    dispatch::Weight,
    log,
    migration::{put_storage_value, storage_iter},
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    BoundedVec,
};
use parity_scale_codec::EncodeLike;
use sp_runtime::traits::{One, Saturating};
use zeitgeist_primitives::{
    constants::BASE,
    types::{AuthorityReport, MarketDisputeMechanism, MarketType, OutcomeReport},
};
use zrml_authorized::{AuthorizedOutcomeReports, Pallet as AuthorizedPallet};
use zrml_court::{Pallet as CourtPallet, Votes};
use zrml_market_commons::{MarketCommonsPalletApi, Pallet as MarketCommonsPallet};

const AUTHORIZED: &[u8] = b"Authorized";
const AUTHORIZED_OUTCOME_REPORTS: &[u8] = b"AuthorizedOutcomeReports";

const PREDICTION_MARKETS: &[u8] = b"PredictionMarkets";
const MARKET_IDS_PER_DISPUTE_BLOCK: &[u8] = b"MarketIdsPerDisputeBlock";

const AUTHORIZED_REQUIRED_STORAGE_VERSION: u16 = 2;
const AUTHORIZED_NEXT_STORAGE_VERSION: u16 = 3;
const COURT_REQUIRED_STORAGE_VERSION: u16 = 1;
const COURT_NEXT_STORAGE_VERSION: u16 = 2;
const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 3;
const MARKET_COMMONS_NEXT_STORAGE_VERSION: u16 = 4;
const PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION: u16 = 6;
const PREDICTION_MARKETS_NEXT_STORAGE_VERSION: u16 = 7;

pub struct AddFieldToAuthorityReport<T>(PhantomData<T>);

// Add resolve_at block number value field to `AuthorizedOutcomeReports` map.
impl<T: Config + zrml_market_commons::Config + zrml_authorized::Config> OnRuntimeUpgrade
    for AddFieldToAuthorityReport<T>
where
    <T as zrml_market_commons::Config>::MarketId: EncodeLike<
        <<T as zrml_authorized::Config>::MarketCommonsAuthorized as MarketCommonsPalletApi>::MarketId,
    >,
{
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);
        let authorized_version = StorageVersion::get::<AuthorizedPallet<T>>();
        if authorized_version != AUTHORIZED_REQUIRED_STORAGE_VERSION {
            log::info!(
                "AddFieldToAuthorityReport: authorized version is {:?}, require {:?};",
                authorized_version,
                AUTHORIZED_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("AddFieldToAuthorityReport: Starting...");

        let mut new_storage_map = Vec::new();
        let now = <frame_system::Pallet<T>>::block_number();
        for (key, old_value) in
            storage_iter::<Option<OutcomeReport>>(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS)
        {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

            if let Some(outcome) = old_value {
                let resolve_at: T::BlockNumber = now
                    .saturating_add(<T as zrml_authorized::Config>::AuthorityReportPeriod::get());
                let new_value = AuthorityReport { resolve_at, outcome };
                new_storage_map.push((key, new_value));
            }
        }

        for (key, new_value) in new_storage_map {
            put_storage_value::<Option<AuthorityReport<T::BlockNumber>>>(
                AUTHORIZED,
                AUTHORIZED_OUTCOME_REPORTS,
                &key,
                Some(new_value),
            );
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        }

        StorageVersion::new(AUTHORIZED_NEXT_STORAGE_VERSION).put::<AuthorizedPallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("AddFieldToAuthorityReport: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        for (key, old_value) in storage_iter::<Option<OutcomeReport>>(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS) {
            if let Some(outcome) = old_value {
                // save outcome and check it in post_upgrade
                unimplemented!();
            }
        }
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        let now = <frame_system::Pallet<T>>::block_number();
        assert_eq!(<T as zrml_authorized::Config>::AuthorityReportPeriod::get(), 43_200u32.into());
        for (key, new_value) in storage_iter::<Option<AuthorityReport<T::BlockNumber>>>(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS) {
            if let Some(AuthorityReport { resolve_at, outcome }) = new_value {
                assert_eq!(resolve_at, now.checked_add(<T as zrml_authorized::Config>::AuthorityReportPeriod::get()).unwrap());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests_authorized {
    use super::*;
    use crate::mock::{ExtBuilder, Runtime};
    use frame_support::Twox64Concat;
    use zeitgeist_primitives::types::{MarketId, OutcomeReport};

    #[test]
    fn on_runtime_upgrade_increments_the_storage_versions() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();
            AddFieldToAuthorityReport::<Runtime>::on_runtime_upgrade();
            let authorized_version = StorageVersion::get::<AuthorizedPallet<Runtime>>();
            assert_eq!(authorized_version, AUTHORIZED_NEXT_STORAGE_VERSION);
        });
    }

    #[test]
    fn on_runtime_sets_new_struct_with_resolve_at() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            <frame_system::Pallet<Runtime>>::set_block_number(10_000);

            let hash = crate::migrations::utility::key_to_hash::<Twox64Concat, MarketId>(0);
            let outcome = OutcomeReport::Categorical(42u16);
            put_storage_value::<Option<OutcomeReport>>(
                AUTHORIZED,
                AUTHORIZED_OUTCOME_REPORTS,
                &hash,
                Some(outcome.clone()),
            );

            AddFieldToAuthorityReport::<Runtime>::on_runtime_upgrade();

            let now = <frame_system::Pallet<Runtime>>::block_number();
            let resolve_at: <Runtime as frame_system::Config>::BlockNumber = now
                .saturating_add(<Runtime as zrml_authorized::Config>::AuthorityReportPeriod::get());
            let expected = Some(AuthorityReport { resolve_at, outcome });

            let actual = frame_support::migration::get_storage_value::<
                Option<AuthorityReport<<Runtime as frame_system::Config>::BlockNumber>>,
            >(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS, &hash)
            .unwrap();
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn on_runtime_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            // storage migration already executed (storage version is incremented already)
            StorageVersion::new(AUTHORIZED_NEXT_STORAGE_VERSION).put::<AuthorizedPallet<Runtime>>();

            let hash = crate::migrations::utility::key_to_hash::<Twox64Concat, MarketId>(0);
            let outcome = OutcomeReport::Categorical(42u16);
            let now = <frame_system::Pallet<Runtime>>::block_number();
            let resolve_at: <Runtime as frame_system::Config>::BlockNumber = now
                .saturating_add(<Runtime as zrml_authorized::Config>::AuthorityReportPeriod::get());
            let report = AuthorityReport { resolve_at, outcome };
            put_storage_value::<
                Option<AuthorityReport<<Runtime as frame_system::Config>::BlockNumber>>,
            >(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS, &hash, Some(report.clone()));

            AddFieldToAuthorityReport::<Runtime>::on_runtime_upgrade();

            let actual = frame_support::migration::get_storage_value::<
                Option<AuthorityReport<<Runtime as frame_system::Config>::BlockNumber>>,
            >(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS, &hash)
            .unwrap();
            assert_eq!(Some(report), actual);
        });
    }

    fn set_up_chain() {
        StorageVersion::new(AUTHORIZED_REQUIRED_STORAGE_VERSION).put::<AuthorizedPallet<Runtime>>();
    }
}

pub struct UpdateMarketIdsPerDisputeBlock<T>(PhantomData<T>);

// Delete the auto resolution of authorized and court from `MarketIdsPerDisputeBlock`
impl<T: Config + zrml_market_commons::Config + zrml_authorized::Config> OnRuntimeUpgrade
    for UpdateMarketIdsPerDisputeBlock<T>
where
    <T as zrml_market_commons::Config>::MarketId:
        EncodeLike<<<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId>,
    <T as zrml_market_commons::Config>::MarketId: EncodeLike<
        <<T as zrml_authorized::Config>::MarketCommonsAuthorized as MarketCommonsPalletApi>::MarketId,
    >,
{
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);
        let prediction_markets_version = StorageVersion::get::<Pallet<T>>();
        if prediction_markets_version != PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "prediction-markets version is {:?}, require {:?}",
                prediction_markets_version,
                PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("UpdateMarketIdsPerDisputeBlock: Starting...");

        let mut new_storage_map = Vec::new();
        let mut authorized_ids = Vec::new();
        for (key, mut bounded_vec) in storage_iter::<BoundedVec<MarketIdOf<T>, CacheSize>>(
            PREDICTION_MARKETS,
            MARKET_IDS_PER_DISPUTE_BLOCK,
        ) {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

            bounded_vec.retain(|id| {
                if let Ok(market) = <T as crate::Config>::MarketCommons::market(id) {
                    match market.dispute_mechanism {
                        MarketDisputeMechanism::Authorized => {
                            authorized_ids.push(*id);
                            false
                        }
                        MarketDisputeMechanism::Court => false,
                        MarketDisputeMechanism::SimpleDisputes => true,
                    }
                } else {
                    // no market for id in MarketIdsPerDisputeBlock
                    false
                }
            });

            new_storage_map.push((key, bounded_vec));
        }

        for (key, new_bounded_vec) in new_storage_map {
            put_storage_value::<BoundedVec<MarketIdOf<T>, CacheSize>>(
                PREDICTION_MARKETS,
                MARKET_IDS_PER_DISPUTE_BLOCK,
                &key,
                new_bounded_vec,
            );
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        }

        let now = <frame_system::Pallet<T>>::block_number();
        for id in authorized_ids {
            let mut resolve_at: T::BlockNumber =
                now.saturating_add(<T as zrml_authorized::Config>::AuthorityReportPeriod::get());
            let mut ids = crate::MarketIdsPerDisputeBlock::<T>::get(resolve_at);
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

            while ids.is_full() {
                resolve_at = resolve_at.saturating_add(One::one());
                ids = crate::MarketIdsPerDisputeBlock::<T>::get(resolve_at);
                total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
            }
            // is_full check above to ensure, that we can force_push
            ids.force_push(id);
            crate::MarketIdsPerDisputeBlock::<T>::insert(resolve_at, ids);

            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        }

        StorageVersion::new(PREDICTION_MARKETS_NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("UpdateMarketIdsPerDisputeBlock: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        Ok(())
    }
}

#[cfg(test)]
mod tests_auto_resolution {
    use super::*;
    use crate::{
        mock::{ExtBuilder, Runtime},
        MomentOf,
    };
    use zeitgeist_primitives::types::{
        Deadlines, MarketCreation, MarketDisputeMechanism, MarketId, MarketPeriod, MarketStatus,
        OutcomeReport, Report, ScoringRule,
    };
    use zrml_market_commons::Markets;

    type Market = zeitgeist_primitives::types::Market<
        <Runtime as frame_system::Config>::AccountId,
        <Runtime as frame_system::Config>::BlockNumber,
        MomentOf<Runtime>,
    >;

    #[test]
    fn on_runtime_upgrade_increments_the_storage_versions() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();
            UpdateMarketIdsPerDisputeBlock::<Runtime>::on_runtime_upgrade();
            let prediction_markets_version = StorageVersion::get::<Pallet<Runtime>>();
            assert_eq!(prediction_markets_version, PREDICTION_MARKETS_NEXT_STORAGE_VERSION);
        });
    }

    #[test]
    fn on_runtime_updates_market_ids_per_dispute_block_authorized_ids_full() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            let market_id = MarketId::from(0u64);
            let market = get_market(MarketDisputeMechanism::Authorized);

            Markets::<Runtime>::insert(market_id, market);

            let now = <frame_system::Pallet<Runtime>>::block_number();
            crate::MarketIdsPerDisputeBlock::<Runtime>::insert(
                now,
                BoundedVec::try_from(vec![market_id]).unwrap(),
            );

            let resolve_at = now
                .saturating_add(<Runtime as zrml_authorized::Config>::AuthorityReportPeriod::get());

            let full_ids: Vec<MarketId> = (MarketId::from(1u64)..=MarketId::from(64u64)).collect();

            for id in full_ids.clone() {
                let market = get_market(MarketDisputeMechanism::SimpleDisputes);
                Markets::<Runtime>::insert(id, market);
            }

            crate::MarketIdsPerDisputeBlock::<Runtime>::insert(
                resolve_at,
                BoundedVec::try_from(full_ids.clone()).unwrap(),
            );
            assert!(crate::MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at).is_full());

            UpdateMarketIdsPerDisputeBlock::<Runtime>::on_runtime_upgrade();

            assert_eq!(crate::MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at), full_ids);
            assert!(
                !crate::MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at).contains(&market_id)
            );
            // store market id at the next block
            assert_eq!(
                crate::MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at + 1),
                vec![market_id]
            );
        });
    }

    #[test]
    fn on_runtime_updates_market_ids_per_dispute_block_simple_disputes_unchanged() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            let market_id = MarketId::from(0u64);
            let market = get_market(MarketDisputeMechanism::SimpleDisputes);

            Markets::<Runtime>::insert(market_id, market);

            crate::MarketIdsPerDisputeBlock::<Runtime>::insert(
                0,
                BoundedVec::try_from(vec![market_id]).unwrap(),
            );

            UpdateMarketIdsPerDisputeBlock::<Runtime>::on_runtime_upgrade();

            // unchanged for simple disputes
            assert_eq!(crate::MarketIdsPerDisputeBlock::<Runtime>::get(0), vec![market_id]);
        });
    }

    #[test]
    fn on_runtime_updates_market_ids_per_dispute_block_authorized_deleted() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            let market_id = MarketId::from(0u64);
            let market = get_market(MarketDisputeMechanism::Authorized);

            Markets::<Runtime>::insert(market_id, market);

            crate::MarketIdsPerDisputeBlock::<Runtime>::insert(
                0,
                BoundedVec::try_from(vec![market_id]).unwrap(),
            );

            UpdateMarketIdsPerDisputeBlock::<Runtime>::on_runtime_upgrade();

            // authority controls market resolution now (no auto resolution)
            assert!(crate::MarketIdsPerDisputeBlock::<Runtime>::get(0).is_empty());
        });
    }

    #[test]
    fn on_runtime_updates_market_ids_per_dispute_block_court_deletion() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            let market_id = MarketId::from(0u64);
            let market = get_market(MarketDisputeMechanism::Court);
            Markets::<Runtime>::insert(market_id, market);

            crate::MarketIdsPerDisputeBlock::<Runtime>::insert(
                0,
                BoundedVec::try_from(vec![market_id]).unwrap(),
            );

            UpdateMarketIdsPerDisputeBlock::<Runtime>::on_runtime_upgrade();

            // court auto resolution is deactivated for now (court is disabled)
            assert!(crate::MarketIdsPerDisputeBlock::<Runtime>::get(0).is_empty());
        });
    }

    #[test]
    fn on_runtime_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            // Don't set up chain to signal that storage is already up to date.

            let market_id = MarketId::from(0u64);
            let market = get_market(MarketDisputeMechanism::Court);
            Markets::<Runtime>::insert(market_id, market);

            crate::MarketIdsPerDisputeBlock::<Runtime>::insert(
                0,
                BoundedVec::try_from(vec![market_id]).unwrap(),
            );

            UpdateMarketIdsPerDisputeBlock::<Runtime>::on_runtime_upgrade();

            // normally court auto resolution gets deleted with the storage migration,
            // but because storage version is already updated,
            // it is not
            assert_eq!(crate::MarketIdsPerDisputeBlock::<Runtime>::get(0), vec![market_id]);
        });
    }

    fn get_market(mdm: MarketDisputeMechanism) -> Market {
        Market {
            creator: 1,
            creation: MarketCreation::Permissionless,
            creator_fee: 2,
            oracle: 3,
            metadata: vec![4, 5],
            market_type: MarketType::Categorical(14),
            period: MarketPeriod::Block(6..7),
            deadlines: Deadlines { grace_period: 8, oracle_duration: 9, dispute_duration: 10 },
            scoring_rule: ScoringRule::CPMM,
            status: MarketStatus::Disputed,
            report: Some(Report { at: 11, by: 12, outcome: OutcomeReport::Categorical(13) }),
            resolved_outcome: Some(OutcomeReport::Categorical(13)),
            dispute_mechanism: mdm,
        }
    }

    fn set_up_chain() {
        StorageVersion::new(PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
    }
}

pub struct TransformScalarMarketsToFixedPoint<T>(PhantomData<T>);

// Transform all scalar intervals by BASE, thereby turning every scalar position into a fixed point
// number with ten digits after the decimal point. This update should only be executed if the
// interpretation of metadata in changed in parallel. If that is the case, market description need
// not be updated.
impl<T: Config + zrml_market_commons::Config + zrml_authorized::Config + zrml_court::Config>
    OnRuntimeUpgrade for TransformScalarMarketsToFixedPoint<T>
where
    <T as zrml_market_commons::Config>::MarketId: EncodeLike<
        <<T as zrml_authorized::Config>::MarketCommonsAuthorized as MarketCommonsPalletApi>::MarketId,
    >,
    <T as zrml_market_commons::Config>::MarketId:
        EncodeLike<<<T as zrml_court::Config>::MarketCommons as MarketCommonsPalletApi>::MarketId>,
    <T as zrml_market_commons::Config>::MarketId:
        EncodeLike<<<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId>,
{
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(4);
        let authorized_version = StorageVersion::get::<AuthorizedPallet<T>>();
        let court_version = StorageVersion::get::<CourtPallet<T>>();
        let market_commons_version = StorageVersion::get::<MarketCommonsPallet<T>>();
        let prediction_markets_version = StorageVersion::get::<Pallet<T>>();
        if authorized_version != AUTHORIZED_REQUIRED_STORAGE_VERSION
            || court_version != COURT_REQUIRED_STORAGE_VERSION
            || market_commons_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION
            || prediction_markets_version != PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION
        {
            log::info!(
                "TransformScalarMarketsToFixedPoint: authorized version is {:?}, require {:?}; \
                 court version is {:?}, require {:?}; market-commons version is {:?}, require \
                 {:?}; prediction-markets version is {:?}, require {:?}",
                authorized_version,
                AUTHORIZED_REQUIRED_STORAGE_VERSION,
                court_version,
                COURT_REQUIRED_STORAGE_VERSION,
                market_commons_version,
                MARKET_COMMONS_REQUIRED_STORAGE_VERSION,
                prediction_markets_version,
                PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("TransformScalarMarketsToFixedPoint: Starting...");

        let mut new_scalar_markets: Vec<_> = vec![];
        for (market_id, mut market) in MarketCommonsPallet::<T>::market_iter() {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
            if let MarketType::Scalar(range) = market.market_type {
                let new_start = to_fixed_point(*range.start());
                let new_end = to_fixed_point(*range.end());
                market.market_type = MarketType::Scalar(new_start..=new_end);

                if let Some(mut report) = market.report {
                    if let OutcomeReport::Scalar(value) = report.outcome {
                        report.outcome = OutcomeReport::Scalar(to_fixed_point(value));
                    }
                    market.report = Some(report);
                }

                if let Some(mut resolved_outcome) = market.resolved_outcome {
                    if let OutcomeReport::Scalar(value) = resolved_outcome {
                        resolved_outcome = OutcomeReport::Scalar(to_fixed_point(value));
                    }
                    market.resolved_outcome = Some(resolved_outcome);
                }

                let old_disputes = Disputes::<T>::get(market_id);
                total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
                let new_disputes = if old_disputes.is_empty() {
                    None
                } else {
                    BoundedVec::try_from(
                        old_disputes
                            .into_iter()
                            .map(|mut dispute| {
                                if let OutcomeReport::Scalar(value) = dispute.outcome {
                                    dispute.outcome = OutcomeReport::Scalar(to_fixed_point(value));
                                };
                                dispute
                            })
                            .collect::<Vec<_>>(),
                    )
                    .ok()
                };

                let authorized_report = match AuthorizedOutcomeReports::<T>::get(market_id) {
                    Some(mut outcome_report) => {
                        if let OutcomeReport::Scalar(value) = outcome_report.outcome {
                            outcome_report.outcome = OutcomeReport::Scalar(to_fixed_point(value));
                        }
                        Some(outcome_report)
                    }
                    None => None,
                };
                total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

                let votes = Votes::<T>::iter_prefix(market_id)
                    .filter_map(|(juror, (block_number, outcome_report))| {
                        total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
                        match outcome_report {
                            OutcomeReport::Scalar(value) => Some((
                                juror,
                                (block_number, OutcomeReport::Scalar(to_fixed_point(value))),
                            )),
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>();

                new_scalar_markets.push((
                    market_id,
                    market,
                    new_disputes,
                    authorized_report,
                    votes,
                ));
            }
        }

        for (market_id, market, disputes, authorized_report, votes) in new_scalar_markets {
            let _ = MarketCommonsPallet::<T>::mutate_market(&market_id, |old_market| {
                *old_market = market;
                Ok(())
            });
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

            if let Some(disputes_unwrapped) = disputes {
                Disputes::<T>::insert(market_id, disputes_unwrapped);
                total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
            }

            if let Some(outcome_report) = authorized_report {
                AuthorizedOutcomeReports::<T>::insert(market_id, outcome_report);
                total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
            }

            for (juror, vote) in votes {
                Votes::<T>::insert(market_id, juror, vote);
                total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
            }
        }

        StorageVersion::new(AUTHORIZED_NEXT_STORAGE_VERSION).put::<AuthorizedPallet<T>>();
        StorageVersion::new(COURT_NEXT_STORAGE_VERSION).put::<CourtPallet<T>>();
        StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION).put::<MarketCommonsPallet<T>>();
        StorageVersion::new(PREDICTION_MARKETS_NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(4));
        log::info!("TransformScalarMarketsToFixedPoint: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        // Check that no saturation occurs.
        for (market_id, market) in MarketCommonsPallet::<T>::market_iter() {
            if let MarketType::Scalar(range) = market.market_type {
                assert!(
                    range.end().checked_mul(BASE).is_some(),
                    "TransformScalarMarketsToFixedPoint: Arithmetic overflow when transforming \
                     market {:?}",
                    market_id,
                );
            }
        }
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        for (market_id, market) in MarketCommonsPallet::<T>::market_iter() {
            if let MarketType::Scalar(range) = market.market_type {
                assert_ne!(
                    range.start(),
                    range.end(),
                    "TransformScalarMarketsToFixedPoint: Scalar range broken after transformation \
                     of market {:?}",
                    market_id,
                );
            }
        }
        Ok(())
    }
}

fn to_fixed_point(value: u128) -> u128 {
    value.saturating_mul(BASE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock::{ExtBuilder, Runtime},
        Disputes, MomentOf,
    };
    use zeitgeist_primitives::types::{
        Deadlines, MarketCreation, MarketDispute, MarketDisputeMechanism, MarketId, MarketPeriod,
        MarketStatus, OutcomeReport, Report, ScoringRule,
    };
    use zrml_market_commons::Markets;

    type Market = zeitgeist_primitives::types::Market<
        <Runtime as frame_system::Config>::AccountId,
        <Runtime as frame_system::Config>::BlockNumber,
        MomentOf<Runtime>,
    >;
    type MarketDisputeOf<T> = MarketDispute<
        <T as frame_system::Config>::AccountId,
        <T as frame_system::Config>::BlockNumber,
    >;

    #[test]
    fn on_runtime_upgrade_increments_the_storage_versions() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();
            TransformScalarMarketsToFixedPoint::<Runtime>::on_runtime_upgrade();
            let authorized_version = StorageVersion::get::<AuthorizedPallet<Runtime>>();
            let court_version = StorageVersion::get::<CourtPallet<Runtime>>();
            let market_commons_version = StorageVersion::get::<MarketCommonsPallet<Runtime>>();
            let prediction_markets_version = StorageVersion::get::<Pallet<Runtime>>();
            assert_eq!(authorized_version, AUTHORIZED_NEXT_STORAGE_VERSION);
            assert_eq!(court_version, COURT_NEXT_STORAGE_VERSION);
            assert_eq!(market_commons_version, MARKET_COMMONS_NEXT_STORAGE_VERSION);
            assert_eq!(prediction_markets_version, PREDICTION_MARKETS_NEXT_STORAGE_VERSION);
        });
    }

    #[test]
    fn on_runtime_upgrade_ignores_categorical_markets() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();
            let market_id: MarketId = 7;
            let market = Market {
                creator: 1,
                creation: MarketCreation::Permissionless,
                creator_fee: 2,
                oracle: 3,
                metadata: vec![4, 5],
                market_type: MarketType::Categorical(14),
                period: MarketPeriod::Block(6..7),
                deadlines: Deadlines { grace_period: 8, oracle_duration: 9, dispute_duration: 10 },
                scoring_rule: ScoringRule::CPMM,
                status: MarketStatus::Resolved,
                report: Some(Report { at: 11, by: 12, outcome: OutcomeReport::Categorical(13) }),
                resolved_outcome: Some(OutcomeReport::Categorical(13)),
                dispute_mechanism: MarketDisputeMechanism::Court,
            };
            Markets::<Runtime>::insert(market_id, market.clone());
            TransformScalarMarketsToFixedPoint::<Runtime>::on_runtime_upgrade();
            let market_after = Markets::<Runtime>::get(market_id);
            assert_eq!(market, market_after.unwrap());
        });
    }

    #[test]
    fn on_runtime_transforms_scalar_markets_and_their_disputes() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            let market_id: MarketId = 7;
            let market = Market {
                creator: 1,
                creation: MarketCreation::Permissionless,
                creator_fee: 2,
                oracle: 3,
                metadata: vec![4, 5],
                market_type: MarketType::Scalar(14..=15),
                period: MarketPeriod::Block(6..7),
                deadlines: Deadlines { grace_period: 8, oracle_duration: 9, dispute_duration: 10 },
                scoring_rule: ScoringRule::CPMM,
                status: MarketStatus::Resolved,
                report: Some(Report { at: 11, by: 12, outcome: OutcomeReport::Categorical(13) }),
                resolved_outcome: Some(OutcomeReport::Categorical(13)),
                dispute_mechanism: MarketDisputeMechanism::Court,
            };
            Markets::<Runtime>::insert(market_id, market.clone());

            let dispute =
                MarketDisputeOf::<Runtime> { at: 16, by: 17, outcome: OutcomeReport::Scalar(18) };
            Disputes::<Runtime>::insert(
                market_id,
                BoundedVec::try_from(vec![dispute.clone()]).unwrap(),
            );

            let report = AuthorityReport { resolve_at: 42, outcome: OutcomeReport::Scalar(19) };
            AuthorizedOutcomeReports::<Runtime>::insert(market_id, report);

            let juror = 20;
            let block_number = 21;
            Votes::<Runtime>::insert(market_id, juror, (block_number, OutcomeReport::Scalar(22)));

            TransformScalarMarketsToFixedPoint::<Runtime>::on_runtime_upgrade();

            let mut market_expected = market;
            market_expected.market_type = MarketType::Scalar(140_000_000_000..=150_000_000_000);
            let market_after = Markets::<Runtime>::get(market_id).unwrap();
            assert_eq!(market_after, market_expected);

            let mut dispute_expected = dispute;
            dispute_expected.outcome = OutcomeReport::Scalar(180_000_000_000);
            let disputes_after = Disputes::<Runtime>::get(market_id);
            assert_eq!(disputes_after.len(), 1);
            assert_eq!(disputes_after[0], dispute_expected);

            let authorized_report_after =
                AuthorizedOutcomeReports::<Runtime>::get(market_id).unwrap();
            assert_eq!(authorized_report_after.outcome, OutcomeReport::Scalar(190_000_000_000));

            let vote_after = Votes::<Runtime>::get(market_id, juror).unwrap();
            assert_eq!(vote_after, (block_number, OutcomeReport::Scalar(220_000_000_000)));
        });
    }

    #[test]
    fn on_runtime_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            // Don't set up chain to signal that storage is already up to date.

            let market_id: MarketId = 7;
            let market = Market {
                creator: 1,
                creation: MarketCreation::Permissionless,
                creator_fee: 2,
                oracle: 3,
                metadata: vec![4, 5],
                market_type: MarketType::Scalar(14..=15),
                period: MarketPeriod::Block(6..7),
                deadlines: Deadlines { grace_period: 8, oracle_duration: 9, dispute_duration: 10 },
                scoring_rule: ScoringRule::CPMM,
                status: MarketStatus::Resolved,
                report: Some(Report { at: 11, by: 12, outcome: OutcomeReport::Categorical(13) }),
                resolved_outcome: Some(OutcomeReport::Categorical(13)),
                dispute_mechanism: MarketDisputeMechanism::Court,
            };
            Markets::<Runtime>::insert(market_id, market.clone());

            let dispute =
                MarketDisputeOf::<Runtime> { at: 16, by: 17, outcome: OutcomeReport::Scalar(18) };
            Disputes::<Runtime>::insert(
                market_id,
                BoundedVec::try_from(vec![dispute.clone()]).unwrap(),
            );

            let report = AuthorityReport { resolve_at: 42, outcome: OutcomeReport::Scalar(19) };
            AuthorizedOutcomeReports::<Runtime>::insert(market_id, report);

            let juror = 20;
            let vote = (21, OutcomeReport::Scalar(22));
            Votes::<Runtime>::insert(market_id, juror, vote.clone());

            TransformScalarMarketsToFixedPoint::<Runtime>::on_runtime_upgrade();

            let market_after = Markets::<Runtime>::get(market_id).unwrap();
            assert_eq!(market_after, market);

            let disputes_after = Disputes::<Runtime>::get(market_id);
            assert_eq!(disputes_after.len(), 1);
            assert_eq!(disputes_after[0], dispute);

            let authorized_report_after =
                AuthorizedOutcomeReports::<Runtime>::get(market_id).unwrap();
            assert_eq!(authorized_report_after.outcome, OutcomeReport::Scalar(19));

            let vote_after = Votes::<Runtime>::get(market_id, juror).unwrap();
            assert_eq!(vote_after, vote);
        });
    }

    fn set_up_chain() {
        StorageVersion::new(AUTHORIZED_REQUIRED_STORAGE_VERSION).put::<AuthorizedPallet<Runtime>>();
        StorageVersion::new(COURT_REQUIRED_STORAGE_VERSION).put::<CourtPallet<Runtime>>();
        StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION)
            .put::<MarketCommonsPallet<Runtime>>();
        StorageVersion::new(PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
    }
}

// We use these utilities to prevent having to make the swaps pallet a dependency of
// prediciton-markets. The calls are based on the implementation of `StorageVersion`, found here:
// https://github.com/paritytech/substrate/blob/bc7a1e6c19aec92bfa247d8ca68ec63e07061032/frame/support/src/traits/metadata.rs#L168-L230
// and previous migrations.
mod utility {
    use crate::{BalanceOf, Config, MarketIdOf};
    use alloc::vec::Vec;
    use frame_support::{
        migration::{get_storage_value, put_storage_value},
        storage::{storage_prefix, unhashed},
        traits::StorageVersion,
        Blake2_128Concat, StorageHasher,
    };
    use parity_scale_codec::Encode;
    use zeitgeist_primitives::types::{Pool, PoolId};

    #[allow(unused)]
    const SWAPS: &[u8] = b"Swaps";
    #[allow(unused)]
    const POOLS: &[u8] = b"Pools";
    #[allow(unused)]
    fn storage_prefix_of_swaps_pallet() -> [u8; 32] {
        storage_prefix(b"Swaps", b":__STORAGE_VERSION__:")
    }
    #[allow(unused)]
    pub fn key_to_hash<H, K>(key: K) -> Vec<u8>
    where
        H: StorageHasher,
        K: Encode,
    {
        key.using_encoded(H::hash).as_ref().to_vec()
    }
    #[allow(unused)]
    pub fn get_on_chain_storage_version_of_swaps_pallet() -> StorageVersion {
        let key = storage_prefix_of_swaps_pallet();
        unhashed::get_or_default(&key)
    }
    #[allow(unused)]
    pub fn put_storage_version_of_swaps_pallet(value: u16) {
        let key = storage_prefix_of_swaps_pallet();
        unhashed::put(&key, &StorageVersion::new(value));
    }
    #[allow(unused)]
    pub fn get_pool<T: Config>(pool_id: PoolId) -> Option<Pool<BalanceOf<T>, MarketIdOf<T>>> {
        let hash = key_to_hash::<Blake2_128Concat, PoolId>(pool_id);
        let pool_maybe =
            get_storage_value::<Option<Pool<BalanceOf<T>, MarketIdOf<T>>>>(SWAPS, POOLS, &hash);
        pool_maybe.unwrap_or(None)
    }
    #[allow(unused)]
    pub fn set_pool<T: Config>(pool_id: PoolId, pool: Pool<BalanceOf<T>, MarketIdOf<T>>) {
        let hash = key_to_hash::<Blake2_128Concat, PoolId>(pool_id);
        put_storage_value(SWAPS, POOLS, &hash, Some(pool));
    }
}
