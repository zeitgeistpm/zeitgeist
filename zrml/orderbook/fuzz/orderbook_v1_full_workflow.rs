// Copyright 2023-2025 Forecasting Technologies LTD.
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
use zeitgeist_primitives::types::{Asset, ScalarPosition};
use zrml_orderbook::mock::{ExtBuilder, Orderbook, RuntimeOrigin};

#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result, Unstructured};

fuzz_target!(|data: Data| {
    let mut ext = ExtBuilder::default().build();
    ext.execute_with(|| {
        // Make arbitrary order and attempt to fill
        let maker_asset = asset(data.maker_asset);
        let taker_asset = asset(data.taker_asset);

        let _ = Orderbook::place_order(
            RuntimeOrigin::signed(data.fill_order_origin.into()),
            data.market_id,
            maker_asset,
            data.maker_amount,
            taker_asset,
            data.taker_amount,
        );

        let _ = Orderbook::fill_order(
            RuntimeOrigin::signed(data.fill_order_origin.into()),
            data.order_id,
            maker_partial_fill(data.maker_partial_fill),
        );

        // Make arbitrary order and attempt to remove
        let _ = Orderbook::place_order(
            RuntimeOrigin::signed(data.place_order_origin.into()),
            data.market_id,
            maker_asset,
            data.maker_amount,
            taker_asset,
            data.taker_amount,
        );

        let _ = Orderbook::remove_order(
            RuntimeOrigin::signed(data.remove_order_origin.into()),
            data.order_id,
        );
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct Data {
    market_id: u128,
    order_id: u128,

    place_order_origin: u8,
    maker_asset: (u128, u16),
    maker_amount: u128,
    taker_asset: (u128, u16),
    taker_amount: u128,

    fill_order_origin: u8,
    maker_partial_fill: u128,

    remove_order_origin: u8,
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
        2 => Asset::PoolShare(seed0),
        _ => Asset::Ztg,
    }
}

fn maker_partial_fill(s: u128) -> Option<u128> {
    if s % 2 == 0 {
        Some(s)
    } else {
        None
    }
}
