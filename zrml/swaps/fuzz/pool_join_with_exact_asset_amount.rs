#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};

use utils::ExactAssetAmountData;
mod utils;
use orml_traits::MultiCurrency;
use utils::construct_asset;
use zeitgeist_primitives::constants::MinLiquidity;
use zrml_swaps::mock::Shares;

fuzz_target!(|data: ExactAssetAmountData| {
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
        let _ = Swaps::pool_join_with_exact_asset_amount(
            Origin::signed(data.origin),
            pool_id,
            construct_asset(data.asset),
            data.asset_amount,
            data.pool_amount,
        );
    });
    let _ = ext.commit_all();
});
