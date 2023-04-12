// Copyright 2022-2023 Forecasting Technologies LTD.
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
use crate::{BalanceOf, Config, MomentOf};
#[cfg(feature = "try-runtime")]
use alloc::collections::BTreeMap;
#[cfg(feature = "try-runtime")]
use alloc::format;
use alloc::vec::Vec;
#[cfg(feature = "try-runtime")]
use frame_support::migration::storage_key_iter;
#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;
use frame_support::{
    dispatch::Weight,
    log,
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
    RuntimeDebug,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::traits::Saturating;
use zeitgeist_primitives::types::{
    Asset, Bond, Deadlines, Market, MarketBonds, MarketCreation, MarketDisputeMechanism,
    MarketPeriod, MarketStatus, MarketType, OutcomeReport, Report, ScoringRule,
};
use zrml_market_commons::{MarketCommonsPalletApi, Pallet as MarketCommonsPallet};

#[cfg(any(feature = "try-runtime", test))]
const MARKET_COMMONS: &[u8] = b"MarketCommons";
#[cfg(any(feature = "try-runtime", test))]
const MARKETS: &[u8] = b"Markets";

const MARKET_COMMONS_REQUIRED_STORAGE_VERSION: u16 = 5;
const MARKET_COMMONS_NEXT_STORAGE_VERSION: u16 = 6;

#[derive(Clone, Decode, Encode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct OldMarketBonds<AI, BA> {
    pub creation: Option<Bond<AI, BA>>,
    pub oracle: Option<Bond<AI, BA>>,
}

#[derive(Clone, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct OldMarket<AI, BA, BN, M, A> {
    pub base_asset: A,
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
    pub bonds: OldMarketBonds<AI, BA>,
}

type OldMarketOf<T> = OldMarket<
    <T as frame_system::Config>::AccountId,
    BalanceOf<T>,
    <T as frame_system::Config>::BlockNumber,
    MomentOf<T>,
    Asset<<T as zrml_market_commons::Config>::MarketId>,
>;

#[frame_support::storage_alias]
pub(crate) type Markets<T: Config> = StorageMap<
    MarketCommonsPallet<T>,
    frame_support::Blake2_128Concat,
    <T as zrml_market_commons::Config>::MarketId,
    OldMarketOf<T>,
>;

pub struct AddOutsiderAndDisputeBond<T>(PhantomData<T>);

