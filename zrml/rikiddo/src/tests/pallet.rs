use core::convert::TryInto;
use frame_support::{
    assert_noop, assert_ok,
    traits::{OnFinalize, OnInitialize},
};
use frame_system::RawOrigin;

type FixedS = <Runtime as Config>::FixedTypeS;
type Balance = <Runtime as Config>::Balance;
use crate::{Config, mock::*, tests::rikiddo_sigmoid_mv::{cost, initial_outstanding_assets, price}, traits::{Fee, FromFixedDecimal, IntoFixedDecimal, RikiddoMVPallet}, types::Timespan};

#[inline]
// Returns the maximum balance difference. If `frac_dec_places` is 10, and
// `max_percent_places_wrong` is 0.3, then the result is `10^3` = 1000
fn max_balance_difference(frac_dec_places: u8, max_percent_places_wrong: f64) -> u128 {
    10u128.pow((frac_dec_places as f64 * max_percent_places_wrong).ceil() as u32)
}

fn default_prepare_calculation() -> (u8, f64, Vec<f64>, Vec<<Runtime as Config>::Balance>) {
    let mut rikiddo = <Runtime as Config>::Rikiddo::default();
    let frac_dec_places = <Runtime as Config>::BalanceFractionalDecimals::get();
    let initial_fee: f64 = rikiddo.config.initial_fee.to_num();
    rikiddo.ma_short.config.ema_period = Timespan::Seconds(1);
    rikiddo.ma_long.config.ema_period = Timespan::Seconds(1);
    let asset_balances_f64 = vec![490f64, 510f64];
    let asset_balances: Vec<<Runtime as Config>::Balance> = vec![
        (asset_balances_f64[0] as u128 * 10u128.pow(frac_dec_places as u32)).try_into().unwrap(),
        (asset_balances_f64[1] as u128 * 10u128.pow(frac_dec_places as u32)).try_into().unwrap(),
    ];
    assert_ok!(Rikiddo::create(0, rikiddo));
    (frac_dec_places, initial_fee, asset_balances_f64, asset_balances)
}

// Adds volume for Timestamp 0 and 2 and returns the new sigmoid fee
fn default_fill_market_volume() -> f64 {
    let frac_dec_places = <Runtime as Config>::BalanceFractionalDecimals::get();
    let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 0).unwrap();
    assert_ok!(Rikiddo::update_volume(0, 1000));
    run_to_block(1);
    let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 2).unwrap();
    assert_eq!(Rikiddo::update_volume(0, 1000).unwrap(), Some(10u128.pow(frac_dec_places as u32)));

    let fee = Rikiddo::fee(0).unwrap();
    FixedS::from_fixed_decimal(fee, frac_dec_places).unwrap().to_num()
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
            Rikiddo::update_volume(0, 1000),
            crate::Error::<Runtime>::RikiddoNotFoundForPool
        );
        let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 0).unwrap();
        assert_ok!(Rikiddo::create(0, rikiddo));
        assert_ok!(Rikiddo::update_volume(0, 1000));
        run_to_block(1);
        let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 2).unwrap();
        assert_eq!(
            Rikiddo::update_volume(0, 1000).unwrap(),
            Some(10u128.pow(<Runtime as Config>::BalanceFractionalDecimals::get() as u32))
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
        assert_ok!(Rikiddo::update_volume(0, 1000));
        run_to_block(1);
        let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 2).unwrap();
        assert_ok!(Rikiddo::update_volume(0, 1000));
        assert_ne!(Rikiddo::fee(0).unwrap(), fee_pallet_balance);
    });
}

