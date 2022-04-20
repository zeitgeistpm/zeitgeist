#![no_main]

use libfuzzer_sys::fuzz_target;

use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};
mod data_structs;
use data_structs::GeneralPoolData;
use zeitgeist_primitives::{traits::Swaps as SwapsTrait, types::ScoringRule};
mod helper_functions;
use helper_functions::{construct_asset, _CREATE_POOL_FAILURE};
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

        match Swaps::create_pool(
            data.pool_creation.origin,
            data.pool_creation.assets.into_iter().map(construct_asset).collect(),
            construct_asset(data.pool_creation.base_asset),
            data.pool_creation.market_id,
            ScoringRule::CPMM,
            Some(data.pool_creation.swap_fee),
            Some(data.pool_creation.weights),
        ) {
            Ok(pool_id) => {
                // join a pool with a valid pool id
                let _ = Swaps::pool_join(
                    Origin::signed(data.origin),
                    pool_id,
                    data.pool_amount,
                    data.assets,
                );
            }
            Err(e) => panic!("{_CREATE_POOL_FAILURE} {:?}", e),
        }
    });

    let _ = ext.commit_all();
});
