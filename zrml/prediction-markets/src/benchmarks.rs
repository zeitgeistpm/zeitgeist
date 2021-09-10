#![allow(
    // Auto-generated code is a no man's land
    clippy::integer_arithmetic
)]
#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Config;
#[cfg(test)]
use crate::Pallet as PredictionMarket;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller};
use frame_support::{
    dispatch::UnfilteredDispatchable,
    traits::{Currency, EnsureOrigin, Get, Hooks},
};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::traits::SaturatedConversion;
use zeitgeist_primitives::{
    constants::{MinLiquidity, MinWeight, BASE},
    traits::DisputeApi,
    types::{
        Asset, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketType, MultiHash,
        OutcomeReport, ScalarPosition,
    },
};
use zrml_market_commons::MarketCommonsPalletApi;

// Get default values for market creation. Also spawns an account with maximum
// amount of native currency
fn create_market_common_parameters<T: Config>(
    permission: MarketCreation,
) -> Result<
    (
        T::AccountId,
        T::AccountId,
        MarketPeriod<T::BlockNumber, MomentOf<T>>,
        MultiHash,
        MarketCreation,
    ),
    &'static str,
> {
    let caller: T::AccountId = whitelisted_caller();
    let _ = CurrencyOf::<T>::deposit_creating(&caller, (u128::MAX).saturated_into());
    let oracle = caller.clone();
    let period = MarketPeriod::Block(0u128.saturated_into()..u128::MAX.saturated_into());
    let mut metadata = [0u8; 50];
    metadata[0] = 0x15;
    metadata[1] = 0x30;
    let creation = permission;
    Ok((caller, oracle, period, MultiHash::Sha3_384(metadata), creation))
}

// Create a market based on common parameters
fn create_market_common<T: Config>(
    permission: MarketCreation,
    options: MarketType,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let (caller, oracle, period, metadata, creation) =
        create_market_common_parameters::<T>(permission)?;

    if let MarketType::Categorical(categories) = options {
        let _ = Call::<T>::create_categorical_market(
            oracle,
            period,
            metadata,
            creation,
            categories,
            MarketDisputeMechanism::SimpleDisputes,
        )
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    } else if let MarketType::Scalar(range) = options {
        let _ = Call::<T>::create_scalar_market(
            oracle,
            period,
            metadata,
            creation,
            range,
            MarketDisputeMechanism::SimpleDisputes,
        )
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    } else {
        panic!("create_market_common: Unsupported market type: {:?}", options);
    }

    let marketid = T::MarketCommons::latest_market_id()?;
    Ok((caller, marketid))
}

fn create_close_and_report_market<T: Config>(
    permission: MarketCreation,
    options: MarketType,
    outcome: OutcomeReport,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let (caller, marketid) = create_market_common::<T>(permission, options)?;
    let _ = Call::<T>::admin_move_market_to_closed(marketid)
        .dispatch_bypass_filter(T::ApprovalOrigin::successful_origin())?;
    let _ = Call::<T>::report(marketid, outcome)
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    Ok((caller, marketid))
}

// Generates `acc_total` accounts, of which `acc_asset` account do own `asset`
fn generate_accounts_with_assets<T: Config>(
    acc_total: u32,
    acc_asset: u32,
    asset: Asset<MarketIdOf<T>>,
) -> Result<(), &'static str> {
    let min_liquidity: BalanceOf<T> = MinLiquidity::get().saturated_into();
    let fake_asset = Asset::CategoricalOutcome::<MarketIdOf<T>>(u128::MAX.saturated_into(), 0);
    let mut mut_acc_asset = acc_asset;

    for i in 0..acc_total {
        let acc = account("AssetHolder", i, 0);
        if mut_acc_asset > 0 {
            let _ = T::Shares::deposit(asset, &acc, min_liquidity)?;
            mut_acc_asset -= 1;
        } else {
            let _ = T::Shares::deposit(fake_asset, &acc, min_liquidity)?;
        }
    }

    Ok(())
}

