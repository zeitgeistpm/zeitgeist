#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use substrate_fixed::{types::extra::U33, FixedI128};
use zrml_rikiddo::{traits::Sigmoid, types::{FeeSigmoid, FeeSigmoidConfig}};

/*
 Workflow 1:
 - FeeSigmoid.calculate() is called

 Workflow 2:
 - EmaMarketVolume is called, update once
   -> create, update once, get ema, clear

 - EmaMarketVolume is called, update multiple times
   -> create, set ema period 1 second, update multiple times and get ema, clear

 Workflow 3:
 - Rikiddo is called with initial fee
   -> create, cost, price, all_prices, clear
 - Rikiddo is called with calculated fee
   -> create, Force EmaMarketVolume, cost, price, all_prices

 Workflow 4:
  - Rikiddo pallet is called with initial fee
   -> create, fee, cost, price, all_prices, clear, destroy
  - Rikiddo pallet is called with other fee
   -> create, force fee by multiple update_volume, cost, price, all_prices, clear, destroy
*/

fuzz_target!(|data: Data| {
    let sigmoid_fee_config = FeeSigmoidConfig {
        m: data.sigmoid_fee_m,
        p: data.sigmoid_fee_p,
        n: data.sigmoid_fee_n,
        initial_fee: data.sigmoid_fee_initial_fee,
        min_revenue: data.sigmoid_fee_min_revenue,
    };
    let sigmoid_fee = FeeSigmoid::new(sigmoid_fee_config);
    let _ = sigmoid_fee.calculate_fee(data.sigmoid_fee_calculate_r);
});

#[derive(Debug, Arbitrary)]
struct Data {
    sigmoid_fee_calculate_r: FixedI128<U33>,
    sigmoid_fee_m: FixedI128<U33>,
    sigmoid_fee_p: FixedI128<U33>,
    sigmoid_fee_n: FixedI128<U33>,
    sigmoid_fee_initial_fee: FixedI128<U33>,
    sigmoid_fee_min_revenue: FixedI128<U33>,
}
