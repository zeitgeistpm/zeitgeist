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
#[cfg(feature = "try-runtime")]
use alloc::string::ToString;
use alloc::vec::Vec;
use frame_support::{
    dispatch::Weight,
    log,
    migration::{put_storage_value, storage_iter},
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
};
#[cfg(feature = "try-runtime")]
use scale_info::prelude::format;
use sp_runtime::traits::{One, Saturating};
use zeitgeist_primitives::types::{AuthorityReport, MarketDisputeMechanism, OutcomeReport};
use zrml_authorized::Pallet as AuthorizedPallet;
use zrml_market_commons::MarketCommonsPalletApi;

const PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION: u16 = 6;
const PREDICTION_MARKETS_NEXT_STORAGE_VERSION: u16 = 7;

const AUTHORIZED: &[u8] = b"Authorized";
const AUTHORIZED_OUTCOME_REPORTS: &[u8] = b"AuthorizedOutcomeReports";

const AUTHORIZED_REQUIRED_STORAGE_VERSION: u16 = 2;
const AUTHORIZED_NEXT_STORAGE_VERSION: u16 = 3;

pub struct UpdateMarketIdsPerDisputeBlock<T>(PhantomData<T>);

// Delete the auto resolution of authorized and court from `MarketIdsPerDisputeBlock`
impl<T: Config + zrml_market_commons::Config + zrml_authorized::Config> OnRuntimeUpgrade
    for UpdateMarketIdsPerDisputeBlock<T>
{
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);
        let prediction_markets_version = StorageVersion::get::<Pallet<T>>();
        if prediction_markets_version != PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "UpdateMarketIdsPerDisputeBlock: prediction-markets version is {:?}, require {:?}",
                prediction_markets_version,
                PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("UpdateMarketIdsPerDisputeBlock: Starting...");

        let mut new_storage_map = Vec::new();
        let mut authorized_ids = Vec::new();
        for (key, mut bounded_vec) in crate::MarketIdsPerDisputeBlock::<T>::iter() {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

            bounded_vec.retain(|id| {
                if let Ok(market) = <zrml_market_commons::Pallet<T>>::market(id) {
                    match market.dispute_mechanism {
                        MarketDisputeMechanism::Authorized => {
                            authorized_ids.push(*id);
                            false
                        }
                        MarketDisputeMechanism::Court => false,
                        MarketDisputeMechanism::SimpleDisputes => true,
                    }
                } else {
                    log::warn!("UpdateMarketIdsPerDisputeBlock: Market id {:?} not found!", id);
                    // no market for id in MarketIdsPerDisputeBlock
                    false
                }
            });

            new_storage_map.push((key, bounded_vec));
        }

        for (key, new_bounded_vec) in new_storage_map {
            crate::MarketIdsPerDisputeBlock::<T>::insert(key, new_bounded_vec);
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        }

        let now = <frame_system::Pallet<T>>::block_number();
        let mut resolve_at =
            now.saturating_add(<T as zrml_authorized::Config>::ReportPeriod::get());
        for id in authorized_ids {
            let mut ids = crate::MarketIdsPerDisputeBlock::<T>::get(resolve_at);
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

            while ids.is_full() {
                resolve_at = resolve_at.saturating_add(One::one());
                ids = crate::MarketIdsPerDisputeBlock::<T>::get(resolve_at);
                total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
            }
            // is_full check above to ensure, that we can force_push
            ids.force_push(id);
            crate::MarketIdsPerDisputeBlock::<T>::insert(resolve_at, ids);

            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        }

        StorageVersion::new(PREDICTION_MARKETS_NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("UpdateMarketIdsPerDisputeBlock: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        Ok(())
    }
}

#[cfg(test)]
mod tests_auto_resolution {
    use super::*;
    use crate::{
        mock::{ExtBuilder, Runtime},
        MomentOf,
    };
    use frame_support::BoundedVec;
    use zeitgeist_primitives::types::{
        Deadlines, MarketCreation, MarketDisputeMechanism, MarketId, MarketPeriod, MarketStatus,
        MarketType, OutcomeReport, Report, ScoringRule,
    };
    use zrml_market_commons::Markets;

    type Market = zeitgeist_primitives::types::Market<
        <Runtime as frame_system::Config>::AccountId,
        <Runtime as frame_system::Config>::BlockNumber,
        MomentOf<Runtime>,
    >;

