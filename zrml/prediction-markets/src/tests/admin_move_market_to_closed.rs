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
use crate::{MarketIdsPerCloseBlock, MomentOf};
use test_case::test_case;
use zeitgeist_primitives::constants::MILLISECS_PER_BLOCK;

#[test]
fn admin_move_market_to_closed_successfully_closes_market_and_sets_end_blocknumber() {
    ExtBuilder::default().build().execute_with(|| {
        run_blocks(7);
        let now = frame_system::Pallet::<Runtime>::block_number();
        let end = 42;
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            now..end,
            ScoringRule::Lmsr,
        );
        run_blocks(3);
        let market_id = 0;
        assert_ok!(PredictionMarkets::admin_move_market_to_closed(
            RuntimeOrigin::signed(CloseOrigin::get()),
            market_id
        ));
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        let new_end = now + 3;
        assert_eq!(market.period, MarketPeriod::Block(now..new_end));
        assert_ne!(new_end, end);
        System::assert_last_event(Event::MarketClosed(market_id).into());
    });
}

#[test]
fn admin_move_market_to_closed_successfully_closes_market_and_sets_end_timestamp() {
    ExtBuilder::default().build().execute_with(|| {
        let start_block = 7;
        set_timestamp_for_on_initialize(start_block * MILLISECS_PER_BLOCK as MomentOf<Runtime>);
        run_blocks(start_block);
        let start = <zrml_market_commons::Pallet<Runtime>>::now();

        let end = start + 42u64 * (MILLISECS_PER_BLOCK as u64);
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
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr
        ));
        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.period, MarketPeriod::Timestamp(start..end));

        let shift_blocks = 3;
        let shift = shift_blocks * MILLISECS_PER_BLOCK as u64;
        // millisecs per block is substracted inside the function
        set_timestamp_for_on_initialize(start + shift + MILLISECS_PER_BLOCK as u64);
        run_blocks(shift_blocks);

        assert_ok!(PredictionMarkets::admin_move_market_to_closed(
            RuntimeOrigin::signed(CloseOrigin::get()),
            market_id
        ));
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        let new_end = start + shift;
        assert_eq!(market.period, MarketPeriod::Timestamp(start..new_end));
        assert_ne!(new_end, end);
        System::assert_last_event(Event::MarketClosed(market_id).into());
    });
}

#[test]
fn admin_move_market_to_closed_fails_if_market_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::admin_move_market_to_closed(
                RuntimeOrigin::signed(CloseOrigin::get()),
                0
            ),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test_case(MarketStatus::Closed; "closed")]
#[test_case(MarketStatus::Reported; "reported")]
#[test_case(MarketStatus::Disputed; "disputed")]
#[test_case(MarketStatus::Resolved; "resolved")]
#[test_case(MarketStatus::Proposed; "proposed")]
fn admin_move_market_to_closed_fails_if_market_is_not_active(market_status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::Lmsr,
        );
        let market_id = 0;
        assert_ok!(MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        }));
        assert_noop!(
            PredictionMarkets::admin_move_market_to_closed(
                RuntimeOrigin::signed(CloseOrigin::get()),
                market_id
            ),
            Error::<Runtime>::MarketIsNotActive,
        );
    });
}

#[test]
fn admin_move_market_to_closed_correctly_clears_auto_close_blocks() {
    ExtBuilder::default().build().execute_with(|| {
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            ALICE,
            MarketPeriod::Block(22..66),
            get_deadlines(),
            gen_metadata(50),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            ALICE,
            MarketPeriod::Block(33..66),
            get_deadlines(),
            gen_metadata(50),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        assert_ok!(PredictionMarkets::admin_move_market_to_closed(
            RuntimeOrigin::signed(CloseOrigin::get()),
            0
        ));

        let auto_close = MarketIdsPerCloseBlock::<Runtime>::get(66).into_inner();
        assert_eq!(auto_close, vec![1]);
    });
}
