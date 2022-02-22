pub mod _0_1_2_move_storage_to_simple_disputes_and_market_commons {
    use crate::{Config, MarketIdOf, MomentOf};
    use alloc::vec::Vec;
    use frame_support::{
        dispatch::Weight, migration, pallet_prelude::ConstU32, traits::Get,
        WeakBoundedVec,
    };
    use parity_scale_codec::Encode;
    use zeitgeist_primitives::types::{MarketDispute, SubsidyUntil};

    const DISPUTES: &[u8] = b"Disputes";
    const MARKET_IDS_PER_DISPUTE_BLOCK: &[u8] = b"MarketIdsPerDisputeBlock";
    const MARKET_IDS_PER_REPORT_BLOCK: &[u8] = b"MarketIdsPerReportBlock";
    const MARKETS_COLLECTING_SUBSIDY: &[u8] = b"MarketsCollectingSubsidy";
    const PM: &[u8] = b"PredictionMarkets";

    pub fn migrate<T>() -> Weight
    where
        T: Config,
    {
        let mut weight: Weight = 0;
        let previous_version = 42; // TODO: Use new versioning system
        let storage_version = 42; 

        if storage_version == previous_version {
            // Disputes storage: Vec -> WeakBoundedVec
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

            // MarketIdsPerDisputeBlock storage: Vec -> WeakBoundedVec
            for (k, v) in
                migration::storage_iter::<Vec<MarketIdOf<T>>>(PM, MARKET_IDS_PER_DISPUTE_BLOCK)
            {
                let new_value: WeakBoundedVec<MarketIdOf<T>, ConstU32<1024>> =
                    WeakBoundedVec::force_from(
                        v,
                        Some("MarketIdsPerDisputeBlock storage: Vec -> WeakBoundedVec"),
                    );
                migration::put_storage_value(&PM, &MARKET_IDS_PER_DISPUTE_BLOCK, &k, new_value);
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
            }

            // MarketIdsPerReportBlock storage: Vec -> WeakBoundedVec
            for (k, v) in
                migration::storage_iter::<Vec<MarketIdOf<T>>>(PM, MARKET_IDS_PER_REPORT_BLOCK)
            {
                let new_value: WeakBoundedVec<MarketIdOf<T>, ConstU32<1024>> =
                    WeakBoundedVec::force_from(
                        v,
                        Some("MarketIdsPerReportBlock storage: Vec -> WeakBoundedVec"),
                    );
                migration::put_storage_value(&PM, &MARKET_IDS_PER_REPORT_BLOCK, &k, new_value);
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
            }

            // MarketsCollectingSubsidy storage: Vec -> WeakBoundedVec
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
        }

        weight
    }

    #[cfg(test)]
    mod test {
        // TODO :)
        #[test]
        fn data_is_consistent_after_migrations() {
            let res = super::migrate::<crate::Runtime>();
        }
    }
}
