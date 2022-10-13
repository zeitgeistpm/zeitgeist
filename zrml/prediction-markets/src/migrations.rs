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

use crate::{Config, MomentOf, Pallet};
use alloc::{vec, vec::Vec};
use frame_support::{
    dispatch::Weight,
    log,
    migration::{get_storage_value, put_storage_value, storage_iter},
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    BoundedVec,
};
use zeitgeist_primitives::{
    constants::BASE,
    types::{Market, MarketDispute, MarketType, OutcomeReport},
};
#[cfg(feature = "try-runtime")]
use zrml_market_commons::MarketCommonsPalletApi;

const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 2;
const MARKET_COMMONS_NEXT_STORAGE_VERSION: u16 = 3;
const PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION: u16 = 5;
const PREDICTION_MARKETS_NEXT_STORAGE_VERSION: u16 = 6;

const MARKET_COMMONS: &[u8] = b"MarketCommons";
const MARKETS: &[u8] = b"Markets";
const PREDICTION_MARKETS: &[u8] = b"PredictionMarkets";
const DISPUTES: &[u8] = b"Disputes";

type MarketDisputeOf<T> =
    MarketDispute<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber>;
type DisputesOf<T> = BoundedVec<MarketDisputeOf<T>, <T as Config>::MaxDisputes>;
type MarketOf<T> = Market<
    <T as frame_system::Config>::AccountId,
    <T as frame_system::Config>::BlockNumber,
    MomentOf<T>,
>;

pub struct TransformScalarMarketsToFixedPoint<T>(PhantomData<T>);

// Transform all scalar intervals by BASE, thereby turning every scalar position into a fixed point
// number with ten digits after the decimal point. This update should only be executed if the
// interpretation of metadata in changed in parallel. If that is the case, market description need
// not be updated.
impl<T: Config> OnRuntimeUpgrade for TransformScalarMarketsToFixedPoint<T> {
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);
        if StorageVersion::get::<Pallet<T>>() != PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "TransformScalarMarketsToFixedPoint: prediction-markets already up to date..."
            );
            return total_weight;
        }
        total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
        if utility::get_on_chain_storage_version_of_market_commons_pallet()
            != MARKET_COMMONS_REQUIRED_STORAGE_VERSION
        {
            log::info!("TransformScalarMarketsToFixedPoint: market-commons already up to date...");
            return total_weight;
        }
        log::info!("TransformScalarMarketsToFixedPoint: Starting...");

        let mut new_scalar_markets: Vec<_> = vec![];
        for (key, mut market) in storage_iter::<MarketOf<T>>(&MARKET_COMMONS, MARKETS) {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(2));
            match market.market_type {
                MarketType::Scalar(range) => {
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

                    // Transform disputes using the same key because both maps have the same key
                    // type and hasher.
                    let old_disputes =
                        get_storage_value::<DisputesOf<T>>(&PREDICTION_MARKETS, DISPUTES, &key);
                    let new_disputes = match old_disputes {
                        Some(disputes_unwrapped) => BoundedVec::try_from(
                            disputes_unwrapped
                                .into_iter()
                                .map(|mut dispute| {
                                    if let OutcomeReport::Scalar(value) = dispute.outcome {
                                        dispute.outcome =
                                            OutcomeReport::Scalar(to_fixed_point(value));
                                    };
                                    dispute
                                })
                                .collect::<Vec<_>>(),
                        )
                        .ok(),
                        None => None,
                    };
                    new_scalar_markets.push((key, market, new_disputes));
                }
                _ => (),
            };
        }

        for (key, market, new_disputes) in new_scalar_markets {
            if let Some(disputes_unwrapped) = new_disputes {
                put_storage_value::<DisputesOf<T>>(
                    &PREDICTION_MARKETS,
                    DISPUTES,
                    &key,
                    disputes_unwrapped,
                );
                total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
            }

            put_storage_value(MARKET_COMMONS, MARKETS, &key, market);
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        }

        StorageVersion::new(PREDICTION_MARKETS_NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        utility::put_storage_version_of_market_commons_pallet(MARKET_COMMONS_NEXT_STORAGE_VERSION);
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(2));
        log::info!("TransformScalarMarketsToFixedPoint: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        // Check that no saturation occurs.
        for (market_id, market) in T::MarketCommons::market_iter().drain() {
            log::info!("foo");
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
}

fn to_fixed_point(value: u128) -> u128 {
    value.saturating_mul(BASE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock::{ExtBuilder, Runtime},
        Disputes,
    };
    use core::fmt::Debug;
    use frame_support::{Blake2_128Concat, StorageHasher};
    use parity_scale_codec::Encode;
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
            TransformScalarMarketsToFixedPoint::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<Pallet<Runtime>>(),
                PREDICTION_MARKETS_NEXT_STORAGE_VERSION
            );
            assert_eq!(
                utility::get_on_chain_storage_version_of_market_commons_pallet(),
                MARKET_COMMONS_NEXT_STORAGE_VERSION
            );
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
            insert_market::<Blake2_128Concat>(market_id, market.clone());
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
        });
    }

    #[test]
    fn on_runtime_ignored_scalar_markets_if_skipped() {
        ExtBuilder::default().build().execute_with(|| {
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
            insert_market::<Blake2_128Concat>(market_id, market.clone());

            let dispute =
                MarketDisputeOf::<Runtime> { at: 16, by: 17, outcome: OutcomeReport::Scalar(18) };
            Disputes::<Runtime>::insert(
                market_id,
                BoundedVec::try_from(vec![dispute.clone()]).unwrap(),
            );

            TransformScalarMarketsToFixedPoint::<Runtime>::on_runtime_upgrade();

            let market_after = Markets::<Runtime>::get(market_id).unwrap();
            assert_eq!(market_after, market);

            let disputes_after = Disputes::<Runtime>::get(market_id);
            assert_eq!(disputes_after.len(), 1);
            assert_eq!(disputes_after[0], dispute);
        });
    }

    fn set_up_chain() {
        StorageVersion::new(PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
        utility::put_storage_version_of_market_commons_pallet(
            MARKET_COMMONS_REQUIRED_STORAGE_VERSION,
        );
    }

    fn insert_market<H>(market_id: MarketId, market: Market)
    where
        H: StorageHasher,
    {
        let storage_hash = key_to_hash::<H, MarketId>(market_id as usize);
        put_storage_value::<Market>(MARKET_COMMONS, MARKETS, &storage_hash, market);
    }

    fn key_to_hash<H, K>(key: usize) -> Vec<u8>
    where
        H: StorageHasher,
        K: TryFrom<usize> + Encode,
        <K as TryFrom<usize>>::Error: Debug,
    {
        K::try_from(key).unwrap().using_encoded(H::hash).as_ref().to_vec()
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

    pub fn storage_prefix_of_market_common_pallet() -> [u8; 32] {
        storage_prefix(b"MarketCommons", b":__STORAGE_VERSION__:")
    }

    pub fn get_on_chain_storage_version_of_market_commons_pallet() -> StorageVersion {
        let key = storage_prefix_of_market_common_pallet();
        unhashed::get_or_default(&key)
    }

    pub fn put_storage_version_of_market_commons_pallet(value: u16) {
        let key = storage_prefix_of_market_common_pallet();
        unhashed::put(&key, &StorageVersion::new(value));
    }

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
