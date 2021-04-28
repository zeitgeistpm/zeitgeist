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
    traits::{Currency, Get, Hooks},
};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::traits::SaturatedConversion;
use zeitgeist_primitives::{Asset, ScalarPosition, BASE, MIN_LIQUIDITY, MIN_WEIGHT};

// Get default values for market creation. Also spawns an account with maximum
// amount of native currency
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

// Create a market based on common parameters
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

// Generates `acc_total` accounts, of which `acc_asset` account do own `asset`
fn generate_accounts_with_assets<T: Config>(
    acc_total: u32,
    acc_asset: u32,
    asset: Asset<T::MarketId>,
) -> Result<(), &'static str> {
    let min_liquidity: BalanceOf<T> = MIN_LIQUIDITY.saturated_into();
    let fake_asset = Asset::CategoricalOutcome::<T::MarketId>(u128::MAX.saturated_into(), 0);
    let mut mut_acc_asset = acc_asset;

    for i in 0..acc_total {
        if mut_acc_asset > 0 {
            let acc = account("AssetHolder", i, 0);
            let _ = T::Shares::deposit(asset, &acc, min_liquidity)?;
            mut_acc_asset -= 1;
        } else {
            let acc = account("AssetHolder", i, 0);
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
) -> Result<(T::AccountId, T::MarketId), &'static str> {
    let (caller, marketid) = create_close_and_report_market::<T>(
        MarketCreation::Permissionless,
        MarketType::Categorical(categories),
        Outcome::Categorical(categories - 1),
    )?;
    let _ = generate_accounts_with_assets::<T>(
        acc_total,
        acc_asset,
        Asset::CategoricalOutcome(marketid, categories - 1),
    )?;
    Ok((caller, marketid))
}

// Setup a categorical market for fn `internal_resolve`
fn setup_redeem_shares_common<T: Config>(
    market_type: MarketType,
) -> Result<(T::AccountId, T::MarketId), &'static str> {
    let (caller, marketid) =
        create_market_common::<T>(MarketCreation::Permissionless, market_type.clone())?;
    let outcome: Outcome;

    if let MarketType::Categorical(categories) = market_type {
        outcome = Outcome::Categorical(categories - 1);
    } else if let MarketType::Scalar(range) = market_type {
        outcome = Outcome::Scalar(range.1);
    } else {
        panic!(
            "setup_redeem_shares_common: Unsupported market type: {:?}",
            market_type
        );
    }

    let _ =
        Pallet::<T>::do_buy_complete_set(caller.clone(), marketid, MIN_LIQUIDITY.saturated_into())?;
    let _ = Call::<T>::admin_move_market_to_closed(marketid)
        .dispatch_bypass_filter(RawOrigin::Root.into())?;
    let _ = Call::<T>::report(marketid, outcome)
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    let _ = Call::<T>::admin_move_market_to_resolved(marketid)
        .dispatch_bypass_filter(RawOrigin::Root.into())?;
    Ok((caller, marketid))
}