// Setup a categorical market for fn `internal_resolve`
fn setup_resolve_common_categorical<T: Config>(
    acc_total: u32,
    acc_asset: u32,
    categories: u16,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let (caller, marketid) = create_close_and_report_market::<T>(
        MarketCreation::Permissionless,
        MarketType::Categorical(categories),
        OutcomeReport::Categorical(categories.saturating_sub(1)),
    )?;
    let _ = generate_accounts_with_assets::<T>(
        acc_total,
        acc_asset,
        Asset::CategoricalOutcome(marketid, categories.saturating_sub(1)),
    )?;
    Ok((caller, marketid))
}

// Setup a categorical market for fn `internal_resolve`
fn setup_redeem_shares_common<T: Config>(
    market_type: MarketType,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let (caller, marketid) =
        create_market_common::<T>(MarketCreation::Permissionless, market_type.clone())?;
    let outcome: OutcomeReport;

    if let MarketType::Categorical(categories) = market_type {
        outcome = OutcomeReport::Categorical(categories.saturating_sub(1));
    } else if let MarketType::Scalar(range) = market_type {
        outcome = OutcomeReport::Scalar(*range.end());
    } else {
        panic!("setup_redeem_shares_common: Unsupported market type: {:?}", market_type);
    }

    let _ = Pallet::<T>::do_buy_complete_set(
        caller.clone(),
        marketid,
        MinLiquidity::get().saturated_into(),
    )?;
    let approval_origin = T::ApprovalOrigin::successful_origin();
    let _ = Call::<T>::admin_move_market_to_closed(marketid)
        .dispatch_bypass_filter(approval_origin.clone())?;
    let _ = Call::<T>::report(marketid, outcome)
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    let _ = Call::<T>::admin_move_market_to_resolved(marketid)
        .dispatch_bypass_filter(approval_origin)?;
    Ok((caller, marketid))
}

// Setup a scalar market for fn `internal_resolve`
fn setup_resolve_common_scalar<T: Config>(
    acc_total: u32,
    acc_asset: u32,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let (caller, marketid) = create_close_and_report_market::<T>(
        MarketCreation::Permissionless,
        MarketType::Scalar(0u128..=u128::MAX),
        OutcomeReport::Scalar(u128::MAX),
    )?;
    let _ = generate_accounts_with_assets::<T>(
        acc_total,
        acc_asset,
        Asset::ScalarOutcome(marketid, ScalarPosition::Long),
    )?;
    Ok((caller, marketid))
}

