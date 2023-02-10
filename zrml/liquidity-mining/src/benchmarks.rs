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

#![allow(
    // Auto-generated code is a no man's land
    clippy::integer_arithmetic
)]
#![cfg(feature = "runtime-benchmarks")]

use crate::pallet::{BalanceOf, Call, Config, Pallet};
#[cfg(test)]
use crate::Pallet as LiquidityMining;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;

benchmarks! {
    set_per_block_distribution {
        let balance = BalanceOf::<T>::max_value();
    }: set_per_block_distribution(RawOrigin::Root, balance)

    impl_benchmark_test_suite!(
        LiquidityMining,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
