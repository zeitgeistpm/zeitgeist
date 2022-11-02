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

use crate::{Config, Disputes, Pallet};
use alloc::{vec, vec::Vec};
use frame_support::{
    dispatch::Weight,
    log,
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    BoundedVec,
};
use parity_scale_codec::EncodeLike;
use zeitgeist_primitives::{
    constants::BASE,
    types::{MarketType, OutcomeReport},
};
use zrml_authorized::{AuthorizedOutcomeReports, Pallet as AuthorizedPallet};
use zrml_court::{Pallet as CourtPallet, Votes};
use zrml_market_commons::{MarketCommonsPalletApi, Pallet as MarketCommonsPallet};

const AUTHORIZED_REQUIRED_STORAGE_VERSION: u16 = 1;
const AUTHORIZED_NEXT_STORAGE_VERSION: u16 = 2;
const COURT_REQUIRED_STORAGE_VERSION: u16 = 1;
const COURT_NEXT_STORAGE_VERSION: u16 = 2;
const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 3;
const MARKET_COMMONS_NEXT_STORAGE_VERSION: u16 = 4;
const PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION: u16 = 5;
const PREDICTION_MARKETS_NEXT_STORAGE_VERSION: u16 = 6;

pub struct TransformScalarMarketsToFixedPoint<T>(PhantomData<T>);

// Transform all scalar intervals by BASE, thereby turning every scalar position into a fixed point
// number with ten digits after the decimal point. This update should only be executed if the
// interpretation of metadata in changed in parallel. If that is the case, market description need
// not be updated.
impl<T: Config + zrml_market_commons::Config + zrml_authorized::Config + zrml_court::Config>
    OnRuntimeUpgrade for TransformScalarMarketsToFixedPoint<T>
where
    <T as zrml_market_commons::Config>::MarketId: EncodeLike<
        <<T as zrml_authorized::Config>::MarketCommons as MarketCommonsPalletApi>::MarketId,
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
                        if let OutcomeReport::Scalar(value) = outcome_report {
                            outcome_report = OutcomeReport::Scalar(to_fixed_point(value));
                        };
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

            AuthorizedOutcomeReports::<Runtime>::insert(market_id, OutcomeReport::Scalar(19));

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
            assert_eq!(authorized_report_after, OutcomeReport::Scalar(190_000_000_000));

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

            AuthorizedOutcomeReports::<Runtime>::insert(market_id, OutcomeReport::Scalar(19));

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
            assert_eq!(authorized_report_after, OutcomeReport::Scalar(19));

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
