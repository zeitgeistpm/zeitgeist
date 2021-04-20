#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[cfg(test)]
use crate::Pallet as PredictionMarket;
use crate::{
    market::{MarketCreation, MarketEnd},
    Config,
};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller, Vec};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;

fn create_market_common_parameters<T: Config>() -> (
    T::AccountId,
    T::AccountId,
    MarketEnd<T::BlockNumber>,
    Vec<u8>,
    MarketCreation,
) {
    let caller: T::AccountId = whitelisted_caller();
    let _ = T::Currency::deposit_creating(&caller, (u128::MAX).saturated_into());
    let oracle = caller.clone();
    let end = <MarketEnd<T::BlockNumber>>::Block((u128::MAX).saturated_into());
    let metadata = <Vec<u8>>::new();
    let creation = MarketCreation::Permissionless;
    (caller, oracle, end, metadata, creation)
}

benchmarks! {
    create_categorical_market {
        let (caller, oracle, end, metadata, creation) = create_market_common_parameters::<T>();
        let categories = T::MaxCategories::get();
    }: _(RawOrigin::Signed(caller), oracle, end, metadata, creation, categories)

    create_scalar_market {
        let (caller, oracle, end, metadata, creation) = create_market_common_parameters::<T>();
        let categories = (0u128, u128::MAX);
    }: _(RawOrigin::Signed(caller), oracle, end, metadata, creation, categories)
}

impl_benchmark_test_suite!(
    PredictionMarket,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
