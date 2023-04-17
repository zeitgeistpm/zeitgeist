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
#![cfg(feature = "runtime-benchmarks")]

use crate::{
    market_mock, AuthorizedOutcomeReports, Call, Config, NegativeImbalanceOf, Pallet as Authorized,
    Pallet,
};
use frame_benchmarking::benchmarks;
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

benchmarks! {
    authorize_market_outcome_first_report {
        let m in 1..63;

        let origin = T::AuthorizedDisputeResolutionOrigin::successful_origin();
        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market).unwrap();

        frame_system::Pallet::<T>::set_block_number(42u32.into());
        let now = frame_system::Pallet::<T>::block_number();
        let correction_period_ends_at = now.saturating_add(T::CorrectionPeriod::get());
        for _ in 1..=m {
            let id = T::MarketCommons::push_market(market_mock::<T>()).unwrap();
            T::DisputeResolution::add_auto_resolve(&id, correction_period_ends_at).unwrap();
        }

        let call = Call::<T>::authorize_market_outcome {
            market_id,
            outcome: OutcomeReport::Scalar(1),
        };
    }: {
        call.dispatch_bypass_filter(origin)?
    } verify {
        let report = AuthorityReport {
            resolve_at: correction_period_ends_at,
            outcome: OutcomeReport::Scalar(1)
        };
        assert_eq!(AuthorizedOutcomeReports::<T>::get(market_id).unwrap(), report);
    }

    authorize_market_outcome_existing_report {
        let origin = T::AuthorizedDisputeResolutionOrigin::successful_origin();
        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market).unwrap();

        frame_system::Pallet::<T>::set_block_number(42u32.into());

        let now = frame_system::Pallet::<T>::block_number();
        let resolve_at = now.saturating_add(T::CorrectionPeriod::get());

        let report = AuthorityReport { resolve_at, outcome: OutcomeReport::Scalar(0) };
        AuthorizedOutcomeReports::<T>::insert(market_id, report);

        let now = frame_system::Pallet::<T>::block_number();
        frame_system::Pallet::<T>::set_block_number(now + 42u32.into());

        let call = Call::<T>::authorize_market_outcome {
            market_id,
            outcome: OutcomeReport::Scalar(1),
        };
    }: {
        call.dispatch_bypass_filter(origin)?
    } verify {
        let report = AuthorityReport { resolve_at, outcome: OutcomeReport::Scalar(1) };
        assert_eq!(AuthorizedOutcomeReports::<T>::get(market_id).unwrap(), report);
    }

    on_dispute_weight {
        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();
    }: {
        Authorized::<T>::on_dispute(&market_id, &market).unwrap();
    }

    on_resolution_weight {
        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();

        frame_system::Pallet::<T>::set_block_number(42u32.into());

        let now = frame_system::Pallet::<T>::block_number();
        let resolve_at = now.saturating_add(T::CorrectionPeriod::get());

        let report = AuthorityReport { resolve_at, outcome: OutcomeReport::Scalar(0) };
        AuthorizedOutcomeReports::<T>::insert(market_id, report);
    }: {
        Authorized::<T>::on_resolution(&market_id, &market).unwrap();
    }

    exchange_weight {
        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();

        let outcome = OutcomeReport::Scalar(0);
        let imb = NegativeImbalanceOf::<T>::zero();
    }: {
        Authorized::<T>::exchange(&market_id, &market, &outcome, imb).unwrap();
    }

    get_auto_resolve_weight {
        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();

        let now = frame_system::Pallet::<T>::block_number();
        let resolve_at = now.saturating_add(T::CorrectionPeriod::get());

        let report = AuthorityReport { resolve_at, outcome: OutcomeReport::Scalar(0) };
        AuthorizedOutcomeReports::<T>::insert(market_id, report);
    }: {
        Authorized::<T>::get_auto_resolve(&market_id, &market).unwrap();
    }

    has_failed_weight {
        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();
    }: {
        Authorized::<T>::has_failed(&market_id, &market).unwrap();
    }

    on_global_dispute_weight {
        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();
    }: {
        Authorized::<T>::on_global_dispute(&market_id, &market).unwrap();
    }

    clear_weight {
        let market_id = 0u32.into();
        let market = market_mock::<T>();
        T::MarketCommons::push_market(market.clone()).unwrap();

        let now = frame_system::Pallet::<T>::block_number();
        let resolve_at = now.saturating_add(T::CorrectionPeriod::get());

        let report = AuthorityReport { resolve_at, outcome: OutcomeReport::Scalar(0) };
        AuthorizedOutcomeReports::<T>::insert(market_id, report);
    }: {
        Authorized::<T>::clear(&market_id, &market).unwrap();
    }

    impl_benchmark_test_suite!(
        Authorized,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
