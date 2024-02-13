// Copyright 2023-2024 Forecasting Technologies LTD.
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

use crate::{AccountIdOf, AssetOf, BalanceOf, BlockNumberOf, Config, Markets, MomentOf, Pallet};
use core::marker::PhantomData;
use frame_support::{
    dispatch::Weight,
    log,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    RuntimeDebug,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::Perbill;
use zeitgeist_primitives::types::{
    Deadlines, EarlyClose, Market, MarketBonds, MarketCreation, MarketDisputeMechanism,
    MarketPeriod, MarketStatus, MarketType, OutcomeReport, Report, ScoringRule,
};

#[cfg(feature = "try-runtime")]
use crate::MarketIdOf;
#[cfg(feature = "try-runtime")]
use alloc::collections::BTreeMap;
#[cfg(feature = "try-runtime")]
use alloc::{format, vec::Vec};
#[cfg(feature = "try-runtime")]
use frame_support::migration::storage_key_iter;
#[cfg(any(feature = "try-runtime", feature = "test"))]
use frame_support::Blake2_128Concat;

#[cfg(any(feature = "try-runtime", test))]
const MARKET_COMMONS: &[u8] = b"MarketCommons";
#[cfg(any(feature = "try-runtime", test))]
const MARKETS: &[u8] = b"Markets";

const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 10;
const MARKET_COMMONS_NEXT_STORAGE_VERSION: u16 = MARKET_COMMONS_REQUIRED_STORAGE_VERSION + 1;

#[derive(Clone, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct OldMarket<AI, BA, BN, M, A> {
    base_asset: A,
    creator: AI,
    creation: MarketCreation,
    creator_fee: Perbill,
    oracle: AI,
    metadata: Vec<u8>,
    market_type: MarketType,
    period: MarketPeriod<BN, M>,
    deadlines: Deadlines<BN>,
    scoring_rule: ScoringRule,
    status: MarketStatus,
    report: Option<Report<AI, BN>>,
    resolved_outcome: Option<OutcomeReport>,
    dispute_mechanism: Option<MarketDisputeMechanism>,
    bonds: MarketBonds<AI, BA>,
    early_close: Option<EarlyClose<BN, M>>,
}

type OldMarketOf<T> =
    OldMarket<AccountIdOf<T>, BalanceOf<T>, BlockNumberOf<T>, MomentOf<T>, AssetOf<T>>;

pub struct AddIdToMarket<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for AddIdToMarket<T>
where
    T: Config,
{
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let market_commons_version = StorageVersion::get::<Pallet<T>>();
        if market_commons_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "AddIdToMarket: neo-swaps version is {:?}, but {:?} is required",
                market_commons_version,
                MARKET_COMMONS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("AddIdToMarket: Starting...");
        let mut translated = 0u64;
        Markets::<T>::translate::<OldMarketOf<T>, _>(|market_id, market| {
            translated = translated.saturating_add(1);
            Some(Market {
                market_id,
                base_asset: market.base_asset,
                creator: market.creator,
                creation: market.creation,
                creator_fee: market.creator_fee,
                oracle: market.oracle,
                metadata: market.metadata,
                market_type: market.market_type,
                period: market.period,
                deadlines: market.deadlines,
                scoring_rule: market.scoring_rule,
                status: market.status,
                report: market.report,
                resolved_outcome: market.resolved_outcome,
                dispute_mechanism: market.dispute_mechanism,
                bonds: market.bonds,
                early_close: market.early_close,
            })
        });
        log::info!("AddIdToMarket: Upgraded {} markets.", translated);
        total_weight =
            total_weight.saturating_add(T::DbWeight::get().reads_writes(translated, translated));
        StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("AddIdToMarket: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        let old_markets = storage_key_iter::<MarketIdOf<T>, OldMarketOf<T>, Blake2_128Concat>(
            MARKET_COMMONS,
            MARKETS,
        )
        .collect::<BTreeMap<_, _>>();
        Ok(old_markets.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(previous_state: Vec<u8>) -> Result<(), &'static str> {
        let old_markets: BTreeMap<MarketIdOf<T>, OldMarketOf<T>> =
            Decode::decode(&mut &previous_state[..])
                .map_err(|_| "Failed to decode state: Invalid state")?;
        let new_market_count = Markets::<T>::iter().count();
        assert_eq!(old_markets.len(), new_market_count);
        for (market_id, new_market) in Markets::<T>::iter() {
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
            assert_eq!(old_market.scoring_rule, new_market.scoring_rule);
            assert_eq!(old_market.status, new_market.status);
            assert_eq!(old_market.report, new_market.report);
            assert_eq!(old_market.resolved_outcome, new_market.resolved_outcome);
            assert_eq!(old_market.dispute_mechanism, new_market.dispute_mechanism);
            assert_eq!(old_market.bonds, new_market.bonds);
            assert_eq!(old_market.early_close, new_market.early_close);
        }
        log::info!("AddIdToMarket: Post-upgrade market count is {}!", new_market_count);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock::{ExtBuilder, Runtime},
        MarketIdOf, MarketOf, Markets,
    };
    use frame_support::{
        dispatch::fmt::Debug, migration::put_storage_value, storage_root, Blake2_128Concat,
        StateVersion, StorageHasher,
    };
    use parity_scale_codec::Encode;
    use zeitgeist_primitives::types::{Asset, Bond, EarlyCloseState};

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            AddIdToMarket::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<Pallet<Runtime>>(),
                MARKET_COMMONS_NEXT_STORAGE_VERSION
            );
        });
    }

    #[test]
    fn on_runtime_upgrade_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION).put::<Pallet<Runtime>>();
            let (_, new_markets) = construct_old_new_tuple();
            populate_test_data::<Blake2_128Concat, MarketIdOf<Runtime>, MarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                new_markets,
            );
            let tmp = storage_root(StateVersion::V1);
            AddIdToMarket::<Runtime>::on_runtime_upgrade();
            assert_eq!(tmp, storage_root(StateVersion::V1));
        });
    }

    #[test]
    fn on_runtime_upgrade_correctly_updates_markets() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            let (old_markets, new_markets) = construct_old_new_tuple();
            populate_test_data::<Blake2_128Concat, MarketIdOf<Runtime>, OldMarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                old_markets,
            );
            AddIdToMarket::<Runtime>::on_runtime_upgrade();
            let actual = Markets::<Runtime>::get(0u128).unwrap();
            assert_eq!(actual, new_markets[0]);
        });
    }

    fn set_up_version() {
        StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
    }

    fn construct_old_new_tuple() -> (Vec<OldMarketOf<Runtime>>, Vec<MarketOf<Runtime>>) {
        let base_asset = Asset::Ztg;
        let creation = MarketCreation::Advised;
        let creator_fee = Perbill::from_rational(2u32, 3u32);
        let oracle = 4;
        let metadata = vec![5; 50];
        let market_type = MarketType::Scalar(6..=7);
        let period = MarketPeriod::Block(8..9);
        let deadlines = Deadlines { grace_period: 10, oracle_duration: 11, dispute_duration: 12 };
        let scoring_rule = ScoringRule::Lmsr;
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
                scoring_rule,
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
                scoring_rule,
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
            let storage_hash = utility::key_to_hash::<H, K>(K::try_from(key).unwrap());
            put_storage_value::<V>(pallet, prefix, &storage_hash, (*value).clone());
        }
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
