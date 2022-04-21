#![no_main]

use libfuzzer_sys::fuzz_target;

use utils::GeneralPoolData;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};
mod utils;
use utils::construct_asset;
use zeitgeist_primitives::constants::MinLiquidity;
use zrml_swaps::mock::Shares;

use orml_traits::MultiCurrency;

fuzz_target!(|data: GeneralPoolData| {
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
        // join a pool with a valid pool id
        let _ =
            Swaps::pool_join(Origin::signed(data.origin), pool_id, data.pool_amount, data.assets);
    });

    let _ = ext.commit_all();
});
