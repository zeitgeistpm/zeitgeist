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

use crate::{BalanceOf, Config, MomentOf};
use frame_support::{
    log,
    migration::{put_storage_value, storage_iter},
    pallet_prelude::{MaxEncodedLen, PhantomData, TypeInfo},
    traits::{Get, OnRuntimeUpgrade},
    RuntimeDebug,
};
extern crate alloc;
use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};
use zeitgeist_primitives::types::{
    Deadlines, Market, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus,
    MarketType, OutcomeReport, Report, ScoringRule,
};

const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 4;
const MARKET_COMMONS_NEXT_STORAGE_VERSION: u16 = 5;

const MARKET_COMMONS: &[u8] = b"MarketCommons";
const MARKETS: &[u8] = b"Markets";

#[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct LegacyReport<AccountId, BlockNumber> {
    pub at: BlockNumber,
    pub by: AccountId,
    pub outcome: OutcomeReport,
}

#[derive(Clone, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct LegacyMarket<AI, BN, M> {
    pub creator: AI,
    pub creation: MarketCreation,
    pub creator_fee: u8,
    pub oracle: AI,
    pub metadata: Vec<u8>,
    pub market_type: MarketType,
    pub period: MarketPeriod<BN, M>,
    pub deadlines: Deadlines<BN>,
    pub scoring_rule: ScoringRule,
    pub status: MarketStatus,
    pub report: Option<LegacyReport<AI, BN>>,
    pub resolved_outcome: Option<OutcomeReport>,
    pub dispute_mechanism: MarketDisputeMechanism,
}

type LegacyMarketOf<T> = LegacyMarket<
    <T as frame_system::Config>::AccountId,
    <T as frame_system::Config>::BlockNumber,
    MomentOf<T>,
>;
type MarketOf<T> = Market<
    <T as frame_system::Config>::AccountId,
    BalanceOf<T>,
    <T as frame_system::Config>::BlockNumber,
    MomentOf<T>,
>;

pub struct UpdateMarketsForOutsiderReport<T>(PhantomData<T>);

