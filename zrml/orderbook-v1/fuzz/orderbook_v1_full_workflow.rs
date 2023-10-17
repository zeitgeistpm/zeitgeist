// Copyright 2023 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

#![no_main]

use libfuzzer_sys::fuzz_target;
use zeitgeist_primitives::types::{Asset, ScalarPosition, SerdeWrapper};
use zrml_orderbook_v1::{
    mock::{ExtBuilder, Orderbook, RuntimeOrigin},
    types::OrderSide,
};

#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result, Unstructured};

fuzz_target!(|data: Data| {
    let mut ext = ExtBuilder::default().build();
    ext.execute_with(|| {
        // Make arbitrary order and attempt to fill
        let outcome_asset = asset(data.fill_order_outcome_asset);

        let _ = Orderbook::place_order(
            RuntimeOrigin::signed(data.fill_order_origin.into()),
            data.market_id,
            outcome_asset,
            orderside(data.fill_order_side),
            data.fill_order_amount,
            data.fill_order_price,
        );

        let _ = Orderbook::fill_order(
            RuntimeOrigin::signed(data.fill_order_origin.into()),
            data.order_id,
            None,
        );

        // Make arbitrary order and attempt to cancel
        let _ = Orderbook::place_order(
            RuntimeOrigin::signed(data.cancel_order_origin.into()),
            data.market_id,
            outcome_asset,
            orderside(data.cancel_order_side),
            data.cancel_order_amount,
            data.cancel_order_price,
        );

        let _ = Orderbook::remove_order(
            RuntimeOrigin::signed(data.cancel_order_origin.into()),
            data.order_id,
        );
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct Data {
    market_id: u128,
    order_id: u128,

    fill_order_amount: u128,
    fill_order_outcome_asset: (u128, u16),
    fill_order_price: u128,
    fill_order_origin: u8,
    fill_order_side: u8,

    cancel_order_amount: u128,
    cancel_order_price: u128,
    cancel_order_origin: u8,
    cancel_order_side: u8,
}

fn asset(seed: (u128, u16)) -> Asset<u128> {
    let (seed0, seed1) = seed;
    let module = seed0 % 3;
    match module {
        0 => Asset::CategoricalOutcome(seed0, seed1),
        1 => {
            let scalar_position =
                if seed1 % 2 == 0 { ScalarPosition::Long } else { ScalarPosition::Short };
            Asset::ScalarOutcome(seed0, scalar_position)
        }
        2 => Asset::PoolShare(SerdeWrapper(seed0)),
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