impl<T: Config + zrml_market_commons::Config> OnRuntimeUpgrade for AddOutsiderAndDisputeBond<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let market_commons_version = StorageVersion::get::<MarketCommonsPallet<T>>();
        if market_commons_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "AddOutsiderAndDisputeBond: market-commons version is {:?}, but {:?} is required",
                market_commons_version,
                MARKET_COMMONS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("AddOutsiderAndDisputeBond: Starting...");

        let mut translated = 0u64;
        zrml_market_commons::Markets::<T>::translate::<OldMarketOf<T>, _>(
            |market_id, old_market| {
                translated.saturating_inc();

                let mut dispute_bond = None;
                // SimpleDisputes is regarded in the following migration `MoveDataToSimpleDisputes`
                if let MarketDisputeMechanism::Authorized = old_market.dispute_mechanism {
                    total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
                    let old_disputes = crate::Disputes::<T>::get(market_id);
                    if let Some(first_dispute) = old_disputes.first() {
                        let OldMarketDispute { at: _, by, outcome: _ } = first_dispute;
                        dispute_bond = Some(Bond::new(by.clone(), T::DisputeBond::get()));
                    } else {
                        log::warn!(
                            "MoveDataToSimpleDisputes: Could not find first dispute for market id \
                             {:?}",
                            market_id
                        );
                    }
                }

                let new_market = Market {
                    base_asset: old_market.base_asset,
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
                    bonds: MarketBonds {
                        creation: old_market.bonds.creation,
                        oracle: old_market.bonds.oracle,
                        outsider: None,
                        dispute: dispute_bond,
                    },
                };

                Some(new_market)
            },
        );
        log::info!("AddOutsiderAndDisputeBond: Upgraded {} markets.", translated);
        total_weight =
            total_weight.saturating_add(T::DbWeight::get().reads_writes(translated, translated));

        StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION).put::<MarketCommonsPallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("AddOutsiderAndDisputeBond: Done!");
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

        let markets = Markets::<T>::iter_keys().count() as u32;
        let decodable_markets = Markets::<T>::iter_values().count() as u32;
        if markets != decodable_markets {
            log::error!(
                "Can only decode {} of {} markets - others will be dropped",
                decodable_markets,
                markets
            );
        } else {
            log::info!("Markets: {}, Decodable Markets: {}", markets, decodable_markets);
        }

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
            assert_eq!(new_market.base_asset, old_market.base_asset);
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
            assert_eq!(new_market.bonds.oracle, old_market.bonds.oracle);
            assert_eq!(new_market.bonds.creation, old_market.bonds.creation);
            // new fields
            assert_eq!(new_market.bonds.outsider, None);
            // other dispute mechanisms are regarded in the migration after this migration
            if let MarketDisputeMechanism::Authorized = new_market.dispute_mechanism {
                let old_disputes = crate::Disputes::<T>::get(market_id);
                if let Some(first_dispute) = old_disputes.first() {
                    let OldMarketDispute { at: _, by, outcome: _ } = first_dispute;
                    assert_eq!(
                        new_market.bonds.dispute,
                        Some(Bond {
                            who: by.clone(),
                            value: T::DisputeBond::get(),
                            is_settled: false
                        })
                    );
                }
            } else {
                assert_eq!(new_market.bonds.dispute, None);
            }
        }

        log::info!(
            "AddOutsiderAndDisputeBond: Market Counter post-upgrade is {}!",
            new_market_count
        );
        assert!(new_market_count > 0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock::{DisputeBond, ExtBuilder, Runtime},
        MarketIdOf, MarketOf,
    };
    use frame_support::{
        dispatch::fmt::Debug, migration::put_storage_value, Blake2_128Concat, StorageHasher,
    };
    use zrml_market_commons::MarketCommonsPalletApi;

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            AddOutsiderAndDisputeBond::<Runtime>::on_runtime_upgrade();
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
            let (_, new_markets) = construct_old_new_tuple(None);
            populate_test_data::<Blake2_128Concat, MarketIdOf<Runtime>, MarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                new_markets.clone(),
            );
            AddOutsiderAndDisputeBond::<Runtime>::on_runtime_upgrade();
            let actual = <zrml_market_commons::Pallet<Runtime>>::market(&0u128).unwrap();
            assert_eq!(actual, new_markets[0]);
        });
    }

    #[test]
    fn on_runtime_upgrade_correctly_updates_markets_with_none_disputor() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            let (old_markets, new_markets) = construct_old_new_tuple(None);
            populate_test_data::<Blake2_128Concat, MarketIdOf<Runtime>, OldMarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                old_markets,
            );
            AddOutsiderAndDisputeBond::<Runtime>::on_runtime_upgrade();
            let actual = <zrml_market_commons::Pallet<Runtime>>::market(&0u128).unwrap();
            assert_eq!(actual, new_markets[0]);
        });
    }

    #[test]
    fn on_runtime_upgrade_correctly_updates_markets_with_some_disputor() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            let mut disputes = crate::Disputes::<Runtime>::get(0);
            let disputor = crate::mock::EVE;
            let dispute =
                OldMarketDispute { at: 0, by: disputor, outcome: OutcomeReport::Categorical(0u16) };
            disputes.try_push(dispute).unwrap();
            crate::Disputes::<Runtime>::insert(0, disputes);
            let (old_markets, new_markets) = construct_old_new_tuple(Some(disputor));
            populate_test_data::<Blake2_128Concat, MarketIdOf<Runtime>, OldMarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                old_markets,
            );
            AddOutsiderAndDisputeBond::<Runtime>::on_runtime_upgrade();
            let actual = <zrml_market_commons::Pallet<Runtime>>::market(&0u128).unwrap();
            assert_eq!(actual, new_markets[0]);
        });
    }

    fn set_up_version() {
        StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION)
            .put::<MarketCommonsPallet<Runtime>>();
    }

    fn construct_old_new_tuple(
        disputor: Option<u128>,
    ) -> (Vec<OldMarketOf<Runtime>>, Vec<MarketOf<Runtime>>) {
        let base_asset = Asset::Ztg;
        let creator = 999;
        let creator_fee = 1;
        let oracle = 2;
        let metadata = vec![3, 4, 5];
        let market_type = MarketType::Categorical(6);
        let period = MarketPeriod::Block(7..8);
        let scoring_rule = ScoringRule::CPMM;
        let status = MarketStatus::Disputed;
        let creation = MarketCreation::Permissionless;
        let report = None;
        let resolved_outcome = None;
        let dispute_mechanism = MarketDisputeMechanism::Authorized;
        let deadlines = Deadlines::default();
        let old_bonds = OldMarketBonds {
            creation: Some(Bond::new(creator, <Runtime as Config>::ValidityBond::get())),
            oracle: Some(Bond::new(creator, <Runtime as Config>::OracleBond::get())),
        };
        let dispute_bond = disputor.map(|disputor| Bond::new(disputor, DisputeBond::get()));
        let new_bonds = MarketBonds {
            creation: Some(Bond::new(creator, <Runtime as Config>::ValidityBond::get())),
            oracle: Some(Bond::new(creator, <Runtime as Config>::OracleBond::get())),
            outsider: None,
            dispute: dispute_bond,
        };

        let old_market = OldMarket {
            base_asset,
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
            bonds: old_bonds,
        };
        let new_market = Market {
            base_asset,
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
            bonds: new_bonds,
        };
        (vec![old_market], vec![new_market])
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

use frame_support::dispatch::EncodeLike;
use sp_runtime::SaturatedConversion;
use zeitgeist_primitives::types::{MarketDispute, OldMarketDispute};

const PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION: u16 = 7;
const PREDICTION_MARKETS_NEXT_STORAGE_VERSION: u16 = 8;

#[cfg(feature = "try-runtime")]
type OldDisputesOf<T> = frame_support::BoundedVec<
    OldMarketDispute<
        <T as frame_system::Config>::AccountId,
        <T as frame_system::Config>::BlockNumber,
    >,
    <T as crate::Config>::MaxDisputes,
>;

pub struct MoveDataToSimpleDisputes<T>(PhantomData<T>);

impl<T: Config + zrml_simple_disputes::Config + zrml_market_commons::Config> OnRuntimeUpgrade
    for MoveDataToSimpleDisputes<T>
where
    <T as zrml_market_commons::Config>::MarketId: EncodeLike<
        <<T as zrml_simple_disputes::Config>::MarketCommons as MarketCommonsPalletApi>::MarketId,
    >,
{
    fn on_runtime_upgrade() -> Weight {
        use orml_traits::NamedMultiReservableCurrency;

        let mut total_weight = T::DbWeight::get().reads(1);
        let pm_version = StorageVersion::get::<crate::Pallet<T>>();
        if pm_version != PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "MoveDataToSimpleDisputes: market-commons version is {:?}, but {:?} is required",
                pm_version,
                PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("MoveDataToSimpleDisputes: Starting...");

        total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

        // important drain disputes storage item from prediction markets pallet
        for (market_id, old_disputes) in crate::Disputes::<T>::drain() {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
            if let Ok(market) = <zrml_market_commons::Pallet<T>>::market(&market_id) {
                match market.dispute_mechanism {
                    MarketDisputeMechanism::Authorized => continue,
                    // just transform SimpleDispute disputes
                    MarketDisputeMechanism::SimpleDisputes => (),
                    MarketDisputeMechanism::Court => continue,
                }
            } else {
                log::warn!(
                    "MoveDataToSimpleDisputes: Could not find market with market id {:?}",
                    market_id
                );
            }

            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
            let mut new_disputes = zrml_simple_disputes::Disputes::<T>::get(market_id);
            for (i, old_dispute) in old_disputes.iter().enumerate() {
                let bond = zrml_simple_disputes::default_outcome_bond::<T>(i);
                let new_dispute = MarketDispute {
                    at: old_dispute.at,
                    by: old_dispute.by.clone(),
                    outcome: old_dispute.outcome.clone(),
                    bond,
                };
                let res = new_disputes.try_push(new_dispute);
                if res.is_err() {
                    log::error!(
                        "MoveDataToSimpleDisputes: Could not push dispute for market id {:?}",
                        market_id
                    );
                }

                // switch to new reserve identifier for simple disputes
                let sd_reserve_id = <zrml_simple_disputes::Pallet<T>>::reserve_id();
                let pm_reserve_id = <crate::Pallet<T>>::reserve_id();

                // charge weight defensivly for unreserve_named
                // https://github.com/open-web3-stack/open-runtime-module-library/blob/24f0a8b6e04e1078f70d0437fb816337cdf4f64c/tokens/src/lib.rs#L1516-L1547
                total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(4, 3));
                let reserved_balance = <T as Config>::AssetManager::reserved_balance_named(
                    &pm_reserve_id,
                    Asset::Ztg,
                    &old_dispute.by,
                );
                if reserved_balance < bond.saturated_into::<u128>().saturated_into() {
                    log::error!(
                        "MoveDataToSimpleDisputes: Could not unreserve {:?} for {:?} because \
                         reserved balance is only {:?}. Market id: {:?}",
                        bond,
                        old_dispute.by,
                        reserved_balance,
                        market_id,
                    );
                }
                <T as Config>::AssetManager::unreserve_named(
                    &pm_reserve_id,
                    Asset::Ztg,
                    &old_dispute.by,
                    bond.saturated_into::<u128>().saturated_into(),
                );

                // charge weight defensivly for reserve_named
                // https://github.com/open-web3-stack/open-runtime-module-library/blob/24f0a8b6e04e1078f70d0437fb816337cdf4f64c/tokens/src/lib.rs#L1486-L1499
                total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(3, 3));
                let res = <T as Config>::AssetManager::reserve_named(
                    &sd_reserve_id,
                    Asset::Ztg,
                    &old_dispute.by,
                    bond.saturated_into::<u128>().saturated_into(),
                );
                if res.is_err() {
                    log::error!(
                        "MoveDataToSimpleDisputes: Could not reserve bond for dispute caller {:?} \
                         and market id {:?}",
                        old_dispute.by,
                        market_id
                    );
                }
            }

            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
            zrml_simple_disputes::Disputes::<T>::insert(market_id, new_disputes);
        }

        StorageVersion::new(PREDICTION_MARKETS_NEXT_STORAGE_VERSION).put::<crate::Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("MoveDataToSimpleDisputes: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        log::info!("MoveDataToSimpleDisputes: Start pre_upgrade!");

        let old_disputes = crate::Disputes::<T>::iter().collect::<BTreeMap<_, _>>();
        Self::set_temp_storage(old_disputes, "old_disputes");

        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        let old_disputes: BTreeMap<MarketIdOf<T>, OldDisputesOf<T>> =
            Self::get_temp_storage("old_disputes").unwrap();

        log::info!("MoveDataToSimpleDisputes: (post_upgrade) Start first try-runtime part!");

        for (market_id, o) in old_disputes.iter() {
            let market = <zrml_market_commons::Pallet<T>>::market(market_id)
                .expect(&format!("Market for market id {:?} not found", market_id)[..]);

            // market id is a reference, but we need the raw value to encode with the where clause
            let disputes = zrml_simple_disputes::Disputes::<T>::get(*market_id);

            match market.dispute_mechanism {
                MarketDisputeMechanism::Authorized => {
                    let simple_disputes_count = disputes.iter().count();
                    assert_eq!(simple_disputes_count, 0);
                    continue;
                }
                MarketDisputeMechanism::SimpleDisputes => {
                    let new_count = disputes.iter().count();
                    let old_count = o.iter().count();
                    assert_eq!(new_count, old_count);
                }
                MarketDisputeMechanism::Court => {
                    panic!("Court should not be contained at all.")
                }
            }
        }

        log::info!("MoveDataToSimpleDisputes: (post_upgrade) Start second try-runtime part!");

        assert!(crate::Disputes::<T>::iter().count() == 0);

        for (market_id, new_disputes) in zrml_simple_disputes::Disputes::<T>::iter() {
            let old_disputes = old_disputes
                .get(&market_id.saturated_into::<u128>().saturated_into())
                .expect(&format!("Disputes for market {:?} not found", market_id)[..]);

            let market = <T as zrml_simple_disputes::Config>::MarketCommons::market(&market_id)
                .expect(&format!("Market for market id {:?} not found", market_id)[..]);
            match market.dispute_mechanism {
                MarketDisputeMechanism::Authorized => {
                    panic!("Authorized should not be contained in simple disputes.");
                }
                MarketDisputeMechanism::SimpleDisputes => (),
                MarketDisputeMechanism::Court => {
                    panic!("Court should not be contained in simple disputes.");
                }
            }

            for (i, new_dispute) in new_disputes.iter().enumerate() {
                let old_dispute =
                    old_disputes.get(i).expect(&format!("Dispute at index {} not found", i)[..]);
                assert_eq!(new_dispute.at, old_dispute.at);
                assert_eq!(new_dispute.by, old_dispute.by);
                assert_eq!(new_dispute.outcome, old_dispute.outcome);
                assert_eq!(new_dispute.bond, zrml_simple_disputes::default_outcome_bond::<T>(i));
            }
        }

        log::info!("MoveDataToSimpleDisputes: Done! (post_upgrade)");
        Ok(())
    }
}

