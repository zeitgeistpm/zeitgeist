pub mod convert_vec_to_weak_bounded_vec {
    use crate::{Config, MarketIdOf, MomentOf, Pallet};
    use alloc::vec::Vec;
    use frame_support::{
        dispatch::Weight,
        migration,
        pallet_prelude::ConstU32,
        traits::{Get, StorageVersion},
        WeakBoundedVec,
    };
    use migration::{put_storage_value, storage_iter};
    use zeitgeist_primitives::types::{MarketDispute, SubsidyUntil};

    const DISPUTES: &[u8] = b"Disputes";
    const MARKET_IDS_PER_DISPUTE_BLOCK: &[u8] = b"MarketIdsPerDisputeBlock";
    const MARKET_IDS_PER_REPORT_BLOCK: &[u8] = b"MarketIdsPerReportBlock";
    const MARKETS_COLLECTING_SUBSIDY: &[u8] = b"MarketsCollectingSubsidy";
    const PM: &[u8] = b"PredictionMarkets";
    const REQUIRED_STORAGE_VERSION: u16 = 0;
    #[allow(clippy::integer_arithmetic)]
    const NEXT_STORAGE_VERSION: u16 = REQUIRED_STORAGE_VERSION + 1;

    pub fn migrate<T: Config>() -> Weight {
        let mut total_weight: Weight = 0;

        if StorageVersion::get::<Pallet<T>>() == REQUIRED_STORAGE_VERSION {
            log::info!("Started migrations: Vec -> WeakBoundedVec");

            total_weight = migrate_disputes::<T>();
            total_weight = total_weight.saturating_add(migrate_market_id_per_dispute_block::<T>());
            total_weight = total_weight.saturating_add(migrate_market_ids_per_report_block::<T>());
            total_weight = total_weight.saturating_add(migrate_markets_collecting_subsidy::<T>());

            StorageVersion::new(NEXT_STORAGE_VERSION).put::<Pallet<T>>();
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

            log::info!(
                "Completed migrations: Vec -> WeakBoundedVec. Total weight consumed: {}",
                total_weight
            );
        }

        total_weight
    }

    // Disputes storage: Vec -> WeakBoundedVec
    fn migrate_disputes<T: Config>() -> Weight {
        log::info!("Started Disputes migration: Vec -> WeakBoundedVec");
        let mut weight: Weight = 0;

        for (k, v) in storage_iter::<Vec<MarketDispute<T::AccountId, T::BlockNumber>>>(PM, DISPUTES)
        {
            let new_value: WeakBoundedVec<
                MarketDispute<T::AccountId, T::BlockNumber>,
                T::MaxDisputes,
            > = WeakBoundedVec::force_from(v, Some("Disputes storage: Vec -> WeakBoundedVec"));
            put_storage_value(&PM, &DISPUTES, &k, new_value);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
        }

        log::info!("Completed Disputes migration. Consumed weight: {}", weight);
        weight
    }

    // MarketIdsPerDisputeBlock storage: Vec -> WeakBoundedVec
    fn migrate_market_id_per_dispute_block<T: Config>() -> Weight {
        log::info!("Started MarketIdsPerDisputeBlock Migration: Vec -> WeakBoundedVec");
        let mut weight: Weight = 0;

        for (k, v) in storage_iter::<Vec<MarketIdOf<T>>>(PM, MARKET_IDS_PER_DISPUTE_BLOCK) {
            let new_value: WeakBoundedVec<MarketIdOf<T>, ConstU32<1024>> =
                WeakBoundedVec::force_from(
                    v,
                    Some("MarketIdsPerDisputeBlock storage: Vec -> WeakBoundedVec"),
                );
            put_storage_value(&PM, &MARKET_IDS_PER_DISPUTE_BLOCK, &k, new_value);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
        }

        log::info!("Completed MarketIdsPerDisputeBlock Migration. Consumed weight: {}", weight);
        weight
    }

    // MarketIdsPerReportBlock storage: Vec -> WeakBoundedVec
    fn migrate_market_ids_per_report_block<T: Config>() -> Weight {
        log::info!("Started MarketIdsPerReportBlock Migration: Vec -> WeakBoundedVec");
        let mut weight: Weight = 0;

        for (k, v) in storage_iter::<Vec<MarketIdOf<T>>>(PM, MARKET_IDS_PER_REPORT_BLOCK) {
            let new_value: WeakBoundedVec<MarketIdOf<T>, ConstU32<1024>> =
                WeakBoundedVec::force_from(
                    v,
                    Some("MarketIdsPerReportBlock storage: Vec -> WeakBoundedVec"),
                );
            put_storage_value(&PM, &MARKET_IDS_PER_REPORT_BLOCK, &k, new_value);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
        }

        log::info!("Completed MarketIdsPerReportBlock Migration. Consumed weight: {}", weight);
        weight
    }

    // MarketsCollectingSubsidy storage: Vec -> WeakBoundedVec
    fn migrate_markets_collecting_subsidy<T: Config>() -> Weight {
        log::info!("Started MarketsCollectingSubsidy Migration: Vec -> WeakBoundedVec");
        let mut weight: Weight = 0;

        let empty_key_hash = &[][..];
        let old_value =
            migration::get_storage_value(&PM, &MARKETS_COLLECTING_SUBSIDY, empty_key_hash);

        if let Some(content) = old_value {
            let new_value: WeakBoundedVec<
                SubsidyUntil<T::BlockNumber, MomentOf<T>, MarketIdOf<T>>,
                frame_support::traits::ConstU32<1_048_576>,
            > = WeakBoundedVec::force_from(
                content,
                Some("MarketsCollectingSubsidy storage: Vec -> WeakBoundedVec"),
            );
            put_storage_value(&PM, &MARKETS_COLLECTING_SUBSIDY, empty_key_hash, new_value);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
        } else {
            weight = weight.saturating_add(T::DbWeight::get().reads(1));
        }

        log::info!("Completed MarketsCollectingSubsidy Migration. Consumed weight: {}", weight);
        weight
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::mock::{ExtBuilder, Runtime};
        use core::{cmp::PartialEq, fmt::Debug, ops::Deref};
        use frame_support::{Blake2_128Concat, StorageHasher, Twox64Concat};
        use parity_scale_codec::{Decode, Encode};

        type AccountId = <Runtime as frame_system::Config>::AccountId;
        type BlockNumber = <Runtime as frame_system::Config>::BlockNumber;

        /*
        #[test]
        fn disputes() {
            check_vec_to_weak_bounded_vec_migration::<
                Vec<MarketDispute<AccountId, BlockNumber>>,
                WeakBoundedVec<MarketDispute<AccountId, BlockNumber>, <Runtime as Config>::MaxDisputes>,
            >(PM, DISPUTES);
        }
        */

        #[test]
        fn market_id_per_dispute_block() {
            ExtBuilder::default().build().execute_with(|| {
                let mut all_disputed_market_ids: Vec<Vec<MarketIdOf<Runtime>>> =
                    Vec::with_capacity(50);
                let mut current_market_ids: Vec<MarketIdOf<Runtime>> = Vec::with_capacity(50);

                for i in 0..50 {
                    current_market_ids.push(i);
                    all_disputed_market_ids.push(current_market_ids.clone())
                }

                populate_test_data::<Twox64Concat, MarketIdOf<Runtime>, Vec<MarketIdOf<Runtime>>>(
                    PM,
                    MARKET_IDS_PER_DISPUTE_BLOCK,
                    all_disputed_market_ids,
                );

                check_vec_to_weak_bounded_vec_migration::<
                    Vec<MarketIdOf<Runtime>>,
                    WeakBoundedVec<MarketIdOf<Runtime>, ConstU32<1024>>,
                >(PM, MARKET_IDS_PER_DISPUTE_BLOCK);
            });
        }

        /*
        #[test]
        fn market_id_per_report_block() {
            check_vec_to_weak_bounded_vec_migration::<
                Vec<MarketIdOf<Runtime>>,
                WeakBoundedVec<MarketIdOf<Runtime>, ConstU32<1024>>,
            >(PM, MARKET_IDS_PER_REPORT_BLOCK);
        }
        */

        fn check_vec_to_weak_bounded_vec_migration<B, A>(pallet: &[u8], prefix: &[u8])
        where
            B: Decode + PartialEq + Debug,
            A: Decode + PartialEq + Debug + Deref<Target = B>,
        {
            let mut data_old = storage_iter::<B>(pallet, prefix).map(|v| v.1);
            migrate::<Runtime>();
            let mut data_new = storage_iter::<A>(pallet, prefix).map(|v| v.1);

            for (before, after) in data_old.by_ref().zip(data_new.by_ref()) {
                assert_eq!(before, *after);
            }

            // Both storages have the same size.
            assert_eq!(data_old.next(), None);
            assert_eq!(data_new.next(), None);
        }

        fn populate_test_data<H, K, V>(pallet: &[u8], prefix: &[u8], data: Vec<V>)
        where
            H: StorageHasher,
            K: TryFrom<usize> + Encode,
            V: Encode,
            <K as TryFrom<usize>>::Error: Debug,
        {
            for (key, value) in data.iter().enumerate() {
                let storage_hash = K::try_from(key).unwrap().using_encoded(H::hash);
                put_storage_value(pallet, prefix, storage_hash.as_ref(), value);
            }
        }
    }
}
