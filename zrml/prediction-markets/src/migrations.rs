pub mod _0_1_1_move_storage_to_simple_disputes_and_market_commons {
    use crate::{Config, MarketIdOf, Pallet};
    use alloc::vec::Vec;
    use frame_support::{
        dispatch::Weight,
        migration,
        traits::{Get, GetPalletVersion, PalletVersion},
        Blake2_128Concat,
    };
    use zeitgeist_primitives::types::{Market, PoolId};
    use zrml_market_commons::MarketCommonsPalletApi;
    use zrml_simple_disputes::DisputeApi;

    const DISPUTES: &[u8] = b"Disputes";
    const MARKET_COUNT: &[u8] = b"MarketCount";
    const MARKET_IDS_PER_DISPUTE_BLOCK: &[u8] = b"MarketIdsPerDisputeBlock";
    const MARKET_IDS_PER_REPORT_BLOCK: &[u8] = b"MarketIdsPerReportBlock";
    const MARKET_TO_SWAP_POOL: &[u8] = b"MarketToSwapPool";
    const MARKETS: &[u8] = b"Markets";
    const PM: &[u8] = b"PredictionMarket";

    pub fn migrate<T>() -> Weight
    where
        T: Config,
    {
        let mut weight: Weight = 0;
        let previous_version = PalletVersion { major: 0, minor: 1, patch: 1 };
        let storage_version = <Pallet<T>>::storage_version().unwrap_or(previous_version);

        if storage_version == previous_version {
            // Simple disputes

            for (k, v) in
                migration::storage_key_iter::<MarketIdOf<T>, Vec<_>, Blake2_128Concat>(PM, DISPUTES)
            {
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
                T::SimpleDisputes::insert_dispute(k, v);
            }
            migration::remove_storage_prefix(PM, DISPUTES, b"");

            for (k, v) in migration::storage_key_iter::<
                T::BlockNumber,
                Vec<MarketIdOf<T>>,
                Blake2_128Concat,
            >(PM, MARKET_IDS_PER_DISPUTE_BLOCK)
            {
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
                T::SimpleDisputes::insert_market_id_per_dispute_block(k, v);
            }
            migration::remove_storage_prefix(PM, MARKET_IDS_PER_DISPUTE_BLOCK, b"");

            for (k, v) in migration::storage_key_iter::<
                T::BlockNumber,
                Vec<MarketIdOf<T>>,
                Blake2_128Concat,
            >(PM, MARKET_IDS_PER_REPORT_BLOCK)
            {
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
                T::SimpleDisputes::insert_market_id_per_report_block(k, v);
            }
            migration::remove_storage_prefix(PM, MARKET_IDS_PER_REPORT_BLOCK, b"");

            // Market commons

            if let Some(market_counter) =
                migration::take_storage_value::<MarketIdOf<T>>(PM, MARKET_COUNT, b"")
            {
                T::MarketCommons::set_market_counter(market_counter);
            }
            migration::remove_storage_prefix(PM, MARKET_COUNT, b"");

            for (k, v) in migration::storage_key_iter::<
                MarketIdOf<T>,
                Option<Market<T::AccountId, T::BlockNumber>>,
                Blake2_128Concat,
            >(PM, MARKETS)
            .filter_map(|(k, v)| Some((k, v?)))
            {
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
                T::MarketCommons::insert_market(k, v);
            }
            migration::remove_storage_prefix(PM, MARKETS, b"");

            for (k, v) in migration::storage_key_iter::<
                MarketIdOf<T>,
                Option<PoolId>,
                Blake2_128Concat,
            >(PM, MARKET_TO_SWAP_POOL)
            .filter_map(|(k, v)| Some((k, v?)))
            {
                weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
                T::MarketCommons::insert_market_pool(k, v);
            }
            migration::remove_storage_prefix(PM, MARKET_TO_SWAP_POOL, b"");
        }

        weight
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use crate::mock::{ExtBuilder, MarketCommons, PredictionMarkets, SimpleDisputes};
        use frame_support::{traits::OnRuntimeUpgrade, Hashable};
        use zeitgeist_primitives::types::{
            Market, MarketCreation, MarketDispute, MarketEnd, MarketStatus, MarketType,
            OutcomeReport,
        };

        #[test]
        fn migration_works() {
            const DEFAULT_MARKET: Market<u128, u64> = Market {
                creation: MarketCreation::Permissionless,
                creator_fee: 0,
                creator: 0,
                end: MarketEnd::Block(0),
                market_type: MarketType::Categorical(0),
                metadata: vec![],
                oracle: 0,
                report: None,
                resolved_outcome: None,
                status: MarketStatus::Closed,
            };
            const DEFAULT_MARKET_DISPUTE: MarketDispute<u128, u64> =
                MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(0) };

            ExtBuilder::default().build().execute_with(|| {
                assert!(SimpleDisputes::dispute(&0).is_none());
                assert!(SimpleDisputes::market_ids_per_dispute_block(&0).is_err());
                assert!(SimpleDisputes::market_ids_per_report_block(&0).is_err());

                assert!(MarketCommons::latest_market_id().is_err());
                assert!(MarketCommons::market(&0).is_err());
                assert!(MarketCommons::market_pool(&0).is_err());

                migration::put_storage_value(
                    PM,
                    DISPUTES,
                    &0u128.blake2_128_concat(),
                    vec![DEFAULT_MARKET_DISPUTE],
                );
                migration::put_storage_value(
                    PM,
                    MARKET_IDS_PER_DISPUTE_BLOCK,
                    &0u128.blake2_128_concat(),
                    vec![1u128],
                );
                migration::put_storage_value(
                    PM,
                    MARKET_IDS_PER_REPORT_BLOCK,
                    &0u128.blake2_128_concat(),
                    vec![1u128],
                );

                migration::put_storage_value(PM, MARKET_COUNT, b"", 0u128);
                migration::put_storage_value(
                    PM,
                    MARKETS,
                    &0u128.blake2_128_concat(),
                    Some(DEFAULT_MARKET),
                );
                migration::put_storage_value(
                    PM,
                    MARKET_TO_SWAP_POOL,
                    &0u128.blake2_128_concat(),
                    Some(1u128),
                );

                PredictionMarkets::on_runtime_upgrade();

                assert!(
                    migration::get_storage_value::<Vec<MarketDispute<u128, u64>>>(
                        PM,
                        DISPUTES,
                        &0u128.blake2_128_concat()
                    )
                    .is_none()
                );
                assert!(
                    migration::get_storage_value::<Vec<u128>>(
                        PM,
                        MARKET_IDS_PER_DISPUTE_BLOCK,
                        &0u128.blake2_128_concat()
                    )
                    .is_none()
                );
                assert!(
                    migration::get_storage_value::<Vec<u128>>(
                        PM,
                        MARKET_IDS_PER_REPORT_BLOCK,
                        &0u128.blake2_128_concat()
                    )
                    .is_none()
                );

                assert!(migration::get_storage_value::<u128>(PM, MARKET_COUNT, b"").is_none());
                assert!(
                    migration::get_storage_value::<Option<Market<u128, u64>>>(
                        PM,
                        MARKETS,
                        &0u128.blake2_128_concat()
                    )
                    .is_none()
                );
                assert!(
                    migration::get_storage_value::<Option<u128>>(
                        PM,
                        MARKET_TO_SWAP_POOL,
                        &0u128.blake2_128_concat()
                    )
                    .is_none()
                );

                assert_eq!(SimpleDisputes::dispute(&0).unwrap(), vec![DEFAULT_MARKET_DISPUTE]);
                assert_eq!(SimpleDisputes::market_ids_per_dispute_block(&0).unwrap(), vec![1]);
                assert_eq!(SimpleDisputes::market_ids_per_report_block(&0).unwrap(), vec![1]);

                assert_eq!(MarketCommons::latest_market_id().unwrap(), 0);
                assert_eq!(MarketCommons::market(&0).unwrap(), DEFAULT_MARKET);
                assert_eq!(MarketCommons::market_pool(&0).unwrap(), 1);
            })
        }
    }
}
