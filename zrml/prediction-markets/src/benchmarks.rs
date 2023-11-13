// Copyright 2022-2023 Forecasting Technologies LTD.
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
    clippy::arithmetic_side_effects
)]
#![allow(clippy::type_complexity)]
#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[cfg(test)]
use crate::Pallet as PredictionMarket;
use alloc::vec::Vec;
use frame_benchmarking::{account, benchmarks, vec, whitelisted_caller};
use frame_support::{
    dispatch::UnfilteredDispatchable,
    traits::{EnsureOrigin, Get},
};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::{
    traits::{One, SaturatedConversion, Saturating, Zero},
    Perbill,
};
use zeitgeist_primitives::{
    constants::mock::{
        CloseEarlyProtectionTimeFramePeriod, CloseEarlyTimeFramePeriod, MaxSwapFee, MinWeight,
        BASE, MILLISECS_PER_BLOCK,
    },
    traits::{DisputeApi, Swaps},
    types::{
        Asset, Deadlines, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus,
        MarketType, MaxRuntimeUsize, MultiHash, OutcomeReport, PoolStatus, ScoringRule,
        SubsidyUntil,
    },
};
use zrml_authorized::Pallet as AuthorizedPallet;
use zrml_global_disputes::GlobalDisputesPalletApi;
use zrml_market_commons::MarketCommonsPalletApi;

use frame_support::{traits::Hooks, BoundedVec};

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

const LIQUIDITY: u128 = 100 * BASE;

// Get default values for market creation. Also spawns an account with maximum
// amount of native currency
fn create_market_common_parameters<T: Config>(
    is_disputable: bool,
) -> Result<(T::AccountId, T::AccountId, Deadlines<T::BlockNumber>, MultiHash), &'static str> {
    let caller: T::AccountId = whitelisted_caller();
    T::AssetManager::deposit(Asset::Ztg, &caller, (100u128 * LIQUIDITY).saturated_into()).unwrap();
    let oracle = caller.clone();
    let deadlines = Deadlines::<T::BlockNumber> {
        grace_period: 1_u32.into(),
        oracle_duration: T::MinOracleDuration::get(),
        dispute_duration: if is_disputable { T::MinDisputeDuration::get() } else { Zero::zero() },
    };
    let mut metadata = [0u8; 50];
    metadata[0] = 0x15;
    metadata[1] = 0x30;
    Ok((caller, oracle, deadlines, MultiHash::Sha3_384(metadata)))
}

