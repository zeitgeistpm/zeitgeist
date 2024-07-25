// Copyright 2024 Forecasting Technologies LTD.
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
use orml_traits::currency::MultiCurrency;
use utils::{construct_asset, ExactAssetAmountData};
use zeitgeist_primitives::types::Asset;
use zrml_swaps::mock::{Currencies, ExtBuilder, RuntimeOrigin, Swaps};

fuzz_target!(|data: ExactAssetAmountData| {
    let mut ext = ExtBuilder::default().build();
    ext.execute_with(|| {
        // ensure that the account origin has a sufficient balance
        for a in &data.pool_creation.assets {
            let _ = Currencies::deposit(
                construct_asset(*a),
                &data.pool_creation.origin,
                data.pool_creation.amount,
            );
        }

        let pool_creator = data.pool_creation.origin;
        let pool_id = data.pool_creation.create_pool();
        // to exit a pool, origin also needs to have the pool tokens of the pool that they're exiting
        let _ = Currencies::deposit(Asset::PoolShare(pool_id), &pool_creator, data.pool_amount);
        let _ = Swaps::pool_exit_with_exact_asset_amount(
            RuntimeOrigin::signed(data.origin),
            pool_id,
            construct_asset(data.asset),
            data.asset_amount,
            data.pool_amount,
        );
    });
    let _ = ext.commit_all();
});
