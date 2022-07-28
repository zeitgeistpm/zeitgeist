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
use zeitgeist_primitives::{traits::Swaps as SwapsTrait, types::ScoringRule};

use zrml_swaps::mock::{ExtBuilder, Swaps};

mod utils;
use utils::{construct_asset, PoolCreationData};

fuzz_target!(|data: PoolCreationData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let _ = Swaps::create_pool(
            data.origin,
            data.assets.into_iter().map(construct_asset).collect(),
            construct_asset(data.base_asset),
            data.market_id,
            ScoringRule::CPMM,
            data.swap_fee,
            data.amount,
            data.weights,
        );
    });
    let _ = ext.commit_all();
});
