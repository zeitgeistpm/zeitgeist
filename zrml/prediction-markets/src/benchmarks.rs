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

use super::*;
#[cfg(test)]
use crate::Pallet as PredictionMarket;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller};
use frame_support::{
    dispatch::UnfilteredDispatchable,
    traits::{EnsureOrigin, Get, Hooks},
    BoundedVec,
};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::traits::{One, SaturatedConversion, Zero};
use zeitgeist_primitives::{
    constants::mock::{MaxSwapFee, MinLiquidity, MinWeight, BASE},
    traits::{DisputeApi, Swaps},
    types::{
        Asset, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus, MarketType,
        MaxRuntimeUsize, MultiHash, OutcomeReport, PoolStatus, ScalarPosition, ScoringRule,
        SubsidyUntil,
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
    let _ = T::AssetManager::deposit(Asset::Ztg, &caller, (u128::MAX).saturated_into());
    let oracle = caller.clone();
    let period = MarketPeriod::Timestamp(T::MinSubsidyPeriod::get()..T::MaxSubsidyPeriod::get());
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
    scoring_rule: ScoringRule,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let (caller, oracle, period, metadata, creation) =
        create_market_common_parameters::<T>(permission)?;
    let _ = Call::<T>::create_market {
        oracle,
        period,
        metadata,
        creation,
        market_type: options,
        dispute_mechanism: MarketDisputeMechanism::SimpleDisputes,
        scoring_rule,
    }
    .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    let market_id = T::MarketCommons::latest_market_id()?;
    Ok((caller, market_id))
}

fn create_close_and_report_market<T: Config>(
    permission: MarketCreation,
    options: MarketType,
    outcome: OutcomeReport,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let (caller, market_id) = create_market_common::<T>(permission, options, ScoringRule::CPMM)?;
    let _ = Call::<T>::admin_move_market_to_closed { market_id }
        .dispatch_bypass_filter(T::CloseOrigin::successful_origin())?;
    let _ = Call::<T>::report { market_id, outcome }
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    Ok((caller, market_id))
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
            T::AssetManager::deposit(asset, &acc, min_liquidity)?;
            mut_acc_asset -= 1;
        } else {
            T::AssetManager::deposit(fake_asset, &acc, min_liquidity)?;
        }
    }

    Ok(())
}

// Setup a reported categorical market and create accounts with outcome assets.
fn setup_resolve_common_categorical<T: Config>(
    acc_total: u32,
    acc_asset: u32,
    categories: u16,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let (caller, market_id) = create_close_and_report_market::<T>(
        MarketCreation::Permissionless,
        MarketType::Categorical(categories),
        OutcomeReport::Categorical(categories.saturating_sub(1)),
    )?;
    generate_accounts_with_assets::<T>(
        acc_total,
        acc_asset,
        Asset::CategoricalOutcome(market_id, categories.saturating_sub(1)),
    )?;
    Ok((caller, market_id))
}

// Setup a disputed categorical market and create accounts with outcome assets.
fn setup_resolve_common_categorical_after_dispute<T: Config>(
    acc_total: u32,
    acc_asset: u32,
    categories: u16,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let (caller, market_id) =
        setup_resolve_common_categorical::<T>(acc_total, acc_asset, categories)?;
    let _ = Call::<T>::dispute { market_id, outcome: OutcomeReport::Categorical(0) }
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    Ok((caller, market_id))
}

// Setup a categorical market for fn `internal_resolve`
fn setup_redeem_shares_common<T: Config>(
    market_type: MarketType,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let (caller, market_id) = create_market_common::<T>(
        MarketCreation::Permissionless,
        market_type.clone(),
        ScoringRule::CPMM,
    )?;
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
        market_id,
        MinLiquidity::get().saturated_into(),
    )?;
    let close_origin = T::CloseOrigin::successful_origin();
    let resolve_origin = T::ResolveOrigin::successful_origin();
    let _ = Call::<T>::admin_move_market_to_closed { market_id }
        .dispatch_bypass_filter(close_origin)?;
    let _ = Call::<T>::report { market_id, outcome }
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    let _ = Call::<T>::admin_move_market_to_resolved { market_id }
        .dispatch_bypass_filter(resolve_origin)?;
    Ok((caller, market_id))
}

