pub mod convert_vec_to_weak_bounded_vec {
    use crate::{Config, MarketIdOf, MomentOf, Pallet};
    use alloc::vec::Vec;
    use frame_support::{
        dispatch::Weight, migration, pallet_prelude::ConstU32, traits::{Get, StorageVersion},
        WeakBoundedVec,
    };
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

            log::info!("Completed migrations: Vec -> WeakBoundedVec. Total weight consumed: {}", total_weight);
        }

        
        total_weight
    }

    // Disputes storage: Vec -> WeakBoundedVec
    fn migrate_disputes<T: Config>() -> Weight {
        log::info!("Started Disputes migration: Vec -> WeakBoundedVec");
        let mut weight: Weight = 0;

        for (k, v) in migration::storage_iter::<Vec<MarketDispute<T::AccountId, T::BlockNumber>>>(
            PM, DISPUTES,
        ) {
            let new_value: WeakBoundedVec<
                MarketDispute<T::AccountId, T::BlockNumber>,
                T::MaxDisputes,
            > = WeakBoundedVec::force_from(v, Some("Disputes storage: Vec -> WeakBoundedVec"));
            migration::put_storage_value(&PM, &DISPUTES, &k, new_value);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
        }

        log::info!("Completed Disputes migration. Consumed weight: {}", weight);
        weight
    }

    // MarketIdsPerDisputeBlock storage: Vec -> WeakBoundedVec
    fn migrate_market_id_per_dispute_block<T: Config>() -> Weight {
        log::info!("Started MarketIdsPerDisputeBlock Migration: Vec -> WeakBoundedVec");
        let mut weight: Weight = 0;

        for (k, v) in migration::storage_iter::<Vec<MarketIdOf<T>>>(PM, MARKET_IDS_PER_DISPUTE_BLOCK)
        {
            let new_value: WeakBoundedVec<MarketIdOf<T>, ConstU32<1024>> =
                WeakBoundedVec::force_from(
                    v,
                    Some("MarketIdsPerDisputeBlock storage: Vec -> WeakBoundedVec"),
                );
            migration::put_storage_value(&PM, &MARKET_IDS_PER_DISPUTE_BLOCK, &k, new_value);
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
        }

        log::info!("Completed MarketIdsPerDisputeBlock Migration. Consumed weight: {}", weight);
        weight
    }

    // MarketIdsPerReportBlock storage: Vec -> WeakBoundedVec
    fn migrate_market_ids_per_report_block<T: Config>() -> Weight {
        log::info!("Started MarketIdsPerReportBlock Migration: Vec -> WeakBoundedVec");
        let mut weight: Weight = 0;

        for (k, v) in migration::storage_iter::<Vec<MarketIdOf<T>>>(PM, MARKET_IDS_PER_REPORT_BLOCK)
        {
            let new_value: WeakBoundedVec<MarketIdOf<T>, ConstU32<1024>> =
                WeakBoundedVec::force_from(
                    v,
                    Some("MarketIdsPerReportBlock storage: Vec -> WeakBoundedVec"),
                );
            migration::put_storage_value(&PM, &MARKET_IDS_PER_REPORT_BLOCK, &k, new_value);
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
            migration::put_storage_value(
                &PM,
                &MARKETS_COLLECTING_SUBSIDY,
                empty_key_hash,
                new_value,
            );
            weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
        } else {
            weight = weight.saturating_add(T::DbWeight::get().reads(1));
        }
        
        log::info!("Completed MarketsCollectingSubsidy Migration. Consumed weight: {}", weight);
        weight
    }

    #[cfg(test)]
    mod test {
        use crate::mock::{ExtBuilder, Runtime};
        use super::*;

        // TODO :)
        // Collect every element, compare length and contents
        #[test]
        fn data_is_consistent_after_migrations() {
            ExtBuilder::default().build().execute_with(|| {
                // let disputes_old: Vec<MarketDispute<<Runtime as frame_system::Config>::AccountId, <Runtime as frame_system::Config>::BlockNumber>> = <Disputes<Runtime>>::iter().collect();
                migrate::<Runtime>();
                // let disputes_new = <Disputes<Runtime>>::iter().collect();
                // assert!(disputes_old.len() == disputes_new.len());
            });
        }

        //fn compare()
    }
}
