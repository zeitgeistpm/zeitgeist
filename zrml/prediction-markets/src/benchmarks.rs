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
use alloc::vec::Vec;
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
    constants::mock::{MaxSwapFee, MinLiquidity, MinWeight, BASE, MILLISECS_PER_BLOCK},
    traits::Swaps,
    types::{
        Asset, Deadlines, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus,
        MarketType, MaxRuntimeUsize, MultiHash, OutcomeReport, PoolStatus, ScoringRule,
        SubsidyUntil,
    },
};
use zrml_market_commons::MarketCommonsPalletApi;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

// Get default values for market creation. Also spawns an account with maximum
// amount of native currency
fn create_market_common_parameters<T: Config>(
    permission: MarketCreation,
) -> Result<
    (T::AccountId, T::AccountId, Deadlines<T::BlockNumber>, MultiHash, MarketCreation),
    &'static str,
> {
    let caller: T::AccountId = whitelisted_caller();
    T::AssetManager::deposit(Asset::Ztg, &caller, (100 * MinLiquidity::get()).saturated_into())
        .unwrap();
    let oracle = caller.clone();
    let deadlines = Deadlines::<T::BlockNumber> {
        grace_period: 1_u32.into(),
        oracle_duration: T::MinOracleDuration::get(),
        dispute_duration: T::MinDisputeDuration::get(),
    };
    let mut metadata = [0u8; 50];
    metadata[0] = 0x15;
    metadata[1] = 0x30;
    let creation = permission;
    Ok((caller, oracle, deadlines, MultiHash::Sha3_384(metadata), creation))
}

