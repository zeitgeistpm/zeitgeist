#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};

mod utils;
use orml_traits::MultiCurrency;
use utils::{construct_asset, SwapExactAmountOutData};
use zrml_swaps::mock::Shares;

fuzz_target!(|data: SwapExactAmountOutData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        // ensure that the account origin has a sufficient balance
        // use orml_traits::MultiCurrency; required for this
        for a in &data.pool_creation.assets {
            let _ = Shares::deposit(
                construct_asset(*a),
                &data.pool_creation.origin,
                data.pool_creation.amount,
            );
        }
        let pool_id = data.pool_creation.create_pool();

        if let Some(amount) = data.asset_amount_in {
            let _ = Shares::deposit(construct_asset(data.asset_in), &data.origin, amount);
        }

        let _ = Swaps::swap_exact_amount_out(
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
