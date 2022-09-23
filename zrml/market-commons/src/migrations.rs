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
use frame_support::{
    log,
    migration::{put_storage_value, storage_iter},
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
};
use sp_runtime::traits::SaturatedConversion;
extern crate alloc;
use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};
use zeitgeist_primitives::{
    constants::BLOCKS_PER_DAY,
    types::{
        Deadlines, Market, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus,
        MarketType, OutcomeReport, Report, ScoringRule,
    },
};
const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 1;
const MARKET_COMMONS_NEXT_STORAGE_VERSION: u16 = 2;

const MARKET_COMMONS: &[u8] = b"MarketCommons";
const MARKETS: &[u8] = b"Markets";

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
    pub dispute_mechanism: MarketDisputeMechanism<AI>,
}

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

pub struct UpdateMarketsForDeadlines<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for UpdateMarketsForDeadlines<T> {
    fn on_runtime_upgrade() -> frame_support::weights::Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);
        let storage_version = StorageVersion::get::<Pallet<T>>();
        if storage_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION {
            log::info!("Skipping updates of markets; prediction-markets already up to date");
            return total_weight;
        }
        log::info!("Starting updates of markets");
        let dispute_duration = if cfg!(feature = "with-zeitgeist-runtime") {
            (4_u64 * BLOCKS_PER_DAY).saturated_into::<u32>().into()
        } else {
            // assuming battery-station
            BLOCKS_PER_DAY.saturated_into::<u32>().into()
        };
        let oracle_duration: T::BlockNumber = BLOCKS_PER_DAY.saturated_into::<u32>().into();
        let deadlines = Deadlines { grace_period: 0_u32.into(), oracle_duration, dispute_duration };
        for (key, legacy_market) in storage_iter::<LegacyMarketOf<T>>(MARKET_COMMONS, MARKETS) {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
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
                dispute_mechanism: legacy_market.dispute_mechanism,
                deadlines,
            };
            put_storage_value::<MarketOf<T>>(MARKET_COMMONS, MARKETS, &key, new_market);
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        }
        log::info!("Completed updates of markets");
        StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        let dispute_duration = if cfg!(feature = "with-zeitgeist-runtime") {
            (4_u64 * BLOCKS_PER_DAY).saturated_into::<u32>().into()
        } else {
            // assuming battery-station
            BLOCKS_PER_DAY.saturated_into::<u32>().into()
        };
        let oracle_duration: T::BlockNumber = BLOCKS_PER_DAY.saturated_into::<u32>().into();
        let deadlines = Deadlines { grace_period: 0_u32.into(), oracle_duration, dispute_duration };
        for (market_id, market) in storage_iter::<MarketOf<T>>(MARKET_COMMONS, MARKETS) {
            assert_eq!(
                market.deadlines, deadlines,
                "found unexpected deadlines in market. market_id: {:?}, deadlines: {:?}",
                market_id, market.deadlines
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock::{ExtBuilder, Runtime},
        Markets,
    };
    use core::fmt::Debug;
    use frame_support::{Blake2_128Concat, StorageHasher};
    use parity_scale_codec::Encode;

    type MarketId = <Runtime as Config>::MarketId;

    #[test]
    fn test_on_runtime_upgrade() {
        ExtBuilder::default().build().execute_with(|| {
            StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
            let (legacy_markets, expected_markets) = create_test_data();
            populate_test_data::<Blake2_128Concat, MarketId, LegacyMarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                legacy_markets,
            );
            UpdateMarketsForDeadlines::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<Pallet<Runtime>>(),
                MARKET_COMMONS_NEXT_STORAGE_VERSION
            );
            for (market_id, market_expected) in expected_markets.iter().enumerate() {
                let market_actual = Markets::<Runtime>::get(market_id as u128).unwrap();
                assert_eq!(market_actual, *market_expected);
            }
        });
    }

    fn populate_test_data<H, K, V>(pallet: &[u8], prefix: &[u8], data: Vec<V>)
    where
        H: StorageHasher,
        K: TryFrom<usize> + Encode,
        V: Encode + Clone,
        <K as TryFrom<usize>>::Error: Debug,
    {
        for (key, value) in data.iter().enumerate() {
            let storage_hash = key_to_hash::<H, K>(key);
            put_storage_value::<V>(pallet, prefix, &storage_hash, (*value).clone());
        }
    }

    fn key_to_hash<H, K>(key: usize) -> Vec<u8>
    where
        H: StorageHasher,
        K: TryFrom<usize> + Encode,
        <K as TryFrom<usize>>::Error: Debug,
    {
        K::try_from(key).unwrap().using_encoded(H::hash).as_ref().to_vec()
    }

    fn create_test_data() -> (Vec<LegacyMarketOf<Runtime>>, Vec<MarketOf<Runtime>>) {
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
                dispute_mechanism: MarketDisputeMechanism::SimpleDisputes,
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
                dispute_mechanism: MarketDisputeMechanism::Authorized(3_u128),
            },
        ];
        let dispute_duration = if cfg!(feature = "with-zeitgeist-runtime") {
            (4_u64 * BLOCKS_PER_DAY).saturated_into::<u32>().into()
        } else {
            // assuming battery-station
            BLOCKS_PER_DAY.saturated_into::<u32>().into()
        };
        let oracle_duration: <Runtime as frame_system::Config>::BlockNumber =
            BLOCKS_PER_DAY.saturated_into::<u32>().into();
        let deadlines = Deadlines { grace_period: 0_u32.into(), oracle_duration, dispute_duration };
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
                status: MarketStatus::Active,
                report: None,
                resolved_outcome: None,
                dispute_mechanism: MarketDisputeMechanism::Authorized(3_u128),
                deadlines,
            },
        ];
        (old_markets, expected_markets)
    }
}
