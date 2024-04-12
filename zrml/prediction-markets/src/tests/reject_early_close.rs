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

use crate::MarketIdsPerCloseBlock;
use zeitgeist_primitives::types::EarlyCloseState;

// TODO(#1239) MarketDoesNotExist
// TODO(#1239) NoEarlyCloseScheduled

#[test]
fn reject_early_close_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );

        let market_id = 0;

        assert_ok!(PredictionMarkets::schedule_early_close(
            RuntimeOrigin::signed(ALICE),
            market_id,
        ));

        assert_ok!(PredictionMarkets::dispute_early_close(RuntimeOrigin::signed(BOB), market_id,));

        assert_ok!(PredictionMarkets::reject_early_close(
            RuntimeOrigin::signed(CloseMarketEarlyOrigin::get()),
            market_id
        ));

        System::assert_last_event(Event::MarketEarlyCloseRejected { market_id }.into());
    });
}

#[test]
fn reject_early_close_fails_if_state_is_scheduled_as_market_creator() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );

        // just to ensure events are emitted
        run_blocks(2);

        let market_id = 0;

        assert_ok!(PredictionMarkets::schedule_early_close(
            RuntimeOrigin::signed(ALICE),
            market_id,
        ));

        assert_noop!(
            PredictionMarkets::reject_early_close(
                RuntimeOrigin::signed(CloseMarketEarlyOrigin::get()),
                market_id
            ),
            Error::<Runtime>::InvalidEarlyCloseState
        );
    });
}

#[test]
fn reject_early_close_fails_if_state_is_rejected() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );

        // just to ensure events are emitted
        run_blocks(2);

        let market_id = 0;

        assert_ok!(PredictionMarkets::schedule_early_close(
            RuntimeOrigin::signed(CloseMarketEarlyOrigin::get()),
            market_id
        ));

        assert_ok!(PredictionMarkets::reject_early_close(
            RuntimeOrigin::signed(CloseMarketEarlyOrigin::get()),
            market_id
        ));

        assert_noop!(
            PredictionMarkets::reject_early_close(
                RuntimeOrigin::signed(CloseMarketEarlyOrigin::get()),
                market_id
            ),
            Error::<Runtime>::InvalidEarlyCloseState
        );
    });
}

#[test]
fn reject_early_close_resets_to_old_market_period() {
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
            ScoringRule::AmmCdaHybrid
        ));

        let market_id = 0;
        assert_ok!(PredictionMarkets::schedule_early_close(
            RuntimeOrigin::signed(CloseMarketEarlyOrigin::get()),
            market_id
        ));

        let now = <frame_system::Pallet<Runtime>>::block_number();
        let new_end = now + <Runtime as Config>::CloseEarlyProtectionBlockPeriod::get();
        let market_ids_at_new_end = <MarketIdsPerCloseBlock<Runtime>>::get(new_end);
        assert_eq!(market_ids_at_new_end, vec![market_id]);

        run_blocks(1);

        assert_ok!(PredictionMarkets::reject_early_close(
            RuntimeOrigin::signed(CloseMarketEarlyOrigin::get()),
            market_id
        ));

        let market_ids_at_new_end = <MarketIdsPerCloseBlock<Runtime>>::get(new_end);
        assert!(market_ids_at_new_end.is_empty());

        let market_ids_at_old_end = <MarketIdsPerCloseBlock<Runtime>>::get(end);
        assert_eq!(market_ids_at_old_end, vec![market_id]);
    });
}

#[test]
fn reject_early_close_settles_bonds() {
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
            ScoringRule::AmmCdaHybrid
        ));

        let market_id = 0;
        assert_ok!(PredictionMarkets::schedule_early_close(
            RuntimeOrigin::signed(ALICE),
            market_id,
        ));

        run_blocks(1);

        assert_ok!(PredictionMarkets::dispute_early_close(RuntimeOrigin::signed(BOB), market_id,));

        let reserved_bob = Balances::reserved_balance(BOB);
        let reserved_alice = Balances::reserved_balance(ALICE);
        let free_bob = Balances::free_balance(BOB);
        let free_alice = Balances::free_balance(ALICE);

        assert_ok!(PredictionMarkets::reject_early_close(
            RuntimeOrigin::signed(CloseMarketEarlyOrigin::get()),
            market_id
        ));

        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.early_close.unwrap().state, EarlyCloseState::Rejected);

        let reserved_bob_after = Balances::reserved_balance(BOB);
        let reserved_alice_after = Balances::reserved_balance(ALICE);
        let free_bob_after = Balances::free_balance(BOB);
        let free_alice_after = Balances::free_balance(ALICE);

        assert_eq!(
            reserved_alice - reserved_alice_after,
            <Runtime as Config>::CloseEarlyRequestBond::get()
        );
        assert_eq!(
            reserved_bob - reserved_bob_after,
            <Runtime as Config>::CloseEarlyDisputeBond::get()
        );
        // disputant Bob gets the bonds
        assert_eq!(
            free_bob_after - free_bob,
            <Runtime as Config>::CloseEarlyRequestBond::get()
                + <Runtime as Config>::CloseEarlyDisputeBond::get()
        );
        assert_eq!(free_alice_after - free_alice, 0);
    });
}
