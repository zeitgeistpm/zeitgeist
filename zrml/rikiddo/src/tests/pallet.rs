use frame_support::{assert_noop, assert_ok};

use crate::{mock::*, traits::RikiddoSigmoidMVPallet, Config};

#[test]
fn rikiddo_pallet_can_create_one_instance_per_pool() {
    ExtBuilder::default().build().execute_with(|| {
        let rikiddo = <Runtime as Config>::Rikiddo::default();
        assert_ok!(Rikiddo::create(0, rikiddo.clone()));
        assert_noop!(
            Rikiddo::create(0, rikiddo.clone()),
            crate::Error::<Runtime>::RikiddoAlreadyExistsForPool
        );
        assert_ok!(Rikiddo::create(1, rikiddo));
    });
}

#[test]
fn rikiddo_pallet_can_only_clear_existing_rikiddo_instances() {
    ExtBuilder::default().build().execute_with(|| {
        let rikiddo = <Runtime as Config>::Rikiddo::default();
        assert_noop!(
            Rikiddo::clear(0),
            crate::Error::<Runtime>::RikiddoNotFoundForPool
        );
        assert_ok!(Rikiddo::create(0, rikiddo));
        assert_ok!(Rikiddo::clear(0));
    });
}

#[test]
fn rikiddo_pallet_can_only_destroy_existing_rikiddo_instances() {
    ExtBuilder::default().build().execute_with(|| {
        let rikiddo = <Runtime as Config>::Rikiddo::default();
        assert_noop!(
            Rikiddo::destroy(0),
            crate::Error::<Runtime>::RikiddoNotFoundForPool
        );
        assert_ok!(Rikiddo::create(0, rikiddo));
        assert_ok!(Rikiddo::destroy(0));
        assert_noop!(
            Rikiddo::clear(0),
            crate::Error::<Runtime>::RikiddoNotFoundForPool
        );
    });
}
