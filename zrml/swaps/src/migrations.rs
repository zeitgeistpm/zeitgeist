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
//
// This file incorporates work covered by the license above but
// published without copyright notice by Balancer Labs
// (<https://balancer.finance>, contact@balancer.finance) in the
// balancer-core repository
// <https://github.com/balancer-labs/balancer-core>.

use crate::{
    types::{Pool, PoolStatus},
    BalanceOf, Config, Pallet as Swaps, PoolOf, PoolsCachedForArbitrage, SubsidyProviders,
};
use alloc::{collections::BTreeMap, vec::Vec};
use core::marker::PhantomData;
use frame_support::{
    log,
    pallet_prelude::{Blake2_128Concat, StorageVersion, Weight},
    traits::{Get, OnRuntimeUpgrade},
};
use parity_scale_codec::{Compact, Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{RuntimeDebug, SaturatedConversion, Saturating};
use zeitgeist_primitives::{
    constants::MAX_ASSETS,
    types::{Asset, MarketId, PoolId, ScoringRule},
};

#[cfg(feature = "try-runtime")]
use frame_support::migration::storage_key_iter;

#[cfg(any(feature = "try-runtime", test))]
const SWAPS: &[u8] = b"Swaps";
#[cfg(any(feature = "try-runtime", test))]
const POOLS: &[u8] = b"Pools";

#[derive(TypeInfo, Clone, Encode, Eq, Decode, PartialEq, RuntimeDebug)]
pub struct OldPool<Balance, MarketId>
where
    MarketId: MaxEncodedLen,
{
    pub assets: Vec<Asset<MarketId>>,
    pub base_asset: Asset<MarketId>,
    pub market_id: MarketId,
    pub pool_status: OldPoolStatus,
    pub scoring_rule: OldScoringRule,
    pub swap_fee: Option<Balance>,
    pub total_subsidy: Option<Balance>,
    pub total_weight: Option<u128>,
    pub weights: Option<BTreeMap<Asset<MarketId>, u128>>,
}

impl<Balance, MarketId> MaxEncodedLen for OldPool<Balance, MarketId>
where
    Balance: MaxEncodedLen,
    MarketId: MaxEncodedLen,
{
    fn max_encoded_len() -> usize {
        let max_encoded_length_bytes = <Compact<u64>>::max_encoded_len();
        let b_tree_map_size = 1usize
            .saturating_add(MAX_ASSETS.saturated_into::<usize>().saturating_mul(
                <Asset<MarketId>>::max_encoded_len().saturating_add(u128::max_encoded_len()),
            ))
            .saturating_add(max_encoded_length_bytes);

        <Asset<MarketId>>::max_encoded_len()
            .saturating_mul(MAX_ASSETS.saturated_into::<usize>())
            .saturating_add(max_encoded_length_bytes)
            .saturating_add(<Option<Asset<MarketId>>>::max_encoded_len())
            .saturating_add(MarketId::max_encoded_len())
            .saturating_add(PoolStatus::max_encoded_len())
            .saturating_add(ScoringRule::max_encoded_len())
            .saturating_add(<Option<Balance>>::max_encoded_len().saturating_mul(2))
            .saturating_add(<Option<u128>>::max_encoded_len())
            .saturating_add(b_tree_map_size)
    }
}

#[derive(TypeInfo, Clone, Copy, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug)]
pub enum OldScoringRule {
    CPMM,
    RikiddoSigmoidFeeMarketEma,
    Lmsr,
    Orderbook,
    Parimutuel,
}

#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Copy,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum OldPoolStatus {
    Active,
    CollectingSubsidy,
    Closed,
    Clean,
    Initialized,
}

pub(crate) type OldPoolOf<T> = OldPool<BalanceOf<T>, MarketId>;

#[frame_support::storage_alias]
pub(crate) type Pools<T: Config> =
    StorageMap<Swaps<T>, Blake2_128Concat, PoolId, Option<OldPoolOf<T>>>;

const SWAPS_REQUIRED_STORAGE_VERSION: u16 = 3;
const SWAPS_NEXT_STORAGE_VERSION: u16 = 4;

#[frame_support::storage_alias]
pub(crate) type Markets<T: Config> = StorageMap<Swaps<T>, Blake2_128Concat, PoolId, OldPoolOf<T>>;

pub struct MigratePools<T>(PhantomData<T>);

