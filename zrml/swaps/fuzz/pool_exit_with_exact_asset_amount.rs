#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};

mod utils;
use orml_traits::MultiCurrency;
use utils::{construct_asset, ExactAssetAmountData};
use zrml_swaps::mock::Shares;

use zeitgeist_primitives::types::{Asset, SerdeWrapper};

fuzz_target!(|data: ExactAssetAmountData| {
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

        let pool_creator = data.pool_creation.origin;
        let pool_id = data.pool_creation.create_pool();
        // to exit a pool, origin also needs to have the pool tokens of the pool that they're exiting
        let _ = Shares::deposit(
            Asset::PoolShare(SerdeWrapper(pool_id)),
            &pool_creator,
            data.pool_amount,
        );
        let _ = Swaps::pool_exit_with_exact_asset_amount(
            Origin::signed(data.origin),
            pool_id,
            construct_asset(data.asset),
            data.asset_amount,
            data.pool_amount,
        );
    });
    let _ = ext.commit_all();
});
