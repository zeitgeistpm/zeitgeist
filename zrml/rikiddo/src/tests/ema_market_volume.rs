use super::max_allowed_error;
use crate::{
    traits::MarketAverage,
    types::{EmaMarketVolume, EmaVolumeConfig, MarketVolumeState, Timespan, TimestampedVolume},
};
use frame_support::assert_err;
use substrate_fixed::{types::extra::U64, FixedI128};

fn ema_create_test_struct(period: u32, smoothing: f64) -> EmaMarketVolume<FixedI128<U64>> {
    let emv_cfg = EmaVolumeConfig::<FixedI128<U64>> {
        ema_period: Timespan::Seconds(period),
        smoothing: <FixedI128<U64>>::from_num(smoothing),
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
    let mut emv = ema_create_test_struct(2, 2.0);
    assert_eq!(emv.state(), &MarketVolumeState::Uninitialized);
    let _ = emv.update(TimestampedVolume { timestamp: 0, volume: 1.into() }).unwrap();
    assert_eq!(emv.state(), &MarketVolumeState::DataCollectionStarted);
    let _ = emv.update(TimestampedVolume { timestamp: 3, volume: 1.into() }).unwrap();
    assert_eq!(emv.state(), &MarketVolumeState::DataCollected);
}

#[test]
fn ema_returns_none_before_final_state() {
    let mut emv = ema_create_test_struct(2, 2.0);
    assert_eq!(emv.get(), None);
    let _ = emv.update(TimestampedVolume { timestamp: 0, volume: 1.into() }).unwrap();
    assert_eq!(emv.get(), None);
    let _ = emv.update(TimestampedVolume { timestamp: 3, volume: 1.into() }).unwrap();
    assert_ne!(emv.get(), None);
}

#[test]
fn ema_returns_correct_ema() {
    let mut emv = ema_create_test_struct(2, 2.0);
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
    let mut emv = ema_create_test_struct(2, 2.0);
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
    let mut emv = ema_create_test_struct(2, 2.0);
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
    // TODO
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
    let mut emv = ema_create_test_struct(2, -1.0001);
    let _ = emv.update(TimestampedVolume { timestamp: 0, volume: 2.into() }).unwrap();
    let max_i64_fixed = <FixedI128<U64>>::from_num(u64::MAX >> 1);
    assert_err!(
        emv.update(TimestampedVolume { timestamp: 2, volume: max_i64_fixed }),
        "[EmaMarketVolume] Overflow during calculation: sma * volumes_per_period + volume"
    );
}