// Create a market based on common parameters
fn create_market_common<T: Config>(
    permission: MarketCreation,
    options: MarketType,
    scoring_rule: ScoringRule,
    period: Option<MarketPeriod<T::BlockNumber, MomentOf<T>>>,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let range_start: MomentOf<T> = 100_000u64.saturated_into();
    let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
    let period = period.unwrap_or(MarketPeriod::Timestamp(range_start..range_end));
    let (caller, oracle, deadlines, metadata, creation) =
        create_market_common_parameters::<T>(permission)?;
    Call::<T>::create_market {
        oracle,
        period,
        deadlines,
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

fn create_close_and_report_market<T: Config + pallet_timestamp::Config>(
    permission: MarketCreation,
    options: MarketType,
    outcome: OutcomeReport,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let range_start: MomentOf<T> = 100_000u64.saturated_into();
    let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
    let period = MarketPeriod::Timestamp(range_start..range_end);
    let (caller, market_id) =
        create_market_common::<T>(permission, options, ScoringRule::CPMM, Some(period))?;
    Call::<T>::admin_move_market_to_closed { market_id }
        .dispatch_bypass_filter(T::CloseOrigin::successful_origin())?;
    let market = T::MarketCommons::market(&market_id)?;
    let end: u32 = match market.period {
        MarketPeriod::Timestamp(range) => range.end.saturated_into::<u32>(),
        _ => {
            return Err("MarketPeriod is block_number based");
        }
    };
    let grace_period: u32 =
        (market.deadlines.grace_period.saturated_into::<u32>() + 1) * MILLISECS_PER_BLOCK;
    pallet_timestamp::Pallet::<T>::set_timestamp((end + grace_period).into());
    Call::<T>::report { market_id, outcome }
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    Ok((caller, market_id))
}

// Setup a categorical market for fn `internal_resolve`
fn setup_redeem_shares_common<T: Config + pallet_timestamp::Config>(
    market_type: MarketType,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let (caller, market_id) = create_market_common::<T>(
        MarketCreation::Permissionless,
        market_type.clone(),
        ScoringRule::CPMM,
        None,
    )?;
    let outcome: OutcomeReport;

    if let MarketType::Categorical(categories) = market_type {
        outcome = OutcomeReport::Categorical(categories.saturating_sub(1));
    } else if let MarketType::Scalar(range) = market_type {
        outcome = OutcomeReport::Scalar(*range.end());
    } else {
        panic!("setup_redeem_shares_common: Unsupported market type: {:?}", market_type);
    }

    Pallet::<T>::do_buy_complete_set(
        caller.clone(),
        market_id,
        MinLiquidity::get().saturated_into(),
    )?;
    let close_origin = T::CloseOrigin::successful_origin();
    let resolve_origin = T::ResolveOrigin::successful_origin();
    Call::<T>::admin_move_market_to_closed { market_id }.dispatch_bypass_filter(close_origin)?;
    let market = T::MarketCommons::market(&market_id)?;
    let end: u32 = match market.period {
        MarketPeriod::Timestamp(range) => range.end.saturated_into::<u32>(),
        _ => {
            return Err("MarketPeriod is block_number based");
        }
    };
    let grace_period: u32 =
        (market.deadlines.grace_period.saturated_into::<u32>() + 1) * MILLISECS_PER_BLOCK;
    pallet_timestamp::Pallet::<T>::set_timestamp((end + grace_period).into());
    Call::<T>::report { market_id, outcome }
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    Call::<T>::admin_move_market_to_resolved { market_id }
        .dispatch_bypass_filter(resolve_origin)?;
    Ok((caller, market_id))
}

fn setup_reported_categorical_market_with_pool<T: Config + pallet_timestamp::Config>(
    categories: u32,
    report_outcome: OutcomeReport,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    let (caller, market_id) = create_market_common::<T>(
        MarketCreation::Permissionless,
        MarketType::Categorical(categories.saturated_into()),
        ScoringRule::CPMM,
        None,
    )?;

    let max_swap_fee: BalanceOf<T> = MaxSwapFee::get().saturated_into();
    let min_liquidity: BalanceOf<T> = MinLiquidity::get().saturated_into();
    Call::<T>::buy_complete_set { market_id, amount: min_liquidity }
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    let weight_len: usize = MaxRuntimeUsize::from(categories).into();
    let weights = vec![MinWeight::get(); weight_len];
    Pallet::<T>::deploy_swap_pool_for_market(
        RawOrigin::Signed(caller.clone()).into(),
        market_id,
        max_swap_fee,
        min_liquidity,
        weights,
    )?;

    Call::<T>::admin_move_market_to_closed { market_id }
        .dispatch_bypass_filter(T::CloseOrigin::successful_origin())?;
    let market = T::MarketCommons::market(&market_id)?;
    let end: u32 = match market.period {
        MarketPeriod::Timestamp(range) => range.end.saturated_into::<u32>(),
        _ => {
            return Err("MarketPeriod is block_number based");
        }
    };
    let grace_period: u32 =
        (market.deadlines.grace_period.saturated_into::<u32>() + 1) * MILLISECS_PER_BLOCK;
    pallet_timestamp::Pallet::<T>::set_timestamp((end + grace_period).into());
    Call::<T>::report { market_id, outcome: report_outcome }
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;

    Ok((caller, market_id))
}

benchmarks! {
    where_clause {
        where T : pallet_timestamp::Config,
    }

    admin_destroy_disputed_market{
        // The number of assets.
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        // The number of disputes.
        let d in 1..T::MaxDisputes::get();
        // The number of market ids per open time frame.
        let o in 0..63;
        // The number of market ids per close time frame.
        let c in 0..63;
        // The number of market ids per dispute block.
        let r in 0..63;

        let (caller, market_id) = setup_reported_categorical_market_with_pool::<T>(
            a,
            OutcomeReport::Categorical(0u16),
        )?;

        let pool_id = T::MarketCommons::market_pool(&market_id)?;

        // minimum of number of disputes and (categories - 1) because of `matches_outcome_report`
        for i in 1..=(a - 1).min(d) {
            let outcome = OutcomeReport::Categorical(i.saturated_into());
            let disputor = account("disputor", i, 0);
            let dispute_bond = crate::pallet::default_dispute_bond::<T>(i as usize);
            T::AssetManager::deposit(Asset::Ztg, &disputor, dispute_bond)?;
            let _ = Pallet::<T>::dispute(RawOrigin::Signed(disputor).into(), market_id, outcome)?;
        }

        let market = T::MarketCommons::market(&market_id)?;

        let (range_start, range_end) = match market.period {
            MarketPeriod::Timestamp(range) => (range.start, range.end),
            _ => panic!("admin_destroy_reported_market: Unsupported market period"),
        };

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

        let disputes = Disputes::<T>::get(market_id);
        let last_dispute = disputes.last().unwrap();
        let dispute_at = last_dispute.at;
        for i in 0..r {
            MarketIdsPerDisputeBlock::<T>::try_mutate(
                dispute_at,
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let destroy_origin = T::DestroyOrigin::successful_origin();
        let call = Call::<T>::admin_destroy_market { market_id };
    }: {
        call.dispatch_bypass_filter(destroy_origin)?
    } verify {
        assert_last_event::<T>(Event::MarketDestroyed::<T>(market_id).into());
    }

    admin_destroy_reported_market {
        // The number of assets.
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        // The number of market ids per open time frame.
        let o in 0..63;
        // The number of market ids per close time frame.
        let c in 0..63;
        // The number of market ids per dispute block.
        let r in 0..63;

        let (caller, market_id) = setup_reported_categorical_market_with_pool::<T>(
            a,
            OutcomeReport::Categorical(0u16),
        )?;

        let pool_id = T::MarketCommons::market_pool(&market_id)?;

        let market = T::MarketCommons::market(&market_id)?;

        let (range_start, range_end) = match market.period {
            MarketPeriod::Timestamp(range) => (range.start, range.end),
            _ => panic!("admin_destroy_reported_market: Unsupported market period"),
        };

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

        let report_at = market.report.unwrap().at;
        for i in 0..r {
            MarketIdsPerReportBlock::<T>::try_mutate(
                report_at,
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let destroy_origin = T::DestroyOrigin::successful_origin();
        let call = Call::<T>::admin_destroy_market { market_id };
    }: {
        call.dispatch_bypass_filter(destroy_origin)?
    } verify {
        assert_last_event::<T>(Event::MarketDestroyed::<T>(
            market_id,
        ).into());
    }

    admin_move_market_to_closed {
        let o in 0..63;
        let c in 0..63;

        let range_start: MomentOf<T> = 100_000u64.saturated_into();
        let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..range_end)),
        )?;

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

    admin_move_market_to_resolved_scalar_reported {
        let r in 0..63;

        let (_, market_id) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar(0u128..=u128::MAX),
            OutcomeReport::Scalar(u128::MAX),
        )?;

        let market = T::MarketCommons::market(&market_id)?;

        let report_at = market.report.unwrap().at;
        for i in 0..r {
            MarketIdsPerReportBlock::<T>::try_mutate(
                report_at,
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let close_origin = T::CloseOrigin::successful_origin();
        let call = Call::<T>::admin_move_market_to_resolved { market_id };
    }: {
        call.dispatch_bypass_filter(close_origin)?
    } verify {
        assert_last_event::<T>(Event::MarketResolved::<T>(
            market_id,
            MarketStatus::Resolved,
            OutcomeReport::Scalar(u128::MAX),
        ).into());
    }

    admin_move_market_to_resolved_categorical_reported {
        let r in 0..63;

        let categories = T::MaxCategories::get();
        let (_, market_id) = setup_reported_categorical_market_with_pool::<T>(
            categories.into(),
            OutcomeReport::Categorical(0u16),
        )?;
        T::MarketCommons::mutate_market(&market_id, |market| {
            let admin = account("admin", 0, 0);
            market.dispute_mechanism = MarketDisputeMechanism::Authorized(admin);
            Ok(())
        })?;

        let market = T::MarketCommons::market(&market_id)?;

        let report_at = market.report.unwrap().at;
        for i in 0..r {
            MarketIdsPerReportBlock::<T>::try_mutate(
                report_at,
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let close_origin = T::CloseOrigin::successful_origin();
        let call = Call::<T>::admin_move_market_to_resolved { market_id };
    }: {
        call.dispatch_bypass_filter(close_origin)?
    } verify {
        assert_last_event::<T>(Event::MarketResolved::<T>(
            market_id,
            MarketStatus::Resolved,
            OutcomeReport::Categorical(0u16),
        ).into());
    }

    admin_move_market_to_resolved_scalar_disputed {
        let r in 0..63;
        let d in 1..T::MaxDisputes::get();

        let (_, market_id) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar(0u128..=u128::MAX),
            OutcomeReport::Scalar(u128::MAX),
        )?;

        T::MarketCommons::mutate_market(&market_id, |market| {
            let admin = account("admin", 0, 0);
            market.dispute_mechanism = MarketDisputeMechanism::Authorized(admin);
            Ok(())
        })?;

        let market = T::MarketCommons::market(&market_id)?;
        if let MarketType::Scalar(range) = market.market_type {
            assert!((d as u128) < *range.end());
        } else {
            panic!("Must create scalar market");
        }

        for i in 1..=d {
            let outcome = OutcomeReport::Scalar(i.saturated_into());
            let disputor = account("disputor", i, 0);
            let dispute_bond = crate::pallet::default_dispute_bond::<T>(i as usize);
            T::AssetManager::deposit(
                Asset::Ztg,
                &disputor,
                dispute_bond,
            )?;
            Pallet::<T>::dispute(RawOrigin::Signed(disputor).into(), market_id, outcome)?;
        }
        let disputes = Disputes::<T>::get(market_id);

        let last_dispute = disputes.last().unwrap();
        let dispute_at = last_dispute.at;
        for i in 0..r {
            MarketIdsPerDisputeBlock::<T>::try_mutate(
                dispute_at,
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let close_origin = T::CloseOrigin::successful_origin();
        let call = Call::<T>::admin_move_market_to_resolved { market_id };
    }: {
        call.dispatch_bypass_filter(close_origin)?
    } verify {
        assert_last_event::<T>(Event::MarketResolved::<T>(
            market_id,
            MarketStatus::Resolved,
            OutcomeReport::Scalar(u128::MAX),
        ).into());
    }

    admin_move_market_to_resolved_categorical_disputed {
        let r in 0..63;
        let d in 1..T::MaxDisputes::get();

        let categories = T::MaxCategories::get();
        let (caller, market_id) =
            setup_reported_categorical_market_with_pool::<T>(
                categories.into(),
                OutcomeReport::Categorical(0u16)
            )?;

        T::MarketCommons::mutate_market(&market_id, |market| {
            let admin = account("admin", 0, 0);
            market.dispute_mechanism = MarketDisputeMechanism::Authorized(admin);
            Ok(())
        })?;

        for i in 1..=d {
            let outcome = OutcomeReport::Categorical((i % 2).saturated_into::<u16>());
            let disputor = account("disputor", i, 0);
            let dispute_bond = crate::pallet::default_dispute_bond::<T>(i as usize);
            T::AssetManager::deposit(
                Asset::Ztg,
                &disputor,
                dispute_bond,
            )?;
Pallet::<T>::dispute(RawOrigin::Signed(disputor).into(), market_id, outcome)?;
        }
        let disputes = Disputes::<T>::get(market_id);

        let last_dispute = disputes.last().unwrap();
        let dispute_at = last_dispute.at;
        for i in 0..r {
            MarketIdsPerDisputeBlock::<T>::try_mutate(
                dispute_at,
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let close_origin = T::CloseOrigin::successful_origin();
        let call = Call::<T>::admin_move_market_to_resolved { market_id };
    }: {
        call.dispatch_bypass_filter(close_origin)?
    } verify {
        assert_last_event::<T>(Event::MarketResolved::<T>(
            market_id,
            MarketStatus::Resolved,
            OutcomeReport::Categorical(0u16),
        ).into());
    }

    approve_market {
        let (_, market_id) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            None,
        )?;

        let approve_origin = T::ApproveOrigin::successful_origin();
        let call = Call::<T>::approve_market { market_id };
    }: { call.dispatch_bypass_filter(approve_origin)? }

    buy_complete_set {
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(a.saturated_into()),
            ScoringRule::CPMM,
            None,
        )?;
        let amount = BASE * 1_000;
    }: _(RawOrigin::Signed(caller), market_id, amount.saturated_into())

    // Beware! We're only benchmarking categorical markets (scalar market creation is essentially
    // the same).
    create_market {
        let m in 0..63;

        let (caller, oracle, deadlines, metadata, creation) =
            create_market_common_parameters::<T>(MarketCreation::Permissionless)?;

        let range_end = T::MaxSubsidyPeriod::get();
        let period = MarketPeriod::Timestamp(T::MinSubsidyPeriod::get()..range_end);

        for i in 0..m {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }
    }: _(
            RawOrigin::Signed(caller),
            oracle,
            period,
            deadlines,
            metadata,
            creation,
            MarketType::Categorical(T::MaxCategories::get()),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        )

    deploy_swap_pool_for_market_future_pool {
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        let o in 0..63;

        let range_start: MomentOf<T> = 100_000u64.saturated_into();
        let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(a.saturated_into()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..range_end)),
        )?;

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
        Pallet::<T>::buy_complete_set(
            RawOrigin::Signed(caller.clone()).into(),
            market_id,
            min_liquidity,
        )?;

        let weight_len: usize = MaxRuntimeUsize::from(a).into();
        let weights = vec![MinWeight::get(); weight_len];

        let call = Call::<T>::deploy_swap_pool_for_market {
            market_id,
            swap_fee: max_swap_fee,
            amount: min_liquidity,
            weights,
        };
    }: {
        call.dispatch_bypass_filter(RawOrigin::Signed(caller).into())?;
    } verify {
        let current_len = MarketIdsPerOpenTimeFrame::<T>::get(
            Pallet::<T>::calculate_time_frame_of_moment(range_start),
        )
        .len();
        assert_eq!(current_len, prev_len + 1);
    }

    deploy_swap_pool_for_market_open_pool {
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();

        // We need to ensure, that period range start is now,
        // because we would like to open the pool now
        let range_start: MomentOf<T> = T::MarketCommons::now();
        let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(a.saturated_into()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..range_end)),
        )?;

        let market = T::MarketCommons::market(&market_id.saturated_into())?;

        let max_swap_fee: BalanceOf::<T> = MaxSwapFee::get().saturated_into();
        let min_liquidity: BalanceOf::<T> = MinLiquidity::get().saturated_into();
        Pallet::<T>::buy_complete_set(
            RawOrigin::Signed(caller.clone()).into(),
            market_id,
            min_liquidity,
        )?;

        let weight_len: usize = MaxRuntimeUsize::from(a).into();
        let weights = vec![MinWeight::get(); weight_len];

        let call = Call::<T>::deploy_swap_pool_for_market {
            market_id,
            swap_fee: max_swap_fee,
            amount: min_liquidity,
            weights,
        };
    }: {
        call.dispatch_bypass_filter(RawOrigin::Signed(caller).into())?;
    } verify {
        let market_pool_id = T::MarketCommons::market_pool(&market_id.saturated_into())?;
        let pool = T::Swaps::pool(market_pool_id)?;
        assert_eq!(pool.pool_status, PoolStatus::Active);
    }

    dispute_authorized {
        let d in 0..(T::MaxDisputes::get() - 1) as u32;
        let b in 0..63;

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

        let market = T::MarketCommons::market(&market_id)?;
        if let MarketType::Scalar(range) = market.market_type {
            assert!((d as u128) < *range.end());
        } else {
            panic!("Must create scalar market");
        }
        for i in 0..d {
            let outcome = OutcomeReport::Scalar(i.into());
            let disputor = account("disputor", i, 0);
            T::AssetManager::deposit(Asset::Ztg, &disputor, (u128::MAX).saturated_into())?;
            Pallet::<T>::dispute(RawOrigin::Signed(disputor).into(), market_id, outcome)?;
        }

        let now = frame_system::Pallet::<T>::block_number();
        for i in 0..b {
            MarketIdsPerDisputeBlock::<T>::try_mutate(
                now,
                |ids| ids.try_push(i.into()),
            ).unwrap();
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
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(T::MinSubsidyPeriod::get()..T::MaxSubsidyPeriod::get())),
        )?;
        let market = T::MarketCommons::market(&market_id.saturated_into())?;
    }: { Pallet::<T>::handle_expired_advised_market(&market_id, market)? }

    internal_resolve_categorical_reported {
        let categories = T::MaxCategories::get();
        let (_, market_id) = setup_reported_categorical_market_with_pool::<T>(
            categories.into(),
            OutcomeReport::Categorical(1u16),
        )?;
        T::MarketCommons::mutate_market(&market_id, |market| {
            let admin = account("admin", 0, 0);
            market.dispute_mechanism = MarketDisputeMechanism::Authorized(admin);
            Ok(())
        })?;
        let market = T::MarketCommons::market(&market_id)?;
    }: {
        Pallet::<T>::on_resolution(&market_id, &market)?;
    } verify {
        let market = T::MarketCommons::market(&market_id)?;
        assert_eq!(market.status, MarketStatus::Resolved);
    }

    internal_resolve_categorical_disputed {
        // d = num. disputes
        let d in 0..T::MaxDisputes::get();

        let categories = T::MaxCategories::get();
        let (caller, market_id) =
            setup_reported_categorical_market_with_pool::<T>(
                categories.into(),
                OutcomeReport::Categorical(1u16)
            )?;
        T::MarketCommons::mutate_market(&market_id, |market| {
            let admin = account("admin", 0, 0);
            market.dispute_mechanism = MarketDisputeMechanism::Authorized(admin);
            Ok(())
        })?;

        for i in 0..d {
            let origin = caller.clone();
            Pallet::<T>::dispute(
                RawOrigin::Signed(origin).into(),
                market_id,
                OutcomeReport::Categorical((i % 2).saturated_into::<u16>()),
            )?;
        }
        let market = T::MarketCommons::market(&market_id)?;
    }: {
        Pallet::<T>::on_resolution(&market_id, &market)?;
    } verify {
        let market = T::MarketCommons::market(&market_id)?;
        assert_eq!(market.status, MarketStatus::Resolved);
    }

    internal_resolve_scalar_reported {
        let (caller, market_id) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar(0u128..=u128::MAX),
            OutcomeReport::Scalar(u128::MAX),
        )?;
        let market = T::MarketCommons::market(&market_id)?;
    }: {
        Pallet::<T>::on_resolution(&market_id, &market)?;
    } verify {
        let market = T::MarketCommons::market(&market_id)?;
        assert_eq!(market.status, MarketStatus::Resolved);
    }

    internal_resolve_scalar_disputed {
        let d in 0..T::MaxDisputes::get();

        let (caller, market_id) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar(0u128..=u128::MAX),
            OutcomeReport::Scalar(u128::MAX),
        )?;
        T::MarketCommons::mutate_market(&market_id, |market| {
            let admin = account("admin", 0, 0);
            market.dispute_mechanism = MarketDisputeMechanism::Authorized(admin);
            Ok(())
        })?;
        let market = T::MarketCommons::market(&market_id)?;
        if let MarketType::Scalar(range) = market.market_type {
            assert!((d as u128) < *range.end());
        } else {
            panic!("Must create scalar market");
        }
        for i in 0..d {
            let origin = caller.clone();
            Pallet::<T>::dispute(
                RawOrigin::Signed(origin).into(),
                market_id,
                OutcomeReport::Scalar(i.into())
            )?;
        }
        let market = T::MarketCommons::market(&market_id)?;
    }: {
        Pallet::<T>::on_resolution(&market_id, &market)?;
    } verify {
        let market = T::MarketCommons::market(&market_id)?;
        assert_eq!(market.status, MarketStatus::Resolved);
    }

    on_initialize_resolve_overhead {
        // wait for timestamp to get initialized (that's why block 2)
        let now = 2u64.saturated_into::<T::BlockNumber>();
    }: { Pallet::<T>::on_initialize(now) }

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
        );
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

        let range_start: MomentOf<T> = 100_000u64.saturated_into();
        let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
        let (_, market_id) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..range_end)),
        )?;

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

        // ensure range.start is now to get the heaviest path
        let range_start: MomentOf<T> = T::MarketCommons::now();
        let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..range_end)),
        )?;

        T::MarketCommons::mutate_market(&market_id, |market| {
            // ensure sender is oracle to succeed extrinsic call
            market.oracle = caller.clone();
            Ok(())
        })?;

        let outcome = OutcomeReport::Categorical(0);
        let close_origin = T::CloseOrigin::successful_origin();
        Pallet::<T>::admin_move_market_to_closed(close_origin, market_id)?;
        let market = T::MarketCommons::market(&market_id)?;
        let end : u32 = match market.period {
            MarketPeriod::Timestamp(range) => {
                range.end.saturated_into::<u32>()
            },
            _ => {
                return Err(frame_benchmarking::BenchmarkError::Stop(
                          "MarketPeriod is block_number based"
                        ));
            },
        };
        let grace_period: u32 =
            (market.deadlines.grace_period.saturated_into::<u32>() + 1) * MILLISECS_PER_BLOCK;
        pallet_timestamp::Pallet::<T>::set_timestamp((end + grace_period).into());
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
            ScoringRule::CPMM,
            None,
        )?;
        let amount: BalanceOf<T> = MinLiquidity::get().saturated_into();
        Pallet::<T>::buy_complete_set(
            RawOrigin::Signed(caller.clone()).into(),
            market_id,
            amount,
        )?;
    }: _(RawOrigin::Signed(caller), market_id, amount)

    start_subsidy {
        // Total event outcome assets.
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();

        // Create advised rikiddo market with a assets (advised -> start_subsidy not invoked).
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(a.saturated_into()),
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            Some(MarketPeriod::Timestamp(T::MinSubsidyPeriod::get()..T::MaxSubsidyPeriod::get())),
        )?;
        let mut market_clone = None;
        T::MarketCommons::mutate_market(&market_id, |market| {
            market.status = MarketStatus::CollectingSubsidy;
            market_clone = Some(market.clone());
            Ok(())
        })?;
    }: { Pallet::<T>::start_subsidy(&market_clone.unwrap(), market_id)? }

    market_status_manager {
        let b in 1..31;
        let f in 1..31;

        // ensure markets exist
        let start_block: T::BlockNumber = 100_000u64.saturated_into();
        let end_block: T::BlockNumber = 1_000_000u64.saturated_into();
        for _ in 0..31 {
            create_market_common::<T>(
                MarketCreation::Permissionless,
                MarketType::Categorical(T::MaxCategories::get()),
                ScoringRule::CPMM,
                Some(MarketPeriod::Block(start_block..end_block)),
            ).unwrap();
        }

        let range_start: MomentOf<T> = 100_000u64.saturated_into();
        let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
        for _ in 31..64 {
            create_market_common::<T>(
                MarketCreation::Permissionless,
                MarketType::Categorical(T::MaxCategories::get()),
                ScoringRule::CPMM,
                Some(MarketPeriod::Timestamp(range_start..range_end)),
            ).unwrap();
        }

        let block_number: T::BlockNumber = Zero::zero();
        let last_time_frame: TimeFrame = Zero::zero();
        for i in 1..=b {
            <MarketIdsPerOpenBlock<T>>::try_mutate(block_number, |ids| {
                ids.try_push(i.into())
            }).unwrap();
        }

        let last_offset: TimeFrame = last_time_frame + 1.saturated_into::<u64>();
        //* quadratic complexity should not be allowed in substrate blockchains
        //* assume at first that the last time frame is one block before the current time frame
        let t = 0;
        let current_time_frame: TimeFrame = last_offset + t.saturated_into::<u64>();
        for i in 1..=f {
            <MarketIdsPerOpenTimeFrame<T>>::try_mutate(current_time_frame, |ids| {
                // + 31 to not conflict with the markets of MarketIdsPerOpenBlock
                ids.try_push((i + 31).into())
            }).unwrap();
        }
    }: {
        Pallet::<T>::market_status_manager::<
            _,
            MarketIdsPerOpenBlock<T>,
            MarketIdsPerOpenTimeFrame<T>,
        >(
            block_number,
            last_time_frame,
            current_time_frame,
            |market_id, market| {
                // noop, because weight is already measured somewhere else
                Ok(())
            },
        )
        .unwrap();
    }

    market_resolution_manager {
        let r in 1..31;
        let d in 1..31;

        let range_start: MomentOf<T> = 100_000u64.saturated_into();
        let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
        // ensure markets exist
        for _ in 0..64 {
            let (_, market_id) = create_market_common::<T>(
                MarketCreation::Permissionless,
                MarketType::Categorical(T::MaxCategories::get()),
                ScoringRule::CPMM,
                Some(MarketPeriod::Timestamp(range_start..range_end)),
            )?;
            // ensure market is reported
            T::MarketCommons::mutate_market(&market_id, |market| {
                market.status = MarketStatus::Reported;
                Ok(())
            })?;
        }

        let block_number: T::BlockNumber = Zero::zero();

        let mut r_ids_vec = Vec::new();
        for i in 1..=r {
           r_ids_vec.push(i.into());
        }
        MarketIdsPerReportBlock::<T>::mutate(block_number, |ids| {
            *ids = BoundedVec::try_from(r_ids_vec).unwrap();
        });

        // + 31 to not conflict with the markets of MarketIdsPerReportBlock
        let d_ids_vec = (1..=d).map(|i| (i + 31).into()).collect::<Vec<_>>();
        MarketIdsPerDisputeBlock::<T>::mutate(block_number, |ids| {
            *ids = BoundedVec::try_from(d_ids_vec).unwrap();
        });
    }: {
        Pallet::<T>::resolution_manager(
            block_number,
            |market_id, market| {
                // noop, because weight is already measured somewhere else
                Ok(())
            },
        ).unwrap();
    }

    process_subsidy_collecting_markets_dummy {
        let current_block: T::BlockNumber = 0u64.saturated_into::<T::BlockNumber>();
        let current_time: MomentOf<T> = 0u64.saturated_into::<MomentOf<T>>();
        let markets = BoundedVec::try_from(Vec::new()).unwrap();
        <MarketsCollectingSubsidy<T>>::put(markets);
    }: {
        let _ = <Pallet<T>>::process_subsidy_collecting_markets(current_block, current_time);
    }
}

impl_benchmark_test_suite!(
    PredictionMarket,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