#[cfg(test)]
mod tests_simple_disputes_migration {
    use super::*;
    use crate::{
        mock::{DisputeBond, ExtBuilder, Runtime},
        MarketOf,
    };
    use orml_traits::NamedMultiReservableCurrency;
    use zrml_market_commons::MarketCommonsPalletApi;

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            MoveDataToSimpleDisputes::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<crate::Pallet<Runtime>>(),
                PREDICTION_MARKETS_NEXT_STORAGE_VERSION
            );
        });
    }

    #[test]
    fn on_runtime_upgrade_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            // Don't set up chain to signal that storage is already up to date.
            let market_id = 0u128;
            let mut disputes = zrml_simple_disputes::Disputes::<Runtime>::get(market_id);
            let dispute = MarketDispute {
                at: 42u64,
                by: 0u128,
                outcome: OutcomeReport::Categorical(0u16),
                bond: DisputeBond::get(),
            };
            disputes.try_push(dispute.clone()).unwrap();
            zrml_simple_disputes::Disputes::<Runtime>::insert(market_id, disputes);
            let market = get_market(MarketDisputeMechanism::SimpleDisputes);
            <zrml_market_commons::Pallet<Runtime>>::push_market(market).unwrap();

            MoveDataToSimpleDisputes::<Runtime>::on_runtime_upgrade();

            let actual = zrml_simple_disputes::Disputes::<Runtime>::get(0);
            assert_eq!(actual, vec![dispute]);
        });
    }

    #[test]
    fn on_runtime_upgrade_correctly_updates_simple_disputes() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            let market_id = 0u128;

            let mut disputes = crate::Disputes::<Runtime>::get(0);
            for i in 0..<Runtime as crate::Config>::MaxDisputes::get() {
                let dispute = OldMarketDispute {
                    at: i as u64 + 42u64,
                    by: i as u128,
                    outcome: OutcomeReport::Categorical(i as u16),
                };
                disputes.try_push(dispute).unwrap();
            }
            crate::Disputes::<Runtime>::insert(market_id, disputes);
            let market = get_market(MarketDisputeMechanism::SimpleDisputes);
            <zrml_market_commons::Pallet<Runtime>>::push_market(market).unwrap();

            MoveDataToSimpleDisputes::<Runtime>::on_runtime_upgrade();

            let mut disputes = zrml_simple_disputes::Disputes::<Runtime>::get(market_id);
            for i in 0..<Runtime as crate::Config>::MaxDisputes::get() {
                let dispute = disputes.get_mut(i as usize).unwrap();

                assert_eq!(dispute.at, i as u64 + 42u64);
                assert_eq!(dispute.by, i as u128);
                assert_eq!(dispute.outcome, OutcomeReport::Categorical(i as u16));

                let bond = zrml_simple_disputes::default_outcome_bond::<Runtime>(i as usize);
                assert_eq!(dispute.bond, bond);
            }
        });
    }

    #[test]
    fn on_runtime_upgrade_correctly_updates_reserve_ids() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            let market_id = 0u128;

            let mut disputes = crate::Disputes::<Runtime>::get(0);
            for i in 0..<Runtime as crate::Config>::MaxDisputes::get() {
                let dispute = OldMarketDispute {
                    at: i as u64 + 42u64,
                    by: i as u128,
                    outcome: OutcomeReport::Categorical(i as u16),
                };
                let bond = zrml_simple_disputes::default_outcome_bond::<Runtime>(i.into());
                let pm_reserve_id = crate::Pallet::<Runtime>::reserve_id();
                let res = <Runtime as crate::Config>::AssetManager::reserve_named(
                    &pm_reserve_id,
                    Asset::Ztg,
                    &dispute.by,
                    bond.saturated_into::<u128>().saturated_into(),
                );
                assert!(res.is_ok());
                disputes.try_push(dispute).unwrap();
            }
            crate::Disputes::<Runtime>::insert(market_id, disputes);
            let market = get_market(MarketDisputeMechanism::SimpleDisputes);
            <zrml_market_commons::Pallet<Runtime>>::push_market(market).unwrap();

            MoveDataToSimpleDisputes::<Runtime>::on_runtime_upgrade();

            let mut disputes = zrml_simple_disputes::Disputes::<Runtime>::get(market_id);
            for i in 0..<Runtime as crate::Config>::MaxDisputes::get() {
                let dispute = disputes.get_mut(i as usize).unwrap();

                let sd_reserve_id = zrml_simple_disputes::Pallet::<Runtime>::reserve_id();
                let reserved_balance =
                    <Runtime as crate::Config>::AssetManager::reserved_balance_named(
                        &sd_reserve_id,
                        Asset::Ztg,
                        &dispute.by,
                    );
                let bond = zrml_simple_disputes::default_outcome_bond::<Runtime>(i.into());
                assert_eq!(reserved_balance, bond);
                assert!(reserved_balance > 0);

                let pm_reserve_id = crate::Pallet::<Runtime>::reserve_id();
                let reserved_balance =
                    <Runtime as crate::Config>::AssetManager::reserved_balance_named(
                        &pm_reserve_id,
                        Asset::Ztg,
                        &dispute.by,
                    );
                assert_eq!(reserved_balance, 0);
            }
        });
    }

    fn set_up_version() {
        StorageVersion::new(PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION)
            .put::<crate::Pallet<Runtime>>();
    }

    fn get_market(dispute_mechanism: MarketDisputeMechanism) -> MarketOf<Runtime> {
        let base_asset = Asset::Ztg;
        let creator = 999;
        let creator_fee = 1;
        let oracle = 2;
        let metadata = vec![3, 4, 5];
        let market_type = MarketType::Categorical(6);
        let period = MarketPeriod::Block(7..8);
        let scoring_rule = ScoringRule::CPMM;
        let status = MarketStatus::Disputed;
        let creation = MarketCreation::Permissionless;
        let report = None;
        let resolved_outcome = None;
        let deadlines = Deadlines::default();
        let bonds = MarketBonds {
            creation: Some(Bond::new(creator, <Runtime as Config>::ValidityBond::get())),
            oracle: Some(Bond::new(creator, <Runtime as Config>::OracleBond::get())),
            outsider: None,
            dispute: None,
        };

        Market {
            base_asset,
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
