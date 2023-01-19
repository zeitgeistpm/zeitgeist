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

#[cfg(test)]
use crate::Pallet as SimpleDisputes;

use super::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller, Vec};
use frame_support::{dispatch::RawOrigin, traits::Get};
use orml_traits::MultiCurrency;
use sp_runtime::traits::{One, Saturating};
use zrml_market_commons::MarketCommonsPalletApi;

benchmarks! {
    reserve_outcome {
        let d in 1..(T::MaxDisputes::get() - 1);
        let r in 1..63;
        let e in 1..63;

        let caller: T::AccountId = whitelisted_caller();
        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();

        let mut now;

        let mut disputes = Vec::new();
        for i in 0..d {
            now = <frame_system::Pallet<T>>::block_number();

            let disputor = account("disputor", i, 0);
            let last_dispute = MarketDispute {
                at: now,
                by: disputor,
                outcome: OutcomeReport::Scalar((2 + i).into()),
                bond: default_outcome_bond::<T>(i as usize),
            };
            disputes.push(last_dispute);
            <frame_system::Pallet<T>>::set_block_number(now.saturating_add(T::BlockNumber::one()));
        }
        let last_dispute = disputes.last().unwrap();
        let auto_resolve = last_dispute.at.saturating_add(market.deadlines.dispute_duration);
        for i in 0..r {
            let id = T::MarketCommons::push_market(market_mock::<T>()).unwrap();
            T::DisputeResolution::add_auto_resolve(&id, auto_resolve).unwrap();
        }

        let now = <frame_system::Pallet<T>>::block_number();

        let bounded_vec = <DisputesOf<T>>::try_from(disputes).unwrap();
        Disputes::<T>::insert(market_id, bounded_vec);

        let dispute_duration_ends_at_block =
                now.saturating_add(market.deadlines.dispute_duration);
        for i in 0..e {
            let id = T::MarketCommons::push_market(market_mock::<T>()).unwrap();
            T::DisputeResolution::add_auto_resolve(&id, dispute_duration_ends_at_block).unwrap();
        }

        let outcome = OutcomeReport::Scalar(1);
        let bond = default_outcome_bond::<T>(T::MaxDisputes::get() as usize);
        T::AssetManager::deposit(Asset::Ztg, &caller, bond).unwrap();
    }: _(RawOrigin::Signed(caller.clone()), market_id, outcome)

    impl_benchmark_test_suite!(
        SimpleDisputes,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