// Setup a scalar market for fn `internal_resolve`
fn setup_resolve_common_scalar<T: Config>(
    acc_total: u32,
    acc_asset: u32,
) -> Result<(T::AccountId, T::MarketId), &'static str> {
    let (caller, marketid) = create_close_and_report_market::<T>(
        MarketCreation::Permissionless,
        MarketType::Scalar((0u128, u128::MAX)),
        Outcome::Scalar(u128::MAX),
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
        let a in 0..10;
        // b = num. accounts with assets
        // Unfortunately frame-benchmarking does not allow to b = b.min(a) here
        let b in 0..10;
        // c = num. asset types
        let c in 0..T::MaxCategories::get() as u32;
        // Complexity: c*a + c*b ∈ O(a)

        let c_u16 = c.saturated_into();
        let (caller, marketid) = setup_resolve_common_categorical::<T>(a, b, c_u16)?;

        for i in 0..c.min(T::MaxDisputes::get() as u32) {
            let _ = Call::<T>::dispute(marketid, Outcome::Categorical(i.saturated_into()))
                .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
        }
    }: admin_destroy_market(RawOrigin::Root, marketid)

    admin_destroy_reported_market{
        // a = total accounts
        let a in 0..10;
        // b = num. accounts with assets
        let b in 0..10;
        // c = num. asset types
        let c in 0..T::MaxCategories::get() as u32;
        // Complexity: c*a + c*b ∈ O(a)

        let c_u16 = c.saturated_into();
        let (caller, marketid) = setup_resolve_common_categorical::<T>(a, b, c_u16)?;
    }: admin_destroy_market(RawOrigin::Root, marketid)

    admin_move_market_to_closed {
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get())
        )?;
    }: _(RawOrigin::Root, marketid)

    // This benchmark measures the cost of fn `admin_move_market_to_resolved`
    // and assumes a scalar market is used. The default cost for this function
    // is the resulting weight from this benchmark minus the weight for
    // fn `internal_resolve` of a reported and non-disputed scalar market.
    admin_move_market_to_resolved_overhead {
        let total_accounts = 10u32;
        let asset_accounts = 10u32;
        let (_, marketid) = setup_resolve_common_scalar::<T>(total_accounts, asset_accounts)?;
    }: admin_move_market_to_resolved(RawOrigin::Root, marketid)

    approve_market {
        let (_, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get())
        )?;
    }: _(RawOrigin::Root, marketid)

    buy_complete_set {
        let a in 0..T::MaxCategories::get() as u32;
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
        let (caller, oracle, end, metadata, creation) =
            create_market_common_parameters::<T>(MarketCreation::Permissionless)?;
        let categories = T::MaxCategories::get();
    }: _(RawOrigin::Signed(caller), oracle, end, metadata, creation, categories)

    create_scalar_market {
        let (caller, oracle, end, metadata, creation) =
            create_market_common_parameters::<T>(MarketCreation::Permissionless)?;
        let outcome_range = (0u128, u128::MAX);
    }: _(RawOrigin::Signed(caller), oracle, end, metadata, creation, outcome_range)

    deploy_swap_pool_for_market {
        let a in 0..T::MaxCategories::get() as u32;
        let (caller, marketid) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(a.saturated_into())
        )?;
        let _ = Pallet::<T>::do_buy_complete_set(
            caller.clone(),
            marketid,
            MIN_LIQUIDITY.saturated_into()
        )?;
        let weights = vec![MIN_WEIGHT; (a + 1) as usize];
    }: _(RawOrigin::Signed(caller), marketid, weights)

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

    internal_resolve_categorical_reported {
        // a = total accounts
        let a in 0..10;
        // b = num. accounts with assets
        let b in 0..10;
        // c = num. asset types
        let c in 0..T::MaxCategories::get() as u32;

        let c_u16 = c.saturated_into();
        let (_, marketid) = setup_resolve_common_categorical::<T>(a, b, c_u16)?;
    }: { Pallet::<T>::internal_resolve(&marketid)? }

    internal_resolve_categorical_disputed {
        // a = total accounts
        let a in 0..10;
        // b = num. accounts with assets
        let b in 0..10;
        // c = num. asset types
        let c in 0..T::MaxCategories::get() as u32;
        // d = num. disputes
        let d in 0..T::MaxDisputes::get() as u32;

        let c_u16 = c.saturated_into();
        let (caller, marketid) = setup_resolve_common_categorical::<T>(a, b, c_u16)?;

        for i in 0..c.min(d) {
            let _ = Call::<T>::dispute(marketid, Outcome::Categorical(i.saturated_into()))
                .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
        }
    }: { Pallet::<T>::internal_resolve(&marketid)? }

    internal_resolve_scalar_reported {
        let total_accounts = 10u32;
        let asset_accounts = 10u32;
        let (_, marketid) = setup_resolve_common_scalar::<T>(total_accounts, asset_accounts)?;
    }: { Pallet::<T>::internal_resolve(&marketid)? }

    internal_resolve_scalar_disputed {
        let total_accounts = 10u32;
        let asset_accounts = 10u32;
        let d in 0..T::MaxDisputes::get() as u32;

        let (caller, marketid) = setup_resolve_common_scalar::<T>(total_accounts, asset_accounts)?;

        for i in 0..d.into() {
            let _ = Call::<T>::dispute(marketid, Outcome::Scalar(i))
                .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
        }
    }: { Pallet::<T>::internal_resolve(&marketid)? }

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
            MarketType::Scalar((0u128, u128::MAX))
        )?;
    }: redeem_shares(RawOrigin::Signed(caller), marketid)

    reject_market {
        let (_, marketid) = create_market_common::<T>(
            MarketCreation::Advised,
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
}

impl_benchmark_test_suite!(
    PredictionMarket,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
