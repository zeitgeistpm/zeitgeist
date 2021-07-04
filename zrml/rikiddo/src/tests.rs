#![cfg(test)]

use frame_support::assert_err;
use substrate_fixed::{types::extra::U64, FixedI128};

use crate::{
    mock::*,
    traits::{MarketAverage, RikiddoFee},
    types::{
        EmaMarketVolume, EmaVolumeConfig, FeeSigmoid, FeeSigmoidConfig, MarketVolumeState,
        Timespan, TimestampedVolume,
    },
};

fn max_allowed_error(fixed_point_bits: u8) -> f64 {
    1.0 / (1u128 << (fixed_point_bits - 1)) as f64
}

fn sigmoid_fee(m: f64, n: f64, p: f64, r: f64) -> f64 {
    (m * (r - n)) / (p + (r - n).powi(2)).sqrt()
}

fn init_default_sigmoid_fee_struct() -> (FeeSigmoid<FixedI128<U64>>, f64, f64, f64) {
    let m = 0.01f64;
    let n = 0f64;
    let p = 2.0f64;

    let config = FeeSigmoidConfig {
        m: <FixedI128<U64>>::from_num(m),
        n: <FixedI128<U64>>::from_num(n),
        p: <FixedI128<U64>>::from_num(p),
    };

    let fee = FeeSigmoid { config };
    (fee, m, n, p)
}

#[test]
fn fee_sigmoid_overflow_r_minus_n() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(i64::MIN);
    fee.config.n = <FixedI128<U64>>::from_num(i64::MAX);
    assert_err!(fee.calculate(r), "[FeeSigmoid] Overflow during calculation: r - n");
}

#[test]
fn fee_sigmoid_overflow_m_times_r_minus_n() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(i64::MIN);
    fee.config.n = <FixedI128<U64>>::from_num(0);
    fee.config.m = <FixedI128<U64>>::from_num(i64::MAX);
    assert_err!(fee.calculate(r), "[FeeSigmoid] Overflow during calculation: m * (r-n)");
}

#[test]
fn fee_sigmoid_overflow_r_minus_n_squared() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(i64::MIN);
    fee.config.n = <FixedI128<U64>>::from_num(0);
    assert_err!(fee.calculate(r), "[FeeSigmoid] Overflow during calculation: (r-n)^2");
}

#[test]
fn fee_sigmoid_overflow_p_plus_r_minus_n_squared() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(0);
    fee.config.n = <FixedI128<U64>>::from_num(1);
    fee.config.m = <FixedI128<U64>>::from_num(0);
    fee.config.p = <FixedI128<U64>>::from_num(i64::MAX);
    assert_err!(fee.calculate(r), "[FeeSigmoid] Overflow during calculation: p + (r-n)^2");
}

#[test]
fn fee_sigmoid_overflow_numerator_div_denominator() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(0.1);
    fee.config.n = <FixedI128<U64>>::from_num(0);
    fee.config.m = <FixedI128<U64>>::from_num(i64::MAX);
    fee.config.p = <FixedI128<U64>>::from_num(-0.0099);
    assert_err!(
        fee.calculate(r),
        "[FeeSigmoid] Overflow during calculation: numerator / denominator"
    );
}

#[test]
fn fee_sigmoid_correct_result() -> Result<(), &'static str> {
    let r = 1.5f64;
    let (fee, m, n, p) = init_default_sigmoid_fee_struct();
    let fee_f64 = sigmoid_fee(m, n, p, r);
    let fee_fixed = fee.calculate(<FixedI128<U64>>::from_num(r))?;
    let fee_fixed_f64: f64 = fee_fixed.to_num();
    let difference_abs = (fee_f64 - fee_fixed_f64).abs();

    assert!(
        difference_abs <= max_allowed_error(64),
        "\nFixed result: {}\nFloat result: {}\nDifference: {}\nMax_Allowed_Difference: {}",
        fee_f64,
        fee_fixed_f64,
        difference_abs,
        max_allowed_error(64)
    );

    Ok(())
}

fn ema_create_test_struct() -> EmaMarketVolume<FixedI128<U64>> {
    let emv_cfg = EmaVolumeConfig::<FixedI128<U64>> {
        ema_period: Timespan::Seconds(2),
        smoothing: <FixedI128<U64>>::from_num(2),
    };

    <EmaMarketVolume<FixedI128<U64>>>::new(emv_cfg)
}

fn ema_get_multiplier(volumes_per_period: u64, smoothing: f64) -> f64 {
    smoothing / (1 + volumes_per_period) as f64
}

fn ema_calculate(old_ema: f64, multiplier: f64, volume: f64) -> f64 {
    volume * multiplier + old_ema * (1.0 - multiplier)
}

#[test]
fn ema_state_transitions_work() {
    let mut emv = ema_create_test_struct();
    assert_eq!(emv.state(), &MarketVolumeState::Uninitialized);
    let _ = emv.update(TimestampedVolume { timestamp: 0, volume: 1.into() }).unwrap();
    assert_eq!(emv.state(), &MarketVolumeState::DataCollectionStarted);
    let _ = emv.update(TimestampedVolume { timestamp: 3, volume: 1.into() }).unwrap();
    assert_eq!(emv.state(), &MarketVolumeState::DataCollected);
}

