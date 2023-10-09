// Copyright 2022-2023 Forecasting Technologies LTD.
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

#![allow(
    // Auto-generated code is a no man's land
    clippy::arithmetic_side_effects
)]
#![cfg(feature = "runtime-benchmarks")]

#[cfg(test)]
use crate::Pallet as Parimutuel;
use crate::*;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use sp_runtime::SaturatedConversion;
use zeitgeist_primitives::types::{Asset, Outcome};

#[benchmarks]
mod benchmarks_parimutuel {
    use super::*;

    #[benchmark]
    fn buy() {
        let buyer = whitelisted_caller();
        let market_id = 0u32.into();
        let asset = Asset::ParimutuelShare(Outcome::CategoricalOutcome(market_id, 2u16));
        let amount = 100_000_000u128.saturated_into::<BalanceOf<T>>();

        #[extrinsic_call]
        buy(RawOrigin::Signed(buyer), asset, amount);
    }

    #[benchmark]
    fn claim_rewards() {
        let buyer = whitelisted_caller();
        let market_id = 0u32.into();
        let market_ = market_mock::<T>();

        #[extrinsic_call]
        claim_rewards(RawOrigin::Signed(buyer), market_id);
    }

    impl_benchmark_test_suite!(
        Parimutuel,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime
    );
}
