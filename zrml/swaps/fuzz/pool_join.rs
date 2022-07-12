#![no_main]

use libfuzzer_sys::fuzz_target;
use zeitgeist_primitives::traits::ZeitgeistAssetManager;

use utils::GeneralPoolData;
use zrml_swaps::mock::{AccountId, ExtBuilder, Origin, Swaps};
mod utils;
use utils::construct_asset;
use zrml_swaps::mock::AssetManager;

fuzz_target!(|data: GeneralPoolData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        // ensure that the account origin has a sufficient balance
        // use orml_traits::MultiCurrency; required for this
        for a in &data.pool_creation.assets {
            <AssetManager as ZeitgeistAssetManager<AccountId>>::deposit(
                construct_asset(*a),
                &data.pool_creation.origin,
                data.pool_creation.amount,
            );
        }
        let pool_id = data.pool_creation.create_pool();
        // join a pool with a valid pool id
        let _ = Swaps::pool_join(
            Origin::signed(data.origin),
            pool_id,
            data.pool_amount,
            data.asset_bounds,
        );
    });

    let _ = ext.commit_all();
});
