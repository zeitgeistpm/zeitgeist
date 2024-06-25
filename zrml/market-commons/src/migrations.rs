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

use crate::{AccountIdOf, BalanceOf, Config, MomentOf, Pallet as MarketCommons};
use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::{
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    weights::Weight,
};
use frame_system::pallet_prelude::BlockNumberFor;
use log;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{Perbill, RuntimeDebug, Saturating};
use zeitgeist_primitives::types::{
    BaseAsset, Deadlines, EarlyClose, Market, MarketBonds, MarketCreation, MarketDisputeMechanism,
    MarketPeriod, MarketStatus, MarketType, OutcomeReport, Report, ScoringRule,
};

#[cfg(feature = "try-runtime")]
use {
    crate::MarketIdOf,
    alloc::{collections::BTreeMap, format},
    frame_support::migration::storage_key_iter,
    sp_runtime::DispatchError,
};

#[cfg(any(feature = "try-runtime", feature = "test"))]
use frame_support::Blake2_128Concat;

#[cfg(any(feature = "try-runtime", test))]
const MARKET_COMMONS: &[u8] = b"MarketCommons";
#[cfg(any(feature = "try-runtime", test))]
const MARKETS: &[u8] = b"Markets";

#[derive(Clone, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct OldMarket<AI, BA, BN, M, A> {
    pub base_asset: A,
    pub creator: AI,
    pub creation: MarketCreation,
    pub creator_fee: Perbill,
    pub oracle: AI,
    pub metadata: Vec<u8>,
    pub market_type: MarketType,
    pub period: MarketPeriod<BN, M>,
    pub deadlines: Deadlines<BN>,
    pub scoring_rule: OldScoringRule,
    pub status: MarketStatus,
    pub report: Option<Report<AI, BN>>,
    pub resolved_outcome: Option<OutcomeReport>,
    pub dispute_mechanism: Option<MarketDisputeMechanism>,
    pub bonds: MarketBonds<AI, BA>,
    pub early_close: Option<EarlyClose<BN, M>>,
}

type OldMarketOf<T> =
    OldMarket<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, MomentOf<T>, BaseAsset>;

#[derive(TypeInfo, Clone, Copy, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug)]
pub enum OldScoringRule {
    Lmsr,
    Orderbook,
    Parimutuel,
}

const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 10;
const MARKET_COMMONS_NEXT_STORAGE_VERSION: u16 = 11;

#[cfg(feature = "try-runtime")]
#[frame_support::storage_alias]
pub(crate) type Markets<T: Config> =
    StorageMap<MarketCommons<T>, Blake2_128Concat, MarketIdOf<T>, OldMarketOf<T>>;

pub struct MigrateScoringRuleAmmCdaHybridAndMarketId<T>(PhantomData<T>);

