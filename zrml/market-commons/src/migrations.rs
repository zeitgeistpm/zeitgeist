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

use crate::{AccountIdOf, BalanceOf, Config, MarketIdOf, MomentOf, Pallet as MarketCommons};
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use frame_support::{
    pallet_prelude::Weight,
    storage::migration::get_storage_value,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    Blake2_128Concat, StorageHasher,
};
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{Perbill, RuntimeDebug, SaturatedConversion, Saturating};
use zeitgeist_primitives::types::{
    Asset, Deadlines, EarlyClose, Market, MarketBonds, MarketCreation, MarketDisputeMechanism,
    MarketPeriod, MarketStatus, MarketType, OutcomeReport, Report, ScoringRule,
};

#[cfg(feature = "try-runtime")]
use {
    alloc::{collections::BTreeMap, format},
    frame_support::migration::storage_key_iter,
    sp_runtime::DispatchError,
};

const MARKET_COMMONS: &[u8] = b"MarketCommons";
const MARKETS: &[u8] = b"Markets";

const MARKET_COMMONS_REQUIRED_STORAGE_VERSION_0: u16 = 11;
const MARKET_COMMONS_NEXT_STORAGE_VERSION_0: u16 = 12;

#[cfg(feature = "try-runtime")]
#[frame_support::storage_alias]
pub(crate) type Markets<T: Config> =
    StorageMap<MarketCommons<T>, Blake2_128Concat, MarketIdOf<T>, OldMarketOf<T>>;

// ID type of the campaign asset class.
pub type CampaignAssetId = u128;

#[derive(Clone, Copy, Debug, Decode, Default, Eq, Encode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum BaseAssetClass {
    #[codec(index = 4)]
    #[default]
    Ztg,

    #[codec(index = 5)]
    ForeignAsset(u32),

    #[codec(index = 7)]
    CampaignAsset(#[codec(compact)] CampaignAssetId),
}

#[derive(Clone, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct OldMarketWithCampaignBaseAsset<
    AccountId,
    Balance,
    BlockNumber,
    Moment,
    BaseAssetClass,
    MarketId,
> {
    pub market_id: MarketId,
    /// Base asset of the market.
    pub base_asset: BaseAssetClass,
    /// Creator of this market.
    pub creator: AccountId,
    /// Creation type.
    pub creation: MarketCreation,
    /// A fee that is charged each trade and given to the market creator.
    pub creator_fee: Perbill,
    /// Oracle that reports the outcome of this market.
    pub oracle: AccountId,
    /// Metadata for the market, usually a content address of IPFS
    /// hosted JSON. Currently limited to 66 bytes (see `MaxEncodedLen` implementation)
    pub metadata: Vec<u8>,
    /// The type of the market.
    pub market_type: MarketType,
    /// Market start and end
    pub period: MarketPeriod<BlockNumber, Moment>,
    /// Market deadlines.
    pub deadlines: Deadlines<BlockNumber>,
    /// The scoring rule used for the market.
    pub scoring_rule: ScoringRule,
    /// The current status of the market.
    pub status: MarketStatus,
    /// The report of the market. Only `Some` if it has been reported.
    pub report: Option<Report<AccountId, BlockNumber>>,
    /// The resolved outcome.
    pub resolved_outcome: Option<OutcomeReport>,
    /// See [`MarketDisputeMechanism`].
    pub dispute_mechanism: Option<OldMarketDisputeMechanism>,
    /// The bonds reserved for this market.
    pub bonds: MarketBonds<AccountId, Balance>,
    /// The time at which the market was closed early.
    pub early_close: Option<EarlyClose<BlockNumber, Moment>>,
}

type CorruptedMarketOf<T> = OldMarketWithCampaignBaseAsset<
    AccountIdOf<T>,
    BalanceOf<T>,
    BlockNumberFor<T>,
    MomentOf<T>,
    BaseAssetClass,
    MarketIdOf<T>,
>;

pub struct RemoveMarkets<T, MarketIds>(PhantomData<T>, MarketIds);

