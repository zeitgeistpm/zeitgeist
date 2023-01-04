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
#![allow(unused_imports)]
use crate::{Config, MarketIdOf, MomentOf, Pallet};
use frame_support::{
    log,
    migration::{put_storage_value, storage_iter, storage_key_iter},
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade},
    Blake2_128Concat,
};
extern crate alloc;
use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};
use zeitgeist_primitives::{
    traits::MarketCommonsPalletApi,
    types::{
        Asset, Deadlines, Market, MarketCreation, MarketDisputeMechanism, MarketPeriod,
        MarketStatus, MarketType, OutcomeReport, Report, ScoringRule,
    },
};

const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 4;
const MARKET_COMMONS_NEXT_STORAGE_VERSION: u16 = 5;

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
    pub deadlines: Deadlines<BN>,
    pub scoring_rule: ScoringRule,
    pub status: MarketStatus,
    pub report: Option<Report<AI, BN>>,
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
    <T as frame_system::Config>::BlockNumber,
    MomentOf<T>,
    Asset<MarketIdOf<T>>,
>;

pub struct UpdateMarketsForBaseAsset<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for UpdateMarketsForBaseAsset<T> {
    fn on_runtime_upgrade() -> frame_support::weights::Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);
        let storage_version = utility::get_on_chain_storage_version_of_market_commons_pallet();
        if storage_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "Skipping updates of markets; market-commons storage version is {:?}, required \
                 {:?}",
                storage_version,
                MARKET_COMMONS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("Starting updates of markets");
        let mut new_markets_data: Vec<(Vec<u8>, MarketOf<T>)> = Vec::new();
        for (key, legacy_market) in storage_iter::<LegacyMarketOf<T>>(MARKET_COMMONS, MARKETS) {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
            let new_market = Market {
                base_asset: Asset::Ztg,
                creator: legacy_market.creator,
                creation: legacy_market.creation,
                creator_fee: legacy_market.creator_fee,
                oracle: legacy_market.oracle,
                metadata: legacy_market.metadata,
                market_type: legacy_market.market_type,
                period: legacy_market.period,
                deadlines: legacy_market.deadlines,
                scoring_rule: legacy_market.scoring_rule,
                status: legacy_market.status,
                report: legacy_market.report,
                resolved_outcome: legacy_market.resolved_outcome,
                dispute_mechanism: legacy_market.dispute_mechanism,
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
        use alloc::string::ToString;
        use frame_support::traits::OnRuntimeUpgradeHelpersExt;
        use scale_info::prelude::format;
        let legacy_markets_count_key = "legacy_markets_count_key".to_string();
        let mut market_count = 0_u32;
        for (market_id, legacy_market) in storage_key_iter::<
            <T as Config>::MarketId,
            LegacyMarketOf<T>,
            Blake2_128Concat,
        >(MARKET_COMMONS, MARKETS)
        {
            Self::set_temp_storage(legacy_market, &format!("{:?}", market_id));
            market_count += 1_u32;
        }
        Self::set_temp_storage(market_count, &legacy_markets_count_key);
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        use alloc::string::ToString;
        use frame_support::traits::OnRuntimeUpgradeHelpersExt;
        use scale_info::prelude::format;
        let mut markets_count = 0_u32;
        let legacy_markets_count_key = "legacy_markets_count_key".to_string();
        for (market_id, updated_market) in <Pallet<T>>::market_iter() {
            // for (market_id, updated_market) in storage_iter::<MarketOf<T>>(MARKET_COMMONS, MARKETS) {
            assert_eq!(
                updated_market.base_asset,
                Asset::Ztg,
                "found unexpected base_asset in market. market_id: {:?}, base_asset: {:?}",
                market_id,
                updated_market.base_asset
            );
            let legacy_market: LegacyMarketOf<T> =
                Self::get_temp_storage(&format!("{:?}", market_id)).unwrap_or_else(|| {
                    panic!("legacy market not found for market_id {:?}", market_id)
                });
            assert_eq!(updated_market.creator, legacy_market.creator);
            assert_eq!(updated_market.creation, legacy_market.creation);
            assert_eq!(updated_market.creator_fee, legacy_market.creator_fee);
            assert_eq!(updated_market.oracle, legacy_market.oracle);
            assert_eq!(updated_market.metadata, legacy_market.metadata);
            assert_eq!(updated_market.market_type, legacy_market.market_type);
            assert_eq!(updated_market.report, legacy_market.report);
            assert_eq!(updated_market.resolved_outcome, legacy_market.resolved_outcome);
            assert_eq!(updated_market.period, legacy_market.period);
            assert_eq!(updated_market.deadlines, legacy_market.deadlines);
            assert_eq!(updated_market.scoring_rule, legacy_market.scoring_rule);
            assert_eq!(updated_market.status, legacy_market.status);
            assert_eq!(updated_market.dispute_mechanism, legacy_market.dispute_mechanism);
            markets_count += 1_u32;
        }
        let legacy_markets_count: u32 = Self::get_temp_storage(&legacy_markets_count_key)
            .expect("temp_market_counts_key storage not found");
        assert_eq!(markets_count, legacy_markets_count);
        Ok(())
    }
}

