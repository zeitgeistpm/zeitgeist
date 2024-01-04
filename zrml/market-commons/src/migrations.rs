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

use crate::{
    AccountIdOf, AssetOf, BalanceOf, BlockNumberOf, Config, MarketIdOf, MomentOf,
    Pallet as MarketCommons,
};
use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::{
    log,
    pallet_prelude::{Blake2_128Concat, StorageVersion, Weight},
    traits::{Get, OnRuntimeUpgrade},
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{Perbill, RuntimeDebug, Saturating};
use zeitgeist_primitives::types::{
    Deadlines, EarlyClose, Market, MarketBonds, MarketCreation, MarketDisputeMechanism,
    MarketPeriod, MarketStatus, MarketType, OutcomeReport, Report, ScoringRule,
};

#[cfg(feature = "try-runtime")]
use {
    alloc::collections::BTreeMap, frame_support::migration::storage_key_iter,
    zeitgeist_primitives::types::MarketId,
};

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
    pub status: OldMarketStatus,
    pub report: Option<Report<AI, BN>>,
    pub resolved_outcome: Option<OutcomeReport>,
    pub dispute_mechanism: Option<MarketDisputeMechanism>,
    pub bonds: MarketBonds<AI, BA>,
    pub early_close: Option<EarlyClose<BN, M>>,
}

type OldMarketOf<T> =
    OldMarket<AccountIdOf<T>, BalanceOf<T>, BlockNumberOf<T>, MomentOf<T>, AssetOf<T>>;

#[derive(TypeInfo, Clone, Copy, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug)]
pub enum OldScoringRule {
    CPMM,
    RikiddoSigmoidFeeMarketEma,
    Lmsr,
    Orderbook,
    Parimutuel,
}

#[derive(Clone, Copy, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub enum OldMarketStatus {
    Proposed,
    Active,
    Suspended,
    Closed,
    CollectingSubsidy,
    InsufficientSubsidy,
    Reported,
    Disputed,
    Resolved,
}

const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 9;
const MARKET_COMMONS_NEXT_STORAGE_VERSION: u16 = 10;

#[frame_support::storage_alias]
pub(crate) type Markets<T: Config> =
    StorageMap<MarketCommons<T>, Blake2_128Concat, MarketIdOf<T>, OldMarketOf<T>>;

pub struct MigrateScoringRuleAndMarketStatus<T>(PhantomData<T>);

