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
    clippy::integer_arithmetic
)]
#![allow(clippy::type_complexity)]
#![cfg(feature = "runtime-benchmarks")]

use crate::Pallet as SimpleDisputes;

use super::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::{
    dispatch::RawOrigin,
    traits::{Get, Imbalance},
};
use orml_traits::MultiCurrency;
use sp_runtime::traits::{One, Saturating};
use zrml_market_commons::MarketCommonsPalletApi;

fn fill_disputes<T: Config>(market_id: MarketIdOf<T>, d: u32) {
    for i in 0..d {
        let now = <frame_system::Pallet<T>>::block_number();
        let disputor = account("disputor", i, 0);
        let bond = default_outcome_bond::<T>(i as usize);
        T::AssetManager::deposit(Asset::Ztg, &disputor, bond).unwrap();
        let outcome = OutcomeReport::Scalar((2 + i).into());
        SimpleDisputes::<T>::suggest_outcome(
            RawOrigin::Signed(disputor).into(),
            market_id,
            outcome,
        )
        .unwrap();
        <frame_system::Pallet<T>>::set_block_number(now.saturating_add(T::BlockNumber::one()));
    }
}

benchmarks! {
    suggest_outcome {
        let d in 1..(T::MaxDisputes::get() - 1);
        let r in 1..63;
        let e in 1..63;

        let caller: T::AccountId = whitelisted_caller();
        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();

        fill_disputes::<T>(market_id, d);
        let disputes = Disputes::<T>::get(market_id);
        let last_dispute = disputes.last().unwrap();
        let auto_resolve = last_dispute.at.saturating_add(market.deadlines.dispute_duration);
        for i in 0..r {
            let id = T::MarketCommons::push_market(market_mock::<T>()).unwrap();
            T::DisputeResolution::add_auto_resolve(&id, auto_resolve).unwrap();
        }

        let now = <frame_system::Pallet<T>>::block_number();

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

    on_dispute_weight {
        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();
    }: {
        SimpleDisputes::<T>::on_dispute(&market_id, &market).unwrap();
    }

    on_resolution_weight {
        let d in 1..T::MaxDisputes::get();

        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();

        fill_disputes::<T>(market_id, d);
    }: {
        SimpleDisputes::<T>::on_resolution(&market_id, &market).unwrap();
    }

    exchange_weight {
        let d in 1..T::MaxDisputes::get();

        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();

        fill_disputes::<T>(market_id, d);

        let outcome = OutcomeReport::Scalar(1);
        let imb = NegativeImbalanceOf::<T>::zero();
    }: {
        SimpleDisputes::<T>::exchange(&market_id, &market, &outcome, imb).unwrap();
    }

    get_auto_resolve_weight {
        let d in 1..T::MaxDisputes::get();

        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();

        fill_disputes::<T>(market_id, d);
    }: {
        SimpleDisputes::<T>::get_auto_resolve(&market_id, &market);
    }

    has_failed_weight {
        let d in 1..T::MaxDisputes::get();

        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();

        fill_disputes::<T>(market_id, d);
    }: {
        SimpleDisputes::<T>::has_failed(&market_id, &market).unwrap();
    }

    on_global_dispute_weight {
        let d in 1..T::MaxDisputes::get();

        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();

        fill_disputes::<T>(market_id, d);
    }: {
        SimpleDisputes::<T>::on_global_dispute(&market_id, &market).unwrap();
    }

    clear_weight {
        let d in 1..T::MaxDisputes::get();

        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();

        fill_disputes::<T>(market_id, d);
    }: {
        SimpleDisputes::<T>::clear(&market_id, &market).unwrap();
    }

    impl_benchmark_test_suite!(
        SimpleDisputes,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
