// Copyright 2023 Forecasting Technologies LTD.
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

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::{consts::*, Pallet as NeoSwaps};
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::SaturatedConversion;
use zeitgeist_primitives::types::Asset;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn buy(a: Linear<2, 2>) {
        let caller: T::AccountId = whitelisted_caller();
        let market_id = 0u8.into(); // TODO
        let asset_out = Asset::CategoricalOutcome(market_id, 0);
        let amount_in = _1.saturated_into();
        let min_amount_out = 0u8.saturated_into();

        #[extrinsic_call]
        _(RawOrigin::Signed(caller.clone()), market_id, 2, asset_out, amount_in, min_amount_out);
    }

    impl_benchmark_test_suite!(
        NeoSwaps,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime
    );
}
