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

use crate::{Config, Pallet};
use alloc::vec::Vec;
use alloc::string::ToString;
use frame_support::{
    log,
    migration::{put_storage_value, storage_iter},
    pallet_prelude::{PhantomData, StorageVersion},
    traits::{Get, OnRuntimeUpgrade},
    RuntimeDebug,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
#[cfg(feature = "try-runtime")]
use scale_info::prelude::format;
use scale_info::TypeInfo;
use zeitgeist_primitives::types::{
    Deadlines, Market, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus,
    MarketType, OutcomeReport, Report, ScoringRule,
};
const MARKET_COMMONS: &[u8] = b"MarketCommons";
const MARKETS: &[u8] = b"Markets";
const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 3_u16;
const MARKET_COMMONS_NEXT_STORAGE_VERSION: u16 = 4_u16;

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
    pub report: Option<Report<AI, BN>>,
    pub resolved_outcome: Option<OutcomeReport>,
    pub dispute_mechanism: LegacyMarketDisputeMechanism<AI>,
}

#[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
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
        let storage_version = StorageVersion::get::<Pallet<T>>();
        if storage_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION {
            log::info!("Skipping UpdateMarketsForAuthorizedMDM; markets already up to date");
            return total_weight;
        }
        let mut new_markets_data = Vec::new();
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
        StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        assert_eq!(StorageVersion::get::<Pallet<T>>(), MARKET_COMMONS_REQUIRED_STORAGE_VERSION);
        use frame_support::traits::OnRuntimeUpgradeHelpersExt;
        let legacy_markets_count_key = "legacy_markets_count_key".to_string();
        Self::set_temp_storage(0_u32, &legacy_markets_count_key);
        for (key, legacy_market) in storage_iter::<LegacyMarketOf<T>>(MARKET_COMMONS, MARKETS) {
            Self::set_temp_storage(legacy_market, &format!("{:?}", key.as_slice()));
            let legacy_markets_count: u32 = Self::get_temp_storage(&legacy_markets_count_key)
                .expect("legacy_markets_count_key storage not found");
            Self::set_temp_storage(legacy_markets_count + 1_u32, &legacy_markets_count_key);
        }
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        assert_eq!(StorageVersion::get::<Pallet<T>>(), MARKET_COMMONS_NEXT_STORAGE_VERSION);
        use frame_support::traits::OnRuntimeUpgradeHelpersExt;
        let mut markets_count = 0_u32;
        let legacy_markets_count_key = "legacy_markets_count_key".to_string();
        for (key, updated_market) in storage_iter::<MarketOf<T>>(MARKET_COMMONS, MARKETS) {
            let legacy_market: LegacyMarketOf<T> =
                Self::get_temp_storage(&format!("{:?}", key.as_slice()))
                    .expect("legacy market not found");
            assert_eq!(updated_market.creator, legacy_market.creator);
            assert_eq!(updated_market.creation, legacy_market.creation);
            assert_eq!(updated_market.creator_fee, legacy_market.creator_fee);
            assert_eq!(updated_market.oracle, legacy_market.oracle);
            assert_eq!(updated_market.metadata, legacy_market.metadata);
            assert_eq!(updated_market.market_type, legacy_market.market_type);
            assert_eq!(updated_market.period, legacy_market.period);
            assert_eq!(updated_market.deadlines, legacy_market.deadlines);
            assert_eq!(updated_market.scoring_rule, legacy_market.scoring_rule);
            assert_eq!(updated_market.status, legacy_market.status);
            assert_eq!(updated_market.report, legacy_market.report);
            assert_eq!(updated_market.resolved_outcome, legacy_market.resolved_outcome);
            markets_count += 1_u32;
        }
        let legacy_markets_count: u32 = Self::get_temp_storage(&legacy_markets_count_key)
            .expect("temp_market_counts_key storage not found");
        assert_eq!(markets_count, legacy_markets_count);
        Ok(())
    }
}

mod utility {
    use alloc::vec::Vec;
    use frame_support::StorageHasher;
    use parity_scale_codec::Encode;

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
            StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
            let (legacy_markets, expected_markets) = create_test_data_for_market_update();
            populate_test_data::<Blake2_128Concat, MarketId, LegacyMarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                legacy_markets,
            );
            UpdateMarketsForAuthorizedMDM::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<Pallet<Runtime>>(),
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
                dispute_mechanism: LegacyMarketDisputeMechanism::Court,
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
                dispute_mechanism: MarketDisputeMechanism::Court,
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