impl<T: Config + zrml_market_commons::Config> OnRuntimeUpgrade
    for UpdateMarketsForOutsiderReport<T>
{
    fn on_runtime_upgrade() -> frame_support::weights::Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);
        let storage_version = utility::get_on_chain_storage_version_of_market_commons_pallet();
        if storage_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION {
            log::info!("Skipping updates of markets; market-commons already up to date");
            return total_weight;
        }
        log::info!("Starting updates of markets");
        let mut new_markets_data: Vec<(Vec<u8>, MarketOf<T>)> = Vec::new();
        for (key, legacy_market) in storage_iter::<LegacyMarketOf<T>>(MARKET_COMMONS, MARKETS) {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
            let mut new_report = None;
            if let Some(old_report) = legacy_market.report {
                new_report = Some(Report {
                    at: old_report.at,
                    by: old_report.by,
                    outcome: old_report.outcome,
                    outsider_bond: None,
                });
            }
            let new_market = Market {
                creator: legacy_market.creator,
                creation: legacy_market.creation,
                creator_fee: legacy_market.creator_fee,
                oracle: legacy_market.oracle,
                metadata: legacy_market.metadata,
                market_type: legacy_market.market_type,
                period: legacy_market.period,
                scoring_rule: legacy_market.scoring_rule,
                status: legacy_market.status,
                report: new_report,
                resolved_outcome: legacy_market.resolved_outcome,
                dispute_mechanism: legacy_market.dispute_mechanism,
                deadlines: legacy_market.deadlines,
            };
            new_markets_data.push((key, new_market));
        }
        for (key, new_market) in new_markets_data {
            put_storage_value::<MarketOf<T>>(MARKET_COMMONS, MARKETS, &key, new_market);
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        }
        log::info!("Completed updates of markets");
        utility::put_storage_version_of_market_commons_pallet(MARKET_COMMONS_NEXT_STORAGE_VERSION);
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
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
mod tests {
    use super::*;
    use crate::{mock::*, Pallet};
    use alloc::{vec, vec::Vec};
    use frame_support::{pallet_prelude::StorageVersion, storage::unhashed};
    use zeitgeist_primitives::{
        constants::mock::OutsiderBond,
        types::{
            Deadlines, Market, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus,
            MarketType, OutcomeReport, Report,
        },
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    #[test]
    fn test_on_runtime_upgrade_on_untouched_chain() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            UpdateMarketsForOutsiderReport::<Runtime>::on_runtime_upgrade();
        });
    }

    #[test]
    fn on_runtime_upgrade_updates_storage_versions() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            UpdateMarketsForOutsiderReport::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<Pallet<Runtime>>(),
                MARKET_COMMONS_NEXT_STORAGE_VERSION
            );
        });
    }

    #[test]
    fn test_individual() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();

            System::set_block_number(1);

            UpdateMarketsForOutsiderReport::<Runtime>::on_runtime_upgrade();
        });
    }

    fn setup_chain() {
        StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
        let key = utility::storage_prefix_of_market_common_pallet();
        unhashed::put(&key, &StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION));
    }

    fn _create_test_data_for_market_update()
    -> (Vec<LegacyMarketOf<Runtime>>, Vec<MarketOf<Runtime>>) {
        let deadlines = Deadlines {
            grace_period: 0_u32.into(),
            oracle_duration: 0u32.into(),
            dispute_duration: 0u32.into(),
        };
        let old_markets: Vec<LegacyMarketOf<Runtime>> = vec![
            LegacyMarket {
                creator: 1_u128,
                creation: MarketCreation::Permissionless,
                creator_fee: 100_u8,
                oracle: 2_u128,
                metadata: vec![],
                market_type: MarketType::Categorical(2),
                period: MarketPeriod::Block(1..10),
                scoring_rule: ScoringRule::CPMM,
                status: MarketStatus::Reported,
                report: Some(LegacyReport {
                    at: 0,
                    by: DAVE,
                    outcome: OutcomeReport::Scalar(1_u128),
                }),
                resolved_outcome: None,
                dispute_mechanism: MarketDisputeMechanism::SimpleDisputes,
                deadlines,
            },
            LegacyMarket {
                creator: 1_u128,
                creation: MarketCreation::Advised,
                creator_fee: 100_u8,
                oracle: 2_u128,
                metadata: vec![],
                market_type: MarketType::Scalar(1_u128..=5_u128),
                period: MarketPeriod::Timestamp(1..10),
                scoring_rule: ScoringRule::CPMM,
                status: MarketStatus::Disputed,
                report: Some(LegacyReport {
                    at: 2,
                    by: EVE,
                    outcome: OutcomeReport::Scalar(3_u128),
                }),
                resolved_outcome: None,
                dispute_mechanism: MarketDisputeMechanism::Authorized,
                deadlines,
            },
        ];
        let expected_markets: Vec<MarketOf<Runtime>> = vec![
            Market {
                creator: 1_u128,
                creation: MarketCreation::Permissionless,
                creator_fee: 100_u8,
                oracle: 2_u128,
                metadata: vec![],
                market_type: MarketType::Categorical(2),
                period: MarketPeriod::Block(1..10),
                scoring_rule: ScoringRule::CPMM,
                status: MarketStatus::Reported,
                report: Some(Report {
                    at: 0,
                    by: DAVE,
                    outcome: OutcomeReport::Scalar(1_u128),
                    outsider_bond: Some(OutsiderBond::get()),
                }),
                resolved_outcome: None,
                dispute_mechanism: MarketDisputeMechanism::SimpleDisputes,
                deadlines,
            },
            Market {
                creator: 1_u128,
                creation: MarketCreation::Advised,
                creator_fee: 100_u8,
                oracle: 2_u128,
                metadata: vec![],
                market_type: MarketType::Scalar(1_u128..=5_u128),
                period: MarketPeriod::Timestamp(1..10),
                scoring_rule: ScoringRule::CPMM,
                status: MarketStatus::Disputed,
                report: Some(Report {
                    at: 2,
                    by: EVE,
                    outcome: OutcomeReport::Scalar(3_u128),
                    outsider_bond: Some(OutsiderBond::get()),
                }),
                resolved_outcome: None,
                dispute_mechanism: MarketDisputeMechanism::Authorized,
                deadlines,
            },
        ];
        (old_markets, expected_markets)
    }

    #[test]
    fn test_on_runtime_upgrade() {
        ExtBuilder::default().build().execute_with(|| {});
    }

    fn _create_test_market() {
        let deadlines = Deadlines {
            grace_period: <Runtime as crate::Config>::MaxGracePeriod::get(),
            oracle_duration: <Runtime as crate::Config>::MaxOracleDuration::get(),
            dispute_duration: <Runtime as crate::Config>::MaxDisputeDuration::get(),
        };
        let mut metadata = [0; 50];
        metadata[0] = 0x15;
        metadata[1] = 0x30;
        let market = Market {
            creation: MarketCreation::Advised,
            creator_fee: 0,
            creator: ALICE,
            market_type: MarketType::Categorical(5),
            dispute_mechanism: MarketDisputeMechanism::Authorized,
            metadata: Vec::from(metadata),
            oracle: BOB,
            period: MarketPeriod::Block(2..10),
            deadlines,
            report: Some(Report {
                at: 0,
                by: DAVE,
                outcome: OutcomeReport::Scalar(1_u128),
                outsider_bond: Some(OutsiderBond::get()),
            }),
            resolved_outcome: None,
            status: MarketStatus::Reported,
            scoring_rule: zeitgeist_primitives::types::ScoringRule::CPMM,
        };
        let _res = <MarketCommons as MarketCommonsPalletApi>::push_market(market);
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