    #[test]
    fn on_runtime_upgrade_increments_the_storage_versions() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();
            UpdateMarketIdsPerDisputeBlock::<Runtime>::on_runtime_upgrade();
            let prediction_markets_version = StorageVersion::get::<Pallet<Runtime>>();
            assert_eq!(prediction_markets_version, PREDICTION_MARKETS_NEXT_STORAGE_VERSION);
        });
    }

    #[test]
    fn on_runtime_updates_market_ids_per_dispute_block_authorized_ids_full() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            let market_id = MarketId::from(0u64);
            let market = get_market(MarketDisputeMechanism::Authorized);

            Markets::<Runtime>::insert(market_id, market);

            <frame_system::Pallet<Runtime>>::set_block_number(42u32.into());

            let now = <frame_system::Pallet<Runtime>>::block_number();
            crate::MarketIdsPerDisputeBlock::<Runtime>::insert(
                now,
                BoundedVec::try_from(vec![market_id]).unwrap(),
            );

            let resolve_at =
                now.saturating_add(<Runtime as zrml_authorized::Config>::ReportPeriod::get());

            let full_ids: Vec<MarketId> = (MarketId::from(1u64)..=MarketId::from(64u64)).collect();

            for id in full_ids.clone() {
                let market = get_market(MarketDisputeMechanism::SimpleDisputes);
                Markets::<Runtime>::insert(id, market);
            }

            crate::MarketIdsPerDisputeBlock::<Runtime>::insert(
                resolve_at,
                BoundedVec::try_from(full_ids.clone()).unwrap(),
            );
            assert!(crate::MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at).is_full());

            UpdateMarketIdsPerDisputeBlock::<Runtime>::on_runtime_upgrade();

            assert_eq!(crate::MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at), full_ids);
            assert!(
                !crate::MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at).contains(&market_id)
            );
            // store market id at the next block
            assert_eq!(
                crate::MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at + 1),
                vec![market_id]
            );
        });
    }

    #[test]
    fn on_runtime_updates_market_ids_per_dispute_block_simple_disputes_unchanged() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            let market_id = MarketId::from(0u64);
            let market = get_market(MarketDisputeMechanism::SimpleDisputes);

            Markets::<Runtime>::insert(market_id, market);

            crate::MarketIdsPerDisputeBlock::<Runtime>::insert(
                0,
                BoundedVec::try_from(vec![market_id]).unwrap(),
            );

            UpdateMarketIdsPerDisputeBlock::<Runtime>::on_runtime_upgrade();

            // unchanged for simple disputes
            assert_eq!(crate::MarketIdsPerDisputeBlock::<Runtime>::get(0), vec![market_id]);
        });
    }

    #[test]
    fn on_runtime_updates_market_ids_per_dispute_block_authorized_deleted() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            let market_id = MarketId::from(0u64);
            let market = get_market(MarketDisputeMechanism::Authorized);

            Markets::<Runtime>::insert(market_id, market);

            crate::MarketIdsPerDisputeBlock::<Runtime>::insert(
                0,
                BoundedVec::try_from(vec![market_id]).unwrap(),
            );

            UpdateMarketIdsPerDisputeBlock::<Runtime>::on_runtime_upgrade();

            // authority controls market resolution now (no auto resolution)
            assert!(crate::MarketIdsPerDisputeBlock::<Runtime>::get(0).is_empty());
        });
    }

    #[test]
    fn on_runtime_updates_market_ids_per_dispute_block_court_deletion() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_chain();

            let market_id = MarketId::from(0u64);
            let market = get_market(MarketDisputeMechanism::Court);
            Markets::<Runtime>::insert(market_id, market);

            crate::MarketIdsPerDisputeBlock::<Runtime>::insert(
                0,
                BoundedVec::try_from(vec![market_id]).unwrap(),
            );

            UpdateMarketIdsPerDisputeBlock::<Runtime>::on_runtime_upgrade();

            // court auto resolution is deactivated for now (court is disabled)
            assert!(crate::MarketIdsPerDisputeBlock::<Runtime>::get(0).is_empty());
        });
    }

    #[test]
    fn on_runtime_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            // Don't set up chain to signal that storage is already up to date.

            let market_id = MarketId::from(0u64);
            let market = get_market(MarketDisputeMechanism::Court);
            Markets::<Runtime>::insert(market_id, market);

            crate::MarketIdsPerDisputeBlock::<Runtime>::insert(
                0,
                BoundedVec::try_from(vec![market_id]).unwrap(),
            );

            UpdateMarketIdsPerDisputeBlock::<Runtime>::on_runtime_upgrade();

            // normally court auto resolution gets deleted with the storage migration,
            // but because storage version is already updated,
            // it is not
            assert_eq!(crate::MarketIdsPerDisputeBlock::<Runtime>::get(0), vec![market_id]);
        });
    }

    fn get_market(mdm: MarketDisputeMechanism) -> Market {
        Market {
            creator: 1,
            creation: MarketCreation::Permissionless,
            creator_fee: 2,
            oracle: 3,
            metadata: vec![4, 5],
            market_type: MarketType::Categorical(14),
            period: MarketPeriod::Block(6..7),
            deadlines: Deadlines { grace_period: 8, oracle_duration: 9, dispute_duration: 10 },
            scoring_rule: ScoringRule::CPMM,
            status: MarketStatus::Disputed,
            report: Some(Report { at: 11, by: 12, outcome: OutcomeReport::Categorical(13) }),
            resolved_outcome: Some(OutcomeReport::Categorical(13)),
            dispute_mechanism: mdm,
        }
    }

    fn set_up_chain() {
        StorageVersion::new(PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
    }
}

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

        let mut new_storage_map = Vec::new();
        let now = <frame_system::Pallet<T>>::block_number();
        for (key, old_value) in
            storage_iter::<Option<OutcomeReport>>(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS)
        {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

            if let Some(outcome) = old_value {
                let resolve_at: T::BlockNumber =
                    now.saturating_add(<T as zrml_authorized::Config>::ReportPeriod::get());
                let new_value = AuthorityReport { resolve_at, outcome };
                new_storage_map.push((key, new_value));
            }
        }

        for (key, new_value) in new_storage_map {
            put_storage_value::<Option<AuthorityReport<T::BlockNumber>>>(
                AUTHORIZED,
                AUTHORIZED_OUTCOME_REPORTS,
                &key,
                Some(new_value),
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
        use frame_support::traits::OnRuntimeUpgradeHelpersExt;
        let counter_key = "counter_key".to_string();
        Self::set_temp_storage(0_u32, &counter_key);
        for (key, value) in
            storage_iter::<Option<OutcomeReport>>(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS)
        {
            Self::set_temp_storage(value, &format!("{:?}", key.as_slice()));

            let counter: u32 =
                Self::get_temp_storage(&counter_key).expect("counter key storage not found");
            Self::set_temp_storage(counter + 1_u32, &counter_key);
        }
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        use frame_support::traits::OnRuntimeUpgradeHelpersExt;
        let mut markets_count = 0_u32;
        let old_counter_key = "counter_key".to_string();
        for (key, new_value) in storage_iter::<Option<AuthorityReport<T::BlockNumber>>>(
            AUTHORIZED,
            AUTHORIZED_OUTCOME_REPORTS,
        ) {
            let key_str = format!("{:?}", key.as_slice());
            if let Some(AuthorityReport { resolve_at: _, outcome }) = new_value {
                let old_value: Option<OutcomeReport> = Self::get_temp_storage(&key_str)
                    .unwrap_or_else(|| panic!("old value not found for market id {:?}", key_str));

                assert_eq!(old_value.unwrap(), outcome);

                markets_count += 1_u32;
            } else {
                panic!(
                    "For market id {:?} storage iter should only find present (Option::Some) \
                     values",
                    key_str
                );
            }
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
    use crate::mock::{ExtBuilder, Runtime};
    use frame_support::Twox64Concat;
    use zeitgeist_primitives::types::{MarketId, OutcomeReport};

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
            put_storage_value::<Option<OutcomeReport>>(
                AUTHORIZED,
                AUTHORIZED_OUTCOME_REPORTS,
                &hash,
                Some(outcome.clone()),
            );

            AddFieldToAuthorityReport::<Runtime>::on_runtime_upgrade();

            let now = <frame_system::Pallet<Runtime>>::block_number();
            let resolve_at: <Runtime as frame_system::Config>::BlockNumber =
                now.saturating_add(<Runtime as zrml_authorized::Config>::ReportPeriod::get());
            let expected = Some(AuthorityReport { resolve_at, outcome });

            let actual = frame_support::migration::get_storage_value::<
                Option<AuthorityReport<<Runtime as frame_system::Config>::BlockNumber>>,
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
            let now = <frame_system::Pallet<Runtime>>::block_number();
            let resolve_at: <Runtime as frame_system::Config>::BlockNumber =
                now.saturating_add(<Runtime as zrml_authorized::Config>::ReportPeriod::get());
            let report = AuthorityReport { resolve_at, outcome };
            put_storage_value::<
                Option<AuthorityReport<<Runtime as frame_system::Config>::BlockNumber>>,
            >(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS, &hash, Some(report.clone()));

            AddFieldToAuthorityReport::<Runtime>::on_runtime_upgrade();

            let actual = frame_support::migration::get_storage_value::<
                Option<AuthorityReport<<Runtime as frame_system::Config>::BlockNumber>>,
            >(AUTHORIZED, AUTHORIZED_OUTCOME_REPORTS, &hash)
            .unwrap();
            assert_eq!(Some(report), actual);
        });
    }

    fn set_up_chain() {
        StorageVersion::new(AUTHORIZED_REQUIRED_STORAGE_VERSION).put::<AuthorizedPallet<Runtime>>();
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