#[test]
fn ema_returns_none_before_final_state() {
    let mut emv = ema_create_test_struct();
    assert_eq!(emv.get(), None);
    let _ = emv.update(TimestampedVolume { timestamp: 0, volume: 1.into() }).unwrap();
    assert_eq!(emv.get(), None);
    let _ = emv.update(TimestampedVolume { timestamp: 3, volume: 1.into() }).unwrap();
    assert_ne!(emv.get(), None);
}

#[test]
fn ema_returns_correct_ema() {
    let mut emv = ema_create_test_struct();
    let _ = emv.update(TimestampedVolume { timestamp: 0, volume: 2.into() }).unwrap();
    let _ = emv.update(TimestampedVolume { timestamp: 1, volume: 6.into() }).unwrap();
    let _ = emv.update(TimestampedVolume { timestamp: 2, volume: 4.into() }).unwrap();
    // Currently it's a sma
    let ema = emv.ema.to_num::<f64>();
    assert_eq!(ema, (2.0 + 6.0 + 4.0) / 3.0);

    let _ = emv.update(TimestampedVolume { timestamp: 3, volume: 20.into() }).unwrap();
    // Now it's an ema
    let ema_fixed_f64: f64 = emv.ema.to_num();
    let multiplier = ema_get_multiplier(3, emv.config.smoothing.to_num());
    let ema_f64 = ema_calculate(ema, multiplier, 20f64);
    let difference_abs = (ema_fixed_f64 - ema_f64).abs();
    assert!(
        difference_abs <= max_allowed_error(64),
        "\nFixed result: {}\nFloat result: {}\nDifference: {}\nMax_Allowed_Difference: {}",
        ema_fixed_f64,
        ema_f64,
        difference_abs,
        max_allowed_error(64)
    );

    // Repeat check using the get() function
    let ema_fixed_f64: f64 = emv.get().unwrap().to_num();
    let difference_abs = (ema_fixed_f64 - ema_f64).abs();
    assert!(
        difference_abs <= max_allowed_error(64),
        "\nFixed result: {}\nFloat result: {}\nDifference: {}\nMax_Allowed_Difference: {}",
        ema_fixed_f64,
        ema_f64,
        difference_abs,
        max_allowed_error(64)
    );
}

#[test]
fn ema_clear_ereases_data() {
    let mut emv = ema_create_test_struct();
    let _ = emv.update(TimestampedVolume { timestamp: 0, volume: 2.into() }).unwrap();
    let _ = emv.update(TimestampedVolume { timestamp: 3, volume: 6.into() }).unwrap();
    emv.clear();
    assert_eq!(emv.ema, <FixedI128<U64>>::from_num(0));
    assert_eq!(emv.multiplier(), &<FixedI128<U64>>::from_num(0));
    assert_eq!(emv.state(), &MarketVolumeState::Uninitialized);
    assert_eq!(emv.start_time(), &0);
    assert_eq!(emv.last_time(), &0);
    assert_eq!(emv.volumes_per_period(), &0);
}

#[test]
fn ema_added_volume_is_older_than_previous() {
    let mut emv = ema_create_test_struct();
    let _ = emv.update(TimestampedVolume { timestamp: 2, volume: 2.into() }).unwrap();
    assert_err!(
        emv.update(TimestampedVolume { timestamp: 1, volume: 2.into() }),
        "[EmaMarketVolume] Incoming volume timestamp is older than previous timestamp"
    );
}

#[test]
fn ema_overflow_sma_times_vpp() {
    let emv_cfg = EmaVolumeConfig::<FixedI128<U64>> {
        ema_period: Timespan::Seconds(3),
        smoothing: <FixedI128<U64>>::from_num(2),
    };

    let mut emv = <EmaMarketVolume<FixedI128<U64>>>::new(emv_cfg);
    let _ = emv.update(TimestampedVolume { timestamp: 0, volume: 2.into() }).unwrap();
    let _ = emv.update(TimestampedVolume { timestamp: 1, volume: 6.into() }).unwrap();
    emv.ema = <FixedI128<U64>>::from_num(i64::MAX);
    assert_err!(
        emv.update(TimestampedVolume { timestamp: 3, volume: 6.into() }),
        "[EmaMarketVolume] Overflow during calculation: sma * volumes_per_period"
    );
}

#[test]
fn ema_overflow_sma_times_vpp_plus_volume() {
    let mut emv = ema_create_test_struct();
    let _ = emv.update(TimestampedVolume { timestamp: 0, volume: 2.into() }).unwrap();
    let max_i64_fixed = <FixedI128<U64>>::from_num(u64::MAX >> 1);
    assert_err!(
        emv.update(TimestampedVolume { timestamp: 2, volume: max_i64_fixed }),
        "[EmaMarketVolume] Overflow during calculation: sma * volumes_per_period + volume"
    );
}

fn ema_overflow_volume_times_multiplier() {
    // TODO
}

fn ema_overflow_ema_times_one_minus_multiplier() {
    // TODO
}

fn ema_overflow_ema_times_current_plus_previous() {
    // TODO
}

#[test]
fn it_is_a_dummy_test() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(true);
    });
}
