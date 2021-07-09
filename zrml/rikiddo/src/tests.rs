#![cfg(test)]

use crate::mock::*;

mod ema_market_volume;
mod rikiddo_sigmoid_mv;
mod sigmoid_fee;

fn max_allowed_error(fractional_bits: u8) -> f64 {
    1.0 / (1u128 << (fractional_bits - 1)) as f64
}

#[test]
fn it_is_a_dummy_test() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(true);
    });
}
