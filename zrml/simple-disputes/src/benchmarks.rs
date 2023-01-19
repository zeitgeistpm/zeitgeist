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
#![allow(clippy::type_complexity)]
#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, vec, whitelisted_caller};

benchmarks! {
    reserve_outcome {
        let d in 1..(T::MaxDisputes::get() - 1);
        let e in 1..63;

        let caller: T::AccountId = whitelisted_caller();
        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market).unwrap();

        let now = <frame_system::Pallet<T>>::block_number();

        for i in 0..d {

        }
        let last_dispute = MarketDispute {
            at: now,
            by: caller.clone(),
            outcome: OutcomeReport::Scalar(2),
            bond: default_outcome_bond::<T>(0usize),
        };
        Disputes::<T>::insert(market_id, last_dispute);


        let dispute_duration_ends_at_block =
                now.saturating_add(market.deadlines.dispute_duration);
        for i in 0..e {
            let id = T::MarketCommons::push_market(market_mock::<T>()).unwrap();
            T::DisputeResolution::add_auto_resolve(&id, dispute_duration_ends_at_block).unwrap();
        }

        let outcome = OutcomeReport::Scalar(1);
        let bond = default_outcome_bond::<T>(0usize);
        let _ = T::Currency::deposit_creating(&caller, bond);
    }: _(RawOrigin::Signed(caller.clone()), market_id, outcome)

    impl_benchmark_test_suite!(
        PredictionMarket,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
