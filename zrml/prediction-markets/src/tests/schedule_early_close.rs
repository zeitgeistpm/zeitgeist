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

use crate::{MarketIdsPerCloseBlock, MarketIdsPerCloseTimeFrame};
use zeitgeist_primitives::{
    constants::MILLISECS_PER_BLOCK,
    types::{EarlyClose, EarlyCloseState},
};

// TODO(#1239) MarketDoesNotExist
// TODO(#1239) RequesterNotCreator
// TODO(#1239) MarketIsNotActive
// TODO(#1239) OnlyAuthorizedCanScheduleEarlyClose
// TODO(#1239) Correct repatriations
// TODO(#1239) reserve_named failure

#[test]
fn schedule_early_close_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::Lmsr,
        );

        let market_id = 0;

        assert_ok!(PredictionMarkets::schedule_early_close(
            RuntimeOrigin::signed(CloseMarketEarlyOrigin::get()),
            market_id
        ));

        let now = <frame_system::Pallet<Runtime>>::block_number();
        let new_end = now + <Runtime as Config>::CloseEarlyProtectionBlockPeriod::get();
        assert!(new_end < end);

        let new_period = MarketPeriod::Block(0..new_end);
        System::assert_last_event(
            Event::MarketEarlyCloseScheduled {
                market_id,
                new_period: new_period.clone(),
                state: EarlyCloseState::ScheduledAsOther,
            }
            .into(),
        );
    });
}

#[test]
fn sudo_schedule_early_close_at_block_works() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::Lmsr
        ));

        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        let old_market_period = market.period;
        assert_eq!(market.status, MarketStatus::Active);
        let market_ids_to_close = <MarketIdsPerCloseBlock<Runtime>>::iter().next().unwrap();
        assert_eq!(market_ids_to_close.0, end);
        assert_eq!(market_ids_to_close.1.into_inner(), vec![market_id]);
        assert!(market.early_close.is_none());

        assert_ok!(PredictionMarkets::schedule_early_close(
            RuntimeOrigin::signed(CloseMarketEarlyOrigin::get()),
            market_id
        ));

        let now = <frame_system::Pallet<Runtime>>::block_number();
        let new_end = now + <Runtime as Config>::CloseEarlyProtectionBlockPeriod::get();
        assert!(new_end < end);

        let market = MarketCommons::market(&market_id).unwrap();
        let new_period = MarketPeriod::Block(0..new_end);
        assert_eq!(
            market.early_close.unwrap(),
            EarlyClose {
                old: old_market_period,
                new: new_period,
                state: EarlyCloseState::ScheduledAsOther,
            }
        );

        let market_ids_to_close = <MarketIdsPerCloseBlock<Runtime>>::iter().collect::<Vec<_>>();
        assert_eq!(market_ids_to_close.len(), 2);

        // The first entry is the old one without a market id inside.
        let first = market_ids_to_close.first().unwrap();
        assert_eq!(first.0, end);
        assert!(first.1.clone().into_inner().is_empty());

        // The second entry is the new one with the market id inside.
        let second = market_ids_to_close.last().unwrap();
        assert_eq!(second.0, new_end);
        assert_eq!(second.1.clone().into_inner(), vec![market_id]);

        run_to_block(new_end + 1);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
    });
}

#[test]
fn sudo_schedule_early_close_at_timeframe_works() {
    ExtBuilder::default().build().execute_with(|| {
        let start_block = 7;
        set_timestamp_for_on_initialize(start_block * MILLISECS_PER_BLOCK as u64);
        run_blocks(start_block);
        let start = <zrml_market_commons::Pallet<Runtime>>::now();

        let end = start + (42 * MILLISECS_PER_BLOCK) as u64;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Timestamp(start..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::Lmsr
        ));

        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        let old_market_period = market.period;
        assert_eq!(market.status, MarketStatus::Active);
        let market_ids_to_close = <MarketIdsPerCloseTimeFrame<Runtime>>::iter().collect::<Vec<_>>();
        assert_eq!(market_ids_to_close.len(), 1);
        let first = market_ids_to_close.first().unwrap();
        assert_eq!(first.0, end.saturating_div(MILLISECS_PER_BLOCK.into()));
        assert_eq!(first.1.clone().into_inner(), vec![market_id]);
        assert!(market.early_close.is_none());

        assert_ok!(PredictionMarkets::schedule_early_close(
            RuntimeOrigin::signed(CloseMarketEarlyOrigin::get()),
            market_id
        ));

        let now = <zrml_market_commons::Pallet<Runtime>>::now();
        let new_end = now + <Runtime as Config>::CloseEarlyProtectionTimeFramePeriod::get();
        assert!(new_end < end);

        let market = MarketCommons::market(&market_id).unwrap();
        let new_period = MarketPeriod::Timestamp(start..new_end);
        assert_eq!(
            market.early_close.unwrap(),
            EarlyClose {
                old: old_market_period,
                new: new_period,
                state: EarlyCloseState::ScheduledAsOther,
            }
        );

        let market_ids_to_close = <MarketIdsPerCloseTimeFrame<Runtime>>::iter().collect::<Vec<_>>();
        assert_eq!(market_ids_to_close.len(), 2);

        // The first entry is the new one with the market id inside.
        let first = market_ids_to_close.first().unwrap();
        assert_eq!(first.0, new_end.saturating_div(MILLISECS_PER_BLOCK.into()));
        assert_eq!(first.1.clone().into_inner(), vec![market_id]);

        // The second entry is the old one without a market id inside.
        let second = market_ids_to_close.last().unwrap();
        assert_eq!(second.0, end.saturating_div(MILLISECS_PER_BLOCK.into()));
        assert!(second.1.clone().into_inner().is_empty());

        set_timestamp_for_on_initialize(start_block * MILLISECS_PER_BLOCK as u64 + new_end);
        run_to_block(start_block + new_end.saturating_div(MILLISECS_PER_BLOCK.into()) + 1);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
    });
}