/// Deletes all Rikiddo markets from storage and migrates CPMM markets to LMSR.
impl<T> OnRuntimeUpgrade for MigrateScoringRuleAndMarketStatus<T>
where
    T: Config,
{
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let market_commons_version = StorageVersion::get::<MarketCommons<T>>();
        if market_commons_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "MigrateScoringRuleAndMarketStatus: market-commons version is {:?}, but {:?} is \
                 required",
                market_commons_version,
                MARKET_COMMONS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("MigrateScoringRuleAndMarketStatus: Starting...");

        let mut translated = 0u64;
        crate::Markets::<T>::translate::<OldMarketOf<T>, _>(|_, old_market| {
            // We proceed by deleting markets which use the Rikiddo scoring rule or have a status
            // that was removed.
            translated.saturating_inc();
            let scoring_rule = match old_market.scoring_rule {
                OldScoringRule::RikiddoSigmoidFeeMarketEma => return None,
                OldScoringRule::CPMM | OldScoringRule::Lmsr => ScoringRule::Lmsr,
                OldScoringRule::Orderbook => ScoringRule::Orderbook,
                OldScoringRule::Parimutuel => ScoringRule::Parimutuel,
            };
            let status = match old_market.status {
                OldMarketStatus::Proposed => MarketStatus::Proposed,
                OldMarketStatus::Active => MarketStatus::Active,
                OldMarketStatus::Suspended => return None,
                OldMarketStatus::Closed => MarketStatus::Closed,
                OldMarketStatus::CollectingSubsidy => return None,
                OldMarketStatus::InsufficientSubsidy => return None,
                OldMarketStatus::Reported => MarketStatus::Reported,
                OldMarketStatus::Disputed => MarketStatus::Disputed,
                OldMarketStatus::Resolved => MarketStatus::Resolved,
            };
            let new_market = Market {
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
                status,
                report: old_market.report,
                resolved_outcome: old_market.resolved_outcome,
                dispute_mechanism: old_market.dispute_mechanism,
                bonds: old_market.bonds,
                early_close: old_market.early_close,
            };
            Some(new_market)
        });
        log::info!("MigrateScoringRuleAndMarketStatus: Upgraded {} markets.", translated);
        total_weight =
            total_weight.saturating_add(T::DbWeight::get().reads_writes(translated, translated));

        StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION).put::<MarketCommons<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("MigrateScoringRuleAndMarketStatus: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
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
    fn post_upgrade(previous_state: Vec<u8>) -> Result<(), &'static str> {
        let old_markets: BTreeMap<MarketId, OldMarketOf<T>> =
            Decode::decode(&mut &previous_state[..]).unwrap();
        let old_market_count = old_markets.len();
        let new_market_count = Markets::<T>::iter().count();
        assert_eq!(old_market_count, new_market_count);
        log::info!(
            "MigrateScoringRuleAndMarketStatus: Market counter post-upgrade is {}!",
            new_market_count
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{ExtBuilder, Runtime};
    use alloc::fmt::Debug;
    use frame_support::{migration::put_storage_value, storage_root, StorageHasher};
    use sp_runtime::{Perbill, StateVersion};
    use test_case::test_case;
    use zeitgeist_primitives::types::{Asset, Bond, MarketId};

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            MigrateScoringRuleAndMarketStatus::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<MarketCommons<Runtime>>(),
                MARKET_COMMONS_NEXT_STORAGE_VERSION
            );
        });
    }

    #[test_case(
        (OldScoringRule::CPMM, OldMarketStatus::Proposed),
        Some((ScoringRule::Lmsr, MarketStatus::Proposed))
    )]
    #[test_case(
        (OldScoringRule::CPMM, OldMarketStatus::Active),
        Some((ScoringRule::Lmsr, MarketStatus::Active))
    )]
    #[test_case((OldScoringRule::CPMM, OldMarketStatus::Suspended), None)]
    #[test_case(
        (OldScoringRule::CPMM, OldMarketStatus::Closed),
        Some((ScoringRule::Lmsr, MarketStatus::Closed))
    )]
    #[test_case((OldScoringRule::CPMM, OldMarketStatus::CollectingSubsidy), None)]
    #[test_case((OldScoringRule::CPMM, OldMarketStatus::InsufficientSubsidy), None)]
    #[test_case(
        (OldScoringRule::CPMM, OldMarketStatus::Reported),
        Some((ScoringRule::Lmsr, MarketStatus::Reported))
    )]
    #[test_case(
        (OldScoringRule::CPMM, OldMarketStatus::Disputed),
        Some((ScoringRule::Lmsr, MarketStatus::Disputed))
    )]
    #[test_case(
        (OldScoringRule::CPMM, OldMarketStatus::Resolved),
        Some((ScoringRule::Lmsr, MarketStatus::Resolved))
    )]
    #[test_case((OldScoringRule::RikiddoSigmoidFeeMarketEma, OldMarketStatus::Proposed), None)]
    #[test_case((OldScoringRule::RikiddoSigmoidFeeMarketEma, OldMarketStatus::Active), None)]
    #[test_case((OldScoringRule::RikiddoSigmoidFeeMarketEma, OldMarketStatus::Suspended), None)]
    #[test_case((OldScoringRule::RikiddoSigmoidFeeMarketEma, OldMarketStatus::Closed), None)]
    #[test_case(
        (OldScoringRule::RikiddoSigmoidFeeMarketEma, OldMarketStatus::CollectingSubsidy),
        None
    )]
    #[test_case(
        (OldScoringRule::RikiddoSigmoidFeeMarketEma, OldMarketStatus::InsufficientSubsidy),
        None
    )]
    #[test_case((OldScoringRule::RikiddoSigmoidFeeMarketEma, OldMarketStatus::Reported), None)]
    #[test_case((OldScoringRule::RikiddoSigmoidFeeMarketEma, OldMarketStatus::Disputed), None)]
    #[test_case((OldScoringRule::RikiddoSigmoidFeeMarketEma, OldMarketStatus::Resolved), None)]
    #[test_case(
        (OldScoringRule::Lmsr, OldMarketStatus::Proposed),
        Some((ScoringRule::Lmsr, MarketStatus::Proposed))
    )]
    #[test_case(
        (OldScoringRule::Lmsr, OldMarketStatus::Active),
        Some((ScoringRule::Lmsr, MarketStatus::Active))
    )]
    #[test_case((OldScoringRule::Lmsr, OldMarketStatus::Suspended), None)]
    #[test_case(
        (OldScoringRule::Lmsr, OldMarketStatus::Closed),
        Some((ScoringRule::Lmsr, MarketStatus::Closed))
    )]
    #[test_case((OldScoringRule::Lmsr, OldMarketStatus::CollectingSubsidy), None)]
    #[test_case((OldScoringRule::Lmsr, OldMarketStatus::InsufficientSubsidy), None)]
    #[test_case(
        (OldScoringRule::Lmsr, OldMarketStatus::Reported),
        Some((ScoringRule::Lmsr, MarketStatus::Reported))
    )]
    #[test_case(
        (OldScoringRule::Lmsr, OldMarketStatus::Disputed),
        Some((ScoringRule::Lmsr, MarketStatus::Disputed))
    )]
    #[test_case(
        (OldScoringRule::Lmsr, OldMarketStatus::Resolved),
        Some((ScoringRule::Lmsr, MarketStatus::Resolved))
    )]
    #[test_case(
        (OldScoringRule::Orderbook, OldMarketStatus::Proposed),
        Some((ScoringRule::Orderbook, MarketStatus::Proposed))
    )]
    #[test_case(
        (OldScoringRule::Orderbook, OldMarketStatus::Active),
        Some((ScoringRule::Orderbook, MarketStatus::Active))
    )]
    #[test_case((OldScoringRule::Orderbook, OldMarketStatus::Suspended), None)]
    #[test_case(
        (OldScoringRule::Orderbook, OldMarketStatus::Closed),
        Some((ScoringRule::Orderbook, MarketStatus::Closed))
    )]
    #[test_case((OldScoringRule::Orderbook, OldMarketStatus::CollectingSubsidy), None)]
    #[test_case((OldScoringRule::Orderbook, OldMarketStatus::InsufficientSubsidy), None)]
    #[test_case(
        (OldScoringRule::Orderbook, OldMarketStatus::Reported),
        Some((ScoringRule::Orderbook, MarketStatus::Reported))
    )]
    #[test_case(
        (OldScoringRule::Orderbook, OldMarketStatus::Disputed),
        Some((ScoringRule::Orderbook, MarketStatus::Disputed))
    )]
    #[test_case(
        (OldScoringRule::Orderbook, OldMarketStatus::Resolved),
        Some((ScoringRule::Orderbook, MarketStatus::Resolved))
    )]
    #[test_case(
        (OldScoringRule::Parimutuel, OldMarketStatus::Proposed),
        Some((ScoringRule::Parimutuel, MarketStatus::Proposed))
    )]
    #[test_case(
        (OldScoringRule::Parimutuel, OldMarketStatus::Active),
        Some((ScoringRule::Parimutuel, MarketStatus::Active))
    )]
    #[test_case((OldScoringRule::Parimutuel, OldMarketStatus::Suspended), None)]
    #[test_case(
        (OldScoringRule::Parimutuel, OldMarketStatus::Closed),
        Some((ScoringRule::Parimutuel, MarketStatus::Closed))
    )]
    #[test_case((OldScoringRule::Parimutuel, OldMarketStatus::CollectingSubsidy), None)]
    #[test_case((OldScoringRule::Parimutuel, OldMarketStatus::InsufficientSubsidy), None)]
    #[test_case(
        (OldScoringRule::Parimutuel, OldMarketStatus::Reported),
        Some((ScoringRule::Parimutuel, MarketStatus::Reported))
    )]
    #[test_case(
        (OldScoringRule::Parimutuel, OldMarketStatus::Disputed),
        Some((ScoringRule::Parimutuel, MarketStatus::Disputed))
    )]
    #[test_case(
        (OldScoringRule::Parimutuel, OldMarketStatus::Resolved),
        Some((ScoringRule::Parimutuel, MarketStatus::Resolved))
    )]
    fn on_runtime_upgrade_works_as_expected(
        old_data: (OldScoringRule, OldMarketStatus),
        new_data: Option<(ScoringRule, MarketStatus)>,
    ) {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            let base_asset = Asset::Ztg;
            let creator = 0;
            let creation = MarketCreation::Permissionless;
            let creator_fee = Perbill::from_rational(1u32, 1_000u32);
            let oracle = 2;
            let metadata = vec![0x03; 50];
            let market_type = MarketType::Categorical(4);
            let period = MarketPeriod::Block(5..6);
            let deadlines = Deadlines { grace_period: 7, oracle_duration: 8, dispute_duration: 9 };
            let report = Some(Report { at: 13, by: 14, outcome: OutcomeReport::Categorical(10) });
            let resolved_outcome = None;
            let dispute_mechanism = Some(MarketDisputeMechanism::Court);
            let bonds = MarketBonds {
                creation: Some(Bond::new(11, 12)),
                oracle: None,
                outsider: None,
                dispute: None,
                close_dispute: None,
                close_request: None,
            };
            let early_close = None;
            let (old_scoring_rule, old_market_status) = old_data;
            let old_market = OldMarket {
                base_asset,
                creator,
                creation: creation.clone(),
                creator_fee,
                oracle,
                metadata: metadata.clone(),
                market_type: market_type.clone(),
                period: period.clone(),
                deadlines,
                scoring_rule: old_scoring_rule,
                status: old_market_status,
                report: report.clone(),
                resolved_outcome: resolved_outcome.clone(),
                dispute_mechanism: dispute_mechanism.clone(),
                bonds: bonds.clone(),
                early_close: early_close.clone(),
            };
            let opt_new_market = if let Some((new_scoring_rule, new_status)) = new_data {
                Some(Market {
                    base_asset,
                    creator,
                    creation,
                    creator_fee,
                    oracle,
                    metadata,
                    market_type,
                    period,
                    deadlines,
                    scoring_rule: new_scoring_rule,
                    status: new_status,
                    report,
                    resolved_outcome,
                    dispute_mechanism,
                    bonds,
                    early_close,
                })
            } else {
                None
            };
            // Don't set up chain to signal that storage is already up to date.
            populate_test_data::<Blake2_128Concat, MarketId, OldMarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                vec![old_market],
            );
            MigrateScoringRuleAndMarketStatus::<Runtime>::on_runtime_upgrade();

            let actual = crate::Markets::<Runtime>::get(0);
            assert_eq!(actual, opt_new_market);
        });
    }

    #[test]
    fn on_runtime_upgrade_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION)
                .put::<MarketCommons<Runtime>>();
            let market = Market {
                base_asset: Asset::<MarketIdOf<Runtime>>::ForeignAsset(0),
                creator: 1,
                creation: MarketCreation::Permissionless,
                creator_fee: Perbill::from_rational(2u32, 3u32),
                oracle: 4,
                metadata: vec![0x05; 50],
                market_type: MarketType::Categorical(999),
                period: MarketPeriod::<BlockNumberOf<Runtime>, MomentOf<Runtime>>::Block(6..7),
                deadlines: Deadlines { grace_period: 7, oracle_duration: 8, dispute_duration: 9 },
                scoring_rule: ScoringRule::Parimutuel,
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
            MigrateScoringRuleAndMarketStatus::<Runtime>::on_runtime_upgrade();
            assert_eq!(tmp, storage_root(StateVersion::V1));
        });
    }

    fn set_up_version() {
        StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION)
            .put::<MarketCommons<Runtime>>();
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
