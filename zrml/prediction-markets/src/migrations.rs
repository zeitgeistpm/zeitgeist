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

#[cfg(feature = "try-runtime")]
use crate::MarketIdOf;
use crate::{Config, MarketOf, MomentOf};
#[cfg(feature = "try-runtime")]
use alloc::format;
use alloc::{collections::BTreeMap, vec::Vec};
#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;
use frame_support::{
    dispatch::Weight,
    log,
    migration::{put_storage_value, storage_iter},
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    RuntimeDebug,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use zeitgeist_primitives::types::{
    Bond, Deadlines, Market, MarketBonds, MarketCreation, MarketDisputeMechanism, MarketPeriod,
    MarketStatus, MarketType, OutcomeReport, Report, ScoringRule,
};
use zrml_market_commons::{MarketCommonsPalletApi, Pallet as MarketCommonsPallet};

const MARKET_COMMONS: &[u8] = b"MarketCommons";
const MARKETS: &[u8] = b"Markets";
const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 4;
const MARKET_COMMONS_NEXT_STORAGE_VERSION: u16 = 5;

#[derive(Clone, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct OldMarket<AI, BN, M> {
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

type OldMarketOf<T> = OldMarket<
    <T as frame_system::Config>::AccountId,
    <T as frame_system::Config>::BlockNumber,
    MomentOf<T>,
>;

pub struct RecordBonds<T>(PhantomData<T>);

impl<T: Config + zrml_market_commons::Config> OnRuntimeUpgrade for RecordBonds<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let market_commons_version = StorageVersion::get::<MarketCommonsPallet<T>>();
        if market_commons_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "RecordBonds: market-commons version is {:?}, but {:?} is required",
                market_commons_version,
                MARKET_COMMONS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("RecordBonds: Starting...");

        let new_markets = storage_iter::<OldMarketOf<T>>(MARKET_COMMONS, MARKETS)
            .map(|(key, old_market)| {
                total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
                let creation = Some(match old_market.creation {
                    MarketCreation::Advised => Bond {
                        who: old_market.creator.clone(),
                        value: T::AdvisoryBond::get(),
                        is_settled: old_market.status != MarketStatus::Proposed,
                    },
                    MarketCreation::Permissionless => Bond {
                        who: old_market.creator.clone(),
                        value: T::ValidityBond::get(),
                        is_settled: matches!(
                            old_market.status,
                            MarketStatus::Resolved | MarketStatus::InsufficientSubsidy,
                        ),
                    },
                });
                let oracle = Some(Bond {
                    who: old_market.creator.clone(),
                    value: T::OracleBond::get(),
                    is_settled: matches!(
                        old_market.status,
                        MarketStatus::Resolved | MarketStatus::InsufficientSubsidy,
                    ),
                });
                let new_market = Market {
                    creator: old_market.creator,
                    creation: old_market.creation,
                    creator_fee: old_market.creator_fee,
                    oracle: old_market.oracle,
                    metadata: old_market.metadata,
                    market_type: old_market.market_type,
                    period: old_market.period,
                    scoring_rule: old_market.scoring_rule,
                    status: old_market.status,
                    report: old_market.report,
                    resolved_outcome: old_market.resolved_outcome,
                    dispute_mechanism: old_market.dispute_mechanism,
                    deadlines: old_market.deadlines,
                    bonds: MarketBonds { creation, oracle, dispute: None },
                };
                (key, new_market)
            })
            .collect::<Vec<_>>();

        for (key, new_market) in new_markets {
            put_storage_value::<MarketOf<T>>(MARKET_COMMONS, MARKETS, &key, new_market);
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        }

        StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION).put::<MarketCommonsPallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("RecordBonds: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        use frame_support::pallet_prelude::Blake2_128Concat;

        let old_markets = storage_key_iter::<MarketIdOf<T>, OldMarketOf<T>, Blake2_128Concat>(
            MARKET_COMMONS,
            MARKETS,
        )
        .collect::<BTreeMap<_, _>>();
        Self::set_temp_storage(old_markets, "old_markets");
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        let old_markets: BTreeMap<MarketIdOf<T>, OldMarketOf<T>> =
            Self::get_temp_storage("old_markets").unwrap();
        let new_market_count = <zrml_market_commons::Pallet<T>>::market_iter().count();
        assert_eq!(old_markets.len(), new_market_count);
        for (market_id, new_market) in <zrml_market_commons::Pallet<T>>::market_iter() {
            let old_market = old_markets
                .get(&market_id)
                .expect(&format!("Market {:?} not found", market_id)[..]);
            assert_eq!(new_market.creator, old_market.creator);
            assert_eq!(new_market.creation, old_market.creation);
            assert_eq!(new_market.creator_fee, old_market.creator_fee);
            assert_eq!(new_market.oracle, old_market.oracle);
            assert_eq!(new_market.metadata, old_market.metadata);
            assert_eq!(new_market.market_type, old_market.market_type);
            assert_eq!(new_market.period, old_market.period);
            assert_eq!(new_market.deadlines, old_market.deadlines);
            assert_eq!(new_market.scoring_rule, old_market.scoring_rule);
            assert_eq!(new_market.status, old_market.status);
            assert_eq!(new_market.report, old_market.report);
            assert_eq!(new_market.resolved_outcome, old_market.resolved_outcome);
            assert_eq!(new_market.dispute_mechanism, old_market.dispute_mechanism);
            assert_eq!(new_market.bonds.oracle.unwrap().value, T::OracleBond::get());
            let expected_creation_bond = match new_market.creation {
                MarketCreation::Advised => T::AdvisoryBond::get(),
                MarketCreation::Permissionless => T::ValidityBond::get(),
            };
            assert_eq!(new_market.bonds.creation.unwrap().value, expected_creation_bond);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock::{ExtBuilder, Runtime},
        MarketIdOf,
    };
    use frame_support::{dispatch::fmt::Debug, Blake2_128Concat, StorageHasher};
    use zrml_market_commons::MarketCommonsPalletApi;

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            RecordBonds::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<MarketCommonsPallet<Runtime>>(),
                MARKET_COMMONS_NEXT_STORAGE_VERSION
            );
        });
    }

    #[test]
    fn on_runtime_upgrade_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            // Don't set up chain to signal that storage is already up to date.
            let test_vector = construct_test_vector();
            let new_markets =
                test_vector.into_iter().map(|(_, new_market)| new_market).collect::<Vec<_>>();
            populate_test_data::<Blake2_128Concat, MarketIdOf<Runtime>, MarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                new_markets.clone(),
            );
            RecordBonds::<Runtime>::on_runtime_upgrade();
            for (market_id, expected) in new_markets.iter().enumerate() {
                let actual =
                    <zrml_market_commons::Pallet<Runtime>>::market(&(market_id as u128)).unwrap();
                assert_eq!(actual, *expected);
            }
        });
    }

    #[test]
    fn on_runtime_upgrade_correctly_updates_markets() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            let test_vector = construct_test_vector();
            let (old_markets, new_markets): (_, Vec<MarketOf<Runtime>>) =
                test_vector.into_iter().unzip();
            populate_test_data::<Blake2_128Concat, MarketIdOf<Runtime>, OldMarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                old_markets,
            );
            RecordBonds::<Runtime>::on_runtime_upgrade();
            for (market_id, expected) in new_markets.iter().enumerate() {
                let actual =
                    <zrml_market_commons::Pallet<Runtime>>::market(&(market_id as u128)).unwrap();
                assert_eq!(actual, *expected);
            }
        });
    }

    fn set_up_version() {
        StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION)
            .put::<MarketCommonsPallet<Runtime>>();
    }

    fn construct_test_vector() -> Vec<(OldMarketOf<Runtime>, MarketOf<Runtime>)> {
        let creator = 999;
        let construct_markets = |creation: MarketCreation, status, bonds| {
            let creator_fee = 1;
            let oracle = 2;
            let metadata = vec![3, 4, 5];
            let market_type = MarketType::Categorical(6);
            let period = MarketPeriod::Block(7..8);
            let scoring_rule = ScoringRule::CPMM;
            let report = None;
            let resolved_outcome = None;
            let dispute_mechanism = MarketDisputeMechanism::Authorized;
            let deadlines = Deadlines::default();

            let old_market = OldMarket {
                creator,
                creation: creation.clone(),
                creator_fee,
                oracle,
                metadata: metadata.clone(),
                market_type: market_type.clone(),
                period: period.clone(),
                scoring_rule,
                status,
                report: report.clone(),
                resolved_outcome: resolved_outcome.clone(),
                dispute_mechanism: dispute_mechanism.clone(),
                deadlines,
            };
            let new_market = Market {
                creator,
                creation,
                creator_fee,
                oracle,
                metadata,
                market_type,
                period,
                scoring_rule,
                status,
                report,
                resolved_outcome,
                dispute_mechanism,
                deadlines,
                bonds,
            };
            (old_market, new_market)
        };
        vec![
            construct_markets(
                MarketCreation::Permissionless,
                MarketStatus::Disputed,
                MarketBonds {
                    creation: Some(Bond {
                        who: creator,
                        value: <Runtime as Config>::ValidityBond::get(),
                        is_settled: false,
                    }),
                    oracle: Some(Bond {
                        who: creator,
                        value: <Runtime as Config>::OracleBond::get(),
                        is_settled: false,
                    }),
                    dispute: None,
                },
            ),
            construct_markets(
                MarketCreation::Permissionless,
                MarketStatus::Resolved,
                MarketBonds {
                    creation: Some(Bond {
                        who: creator,
                        value: <Runtime as Config>::ValidityBond::get(),
                        is_settled: true,
                    }),
                    oracle: Some(Bond {
                        who: creator,
                        value: <Runtime as Config>::OracleBond::get(),
                        is_settled: true,
                    }),
                    dispute: None,
                },
            ),
            construct_markets(
                MarketCreation::Advised,
                MarketStatus::Proposed,
                MarketBonds {
                    creation: Some(Bond {
                        who: creator,
                        value: <Runtime as Config>::AdvisoryBond::get(),
                        is_settled: false,
                    }),
                    oracle: Some(Bond {
                        who: creator,
                        value: <Runtime as Config>::OracleBond::get(),
                        is_settled: false,
                    }),
                    dispute: None,
                },
            ),
            construct_markets(
                MarketCreation::Advised,
                MarketStatus::Active,
                MarketBonds {
                    creation: Some(Bond {
                        who: creator,
                        value: <Runtime as Config>::AdvisoryBond::get(),
                        is_settled: true,
                    }),
                    oracle: Some(Bond {
                        who: creator,
                        value: <Runtime as Config>::OracleBond::get(),
                        is_settled: false,
                    }),
                    dispute: None,
                },
            ),
            construct_markets(
                MarketCreation::Advised,
                MarketStatus::Resolved,
                MarketBonds {
                    creation: Some(Bond {
                        who: creator,
                        value: <Runtime as Config>::AdvisoryBond::get(),
                        is_settled: true,
                    }),
                    oracle: Some(Bond {
                        who: creator,
                        value: <Runtime as Config>::OracleBond::get(),
                        is_settled: true,
                    }),
                    dispute: None,
                },
            ),
            // Technically, the market below has the wrong scoring rule, but that's irrelevant to
            // the test.
            construct_markets(
                MarketCreation::Permissionless,
                MarketStatus::InsufficientSubsidy,
                MarketBonds {
                    creation: Some(Bond {
                        who: creator,
                        value: <Runtime as Config>::ValidityBond::get(),
                        is_settled: true,
                    }),
                    oracle: Some(Bond {
                        who: creator,
                        value: <Runtime as Config>::OracleBond::get(),
                        is_settled: true,
                    }),
                    dispute: None,
                },
            ),
        ]
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

#[cfg(feature = "try-runtime")]
use alloc::string::ToString;
use frame_support::{migration::storage_key_iter, Twox64Concat};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::traits::Saturating;
use zeitgeist_primitives::types::AuthorityReport;
use zrml_authorized::Pallet as AuthorizedPallet;

const AUTHORIZED: &[u8] = b"Authorized";
const AUTHORIZED_OUTCOME_REPORTS: &[u8] = b"AuthorizedOutcomeReports";

const AUTHORIZED_REQUIRED_STORAGE_VERSION: u16 = 2;
const AUTHORIZED_NEXT_STORAGE_VERSION: u16 = 3;

pub struct AddFieldToAuthorityReport<T>(PhantomData<T>);

// Add resolve_at block number value field to `AuthorizedOutcomeReports` map.
impl<T: Config + zrml_market_commons::Config + zrml_authorized::Config> OnRuntimeUpgrade
    for AddFieldToAuthorityReport<T>
{
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);
        let authorized_version = StorageVersion::get::<AuthorizedPallet<T>>();
        if authorized_version != AUTHORIZED_REQUIRED_STORAGE_VERSION {
            log::info!(
                "AddFieldToAuthorityReport: authorized version is {:?}, require {:?};",
                authorized_version,
                AUTHORIZED_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("AddFieldToAuthorityReport: Starting...");

        let mut authorized_resolutions =
            BTreeMap::<<T as zrml_market_commons::Config>::MarketId, BlockNumberFor<T>>::new();
        for (resolve_at, bounded_vec) in crate::MarketIdsPerDisputeBlock::<T>::iter() {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

            for id in bounded_vec.into_inner().iter() {
                if let Ok(market) = <zrml_market_commons::Pallet<T>>::market(id) {
                    if market.dispute_mechanism == MarketDisputeMechanism::Authorized {
                        authorized_resolutions.insert(*id, resolve_at);
                    }
                } else {
                    log::warn!("AddFieldToAuthorityReport: Could not find market with id {:?}", id);
                }
            }
        }

        let mut new_storage_map: Vec<(
            <T as zrml_market_commons::Config>::MarketId,
            AuthorityReport<BlockNumberFor<T>>,
        )> = Vec::new();

        let now = frame_system::Pallet::<T>::block_number();
        total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

        for (market_id, old_value) in storage_key_iter::<
            <T as zrml_market_commons::Config>::MarketId,
            OutcomeReport,
            Twox64Concat,
        >(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS)
        {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

            let resolve_at: Option<BlockNumberFor<T>> =
                authorized_resolutions.get(&market_id).cloned();

            match resolve_at {
                Some(block) if now <= block => {
                    new_storage_map.push((
                        market_id,
                        AuthorityReport { resolve_at: block, outcome: old_value },
                    ));
                }
                _ => {
                    log::warn!(
                        "AddFieldToAuthorityReport: Market was not found in \
                         MarketIdsPerDisputeBlock; market id: {:?}",
                        market_id
                    );
                    // example case market id 432
                    // https://github.com/zeitgeistpm/zeitgeist/pull/701 market id 432 is invalid, because of zero-division error in the past
                    // we have to handle manually here, because MarketIdsPerDisputeBlock does not contain 432
                    let mut resolve_at = now.saturating_add(T::CorrectionPeriod::get());
                    total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

                    let mut bounded_vec = <crate::MarketIdsPerDisputeBlock<T>>::get(resolve_at);
                    while bounded_vec.is_full() {
                        // roll the dice until we find a block that is not full
                        total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
                        resolve_at = resolve_at.saturating_add(1u32.into());
                        bounded_vec = <crate::MarketIdsPerDisputeBlock<T>>::get(resolve_at);
                    }
                    // is not full, so we can push
                    bounded_vec.force_push(market_id);
                    <crate::MarketIdsPerDisputeBlock<T>>::insert(resolve_at, bounded_vec);
                    total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

                    new_storage_map
                        .push((market_id, AuthorityReport { resolve_at, outcome: old_value }));
                }
            }
        }

        for (market_id, new_value) in new_storage_map {
            let hash = utility::key_to_hash::<
                Twox64Concat,
                <T as zrml_market_commons::Config>::MarketId,
            >(market_id);
            put_storage_value::<AuthorityReport<T::BlockNumber>>(
                AUTHORIZED,
                AUTHORIZED_OUTCOME_REPORTS,
                &hash,
                new_value,
            );
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        }

        StorageVersion::new(AUTHORIZED_NEXT_STORAGE_VERSION).put::<AuthorizedPallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("AddFieldToAuthorityReport: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        let mut counter = 0_u32;
        for (key, value) in storage_iter::<OutcomeReport>(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS) {
            Self::set_temp_storage(value, &format!("{:?}", key.as_slice()));

            counter = counter.saturating_add(1_u32);
        }
        let counter_key = "counter_key".to_string();
        Self::set_temp_storage(counter, &counter_key);
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        let mut markets_count = 0_u32;
        let old_counter_key = "counter_key".to_string();
        for (key, new_value) in
            storage_iter::<AuthorityReport<T::BlockNumber>>(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS)
        {
            let key_str = format!("{:?}", key.as_slice());

            let AuthorityReport { resolve_at: _, outcome } = new_value;
            let old_value: OutcomeReport = Self::get_temp_storage(&key_str)
                .unwrap_or_else(|| panic!("old value not found for market id {:?}", key_str));

            assert_eq!(old_value, outcome);

            markets_count += 1_u32;
        }
        let old_markets_count: u32 =
            Self::get_temp_storage(&old_counter_key).expect("old counter key storage not found");
        assert_eq!(markets_count, old_markets_count);
        Ok(())
    }
}

#[cfg(test)]
mod tests_authorized {
    use super::*;
    use crate::{
        mock::{ExtBuilder, MarketCommons, Runtime, ALICE, BOB},
        CacheSize, MarketIdOf,
    };
    use frame_support::{BoundedVec, Twox64Concat};
    use zeitgeist_primitives::types::{MarketId, OutcomeReport};
    use zrml_market_commons::MarketCommonsPalletApi;

    #[test]
    fn on_runtime_upgrade_increments_the_storage_versions() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();
            AddFieldToAuthorityReport::<Runtime>::on_runtime_upgrade();
            let authorized_version = StorageVersion::get::<AuthorizedPallet<Runtime>>();
            assert_eq!(authorized_version, AUTHORIZED_NEXT_STORAGE_VERSION);
        });
    }

    #[test]
    fn on_runtime_sets_new_struct_with_resolve_at() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            <frame_system::Pallet<Runtime>>::set_block_number(10_000);

            let hash = crate::migrations::utility::key_to_hash::<Twox64Concat, MarketId>(0);
            let outcome = OutcomeReport::Categorical(42u16);
            put_storage_value::<OutcomeReport>(
                AUTHORIZED,
                AUTHORIZED_OUTCOME_REPORTS,
                &hash,
                outcome.clone(),
            );

            let resolve_at = 42_000;

            let sample_market = get_sample_market();
            let market_id: MarketId = MarketCommons::push_market(sample_market).unwrap();
            let bounded_vec =
                BoundedVec::<MarketIdOf<Runtime>, CacheSize>::try_from(vec![market_id])
                    .expect("BoundedVec should be created");
            crate::MarketIdsPerDisputeBlock::<Runtime>::insert(resolve_at, bounded_vec);

            AddFieldToAuthorityReport::<Runtime>::on_runtime_upgrade();

            let expected = AuthorityReport { resolve_at, outcome };

            let actual = frame_support::migration::get_storage_value::<
                AuthorityReport<<Runtime as frame_system::Config>::BlockNumber>,
            >(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS, &hash)
            .unwrap();
            assert_eq!(expected, actual);
        });
    }

    #[test]
    fn on_runtime_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            // storage migration already executed (storage version is incremented already)
            StorageVersion::new(AUTHORIZED_NEXT_STORAGE_VERSION).put::<AuthorizedPallet<Runtime>>();

            let hash = crate::migrations::utility::key_to_hash::<Twox64Concat, MarketId>(0);
            let outcome = OutcomeReport::Categorical(42u16);

            let report = AuthorityReport { resolve_at: 42, outcome };
            put_storage_value::<AuthorityReport<<Runtime as frame_system::Config>::BlockNumber>>(
                AUTHORIZED,
                AUTHORIZED_OUTCOME_REPORTS,
                &hash,
                report.clone(),
            );

            AddFieldToAuthorityReport::<Runtime>::on_runtime_upgrade();

            let actual = frame_support::migration::get_storage_value::<
                AuthorityReport<<Runtime as frame_system::Config>::BlockNumber>,
            >(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS, &hash)
            .unwrap();
            assert_eq!(report, actual);
        });
    }

    fn set_up_chain() {
        StorageVersion::new(AUTHORIZED_REQUIRED_STORAGE_VERSION).put::<AuthorizedPallet<Runtime>>();
    }

    fn get_sample_market() -> zeitgeist_primitives::types::Market<u128, u128, u64, u64> {
        zeitgeist_primitives::types::Market {
            creation: zeitgeist_primitives::types::MarketCreation::Permissionless,
            creator_fee: 0,
            creator: ALICE,
            market_type: zeitgeist_primitives::types::MarketType::Scalar(0..=100),
            dispute_mechanism: zeitgeist_primitives::types::MarketDisputeMechanism::Authorized,
            metadata: Default::default(),
            oracle: BOB,
            period: zeitgeist_primitives::types::MarketPeriod::Block(Default::default()),
            deadlines: zeitgeist_primitives::types::Deadlines {
                grace_period: 1_u32.into(),
                oracle_duration: 1_u32.into(),
                dispute_duration: 1_u32.into(),
            },
            report: None,
            resolved_outcome: None,
            scoring_rule: zeitgeist_primitives::types::ScoringRule::CPMM,
            status: zeitgeist_primitives::types::MarketStatus::Disputed,
            bonds: Default::default(),
        }
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
