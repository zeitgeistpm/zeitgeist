use sp_std::convert::TryInto;

use frame_support::{
    assert_noop, assert_ok,
    traits::{OnFinalize, OnInitialize},
};
use frame_system::RawOrigin;
use zeitgeist_primitives::constants::BALANCE_FRACTIONAL_DECIMAL_PLACES;

use crate::{
    mock::*,
    tests::rikiddo_sigmoid_mv::cost,
    traits::RikiddoSigmoidMVPallet,
    types::{FromFixedDecimal, IntoFixedDecimal, Timespan},
    Config,
};

#[inline]
fn max_balance_difference(frac_dec_places: u8, max_percent_places_wrong: f64) -> u128 {
    10u128.pow((frac_dec_places as f64 * max_percent_places_wrong).ceil() as u32)
}

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
fn rikiddo_pallet_fee_return_correct_result() {
    ExtBuilder::default().build().execute_with(|| {
        // First we check that the returned initial fee is correct
        let mut rikiddo = <Runtime as Config>::Rikiddo::default();
        type FixedS = <Runtime as Config>::FixedTypeS;
        type Balance = <Runtime as Config>::Balance;
        let frac_dec_places = <Runtime as Config>::BalanceFractionalDecimals::get();
        let initial_fee: f64 = rikiddo.config.initial_fee.to_num();
        rikiddo.ma_short.config.ema_period = Timespan::Seconds(1);
        rikiddo.ma_long.config.ema_period = Timespan::Seconds(1);
        assert_noop!(Rikiddo::fee(0), crate::Error::<Runtime>::RikiddoNotFoundForPool);
        let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 0).unwrap();
        assert_ok!(Rikiddo::create(0, rikiddo));
        let fee_reference_balance: Balance =
            FixedS::from_num(initial_fee).to_fixed_decimal(frac_dec_places).unwrap();
        let fee_pallet_balance = Rikiddo::fee(0).unwrap();
        let difference_abs =
            (fee_pallet_balance as i128 - fee_reference_balance as i128).abs() as u128;
        let max_difference = max_balance_difference(frac_dec_places, 0.3);
        assert!(
            difference_abs <= max_difference,
            "\nReference fee result (Balance): {}\nRikiddo pallet fee result (Balance): \
             {}\nDifference: {}\nMax_Allowed_Difference: {}",
            fee_reference_balance,
            fee_pallet_balance,
            difference_abs,
            max_difference,
        );

        // Now we check if the fee has changed, since enough volume data was collected
        assert_ok!(Rikiddo::update(0, 10000000000));
        run_to_block(1);
        let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 2).unwrap();
        assert_ok!(Rikiddo::update(0, 10000000000));
        assert_ne!(Rikiddo::fee(0).unwrap(), fee_pallet_balance);
    });
}

#[test]
fn rikiddo_pallet_cost_returns_correct_result() {
    ExtBuilder::default().build().execute_with(|| {
        // The first part compares the result from the f64 reference cost function with
        // what the pallet returns. It uses the initial fee.
        let mut rikiddo = <Runtime as Config>::Rikiddo::default();
        type FixedS = <Runtime as Config>::FixedTypeS;
        type Balance = <Runtime as Config>::Balance;
        let frac_dec_places = <Runtime as Config>::BalanceFractionalDecimals::get();
        let initial_fee: f64 = rikiddo.config.initial_fee.to_num();
        rikiddo.ma_short.config.ema_period = Timespan::Seconds(1);
        rikiddo.ma_long.config.ema_period = Timespan::Seconds(1);
        assert_ok!(Rikiddo::create(0, rikiddo));
        let asset_balance: <Runtime as Config>::Balance =
            (500u128 * 10u128.pow(frac_dec_places as u32)).try_into().unwrap();
        let cost_pallet_balance = Rikiddo::cost(0, &[asset_balance, asset_balance]).unwrap();
        let cost_reference = cost(initial_fee, &vec![500.0f64, 500.0f64]);
        let cost_reference_balance: Balance =
            FixedS::from_num(cost_reference).to_fixed_decimal(frac_dec_places).unwrap();
        let difference_abs =
            (cost_pallet_balance as i128 - cost_reference_balance as i128).abs() as u128;
        let max_difference = max_balance_difference(frac_dec_places, 0.3);
        assert!(
            difference_abs <= max_difference,
            "\nReference cost result (Balance): {}\nRikiddo pallet cost result (Balance): \
             {}\nDifference: {}\nMax_Allowed_Difference: {}",
            cost_reference_balance,
            cost_pallet_balance,
            difference_abs,
            max_difference,
        );

        // The second part also compares the cost results, but uses the sigmoid fee.
        let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 0).unwrap();
        assert_ok!(Rikiddo::update(0, 10000000000));
        run_to_block(1);
        let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 2).unwrap();
        assert_eq!(
            Rikiddo::update(0, 10000000000).unwrap(),
            Some(10u128.pow(BALANCE_FRACTIONAL_DECIMAL_PLACES as u32))
        );
        let cost_pallet_balance_with_fee =
            Rikiddo::cost(0, &[asset_balance, asset_balance]).unwrap();
        let fee = Rikiddo::fee(0).unwrap();
        let fee_f64: f64 = FixedS::from_fixed_decimal(fee, frac_dec_places).unwrap().to_num();
        let cost_reference_with_fee = cost(fee_f64, &vec![500.0f64, 500.0f64]);
        let cost_reference_balance_with_fee: Balance =
            FixedS::from_num(cost_reference_with_fee).to_fixed_decimal(frac_dec_places).unwrap();
        let difference_abs_with_fee = (cost_pallet_balance_with_fee as i128
            - cost_reference_balance_with_fee as i128)
            .abs() as u128;
        let max_difference_with_fee = max_balance_difference(frac_dec_places, 0.3);
        assert!(
            difference_abs_with_fee <= max_difference_with_fee,
            "\nReference cost result (Balance): {}\nRikiddo pallet cost result (Balance): \
             {}\nDifference: {}\nMax_Allowed_Difference: {}",
            cost_reference_balance_with_fee,
            cost_pallet_balance_with_fee,
            difference_abs_with_fee,
            max_difference_with_fee,
        );
    });
}
