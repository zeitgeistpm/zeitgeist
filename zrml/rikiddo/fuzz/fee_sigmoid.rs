#![no_main]
//! Fuzz test: FeeSigmoid.calculate() is called

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use substrate_fixed::{FixedI128, types::extra::U33};
use zrml_rikiddo::{
    traits::Sigmoid,
    types::{FeeSigmoid, FeeSigmoidConfig},
};

mod shared;
use shared::fixed_from_i128;

/*
 Target 1:
 - FeeSigmoid.calculate() is called

 Target 2:
 - EmaMarketVolume is called, update once
   -> create, update once, get ema, clear

 Target 3:
 - EmaMarketVolume is called, update multiple times
   -> create, set ema period 1 second, update multiple times and get ema, clear

 Target 4:
 - Rikiddo is called with initial fee
   -> create, cost, price, all_prices, clear

 Target 5:
 - Rikiddo is called with calculated fee
   -> create, Force EmaMarketVolume, cost, price, all_prices

 Target 6:
  - Rikiddo pallet is called with initial fee
   -> create, fee, cost, price, all_prices, clear, destroy

 Target 7:
  - Rikiddo pallet is called with other fee
   -> create, force fee by multiple update_volume, cost, price, all_prices, clear, destroy

 Target 8:
   - Conversion FixedU -> FixedI

 Target 9:
   - Conversion FixedI -> FixedU

 Target 10:
   - Conversion Fixed -> Balance

 Target 11:
   - Conversion Balance -> Fixed
*/

fuzz_target!(|data: Data| {
    let _ = data.sigmoid_fee.calculate_fee(fixed_from_i128(data.sigmoid_fee_calculate_r));
});

#[derive(Debug, Arbitrary)]
struct Data {
    sigmoid_fee_calculate_r: i128,
    sigmoid_fee: FeeSigmoid<FixedI128<U33>>
    /*sigmoid_fee_m: i128,
    sigmoid_fee_p: i128,
    sigmoid_fee_n: i128,
    sigmoid_fee_initial_fee: i128,
    sigmoid_fee_min_revenue: i128,*/
}
