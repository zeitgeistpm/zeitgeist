use crate::{Config, Disputes, Pallet};
use frame_support::{
    dispatch::Weight,
    log,
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
};
use zeitgeist_primitives::types::MarketStatus;
use zrml_market_commons::MarketCommonsPalletApi;

const PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION: u16 = 1;
const PREDICTION_MARKETS_NEXT_STORAGE_VERSION: u16 = 2;

pub struct RemoveDisputesOfResolvedMarkets<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for RemoveDisputesOfResolvedMarkets<T> {
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);

        if StorageVersion::get::<Pallet<T>>() != PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION {
            log::info!(
                "Skipping storage removal of disputes of resolved markets; prediction-markets \
                 already up to date"
            );
            return total_weight;
        }
        total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
        log::info!("Starting storage migration of RemoveDisputesOfResolvedMarkets");

        for (market_id, market) in T::MarketCommons::market_iter() {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

            if market.status == MarketStatus::Resolved {
                Disputes::<T>::remove(market_id);
                total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
            }
        }
        StorageVersion::new(PREDICTION_MARKETS_NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

        log::info!("Completed storage removal of RemoveDisputesOfResolvedMarkets");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        for (market_id, market) in T::MarketCommons::market_iter() {
            if market.status == MarketStatus::Resolved {
                let disputes = Disputes::<T>::get(market_id);
                assert_eq!(disputes.len(), 0);
            }
        }

        let prediction_markets_storage_version = StorageVersion::get::<Pallet<T>>();
        assert_eq!(
            prediction_markets_storage_version, PREDICTION_MARKETS_NEXT_STORAGE_VERSION,
            "found unexpected prediction-markets pallet storage version. Found: {:?}. Expected: \
             {:?}",
            prediction_markets_storage_version, PREDICTION_MARKETS_NEXT_STORAGE_VERSION,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{mock::*, MomentOf};
    use frame_support::assert_ok;
    use orml_traits::MultiCurrency;
    use zeitgeist_primitives::{
        constants::{BASE, MILLISECS_PER_BLOCK},
        types::{
            Asset, BlockNumber, MarketCreation, MarketDispute, MarketDisputeMechanism,
            MarketPeriod, MarketType, MultiHash, OutcomeReport, ScoringRule,
        },
    };

    #[test]
    fn test_on_runtime_upgrade_on_untouched_chain() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            RemoveDisputesOfResolvedMarkets::<Runtime>::on_runtime_upgrade();
        });
    }

    #[test]
    fn on_runtime_upgrade_updates_storage_versions() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            RemoveDisputesOfResolvedMarkets::<Runtime>::on_runtime_upgrade();
            assert_eq!(
                StorageVersion::get::<Pallet<Runtime>>(),
                PREDICTION_MARKETS_NEXT_STORAGE_VERSION
            );
        });
    }

    #[test]
    fn test_on_runtime_upgrade_with_sample_markets() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            let _ = Currency::deposit(Asset::Ztg, &ALICE, 1_000 * BASE);

            let short_time: MomentOf<Runtime> = (5 * MILLISECS_PER_BLOCK).into();

            create_test_market(
                MarketPeriod::Timestamp(0..short_time),
                MarketCreation::Permissionless,
                MarketStatus::Resolved,
            );

            create_test_market(
                MarketPeriod::Timestamp(0..short_time),
                MarketCreation::Permissionless,
                MarketStatus::Resolved,
            );

            // Add one simple dispute for alreay resolved market to simulate a case
            // where there is pendig dispute(s) for resolver market and that should
            // be cleaned in storage migration.
            let market_dispute =
                MarketDispute { at: 1, by: CHARLIE, outcome: OutcomeReport::Categorical(0) };
            let _res = crate::Disputes::<Runtime>::try_mutate(0, |disputes| {
                let _ = disputes.try_push(market_dispute.clone());
                disputes.try_push(market_dispute.clone())
            });
            let _res = crate::Disputes::<Runtime>::try_mutate(1, |disputes| {
                let _ = disputes.try_push(market_dispute.clone());
                disputes.try_push(market_dispute)
            });

            let disputes = crate::Disputes::<Runtime>::get(&0);
            assert_eq!(disputes.len(), 2);

            let disputes = crate::Disputes::<Runtime>::get(&1);
            assert_eq!(disputes.len(), 2);
            RemoveDisputesOfResolvedMarkets::<Runtime>::on_runtime_upgrade();

            assert_eq!(MarketCommons::market(&0).unwrap().status, MarketStatus::Resolved);
            let disputes = crate::Disputes::<Runtime>::get(&0);
            assert_eq!(disputes.len(), 0);
            let disputes = crate::Disputes::<Runtime>::get(&1);
            assert_eq!(disputes.len(), 0);
        });
    }

    fn setup_chain() {
        StorageVersion::new(PREDICTION_MARKETS_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
    }

    fn create_test_market(
        period: MarketPeriod<BlockNumber, MomentOf<Runtime>>,
        market_creation: MarketCreation,
        market_status: MarketStatus,
    ) {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            period,
            gen_metadata(0),
            market_creation,
            MarketType::Categorical(5),
            MarketDisputeMechanism::Authorized(CHARLIE),
            ScoringRule::CPMM,
        ));
        let market_id = MarketCommons::latest_market_id().unwrap();
        assert_ok!(MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        }));
    }

    fn gen_metadata(byte: u8) -> MultiHash {
        let mut metadata = [byte; 50];
        metadata[0] = 0x15;
        metadata[1] = 0x30;
        MultiHash::Sha3_384(metadata)
    }
}
