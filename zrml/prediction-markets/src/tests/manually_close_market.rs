// Copyright 2022-2024 Forecasting Technologies LTD.
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

use super::*;

use crate::{LastTimeFrame, MarketIdsPerCloseTimeFrame};
use zeitgeist_primitives::constants::MILLISECS_PER_BLOCK;

// TODO(#1239) MarketDoesNotExist
// TODO(#1239) MarketPeriodEndNotAlreadyReachedYet

#[test]
fn manually_close_market_after_long_stall() {
    // We check that `on_market_close` works correctly even if a block takes much longer than 12sec
    // to be produced and multiple markets are involved.
    ExtBuilder::default().build().execute_with(|| {
        // Mock last time frame to prevent it from defaulting.
        LastTimeFrame::<Runtime>::set(Some(0));

        let end = (5 * MILLISECS_PER_BLOCK) as u64;
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            ALICE,
            MarketPeriod::Timestamp(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::AmmCdaHybrid,
        ));
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            ALICE,
            MarketPeriod::Timestamp(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::AmmCdaHybrid,
        ));

        // This block takes much longer than 12sec, but markets and pools still close correctly.
        set_timestamp_for_on_initialize(
            end + (crate::MAX_RECOVERY_TIME_FRAMES + 1) * MILLISECS_PER_BLOCK as u64,
        );
        run_to_block(2); // Trigger `on_initialize`; must be at least block #2!
        let new_end = <zrml_market_commons::Pallet<Runtime>>::now();
        assert_ne!(end, new_end);

        // still active, not closed, because recovery limit reached
        let market_after_close = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after_close.status, MarketStatus::Active);

        let market_after_close = MarketCommons::market(&1).unwrap();
        assert_eq!(market_after_close.status, MarketStatus::Active);

        let range_end_time_frame = crate::Pallet::<Runtime>::calculate_time_frame_of_moment(end);
        assert_eq!(MarketIdsPerCloseTimeFrame::<Runtime>::get(range_end_time_frame), vec![0, 1]);

        assert_ok!(PredictionMarkets::manually_close_market(RuntimeOrigin::signed(ALICE), 0));
        assert_eq!(MarketIdsPerCloseTimeFrame::<Runtime>::get(range_end_time_frame), vec![1]);
        let market_after_manual_close = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after_manual_close.status, MarketStatus::Closed);

        assert_eq!(market_after_manual_close.period, MarketPeriod::Timestamp(0..new_end));
        assert_ok!(PredictionMarkets::manually_close_market(RuntimeOrigin::signed(ALICE), 1));
        assert_eq!(MarketIdsPerCloseTimeFrame::<Runtime>::get(range_end_time_frame), vec![]);
        let market_after_manual_close = MarketCommons::market(&1).unwrap();
        assert_eq!(market_after_manual_close.status, MarketStatus::Closed);
        assert_eq!(market_after_manual_close.period, MarketPeriod::Timestamp(0..new_end));
    });
}

#[test]
fn manually_close_market_fails_if_market_not_in_close_time_frame_list() {
    // We check that `on_market_close` works correctly even if a block takes much longer than 12sec
    // to be produced and multiple markets are involved.
    ExtBuilder::default().build().execute_with(|| {
        // Mock last time frame to prevent it from defaulting.
        LastTimeFrame::<Runtime>::set(Some(0));

        let end = (5 * MILLISECS_PER_BLOCK) as u64;
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            ALICE,
            MarketPeriod::Timestamp(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::AmmCdaHybrid,
        ));

        // remove market from open time frame list
        let range_end_time_frame = crate::Pallet::<Runtime>::calculate_time_frame_of_moment(end);
        crate::MarketIdsPerCloseTimeFrame::<Runtime>::remove(range_end_time_frame);

        // This block takes much longer than 12sec, but markets and pools still close correctly.
        set_timestamp_for_on_initialize(
            end + (crate::MAX_RECOVERY_TIME_FRAMES + 1) * MILLISECS_PER_BLOCK as u64,
        );
        run_to_block(2); // Trigger `on_initialize`; must be at least block #2!

        assert_noop!(
            PredictionMarkets::manually_close_market(RuntimeOrigin::signed(ALICE), 0),
            Error::<Runtime>::MarketNotInCloseTimeFrameList
        );
    });
}

#[test]
fn manually_close_market_fails_if_not_allowed_for_block_based_markets() {
    // We check that `on_market_close` works correctly even if a block takes much longer than 12sec
    // to be produced and multiple markets are involved.
    ExtBuilder::default().build().execute_with(|| {
        // Mock last time frame to prevent it from defaulting.
        LastTimeFrame::<Runtime>::set(Some(0));

        let category_count = 3;
        let end = 5;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            ALICE,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::AmmCdaHybrid,
        ));

        // This block takes much longer than 12sec, but markets and pools still close correctly.
        set_timestamp_for_on_initialize(
            end + (crate::MAX_RECOVERY_TIME_FRAMES + 1) * MILLISECS_PER_BLOCK as u64,
        );
        run_to_block(2); // Trigger `on_initialize`; must be at least block #2!

        assert_noop!(
            PredictionMarkets::manually_close_market(RuntimeOrigin::signed(ALICE), 0),
            Error::<Runtime>::NotAllowedForBlockBasedMarkets
        );
    });
}
