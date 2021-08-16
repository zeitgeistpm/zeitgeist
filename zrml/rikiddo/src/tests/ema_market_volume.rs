use super::max_allowed_error;
use crate::{
    traits::MarketAverage,
    types::{EmaConfig, EmaMarketVolume, MarketVolumeState, Timespan, TimestampedVolume},
};
use frame_support::assert_err;
use substrate_fixed::{
    types::extra::{U64, U96},
    FixedU128,
};

pub(super) fn ema_create_test_struct(
    period: u32,
    smoothing: f64,
) -> EmaMarketVolume<FixedU128<U64>> {
    let emv_cfg = EmaConfig::<FixedU128<U64>> {
        ema_period: Timespan::Seconds(period),
        ema_period_estimate_after: None,
        smoothing: <FixedU128<U64>>::from_num(smoothing),
    };

    <EmaMarketVolume<FixedU128<U64>>>::new(emv_cfg)
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
    let _ = emv.update(&TimestampedVolume { timestamp: 0, volume: 1u32.into() }).unwrap();
    assert_eq!(emv.state(), &MarketVolumeState::DataCollectionStarted);
    let _ = emv.update(&TimestampedVolume { timestamp: 3, volume: 1u32.into() }).unwrap();
    assert_eq!(emv.state(), &MarketVolumeState::DataCollected);
}

#[test]
fn ema_returns_none_before_final_state() {
    let mut emv = ema_create_test_struct(2, 2.0);
    assert_eq!(emv.get(), None);
    let _ = emv.update(&TimestampedVolume { timestamp: 0, volume: 1u32.into() }).unwrap();
    assert_eq!(emv.get(), None);
    let _ = emv.update(&TimestampedVolume { timestamp: 3, volume: 1u32.into() }).unwrap();
    assert_ne!(emv.get(), None);
}

#[test]
fn ema_returns_correct_ema() {
    let mut emv = ema_create_test_struct(2, 2.0);
    let _ = emv.update(&TimestampedVolume { timestamp: 0, volume: 2u32.into() }).unwrap();
    let _ = emv.update(&TimestampedVolume { timestamp: 1, volume: 6u32.into() }).unwrap();
    let _ = emv.update(&TimestampedVolume { timestamp: 2, volume: 4u32.into() }).unwrap();
    // Currently it's a sma
    let ema = emv.ema.to_num::<f64>();
    assert_eq!(ema, (2.0 + 6.0 + 4.0) / 3.0);

    let _ = emv.update(&TimestampedVolume { timestamp: 3, volume: 20u32.into() }).unwrap();
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
fn ema_returns_correct_ema_after_estimated_period() {
    let mut emv = EmaMarketVolume::new(EmaConfig::<FixedU128<U64>> {
        ema_period: Timespan::Seconds(8),
        ema_period_estimate_after: Some(Timespan::Seconds(2)),
        smoothing: <FixedU128<U64>>::from_num(2.0),
    });
    let _ = emv.update(&TimestampedVolume { timestamp: 0, volume: 2u32.into() }).unwrap();
    let _ = emv.update(&TimestampedVolume { timestamp: 1, volume: 6u32.into() }).unwrap();
    let _ = emv.update(&TimestampedVolume { timestamp: 2, volume: 4u32.into() }).unwrap();
    // Currently it's a sma
    let ema = emv.ema.to_num::<f64>();
    assert_eq!(ema, (2.0 + 6.0 + 4.0) / 3.0);

    let _ = emv.update(&TimestampedVolume { timestamp: 3, volume: 20u32.into() }).unwrap();
    // Now it's an ema (using estimated transaction count per period)
    let ema_fixed_f64: f64 = emv.ema.to_num();
    let extrapolation_factor = emv.config.ema_period.to_seconds() as f64
        / emv.config.ema_period_estimate_after.unwrap().to_seconds() as f64;
    let multiplier = ema_get_multiplier(
        (3f64 * extrapolation_factor).ceil() as u64,
        emv.config.smoothing.to_num(),
    );
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
}

#[test]
fn ema_clear_ereases_data() {
    let mut emv = ema_create_test_struct(2, 2.0);
    let _ = emv.update(&TimestampedVolume { timestamp: 0, volume: 2u32.into() }).unwrap();
    let _ = emv.update(&TimestampedVolume { timestamp: 3, volume: 6u32.into() }).unwrap();
    emv.clear();
    assert_eq!(emv.ema, <FixedU128<U64>>::from_num(0));
    assert_eq!(emv.multiplier(), &<FixedU128<U64>>::from_num(0));
    assert_eq!(emv.state(), &MarketVolumeState::Uninitialized);
    assert_eq!(emv.start_time(), &0);
    assert_eq!(emv.last_time(), &0);
    assert_eq!(emv.volumes_per_period(), &0);
}

#[test]
fn ema_added_volume_is_older_than_previous() {
    let mut emv = ema_create_test_struct(2, 2.0);
    let _ = emv.update(&TimestampedVolume { timestamp: 2, volume: 2u32.into() }).unwrap();
    assert_err!(
        emv.update(&TimestampedVolume { timestamp: 1, volume: 2u32.into() }),
        "[EmaMarketVolume] Incoming volume timestamp is older than previous timestamp"
    );
}

#[test]
fn ema_overflow_sma_times_vpp() {
    let mut emv = ema_create_test_struct(3, 2.0);
    let _ = emv.update(&TimestampedVolume { timestamp: 0, volume: 2u32.into() }).unwrap();
    let _ = emv.update(&TimestampedVolume { timestamp: 1, volume: 6u32.into() }).unwrap();
    emv.ema = <FixedU128<U64>>::from_num(u64::MAX);
    assert_err!(
        emv.update(&TimestampedVolume { timestamp: 3, volume: 6u32.into() }),
        "[EmaMarketVolume] Overflow during calculation: sma * volumes_per_period"
    );
}

#[test]
fn ema_overflow_sma_times_vpp_plus_volume() {
    let mut emv = ema_create_test_struct(2, 2.0);
    let _ = emv.update(&TimestampedVolume { timestamp: 0, volume: 2u32.into() }).unwrap();
    let max_u64_fixed = <FixedU128<U64>>::from_num(u64::MAX);
    assert_err!(
        emv.update(&TimestampedVolume { timestamp: 2, volume: max_u64_fixed }),
        "[EmaMarketVolume] Overflow during calculation: sma * volumes_per_period + volume"
    );
}

#[test]
fn ema_overflow_estimated_tx_per_period_does_not_fit() {
    let mut emv = EmaMarketVolume::new(EmaConfig::<FixedU128<U96>> {
        ema_period: Timespan::Hours(2_386_093),
        ema_period_estimate_after: Some(Timespan::Seconds(0)),
        smoothing: <FixedU128<U96>>::from_num(2.0),
    });
    let _ = emv.update(&TimestampedVolume { timestamp: 0, volume: 2u32.into() }).unwrap();
    let _ = emv.update(&TimestampedVolume { timestamp: 0, volume: 2u32.into() }).unwrap();
    assert_err!(
        emv.update(&TimestampedVolume { timestamp: 1, volume: 6u32.into() }),
        "[EmaMarketVolume] Overflow during estimation of transactions per period"
    );
}
