#![no_main]
//! Fuzz test: Rikiddo is called with initial fee -> cost, price, all_prices, clear

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

mod shared;
use shared::fixed_from_u128;
use substrate_fixed::{types::extra::U33, FixedI128, FixedU128};
use zrml_rikiddo::{
    traits::Lmsr,
    types::{EmaMarketVolume, FeeSigmoid, RikiddoSigmoidMV},
};

fuzz_target!(|data: Data| {
    let asset_balances_fixed: Vec<FixedU128<U33>> =
        data.asset_balances.iter().map(|e| fixed_from_u128(*e)).collect();
    let price_for_fixed = fixed_from_u128(data.price_for);
    let _ = data.rikiddo.cost(&asset_balances_fixed[..]);
    let _ = data.rikiddo.price(&asset_balances_fixed[..], &price_for_fixed);
    let _ = data.rikiddo.all_prices(&asset_balances_fixed[..]);
});

#[derive(Debug, Arbitrary)]
struct Data {
    rikiddo: RikiddoSigmoidMV<
        FixedU128<U33>,
        FixedI128<U33>,
        FeeSigmoid<FixedI128<U33>>,
        EmaMarketVolume<FixedU128<U33>>,
    >,
    asset_balances: [u128; 8],
    price_for: u128,
}
