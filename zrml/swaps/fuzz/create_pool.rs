// Copyright 2023-2024 Forecasting Technologies LTD.
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

mod utils;

use libfuzzer_sys::fuzz_target;
use utils::{construct_asset, PoolCreationData};
use zeitgeist_primitives::traits::Swaps as SwapsTrait;
use zrml_swaps::mock::{ExtBuilder, Swaps};

fuzz_target!(|data: PoolCreationData| {
    let mut ext = ExtBuilder::default().build();
    ext.execute_with(|| {
        let _ = Swaps::create_pool(
            data.origin,
            data.assets.into_iter().map(construct_asset).collect(),
            data.swap_fee,
            data.amount,
            data.weights,
        );
    });
    let _ = ext.commit_all();
});