#[test]
fn rikiddo_pallet_cost_returns_correct_result() {
    ExtBuilder::default().build().execute_with(|| {
        // The first part compares the result from the f64 reference cost function with
        // what the pallet returns. It uses the initial fee.
        let (frac_dec_places, initial_fee, asset_balances_f64, asset_balances) =
            default_prepare_calculation();

        let cost_pallet_balance = Rikiddo::cost(0, &asset_balances).unwrap();
        let cost_reference = cost(initial_fee, &asset_balances_f64);
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
        let fee = default_fill_market_volume();
        let cost_pallet_balance_with_fee = Rikiddo::cost(0, &asset_balances).unwrap();
        let cost_reference_with_fee = cost(fee, &asset_balances_f64);
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

#[test]
fn rikiddo_pallet_initial_outstanding_assets_returns_correct_result() {
    ExtBuilder::default().build().execute_with(|| {
        let _ = default_prepare_calculation();
        let frac_places = Balance::from(<Runtime as Config>::BalanceFractionalDecimals::get());
        let num_assets = 4u32;
        let subsidy_f64 = 1000f64;
        let subsidy = subsidy_f64 as u128 * frac_places.pow(10);
        let fee: f64 = Rikiddo::get_rikiddo(&0).unwrap().fees.minimum_fee().to_num();
        let outstanding_assets =
            Rikiddo::initial_outstanding_assets(0, num_assets, subsidy).unwrap();
        let outstanding_assets_shifted =
            (outstanding_assets as f64) / (10f64.powf(frac_places as f64));
        let outstanding_assets_f64 = initial_outstanding_assets(num_assets, subsidy_f64, fee);
        let difference_abs = (outstanding_assets_f64 - outstanding_assets_shifted as f64).abs();
        assert!(
            difference_abs <= 0.000001f64,
            "\nFixed result: {}\nFloat result: {}\nDifference: {}\nMax_Allowed_Difference: {}",
            outstanding_assets_shifted,
            outstanding_assets_f64,
            difference_abs,
            0.000001f64
        );
    });
}

#[test]
fn rikiddo_pallet_price_returns_correct_result() {
    ExtBuilder::default().build().execute_with(|| {
        // The first part compares the result from the f64 reference price function with
        // what the pallet returns. It uses the initial fee.
        let (frac_dec_places, initial_fee, asset_balances_f64, asset_balances) =
            default_prepare_calculation();

        let price_pallet_balance = Rikiddo::price(0, asset_balances[0], &asset_balances).unwrap();
        let price_reference = price(initial_fee, &asset_balances_f64, asset_balances_f64[0]);
        let price_reference_balance: Balance =
            FixedS::from_num(price_reference).to_fixed_decimal(frac_dec_places).unwrap();
        let difference_abs =
            (price_reference_balance as i128 - price_pallet_balance as i128).abs() as u128;
        let max_difference = max_balance_difference(frac_dec_places, 0.3);
        assert!(
            difference_abs <= max_difference,
            "\nReference price result (Balance): {}\nRikiddo pallet price result (Balance): \
             {}\nDifference: {}\nMax_Allowed_Difference: {}",
            price_reference_balance,
            price_pallet_balance,
            difference_abs,
            max_difference,
        );

        // The second part also compares the price results, but uses the sigmoid fee.
        let fee = default_fill_market_volume();
        let price_pallet_balance_fee =
            Rikiddo::price(0, asset_balances[0], &asset_balances).unwrap();
        let price_reference_fee = price(fee, &asset_balances_f64, asset_balances_f64[0]);
        let price_reference_balance_fee: Balance =
            FixedS::from_num(price_reference_fee).to_fixed_decimal(frac_dec_places).unwrap();
        let difference_abs_fee =
            (price_reference_balance_fee as i128 - price_pallet_balance_fee as i128).abs() as u128;
        let max_difference_fee = max_balance_difference(frac_dec_places, 0.3);
        assert!(
            difference_abs_fee <= max_difference_fee,
            "\nReference price result (Balance): {}\nRikiddo pallet price result (Balance): \
             {}\nDifference: {}\nMax_Allowed_Difference: {}",
            price_reference_balance_fee,
            price_pallet_balance_fee,
            difference_abs_fee,
            max_difference_fee,
        );
    });
}

#[test]
fn rikiddo_pallet_all_prices_returns_correct_result() {
    ExtBuilder::default().build().execute_with(|| {
        // The first part compares the result from the f64 reference price function used
        // on every asset, with what the pallet returns. It uses the initial fee.
        let (frac_dec_places, initial_fee, asset_balances_f64, asset_balances) =
            default_prepare_calculation();

        let all_prices_pallet_balance = Rikiddo::all_prices(0, &asset_balances).unwrap();
        let all_prices_reference_balance: Vec<Balance> = asset_balances_f64
            .iter()
            .map(|e| {
                let price_reference = price(initial_fee, &asset_balances_f64, *e);
                FixedS::from_num(price_reference).to_fixed_decimal(frac_dec_places).unwrap()
            })
            .collect();
        let difference_abs = all_prices_reference_balance
            .iter()
            .zip(all_prices_pallet_balance.iter())
            .fold(0u128, |acc, elems| acc + (*elems.0 as i128 - *elems.1 as i128).abs() as u128);

        let max_difference =
            asset_balances.len() as u128 * max_balance_difference(frac_dec_places, 0.3);
        assert!(
            difference_abs <= max_difference,
            "\nReference all_prices result (Balance): {:?}\nRikiddo all_prices result (Balance): \
             {:?}\nDifference: {}\nMax_Allowed_Difference: {}",
            all_prices_reference_balance,
            all_prices_pallet_balance,
            difference_abs,
            max_difference,
        );

        // The second part also compares the price results for all prices, but uses
        // the sigmoid fee.
        let fee = default_fill_market_volume();

        let all_prices_pallet_balance_fee = Rikiddo::all_prices(0, &asset_balances).unwrap();
        let all_prices_reference_balance_fee: Vec<Balance> = asset_balances_f64
            .iter()
            .map(|e| {
                let price_reference = price(fee, &asset_balances_f64, *e);
                FixedS::from_num(price_reference).to_fixed_decimal(frac_dec_places).unwrap()
            })
            .collect();
        let difference_abs_fee = all_prices_reference_balance_fee
            .iter()
            .zip(all_prices_pallet_balance_fee.iter())
            .fold(0u128, |acc, elems| acc + (*elems.0 as i128 - *elems.1 as i128).abs() as u128);

        let max_difference_fee =
            asset_balances.len() as u128 * max_balance_difference(frac_dec_places, 0.3);
        assert!(
            difference_abs_fee <= max_difference_fee,
            "\nReference all_prices result (Balance): {:?}\nRikiddo pallet all_prices result \
             (Balance): {:?}\nDifference: {}\nMax_Allowed_Difference: {}",
            all_prices_reference_balance_fee,
            all_prices_pallet_balance_fee,
            difference_abs_fee,
            max_difference_fee,
        );
    });
}
