#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[cfg(test)]
use crate::Pallet as PredictionMarket;
use crate::{
    market::{MarketCreation, MarketEnd, MarketType, Outcome},
    Config,
};
use frame_benchmarking::{
    account, benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller, Vec,
};
use frame_support::{
    dispatch::UnfilteredDispatchable,
    traits::{Currency, Get},
};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::traits::SaturatedConversion;
use zeitgeist_primitives::{Asset, BASE, MIN_LIQUIDITY, MIN_WEIGHT};

fn create_market_common_parameters<T: Config>(
    permission: MarketCreation,
) -> Result<
    (
        T::AccountId,
        T::AccountId,
        MarketEnd<T::BlockNumber>,
        Vec<u8>,
        MarketCreation,
    ),
    &'static str,
> {
    let caller: T::AccountId = whitelisted_caller();
    let _ = T::Currency::deposit_creating(&caller, (u128::MAX).saturated_into());
    let oracle = caller.clone();
    let end = <MarketEnd<T::BlockNumber>>::Block((u128::MAX).saturated_into());
    let metadata = <Vec<u8>>::new();
    let creation = permission;
    Ok((caller, oracle, end, metadata, creation))
}

fn create_market_common<T: Config>(
    permission: MarketCreation,
    options: MarketType,
) -> Result<(T::AccountId, T::MarketId), &'static str> {
    let (caller, oracle, end, metadata, creation) =
        create_market_common_parameters::<T>(permission)?;

    if let MarketType::Categorical(categories) = options {
        let _ = Call::<T>::create_categorical_market(oracle, end, metadata, creation, categories)
            .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    } else if let MarketType::Scalar(range) = options {
        let _ = Call::<T>::create_scalar_market(oracle, end, metadata, creation, range)
            .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    } else {
        panic!(
            "create_market_common: Unsupported market type: {:?}",
            options
        );
    }

    let marketid = Pallet::<T>::market_count() - 1u32.into();
    Ok((caller, marketid))
}

fn create_close_and_report_market<T: Config>(
    permission: MarketCreation,
    options: MarketType,
    outcome: Outcome,
) -> Result<(T::AccountId, T::MarketId), &'static str> {
    let (caller, marketid) = create_market_common::<T>(permission, options)?;
    let _ = Call::<T>::admin_move_market_to_closed(marketid)
        .dispatch_bypass_filter(RawOrigin::Root.into())?;
    let _ = Call::<T>::report(marketid, outcome)
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    Ok((caller, marketid))
}

benchmarks! {
    create_categorical_market {
        let (caller, oracle, end, metadata, creation) =
            create_market_common_parameters::<T>(MarketCreation::Permissionless)?;
        let categories = T::MaxCategories::get();
    }: _(RawOrigin::Signed(caller), oracle, end, metadata, creation, categories)

    create_scalar_market {
        let (caller, oracle, end, metadata, creation) =
            create_market_common_parameters::<T>(MarketCreation::Permissionless)?;
        let outcome_range = (0u128, u128::MAX);
    }: _(RawOrigin::Signed(caller), oracle, end, metadata, creation, outcome_range)

    approve_market {
        let (_, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get())
        )?;
    }: _(RawOrigin::Root, marketid)

    reject_market {
        let (_, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get())
        )?;
    }: _(RawOrigin::Root, marketid)

    cancel_pending_market {
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get())
        )?;
    }: _(RawOrigin::Signed(caller), marketid)

    buy_complete_set {
        let a in 0..T::MaxCategories::get() as u32;
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(a.saturated_into())
        )?;
        let amount = BASE * 1_000;
    }: _(RawOrigin::Signed(caller), marketid, amount.saturated_into())

    sell_complete_set {
        let a in 0..T::MaxCategories::get() as u32;
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(a.saturated_into())
        )?;
        let amount: BalanceOf<T> = (BASE * 1_000).saturated_into();
        let _ = Call::<T>::buy_complete_set(marketid, amount)
            .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    }: _(RawOrigin::Signed(caller), marketid, amount)

    /*
    admin_destroy_disputed_market{
        let (_, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get())
        )?;
    }: admin_destroy_market(RawOrigin::Root, marketid)
    */

    admin_destroy_reported_market{
        // a = num. accounts with shares
        let a in 0..100;
        let (caller, marketid) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get()),
            Outcome::Categorical(0)
        )?;

        let min_liquidity: BalanceOf<T> = MIN_LIQUIDITY.saturated_into();

        for i in 0..a {
            let acc = account("AssetHolder", i, 0);
            let _ = T::Shares::deposit(
                Asset::CategoricalOutcome(marketid, 0),
                &acc,
                min_liquidity
            )?;
        }
    }: admin_destroy_market(RawOrigin::Root, marketid)

    admin_move_market_to_closed {
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get())
        )?;
    }: _(RawOrigin::Root, marketid)

    report {
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get())
        )?;
        let outcome = Outcome::Categorical(0);
        let _ = Call::<T>::admin_move_market_to_closed(marketid)
            .dispatch_bypass_filter(RawOrigin::Root.into())?;
    }: _(RawOrigin::Signed(caller), marketid, outcome)

    dispute {
        let a in 0..(T::MaxDisputes::get() - 1) as u32;
        let (caller, marketid) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar((0u128, u128::MAX)),
            Outcome::Scalar(42)
        )?;

        for i in 0..a as u128 {
            let _ = Call::<T>::dispute(marketid, Outcome::Scalar(i))
                .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
        }
    }: _(RawOrigin::Signed(caller), marketid, Outcome::Scalar((a + 1) as u128))

    deploy_swap_pool_for_market {
        let a in 0..T::MaxCategories::get() as u32;
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(a.saturated_into())
        )?;
        let _ = Pallet::<T>::do_buy_complete_set(caller.clone(), marketid, MIN_LIQUIDITY.saturated_into())?;
        let weights = vec![MIN_WEIGHT; (a + 1) as usize];
    }: _(RawOrigin::Signed(caller), marketid, weights)

    /*
    redeem_shares_categorical {
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get())
        );
        let signed_call = RawOrigin::Signed(caller);
        Call::<T>::admin_move_market_to_closed(marketid)
            .dispatch_bypass_filter(RawOrigin::Root.into())?;
        Call::<T>::report(marketid, Outcome::Categorical(0))
            .dispatch_bypass_filter(signed_call.clone().into())?;
        // TODO: Resolve
    }: redeem_shares(signed_call, marketid)


    redeem_shares_scalar {
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar((0u128, u128::MAX))
        );
        let signed_call = RawOrigin::Signed(caller);
        Call::<T>::admin_move_market_to_closed(marketid)
            .dispatch_bypass_filter(RawOrigin::Root.into())?;
        Call::<T>::report(marketid, Outcome::Scalar(42))
            .dispatch_bypass_filter(signed_call.clone().into())?;
        // TODO: Resolve
    }: redeem_shares(signed_call, marketid)

    */
}

impl_benchmark_test_suite!(
    PredictionMarket,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