// Create a market based on common parameters
fn create_market_common<T: Config + pallet_timestamp::Config>(
    creation: MarketCreation,
    options: MarketType,
    scoring_rule: ScoringRule,
    period: Option<MarketPeriod<T::BlockNumber, MomentOf<T>>>,
    dispute_mechanism: Option<MarketDisputeMechanism>,
) -> Result<(T::AccountId, MarketIdOf<T>), &'static str> {
    pallet_timestamp::Pallet::<T>::set_timestamp(0u32.into());
    let range_start: MomentOf<T> = 100_000u64.saturated_into();
    let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
    let creator_fee: Perbill = Perbill::zero();
    let period = period.unwrap_or(MarketPeriod::Timestamp(range_start..range_end));
    let (caller, oracle, deadlines, metadata) =
        create_market_common_parameters::<T>(dispute_mechanism.is_some())?;
    Call::<T>::create_market {
        base_asset: Asset::Ztg,
        creator_fee,
        oracle,
        period,
        deadlines,
        metadata,
        creation,
        market_type: options,
        dispute_mechanism,
        scoring_rule,
    }
    .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    let market_id = <zrml_market_commons::Pallet<T>>::latest_market_id()?;
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
    let (caller, market_id) = create_market_common::<T>(
        permission,
        options,
        ScoringRule::CPMM,
        Some(period),
        Some(MarketDisputeMechanism::Court),
    )?;
    Call::<T>::admin_move_market_to_closed { market_id }
        .dispatch_bypass_filter(T::CloseOrigin::try_successful_origin().unwrap())?;
    let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
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
        Some(MarketDisputeMechanism::Court),
    )?;
    let outcome: OutcomeReport;

    if let MarketType::Categorical(categories) = market_type {
        outcome = OutcomeReport::Categorical(categories.saturating_sub(1));
    } else if let MarketType::Scalar(range) = market_type {
        outcome = OutcomeReport::Scalar(*range.end());
    } else {
        panic!("setup_redeem_shares_common: Unsupported market type: {market_type:?}");
    }

    Call::<T>::buy_complete_set { market_id, amount: LIQUIDITY.saturated_into() }
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    let close_origin = T::CloseOrigin::try_successful_origin().unwrap();
    let resolve_origin = T::ResolveOrigin::try_successful_origin().unwrap();
    Call::<T>::admin_move_market_to_closed { market_id }.dispatch_bypass_filter(close_origin)?;
    let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
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
        Some(MarketDisputeMechanism::Court),
    )?;

    let max_swap_fee: BalanceOf<T> = MaxSwapFee::get().saturated_into();
    let min_liquidity: BalanceOf<T> = LIQUIDITY.saturated_into();
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
        .dispatch_bypass_filter(T::CloseOrigin::try_successful_origin().unwrap())?;
    let market = <zrml_market_commons::Pallet<T>>::market(&market_id)?;
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
        where
            T: pallet_timestamp::Config + zrml_authorized::Config + zrml_simple_disputes::Config + zrml_court::Config,
            <<T as zrml_authorized::Config>::MarketCommons as MarketCommonsPalletApi>::MarketId:
                From<<T as zrml_market_commons::Config>::MarketId>,
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
            Some(MarketDisputeMechanism::Court),
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

        let close_origin = T::CloseOrigin::try_successful_origin().unwrap();
        let call = Call::<T>::admin_move_market_to_closed { market_id };
    }: { call.dispatch_bypass_filter(close_origin)? }

    admin_move_market_to_resolved_scalar_reported {
        let r in 0..63;

        let (_, market_id) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar(0u128..=u128::MAX),
            OutcomeReport::Scalar(u128::MAX),
        )?;

        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;

        let report_at = market.report.unwrap().at;
        let resolves_at = report_at.saturating_add(market.deadlines.dispute_duration);
        for i in 0..r {
            MarketIdsPerReportBlock::<T>::try_mutate(
                resolves_at,
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let close_origin = T::CloseOrigin::try_successful_origin().unwrap();
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
        <zrml_market_commons::Pallet::<T>>::mutate_market(&market_id, |market| {
            market.dispute_mechanism = Some(MarketDisputeMechanism::Authorized);
            Ok(())
        })?;

        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;

        let report_at = market.report.unwrap().at;
        let resolves_at = report_at.saturating_add(market.deadlines.dispute_duration);
        for i in 0..r {
            MarketIdsPerReportBlock::<T>::try_mutate(
                resolves_at,
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let close_origin = T::CloseOrigin::try_successful_origin().unwrap();
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

        let (_, market_id) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar(0u128..=u128::MAX),
            OutcomeReport::Scalar(u128::MAX),
        )?;

        <zrml_market_commons::Pallet::<T>>::mutate_market(&market_id, |market| {
            market.dispute_mechanism = Some(MarketDisputeMechanism::Authorized);
            Ok(())
        })?;

        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;

        let outcome = OutcomeReport::Scalar(0);
        let disputor = account("disputor", 1, 0);
        <T as pallet::Config>::AssetManager::deposit(
            Asset::Ztg,
            &disputor,
            u128::MAX.saturated_into(),
        ).unwrap();
        Pallet::<T>::dispute(RawOrigin::Signed(disputor).into(), market_id)?;

        let now = <frame_system::Pallet<T>>::block_number();
        AuthorizedPallet::<T>::authorize_market_outcome(
            T::AuthorizedDisputeResolutionOrigin::try_successful_origin().unwrap(),
            market_id.into(),
            OutcomeReport::Scalar(0),
        )?;

        let resolves_at = now.saturating_add(<T as zrml_authorized::Config>::CorrectionPeriod::get());
        for i in 0..r {
            MarketIdsPerDisputeBlock::<T>::try_mutate(
                resolves_at,
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let close_origin = T::CloseOrigin::try_successful_origin().unwrap();
        let call = Call::<T>::admin_move_market_to_resolved { market_id };
    }: {
        call.dispatch_bypass_filter(close_origin)?
    } verify {
        assert_last_event::<T>(Event::MarketResolved::<T>(
            market_id,
            MarketStatus::Resolved,
            OutcomeReport::Scalar(0),
        ).into());
    }

    admin_move_market_to_resolved_categorical_disputed {
        let r in 0..63;

        let categories = T::MaxCategories::get();
        let (caller, market_id) =
            setup_reported_categorical_market_with_pool::<T>(
                categories.into(),
                OutcomeReport::Categorical(2)
            )?;

        <zrml_market_commons::Pallet::<T>>::mutate_market(&market_id, |market| {
            market.dispute_mechanism = Some(MarketDisputeMechanism::Authorized);
            Ok(())
        })?;

        let disputor = account("disputor", 1, 0);
        <T as pallet::Config>::AssetManager::deposit(
            Asset::Ztg,
            &disputor,
            u128::MAX.saturated_into(),
        ).unwrap();
        Pallet::<T>::dispute(RawOrigin::Signed(disputor).into(), market_id)?;

        // Authorize the outcome with the highest number of correct reporters to maximize the
        // number of transfers required (0 has (d+1)//2 reports, 1 has d//2 reports).
        AuthorizedPallet::<T>::authorize_market_outcome(
            T::AuthorizedDisputeResolutionOrigin::try_successful_origin().unwrap(),
            market_id.into(),
            OutcomeReport::Categorical(0),
        )?;

        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;
        let now = <frame_system::Pallet<T>>::block_number();
        let resolves_at = now.saturating_add(<T as zrml_authorized::Config>::CorrectionPeriod::get());
        for i in 0..r {
            MarketIdsPerDisputeBlock::<T>::try_mutate(
                resolves_at,
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let close_origin = T::CloseOrigin::try_successful_origin().unwrap();
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
            Some(MarketDisputeMechanism::Court),
        )?;

        let approve_origin = T::ApproveOrigin::try_successful_origin().unwrap();
        let call = Call::<T>::approve_market { market_id };
    }: { call.dispatch_bypass_filter(approve_origin)? }

    request_edit {
        let r in 0..<T as Config>::MaxEditReasonLen::get();
        let (_, market_id) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            None,
            Some(MarketDisputeMechanism::Court),
        )?;

        let approve_origin = T::ApproveOrigin::try_successful_origin().unwrap();
        let edit_reason = vec![0_u8; r as usize];
        let call = Call::<T>::request_edit{ market_id, edit_reason };
    }: { call.dispatch_bypass_filter(approve_origin)? } verify {}

    buy_complete_set {
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(a.saturated_into()),
            ScoringRule::CPMM,
            None,
            Some(MarketDisputeMechanism::Court),
        )?;
        let amount = BASE * 1_000;
    }: _(RawOrigin::Signed(caller), market_id, amount.saturated_into())

    // Beware! We're only benchmarking categorical markets (scalar market creation is essentially
    // the same).
    create_market {
        let m in 0..63;

        let (caller, oracle, deadlines, metadata) = create_market_common_parameters::<T>(true)?;

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
            Asset::Ztg,
            Perbill::zero(),
            oracle,
            period,
            deadlines,
            metadata,
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::CPMM
    )

    edit_market {
        let m in 0..63;

        let market_type = MarketType::Categorical(T::MaxCategories::get());
        let dispute_mechanism = Some(MarketDisputeMechanism::Court);
        let scoring_rule = ScoringRule::CPMM;
        let range_start: MomentOf<T> = 100_000u64.saturated_into();
        let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
        let period = MarketPeriod::Timestamp(range_start..range_end);
        let (caller, oracle, deadlines, metadata) =
            create_market_common_parameters::<T>(true)?;
        Call::<T>::create_market {
            base_asset: Asset::Ztg,
            creator_fee: Perbill::zero(),
            oracle: oracle.clone(),
            period: period.clone(),
            deadlines,
            metadata: metadata.clone(),
            creation: MarketCreation::Advised,
            market_type: market_type.clone(),
            dispute_mechanism: dispute_mechanism.clone(),
            scoring_rule,
        }
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
        let market_id = <zrml_market_commons::Pallet::<T>>::latest_market_id()?;

        let approve_origin = T::ApproveOrigin::try_successful_origin().unwrap();
        let edit_reason = vec![0_u8; 1024];
        Call::<T>::request_edit{ market_id, edit_reason }
        .dispatch_bypass_filter(approve_origin)?;

        for i in 0..m {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }
        let new_deadlines = Deadlines::<T::BlockNumber> {
            grace_period: 2_u32.into(),
            oracle_duration: T::MinOracleDuration::get(),
            dispute_duration: T::MinDisputeDuration::get(),
        };
    }: _(
            RawOrigin::Signed(caller),
            Asset::Ztg,
            market_id,
            oracle,
            period,
            new_deadlines,
            metadata,
            market_type,
            dispute_mechanism,
            scoring_rule
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
            Some(MarketDisputeMechanism::Court),
        )?;

        assert!(
            Pallet::<T>::calculate_time_frame_of_moment(<zrml_market_commons::Pallet::<T>>::now())
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
        let min_liquidity: BalanceOf::<T> = LIQUIDITY.saturated_into();
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
        let range_start: MomentOf<T> = <zrml_market_commons::Pallet::<T>>::now();
        let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(a.saturated_into()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..range_end)),
            Some(MarketDisputeMechanism::Court),
        )?;

        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id.saturated_into())?;

        let max_swap_fee: BalanceOf::<T> = MaxSwapFee::get().saturated_into();
        let min_liquidity: BalanceOf::<T> = LIQUIDITY.saturated_into();
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
        let market_pool_id =
            <zrml_market_commons::Pallet::<T>>::market_pool(&market_id.saturated_into())?;
        let pool = T::Swaps::pool(market_pool_id)?;
        assert_eq!(pool.pool_status, PoolStatus::Active);
    }

    start_global_dispute {
        let m in 1..CacheSize::get();
        let n in 1..CacheSize::get();

        // no benchmarking component for max disputes here,
        // because MaxDisputes is enforced for the extrinsic
        let (caller, market_id) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar(0u128..=u128::MAX),
            OutcomeReport::Scalar(u128::MAX),
        )?;

        <zrml_market_commons::Pallet::<T>>::mutate_market(&market_id, |market| {
            market.dispute_mechanism = Some(MarketDisputeMechanism::Court);
            Ok(())
        })?;

        // first element is the market id from above
        let mut market_ids_1: BoundedVec<MarketIdOf<T>, CacheSize> = Default::default();
        assert_eq!(market_id, 0u128.saturated_into());
        for i in 1..m {
            market_ids_1.try_push(i.saturated_into()).unwrap();
        }

        <zrml_court::Pallet<T>>::on_initialize(1u32.into());
        <frame_system::Pallet<T>>::set_block_number(1u32.into());

        let min_amount = <T as zrml_court::Config>::MinJurorStake::get();
        for i in 0..<zrml_court::Pallet<T>>::necessary_draws_weight(0usize) {
            let juror: T::AccountId = account("Jurori", i.try_into().unwrap(), 0);
            <T as pallet::Config>::AssetManager::deposit(
                Asset::Ztg,
                &juror,
                (u128::MAX / 2).saturated_into(),
            ).unwrap();
            <zrml_court::Pallet<T>>::join_court(
                RawOrigin::Signed(juror.clone()).into(),
                min_amount + i.saturated_into(),
            )?;
        }

        let disputor: T::AccountId = account("Disputor", 1, 0);
        <T as pallet::Config>::AssetManager::deposit(
            Asset::Ztg,
            &disputor,
            u128::MAX.saturated_into(),
        ).unwrap();
        let _ = Call::<T>::dispute {
            market_id,
        }
        .dispatch_bypass_filter(RawOrigin::Signed(disputor).into())?;

        let market = <zrml_market_commons::Pallet<T>>::market(&market_id.saturated_into()).unwrap();
        let appeal_end = T::Court::get_auto_resolve(&market_id, &market).result.unwrap();
        let mut market_ids_2: BoundedVec<MarketIdOf<T>, CacheSize> = BoundedVec::try_from(
            vec![market_id],
        ).unwrap();
        for i in 1..n {
            market_ids_2.try_push(i.saturated_into()).unwrap();
        }
        MarketIdsPerDisputeBlock::<T>::insert(appeal_end, market_ids_2);

        <frame_system::Pallet<T>>::set_block_number(appeal_end - 1u64.saturated_into::<T::BlockNumber>());

        let now = <frame_system::Pallet<T>>::block_number();

        let add_outcome_end = now +
            <T as Config>::GlobalDisputes::get_add_outcome_period();
        let vote_end = add_outcome_end + <T as Config>::GlobalDisputes::get_vote_period();
        // the complexity depends on MarketIdsPerDisputeBlock at the current block
        // this is because a variable number of market ids need to be decoded from the storage
        MarketIdsPerDisputeBlock::<T>::insert(vote_end, market_ids_1);

        let call = Call::<T>::start_global_dispute { market_id };
    }: {
        call.dispatch_bypass_filter(RawOrigin::Signed(caller).into())?;
    }

    dispute_authorized {
        let report_outcome = OutcomeReport::Scalar(u128::MAX);
        let (caller, market_id) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar(0u128..=u128::MAX),
            report_outcome,
        )?;

        <zrml_market_commons::Pallet::<T>>::mutate_market(&market_id, |market| {
            market.dispute_mechanism = Some(MarketDisputeMechanism::Authorized);
            Ok(())
        })?;

        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;

        let call = Call::<T>::dispute { market_id };
    }: {
        call.dispatch_bypass_filter(RawOrigin::Signed(caller).into())?;
    }

    handle_expired_advised_market {
        let (_, market_id) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(T::MinSubsidyPeriod::get()..T::MaxSubsidyPeriod::get())),
            Some(MarketDisputeMechanism::Court),
        )?;
        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id.saturated_into())?;
    }: { Pallet::<T>::handle_expired_advised_market(&market_id, market)? }

    internal_resolve_categorical_reported {
        let categories = T::MaxCategories::get();
        let (_, market_id) = setup_reported_categorical_market_with_pool::<T>(
            categories.into(),
            OutcomeReport::Categorical(1u16),
        )?;
        <zrml_market_commons::Pallet::<T>>::mutate_market(&market_id, |market| {
            market.dispute_mechanism = Some(MarketDisputeMechanism::Authorized);
            Ok(())
        })?;
        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;
    }: {
        Pallet::<T>::on_resolution(&market_id, &market)?;
    } verify {
        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;
        assert_eq!(market.status, MarketStatus::Resolved);
    }

    internal_resolve_categorical_disputed {
        let categories = T::MaxCategories::get();
        let (caller, market_id) =
            setup_reported_categorical_market_with_pool::<T>(
                categories.into(),
                OutcomeReport::Categorical(1u16)
            )?;
        <zrml_market_commons::Pallet::<T>>::mutate_market(&market_id, |market| {
            market.dispute_mechanism = Some(MarketDisputeMechanism::Authorized);
            Ok(())
        })?;

        Pallet::<T>::dispute(
            RawOrigin::Signed(caller).into(),
            market_id,
        )?;

        AuthorizedPallet::<T>::authorize_market_outcome(
            T::AuthorizedDisputeResolutionOrigin::try_successful_origin().unwrap(),
            market_id.into(),
            OutcomeReport::Categorical(0),
        )?;
        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;
    }: {
        Pallet::<T>::on_resolution(&market_id, &market)?;
    } verify {
        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;
        assert_eq!(market.status, MarketStatus::Resolved);
    }

    internal_resolve_scalar_reported {
        let (caller, market_id) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar(0u128..=u128::MAX),
            OutcomeReport::Scalar(u128::MAX),
        )?;
        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;
    }: {
        Pallet::<T>::on_resolution(&market_id, &market)?;
    } verify {
        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;
        assert_eq!(market.status, MarketStatus::Resolved);
    }

    internal_resolve_scalar_disputed {
        let (caller, market_id) = create_close_and_report_market::<T>(
            MarketCreation::Permissionless,
            MarketType::Scalar(0u128..=u128::MAX),
            OutcomeReport::Scalar(u128::MAX),
        )?;
        <zrml_market_commons::Pallet::<T>>::mutate_market(&market_id, |market| {
            market.dispute_mechanism = Some(MarketDisputeMechanism::Authorized);
            Ok(())
        })?;
        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;
        Pallet::<T>::dispute(
            RawOrigin::Signed(caller).into(),
            market_id,
        )?;

        AuthorizedPallet::<T>::authorize_market_outcome(
            T::AuthorizedDisputeResolutionOrigin::try_successful_origin().unwrap(),
            market_id.into(),
            OutcomeReport::Scalar(0),
        )?;
        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;
    }: {
        Pallet::<T>::on_resolution(&market_id, &market)?;
    } verify {
        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;
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
        let r in 0..<T as Config>::MaxRejectReasonLen::get();

        let range_start: MomentOf<T> = 100_000u64.saturated_into();
        let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
        let (_, market_id) = create_market_common::<T>(
            MarketCreation::Advised,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..range_end)),
            Some(MarketDisputeMechanism::Court),
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

        let reject_origin = T::RejectOrigin::try_successful_origin().unwrap();
        let reject_reason: Vec<u8> = vec![0; r as usize];
        let call = Call::<T>::reject_market { market_id, reject_reason };
    }: { call.dispatch_bypass_filter(reject_origin)? }

    report_market_with_dispute_mechanism {
        let m in 0..63;

        // ensure range.start is now to get the heaviest path
        let range_start: MomentOf<T> = <zrml_market_commons::Pallet::<T>>::now();
        let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..range_end)),
            Some(MarketDisputeMechanism::Court),
        )?;

        <zrml_market_commons::Pallet::<T>>::mutate_market(&market_id, |market| {
            // ensure sender is oracle to succeed extrinsic call
            market.oracle = caller.clone();
            Ok(())
        })?;

        let outcome = OutcomeReport::Categorical(0);
        let close_origin = T::CloseOrigin::try_successful_origin().unwrap();
        Pallet::<T>::admin_move_market_to_closed(close_origin, market_id)?;
        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;
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
        let report_at = frame_system::Pallet::<T>::block_number();
        let resolves_at = report_at.saturating_add(market.deadlines.dispute_duration);
        for i in 0..m {
            MarketIdsPerReportBlock::<T>::try_mutate(resolves_at, |ids| {
                ids.try_push(i.into())
            }).unwrap();
        }
        let call = Call::<T>::report { market_id, outcome };
    }: {
        call.dispatch_bypass_filter(RawOrigin::Signed(caller).into())?;
    }

    report_trusted_market {
        pallet_timestamp::Pallet::<T>::set_timestamp(0u32.into());
        let start: MomentOf<T> = <zrml_market_commons::Pallet::<T>>::now();
        let end: MomentOf<T> = 1_000_000u64.saturated_into();
        let (caller, oracle, _, metadata) = create_market_common_parameters::<T>(false)?;
        Call::<T>::create_market {
            base_asset: Asset::Ztg,
            creator_fee: Perbill::zero(),
            oracle: caller.clone(),
            period: MarketPeriod::Timestamp(start..end),
            deadlines: Deadlines::<T::BlockNumber> {
                grace_period: 0u8.into(),
                oracle_duration: T::MinOracleDuration::get(),
                dispute_duration: 0u8.into(),
            },
            metadata,
            creation: MarketCreation::Permissionless,
            market_type: MarketType::Categorical(3),
            dispute_mechanism: None,
            scoring_rule: ScoringRule::CPMM,
        }
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
        let market_id = <zrml_market_commons::Pallet<T>>::latest_market_id()?;
        let close_origin = T::CloseOrigin::try_successful_origin().unwrap();
        Pallet::<T>::admin_move_market_to_closed(close_origin, market_id)?;
        let outcome = OutcomeReport::Categorical(0);
        let call = Call::<T>::report { market_id, outcome };
    }: {
        call.dispatch_bypass_filter(RawOrigin::Signed(caller).into())?;
    }

    sell_complete_set {
        let a in (T::MinCategories::get().into())..T::MaxCategories::get().into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(a.saturated_into()),
            ScoringRule::CPMM,
            None,
            Some(MarketDisputeMechanism::Court),
        )?;
        let amount: BalanceOf<T> = LIQUIDITY.saturated_into();
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
            Some(MarketDisputeMechanism::Court),
        )?;
        let mut market_clone = None;
        <zrml_market_commons::Pallet::<T>>::mutate_market(&market_id, |market| {
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
                Some(MarketDisputeMechanism::Court),
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
                Some(MarketDisputeMechanism::Court),
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
                Some(MarketDisputeMechanism::Court),
            )?;
            // ensure market is reported
            <zrml_market_commons::Pallet::<T>>::mutate_market(&market_id, |market| {
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

    schedule_early_close_as_authority {
        let o in 0..63;
        let n in 0..63;

        let range_start: MomentOf<T> = 0u64.saturated_into();
        let old_range_end: MomentOf<T> = 100_000_000_000u64.saturated_into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..old_range_end)),
            Some(MarketDisputeMechanism::Court),
        )?;

        for i in 0..o {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(old_range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let now_time = <zrml_market_commons::Pallet::<T>>::now();
        let new_range_end: MomentOf<T> = now_time + CloseEarlyProtectionTimeFramePeriod::get();

        for i in 0..n {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(new_range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let close_origin = T::CloseMarketEarlyOrigin::try_successful_origin().unwrap();
        let call = Call::<T>::schedule_early_close { market_id };
    }: { call.dispatch_bypass_filter(close_origin)? }

    schedule_early_close_after_dispute {
        let o in 0..63;
        let n in 0..63;

        let range_start: MomentOf<T> = 0u64.saturated_into();
        let old_range_end: MomentOf<T> = 100_000_000_000u64.saturated_into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..old_range_end)),
            Some(MarketDisputeMechanism::Court),
        )?;

        for i in 0..o {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(old_range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let now_time = <zrml_market_commons::Pallet::<T>>::now();
        let new_range_end: MomentOf<T> = now_time + CloseEarlyProtectionTimeFramePeriod::get();

        for i in 0..n {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(new_range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        Pallet::<T>::schedule_early_close(
            RawOrigin::Signed(caller.clone()).into(),
            market_id,
        )?;

        Pallet::<T>::dispute_early_close(
            RawOrigin::Signed(caller.clone()).into(),
            market_id,
        )?;

        let close_origin = T::CloseMarketEarlyOrigin::try_successful_origin().unwrap();
        let call = Call::<T>::schedule_early_close { market_id };
    }: { call.dispatch_bypass_filter(close_origin)? }

    schedule_early_close_as_market_creator {
        let o in 0..63;
        let n in 0..63;

        let range_start: MomentOf<T> = 0u64.saturated_into();
        let old_range_end: MomentOf<T> = 100_000_000_000u64.saturated_into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..old_range_end)),
            Some(MarketDisputeMechanism::Court),
        )?;

        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;
        let market_creator = market.creator.clone();

        for i in 0..o {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(old_range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let now_time = <zrml_market_commons::Pallet::<T>>::now();
        let new_range_end: MomentOf<T> = now_time + CloseEarlyTimeFramePeriod::get();

        for i in 0..n {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(new_range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let origin = RawOrigin::Signed(market_creator).into();
        let call = Call::<T>::schedule_early_close { market_id };
    }: { call.dispatch_bypass_filter(origin)? }

    dispute_early_close {
        let o in 0..63;
        let n in 0..63;

        let range_start: MomentOf<T> = 0u64.saturated_into();
        let old_range_end: MomentOf<T> = 100_000_000_000u64.saturated_into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..old_range_end)),
            Some(MarketDisputeMechanism::Court),
        )?;

        let market_creator = caller.clone();

        Pallet::<T>::schedule_early_close(
            RawOrigin::Signed(market_creator.clone()).into(),
            market_id,
        )?;

        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;
        let new_range_end = match market.period {
            MarketPeriod::Timestamp(range) => {
                range.end
            },
            _ => {
                return Err(frame_benchmarking::BenchmarkError::Stop(
                          "MarketPeriod is block_number based"
                        ));
            },
        };

        for i in 0..o {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(old_range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        for i in 0..n {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(new_range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let origin = RawOrigin::Signed(market_creator).into();
        let call = Call::<T>::dispute_early_close { market_id };
    }: { call.dispatch_bypass_filter(origin)? }

    reject_early_close_after_authority {
        let o in 0..63;
        let n in 0..63;

        let range_start: MomentOf<T> = 0u64.saturated_into();
        let old_range_end: MomentOf<T> = 100_000_000_000u64.saturated_into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..old_range_end)),
            Some(MarketDisputeMechanism::Court),
        )?;

        let market_creator = caller.clone();

        let close_origin = T::CloseMarketEarlyOrigin::try_successful_origin().unwrap();
        Pallet::<T>::schedule_early_close(
            close_origin.clone(),
            market_id,
        )?;

        let market = <zrml_market_commons::Pallet::<T>>::market(&market_id)?;
        let new_range_end = match market.period {
            MarketPeriod::Timestamp(range) => {
                range.end
            },
            _ => {
                return Err(frame_benchmarking::BenchmarkError::Stop(
                          "MarketPeriod is block_number based"
                        ));
            },
        };

        for i in 0..o {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(old_range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        for i in 0..n {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(new_range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }

        let call = Call::<T>::reject_early_close { market_id };
    }: { call.dispatch_bypass_filter(close_origin)? }

    reject_early_close_after_dispute {
        let range_start: MomentOf<T> = 0u64.saturated_into();
        let old_range_end: MomentOf<T> = 100_000_000_000u64.saturated_into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..old_range_end)),
            Some(MarketDisputeMechanism::Court),
        )?;

        let market_creator = caller.clone();

        Pallet::<T>::schedule_early_close(
            RawOrigin::Signed(market_creator.clone()).into(),
            market_id,
        )?;

        Pallet::<T>::dispute_early_close(
            RawOrigin::Signed(caller.clone()).into(),
            market_id,
        )?;

        let close_origin = T::CloseMarketEarlyOrigin::try_successful_origin().unwrap();

        let call = Call::<T>::reject_early_close { market_id };
    }: { call.dispatch_bypass_filter(close_origin)? }

    close_trusted_market {
        let o in 0..63;
        let c in 0..63;

        let range_start: MomentOf<T> = 100_000u64.saturated_into();
        let range_end: MomentOf<T> = 1_000_000u64.saturated_into();
        let (caller, market_id) = create_market_common::<T>(
            MarketCreation::Permissionless,
            MarketType::Categorical(T::MaxCategories::get()),
            ScoringRule::CPMM,
            Some(MarketPeriod::Timestamp(range_start..range_end)),
            None,
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

        let call = Call::<T>::close_trusted_market { market_id };
    }: { call.dispatch_bypass_filter(RawOrigin::Signed(caller).into())? }

    create_market_and_deploy_pool {
        let m in 0..63; // Number of markets closing on the same block.

        let base_asset = Asset::Ztg;
        let range_start = (5 * MILLISECS_PER_BLOCK) as u64;
        let range_end = (100 * MILLISECS_PER_BLOCK) as u64;
        let period = MarketPeriod::Timestamp(range_start..range_end);
        let market_type = MarketType::Categorical(2);
        let (caller, oracle, deadlines, metadata) = create_market_common_parameters::<T>(true)?;
        let price = (BASE / 2).saturated_into();
        let amount = (10u128 * BASE).saturated_into();

        <T as pallet::Config>::AssetManager::deposit(
            base_asset,
            &caller,
            amount,
        )?;
        for i in 0..m {
            MarketIdsPerCloseTimeFrame::<T>::try_mutate(
                Pallet::<T>::calculate_time_frame_of_moment(range_end),
                |ids| ids.try_push(i.into()),
            ).unwrap();
        }
    }: _(
            RawOrigin::Signed(caller),
            base_asset,
            Perbill::zero(),
            oracle,
            period,
            deadlines,
            metadata,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::Court),
            amount,
            vec![price, price],
            (BASE / 100).saturated_into()
    )

    impl_benchmark_test_suite!(
        PredictionMarket,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
