#![allow(
    // Auto-generated code is a no man's land
    clippy::integer_arithmetic
)]
#![cfg(feature = "runtime-benchmarks")]

#[cfg(test)]
use crate::Pallet as Court;
use crate::{market_mock, Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use zeitgeist_primitives::types::OutcomeReport;
use zrml_market_commons::MarketCommonsPalletApi;

benchmarks! {
    authorize_market_outcome {
        let caller: T::AccountId = whitelisted_caller();
        let market = market_mock::<T>(caller.clone());
        T::MarketCommons::push_market(market).unwrap();
    }: _(RawOrigin::Signed(caller), 0u32.into(), OutcomeReport::Scalar(1))
}

impl_benchmark_test_suite!(Court, crate::mock::ExtBuilder::default().build(), crate::mock::Runtime);
