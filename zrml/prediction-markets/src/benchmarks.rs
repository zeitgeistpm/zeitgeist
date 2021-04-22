#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[cfg(test)]
use crate::Pallet as PredictionMarket;
use crate::{
    market::{MarketCreation, MarketEnd, MarketType, Outcome},
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

fn create_market_common<T: Config>(
    permission: MarketCreation,
    options: MarketType,
) -> (T::AccountId, T::MarketId) {
    let (caller, oracle, end, metadata, creation) =
        create_market_common_parameters::<T>(permission);

    if let MarketType::Categorical(categories) = options {
        let _ = Call::<T>::create_categorical_market(oracle, end, metadata, creation, categories)
            .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into());
    } else if let MarketType::Scalar(range) = options {
        let _ = Call::<T>::create_scalar_market(oracle, end, metadata, creation, range)
            .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into());
    } else {
        panic!(
            "create_market_common: Unsupported market type: {:?}",
            options
        );
    }

    let marketid = Pallet::<T>::market_count() - 1u32.into();
    (caller, marketid)
}

fn create_close_and_report_market<T: Config>(
    permission: MarketCreation,
    options: MarketType,
    outcome: Outcome,
) -> (T::AccountId, T::MarketId) {
    let (caller, marketid) = create_market_common::<T>(permission, options);
    let _ = Call::<T>::admin_move_market_to_closed(marketid)
        .dispatch_bypass_filter(RawOrigin::Root.into());
    let _ = Call::<T>::report(marketid, outcome)
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into());
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
        let (_, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get())
        );
    }: _(RawOrigin::Root, marketid)

    reject_market {
        let (_, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get())
        );
    }: _(RawOrigin::Root, marketid)

    cancel_pending_market {
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get())
        );
    }: _(RawOrigin::Signed(caller), marketid)

    buy_complete_set {
        let a in 0..T::MaxCategories::get() as u32;

        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(a.saturated_into())
        );

        let amount = BASE * 1_000;
    }: _(RawOrigin::Signed(caller), marketid, amount.saturated_into())

    sell_complete_set {
        let a in 0..T::MaxCategories::get() as u32;

        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(a.saturated_into())
        );

        let amount: BalanceOf<T> = (BASE * 1_000).saturated_into();
        let _ = Call::<T>::buy_complete_set(marketid, amount)
            .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into());
    }: _(RawOrigin::Signed(caller), marketid, amount)

    // TODO: logical paths + different asset count benchmarks for admin_*

    /*
    admin_destroy_market{
        let (_, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get())
        );
    }: _(RawOrigin::Root, marketid)
    */

    admin_move_market_to_closed {
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get())
        );
    }: _(RawOrigin::Root, marketid)

    report {
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get())
        );
        let outcome = Outcome::Categorical(0);
        let _ = Call::<T>::admin_move_market_to_closed(marketid)
            .dispatch_bypass_filter(RawOrigin::Root.into());
    }: _(RawOrigin::Signed(caller), marketid, outcome)

    dispute {
        let a in 0..(T::MaxDisputes::get() - 1) as u32;
        let (caller, marketid) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar((0u128, u128::MAX)),
            Outcome::Scalar(42)
        );

        for i in 0..a as u128 {
            let _ = Call::<T>::dispute(marketid, Outcome::Scalar(i))
                .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into());
        }
    }: _(RawOrigin::Signed(caller), marketid, Outcome::Scalar((a + 1) as u128))

    /*
    redeem_shares_categorical {
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get())
        );
        let signed_call = RawOrigin::Signed(caller);
        let _ = Call::<T>::admin_move_market_to_closed(marketid)
            .dispatch_bypass_filter(RawOrigin::Root.into());
        let _ = Call::<T>::report(marketid, Outcome::Categorical(0))
            .dispatch_bypass_filter(signed_call.clone().into());
        // TODO: Resolve
    }: redeem_shares(signed_call, marketid)


    redeem_shares_scalar {
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar((0u128, u128::MAX))
        );
        let signed_call = RawOrigin::Signed(caller);
        let _ = Call::<T>::admin_move_market_to_closed(marketid)
            .dispatch_bypass_filter(RawOrigin::Root.into());
        let _ = Call::<T>::report(marketid, Outcome::Scalar(42))
            .dispatch_bypass_filter(signed_call.clone().into());
        // TODO: Resolve
    }: redeem_shares(signed_call, marketid)

    */
}

impl_benchmark_test_suite!(
    PredictionMarket,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
