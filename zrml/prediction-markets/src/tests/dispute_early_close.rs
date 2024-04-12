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
use zeitgeist_primitives::types::{Bond, EarlyClose, EarlyCloseState};

// TODO(#1239) MarketDoesNotExist
// TODO(#1239) MarketIsNotActive
// TODO(#1239) dispute bond failure

#[test]
fn dispute_early_close_emits_event() {
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

        System::assert_last_event(Event::MarketEarlyCloseDisputed { market_id }.into());
    });
}

#[test]
fn dispute_early_close_from_market_creator_works() {
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
        let market = MarketCommons::market(&market_id).unwrap();
        let old_market_period = market.period;

        assert_ok!(PredictionMarkets::schedule_early_close(
            RuntimeOrigin::signed(ALICE),
            market_id,
        ));

        let now = <frame_system::Pallet<Runtime>>::block_number();
        let new_end = now + <Runtime as Config>::CloseEarlyBlockPeriod::get();
        let market_ids_at_new_end = <MarketIdsPerCloseBlock<Runtime>>::get(new_end);
        assert_eq!(market_ids_at_new_end, vec![market_id]);

        run_blocks(1);

        let reserved_bob = Balances::reserved_balance(BOB);

        assert_ok!(PredictionMarkets::dispute_early_close(RuntimeOrigin::signed(BOB), market_id,));

        let reserved_bob_after = Balances::reserved_balance(BOB);
        assert_eq!(
            reserved_bob_after - reserved_bob,
            <Runtime as Config>::CloseEarlyDisputeBond::get()
        );

        let market_ids_at_new_end = <MarketIdsPerCloseBlock<Runtime>>::get(new_end);
        assert!(market_ids_at_new_end.is_empty());

        let market_ids_at_old_end = <MarketIdsPerCloseBlock<Runtime>>::get(end);
        assert_eq!(market_ids_at_old_end, vec![market_id]);

        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.period, old_market_period);
        assert_eq!(
            market.bonds.close_dispute,
            Some(Bond::new(BOB, <Runtime as Config>::CloseEarlyDisputeBond::get()))
        );
        let new_period = MarketPeriod::Block(0..new_end);
        assert_eq!(
            market.early_close.unwrap(),
            EarlyClose {
                old: old_market_period,
                new: new_period,
                state: EarlyCloseState::Disputed,
            }
        );

        run_to_block(new_end + 1);

        // verify the market doesn't close after proposed new market period end
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Active);
    });
}

#[test]
fn dispute_early_close_fails_if_scheduled_as_sudo() {
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

        run_blocks(1);

        assert_noop!(
            PredictionMarkets::dispute_early_close(RuntimeOrigin::signed(BOB), market_id,),
            Error::<Runtime>::InvalidEarlyCloseState
        );
    });
}

#[test]
fn dispute_early_close_fails_if_already_disputed() {
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

        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.early_close.unwrap().state, EarlyCloseState::Disputed);

        assert_noop!(
            PredictionMarkets::dispute_early_close(RuntimeOrigin::signed(BOB), market_id,),
            Error::<Runtime>::InvalidEarlyCloseState
        );
    });
}

#[test]
fn dispute_early_close_fails_if_already_rejected() {
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

        assert_ok!(PredictionMarkets::reject_early_close(
            RuntimeOrigin::signed(CloseMarketEarlyOrigin::get()),
            market_id
        ));

        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.early_close.unwrap().state, EarlyCloseState::Rejected);

        assert_noop!(
            PredictionMarkets::dispute_early_close(RuntimeOrigin::signed(BOB), market_id,),
            Error::<Runtime>::InvalidEarlyCloseState
        );
    });
}

#[test]
fn settles_early_close_bonds_with_resolution_in_state_disputed() {
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

        let alice_free = Balances::free_balance(ALICE);
        let alice_reserved = Balances::reserved_balance(ALICE);

        run_blocks(1);

        assert_ok!(PredictionMarkets::dispute_early_close(RuntimeOrigin::signed(BOB), market_id,));

        let bob_free = Balances::free_balance(BOB);
        let bob_reserved = Balances::reserved_balance(BOB);

        run_to_block(end + 1);

        // verify the market doesn't close after proposed new market period end
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);

        let alice_free_after = Balances::free_balance(ALICE);
        let alice_reserved_after = Balances::reserved_balance(ALICE);
        // moved <Runtime as Config>::CloseEarlyRequestBond from reserved to free
        assert_eq!(
            alice_reserved - alice_reserved_after,
            <Runtime as Config>::CloseEarlyRequestBond::get()
        );
        assert_eq!(
            alice_free_after - alice_free,
            <Runtime as Config>::CloseEarlyRequestBond::get()
        );

        let bob_free_after = Balances::free_balance(BOB);
        let bob_reserved_after = Balances::reserved_balance(BOB);
        // moved <Runtime as Config>::CloseEarlyDisputeBond from reserved to free
        assert_eq!(
            bob_reserved - bob_reserved_after,
            <Runtime as Config>::CloseEarlyDisputeBond::get()
        );
        assert_eq!(bob_free_after - bob_free, <Runtime as Config>::CloseEarlyDisputeBond::get());
    });
}

#[test]
fn settles_early_close_bonds_with_resolution_in_state_scheduled_as_market_creator() {
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

        let alice_free = Balances::free_balance(ALICE);
        let alice_reserved = Balances::reserved_balance(ALICE);

        run_to_block(end + 1);

        // verify the market doesn't close after proposed new market period end
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);

        let alice_free_after = Balances::free_balance(ALICE);
        let alice_reserved_after = Balances::reserved_balance(ALICE);
        // moved <Runtime as Config>::CloseEarlyRequestBond from reserved to free
        assert_eq!(
            alice_reserved - alice_reserved_after,
            <Runtime as Config>::CloseEarlyRequestBond::get()
        );
        assert_eq!(
            alice_free_after - alice_free,
            <Runtime as Config>::CloseEarlyRequestBond::get()
        );
    });
}

#[test]
fn schedule_early_close_disputed_sudo_schedule_and_settle_bonds() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        let old_period = MarketPeriod::Block(0..end);
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            old_period.clone(),
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

        assert_ok!(PredictionMarkets::schedule_early_close(
            RuntimeOrigin::signed(CloseMarketEarlyOrigin::get()),
            market_id
        ));

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
        // market creator Alice gets the bonds
        assert_eq!(
            free_alice_after - free_alice,
            <Runtime as Config>::CloseEarlyRequestBond::get()
                + <Runtime as Config>::CloseEarlyDisputeBond::get()
        );
        assert_eq!(free_bob_after - free_bob, 0);

        let now = <frame_system::Pallet<Runtime>>::block_number();
        let new_end = now + <Runtime as Config>::CloseEarlyProtectionBlockPeriod::get();
        let market_ids_at_new_end = <MarketIdsPerCloseBlock<Runtime>>::get(new_end);
        assert_eq!(market_ids_at_new_end, vec![market_id]);

        let market = MarketCommons::market(&market_id).unwrap();
        let new_period = MarketPeriod::Block(0..new_end);
        assert_eq!(
            market.early_close.unwrap(),
            EarlyClose {
                old: old_period,
                new: new_period,
                state: EarlyCloseState::ScheduledAsOther,
            }
        );
    });
}