/// Migrates AMM and CDA markets to the new combined scoring rule and adds the market's ID to the
/// struct.
impl<T> OnRuntimeUpgrade for MigrateScoringRuleAmmCdaHybridAndMarketId<T>
where
    T: Config,
{
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let market_commons_version = StorageVersion::get::<MarketCommons<T>>();
        if market_commons_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "MigrateScoringRuleAmmCdaHybridAndMarketId: market-commons version is {:?}, but \
                 {:?} is required",
                market_commons_version,
                MARKET_COMMONS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("MigrateScoringRuleAmmCdaHybridAndMarketId: Starting...");

        let mut translated = 0u64;
        crate::Markets::<T>::translate::<OldMarketOf<T>, _>(|market_id, old_market| {
            translated.saturating_inc();
            let scoring_rule = match old_market.scoring_rule {
                OldScoringRule::Lmsr => ScoringRule::AmmCdaHybrid,
                OldScoringRule::Orderbook => ScoringRule::AmmCdaHybrid,
                OldScoringRule::Parimutuel => ScoringRule::Parimutuel,
            };
            let new_market = Market {
                market_id,
                base_asset: old_market.base_asset,
                creator: old_market.creator,
                creation: old_market.creation,
                creator_fee: old_market.creator_fee,
                oracle: old_market.oracle,
                metadata: old_market.metadata,
                market_type: old_market.market_type,
                period: old_market.period,
                deadlines: old_market.deadlines,
                scoring_rule,
                status: old_market.status,
                report: old_market.report,
                resolved_outcome: old_market.resolved_outcome,
                dispute_mechanism: old_market.dispute_mechanism,
                bonds: old_market.bonds,
                early_close: old_market.early_close,
            };
            Some(new_market)
        });
        log::info!("MigrateScoringRuleAmmCdaHybridAndMarketId: Upgraded {} markets.", translated);
        total_weight =
            total_weight.saturating_add(T::DbWeight::get().reads_writes(translated, translated));

        StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION).put::<MarketCommons<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("MigrateScoringRuleAmmCdaHybridAndMarketId: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, DispatchError> {
        let old_markets = storage_key_iter::<MarketIdOf<T>, OldMarketOf<T>, Blake2_128Concat>(
            MARKET_COMMONS,
            MARKETS,
        )
        .collect::<BTreeMap<_, _>>();
        let markets = Markets::<T>::iter_keys().count();
        let decodable_markets = Markets::<T>::iter_values().count();
        if markets == decodable_markets {
            log::info!("All {} markets could successfully be decoded.", markets);
        } else {
            log::error!(
                "Can only decode {} of {} markets - others will be dropped.",
                decodable_markets,
                markets
            );
        }

        Ok(old_markets.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(previous_state: Vec<u8>) -> Result<(), DispatchError> {
        let old_markets: BTreeMap<MarketIdOf<T>, OldMarketOf<T>> =
            Decode::decode(&mut &previous_state[..]).unwrap();
        let old_market_count = old_markets.len();
        let new_market_count = crate::Markets::<T>::iter().count();
        assert_eq!(old_market_count, new_market_count);
        for (market_id, new_market) in crate::Markets::<T>::iter() {
            let old_market = old_markets
                .get(&market_id)
                .expect(&format!("Market {:?} not found", market_id)[..]);
            assert_eq!(market_id, new_market.market_id);
            assert_eq!(old_market.base_asset, new_market.base_asset);
            assert_eq!(old_market.creator, new_market.creator);
            assert_eq!(old_market.creation, new_market.creation);
            assert_eq!(old_market.creator_fee, new_market.creator_fee);
            assert_eq!(old_market.oracle, new_market.oracle);
            assert_eq!(old_market.metadata, new_market.metadata);
            assert_eq!(old_market.market_type, new_market.market_type);
            assert_eq!(old_market.period, new_market.period);
            assert_eq!(old_market.deadlines, new_market.deadlines);
            assert_eq!(old_market.status, new_market.status);
            assert_eq!(old_market.report, new_market.report);
            assert_eq!(old_market.resolved_outcome, new_market.resolved_outcome);
            assert_eq!(old_market.dispute_mechanism, new_market.dispute_mechanism);
            assert_eq!(old_market.bonds, new_market.bonds);
            assert_eq!(old_market.early_close, new_market.early_close);
        }
        log::info!(
            "MigrateScoringRuleAmmCdaHybridAndMarketId: Post-upgrade market count is {}!",
            new_market_count
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock::{ExtBuilder, Runtime},
        MarketOf,
    };
    use alloc::fmt::Debug;
    use frame_support::{migration::put_storage_value, Blake2_128Concat, StorageHasher};
    use parity_scale_codec::Encode;
    use sp_io::storage::root as storage_root;
    use sp_runtime::{Perbill, StateVersion};
    use test_case::test_case;
    use zeitgeist_primitives::types::{BaseAssetClass, Bond, EarlyCloseState, MarketId};

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            MigrateScoringRuleAmmCdaHybridAndMarketId::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<MarketCommons<Runtime>>(),
                MARKET_COMMONS_NEXT_STORAGE_VERSION
            );
        });
    }

    #[test_case(OldScoringRule::Orderbook, ScoringRule::AmmCdaHybrid)]
    #[test_case(OldScoringRule::Lmsr, ScoringRule::AmmCdaHybrid)]
    #[test_case(OldScoringRule::Parimutuel, ScoringRule::Parimutuel)]
    fn on_runtime_upgrade_works_as_expected(
        old_scoring_rule: OldScoringRule,
        new_scoring_rule: ScoringRule,
    ) {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            let (old_markets, new_markets) =
                construct_old_new_tuple(old_scoring_rule, new_scoring_rule);
            populate_test_data::<Blake2_128Concat, MarketId, OldMarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                old_markets,
            );
            MigrateScoringRuleAmmCdaHybridAndMarketId::<Runtime>::on_runtime_upgrade();
            assert_eq!(crate::Markets::<Runtime>::get(0).unwrap(), new_markets[0]);
            assert_eq!(crate::Markets::<Runtime>::get(1).unwrap(), new_markets[1]);
            assert_eq!(crate::Markets::<Runtime>::get(2).unwrap(), new_markets[2]);
        });
    }

    #[test]
    fn on_runtime_upgrade_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION)
                .put::<MarketCommons<Runtime>>();
            let market = Market {
                market_id: 7,
                base_asset: BaseAssetClass::Ztg,
                creator: 1,
                creation: MarketCreation::Permissionless,
                creator_fee: Perbill::from_rational(2u32, 3u32),
                oracle: 4,
                metadata: vec![0x05; 50],
                market_type: MarketType::Categorical(999),
                period: MarketPeriod::<BlockNumberFor<Runtime>, MomentOf<Runtime>>::Block(6..7),
                deadlines: Deadlines { grace_period: 7, oracle_duration: 8, dispute_duration: 9 },
                scoring_rule: ScoringRule::AmmCdaHybrid,
                status: MarketStatus::Active,
                report: Some(Report { at: 13, by: 14, outcome: OutcomeReport::Categorical(10) }),
                resolved_outcome: None,
                dispute_mechanism: Some(MarketDisputeMechanism::Court),
                bonds: MarketBonds {
                    creation: Some(Bond::new(11, 12)),
                    oracle: None,
                    outsider: None,
                    dispute: None,
                    close_dispute: None,
                    close_request: None,
                },
                early_close: None,
            };
            crate::Markets::<Runtime>::insert(333, market);
            let tmp = storage_root(StateVersion::V1);
            MigrateScoringRuleAmmCdaHybridAndMarketId::<Runtime>::on_runtime_upgrade();
            assert_eq!(tmp, storage_root(StateVersion::V1));
        });
    }

    fn set_up_version() {
        StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION)
            .put::<MarketCommons<Runtime>>();
    }

    fn construct_old_new_tuple(
        old_scoring_rule: OldScoringRule,
        new_scoring_rule: ScoringRule,
    ) -> (Vec<OldMarketOf<Runtime>>, Vec<MarketOf<Runtime>>) {
        let base_asset = BaseAsset::Ztg;
        let creation = MarketCreation::Advised;
        let creator_fee = Perbill::from_rational(2u32, 3u32);
        let oracle = 4;
        let metadata = vec![5; 50];
        let market_type = MarketType::Scalar(6..=7);
        let period = MarketPeriod::Block(8..9);
        let deadlines = Deadlines { grace_period: 10, oracle_duration: 11, dispute_duration: 12 };
        let status = MarketStatus::Resolved;
        let report = Some(Report { at: 13, by: 14, outcome: OutcomeReport::Categorical(15) });
        let resolved_outcome = Some(OutcomeReport::Categorical(16));
        let dispute_mechanism = Some(MarketDisputeMechanism::Court);
        let bonds = MarketBonds {
            creation: Some(Bond { who: 17, value: 18, is_settled: true }),
            oracle: Some(Bond { who: 19, value: 20, is_settled: true }),
            outsider: Some(Bond { who: 21, value: 22, is_settled: true }),
            dispute: Some(Bond { who: 23, value: 24, is_settled: true }),
            close_request: Some(Bond { who: 25, value: 26, is_settled: true }),
            close_dispute: Some(Bond { who: 27, value: 28, is_settled: true }),
        };
        let early_close = Some(EarlyClose {
            old: MarketPeriod::Block(29..30),
            new: MarketPeriod::Block(31..32),
            state: EarlyCloseState::Disputed,
        });
        let sentinels = (0..3).map(|i| 10 - i);
        let old_markets = sentinels
            .clone()
            .map(|sentinel| OldMarket {
                base_asset,
                creator: sentinel,
                creation: creation.clone(),
                creator_fee,
                oracle,
                metadata: metadata.clone(),
                market_type: market_type.clone(),
                period: period.clone(),
                deadlines,
                scoring_rule: old_scoring_rule,
                status,
                report: report.clone(),
                resolved_outcome: resolved_outcome.clone(),
                dispute_mechanism: dispute_mechanism.clone(),
                bonds: bonds.clone(),
                early_close: early_close.clone(),
            })
            .collect();
        let new_markets = sentinels
            .enumerate()
            .map(|(index, sentinel)| Market {
                market_id: index as u128,
                base_asset,
                creator: sentinel,
                creation: creation.clone(),
                creator_fee,
                oracle,
                metadata: metadata.clone(),
                market_type: market_type.clone(),
                period: period.clone(),
                deadlines,
                scoring_rule: new_scoring_rule,
                status,
                report: report.clone(),
                resolved_outcome: resolved_outcome.clone(),
                dispute_mechanism: dispute_mechanism.clone(),
                bonds: bonds.clone(),
                early_close: early_close.clone(),
            })
            .collect();
        (old_markets, new_markets)
    }

    #[allow(unused)]
    fn populate_test_data<H, K, V>(pallet: &[u8], prefix: &[u8], data: Vec<V>)
    where
        H: StorageHasher,
        K: TryFrom<usize> + Encode,
        V: Encode + Clone,
        <K as TryFrom<usize>>::Error: Debug,
    {
        for (key, value) in data.iter().enumerate() {
            let storage_hash = K::try_from(key).unwrap().using_encoded(H::hash).as_ref().to_vec();
            put_storage_value::<V>(pallet, prefix, &storage_hash, (*value).clone());
        }
    }
}