#[test]
fn schedule_early_close_block_fails_if_early_close_request_too_late() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::Lmsr
        ));

        run_to_block(end - 1);

        let market_id = 0;
        assert_noop!(
            PredictionMarkets::schedule_early_close(RuntimeOrigin::signed(ALICE), market_id,),
            Error::<Runtime>::EarlyCloseRequestTooLate
        );
    });
}

#[test]
fn schedule_early_close_timestamp_fails_if_early_close_request_too_late() {
    ExtBuilder::default().build().execute_with(|| {
        let start_block = 7;
        set_timestamp_for_on_initialize(start_block * MILLISECS_PER_BLOCK as u64);
        run_blocks(start_block);
        let start = <zrml_market_commons::Pallet<Runtime>>::now();
        let end = start + (42 * MILLISECS_PER_BLOCK) as u64;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Timestamp(start..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::Lmsr
        ));

        run_to_block(end.saturating_div(MILLISECS_PER_BLOCK.into()) - 1);
        set_timestamp_for_on_initialize(end - MILLISECS_PER_BLOCK as u64);

        let market_id = 0;
        assert_noop!(
            PredictionMarkets::schedule_early_close(RuntimeOrigin::signed(ALICE), market_id,),
            Error::<Runtime>::EarlyCloseRequestTooLate
        );
    });
}

#[test]
fn schedule_early_close_as_market_creator_works() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::Lmsr
        ));

        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        let old_market_period = market.period;
        assert_eq!(market.status, MarketStatus::Active);
        let market_ids_to_close = <MarketIdsPerCloseBlock<Runtime>>::iter().next().unwrap();
        assert_eq!(market_ids_to_close.0, end);
        assert_eq!(market_ids_to_close.1.into_inner(), vec![market_id]);
        assert!(market.early_close.is_none());

        let reserved_balance_alice = Balances::reserved_balance(ALICE);

        assert_ok!(PredictionMarkets::schedule_early_close(
            RuntimeOrigin::signed(ALICE),
            market_id,
        ));

        let reserved_balance_alice_after = Balances::reserved_balance(ALICE);
        assert_eq!(
            reserved_balance_alice_after - reserved_balance_alice,
            <Runtime as Config>::CloseEarlyRequestBond::get()
        );

        let now = <frame_system::Pallet<Runtime>>::block_number();
        let new_end = now + <Runtime as Config>::CloseEarlyBlockPeriod::get();
        assert!(new_end < end);

        let market = MarketCommons::market(&market_id).unwrap();
        let new_period = MarketPeriod::Block(0..new_end);
        assert_eq!(
            market.early_close.unwrap(),
            EarlyClose {
                old: old_market_period,
                new: new_period,
                state: EarlyCloseState::ScheduledAsMarketCreator,
            }
        );

        let market_ids_to_close = <MarketIdsPerCloseBlock<Runtime>>::iter().collect::<Vec<_>>();
        assert_eq!(market_ids_to_close.len(), 2);

        // The first entry is the old one without a market id inside.
        let first = market_ids_to_close.first().unwrap();
        assert_eq!(first.0, end);
        assert!(first.1.clone().into_inner().is_empty());

        // The second entry is the new one with the market id inside.
        let second = market_ids_to_close.last().unwrap();
        assert_eq!(second.0, new_end);
        assert_eq!(second.1.clone().into_inner(), vec![market_id]);

        run_to_block(new_end + 1);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
    });
}
