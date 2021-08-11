use frame_support::assert_err;
use substrate_fixed::{types::extra::U64, FixedU128};

use super::{ema_create_test_struct, Rikiddo};
use crate::{
    traits::{MarketAverage, RikiddoMV},
    types::{FeeSigmoid, RikiddoConfig, TimestampedVolume},
};

#[test]
fn rikiddo_get_fee_catches_zero_divison() {
    let emv = ema_create_test_struct(1, 0.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv.clone(), emv);
    let _ = rikiddo.update_volume(&TimestampedVolume { timestamp: 0, volume: 0u32.into() });
    let _ = rikiddo.update_volume(&TimestampedVolume { timestamp: 2, volume: 0u32.into() });
    assert_err!(
        rikiddo.fee(),
        "[RikiddoSigmoidMV] Zero division error during calculation: ma_short / ma_long"
    );
}

#[test]
fn rikiddo_get_fee_overflows_during_ratio_calculation() {
    let emv = ema_create_test_struct(1, 2.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv.clone(), emv);
    let _ = rikiddo.update_volume(&TimestampedVolume { timestamp: 0, volume: 0u32.into() });
    let _ = rikiddo.update_volume(&TimestampedVolume { timestamp: 2, volume: 0u32.into() });
    rikiddo.ma_short.ema = <FixedU128<U64>>::from_num(u64::MAX);
    rikiddo.ma_long.ema = <FixedU128<U64>>::from_num(0.1f64);
    assert_err!(
        rikiddo.fee(),
        "[RikiddoSigmoidMV] Overflow during calculation: ma_short / ma_long"
    );
}

#[test]
fn rikiddo_get_fee_ratio_does_not_fit_in_type() {
    let emv = ema_create_test_struct(1, 2.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv.clone(), emv);
    let _ = rikiddo.update_volume(&TimestampedVolume { timestamp: 0, volume: 0u32.into() });
    let _ = rikiddo.update_volume(&TimestampedVolume { timestamp: 2, volume: 0u32.into() });
    rikiddo.ma_short.ema = <FixedU128<U64>>::from_num(u64::MAX);
    rikiddo.ma_long.ema = <FixedU128<U64>>::from_num(1u64);
    assert_err!(rikiddo.fee(), "Fixed point conversion failed: FROM type does not fit in TO type");
}

#[test]
fn rikiddo_get_fee_returns_the_correct_result() {
    let emv_short = ema_create_test_struct(1, 2.0);
    let emv_long = ema_create_test_struct(2, 2.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv_short, emv_long);
    assert_eq!(rikiddo.ma_short.get(), None);
    assert_eq!(rikiddo.ma_long.get(), None);
    assert_eq!(rikiddo.fee().unwrap(), rikiddo.config.initial_fee);
    let _ = rikiddo.update_volume(&TimestampedVolume { timestamp: 0, volume: 100u32.into() });
    let _ = rikiddo.update_volume(&TimestampedVolume { timestamp: 2, volume: 100u32.into() });
    assert_ne!(rikiddo.ma_short.get(), None);
    assert_eq!(rikiddo.ma_long.get(), None);
    assert_eq!(rikiddo.fee().unwrap(), rikiddo.config.initial_fee);
    let _ = rikiddo.update_volume(&TimestampedVolume { timestamp: 3, volume: 100u32.into() });
    assert_ne!(rikiddo.ma_short.get(), None);
    assert_ne!(rikiddo.ma_long.get(), None);
    // We don't want to test the exact result (that is the responsibility of the fee module),
    // but rather if rikiddo toggles properly between initial fee and the calculated fee
    assert_ne!(rikiddo.fee().unwrap(), rikiddo.config.initial_fee);
}
