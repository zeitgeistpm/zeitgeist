#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};

mod data_structs;
use data_structs::ExactAssetAmountData;
mod helper_functions;
use helper_functions::asset;
use orml_traits::MultiCurrency;
use zeitgeist_primitives::constants::MinLiquidity;
use zrml_swaps::mock::Shares;

use zeitgeist_primitives::{traits::Swaps as SwapsTrait, types::ScoringRule};

fuzz_target!(|data: ExactAssetAmountData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        // ensure that the account origin has a sufficient balance
        // use orml_traits::MultiCurrency; required for this
        for a in &data.pool_creation.assets {
            let _ = Shares::deposit(asset(*a), &data.pool_creation.origin, MinLiquidity::get());
        }
        match Swaps::create_pool(
            data.pool_creation.origin,
            data.pool_creation.assets.into_iter().map(asset).collect(),
            asset(data.pool_creation.base_asset),
            data.pool_creation.market_id,
            ScoringRule::CPMM,
            Some(data.pool_creation.swap_fee),
            Some(data.pool_creation.weights),
        ) {
            Ok(pool_id) => {
                let _ = Swaps::pool_join_with_exact_asset_amount(
                    Origin::signed(data.origin),
                    pool_id,
                    asset(data.asset),
                    data.asset_amount,
                    data.pool_amount,
                );
            }
            Err(e) => panic!(
                "There needs to be a valid pool creation! This Swaps::create_pool call returns an \
                 error, but should be ok. Error: {:?}",
                e
            ),
        }
    });
    let _ = ext.commit_all();
});
