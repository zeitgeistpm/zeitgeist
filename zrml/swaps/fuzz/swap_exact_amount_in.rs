#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{AccountId, AssetManager, ExtBuilder, Origin, Swaps};

use utils::SwapExactAmountInData;
mod utils;
use utils::construct_asset;
use zeitgeist_primitives::traits::ZeitgeistAssetManager;

fuzz_target!(|data: SwapExactAmountInData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        // ensure that the account origin has a sufficient balance
        // use orml_traits::MultiCurrency; required for this
        for a in &data.pool_creation.assets {
            let _ = <AssetManager as ZeitgeistAssetManager<AccountId>>::deposit(
                construct_asset(*a),
                &data.pool_creation.origin,
                data.pool_creation.amount,
            );
        }
        let pool_id = data.pool_creation.create_pool();
        let _ = <AssetManager as ZeitgeistAssetManager<AccountId>>::deposit(
            construct_asset(data.asset_in),
            &data.origin,
            data.asset_amount_in,
        );
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