impl<T, MarketIds> OnRuntimeUpgrade for RemoveMarkets<T, MarketIds>
where
    T: Config,
    MarketIds: Get<Vec<u32>>,
{
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let market_commons_version = StorageVersion::get::<MarketCommons<T>>();
        if market_commons_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION_0 {
            log::info!(
                "RemoveMarkets: market-commons version is {:?}, but {:?} is required",
                market_commons_version,
                MARKET_COMMONS_REQUIRED_STORAGE_VERSION_0,
            );
            return total_weight;
        }
        log::info!("RemoveMarkets: Starting...");

        let mut corrupted_markets = vec![];

        for &market_id in MarketIds::get().iter() {
            let market_id = market_id.saturated_into::<MarketIdOf<T>>();
            let is_corrupted = || {
                if let Some(market) = get_storage_value::<CorruptedMarketOf<T>>(
                    MARKET_COMMONS,
                    MARKETS,
                    &MarketIdOf::<T>::from(market_id).using_encoded(Blake2_128Concat::hash),
                ) {
                    matches!(market.base_asset, BaseAssetClass::CampaignAsset(_))
                } else {
                    false
                }
            };
            if crate::Markets::<T>::contains_key(market_id) && is_corrupted() {
                crate::Markets::<T>::remove(market_id);
                corrupted_markets.push(market_id);
            } else {
                log::warn!(
                    "RemoveMarkets: Market {:?} was expected to be corrupted, but isn't.",
                    market_id
                );
            }
        }

        log::info!("RemoveMarkets: Removed markets {:?}.", corrupted_markets);
        let count = MarketIds::get().len() as u64;
        total_weight = total_weight
            .saturating_add(T::DbWeight::get().reads_writes(count.saturating_mul(2u64), count));

        StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION_0).put::<MarketCommons<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("RemoveMarkets: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, DispatchError> {
        Ok(vec![])
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_previous_state: Vec<u8>) -> Result<(), DispatchError> {
        for &market_id in MarketIds::get().iter() {
            let market_id = market_id.saturated_into::<MarketIdOf<T>>();
            assert!(!crate::Markets::<T>::contains_key(market_id));
            assert!(crate::Markets::<T>::try_get(market_id).is_err());
        }

        log::info!("RemoveMarkets: Post-upgrade done!");
        Ok(())
    }
}

const MARKET_COMMONS_REQUIRED_STORAGE_VERSION_1: u16 = 12;
const MARKET_COMMONS_NEXT_STORAGE_VERSION_1: u16 = 13;

#[derive(Clone, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct OldMarket<AccountId, Balance, BlockNumber, Moment, MarketId> {
    pub market_id: MarketId,
    /// Base asset of the market.
    pub base_asset: Asset<MarketId>,
    /// Creator of this market.
    pub creator: AccountId,
    /// Creation type.
    pub creation: MarketCreation,
    /// A fee that is charged each trade and given to the market creator.
    pub creator_fee: Perbill,
    /// Oracle that reports the outcome of this market.
    pub oracle: AccountId,
    /// Metadata for the market, usually a content address of IPFS
    /// hosted JSON. Currently limited to 66 bytes (see `MaxEncodedLen` implementation)
    pub metadata: Vec<u8>,
    /// The type of the market.
    pub market_type: MarketType,
    /// Market start and end
    pub period: MarketPeriod<BlockNumber, Moment>,
    /// Market deadlines.
    pub deadlines: Deadlines<BlockNumber>,
    /// The scoring rule used for the market.
    pub scoring_rule: ScoringRule,
    /// The current status of the market.
    pub status: MarketStatus,
    /// The report of the market. Only `Some` if it has been reported.
    pub report: Option<Report<AccountId, BlockNumber>>,
    /// The resolved outcome.
    pub resolved_outcome: Option<OutcomeReport>,
    /// See [`MarketDisputeMechanism`].
    pub dispute_mechanism: Option<OldMarketDisputeMechanism>,
    /// The bonds reserved for this market.
    pub bonds: MarketBonds<AccountId, Balance>,
    /// The time at which the market was closed early.
    pub early_close: Option<EarlyClose<BlockNumber, Moment>>,
}

type OldMarketOf<T> =
    OldMarket<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, MomentOf<T>, MarketIdOf<T>>;

#[derive(TypeInfo, Clone, Copy, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug)]
pub enum OldMarketDisputeMechanism {
    Authorized,
    Court,
    SimpleDisputes,
}

pub struct MigrateDisputeMechanism<T>(PhantomData<T>);

