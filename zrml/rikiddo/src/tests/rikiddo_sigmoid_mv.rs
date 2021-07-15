use frame_support::assert_err;
use substrate_fixed::{types::extra::U64, FixedI128, FixedU128};

use super::ema_market_volume::ema_create_test_struct;
use crate::{
    traits::{MarketAverage, RikiddoMV},
    types::{EmaMarketVolume, FeeSigmoid, RikiddoConfig, RikiddoSigmoidMV, TimestampedVolume},
};

type Rikiddo = RikiddoSigmoidMV<
    FixedU128<U64>,
    FixedI128<U64>,
    FeeSigmoid<FixedI128<U64>, FixedU128<U64>>,
    EmaMarketVolume<FixedU128<U64>>,
>;

#[test]
fn rikiddo_updates_mv_and_returns_some() {
    let emv = ema_create_test_struct(1, 2.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv.clone(), emv);
    rikiddo.update(&TimestampedVolume { timestamp: 0, volume: 1u32.into() }).unwrap();
    let res = rikiddo.update(&TimestampedVolume { timestamp: 2, volume: 2u32.into() }).unwrap();
    assert_eq!(res, Some(1u32.into()));
}

#[test]
fn rikiddo_updates_mv_and_returns_none() {
    let mut rikiddo = Rikiddo::default();
    let vol = TimestampedVolume::default();
    assert_eq!(rikiddo.update(&vol).unwrap(), None);
}

#[test]
fn rikiddo_clear_clears_market_data() {
    let mut rikiddo = Rikiddo::default();
    let rikiddo_clone = rikiddo.clone();
    let _ = rikiddo.update(&<TimestampedVolume<FixedU128<U64>>>::default());
    assert_ne!(rikiddo, rikiddo_clone);
    rikiddo.clear();
    assert_eq!(rikiddo, rikiddo_clone);
}

#[test]
fn rikiddo_get_fee_catches_zero_divison() {
    let emv = ema_create_test_struct(1, 0.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv.clone(), emv);
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 0, volume: 0u32.into() });
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 2, volume: 0u32.into() });
    assert_err!(
        rikiddo.get_fee(),
        "[RikiddoSigmoidMV] Zero division error during calculation: ma_short / ma_long"
    );
}

#[test]
fn rikiddo_get_fee_overflows_during_ratio_calculation() {
    let emv = ema_create_test_struct(1, 2.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv.clone(), emv);
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 0, volume: 0u32.into() });
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 2, volume: 0u32.into() });
    rikiddo.ma_short.ema = <FixedU128<U64>>::from_num(u64::MAX);
    rikiddo.ma_long.ema = <FixedU128<U64>>::from_num(0.1f64);
    assert_err!(
        rikiddo.get_fee(),
        "[RikiddoSigmoidMV] Overflow during calculation: ma_short / ma_long"
    );
}

#[test]
fn rikiddo_get_fee_ratio_does_not_fit_in_type() {
    let emv = ema_create_test_struct(1, 2.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv.clone(), emv);
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 0, volume: 0u32.into() });
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 2, volume: 0u32.into() });
    rikiddo.ma_short.ema = <FixedU128<U64>>::from_num(u64::MAX);
    rikiddo.ma_long.ema = <FixedU128<U64>>::from_num(1u64);
    assert_err!(
        rikiddo.get_fee(),
        "Fixed point conversion failed: FROM type does not fit in TO type"
    );
}

#[test]
fn rikiddo_get_fee_returns_the_correct_result() {
    let emv_short = ema_create_test_struct(1, 2.0);
    let emv_long = ema_create_test_struct(2, 2.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv_short, emv_long);
    assert_eq!(rikiddo.ma_short.get(), None);
    assert_eq!(rikiddo.ma_long.get(), None);
    assert_eq!(rikiddo.get_fee().unwrap(), rikiddo.config.initial_fee);
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 0, volume: 100u32.into() });
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 2, volume: 100u32.into() });
    assert_ne!(rikiddo.ma_short.get(), None);
    assert_eq!(rikiddo.ma_long.get(), None);
    assert_eq!(rikiddo.get_fee().unwrap(), rikiddo.config.initial_fee);
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 3, volume: 100u32.into() });
    assert_ne!(rikiddo.ma_short.get(), None);
    assert_ne!(rikiddo.ma_long.get(), None);
    // We don't want to test the exact result (that is the responsibility of the fee module),
    // but rather if rikiddo toggles properly between initial fee and the calculated fee
    println!("{}", rikiddo.get_fee().unwrap());
    assert_ne!(rikiddo.get_fee().unwrap(), rikiddo.config.initial_fee);
}