benchmarks! {
    admin_destroy_disputed_market{
        // a = total accounts
        // An higher number increases the benchmark runtime significantly, while increasing the
        // error due to a lower number of repetitions (the data points that are used to approximate
        // the weight function weight(a) are less precise).
        // The required weight per account is a linear function of degree 1, i.e.
        // fn(a) = ba^1 + ca^0. The first few data points of the curve are already approximating
        // the function and its constant gradient fairly well, provided that these few data points
        // are very precise (many repetitions). If the gradient is well estimated, any value can be
        // derived, up to infinity, assuming that at no point the function fn(a) is non-linear.
        let a in 0..10;
        // b = num. accounts with assets
        // Unfortunately frame-benchmarking does not allow to b = b.min(a) here
        let b in 0..10;
        // c = num. asset types
        let c in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        // Complexity: c*a + c*b ∈ O(a)
        let c_u16 = c.saturated_into();
        let (caller, marketid) = setup_resolve_common_categorical::<T>(a, b, c_u16)?;

        for i in 0..c.min(T::MaxDisputes::get()) {
            let origin = caller.clone();
            let disputes = crate::Disputes::<T>::get(&marketid);
            let _ = T::SimpleDisputes::on_dispute(Default::default(), &disputes, &marketid, &origin)?;
        }

        let approval_origin = T::ApprovalOrigin::successful_origin();
        let call = Call::<T>::admin_destroy_market(marketid);
    }: { call.dispatch_bypass_filter(approval_origin)? }

    admin_destroy_reported_market{
        // a = total accounts
        let a in 0..10;
        // b = num. accounts with assets
        let b in 0..10;
        // c = num. asset types
        let c in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        // Complexity: c*a + c*b ∈ O(a)

        let c_u16 = c.saturated_into();
        let (caller, marketid) = setup_resolve_common_categorical::<T>(a, b, c_u16)?;
        let approval_origin = T::ApprovalOrigin::successful_origin();
        let call = Call::<T>::admin_destroy_market(marketid);
    }: { call.dispatch_bypass_filter(approval_origin)? }

    admin_move_market_to_closed {
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get())
        )?;
        let approval_origin = T::ApprovalOrigin::successful_origin();
        let call = Call::<T>::admin_move_market_to_closed(marketid);
    }: { call.dispatch_bypass_filter(approval_origin)? }

    // This benchmark measures the cost of fn `admin_move_market_to_resolved`
    // and assumes a scalar market is used. The default cost for this function
    // is the resulting weight from this benchmark minus the weight for
    // fn `internal_resolve` of a reported and non-disputed scalar market.
    admin_move_market_to_resolved_overhead {
        let total_accounts = 10u32;
        let asset_accounts = 10u32;
        let (_, marketid) = setup_resolve_common_scalar::<T>(total_accounts, asset_accounts)?;
        let approval_origin = T::ApprovalOrigin::successful_origin();
        let call = Call::<T>::admin_move_market_to_resolved(marketid);
    }: { call.dispatch_bypass_filter(approval_origin)? }

    approve_market {
        let (_, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get())
        )?;

        let origin = T::ApprovalOrigin::successful_origin();
        let call = Call::<T>::approve_market(marketid);
    }: { call.dispatch_bypass_filter(origin)? }

    buy_complete_set {
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(a.saturated_into())
        )?;
        let amount = BASE * 1_000;
    }: _(RawOrigin::Signed(caller), marketid, amount.saturated_into())

    cancel_pending_market {
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get())
        )?;
    }: _(RawOrigin::Signed(caller), marketid)

    create_categorical_market {
        let (caller, oracle, period, metadata, creation) =
            create_market_common_parameters::<T>(MarketCreation::Permissionless)?;
        let categories = T::MaxCategories::get();
    }: _(RawOrigin::Signed(caller), oracle, period, metadata, creation, categories, MarketDisputeMechanism::SimpleDisputes)

    create_scalar_market {
        let (caller, oracle, period, metadata, creation) =
            create_market_common_parameters::<T>(MarketCreation::Permissionless)?;
        let outcome_range = 0u128..=u128::MAX;
    }: _(RawOrigin::Signed(caller), oracle, period, metadata, creation, outcome_range, MarketDisputeMechanism::SimpleDisputes)

    deploy_swap_pool_for_market {
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(a.saturated_into())
        )?;
        let min_liquidity: BalanceOf::<T> = MinLiquidity::get().saturated_into();
        let _ = Call::<T>::buy_complete_set(marketid, min_liquidity)
            .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;

        let weight_len: usize = MaxRuntimeUsize::from(a).into();
        let weights = vec![MinWeight::get(); weight_len.saturating_add(1)];
    }: _(RawOrigin::Signed(caller), marketid, weights)

    dispute {
        let a in 0..(T::MaxDisputes::get() - 1) as u32;
        let (caller, marketid) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar(0u128..=u128::MAX),
            OutcomeReport::Scalar(42)
        )?;
    }:  {
        let origin = caller.clone();
        let disputes = crate::Disputes::<T>::get(&marketid);
        let _ = T::SimpleDisputes::on_dispute(Default::default(), &disputes, &marketid, &origin)?;
    }

    internal_resolve_categorical_reported {
        // a = total accounts
        let a in 0..10;
        // b = num. accounts with assets
        let b in 0..10;
        // c = num. asset types
        let c in (T::MinCategories::get().into())..T::MaxCategories::get().into();

        let c_u16 = c.saturated_into();
        let (_, marketid) = setup_resolve_common_categorical::<T>(a, b, c_u16)?;
    }: {
        let market = T::MarketCommons::market(&marketid)?;
        let disputes = crate::Disputes::<T>::get(&marketid);
        T::SimpleDisputes::on_resolution(&default_dispute_bond::<T>, &disputes, &marketid, &market)?
    }

    internal_resolve_categorical_disputed {
        // a = total accounts
        let a in 0..10;
        // b = num. accounts with assets
        let b in 0..10;
        // c = num. asset types
        let c in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        // d = num. disputes
        let d in 0..T::MaxDisputes::get();

        let c_u16 = c.saturated_into();
        let (caller, marketid) = setup_resolve_common_categorical::<T>(a, b, c_u16)?;

        for i in 0..c.min(d) {
            let origin = caller.clone();
            let disputes = crate::Disputes::<T>::get(&marketid);
            let _ = T::SimpleDisputes::on_dispute(Default::default(), &disputes, &marketid, &origin)?;
        }
    }: {
        let market = T::MarketCommons::market(&marketid)?;
        let disputes = crate::Disputes::<T>::get(&marketid);
        T::SimpleDisputes::on_resolution(&default_dispute_bond::<T>, &disputes, &marketid, &market)?
    }

    internal_resolve_scalar_reported {
        let total_accounts = 10u32;
        let asset_accounts = 10u32;
        let (_, marketid) = setup_resolve_common_scalar::<T>(total_accounts, asset_accounts)?;
    }: {
        let market = T::MarketCommons::market(&marketid)?;
        let disputes = crate::Disputes::<T>::get(&marketid);
        T::SimpleDisputes::on_resolution(&default_dispute_bond::<T>, &disputes, &marketid, &market)?
    }

    internal_resolve_scalar_disputed {
        let total_accounts = 10u32;
        let asset_accounts = 10u32;
        let d in 0..T::MaxDisputes::get();

        let (caller, marketid) = setup_resolve_common_scalar::<T>(total_accounts, asset_accounts)?;

        for i in 0..d {
            let disputes = crate::Disputes::<T>::get(&marketid);
            let origin = caller.clone();
            let _ = T::SimpleDisputes::on_dispute(Default::default(), &disputes, &marketid, &origin)?;
        }
    }: {
        let market = T::MarketCommons::market(&marketid)?;
        let disputes = crate::Disputes::<T>::get(&marketid);
        T::SimpleDisputes::on_resolution(&default_dispute_bond::<T>, &disputes, &marketid, &market)?
    }

    // This benchmark measures the cost of fn `on_initialize` minus the resolution.
    on_initialize_resolve_overhead {
        let starting_block = frame_system::Pallet::<T>::block_number() + T::DisputePeriod::get();
    }: { Pallet::<T>::on_initialize(starting_block * 2u32.into()) }

    redeem_shares_categorical {
        let (caller, marketid) = setup_redeem_shares_common::<T>(
            MarketType::Categorical(T::MaxCategories::get())
        )?;
    }: redeem_shares(RawOrigin::Signed(caller), marketid)

    redeem_shares_scalar {
        let (caller, marketid) = setup_redeem_shares_common::<T>(
            MarketType::Scalar(0u128..=u128::MAX)
        )?;
    }: redeem_shares(RawOrigin::Signed(caller), marketid)

    reject_market {
        let (_, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get())
        )?;
        let approval_origin = T::ApprovalOrigin::successful_origin();
        let call = Call::<T>::reject_market(marketid);
    }: { call.dispatch_bypass_filter(approval_origin)? }

    report {
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get())
        )?;
        let outcome = OutcomeReport::Categorical(0);
        let approval_origin = T::ApprovalOrigin::successful_origin();
        let _ = Call::<T>::admin_move_market_to_closed(marketid)
            .dispatch_bypass_filter(approval_origin)?;
    }: _(RawOrigin::Signed(caller), marketid, outcome)

    sell_complete_set {
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(a.saturated_into())
        )?;
        let amount: BalanceOf<T> = MinLiquidity::get().saturated_into();
        let _ = Call::<T>::buy_complete_set(marketid, amount)
            .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    }: _(RawOrigin::Signed(caller), marketid, amount)
}

impl_benchmark_test_suite!(
    PredictionMarket,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
