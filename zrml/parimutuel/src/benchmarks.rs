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

use crate::{
    market_mock, AuthorizedOutcomeReports, Call, Config, NegativeImbalanceOf, Pallet as Authorized,
    Pallet as Parimutuel,
};
use frame_benchmarking::v2::*;
use frame_support::{
    dispatch::UnfilteredDispatchable,
    traits::{EnsureOrigin, Get, Imbalance},
};
use sp_runtime::traits::Saturating;
use zeitgeist_primitives::{
    traits::{DisputeApi, DisputeResolutionApi},
    types::{AuthorityReport, OutcomeReport},
};
use zrml_market_commons::MarketCommonsPalletApi;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn buy() {}

    #[benchmark]
    fn claim_reward() {}

    impl_benchmark_test_suite!(
        Parimutuel,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime
    );
}
