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
    Config, MarketIdsPerOpenBlock, MarketIdsPerOpenTimeFrame, MarketsCollectingSubsidy,
    Pallet as PredictionMarkets,
};
use core::marker::PhantomData;
use frame_support::{
    log,
    pallet_prelude::{StorageVersion, Weight},
    traits::{Get, OnRuntimeUpgrade},
};

#[cfg(feature = "try-runtime")]
use alloc::vec::Vec;

const PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION: u16 = 7;
const PREDICTION_MARKETS_NEXT_STORAGE_VERSION: u16 = 8;

pub struct DrainDeprecatedStorage<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for DrainDeprecatedStorage<T>
where
    T: Config,
{
    fn on_runtime_upgrade() -> Weight {
        let mut total_weight = T::DbWeight::get().reads(1);
        let prediction_markets_version = StorageVersion::get::<PredictionMarkets<T>>();
        if prediction_markets_version != PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "DrainDeprecatedStorage: prediction-markets version is {:?}, but {:?} is required",
                prediction_markets_version,
                PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION,
            );
            return total_weight;
        }
        log::info!("DrainDeprecatedStorage: Starting...");
        let mut reads_writes = 1u64; // For killing MarketsCollectingSubsidy
        reads_writes =
            reads_writes.saturating_add(MarketIdsPerOpenBlock::<T>::drain().count() as u64);
        reads_writes =
            reads_writes.saturating_add(MarketIdsPerOpenTimeFrame::<T>::drain().count() as u64);
        MarketsCollectingSubsidy::<T>::kill();
        log::info!("DrainDeprecatedStorage: Drained {} keys.", reads_writes);
        total_weight = total_weight
            .saturating_add(T::DbWeight::get().reads_writes(reads_writes, reads_writes));
        StorageVersion::new(PREDICTION_MARKETS_NEXT_STORAGE_VERSION).put::<PredictionMarkets<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("DrainDeprecatedStorage: Done!");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_: Vec<u8>) -> Result<(), &'static str> {
        if MarketIdsPerOpenBlock::<T>::iter().count() != 0 {
            return Err("DrainDeprecatedStorage: MarketIdsPerOpenBlock is not empty!");
        }
        if MarketIdsPerOpenTimeFrame::<T>::iter().count() != 0 {
            return Err("DrainDeprecatedStorage: MarketIdsPerOpenTimeFrame is not empty!");
        }
        if MarketsCollectingSubsidy::<T>::exists() {
            return Err("DrainDeprecatedStorage: MarketsCollectingSubsidy still exists!");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock::{ExtBuilder, Runtime},
        CacheSize,
    };
    use frame_support::{
        dispatch::fmt::Debug, migration::put_storage_value, storage_root, StorageHasher,
    };
    use parity_scale_codec::Encode;
    use sp_runtime::{traits::ConstU32, BoundedVec, StateVersion};
    use zeitgeist_primitives::types::{MarketPeriod, SubsidyUntil};

    #[test]
    fn on_runtime_upgrade_increments_the_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            DrainDeprecatedStorage::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<PredictionMarkets<Runtime>>(),
                PREDICTION_MARKETS_NEXT_STORAGE_VERSION
            );
        });
    }

    #[test]
    fn on_runtime_upgrade_works() {
        ExtBuilder::default().build().execute_with(|| {
            set_up_version();
            set_up_storage();
            DrainDeprecatedStorage::<Runtime>::on_runtime_upgrade();
            assert_eq!(MarketIdsPerOpenBlock::<Runtime>::iter().count(), 0);
            assert_eq!(MarketIdsPerOpenTimeFrame::<Runtime>::iter().count(), 0);
            assert!(!MarketsCollectingSubsidy::<Runtime>::exists());
        });
    }

    #[test]
    fn on_runtime_upgrade_is_noop_if_versions_are_not_correct() {
        ExtBuilder::default().build().execute_with(|| {
            StorageVersion::new(PREDICTION_MARKETS_NEXT_STORAGE_VERSION)
                .put::<PredictionMarkets<Runtime>>();
            set_up_storage();
            let tmp = storage_root(StateVersion::V1);
            DrainDeprecatedStorage::<Runtime>::on_runtime_upgrade();
            assert_eq!(tmp, storage_root(StateVersion::V1));
        });
    }

    fn set_up_version() {
        StorageVersion::new(PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION)
            .put::<PredictionMarkets<Runtime>>();
    }

    fn set_up_storage() {
        let market_ids_per_open_block: BoundedVec<_, CacheSize> = vec![1, 2, 3].try_into().unwrap();
        MarketIdsPerOpenBlock::<Runtime>::insert(1, market_ids_per_open_block);
        let market_ids_per_open_time_frame: BoundedVec<_, CacheSize> =
            vec![4, 5, 6].try_into().unwrap();
        MarketIdsPerOpenTimeFrame::<Runtime>::insert(2, market_ids_per_open_time_frame);
        let subsidy_until: BoundedVec<_, ConstU32<16>> =
            vec![SubsidyUntil { market_id: 7, period: MarketPeriod::Block(8..9) }]
                .try_into()
                .unwrap();
        MarketsCollectingSubsidy::<Runtime>::put(subsidy_until);
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