/// Deletes all Rikiddo markets from storage, migrates CPMM markets to their new storage layout and
/// closes them. Due to us abstracting `MarketId` away from the `Asset` type of the `Config` object,
/// we require that the old asset type `Asset<MarketId>` be convertible to the generic `T::Asset`.
/// The migration also clears the `SubsidyProviders` and `PoolsCachedForArbitrage` storage elements.
impl<T> OnRuntimeUpgrade for MigratePools<T>
where
    T: Config,
    <T as Config>::Asset: From<Asset<MarketId>>,
{
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let swaps_version = StorageVersion::get::<Swaps<T>>();
        if swaps_version != SWAPS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "MigratePools: swaps version is {:?}, but {:?} is required",
                swaps_version,
                SWAPS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("MigratePools: Starting...");

        let mut reads_writes = 0u64;
        crate::Pools::<T>::translate::<Option<OldPoolOf<T>>, _>(|_, opt_old_pool| {
            // We proceed by deleting Rikiddo pools; CPMM pools are migrated to the new version
            // _and_ closed (because their respective markets are being switched to LMSR).
            reads_writes.saturating_inc();
            let old_pool = opt_old_pool?;
            if old_pool.scoring_rule != OldScoringRule::CPMM {
                return None;
            }
            // These conversions should all be infallible.
            let assets_unbounded =
                old_pool.assets.into_iter().map(Into::into).collect::<Vec<T::Asset>>();
            let assets = assets_unbounded.try_into().ok()?;
            let status = match old_pool.pool_status {
                OldPoolStatus::Active => PoolStatus::Closed,
                OldPoolStatus::CollectingSubsidy => return None,
                OldPoolStatus::Closed => PoolStatus::Closed,
                OldPoolStatus::Clean => PoolStatus::Closed,
                OldPoolStatus::Initialized => PoolStatus::Closed,
            };
            let swap_fee = old_pool.swap_fee?;
            let weights_unbounded = old_pool
                .weights?
                .into_iter()
                .map(|(k, v)| (k.into(), v))
                .collect::<BTreeMap<T::Asset, u128>>();
            let weights = weights_unbounded.try_into().ok()?;
            let total_weight = old_pool.total_weight?;
            let new_pool: PoolOf<T> = Pool { assets, status, swap_fee, total_weight, weights };
            Some(new_pool)
        });
        log::info!("MigratePools: Upgraded {} pools.", reads_writes);
        reads_writes = reads_writes.saturating_add(SubsidyProviders::<T>::drain().count() as u64);
        reads_writes =
            reads_writes.saturating_add(PoolsCachedForArbitrage::<T>::drain().count() as u64);
        total_weight = total_weight
            .saturating_add(T::DbWeight::get().reads_writes(reads_writes, reads_writes));
        StorageVersion::new(SWAPS_NEXT_STORAGE_VERSION).put::<Swaps<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("MigratePools: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        let old_pools =
            storage_key_iter::<PoolId, Option<OldPoolOf<T>>, Blake2_128Concat>(SWAPS, POOLS)
                .collect::<BTreeMap<_, _>>();
        let pools = Pools::<T>::iter_keys().count();
        let decodable_pools = Pools::<T>::iter_values().count();
        if pools == decodable_pools {
            log::info!("All {} pools could successfully be decoded.", pools);
        } else {
            log::error!(
                "Can only decode {} of {} pools - others will be dropped.",
                decodable_pools,
                pools
            );
        }
        Ok(old_pools.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(previous_state: Vec<u8>) -> Result<(), &'static str> {
        let old_pools: BTreeMap<PoolId, Option<OldPoolOf<T>>> =
            Decode::decode(&mut &previous_state[..]).unwrap();
        let old_pool_count = old_pools.len();
        let new_pool_count = crate::Pools::<T>::iter().count();
        assert_eq!(old_pool_count, new_pool_count);
        log::info!("MigratePools: Pool counter post-upgrade is {}!", new_pool_count);
        if PoolsCachedForArbitrage::<T>::iter().count() != 0 {
            return Err("MigratePools: PoolsCachedForArbitrage is not empty!");
        }
        if SubsidyProviders::<T>::iter().count() != 0 {
            return Err("MigratePools: SubsidyProviders is not empty!");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{ExtBuilder, Runtime};
    use alloc::fmt::Debug;
    use frame_support::{migration::put_storage_value, storage_root, StorageHasher};
    use sp_runtime::StateVersion;
    use test_case::test_case;
    use zeitgeist_macros::create_b_tree_map;
    use zeitgeist_primitives::types::Asset;

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            MigratePools::<Runtime>::on_runtime_upgrade();
            assert_eq!(StorageVersion::get::<Swaps<Runtime>>(), SWAPS_NEXT_STORAGE_VERSION);
        });
    }

    #[test]
    fn on_runtime_upgrade_clears_storage_correctly() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            PoolsCachedForArbitrage::<Runtime>::insert(4, ());
            SubsidyProviders::<Runtime>::insert(1, 2, 3);
            MigratePools::<Runtime>::on_runtime_upgrade();
            assert_eq!(PoolsCachedForArbitrage::<Runtime>::iter().count(), 0);
            assert_eq!(SubsidyProviders::<Runtime>::iter().count(), 0);
        });
    }

    #[test_case(OldPoolStatus::Active)]
    #[test_case(OldPoolStatus::Closed)]
    #[test_case(OldPoolStatus::Clean)]
    #[test_case(OldPoolStatus::Initialized)]
    fn on_runtime_upgrade_works_as_expected_with_cpmm(old_pool_status: OldPoolStatus) {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            let base_asset = Asset::ForeignAsset(4);
            let market_id = 1;
            let assets = vec![
                Asset::CategoricalOutcome(market_id, 0),
                Asset::CategoricalOutcome(market_id, 1),
                Asset::CategoricalOutcome(market_id, 2),
                base_asset,
            ];
            let swap_fee = 5;
            let total_weight = 8;
            let weights = create_b_tree_map!({
                Asset::CategoricalOutcome(market_id, 0) => 1,
                Asset::CategoricalOutcome(market_id, 1) => 2,
                Asset::CategoricalOutcome(market_id, 2) => 1,
                base_asset => 8,
            });
            let opt_old_pool = Some(OldPool {
                assets: assets.clone(),
                base_asset,
                market_id,
                pool_status: old_pool_status,
                scoring_rule: OldScoringRule::CPMM,
                swap_fee: Some(swap_fee),
                total_subsidy: None,
                total_weight: Some(total_weight),
                weights: Some(weights.clone()),
            });
            populate_test_data::<Blake2_128Concat, PoolId, Option<OldPoolOf<Runtime>>>(
                SWAPS,
                POOLS,
                vec![opt_old_pool],
            );
            MigratePools::<Runtime>::on_runtime_upgrade();
            let actual = crate::Pools::<Runtime>::get(0);
            let expected = Some(Pool {
                assets: assets.try_into().unwrap(),
                status: PoolStatus::Closed,
                swap_fee,
                total_weight,
                weights: weights.try_into().unwrap(),
            });
            assert_eq!(actual, expected);
        });
    }

    #[test_case(OldPoolStatus::Active)]
    #[test_case(OldPoolStatus::CollectingSubsidy)]
    #[test_case(OldPoolStatus::Closed)]
    #[test_case(OldPoolStatus::Clean)]
    #[test_case(OldPoolStatus::Initialized)]
    fn on_runtime_upgrade_works_as_expected_with_rikiddo(pool_status: OldPoolStatus) {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            let base_asset = Asset::ForeignAsset(4);
            let market_id = 1;
            let assets = vec![
                Asset::CategoricalOutcome(market_id, 0),
                Asset::CategoricalOutcome(market_id, 1),
                Asset::CategoricalOutcome(market_id, 2),
                base_asset,
            ];
            let opt_old_pool = Some(OldPool {
                assets: assets.clone(),
                base_asset,
                market_id,
                pool_status,
                scoring_rule: OldScoringRule::RikiddoSigmoidFeeMarketEma,
                swap_fee: Some(5),
                total_subsidy: Some(123),
                total_weight: None,
                weights: None,
            });
            populate_test_data::<Blake2_128Concat, PoolId, Option<OldPoolOf<Runtime>>>(
                SWAPS,
                POOLS,
                vec![opt_old_pool],
            );
            MigratePools::<Runtime>::on_runtime_upgrade();
            let actual = crate::Pools::<Runtime>::get(0);
            assert_eq!(actual, None);
        });
    }

    #[test]
    fn on_runtime_upgrade_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            StorageVersion::new(SWAPS_NEXT_STORAGE_VERSION).put::<Swaps<Runtime>>();
            let assets =
                vec![Asset::ForeignAsset(0), Asset::ForeignAsset(1), Asset::ForeignAsset(2)];
            let weights = create_b_tree_map!({
                Asset::ForeignAsset(0) => 3,
                Asset::ForeignAsset(1) => 4,
                Asset::ForeignAsset(2) => 5,
            });
            let pool = Pool {
                assets: assets.try_into().unwrap(),
                status: PoolStatus::Open,
                swap_fee: 4,
                total_weight: 12,
                weights: weights.try_into().unwrap(),
            };
            crate::Pools::<Runtime>::insert(0, pool);
            PoolsCachedForArbitrage::<Runtime>::insert(4, ());
            SubsidyProviders::<Runtime>::insert(1, 2, 3);
            let tmp = storage_root(StateVersion::V1);
            MigratePools::<Runtime>::on_runtime_upgrade();
            assert_eq!(tmp, storage_root(StateVersion::V1));
        });
    }

    fn set_up_version() {
        StorageVersion::new(SWAPS_REQUIRED_STORAGE_VERSION).put::<Swaps<Runtime>>();
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