// Setup a reported scalar market and create accounts with outcome assets.
fn setup_resolve_common_scalar<T: Config>(
    acc_total: u32,
    acc_asset: u32,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let (caller, market_id) = create_close_and_report_market::<T>(
        MarketCreation::Permissionless,
        MarketType::Scalar(0u128..=u128::MAX),
        OutcomeReport::Scalar(u128::MAX),
    )?;
    generate_accounts_with_assets::<T>(
        acc_total,
        acc_asset,
        Asset::ScalarOutcome(market_id, ScalarPosition::Long),
    )?;
    Ok((caller, market_id))
}

// Setup a disputed scalar market and create accounts with outcome assets.
fn setup_resolve_common_scalar_after_dispute<T: Config>(
    acc_total: u32,
    acc_asset: u32,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let (caller, market_id) = setup_resolve_common_scalar::<T>(acc_total, acc_asset)?;
    let _ = Call::<T>::dispute { market_id, outcome: OutcomeReport::Scalar(1u128) }
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    Ok((caller, market_id))
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
        let (caller, market_id) = setup_resolve_common_categorical::<T>(a, b, c_u16)?;

        for i in 0..c.min(T::MaxDisputes::get()) {
            let origin = caller.clone();
            let disputes = crate::Disputes::<T>::get(market_id);
            let market = T::MarketCommons::market(&Default::default()).unwrap();
            T::SimpleDisputes::on_dispute(&disputes, &market_id, &market)?;
        }

        let destroy_origin = T::DestroyOrigin::successful_origin();
        let call = Call::<T>::admin_destroy_market { market_id };
    }: { call.dispatch_bypass_filter(destroy_origin)? }

    admin_destroy_reported_market{
        // a = total accounts
        let a in 0..10;
        // b = num. accounts with assets
        let b in 0..10;
        // c = num. asset types
        let c in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        // Complexity: c*a + c*b ∈ O(a)

        let c_u16 = c.saturated_into();
        let (caller, market_id) = setup_resolve_common_categorical::<T>(a, b, c_u16)?;
        let destroy_origin = T::DestroyOrigin::successful_origin();
        let call = Call::<T>::admin_destroy_market { market_id };
    }: { call.dispatch_bypass_filter(destroy_origin)? }

    admin_move_market_to_closed {
        let o in 0..63;
        let c in 0..63;

        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM
        )?;

        let range_start = T::MinSubsidyPeriod::get();
        let range_end = T::MaxSubsidyPeriod::get();
        T::MarketCommons::mutate_market(&market_id, |market| {
            market.period = MarketPeriod::Timestamp(range_start..range_end);
            Ok(())
        })?;

        for i in 0..o {
            MarketIdsPerOpenTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(range_start),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        for i in 0..c {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let close_origin = T::CloseOrigin::successful_origin();
        let call = Call::<T>::admin_move_market_to_closed { market_id };
    }: { call.dispatch_bypass_filter(close_origin)? }

    // This benchmark measures the cost of fn `admin_move_market_to_resolved`
    // and assumes a scalar market is used. The default cost for this function
    // is the resulting weight from this benchmark minus the weight for
    // fn `internal_resolve` of a reported and non-disputed scalar market.
    admin_move_market_to_resolved_overhead {
        let total_accounts = 10u32;
        let asset_accounts = 10u32;
        let (_, market_id) = setup_resolve_common_scalar::<T>(total_accounts, asset_accounts)?;
        let close_origin = T::CloseOrigin::successful_origin();
        let call = Call::<T>::admin_move_market_to_resolved { market_id };
    }: { call.dispatch_bypass_filter(close_origin)? }

    approve_market {
        let (_, market_id) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM
        )?;

        let approve_origin = T::ApproveOrigin::successful_origin();
        let call = Call::<T>::approve_market { market_id };
    }: { call.dispatch_bypass_filter(approve_origin)? }

    buy_complete_set {
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(a.saturated_into()),
            ScoringRule::CPMM
        )?;
        let amount = BASE * 1_000;
    }: _(RawOrigin::Signed(caller), market_id, amount.saturated_into())

    // Beware! We're only benchmarking categorical markets (scalar market creation is essentially
    // the same).
    create_market {
        let m in 0..63;

        let (caller, oracle, _, metadata, creation) =
            create_market_common_parameters::<T>(MarketCreation::Permissionless)?;

        let range_end = T::MaxSubsidyPeriod::get();
        let period = MarketPeriod::Timestamp(T::MinSubsidyPeriod::get()..range_end);

        for i in 0..m {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }
    }: _(RawOrigin::Signed(caller), oracle, period, metadata, creation,
            MarketType::Categorical(T::MaxCategories::get()),
            MarketDisputeMechanism::SimpleDisputes, ScoringRule::CPMM)

    deploy_swap_pool_for_market_future_pool {
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        let o in 0..63;

        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(a.saturated_into()),
            ScoringRule::CPMM
        )?;

        let range_start = T::MinSubsidyPeriod::get();
        let range_end = T::MaxSubsidyPeriod::get();
        T::MarketCommons::mutate_market(&market_id, |market| {
            market.period = MarketPeriod::Timestamp(range_start..range_end);
            Ok(())
        })?;

        assert!(
            Pallet::<T>::calculate_time_frame_of_moment(T::MarketCommons::now())
                < Pallet::<T>::calculate_time_frame_of_moment(range_start)
        );

        for i in 0..o {
            MarketIdsPerOpenTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(range_start),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let prev_len = MarketIdsPerOpenTimeFrame::<T>::get(
            Pallet::<T>::calculate_time_frame_of_moment(range_start)).len();

        let max_swap_fee: BalanceOf::<T> = MaxSwapFee::get().saturated_into();
        let min_liquidity: BalanceOf::<T> = MinLiquidity::get().saturated_into();
        let _ = Pallet::<T>::buy_complete_set(RawOrigin::Signed(caller.clone()).into(),
            market_id, min_liquidity).unwrap();

        let weight_len: usize = MaxRuntimeUsize::from(a).into();
        let weights = vec![MinWeight::get(); weight_len];

        let call = Call::<T>::deploy_swap_pool_for_market { market_id, swap_fee: max_swap_fee,
            amount: min_liquidity, weights };
    }: {
        call.dispatch_bypass_filter(RawOrigin::Signed(caller).into())?;
    } verify {
        let current_len = MarketIdsPerOpenTimeFrame::<T>::get(
            Pallet::<T>::calculate_time_frame_of_moment(range_start)).len();
        assert!(current_len == prev_len + 1);
    }

    deploy_swap_pool_for_market_open_pool {
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();

        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(a.saturated_into()),
            ScoringRule::CPMM
        )?;

        T::MarketCommons::mutate_market(&market_id, |market| {
            // We need to ensure, that period range start is now, because we would like to open the pool now
            market.period = MarketPeriod::Timestamp(T::MarketCommons::now()..T::MaxSubsidyPeriod::get());
            Ok(())
        })?;

        let market = T::MarketCommons::market(&market_id.saturated_into()).unwrap();

        let max_swap_fee: BalanceOf::<T> = MaxSwapFee::get().saturated_into();
        let min_liquidity: BalanceOf::<T> = MinLiquidity::get().saturated_into();
        let _ = Pallet::<T>::buy_complete_set(RawOrigin::Signed(caller.clone()).into(),
            market_id, min_liquidity).unwrap();

        let weight_len: usize = MaxRuntimeUsize::from(a).into();
        let weights = vec![MinWeight::get(); weight_len];

        let call = Call::<T>::deploy_swap_pool_for_market { market_id, swap_fee: max_swap_fee,
            amount: min_liquidity, weights };
    }: {
        call.dispatch_bypass_filter(RawOrigin::Signed(caller).into())?;
    } verify {
        let market_pool_id = T::MarketCommons::market_pool(&market_id.saturated_into()).unwrap();
        let pool = T::Swaps::pool(market_pool_id).unwrap();
        assert!(pool.pool_status == PoolStatus::Active);
    }

    dispute_authorized {
        let d in 0..(T::MaxDisputes::get() - 1) as u32;

        let report_outcome = OutcomeReport::Scalar(u128::MAX);
        let (caller, market_id) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar(0u128..=u128::MAX),
            report_outcome,
        )?;

        T::MarketCommons::mutate_market(&market_id, |market| {
            let admin = account("admin", 0, 0);
            market.dispute_mechanism = MarketDisputeMechanism::Authorized(admin);
            Ok(())
        })?;

        for i in 0..d {
            let outcome = OutcomeReport::Scalar(i.into());
            let disputor = account("disputor", i, 0);
            let _ = T::AssetManager::deposit(Asset::Ztg, &disputor, (u128::MAX).saturated_into());
            let _ = Pallet::<T>::dispute(RawOrigin::Signed(disputor).into(), market_id, outcome).unwrap();
        }

        let dispute_outcome = OutcomeReport::Scalar((d + 1).into());
        let call = Call::<T>::dispute { market_id, outcome: dispute_outcome };
    }: {
        call.dispatch_bypass_filter(RawOrigin::Signed(caller).into())?;
    }

    handle_expired_advised_market {
        let (_, market_id) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM
        )?;
        let market = T::MarketCommons::market(&market_id.saturated_into()).unwrap();
    }: { Pallet::<T>::handle_expired_advised_market(&market_id, market)? }

    internal_resolve_categorical_reported {
        // a = total accounts
        let a in 0..10;
        // b = num. accounts with assets
        let b in 0..10;
        // c = num. asset types
        let c in (T::MinCategories::get().into())..T::MaxCategories::get().into();

        let c_u16 = c.saturated_into();
        let (_, market_id) = setup_resolve_common_categorical_after_dispute::<T>(a, b, c_u16)?;
    }: {
        let market = T::MarketCommons::market(&market_id)?;
        let disputes = crate::Disputes::<T>::get(market_id);
        T::SimpleDisputes::on_resolution(&disputes, &market_id, &market)?
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
        let (caller, market_id) = setup_resolve_common_categorical_after_dispute::<T>(a, b, c_u16)?;

        for i in 0..c.min(d) {
            let origin = caller.clone();
            let disputes = crate::Disputes::<T>::get(market_id);
            let market = T::MarketCommons::market(&Default::default()).unwrap();
            T::SimpleDisputes::on_dispute(&disputes, &market_id, &market)?;
        }
    }: {
        let market = T::MarketCommons::market(&market_id)?;
        let disputes = crate::Disputes::<T>::get(market_id);
        T::SimpleDisputes::on_resolution(&disputes, &market_id, &market)?
    }

    internal_resolve_scalar_reported {
        let total_accounts = 10u32;
        let asset_accounts = 10u32;
        let (_, market_id) = setup_resolve_common_scalar_after_dispute::<T>(total_accounts, asset_accounts)?;
    }: {
        let market = T::MarketCommons::market(&market_id)?;
        let disputes = crate::Disputes::<T>::get(market_id);
        T::SimpleDisputes::on_resolution(&disputes, &market_id, &market)?
    }

    internal_resolve_scalar_disputed {
        let total_accounts = 10u32;
        let asset_accounts = 10u32;
        let d in 0..T::MaxDisputes::get();

        let (caller, market_id) = setup_resolve_common_scalar_after_dispute::<T>(total_accounts, asset_accounts)?;

        for i in 0..d {
            let disputes = crate::Disputes::<T>::get(market_id);
            let origin = caller.clone();
            let market = T::MarketCommons::market(&Default::default()).unwrap();
            T::SimpleDisputes::on_dispute(&disputes, &market_id, &market)?;
        }
    }: {
        let market = T::MarketCommons::market(&market_id)?;
        let disputes = crate::Disputes::<T>::get(market_id);
        T::SimpleDisputes::on_resolution(&disputes, &market_id, &market)?
    }

    // This benchmark measures the cost of fn `on_initialize` minus the resolution.
    on_initialize_resolve_overhead {
        let starting_block = frame_system::Pallet::<T>::block_number() + T::DisputePeriod::get();
    }: { Pallet::<T>::on_initialize(starting_block * 2u32.into()) }

    // Benchmark iteration and market validity check without ending subsidy / discarding market.
    process_subsidy_collecting_markets_raw {
        // Number of markets collecting subsidy.
        let a in 0..10;

        let market_info = SubsidyUntil {
            market_id: MarketIdOf::<T>::zero(),
            period: MarketPeriod::Block(T::BlockNumber::one()..T::BlockNumber::one())
        };

        let markets = BoundedVec::try_from(vec![market_info; a as usize]).unwrap();
        <MarketsCollectingSubsidy<T>>::put(markets);
    }: {
        Pallet::<T>::process_subsidy_collecting_markets(
            T::BlockNumber::zero(),
            MomentOf::<T>::zero()
        )
    }

    redeem_shares_categorical {
        let (caller, market_id) = setup_redeem_shares_common::<T>(
            MarketType::Categorical(T::MaxCategories::get())
        )?;
    }: redeem_shares(RawOrigin::Signed(caller), market_id)

    redeem_shares_scalar {
        let (caller, market_id) = setup_redeem_shares_common::<T>(
            MarketType::Scalar(0u128..=u128::MAX)
        )?;
    }: redeem_shares(RawOrigin::Signed(caller), market_id)

    reject_market {
        let c in 0..63;
        let o in 0..63;

        let (_, market_id) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM
        )?;

        let range_start = T::MinSubsidyPeriod::get();
        let range_end = T::MaxSubsidyPeriod::get();
        T::MarketCommons::mutate_market(&market_id, |market| {
            market.period = MarketPeriod::Timestamp(range_start..range_end);
            Ok(())
        })?;

        for i in 0..o {
            MarketIdsPerOpenTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(range_start),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        for i in 0..c {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let reject_origin = T::RejectOrigin::successful_origin();
        let call = Call::<T>::reject_market { market_id };
    }: { call.dispatch_bypass_filter(reject_origin)? }

    report {
        let m in 0..63;

        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM
        )?;

        T::MarketCommons::mutate_market(&market_id, |market| {
            // ensure range.start is now to get the heaviest path
            market.period = MarketPeriod::Timestamp(T::MarketCommons::now()..T::MaxSubsidyPeriod::get());
            // ensure sender is oracle to succeed extrinsic call
            market.oracle = caller.clone();
            Ok(())
        })?;

        let outcome = OutcomeReport::Categorical(0);
        let close_origin = T::CloseOrigin::successful_origin();
        let _ = Pallet::<T>::admin_move_market_to_closed(close_origin, market_id).unwrap();
        let current_block = frame_system::Pallet::<T>::block_number();
        for i in 0..m {
            MarketIdsPerReportBlock::<T>::try_mutate(current_block, |ids| {
                ids.try_push(i.into())
            }).unwrap();
        }
    }: _(RawOrigin::Signed(caller), market_id, outcome)

    sell_complete_set {
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(a.saturated_into()),
            ScoringRule::CPMM
        )?;
        let amount: BalanceOf<T> = MinLiquidity::get().saturated_into();
        let _ = Pallet::<T>::buy_complete_set(
                RawOrigin::Signed(caller.clone()).into(), market_id, amount).unwrap();
    }: _(RawOrigin::Signed(caller), market_id, amount)

    start_subsidy {
        // Total event outcome assets.
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();

        // Create advised rikiddo market with a assets (advised -> start_subsidy not invoked).
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(a.saturated_into()),
            ScoringRule::RikiddoSigmoidFeeMarketEma
        )?;
        let mut market_clone = None;
        T::MarketCommons::mutate_market(&market_id, |market| {
            market.status = MarketStatus::CollectingSubsidy;
            market_clone = Some(market.clone());
            Ok(())
        })?;
    }: { Pallet::<T>::start_subsidy(&market_clone.unwrap(), market_id)? }
}

impl_benchmark_test_suite!(
    PredictionMarket,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
