#![no_main]

use frame_system::ensure_signed;
use libfuzzer_sys::fuzz_target;
use zeitgeist_primitives::types::{Asset, ScalarPosition, SerdeWrapper};
use zrml_orderbook_v1::{
    mock::{ExtBuilder, Orderbook, Origin},
    OrderSide,
};

#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result, Unstructured};

fuzz_target!(|data: Data| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        // Make arbitrary order and attempt to fill
        let order_asset = asset(data.make_fill_order_asset);
        let order_hash = Orderbook::order_hash(
            &ensure_signed(Origin::signed(data.make_fill_order_origin.into())).unwrap(),
            order_asset,
            Orderbook::nonce(),
        );

        let _ = Orderbook::make_order(
            Origin::signed(data.make_fill_order_origin.into()),
            order_asset,
            orderside(data.make_fill_order_side),
            data.make_fill_order_amount,
            data.make_fill_order_price,
        );

        let _ = Orderbook::fill_order(Origin::signed(data.fill_order_origin.into()), order_hash);

        // Make arbitrary order and attempt to cancel
        let order_asset = asset(data.make_cancel_order_asset);
        let order_hash = Orderbook::order_hash(
            &ensure_signed(Origin::signed(data.make_cancel_order_origin.into())).unwrap(),
            order_asset,
            Orderbook::nonce(),
        );

        let _ = Orderbook::make_order(
            Origin::signed(data.make_cancel_order_origin.into()),
            order_asset,
            orderside(data.make_cancel_order_side),
            data.make_cancel_order_amount,
            data.make_cancel_order_price,
        );

        let _ = Orderbook::cancel_order(
            Origin::signed(data.make_cancel_order_origin.into()),
            order_asset,
            order_hash,
        );
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct Data {
    make_fill_order_amount: u128,
    make_fill_order_asset: (u128, u16),
    make_fill_order_price: u128,
    make_fill_order_origin: u8,
    make_fill_order_side: u8,

    fill_order_origin: u8,

    make_cancel_order_amount: u128,
    make_cancel_order_asset: (u128, u16),
    make_cancel_order_price: u128,
    make_cancel_order_origin: u8,
    make_cancel_order_side: u8,
}

fn asset(seed: (u128, u16)) -> Asset<u128> {
    let (seed0, seed1) = seed;
    let module = seed0 % 5;
    match module {
        0 => Asset::CategoricalOutcome(seed0, seed1),
        1 => {
            let scalar_position =
                if seed1 % 2 == 0 { ScalarPosition::Long } else { ScalarPosition::Short };
            Asset::ScalarOutcome(seed0, scalar_position)
        }
        2 => Asset::CombinatorialOutcome,
        3 => Asset::PoolShare(SerdeWrapper(seed0)),
        _ => Asset::ZTG,
    }
}

fn orderside(seed: u8) -> OrderSide {
    let module = seed % 2;
    match module {
        0 => OrderSide::Bid,
        _ => OrderSide::Ask,
    }
}
