use crate::{AuthorizedOutcomeReports, Config, Outcomes, Pallet};
use frame_support::{
    dispatch::Weight,
    log,
    pallet_prelude::PhantomData,
    traits::{Get, OnRuntimeUpgrade, StorageVersion},
};

const REQUIRED_STORAGE_VERSION: u16 = 0;
const NEXT_STORAGE_VERSION: u16 = 1;

pub struct MigrateAuthorizedStorage<T>(PhantomData<T>);

// Due to problems with `storage_key_iter`, we temporarily use two maps. The old `Outcomes` will
// no longer be used and removed at the next opportunity.
impl<T: Config> OnRuntimeUpgrade for MigrateAuthorizedStorage<T> {
    fn on_runtime_upgrade() -> Weight
    where
        T: Config,
    {
        if StorageVersion::get::<Pallet<T>>() != REQUIRED_STORAGE_VERSION {
            return 0;
        }
        let mut total_weight: Weight = 0;
        log::info!("Starting storage migration of authorized reports");

        for (market_id, _, outcome_report) in <Outcomes<T>>::drain() {
            <AuthorizedOutcomeReports<T>>::insert(market_id, outcome_report);
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
        }

        StorageVersion::new(NEXT_STORAGE_VERSION).put::<Pallet<T>>();
        total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

        log::info!("Completed storage migration of authorized reports");
        total_weight
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{ExtBuilder, Runtime};
    use zeitgeist_primitives::types::OutcomeReport;

    #[test]
    fn test_on_runtime_upgrade_generic_values() {
        ExtBuilder::default().build().execute_with(|| {
            let test_vector = vec![
                (12, 34, OutcomeReport::Categorical(56)),
                (32, 19, OutcomeReport::Scalar(87)),
                (1234, 5678, OutcomeReport::Categorical(9012)),
                (9876, 5432, OutcomeReport::Scalar(1098)),
            ];
            for (market_id, account, outcome_report) in test_vector.iter() {
                <Outcomes<Runtime>>::insert(market_id, account, outcome_report);
            }
            MigrateAuthorizedStorage::<Runtime>::on_runtime_upgrade();
            for (market_id, _, outcome_report) in test_vector.iter() {
                assert_eq!(
                    *outcome_report,
                    <AuthorizedOutcomeReports<Runtime>>::get(market_id).unwrap()
                );
            }
            // Ensure that `Outcomes` has been properly drained.
            assert!(<Outcomes<Runtime>>::iter().peekable().peek().is_none());
        });
    }
}
