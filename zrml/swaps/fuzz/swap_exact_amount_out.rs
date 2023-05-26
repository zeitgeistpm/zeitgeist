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
use zrml_swaps::mock::{AssetManager, ExtBuilder, RuntimeOrigin, Swaps};

mod utils;
use orml_traits::currency::MultiCurrency;
use utils::{construct_asset, SwapExactAmountOutData};

fuzz_target!(|data: SwapExactAmountOutData| {
    let mut ext = ExtBuilder::default().build();
    ext.execute_with(|| {
        // ensure that the account origin has a sufficient balance
        // use orml_traits::MultiCurrency; required for this
        for a in &data.pool_creation.assets {
            let _ = AssetManager::deposit(
                construct_asset(*a),
                &data.pool_creation.origin,
                data.pool_creation.amount,
            );
        }
        let pool_id = data.pool_creation.create_pool();

        if let Some(amount) = data.asset_amount_in {
            let _ = AssetManager::deposit(construct_asset(data.asset_in), &data.origin, amount);
        }

        let _ = Swaps::swap_exact_amount_out(
            RuntimeOrigin::signed(data.origin),
            pool_id,
            construct_asset(data.asset_in),
            data.asset_amount_in,
            construct_asset(data.asset_out),
            data.asset_amount_out,
            data.max_price,
        );
    });
    let _ = ext.commit_all();
});
