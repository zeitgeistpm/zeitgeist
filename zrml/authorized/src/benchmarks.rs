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

#[cfg(test)]
use crate::Pallet as Authorized;
use crate::{market_mock, Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_support::{dispatch::UnfilteredDispatchable, traits::EnsureOrigin};
use zeitgeist_primitives::types::OutcomeReport;
use zrml_market_commons::MarketCommonsPalletApi;

benchmarks! {
    authorize_market_outcome {
        let origin = T::AuthorizedDisputeResolutionOrigin::successful_origin();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market).unwrap();
        let call = Call::<T>::authorize_market_outcome { market_id: 0_u32.into(), outcome: OutcomeReport::Scalar(1) };
    }: { call.dispatch_bypass_filter(origin)? }
}

impl_benchmark_test_suite!(Authorized, crate::mock::ExtBuilder::default().build(), crate::mock::Runtime);
