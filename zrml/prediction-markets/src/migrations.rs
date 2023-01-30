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
#[cfg(feature = "try-runtime")]
use zrml_market_commons::MarketCommonsPalletApi;
use zrml_market_commons::Pallet as MarketCommonsPallet;

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

pub struct AddOutsiderBond<T>(PhantomData<T>);

impl<T: Config + zrml_market_commons::Config> OnRuntimeUpgrade for AddOutsiderBond<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let market_commons_version = StorageVersion::get::<MarketCommonsPallet<T>>();
        if market_commons_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "AddOutsiderBond: market-commons version is {:?}, but {:?} is required",
                market_commons_version,
                MARKET_COMMONS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("AddOutsiderBond: Starting...");

        let mut translated = 0u64;
        zrml_market_commons::Markets::<T>::translate::<OldMarketOf<T>, _>(|_key, old_market| {
            translated.saturating_inc();

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
                },
            };

            Some(new_market)
        });
        log::info!("AddOutsiderBond: Upgraded {} markets.", translated);
        total_weight =
            total_weight.saturating_add(T::DbWeight::get().reads_writes(translated, translated));

        StorageVersion::new(MARKET_COMMONS_NEXT_STORAGE_VERSION).put::<MarketCommonsPallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("AddOutsiderBond: Done!");
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
            // new field
            assert_eq!(new_market.bonds.outsider, None);
        }
        log::info!("AddOutsiderBond: Market Counter post-upgrade is {}!", new_market_count);
        assert!(new_market_count > 0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock::{ExtBuilder, Runtime},
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
            AddOutsiderBond::<Runtime>::on_runtime_upgrade();
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
            let (_, new_markets) = construct_old_new_tuple();
            populate_test_data::<Blake2_128Concat, MarketIdOf<Runtime>, MarketOf<Runtime>>(
                MARKET_COMMONS,
                MARKETS,
                new_markets.clone(),
            );
            AddOutsiderBond::<Runtime>::on_runtime_upgrade();
            let actual = <zrml_market_commons::Pallet<Runtime>>::market(&0u128).unwrap();
            assert_eq!(actual, new_markets[0]);
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
            AddOutsiderBond::<Runtime>::on_runtime_upgrade();
            let actual = <zrml_market_commons::Pallet<Runtime>>::market(&0u128).unwrap();
            assert_eq!(actual, new_markets[0]);
        });
    }

    fn set_up_version() {
        StorageVersion::new(MARKET_COMMONS_REQUIRED_STORAGE_VERSION)
            .put::<MarketCommonsPallet<Runtime>>();
    }

    fn construct_old_new_tuple() -> (Vec<OldMarketOf<Runtime>>, Vec<MarketOf<Runtime>>) {
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
        let new_bonds = MarketBonds {
            creation: Some(Bond::new(creator, <Runtime as Config>::ValidityBond::get())),
            oracle: Some(Bond::new(creator, <Runtime as Config>::OracleBond::get())),
            outsider: None,
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
