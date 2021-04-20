#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[cfg(test)]
use crate::Pallet as PredictionMarket;
use crate::{
    market::{MarketCreation, MarketEnd},
    Config,
};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller, Vec};
use frame_support::{
    dispatch::UnfilteredDispatchable,
    traits::{Currency, Get},
};
use frame_system::RawOrigin;
use sp_runtime::traits::SaturatedConversion;
use zeitgeist_primitives::BASE;

fn create_market_common_parameters<T: Config>(
    permission: MarketCreation,
) -> (
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
    let creation = permission;
    (caller, oracle, end, metadata, creation)
}

fn create_categorical_market_common<T: Config>(
    permission: MarketCreation,
) -> (T::AccountId, T::MarketId) {
    let (caller, oracle, end, metadata, creation) =
        create_market_common_parameters::<T>(permission);
    let _ = Call::<T>::create_categorical_market(
        oracle,
        end,
        metadata,
        creation,
        T::MaxCategories::get(),
    )
    .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into());
    let marketid = Pallet::<T>::market_count() - 1u32.into();
    (caller, marketid)
}

benchmarks! {
    create_categorical_market {
        let (caller, oracle, end, metadata, creation) =
            create_market_common_parameters::<T>(MarketCreation::Permissionless);
        let categories = T::MaxCategories::get();
    }: _(RawOrigin::Signed(caller), oracle, end, metadata, creation, categories)

    create_scalar_market {
        let (caller, oracle, end, metadata, creation) =
            create_market_common_parameters::<T>(MarketCreation::Permissionless);
        let outcome_range = (0u128, u128::MAX);
    }: _(RawOrigin::Signed(caller), oracle, end, metadata, creation, outcome_range)

    approve_market {
        let (_, marketid) = create_categorical_market_common::<T>(MarketCreation::Advised);
    }: _(RawOrigin::Root, marketid)

    reject_market {
        let (_, marketid) = create_categorical_market_common::<T>(MarketCreation::Advised);
    }: _(RawOrigin::Root, marketid)

    admin_destroy_market {
        let (_, marketid) = create_categorical_market_common::<T>(MarketCreation::Advised);
    }: _(RawOrigin::Root, marketid)

    cancel_pending_market {
        let (caller, marketid) = create_categorical_market_common::<T>(MarketCreation::Advised);
    }: _(RawOrigin::Signed(caller), marketid)

    buy_complete_set {
        // Note: buy_complete_sets weight consumption is dependant on how many assets exists
        // Unfortunately this information can only be retrieved with a storage call, therefore
        // The worst-case scenario is assumed and weight is returned at the end of buy_complete_set
        let (caller, marketid) = create_categorical_market_common::<T>(MarketCreation::Permissionless);
        let amount = BASE * 1_000;
    }: _(RawOrigin::Signed(caller), marketid, amount.saturated_into())
}

impl_benchmark_test_suite!(
    PredictionMarket,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
