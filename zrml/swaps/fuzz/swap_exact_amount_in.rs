#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};

use utils::SwapExactAmountData;
mod utils;
use orml_traits::MultiCurrency;
use utils::construct_asset;
use zeitgeist_primitives::constants::MinLiquidity;
use zrml_swaps::mock::Shares;

fuzz_target!(|data: SwapExactAmountData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        // ensure that the account origin has a sufficient balance
        // use orml_traits::MultiCurrency; required for this
        for a in &data.pool_creation.assets {
            let _ = Shares::deposit(
                construct_asset(*a),
                &data.pool_creation.origin,
                MinLiquidity::get(),
            );
        }
        let pool_id = data.pool_creation._create_pool();
        let _ = Shares::deposit(construct_asset(data.asset_in), &data.origin, data.asset_amount_in);
        let _ = Swaps::swap_exact_amount_in(
            Origin::signed(data.origin),
            pool_id,
            construct_asset(data.asset_in),
            data.asset_amount_in,
            construct_asset(data.asset_out),
            data.asset_amount_out,
            data.max_price,
        );
    });
    let _ = ext.commit_all();
});
