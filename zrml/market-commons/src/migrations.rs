use crate::{Config, MarketCounter, Pallet};
use frame_support::{
    dispatch::Weight,
    log,
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
};
use sp_runtime::traits::Saturating;

const REQUIRED_STORAGE_VERSION: u16 = 0;
const NEXT_STORAGE_VERSION: u16 = 1;

pub struct MigrateMarketCounter<T>(PhantomData<T>);

// Increment the `MarketCounter` by one _if_ it is present in storage.
impl<T: Config> OnRuntimeUpgrade for MigrateMarketCounter<T> {
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);
        if StorageVersion::get::<Pallet<T>>() != REQUIRED_STORAGE_VERSION {
            return total_weight;
        }
        log::info!("Starting storage migration of market counter");

        if let Ok(market_id) = <MarketCounter<T>>::try_get() {
            <MarketCounter<T>>::put(market_id.saturating_add(1u8.into()));
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        }
        total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

        StorageVersion::new(NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        log::info!("Completed storage migration of market counter");
        total_weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{ExtBuilder, Runtime};

    #[test]
    fn on_runtime_upgrade_increments_market_counter_if_it_exists() {
        ExtBuilder::default().build().execute_with(|| {
            <MarketCounter<Runtime>>::put::<<Runtime as Config>::MarketId>(33u8.into());
            MigrateMarketCounter::<Runtime>::on_runtime_upgrade();
            assert_eq!(<MarketCounter<Runtime>>::get(), 34);
        });
    }

    #[test]
    fn on_runtime_upgrade_leaves_market_counter_unchanged_if_it_does_not_exist() {
        ExtBuilder::default().build().execute_with(|| {
            MigrateMarketCounter::<Runtime>::on_runtime_upgrade();
            assert!(!<MarketCounter<Runtime>>::exists());
        });
    }

    #[test]
    fn on_runtime_increments_storage_version() {
        ExtBuilder::default().build().execute_with(|| {
            MigrateMarketCounter::<Runtime>::on_runtime_upgrade();
            assert_eq!(StorageVersion::get::<Pallet<Runtime>>(), NEXT_STORAGE_VERSION);
        });
    }
}
