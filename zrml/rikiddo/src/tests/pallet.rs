use frame_support::{
    assert_noop, assert_ok,
    traits::{OnFinalize, OnInitialize},
};
use frame_system::RawOrigin;
use zeitgeist_primitives::constants::BALANCE_FRACTIONAL_DECIMAL_PLACES;

use crate::{mock::*, traits::RikiddoSigmoidMVPallet, types::Timespan, Config};

fn run_to_block(n: u64) {
    while System::block_number() < n {
        Timestamp::on_finalize(System::block_number());
        Balances::on_finalize(System::block_number());
        Rikiddo::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Timestamp::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
        Rikiddo::on_initialize(System::block_number());
    }
}

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
        assert_noop!(Rikiddo::clear(0), crate::Error::<Runtime>::RikiddoNotFoundForPool);
        assert_ok!(Rikiddo::create(0, rikiddo));
        assert_ok!(Rikiddo::clear(0));
    });
}

#[test]
fn rikiddo_pallet_can_only_destroy_existing_rikiddo_instances() {
    ExtBuilder::default().build().execute_with(|| {
        let rikiddo = <Runtime as Config>::Rikiddo::default();
        assert_noop!(Rikiddo::destroy(0), crate::Error::<Runtime>::RikiddoNotFoundForPool);
        assert_ok!(Rikiddo::create(0, rikiddo));
        assert_ok!(Rikiddo::destroy(0));
        assert_noop!(Rikiddo::clear(0), crate::Error::<Runtime>::RikiddoNotFoundForPool);
    });
}

#[test]
fn rikiddo_pallet_update_market_data_returns_correct_result() {
    ExtBuilder::default().build().execute_with(|| {
        let mut rikiddo = <Runtime as Config>::Rikiddo::default();
        rikiddo.ma_short.config.ema_period = Timespan::Seconds(1);
        rikiddo.ma_long.config.ema_period = Timespan::Seconds(1);
        assert_noop!(
            Rikiddo::update(0, 10000000000),
            crate::Error::<Runtime>::RikiddoNotFoundForPool
        );
        let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 0).unwrap();
        assert_ok!(Rikiddo::create(0, rikiddo));
        assert_ok!(Rikiddo::update(0, 10000000000));
        run_to_block(1);
        let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 2).unwrap();
        assert_eq!(
            Rikiddo::update(0, 10000000000).unwrap(),
            Some(10u128.pow(BALANCE_FRACTIONAL_DECIMAL_PLACES as u32))
        );
    });
}

#[test]
fn rikiddo_pallet_cost_returns_correct_result() {
    ExtBuilder::default().build().execute_with(|| {
        let mut rikiddo = <Runtime as Config>::Rikiddo::default();
        rikiddo.ma_short.config.ema_period = Timespan::Seconds(1);
        rikiddo.ma_long.config.ema_period = Timespan::Seconds(1);
        // TODO
        /*
        assert_noop!(Rikiddo::update(0, 10000000000), crate::Error::<Runtime>::RikiddoNotFoundForPool);
        let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 0).unwrap();
        assert_ok!(Rikiddo::create(0, rikiddo));
        assert_ok!(Rikiddo::update(0, 10000000000));
        run_to_block(1);
        let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 2).unwrap();
        assert_eq!(Rikiddo::update(0, 10000000000).unwrap(), Some(10u128.pow(BALANCE_FRACTIONAL_DECIMAL_PLACES as u32)));
        */
    });
}
