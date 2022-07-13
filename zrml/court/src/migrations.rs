use crate::{Config, Jurors, Pallet};
use frame_support::{
    dispatch::Weight,
    log,
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
};

const COURT_REQUIRED_STORAGE_VERSION: u16 = 0;
const COURT_NEXT_STORAGE_VERSION: u16 = 1;

pub struct JurorsConuntedStorageMapMigration<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for JurorsConuntedStorageMapMigration<T> {
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);

        if StorageVersion::get::<Pallet<T>>() != COURT_REQUIRED_STORAGE_VERSION {
            log::info!("Skipping storage migration of jurors of court already up to date");
            return total_weight;
        }
        log::info!("Starting storage migration of jurors of court");
        Jurors::<T>::initialize_counter();
        total_weight =
            total_weight.saturating_add(T::DbWeight::get().writes(Jurors::<T>::count() as u64));

        StorageVersion::new(COURT_NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

        log::info!("Completed storage migration of jurors of court");
        total_weight
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        let court_storage_version = StorageVersion::get::<Pallet<T>>();
        assert_eq!(
            court_storage_version, COURT_NEXT_STORAGE_VERSION,
            "found unexpected court pallet storage version. Found: {:?}. Expected: {:?}",
            court_storage_version, COURT_NEXT_STORAGE_VERSION,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{mock::*, Juror};
    use frame_support::Hashable;

    #[test]
    fn test_on_runtime_upgrade_on_untouched_chain() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            JurorsConuntedStorageMapMigration::<Runtime>::on_runtime_upgrade();
        });
    }

    #[test]
    fn on_runtime_upgrade_updates_storage_versions() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            JurorsConuntedStorageMapMigration::<Runtime>::on_runtime_upgrade();
            assert_eq!(StorageVersion::get::<Pallet<Runtime>>(), COURT_NEXT_STORAGE_VERSION,);
        });
    }

    #[test]
    fn test_on_runtime_upgrade() {
        ExtBuilder::default().build().execute_with(|| {
            setup_chain();
            let alice_hash = ALICE.blake2_128_concat();
            let alice_juror = Juror { status: crate::JurorStatus::Ok };
            frame_support::migration::put_storage_value(
                b"Court",
                b"Jurors",
                &alice_hash,
                alice_juror,
            );
            let juror: Option<Juror> =
                frame_support::migration::get_storage_value(b"Court", b"Jurors", &alice_hash);
            assert!(juror.is_some());
            assert!(Jurors::<Runtime>::get(&ALICE).is_some());
            assert_eq!(Jurors::<Runtime>::count(), 0_u32);
            JurorsConuntedStorageMapMigration::<Runtime>::on_runtime_upgrade();
            assert_eq!(Jurors::<Runtime>::count(), 1_u32);
        });
    }

    fn setup_chain() {
        StorageVersion::new(COURT_REQUIRED_STORAGE_VERSION).put::<Pallet<Runtime>>();
    }
}
