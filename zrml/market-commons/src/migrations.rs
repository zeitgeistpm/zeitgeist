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

use crate::Config;
use frame_support::{
    log,
    migration::{put_storage_value, storage_iter},
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade},
};
use parity_scale_codec::{Decode, Encode};
use zeitgeist_primitives::types::{
    Deadlines, Market, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus,
    MarketType, OutcomeReport, Report, ScoringRule,
};
use alloc::vec::Vec;

const MARKET_COMMONS: &[u8] = b"MarketCommons";
const MARKETS: &[u8] = b"Markets";
const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 2_u16;
const MARKET_COMMONS_NEXT_STORAGE_VERSION: u16 = 3_u16;

#[derive(Clone, Decode, Encode)]
pub struct LegacyMarket<AI, BN, M> {
    pub creator: AI,
    pub creation: MarketCreation,
    pub creator_fee: u8,
    pub oracle: AI,
    pub metadata: Vec<u8>,
    pub market_type: MarketType,
    pub period: MarketPeriod<BN, M>,
    pub scoring_rule: ScoringRule,
    pub status: MarketStatus,
    pub report: Option<Report<AI, BN>>,
    pub resolved_outcome: Option<OutcomeReport>,
    pub dispute_mechanism: LegacyMarketDisputeMechanism<AI>,
    pub deadlines: Deadlines<BN>,
}

#[derive(Clone, Decode, Encode)]
pub enum LegacyMarketDisputeMechanism<AI> {
    Authorized(AI),
    Court,
    SimpleDisputes,
}

type MomentOf<T> = <<T as Config>::Timestamp as frame_support::traits::Time>::Moment;

type LegacyMarketOf<T> = LegacyMarket<
    <T as frame_system::Config>::AccountId,
    <T as frame_system::Config>::BlockNumber,
    MomentOf<T>,
>;
type MarketOf<T> = Market<
    <T as frame_system::Config>::AccountId,
    <T as frame_system::Config>::BlockNumber,
    MomentOf<T>,
>;

pub struct UpdateMarketsForAuthorizedMDM<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for UpdateMarketsForAuthorizedMDM<T> {
    fn on_runtime_upgrade() -> frame_support::weights::Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);
        let storage_version = utility::get_on_chain_storage_version_of_market_commons_pallet();
        if storage_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION {
            log::info!("Skipping updates of markets; markets already up to date");
            return total_weight;
        }
        let mut new_markets_data: Vec<(Vec<u8>, MarketOf<T>)> = Vec::new();
        for (key, legacy_market) in storage_iter::<LegacyMarketOf<T>>(MARKET_COMMONS, MARKETS) {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
            let dispute_mechanism = match legacy_market.dispute_mechanism {
                LegacyMarketDisputeMechanism::Authorized(_) => MarketDisputeMechanism::Authorized,
                LegacyMarketDisputeMechanism::Court => MarketDisputeMechanism::Court,
                LegacyMarketDisputeMechanism::SimpleDisputes => {
                    MarketDisputeMechanism::SimpleDisputes
                }
            };
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
                report: legacy_market.report,
                resolved_outcome: legacy_market.resolved_outcome,
                dispute_mechanism,
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

mod utility {
    use alloc::vec::Vec;
    use frame_support::{
        storage::{storage_prefix, unhashed},
        traits::StorageVersion,
        StorageHasher,
    };
    use parity_scale_codec::Encode;

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
    pub fn key_to_hash<H, K>(key: K) -> Vec<u8>
    where
        H: StorageHasher,
        K: Encode,
    {
        key.using_encoded(H::hash).as_ref().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{market_commons_pallet_api::MarketCommonsPalletApi, mock::*};
    use alloc::{vec, vec::Vec};
    use core::fmt::Debug;
    use frame_support::{Blake2_128Concat, StorageHasher};
    use parity_scale_codec::Encode;
    use zeitgeist_primitives::types::{
        Deadlines, Market, MarketCreation, MarketDisputeMechanism, MarketId, MarketPeriod,
        MarketStatus, MarketType,
    };

    #[test]
    fn test_on_runtime_upgrade() {
        ExtBuilder::default().build().execute_with(|| {
            utility::put_storage_version_of_market_commons_pallet(
                MARKET_COMMONS_REQUIRED_STORAGE_VERSION,
            );
            let (legacy_markets, expected_markets) = create_test_data_for_market_update();
            populate_test_data::<Blake2_128Concat, MarketId, LegacyMarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                legacy_markets,
            );
            UpdateMarketsForAuthorizedMDM::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                utility::get_on_chain_storage_version_of_market_commons_pallet(),
                MARKET_COMMONS_NEXT_STORAGE_VERSION
            );
            for (market_id, market_expected) in expected_markets.iter().enumerate() {
                let market_actual = <crate::Pallet<Runtime> as MarketCommonsPalletApi>::market(
                    &(market_id as u128),
                )
                .unwrap();
                assert_eq!(market_actual, *market_expected);
            }
        });
    }

    fn create_test_data_for_market_update() -> (Vec<LegacyMarketOf<Runtime>>, Vec<MarketOf<Runtime>>)
    {
        let deadlines = Deadlines {
            grace_period: 2_u32.into(),
            oracle_duration: 2_u32.into(),
            dispute_duration: 2_u32.into(),
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
                status: MarketStatus::Proposed,
                report: None,
                resolved_outcome: None,
                dispute_mechanism: LegacyMarketDisputeMechanism::Authorized(2_u128),
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
                status: MarketStatus::Active,
                report: None,
                resolved_outcome: None,
                dispute_mechanism: LegacyMarketDisputeMechanism::Authorized(3_u128),
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
                status: MarketStatus::Proposed,
                report: None,
                resolved_outcome: None,
                dispute_mechanism: MarketDisputeMechanism::Authorized,
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
                status: MarketStatus::Active,
                report: None,
                resolved_outcome: None,
                dispute_mechanism: MarketDisputeMechanism::Authorized,
                deadlines,
            },
        ];
        (old_markets, expected_markets)
    }
    fn populate_test_data<H, K, V>(pallet: &[u8], prefix: &[u8], data: Vec<V>)
    where
        H: StorageHasher,
        K: TryFrom<usize> + Encode,
        V: Encode + Clone,
        <K as TryFrom<usize>>::Error: Debug,
    {
        for (key, value) in data.iter().enumerate() {
            let storage_hash = utility::key_to_hash::<H, K>(
                K::try_from(key).expect("usize to K conversion failed"),
            );
            put_storage_value::<V>(pallet, prefix, &storage_hash, (*value).clone());
        }
    }
}