/// Removes the `SimpleDisputes` MDM by switching markets that use `SimpleDisputes` to `Authorized`.
/// Note that this migration does not unreserve any funds bonded with zrml-simple-dispute's reserve
/// ID.
impl<T> OnRuntimeUpgrade for MigrateDisputeMechanism<T>
where
    T: Config,
{
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let market_commons_version = StorageVersion::get::<MarketCommons<T>>();
        if market_commons_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION_1 {
            log::info!(
                "MigrateDisputeMechanism: market-commons version is {:?}, but {:?} is required",
                market_commons_version,
                MARKET_COMMONS_REQUIRED_STORAGE_VERSION_1,
            );
            return total_weight;
        }
        log::info!("MigrateDisputeMechanism: Starting...");

        let mut translated = 0u64;
        crate::Markets::<T>::translate::<OldMarketOf<T>, _>(|_, old_market| {
            translated.saturating_inc();
            let dispute_mechanism = match old_market.dispute_mechanism {
                Some(OldMarketDisputeMechanism::Court) => Some(MarketDisputeMechanism::Court),
                Some(_) => Some(MarketDisputeMechanism::Authorized),
                None => None,
            };
            let new_market = Market {
                market_id: old_market.market_id,
                base_asset: old_market.base_asset,
                creator: old_market.creator,
                creation: old_market.creation,
                creator_fee: old_market.creator_fee,
                oracle: old_market.oracle,
                metadata: old_market.metadata,
                market_type: old_market.market_type,
                period: old_market.period,
                deadlines: old_market.deadlines,
                scoring_rule: old_market.scoring_rule,
                status: old_market.status,
                report: old_market.report,
                resolved_outcome: old_market.resolved_outcome,
                dispute_mechanism,
                bonds: old_market.bonds,
                early_close: old_market.early_close,
            };
            Some(new_market)
        });
        log::info!("MigrateDisputeMechanism: Upgraded {} markets.", translated);
        total_weight =
            total_weight.saturating_add(T::DbWeight::get().reads_writes(translated, translated));

        StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION_1).put::<MarketCommons<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("MigrateDisputeMechanism: Done!");
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
            assert_eq!(old_market.market_id, new_market.market_id);
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
            assert_eq!(old_market.bonds, new_market.bonds);
            assert_eq!(old_market.early_close, new_market.early_close);
        }

        log::info!("MigrateDisputeMechanism: Post-upgrade market count is {}!", new_market_count);
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
    use frame_support::{
        migration::put_storage_value, parameter_types, Blake2_128Concat, StorageHasher,
    };
    use parity_scale_codec::Encode;
    use sp_io::storage::root as storage_root;
    use sp_runtime::{Perbill, StateVersion};
    use test_case::test_case;
    use zeitgeist_primitives::types::{Bond, EarlyCloseState, MarketId};

    parameter_types! {
        pub RemovableMarketIds: Vec<u32> = vec![879u32, 877u32, 878u32, 880u32, 882u32];
        pub NoRemovableMarketIds: Vec<u32> = vec![];
    }

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version_remove_markets();
            RemoveMarkets::<Runtime, NoRemovableMarketIds>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<MarketCommons<Runtime>>(),
                MARKET_COMMONS_NEXT_STORAGE_VERSION_0
            );
            MigrateDisputeMechanism::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<MarketCommons<Runtime>>(),
                MARKET_COMMONS_NEXT_STORAGE_VERSION_1
            );
        });
    }

    #[test]
    fn on_runtime_upgrade_remove_corrupted_markets_works_as_expected() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version_remove_markets();
            let pallet = MARKET_COMMONS;
            let prefix = MARKETS;
            let corrupted_market = construct_corrupt_market();
            for market_id in RemovableMarketIds::get().iter() {
                let storage_hash = MarketId::from(*market_id).using_encoded(Blake2_128Concat::hash);
                put_storage_value::<CorruptedMarketOf<Runtime>>(
                    pallet,
                    prefix,
                    &storage_hash,
                    corrupted_market.clone(),
                );
            }

            for market_id in RemovableMarketIds::get().iter() {
                let market_id = MarketId::from(*market_id);
                assert!(crate::Markets::<Runtime>::contains_key(market_id));
            }

            RemoveMarkets::<Runtime, RemovableMarketIds>::on_runtime_upgrade();

            for market_id in RemovableMarketIds::get().iter() {
                let market_id = MarketId::from(*market_id);
                assert!(!crate::Markets::<Runtime>::contains_key(market_id));
            }
        });
    }

    #[test]
    fn on_runtime_upgrade_does_not_remove_valid_markets() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version_remove_markets();
            let pallet = MARKET_COMMONS;
            let prefix = MARKETS;
            let (old_market, _) = construct_old_new_tuple(
                Some(OldMarketDisputeMechanism::SimpleDisputes),
                Some(MarketDisputeMechanism::Authorized),
            );
            for market_id in RemovableMarketIds::get().iter() {
                let storage_hash = MarketId::from(*market_id).using_encoded(Blake2_128Concat::hash);
                // base asset for the market is not a campaign asset, so not corrupted
                put_storage_value::<OldMarketOf<Runtime>>(
                    pallet,
                    prefix,
                    &storage_hash,
                    old_market.clone(),
                );
            }

            for market_id in RemovableMarketIds::get().iter() {
                let market_id = MarketId::from(*market_id);
                assert!(crate::Markets::<Runtime>::contains_key(market_id));
            }

            RemoveMarkets::<Runtime, RemovableMarketIds>::on_runtime_upgrade();

            for market_id in RemovableMarketIds::get().iter() {
                let market_id = MarketId::from(*market_id);
                // all markets still present, because no market was in a corrupted storage layout
                assert!(crate::Markets::<Runtime>::contains_key(market_id));
            }
        });
    }

    #[test_case(None, None; "none")]
    #[test_case(
        Some(OldMarketDisputeMechanism::Authorized),
        Some(MarketDisputeMechanism::Authorized)
    )]
    #[test_case(Some(OldMarketDisputeMechanism::Court), Some(MarketDisputeMechanism::Court))]
    #[test_case(
        Some(OldMarketDisputeMechanism::SimpleDisputes),
        Some(MarketDisputeMechanism::Authorized)
    )]
    fn on_runtime_upgrade_mdm_works_as_expected(
        old_scoring_rule: Option<OldMarketDisputeMechanism>,
        new_scoring_rule: Option<MarketDisputeMechanism>,
    ) {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version_mdm();
            let (old_market, new_market) =
                construct_old_new_tuple(old_scoring_rule, new_scoring_rule);
            populate_test_data::<Blake2_128Concat, MarketId, OldMarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                vec![old_market],
            );
            MigrateDisputeMechanism::<Runtime>::on_runtime_upgrade();
            assert_eq!(crate::Markets::<Runtime>::get(0).unwrap(), new_market);
        });
    }

    #[test]
    fn on_runtime_upgrade_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION_1)
                .put::<MarketCommons<Runtime>>();
            let market = Market {
                market_id: 7,
                base_asset: Asset::Ztg,
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
            MigrateDisputeMechanism::<Runtime>::on_runtime_upgrade();
            assert_eq!(tmp, storage_root(StateVersion::V1));
        });
    }

    fn set_up_version_remove_markets() {
        StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION_0)
            .put::<MarketCommons<Runtime>>();
    }

    fn set_up_version_mdm() {
        StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION_1)
            .put::<MarketCommons<Runtime>>();
    }

    fn construct_old_new_tuple(
        old_dispute_mechanism: Option<OldMarketDisputeMechanism>,
        new_dispute_mechanism: Option<MarketDisputeMechanism>,
    ) -> (OldMarketOf<Runtime>, MarketOf<Runtime>) {
        let market_id = 0;
        let base_asset = Asset::Ztg;
        let creator = 1;
        let creation = MarketCreation::Advised;
        let creator_fee = Perbill::from_rational(2u32, 3u32);
        let oracle = 4;
        let metadata = vec![5; 50];
        let market_type = MarketType::Scalar(6..=7);
        let period = MarketPeriod::Block(8..9);
        let deadlines = Deadlines { grace_period: 10, oracle_duration: 11, dispute_duration: 12 };
        let scoring_rule = ScoringRule::AmmCdaHybrid;
        let status = MarketStatus::Resolved;
        let report = Some(Report { at: 13, by: 14, outcome: OutcomeReport::Categorical(15) });
        let resolved_outcome = Some(OutcomeReport::Categorical(16));
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
        let old_markets = OldMarket {
            market_id,
            base_asset,
            creator,
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
            dispute_mechanism: old_dispute_mechanism,
            bonds: bonds.clone(),
            early_close: early_close.clone(),
        };
        let new_market = Market {
            market_id,
            base_asset,
            creator,
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
            dispute_mechanism: new_dispute_mechanism,
            bonds: bonds.clone(),
            early_close: early_close.clone(),
        };
        (old_markets, new_market)
    }

    fn construct_corrupt_market() -> CorruptedMarketOf<Runtime> {
        let market_id = 0;
        let base_asset = BaseAssetClass::CampaignAsset(5u128);
        let creator = 1;
        let creation = MarketCreation::Advised;
        let creator_fee = Perbill::from_rational(2u32, 3u32);
        let oracle = 4;
        let metadata = vec![5; 50];
        let market_type = MarketType::Scalar(6..=7);
        let period = MarketPeriod::Block(8..9);
        let deadlines = Deadlines { grace_period: 10, oracle_duration: 11, dispute_duration: 12 };
        let scoring_rule = ScoringRule::AmmCdaHybrid;
        let status = MarketStatus::Resolved;
        let report = Some(Report { at: 13, by: 14, outcome: OutcomeReport::Categorical(15) });
        let resolved_outcome = Some(OutcomeReport::Categorical(16));
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
        let dispute_mechanism: Option<OldMarketDisputeMechanism> =
            Some(OldMarketDisputeMechanism::Court);

        OldMarketWithCampaignBaseAsset {
            market_id,
            creator,
            base_asset,
            creation,
            creator_fee,
            oracle,
            metadata,
            market_type,
            period,
            deadlines,
            scoring_rule,
            status,
            report,
            resolved_outcome,
            dispute_mechanism,
            bonds,
            early_close,
        }
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
