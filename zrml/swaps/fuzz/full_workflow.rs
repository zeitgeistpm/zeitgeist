#![no_main]

use libfuzzer_sys::fuzz_target;
use zeitgeist_primitives::types::{Asset, ScalarPosition, SerdeWrapper};
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};

fuzz_target!(|data: Data| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let _ = Swaps::create_pool(
            Origin::signed(data.create_pool_origin.into()),
            data.create_pool_assets.into_iter().map(asset).collect(),
            data.create_pool_weights,
        );

        let _ = Swaps::pool_join(
            Origin::signed(data.pool_join_origin.into()),
            data.pool_join_pool_id.into(),
            data.pool_join_pool_amount,
            data.pool_join_max_assets_in,
        );

        let _ = Swaps::pool_join_with_exact_asset_amount(
            Origin::signed(data.pool_join_with_exact_asset_amount_origin.into()),
            data.pool_join_with_exact_asset_amount_pool_id.into(),
            asset(data.pool_join_with_exact_asset_amount_asset_in),
            data.pool_join_with_exact_asset_amount_asset_amount,
            data.pool_join_with_exact_asset_amount_min_pool_amount,
        );

        let _ = Swaps::pool_join_with_exact_pool_amount(
            Origin::signed(data.pool_join_with_exact_pool_amount_origin.into()),
            data.pool_join_with_exact_pool_amount_pool_id.into(),
            asset(data.pool_join_with_exact_pool_amount_asset),
            data.pool_join_with_exact_pool_amount_pool_amount,
            data.pool_join_with_exact_pool_amount_max_asset_amount,
        );

        let _ = Swaps::swap_exact_amount_in(
            Origin::signed(data.swap_exact_amount_in_origin.into()),
            data.swap_exact_amount_in_pool_id.into(),
            asset(data.swap_exact_amount_in_asset_in),
            data.swap_exact_amount_in_asset_amount_in,
            asset(data.swap_exact_amount_in_asset_out),
            data.swap_exact_amount_in_min_asset_amount_out,
            data.swap_exact_amount_in_max_price,
        );

        let _ = Swaps::swap_exact_amount_out(
            Origin::signed(data.swap_exact_amount_out_origin.into()),
            data.swap_exact_amount_out_pool_id.into(),
            asset(data.swap_exact_amount_out_asset_in),
            data.swap_exact_amount_out_max_amount_asset_in,
            asset(data.swap_exact_amount_out_asset_out),
            data.swap_exact_amount_out_asset_amount_out,
            data.swap_exact_amount_out_max_price,
        );

        let _ = Swaps::pool_exit_with_exact_pool_amount(
            Origin::signed(data.pool_exit_with_exact_pool_amount_origin.into()),
            data.pool_exit_with_exact_pool_amount_pool_id.into(),
            asset(data.pool_exit_with_exact_pool_amount_asset),
            data.pool_exit_with_exact_pool_amount_pool_amount,
            data.pool_exit_with_exact_pool_amount_min_asset_amount,
        );

        let _ = Swaps::pool_exit_with_exact_asset_amount(
            Origin::signed(data.pool_exit_with_exact_asset_amount_origin.into()),
            data.pool_exit_with_exact_asset_amount_pool_id.into(),
            asset(data.pool_exit_with_exact_asset_amount_asset),
            data.pool_exit_with_exact_asset_amount_asset_amount,
            data.pool_exit_with_exact_asset_amount_max_pool_amount,
        );

        let _ = Swaps::pool_exit(
            Origin::signed(data.pool_exit_origin.into()),
            data.pool_exit_pool_id.into(),
            data.pool_exit_pool_amount,
            data.pool_exit_min_assets_out,
        );
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct Data {
    create_pool_origin: u8,
    create_pool_assets: Vec<(u128, u16)>,
    create_pool_weights: Vec<u128>,

    pool_join_origin: u8,
    pool_join_pool_id: u8,
    pool_join_pool_amount: u128,
    pool_join_max_assets_in: Vec<u128>,

    pool_join_with_exact_asset_amount_origin: u8,
    pool_join_with_exact_asset_amount_pool_id: u8,
    pool_join_with_exact_asset_amount_asset_in: (u128, u16),
    pool_join_with_exact_asset_amount_asset_amount: u128,
    pool_join_with_exact_asset_amount_min_pool_amount: u128,

    pool_join_with_exact_pool_amount_origin: u8,
    pool_join_with_exact_pool_amount_pool_id: u8,
    pool_join_with_exact_pool_amount_asset: (u128, u16),
    pool_join_with_exact_pool_amount_pool_amount: u128,
    pool_join_with_exact_pool_amount_max_asset_amount: u128,

    swap_exact_amount_in_origin: u8,
    swap_exact_amount_in_pool_id: u8,
    swap_exact_amount_in_asset_in: (u128, u16),
    swap_exact_amount_in_asset_amount_in: u128,
    swap_exact_amount_in_asset_out: (u128, u16),
    swap_exact_amount_in_min_asset_amount_out: u128,
    swap_exact_amount_in_max_price: u128,

    swap_exact_amount_out_origin: u8,
    swap_exact_amount_out_pool_id: u8,
    swap_exact_amount_out_asset_in: (u128, u16),
    swap_exact_amount_out_max_amount_asset_in: u128,
    swap_exact_amount_out_asset_out: (u128, u16),
    swap_exact_amount_out_asset_amount_out: u128,
    swap_exact_amount_out_max_price: u128,

    pool_exit_with_exact_pool_amount_origin: u8,
    pool_exit_with_exact_pool_amount_pool_id: u8,
    pool_exit_with_exact_pool_amount_asset: (u128, u16),
    pool_exit_with_exact_pool_amount_pool_amount: u128,
    pool_exit_with_exact_pool_amount_min_asset_amount: u128,

    pool_exit_with_exact_asset_amount_origin: u8,
    pool_exit_with_exact_asset_amount_pool_id: u8,
    pool_exit_with_exact_asset_amount_asset: (u128, u16),
    pool_exit_with_exact_asset_amount_asset_amount: u128,
    pool_exit_with_exact_asset_amount_max_pool_amount: u128,

    pool_exit_origin: u8,
    pool_exit_pool_id: u8,
    pool_exit_pool_amount: u128,
    pool_exit_min_assets_out: Vec<u128>,
}

fn asset(seed: (u128, u16)) -> Asset<u128> {
    let (seed0, seed1) = seed;
    let module = seed0 % 4;
    match module {
        0 => Asset::CategoricalOutcome(seed0, seed1),
        1 => {
            let scalar_position =
                if seed1 % 2 == 0 { ScalarPosition::Long } else { ScalarPosition::Short };
            Asset::ScalarOutcome(seed0, scalar_position)
        }
        2 => Asset::CombinatorialOutcome,
        3 => Asset::PoolShare(SerdeWrapper(seed0)),
        _ => Asset::Ztg,
    }
}
