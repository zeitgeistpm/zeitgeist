#![no_main]

use libfuzzer_sys::fuzz_target;
use frame_system::ensure_signed;
use zeitgeist_primitives::types::{Asset, ScalarPosition,SerdeWrapper};
use zrml_orderbook_v1::{
    OrderSide,
    mock::{ExtBuilder, Origin, Orderbook},
};

#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result, Unstructured};

fuzz_target!(|data: Data| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let order_asset = asset(data.make_order_asset);
        let order_hash = Orderbook::order_hash(
            &ensure_signed(
                Origin::signed(data.make_order_origin.into())
            ).unwrap(),
            order_asset,
            Orderbook::nonce(),
        );

        let _ = Orderbook::make_order(
            Origin::signed(data.make_order_origin.into()),
            order_asset,
            orderside(data.make_order_side),
            data.make_order_amount,
            data.make_order_price,
        );

        let _ = Orderbook::fill_order(
            Origin::signed(data.fill_order_origin.into()),
            order_hash,
        );
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct Data {
    make_order_amount: u128,
    make_order_asset: (u128, u16),
    make_order_price: u128,
    make_order_origin: u8,
    make_order_side: u8,

    fill_order_hash: u8,
    fill_order_origin: u8,
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

fn orderside(seed: u8) -> OrderSide {
    let module = seed % 2;
    match module {
        0 => OrderSide::Bid,
        _ => OrderSide::Ask,
    }
}