// We use these utilities to prevent having to make the swaps pallet a dependency of
// prediciton-markets. The calls are based on the implementation of `StorageVersion`, found here:
// https://github.com/paritytech/substrate/blob/bc7a1e6c19aec92bfa247d8ca68ec63e07061032/frame/support/src/traits/metadata.rs#L168-L230
// and previous migrations.
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{mock::*, MarketCommonsPalletApi, Pallet};
    use alloc::{vec, vec::Vec};
    use core::fmt::Debug;
    use frame_support::{pallet_prelude::StorageVersion, Blake2_128Concat, StorageHasher};
    use parity_scale_codec::Encode;
    use zeitgeist_primitives::types::{
        Deadlines, Market, MarketCreation, MarketDisputeMechanism, MarketId, MarketPeriod,
        MarketStatus, MarketType,
    };

    #[test]
    fn test_on_runtime_upgrade_on_empty_chain() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            UpdateMarketsForBaseAsset::<Runtime>::on_runtime_upgrade();
        });
    }

    #[test]
    fn test_on_runtime_upgrade_with_storate_version_not_equal_to_required() {
        ExtBuilder::default().build().execute_with(|| {
            StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION + 1)
                .put::<Pallet<Runtime>>();
            let (_legacy_markets, expected_markets) = create_test_data_for_market_update();
            populate_test_data::<Blake2_128Concat, MarketId, MarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                expected_markets.clone(),
            );
            UpdateMarketsForBaseAsset::<Runtime>::on_runtime_upgrade();
            // verify no change in storage version
            assert_eq!(
                utility::get_on_chain_storage_version_of_market_commons_pallet(),
                MARKET_COMMONS_REQUIRED_STORAGE_VERSION + 1
            );
            // verify that nothing changed in storage
            for (market_id, market_expected) in expected_markets.iter().enumerate() {
                let market_actual = MarketCommons::market(&(market_id as u128)).unwrap();
                assert_eq!(market_actual, *market_expected);
            }
        });
    }

    #[test]
    fn on_runtime_upgrade_updates_storage_versions() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            UpdateMarketsForBaseAsset::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<Pallet<Runtime>>(),
                MARKET_COMMONS_NEXT_STORAGE_VERSION
            );
        });
    }

    fn setup_chain() {
        StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
    }

    fn create_test_data_for_market_update() -> (Vec<LegacyMarketOf<Runtime>>, Vec<MarketOf<Runtime>>)
    {
        let deadlines = Deadlines {
            grace_period: 0_u32.into(),
            oracle_duration: 5_u32.into(),
            dispute_duration: 5_u32.into(),
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
                deadlines,
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
                deadlines,
                scoring_rule: ScoringRule::CPMM,
                status: MarketStatus::Active,
                report: None,
                resolved_outcome: None,
                dispute_mechanism: MarketDisputeMechanism::Authorized,
            },
        ];
        let expected_markets: Vec<MarketOf<Runtime>> = vec![
            Market {
                base_asset: Asset::Ztg,
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
                base_asset: Asset::Ztg,
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
            UpdateMarketsForBaseAsset::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                utility::get_on_chain_storage_version_of_market_commons_pallet(),
                MARKET_COMMONS_NEXT_STORAGE_VERSION
            );
            for (market_id, market_expected) in expected_markets.iter().enumerate() {
                let market_actual = MarketCommons::market(&(market_id as u128)).unwrap();
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
            let storage_hash = utility::key_to_hash::<H, K>(
                K::try_from(key).expect("usize to K conversion failed"),
            );
            put_storage_value::<V>(pallet, prefix, &storage_hash, (*value).clone());
        }
    }
}
