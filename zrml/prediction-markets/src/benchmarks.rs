#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::{
    Config,
    market::{MarketCreation, MarketEnd}
};
#[cfg(test)]
use crate::Pallet as PredictionMarket;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller, Vec};
use frame_support::traits::{Get, Currency};
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;

benchmarks! {
    create_categorical_market {
        let caller: T::AccountId = whitelisted_caller();
        let _ = T::Currency::deposit_creating(&caller, (u128::MAX).saturated_into());
        let oracle = caller.clone();
        let end = <MarketEnd<T::BlockNumber>>::Block((u128::MAX).saturated_into());
        let metadata = <Vec<u8>>::new();
        let creation = MarketCreation::Permissionless;
        let categories = T::MaxCategories::get();
    }: _(RawOrigin::Signed(caller), oracle, end, metadata, creation, categories)
}

impl_benchmark_test_suite!(
    PredictionMarket,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
