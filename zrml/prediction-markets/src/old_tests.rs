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

#![cfg(all(feature = "mock", test))]
#![allow(clippy::reversed_empty_ranges)]

extern crate alloc;

use crate::{
    mock::*, Config, Error, Event, MarketIdsPerCloseBlock, MarketIdsPerDisputeBlock,
    MarketIdsPerReportBlock,
};
use alloc::collections::BTreeMap;
use core::ops::Range;
use frame_support::{assert_noop, assert_ok, traits::NamedReservableCurrency};
use sp_runtime::{traits::BlakeTwo256, Perquintill};
use test_case::test_case;
use zeitgeist_primitives::types::{EarlyClose, EarlyCloseState};
use zrml_court::{types::*, Error as CError};

use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_arithmetic::Perbill;
use sp_runtime::traits::{Hash, Zero};
use zeitgeist_primitives::{
    constants::mock::{
        CloseEarlyDisputeBond, CloseEarlyProtectionBlockPeriod, CloseEarlyRequestBond, MaxAppeals,
        MaxSelectedDraws, MinJurorStake, OutcomeBond, OutcomeFactor, OutsiderBond, BASE, CENT,
        MILLISECS_PER_BLOCK,
    },
    types::{
        AccountIdTest, Asset, Balance, Bond, Deadlines, MarketBonds, MarketCreation,
        MarketDisputeMechanism, MarketId, MarketPeriod, MarketStatus, MarketType, MultiHash,
        OutcomeReport, Report, ScalarPosition, ScoringRule,
    },
};
use zrml_global_disputes::{
    types::{OutcomeInfo, Possession},
    GlobalDisputesPalletApi, Outcomes, PossessionOf,
};
use zrml_market_commons::MarketCommonsPalletApi;

const SENTINEL_AMOUNT: u128 = BASE;

fn get_deadlines() -> Deadlines<<Runtime as frame_system::Config>::BlockNumber> {
    Deadlines {
        grace_period: 1_u32.into(),
        oracle_duration: <Runtime as crate::Config>::MinOracleDuration::get(),
        dispute_duration: <Runtime as crate::Config>::MinDisputeDuration::get(),
    }
}

fn gen_metadata(byte: u8) -> MultiHash {
    let mut metadata = [byte; 50];
    metadata[0] = 0x15;
    metadata[1] = 0x30;
    MultiHash::Sha3_384(metadata)
}

fn reserve_sentinel_amounts() {
    // Reserve a sentinel amount to check that we don't unreserve too much.
    assert_ok!(Balances::reserve_named(&PredictionMarkets::reserve_id(), &ALICE, SENTINEL_AMOUNT));
    assert_ok!(Balances::reserve_named(&PredictionMarkets::reserve_id(), &BOB, SENTINEL_AMOUNT));
    assert_ok!(Balances::reserve_named(
        &PredictionMarkets::reserve_id(),
        &CHARLIE,
        SENTINEL_AMOUNT
    ));
    assert_ok!(Balances::reserve_named(&PredictionMarkets::reserve_id(), &DAVE, SENTINEL_AMOUNT));
    assert_ok!(Balances::reserve_named(&PredictionMarkets::reserve_id(), &EVE, SENTINEL_AMOUNT));
    assert_ok!(Balances::reserve_named(&PredictionMarkets::reserve_id(), &FRED, SENTINEL_AMOUNT));
    assert_eq!(Balances::reserved_balance(ALICE), SENTINEL_AMOUNT);
    assert_eq!(Balances::reserved_balance(BOB), SENTINEL_AMOUNT);
    assert_eq!(Balances::reserved_balance(CHARLIE), SENTINEL_AMOUNT);
    assert_eq!(Balances::reserved_balance(DAVE), SENTINEL_AMOUNT);
    assert_eq!(Balances::reserved_balance(EVE), SENTINEL_AMOUNT);
    assert_eq!(Balances::reserved_balance(FRED), SENTINEL_AMOUNT);
}

fn check_reserve(account: &AccountIdTest, expected: Balance) {
    assert_eq!(Balances::reserved_balance(account), SENTINEL_AMOUNT + expected);
}

fn simple_create_categorical_market(
    base_asset: Asset<MarketId>,
    creation: MarketCreation,
    period: Range<u64>,
    scoring_rule: ScoringRule,
) {
    assert_ok!(PredictionMarkets::create_market(
        RuntimeOrigin::signed(ALICE),
        base_asset,
        Perbill::zero(),
        BOB,
        MarketPeriod::Block(period),
        get_deadlines(),
        gen_metadata(2),
        creation,
        MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
        Some(MarketDisputeMechanism::SimpleDisputes),
        scoring_rule
    ));
}

fn simple_create_scalar_market(
    base_asset: Asset<MarketId>,
    creation: MarketCreation,
    period: Range<u64>,
    scoring_rule: ScoringRule,
) {
    assert_ok!(PredictionMarkets::create_market(
        RuntimeOrigin::signed(ALICE),
        base_asset,
        Perbill::zero(),
        BOB,
        MarketPeriod::Block(period),
        get_deadlines(),
        gen_metadata(2),
        creation,
        MarketType::Scalar(100..=200),
        Some(MarketDisputeMechanism::SimpleDisputes),
        scoring_rule
    ));
}

#[test]
fn reject_early_close_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::Lmsr,
        );

        // just to ensure events are emitted
        run_blocks(2);

        let market_id = 0;

        assert_ok!(PredictionMarkets::schedule_early_close(
            RuntimeOrigin::signed(ALICE),
            market_id,
        ));

        assert_ok!(PredictionMarkets::dispute_early_close(RuntimeOrigin::signed(BOB), market_id,));

        assert_ok!(PredictionMarkets::reject_early_close(RuntimeOrigin::signed(SUDO), market_id,));

        System::assert_last_event(Event::MarketEarlyCloseRejected { market_id }.into());
    });
}

#[test]
fn reject_early_close_fails_if_state_is_scheduled_as_market_creator() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::Lmsr,
        );

        // just to ensure events are emitted
        run_blocks(2);

        let market_id = 0;

        assert_ok!(PredictionMarkets::schedule_early_close(
            RuntimeOrigin::signed(ALICE),
            market_id,
        ));

        assert_noop!(
            PredictionMarkets::reject_early_close(RuntimeOrigin::signed(SUDO), market_id,),
            Error::<Runtime>::InvalidEarlyCloseState
        );
    });
}

#[test]
fn reject_early_close_fails_if_state_is_rejected() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::Lmsr,
        );

        // just to ensure events are emitted
        run_blocks(2);

        let market_id = 0;

        assert_ok!(
            PredictionMarkets::schedule_early_close(RuntimeOrigin::signed(SUDO), market_id,)
        );

        assert_ok!(PredictionMarkets::reject_early_close(RuntimeOrigin::signed(SUDO), market_id,));

        assert_noop!(
            PredictionMarkets::reject_early_close(RuntimeOrigin::signed(SUDO), market_id,),
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
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::Lmsr
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
        // moved CloseEarlyRequestBond from reserved to free
        assert_eq!(alice_reserved - alice_reserved_after, CloseEarlyRequestBond::get());
        assert_eq!(alice_free_after - alice_free, CloseEarlyRequestBond::get());

        let bob_free_after = Balances::free_balance(BOB);
        let bob_reserved_after = Balances::reserved_balance(BOB);
        // moved CloseEarlyDisputeBond from reserved to free
        assert_eq!(bob_reserved - bob_reserved_after, CloseEarlyDisputeBond::get());
        assert_eq!(bob_free_after - bob_free, CloseEarlyDisputeBond::get());
    });
}

#[test]
fn settles_early_close_bonds_with_resolution_in_state_scheduled_as_market_creator() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::Lmsr
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
        // moved CloseEarlyRequestBond from reserved to free
        assert_eq!(alice_reserved - alice_reserved_after, CloseEarlyRequestBond::get());
        assert_eq!(alice_free_after - alice_free, CloseEarlyRequestBond::get());
    });
}

#[test]
fn reject_early_close_resets_to_old_market_period() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::Lmsr
        ));

        let market_id = 0;
        assert_ok!(
            PredictionMarkets::schedule_early_close(RuntimeOrigin::signed(SUDO), market_id,)
        );

        let now = <frame_system::Pallet<Runtime>>::block_number();
        let new_end = now + CloseEarlyProtectionBlockPeriod::get();
        let market_ids_at_new_end = <MarketIdsPerCloseBlock<Runtime>>::get(new_end);
        assert_eq!(market_ids_at_new_end, vec![market_id]);

        run_blocks(1);

        assert_ok!(PredictionMarkets::reject_early_close(RuntimeOrigin::signed(SUDO), market_id,));

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
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::Lmsr
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

        assert_ok!(PredictionMarkets::reject_early_close(RuntimeOrigin::signed(SUDO), market_id,));

        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.early_close.unwrap().state, EarlyCloseState::Rejected);

        let reserved_bob_after = Balances::reserved_balance(BOB);
        let reserved_alice_after = Balances::reserved_balance(ALICE);
        let free_bob_after = Balances::free_balance(BOB);
        let free_alice_after = Balances::free_balance(ALICE);

        assert_eq!(reserved_alice - reserved_alice_after, CloseEarlyRequestBond::get());
        assert_eq!(reserved_bob - reserved_bob_after, CloseEarlyDisputeBond::get());
        // disputant Bob gets the bonds
        assert_eq!(
            free_bob_after - free_bob,
            CloseEarlyRequestBond::get() + CloseEarlyDisputeBond::get()
        );
        assert_eq!(free_alice_after - free_alice, 0);
    });
}

#[test]
fn schedule_early_close_disputed_sudo_schedule_and_settle_bonds() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        let old_period = MarketPeriod::Block(0..end);
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            old_period.clone(),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::Lmsr
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

        assert_ok!(
            PredictionMarkets::schedule_early_close(RuntimeOrigin::signed(SUDO), market_id,)
        );

        let reserved_bob_after = Balances::reserved_balance(BOB);
        let reserved_alice_after = Balances::reserved_balance(ALICE);
        let free_bob_after = Balances::free_balance(BOB);
        let free_alice_after = Balances::free_balance(ALICE);

        assert_eq!(reserved_alice - reserved_alice_after, CloseEarlyRequestBond::get());
        assert_eq!(reserved_bob - reserved_bob_after, CloseEarlyDisputeBond::get());
        // market creator Alice gets the bonds
        assert_eq!(
            free_alice_after - free_alice,
            CloseEarlyRequestBond::get() + CloseEarlyDisputeBond::get()
        );
        assert_eq!(free_bob_after - free_bob, 0);

        let now = <frame_system::Pallet<Runtime>>::block_number();
        let new_end = now + CloseEarlyProtectionBlockPeriod::get();
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

#[test]
fn dispute_updates_market() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Authorized),
            ScoringRule::Lmsr,
        ));

        // Run to the end of the trading phase.
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let dispute_at = grace_period + 2;
        run_to_block(dispute_at);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Reported);
        assert_eq!(market.bonds.dispute, None);

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0,));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);
        assert_eq!(
            market.bonds.dispute,
            Some(Bond { who: CHARLIE, value: DisputeBond::get(), is_settled: false })
        );
    });
}

#[test]
fn dispute_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Authorized),
            ScoringRule::Lmsr,
        ));

        // Run to the end of the trading phase.
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let dispute_at = grace_period + 2;
        run_to_block(dispute_at);

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0,));

        System::assert_last_event(
            Event::MarketDisputed(0u32.into(), MarketStatus::Disputed, CHARLIE).into(),
        );
    });
}

#[test]
fn it_correctly_resolves_a_market_that_was_reported_on() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::Lmsr,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(CHARLIE), 0, CENT));

        let market = MarketCommons::market(&0).unwrap();
        let report_at = end + market.deadlines.grace_period + 1;
        run_to_block(report_at);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let reported_ids =
            MarketIdsPerReportBlock::<Runtime>::get(report_at + market.deadlines.dispute_duration);
        assert_eq!(reported_ids.len(), 1);
        let id = reported_ids[0];
        assert_eq!(id, 0);

        run_blocks(market.deadlines.dispute_duration);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Resolved);

        // Check balance of winning outcome asset.
        let share_b = Asset::CategoricalOutcome(0, 1);
        let share_b_total = Tokens::total_issuance(share_b);
        assert_eq!(share_b_total, CENT);
        let share_b_bal = Tokens::free_balance(share_b, &CHARLIE);
        assert_eq!(share_b_bal, CENT);

        // TODO(#792): Remove other assets.
        let share_a = Asset::CategoricalOutcome(0, 0);
        let share_a_total = Tokens::total_issuance(share_a);
        assert_eq!(share_a_total, CENT);
        let share_a_bal = Tokens::free_balance(share_a, &CHARLIE);
        assert_eq!(share_a_bal, CENT);

        let share_c = Asset::CategoricalOutcome(0, 2);
        let share_c_total = Tokens::total_issuance(share_c);
        assert_eq!(share_c_total, 0);
        let share_c_bal = Tokens::free_balance(share_c, &CHARLIE);
        assert_eq!(share_c_bal, 0);

        assert!(market.bonds.creation.unwrap().is_settled);
        assert!(market.bonds.oracle.unwrap().is_settled);
    });
}

#[test]
fn it_resolves_a_disputed_market() {
    let test = |base_asset: Asset<MarketId>| {
        let end = 2;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::Lmsr,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(CHARLIE), 0, CENT));
        let market = MarketCommons::market(&0).unwrap();

        let report_at = end + market.deadlines.grace_period + 1;
        run_to_block(report_at);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0,));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        let charlie_reserved = Balances::reserved_balance(CHARLIE);
        assert_eq!(charlie_reserved, DisputeBond::get());

        let dispute_at_0 = report_at + 1;
        run_to_block(dispute_at_0);

        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));

        let dispute_at_1 = report_at + 2;
        run_to_block(dispute_at_1);

        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(DAVE),
            0,
            OutcomeReport::Categorical(0)
        ));

        let dispute_at_2 = report_at + 3;
        run_to_block(dispute_at_2);

        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        // check everyone's deposits
        let charlie_reserved = Balances::reserved_balance(CHARLIE);
        assert_eq!(charlie_reserved, DisputeBond::get() + OutcomeBond::get());

        let dave_reserved = Balances::reserved_balance(DAVE);
        assert_eq!(dave_reserved, OutcomeBond::get() + OutcomeFactor::get());

        let eve_reserved = Balances::reserved_balance(EVE);
        assert_eq!(eve_reserved, OutcomeBond::get() + 2 * OutcomeFactor::get());

        // check disputes length
        let disputes = zrml_simple_disputes::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 3);

        // make sure the old mappings of market id per dispute block are erased
        let market_ids_1 = MarketIdsPerDisputeBlock::<Runtime>::get(
            dispute_at_0 + market.deadlines.dispute_duration,
        );
        assert_eq!(market_ids_1.len(), 0);

        let market_ids_2 = MarketIdsPerDisputeBlock::<Runtime>::get(
            dispute_at_1 + market.deadlines.dispute_duration,
        );
        assert_eq!(market_ids_2.len(), 0);

        let market_ids_3 = MarketIdsPerDisputeBlock::<Runtime>::get(
            dispute_at_2 + market.deadlines.dispute_duration,
        );
        assert_eq!(market_ids_3.len(), 1);

        run_blocks(market.deadlines.dispute_duration);

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);
        let disputes = zrml_simple_disputes::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 0);

        assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(CHARLIE), 0));

        // Make sure rewards are right:
        //
        // Slashed amounts:
        //     - Dave's reserve: OutcomeBond::get() + OutcomeFactor::get()
        //     - Alice's oracle bond: OracleBond::get()
        // simple-disputes reward: OutcomeBond::get() + OutcomeFactor::get()
        // Charlie gets OracleBond, because the dispute was justified.
        // A dispute is justified if the oracle's report is different to the final outcome.
        //
        // Charlie and Eve each receive half of the simple-disputes reward as bounty.
        let dave_reserved = OutcomeBond::get() + OutcomeFactor::get();
        let total_slashed = dave_reserved;

        let charlie_balance = Balances::free_balance(CHARLIE);
        assert_eq!(charlie_balance, 1_000 * BASE + OracleBond::get() + total_slashed / 2);
        let charlie_reserved_2 = Balances::reserved_balance(CHARLIE);
        assert_eq!(charlie_reserved_2, 0);
        let eve_balance = Balances::free_balance(EVE);
        assert_eq!(eve_balance, 1_000 * BASE + total_slashed / 2);

        let dave_balance = Balances::free_balance(DAVE);
        assert_eq!(dave_balance, 1_000 * BASE - dave_reserved);

        let alice_balance = Balances::free_balance(ALICE);
        assert_eq!(alice_balance, 1_000 * BASE - OracleBond::get());

        // bob kinda gets away scot-free since Alice is held responsible
        // for her designated reporter
        let bob_balance = Balances::free_balance(BOB);
        assert_eq!(bob_balance, 1_000 * BASE);

        assert!(market_after.bonds.creation.unwrap().is_settled);
        assert!(market_after.bonds.oracle.unwrap().is_settled);
        assert!(market_after.bonds.dispute.unwrap().is_settled);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn it_resolves_a_disputed_court_market() {
    let test = |base_asset: Asset<MarketId>| {
        let juror_0 = 1000;
        let juror_1 = 1001;
        let juror_2 = 1002;
        let juror_3 = 1003;
        let juror_4 = 1004;
        let juror_5 = 1005;

        for j in &[juror_0, juror_1, juror_2, juror_3, juror_4, juror_5] {
            let amount = MinJurorStake::get() + *j;
            assert_ok!(AssetManager::deposit(Asset::Ztg, j, amount + SENTINEL_AMOUNT));
            assert_ok!(Court::join_court(RuntimeOrigin::signed(*j), amount));
        }

        // just to have enough jurors for the dispute
        for j in 1006..(1006 + Court::necessary_draws_weight(0usize) as u32) {
            let juror = j as u128;
            let amount = MinJurorStake::get() + juror;
            assert_ok!(AssetManager::deposit(Asset::Ztg, &juror, amount + SENTINEL_AMOUNT));
            assert_ok!(Court::join_court(RuntimeOrigin::signed(juror), amount));
        }

        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::Lmsr,
        ));

        let market_id = 0;
        let market = MarketCommons::market(&0).unwrap();

        let report_at = end + market.deadlines.grace_period + 1;
        run_to_block(report_at);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            market_id,
            OutcomeReport::Categorical(0)
        ));

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), market_id,));

        let court = zrml_court::Courts::<Runtime>::get(market_id).unwrap();
        let vote_start = court.round_ends.pre_vote + 1;

        run_to_block(vote_start);

        // overwrite draws to disregard randomness
        zrml_court::SelectedDraws::<Runtime>::remove(market_id);
        let mut draws = zrml_court::SelectedDraws::<Runtime>::get(market_id);
        for juror in &[juror_0, juror_1, juror_2, juror_3, juror_4, juror_5] {
            let draw = Draw {
                court_participant: *juror,
                weight: 1,
                vote: Vote::Drawn,
                slashable: MinJurorStake::get(),
            };
            let index = draws
                .binary_search_by_key(juror, |draw| draw.court_participant)
                .unwrap_or_else(|j| j);
            draws.try_insert(index, draw).unwrap();
        }
        let old_draws = draws.clone();
        zrml_court::SelectedDraws::<Runtime>::insert(market_id, draws);

        let salt = <Runtime as frame_system::Config>::Hash::default();

        // outcome_0 is the plurality decision => right outcome
        let outcome_0 = OutcomeReport::Categorical(0);
        let vote_item_0 = VoteItem::Outcome(outcome_0.clone());
        // outcome_1 is the wrong outcome
        let outcome_1 = OutcomeReport::Categorical(1);
        let vote_item_1 = VoteItem::Outcome(outcome_1);

        let commitment_0 = BlakeTwo256::hash_of(&(juror_0, vote_item_0.clone(), salt));
        assert_ok!(Court::vote(RuntimeOrigin::signed(juror_0), market_id, commitment_0));

        // juror_1 votes for non-plurality outcome => slashed later
        let commitment_1 = BlakeTwo256::hash_of(&(juror_1, vote_item_1.clone(), salt));
        assert_ok!(Court::vote(RuntimeOrigin::signed(juror_1), market_id, commitment_1));

        let commitment_2 = BlakeTwo256::hash_of(&(juror_2, vote_item_0.clone(), salt));
        assert_ok!(Court::vote(RuntimeOrigin::signed(juror_2), market_id, commitment_2));

        let commitment_3 = BlakeTwo256::hash_of(&(juror_3, vote_item_0.clone(), salt));
        assert_ok!(Court::vote(RuntimeOrigin::signed(juror_3), market_id, commitment_3));

        // juror_4 fails to vote in time

        let commitment_5 = BlakeTwo256::hash_of(&(juror_5, vote_item_0.clone(), salt));
        assert_ok!(Court::vote(RuntimeOrigin::signed(juror_5), market_id, commitment_5));

        // juror_3 is denounced by juror_0 => slashed later
        assert_ok!(Court::denounce_vote(
            RuntimeOrigin::signed(juror_0),
            market_id,
            juror_3,
            vote_item_0.clone(),
            salt
        ));

        let aggregation_start = court.round_ends.vote + 1;
        run_to_block(aggregation_start);

        assert_ok!(Court::reveal_vote(
            RuntimeOrigin::signed(juror_0),
            market_id,
            vote_item_0.clone(),
            salt
        ));
        assert_ok!(Court::reveal_vote(
            RuntimeOrigin::signed(juror_1),
            market_id,
            vote_item_1,
            salt
        ));

        let wrong_salt = BlakeTwo256::hash_of(&69);
        assert_noop!(
            Court::reveal_vote(
                RuntimeOrigin::signed(juror_2),
                market_id,
                vote_item_0.clone(),
                wrong_salt
            ),
            CError::<Runtime>::CommitmentHashMismatch
        );
        assert_ok!(Court::reveal_vote(
            RuntimeOrigin::signed(juror_2),
            market_id,
            vote_item_0.clone(),
            salt
        ));

        assert_noop!(
            Court::reveal_vote(
                RuntimeOrigin::signed(juror_3),
                market_id,
                vote_item_0.clone(),
                salt
            ),
            CError::<Runtime>::VoteAlreadyDenounced
        );

        assert_noop!(
            Court::reveal_vote(
                RuntimeOrigin::signed(juror_4),
                market_id,
                vote_item_0.clone(),
                salt
            ),
            CError::<Runtime>::JurorDidNotVote
        );

        // juror_5 fails to reveal in time

        let resolve_at = court.round_ends.appeal;
        let market_ids = MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at);
        assert_eq!(market_ids.len(), 1);

        run_blocks(resolve_at);

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);
        assert_eq!(market_after.resolved_outcome, Some(outcome_0));
        let court_after = zrml_court::Courts::<Runtime>::get(market_id).unwrap();
        assert_eq!(court_after.status, CourtStatus::Closed { winner: vote_item_0 });

        let free_juror_0_before = Balances::free_balance(juror_0);
        let free_juror_1_before = Balances::free_balance(juror_1);
        let free_juror_2_before = Balances::free_balance(juror_2);
        let free_juror_3_before = Balances::free_balance(juror_3);
        let free_juror_4_before = Balances::free_balance(juror_4);
        let free_juror_5_before = Balances::free_balance(juror_5);

        assert_ok!(Court::reassign_court_stakes(RuntimeOrigin::signed(juror_0), market_id));

        let free_juror_0_after = Balances::free_balance(juror_0);
        let slashable_juror_0 =
            old_draws.iter().find(|draw| draw.court_participant == juror_0).unwrap().slashable;
        let free_juror_1_after = Balances::free_balance(juror_1);
        let slashable_juror_1 =
            old_draws.iter().find(|draw| draw.court_participant == juror_1).unwrap().slashable;
        let free_juror_2_after = Balances::free_balance(juror_2);
        let slashable_juror_2 =
            old_draws.iter().find(|draw| draw.court_participant == juror_2).unwrap().slashable;
        let free_juror_3_after = Balances::free_balance(juror_3);
        let slashable_juror_3 =
            old_draws.iter().find(|draw| draw.court_participant == juror_3).unwrap().slashable;
        let free_juror_4_after = Balances::free_balance(juror_4);
        let slashable_juror_4 =
            old_draws.iter().find(|draw| draw.court_participant == juror_4).unwrap().slashable;
        let free_juror_5_after = Balances::free_balance(juror_5);
        let slashable_juror_5 =
            old_draws.iter().find(|draw| draw.court_participant == juror_5).unwrap().slashable;

        let mut total_slashed = 0;
        // juror_1 voted for the wrong outcome => slashed
        assert_eq!(free_juror_1_before - free_juror_1_after, slashable_juror_1);
        total_slashed += slashable_juror_1;
        // juror_3 was denounced by juror_0 => slashed
        assert_eq!(free_juror_3_before - free_juror_3_after, slashable_juror_3);
        total_slashed += slashable_juror_3;
        // juror_4 failed to vote => slashed
        assert_eq!(free_juror_4_before - free_juror_4_after, slashable_juror_4);
        total_slashed += slashable_juror_4;
        // juror_5 failed to reveal => slashed
        assert_eq!(free_juror_5_before - free_juror_5_after, slashable_juror_5);
        total_slashed += slashable_juror_5;
        // juror_0 and juror_2 voted for the right outcome => rewarded
        let total_winner_stake = slashable_juror_0 + slashable_juror_2;
        let juror_0_share = Perquintill::from_rational(slashable_juror_0, total_winner_stake);
        assert_eq!(free_juror_0_after, free_juror_0_before + juror_0_share * total_slashed);
        let juror_2_share = Perquintill::from_rational(slashable_juror_2, total_winner_stake);
        assert_eq!(free_juror_2_after, free_juror_2_before + juror_2_share * total_slashed);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

fn simulate_appeal_cycle(market_id: MarketId) {
    let court = zrml_court::Courts::<Runtime>::get(market_id).unwrap();
    let vote_start = court.round_ends.pre_vote + 1;

    run_to_block(vote_start);

    let salt = <Runtime as frame_system::Config>::Hash::default();

    let wrong_outcome = OutcomeReport::Categorical(1);
    let wrong_vote_item = VoteItem::Outcome(wrong_outcome);

    let draws = zrml_court::SelectedDraws::<Runtime>::get(market_id);
    for draw in &draws {
        let commitment =
            BlakeTwo256::hash_of(&(draw.court_participant, wrong_vote_item.clone(), salt));
        assert_ok!(Court::vote(
            RuntimeOrigin::signed(draw.court_participant),
            market_id,
            commitment
        ));
    }

    let aggregation_start = court.round_ends.vote + 1;
    run_to_block(aggregation_start);

    for draw in draws {
        assert_ok!(Court::reveal_vote(
            RuntimeOrigin::signed(draw.court_participant),
            market_id,
            wrong_vote_item.clone(),
            salt,
        ));
    }

    let resolve_at = court.round_ends.appeal;
    let market_ids = MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at);
    assert_eq!(market_ids.len(), 1);

    run_to_block(resolve_at - 1);

    let market_after = MarketCommons::market(&0).unwrap();
    assert_eq!(market_after.status, MarketStatus::Disputed);
}

#[test]
fn it_appeals_a_court_market_to_global_dispute() {
    let test = |base_asset: Asset<MarketId>| {
        let mut free_before = BTreeMap::new();
        let jurors = 1000..(1000 + MaxSelectedDraws::get() as u128);
        for j in jurors {
            let amount = MinJurorStake::get() + j;
            assert_ok!(AssetManager::deposit(Asset::Ztg, &j, amount + SENTINEL_AMOUNT));
            assert_ok!(Court::join_court(RuntimeOrigin::signed(j), amount));
            free_before.insert(j, Balances::free_balance(j));
        }

        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::Lmsr,
        ));

        let market_id = 0;
        let market = MarketCommons::market(&0).unwrap();

        let report_at = end + market.deadlines.grace_period + 1;
        run_to_block(report_at);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            market_id,
            OutcomeReport::Categorical(0)
        ));

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), market_id,));

        for _ in 0..(MaxAppeals::get() - 1) {
            simulate_appeal_cycle(market_id);
            assert_ok!(Court::appeal(RuntimeOrigin::signed(BOB), market_id));
        }

        let court = zrml_court::Courts::<Runtime>::get(market_id).unwrap();
        let appeals = court.appeals;
        assert_eq!(appeals.len(), (MaxAppeals::get() - 1) as usize);

        assert_noop!(
            PredictionMarkets::start_global_dispute(RuntimeOrigin::signed(BOB), market_id),
            Error::<Runtime>::MarketDisputeMechanismNotFailed
        );

        simulate_appeal_cycle(market_id);
        assert_ok!(Court::appeal(RuntimeOrigin::signed(BOB), market_id));

        assert_noop!(
            Court::appeal(RuntimeOrigin::signed(BOB), market_id),
            CError::<Runtime>::MaxAppealsReached
        );

        assert!(!GlobalDisputes::does_exist(&market_id));

        assert_ok!(PredictionMarkets::start_global_dispute(RuntimeOrigin::signed(BOB), market_id));

        let now = <frame_system::Pallet<Runtime>>::block_number();

        assert!(GlobalDisputes::does_exist(&market_id));
        System::assert_last_event(Event::GlobalDisputeStarted(market_id).into());

        // report check
        let possession: PossessionOf<Runtime> =
            Possession::Shared { owners: frame_support::BoundedVec::try_from(vec![BOB]).unwrap() };
        let outcome_info = OutcomeInfo { outcome_sum: Zero::zero(), possession };
        assert_eq!(
            Outcomes::<Runtime>::get(market_id, &OutcomeReport::Categorical(0)).unwrap(),
            outcome_info
        );

        let add_outcome_end = now + GlobalDisputes::get_add_outcome_period();
        let vote_end = add_outcome_end + GlobalDisputes::get_vote_period();
        let market_ids = MarketIdsPerDisputeBlock::<Runtime>::get(vote_end);
        assert_eq!(market_ids, vec![market_id]);
        assert!(GlobalDisputes::is_active(&market_id));

        assert_noop!(
            PredictionMarkets::start_global_dispute(RuntimeOrigin::signed(CHARLIE), market_id),
            Error::<Runtime>::GlobalDisputeExistsAlready
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test_case(MarketStatus::Active; "active")]
#[test_case(MarketStatus::Closed; "closed")]
#[test_case(MarketStatus::Proposed; "proposed")]
#[test_case(MarketStatus::Resolved; "resolved")]
fn dispute_fails_unless_reported_or_disputed_market(status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::Lmsr,
        );

        assert_ok!(MarketCommons::mutate_market(&0, |market_inner| {
            market_inner.status = status;
            Ok(())
        }));

        assert_noop!(
            PredictionMarkets::dispute(RuntimeOrigin::signed(EVE), 0),
            Error::<Runtime>::InvalidMarketStatus
        );
    });
}

#[test]
fn start_global_dispute_fails_on_wrong_mdm() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..2),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MaxDisputes::get() + 1),
            Some(MarketDisputeMechanism::Authorized),
            ScoringRule::Lmsr,
        ));
        let market_id = MarketCommons::latest_market_id().unwrap();

        let market = MarketCommons::market(&market_id).unwrap();
        let grace_period = market.deadlines.grace_period;
        run_to_block(end + grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            market_id,
            OutcomeReport::Categorical(0)
        ));
        let dispute_at_0 = end + grace_period + 2;
        run_to_block(dispute_at_0);

        // only one dispute allowed for authorized mdm
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), market_id,));
        run_blocks(1);
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        assert_noop!(
            PredictionMarkets::start_global_dispute(RuntimeOrigin::signed(CHARLIE), market_id),
            Error::<Runtime>::InvalidDisputeMechanism
        );
    });
}

#[test]
fn it_allows_to_redeem_shares() {
    let test = |base_asset: Asset<MarketId>| {
        let end = 2;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::Lmsr,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(CHARLIE), 0, CENT));
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Resolved);

        assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(CHARLIE), 0));
        let bal = Balances::free_balance(CHARLIE);
        assert_eq!(bal, 1_000 * BASE);
        System::assert_last_event(
            Event::TokensRedeemed(0, Asset::CategoricalOutcome(0, 1), CENT, CENT, CHARLIE).into(),
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test_case(ScoringRule::Parimutuel; "parimutuel")]
fn redeem_shares_fails_if_invalid_resolution_mechanism(scoring_rule: ScoringRule) {
    let test = |base_asset: Asset<MarketId>| {
        let end = 2;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..end,
            scoring_rule,
        );

        assert_ok!(MarketCommons::mutate_market(&0, |market_inner| {
            market_inner.status = MarketStatus::Resolved;
            Ok(())
        }));

        assert_noop!(
            PredictionMarkets::redeem_shares(RuntimeOrigin::signed(CHARLIE), 0),
            Error::<Runtime>::InvalidResolutionMechanism
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn the_entire_market_lifecycle_works_with_timestamps() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr
        ));

        // is ok
        assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(BOB), 0, CENT));
        let market = MarketCommons::market(&0).unwrap();

        // set the timestamp
        set_timestamp_for_on_initialize(100_000_000);
        run_to_block(2); // Trigger `on_initialize`; must be at least block #2.
        let grace_period: u64 = market.deadlines.grace_period * MILLISECS_PER_BLOCK as u64;
        Timestamp::set_timestamp(100_000_000 + grace_period);

        assert_noop!(
            PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(BOB), 0, CENT),
            Error::<Runtime>::MarketIsNotActive,
        );

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
    });
}

#[test]
fn full_scalar_market_lifecycle() {
    let test = |base_asset: Asset<MarketId>| {
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(3),
            MarketCreation::Permissionless,
            MarketType::Scalar(10..=30),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr
        ));

        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(CHARLIE),
            0,
            100 * BASE
        ));

        // check balances
        let assets = PredictionMarkets::outcome_assets(0, &MarketCommons::market(&0).unwrap());
        assert_eq!(assets.len(), 2);
        for asset in assets.iter() {
            let bal = Tokens::free_balance(*asset, &CHARLIE);
            assert_eq!(bal, 100 * BASE);
        }
        let market = MarketCommons::market(&0).unwrap();

        set_timestamp_for_on_initialize(100_000_000);
        let report_at = 2;
        run_to_block(report_at); // Trigger `on_initialize`; must be at least block #2.
        let grace_period: u64 = market.deadlines.grace_period * MILLISECS_PER_BLOCK as u64;
        Timestamp::set_timestamp(100_000_000 + grace_period);

        // report
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Scalar(100)
        ));

        let market_after_report = MarketCommons::market(&0).unwrap();
        assert!(market_after_report.report.is_some());
        let report = market_after_report.report.unwrap();
        assert_eq!(report.at, report_at);
        assert_eq!(report.by, BOB);
        assert_eq!(report.outcome, OutcomeReport::Scalar(100));

        // dispute
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(DAVE), 0));
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(DAVE),
            0,
            OutcomeReport::Scalar(25)
        ));
        let disputes = zrml_simple_disputes::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 1);

        run_blocks(market.deadlines.dispute_duration);

        let market_after_resolve = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after_resolve.status, MarketStatus::Resolved);
        let disputes = zrml_simple_disputes::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 0);

        // give EVE some shares
        assert_ok!(Tokens::transfer(
            RuntimeOrigin::signed(CHARLIE),
            EVE,
            Asset::ScalarOutcome(0, ScalarPosition::Short),
            50 * BASE
        ));

        assert_eq!(
            Tokens::free_balance(Asset::ScalarOutcome(0, ScalarPosition::Short), &CHARLIE),
            50 * BASE
        );

        assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(CHARLIE), 0));
        for asset in assets.iter() {
            let bal = Tokens::free_balance(*asset, &CHARLIE);
            assert_eq!(bal, 0);
        }

        // check payouts is right for each CHARLIE and EVE
        let base_asset_bal_charlie = AssetManager::free_balance(base_asset, &CHARLIE);
        let base_asset_bal_eve = AssetManager::free_balance(base_asset, &EVE);
        assert_eq!(base_asset_bal_charlie, 98750 * CENT); // 75 (LONG) + 12.5 (SHORT) + 900 (balance)
        assert_eq!(base_asset_bal_eve, 1000 * BASE);
        System::assert_has_event(
            Event::TokensRedeemed(
                0,
                Asset::ScalarOutcome(0, ScalarPosition::Long),
                100 * BASE,
                75 * BASE,
                CHARLIE,
            )
            .into(),
        );
        System::assert_has_event(
            Event::TokensRedeemed(
                0,
                Asset::ScalarOutcome(0, ScalarPosition::Short),
                50 * BASE,
                1250 * CENT, // 12.5
                CHARLIE,
            )
            .into(),
        );

        assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(EVE), 0));
        let base_asset_bal_eve_after = AssetManager::free_balance(base_asset, &EVE);
        assert_eq!(base_asset_bal_eve_after, 101250 * CENT); // 12.5 (SHORT) + 1000 (balance)
        System::assert_last_event(
            Event::TokensRedeemed(
                0,
                Asset::ScalarOutcome(0, ScalarPosition::Short),
                50 * BASE,
                1250 * CENT, // 12.5
                EVE,
            )
            .into(),
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn scalar_market_correctly_resolves_on_out_of_range_outcomes_below_threshold() {
    let test = |base_asset: Asset<MarketId>| {
        scalar_market_correctly_resolves_common(base_asset, 50);
        assert_eq!(AssetManager::free_balance(base_asset, &CHARLIE), 900 * BASE);
        assert_eq!(AssetManager::free_balance(base_asset, &EVE), 1100 * BASE);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn scalar_market_correctly_resolves_on_out_of_range_outcomes_above_threshold() {
    let test = |base_asset: Asset<MarketId>| {
        scalar_market_correctly_resolves_common(base_asset, 250);
        assert_eq!(AssetManager::free_balance(base_asset, &CHARLIE), 1000 * BASE);
        assert_eq!(AssetManager::free_balance(base_asset, &EVE), 1000 * BASE);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn authorized_correctly_resolves_disputed_market() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Authorized),
            ScoringRule::Lmsr,
        ));
        assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(CHARLIE), 0, CENT));

        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));

        let charlie_balance = AssetManager::free_balance(base_asset, &CHARLIE);
        assert_eq!(charlie_balance, 1_000 * BASE - CENT);

        let dispute_at = grace_period + 1 + 1;
        run_to_block(dispute_at);
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0,));

        if base_asset == Asset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT - DisputeBond::get());
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - DisputeBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT);
        }

        // Fred authorizses an outcome, but fat-fingers it on the first try.
        assert_ok!(Authorized::authorize_market_outcome(
            RuntimeOrigin::signed(AuthorizedDisputeResolutionUser::get()),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(Authorized::authorize_market_outcome(
            RuntimeOrigin::signed(AuthorizedDisputeResolutionUser::get()),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        // check everyone's deposits
        let charlie_reserved = Balances::reserved_balance(CHARLIE);
        assert_eq!(charlie_reserved, DisputeBond::get());

        let market_ids_1 = MarketIdsPerDisputeBlock::<Runtime>::get(
            dispute_at + <Runtime as zrml_authorized::Config>::CorrectionPeriod::get(),
        );
        assert_eq!(market_ids_1.len(), 1);

        if base_asset == Asset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT - DisputeBond::get());
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - DisputeBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT);
        }

        run_blocks(<Runtime as zrml_authorized::Config>::CorrectionPeriod::get() - 1);

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Disputed);

        if base_asset == Asset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT - DisputeBond::get());
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - DisputeBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT);
        }

        run_blocks(1);

        if base_asset == Asset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT + OracleBond::get());
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE + OracleBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT);
        }

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);
        let disputes = zrml_simple_disputes::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 0);

        assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(CHARLIE), 0));

        if base_asset == Asset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE + OracleBond::get());
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE + OracleBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE);
        }
        let charlie_reserved_2 = AssetManager::reserved_balance(Asset::Ztg, &CHARLIE);
        assert_eq!(charlie_reserved_2, 0);

        let alice_balance = AssetManager::free_balance(Asset::Ztg, &ALICE);
        assert_eq!(alice_balance, 1_000 * BASE - OracleBond::get());

        // bob kinda gets away scot-free since Alice is held responsible
        // for her designated reporter
        let bob_balance = AssetManager::free_balance(Asset::Ztg, &BOB);
        assert_eq!(bob_balance, 1_000 * BASE);

        assert!(market_after.bonds.creation.unwrap().is_settled);
        assert!(market_after.bonds.oracle.unwrap().is_settled);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_on_oracle_report()
 {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_to_block(grace_period + market.deadlines.dispute_duration + 1);
        check_reserve(&ALICE, 0);
        assert_eq!(
            Balances::free_balance(ALICE),
            alice_balance_before + ValidityBond::get() + OracleBond::get()
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_on_outsider_report()
 {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..100),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());

        let charlie_balance_before = Balances::free_balance(CHARLIE);
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        let report_at = grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(report_at);

        assert!(market.bonds.outsider.is_none());
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.bonds.outsider, Some(Bond::new(CHARLIE, OutsiderBond::get())));
        check_reserve(&CHARLIE, OutsiderBond::get());
        assert_eq!(Balances::free_balance(CHARLIE), charlie_balance_before - OutsiderBond::get());
        let charlie_balance_before = Balances::free_balance(CHARLIE);

        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // Check that validity bond didn't get slashed, but oracle bond did
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before + ValidityBond::get());

        check_reserve(&CHARLIE, 0);
        // Check that the outsider gets the OracleBond together with the OutsiderBond
        assert_eq!(
            Balances::free_balance(CHARLIE),
            charlie_balance_before + OracleBond::get() + OutsiderBond::get()
        );
        let market = MarketCommons::market(&0).unwrap();
        assert!(market.bonds.outsider.unwrap().is_settled);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn outsider_reports_wrong_outcome() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();

        let end = 100;
        let alice_balance_before = Balances::free_balance(ALICE);
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));

        let outsider = CHARLIE;

        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        let report_at = grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(report_at);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(outsider),
            0,
            OutcomeReport::Categorical(1)
        ));

        let outsider_balance_before = Balances::free_balance(outsider);
        check_reserve(&outsider, OutsiderBond::get());

        let dispute_at_0 = report_at + 1;
        run_to_block(dispute_at_0);
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(EVE), 0,));
        check_reserve(&EVE, DisputeBond::get());

        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(DAVE),
            0,
            OutcomeReport::Categorical(0)
        ));

        let outcome_bond = zrml_simple_disputes::default_outcome_bond::<Runtime>(0);

        check_reserve(&DAVE, outcome_bond);

        let eve_balance_before = Balances::free_balance(EVE);
        let dave_balance_before = Balances::free_balance(DAVE);

        // on_resolution called
        run_blocks(market.deadlines.dispute_duration);

        assert_eq!(Balances::free_balance(ALICE), alice_balance_before - OracleBond::get());

        check_reserve(&outsider, 0);
        assert_eq!(Balances::free_balance(outsider), outsider_balance_before);

        // disputor EVE gets the OracleBond and OutsiderBond and DisputeBond
        assert_eq!(
            Balances::free_balance(EVE),
            eve_balance_before + DisputeBond::get() + OutsiderBond::get() + OracleBond::get()
        );
        // DAVE gets his outcome bond back
        assert_eq!(Balances::free_balance(DAVE), dave_balance_before + outcome_bond);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_approved_advised_market_on_oracle_report()
 {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        assert_ok!(PredictionMarkets::approve_market(RuntimeOrigin::signed(SUDO), 0));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        let report_at = grace_period + 1;
        run_to_block(report_at);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // Check that nothing got slashed
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before + OracleBond::get());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_approved_advised_market_on_outsider_report()
 {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        assert_ok!(PredictionMarkets::approve_market(RuntimeOrigin::signed(SUDO), 0));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        let report_at = grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(report_at);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        // Check that oracle bond got slashed
        check_reserve(&ALICE, 0);
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_with_correct_disputed_outcome_with_oracle_report()
 {
    // Oracle reports in time but incorrect report, so OracleBond gets slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0,));
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is slashed
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before + ValidityBond::get());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_approved_advised_market_with_correct_disputed_outcome_with_oracle_report()
 {
    // Oracle reports in time but incorrect report, so OracleBond gets slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        assert_ok!(PredictionMarkets::approve_market(RuntimeOrigin::signed(SUDO), 0));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0,));
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is slashed
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_with_wrong_disputed_outcome_with_oracle_report()
 {
    // Oracle reports in time and correct report, so OracleBond does not get slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(EVE), 0,));
        // EVE disputes with wrong outcome
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is not slashed
        assert_eq!(
            Balances::free_balance(ALICE),
            alice_balance_before + ValidityBond::get() + OracleBond::get()
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_advised_approved_market_with_wrong_disputed_outcome_with_oracle_report()
 {
    // Oracle reports in time and correct report, so OracleBond does not get slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        assert_ok!(PredictionMarkets::approve_market(RuntimeOrigin::signed(SUDO), 0));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(EVE), 0,));
        // EVE disputes with wrong outcome
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is not slashed
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before + OracleBond::get());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_with_disputed_outcome_with_outsider_report()
 {
    // Oracle does not report in time, so OracleBond gets slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));

        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());

        let outsider = CHARLIE;

        let market = MarketCommons::market(&0).unwrap();
        let after_oracle_duration =
            end + market.deadlines.grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(after_oracle_duration);
        // CHARLIE is not an Oracle
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(outsider),
            0,
            OutcomeReport::Categorical(0)
        ));
        let outsider_balance_before = Balances::free_balance(outsider);
        check_reserve(&outsider, OutsiderBond::get());

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(EVE), 0,));
        // EVE disputes with wrong outcome
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(FRED),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is slashed
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before + ValidityBond::get());

        check_reserve(&outsider, 0);
        assert_eq!(
            Balances::free_balance(outsider),
            outsider_balance_before + OracleBond::get() + OutsiderBond::get()
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_advised_approved_market_with_disputed_outcome_with_outsider_report()
 {
    // Oracle does not report in time, so OracleBond gets slashed on resolution
    // NOTE: Bonds are always in ZTG
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            base_asset,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));

        let outsider = CHARLIE;

        assert_ok!(PredictionMarkets::approve_market(RuntimeOrigin::signed(SUDO), 0));
        let alice_balance_before = Balances::free_balance(ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let after_oracle_duration =
            end + market.deadlines.grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(after_oracle_duration);
        // CHARLIE is not an Oracle
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(outsider),
            0,
            OutcomeReport::Categorical(0)
        ));
        let outsider_balance_before = Balances::free_balance(outsider);
        check_reserve(&outsider, OutsiderBond::get());

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(EVE), 0,));
        // EVE disputes with wrong outcome
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(SimpleDisputes::suggest_outcome(
            RuntimeOrigin::signed(FRED),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is slashed
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before);

        check_reserve(&outsider, 0);
        assert_eq!(
            Balances::free_balance(outsider),
            outsider_balance_before + OracleBond::get() + OutsiderBond::get()
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn create_market_and_deploy_pool_works() {
    ExtBuilder::default().build().execute_with(|| {
        let creator = ALICE;
        let creator_fee = Perbill::from_parts(1);
        let oracle = BOB;
        let period = MarketPeriod::Block(1..2);
        let deadlines = Deadlines {
            grace_period: 1,
            oracle_duration: <Runtime as crate::Config>::MinOracleDuration::get() + 2,
            dispute_duration: <Runtime as crate::Config>::MinDisputeDuration::get() + 3,
        };
        let metadata = gen_metadata(0x99);
        let MultiHash::Sha3_384(multihash) = metadata;
        let market_type = MarketType::Categorical(7);
        let dispute_mechanism = Some(MarketDisputeMechanism::Authorized);
        let amount = 1234567890;
        let swap_prices = vec![50 * CENT, 50 * CENT];
        let swap_fee = CENT;
        let market_id = 0;
        assert_ok!(PredictionMarkets::create_market_and_deploy_pool(
            RuntimeOrigin::signed(creator),
            Asset::Ztg,
            creator_fee,
            oracle,
            period.clone(),
            deadlines,
            metadata,
            market_type.clone(),
            dispute_mechanism.clone(),
            amount,
            swap_prices.clone(),
            swap_fee,
        ));
        let market = MarketCommons::market(&0).unwrap();
        let bonds = MarketBonds {
            creation: Some(Bond::new(ALICE, <Runtime as Config>::ValidityBond::get())),
            oracle: Some(Bond::new(ALICE, <Runtime as Config>::OracleBond::get())),
            outsider: None,
            dispute: None,
            close_dispute: None,
            close_request: None,
        };
        assert_eq!(market.creator, creator);
        assert_eq!(market.creation, MarketCreation::Permissionless);
        assert_eq!(market.creator_fee, creator_fee);
        assert_eq!(market.oracle, oracle);
        assert_eq!(market.metadata, multihash);
        assert_eq!(market.market_type, market_type);
        assert_eq!(market.period, period);
        assert_eq!(market.deadlines, deadlines);
        assert_eq!(market.scoring_rule, ScoringRule::Lmsr);
        assert_eq!(market.status, MarketStatus::Active);
        assert_eq!(market.report, None);
        assert_eq!(market.resolved_outcome, None);
        assert_eq!(market.dispute_mechanism, dispute_mechanism);
        assert_eq!(market.bonds, bonds);
        // Check that the correct amount of full sets were bought.
        assert_eq!(
            AssetManager::free_balance(Asset::CategoricalOutcome(market_id, 0), &ALICE),
            amount
        );
        assert!(DeployPoolMock::called_once_with(
            creator,
            market_id,
            amount,
            swap_prices,
            swap_fee
        ));
    });
}

#[test]
fn trusted_market_complete_lifecycle() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 3;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            Deadlines {
                grace_period: 0,
                oracle_duration: <Runtime as crate::Config>::MinOracleDuration::get(),
                dispute_duration: Zero::zero(),
            },
            gen_metadata(0x99),
            MarketCreation::Permissionless,
            MarketType::Categorical(3),
            None,
            ScoringRule::Lmsr,
        ));
        let market_id = 0;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(FRED),
            market_id,
            BASE
        ));
        run_to_block(end);
        let outcome = OutcomeReport::Categorical(1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            market_id,
            outcome.clone()
        ));
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Resolved);
        assert_eq!(market.report, Some(Report { at: end, by: BOB, outcome: outcome.clone() }));
        assert_eq!(market.resolved_outcome, Some(outcome));
        assert_eq!(market.dispute_mechanism, None);
        assert!(market.bonds.oracle.unwrap().is_settled);
        assert_eq!(market.bonds.outsider, None);
        assert_eq!(market.bonds.dispute, None);
        assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(FRED), market_id));
        // Ensure that we don't accidentally leave any artifacts.
        assert!(MarketIdsPerDisputeBlock::<Runtime>::iter().next().is_none());
        assert!(MarketIdsPerReportBlock::<Runtime>::iter().next().is_none());
    });
}

#[test]
fn close_trusted_market_works() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 10;
        let market_creator = ALICE;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(market_creator),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            Deadlines {
                grace_period: 0,
                oracle_duration: <Runtime as crate::Config>::MinOracleDuration::get(),
                dispute_duration: Zero::zero(),
            },
            gen_metadata(0x99),
            MarketCreation::Permissionless,
            MarketType::Categorical(3),
            None,
            ScoringRule::Lmsr,
        ));

        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.dispute_mechanism, None);

        let new_end = end / 2;
        assert_ne!(new_end, end);
        run_to_block(new_end);

        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Active);

        let auto_closes = MarketIdsPerCloseBlock::<Runtime>::get(end);
        assert_eq!(auto_closes.first().cloned().unwrap(), market_id);

        assert_noop!(
            PredictionMarkets::close_trusted_market(RuntimeOrigin::signed(BOB), market_id),
            Error::<Runtime>::CallerNotMarketCreator
        );

        assert_ok!(PredictionMarkets::close_trusted_market(
            RuntimeOrigin::signed(market_creator),
            market_id
        ));
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.period, MarketPeriod::Block(0..new_end));
        assert_eq!(market.status, MarketStatus::Closed);

        let auto_closes = MarketIdsPerCloseBlock::<Runtime>::get(end);
        assert_eq!(auto_closes.len(), 0);
    });
}

#[test]
fn close_trusted_market_fails_if_not_trusted() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 10;
        let market_creator = ALICE;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(market_creator),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            Deadlines {
                grace_period: 0,
                oracle_duration: <Runtime as crate::Config>::MinOracleDuration::get(),
                dispute_duration: <Runtime as crate::Config>::MinDisputeDuration::get(),
            },
            gen_metadata(0x99),
            MarketCreation::Permissionless,
            MarketType::Categorical(3),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::Lmsr,
        ));

        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.dispute_mechanism, Some(MarketDisputeMechanism::Court));

        run_to_block(end / 2);

        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Active);

        assert_noop!(
            PredictionMarkets::close_trusted_market(
                RuntimeOrigin::signed(market_creator),
                market_id
            ),
            Error::<Runtime>::MarketIsNotTrusted
        );
    });
}

#[test_case(MarketStatus::Closed; "closed")]
#[test_case(MarketStatus::Proposed; "proposed")]
#[test_case(MarketStatus::Resolved; "resolved")]
#[test_case(MarketStatus::Disputed; "disputed")]
#[test_case(MarketStatus::Reported; "report")]
fn close_trusted_market_fails_if_invalid_market_state(status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let end = 10;
        let market_creator = ALICE;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(market_creator),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            Deadlines {
                grace_period: 0,
                oracle_duration: <Runtime as crate::Config>::MinOracleDuration::get(),
                dispute_duration: Zero::zero(),
            },
            gen_metadata(0x99),
            MarketCreation::Permissionless,
            MarketType::Categorical(3),
            None,
            ScoringRule::Lmsr,
        ));

        let market_id = 0;
        assert_ok!(MarketCommons::mutate_market(&market_id, |market| {
            market.status = status;
            Ok(())
        }));

        assert_noop!(
            PredictionMarkets::close_trusted_market(
                RuntimeOrigin::signed(market_creator),
                market_id
            ),
            Error::<Runtime>::MarketIsNotActive
        );
    });
}

// Common code of `scalar_market_correctly_resolves_*`
fn scalar_market_correctly_resolves_common(base_asset: Asset<MarketId>, reported_value: u128) {
    let end = 100;
    simple_create_scalar_market(
        base_asset,
        MarketCreation::Permissionless,
        0..end,
        ScoringRule::Lmsr,
    );
    assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(CHARLIE), 0, 100 * BASE));
    assert_ok!(Tokens::transfer(
        RuntimeOrigin::signed(CHARLIE),
        EVE,
        Asset::ScalarOutcome(0, ScalarPosition::Short),
        100 * BASE
    ));
    // (Eve now has 100 SHORT, Charlie has 100 LONG)

    let market = MarketCommons::market(&0).unwrap();
    let grace_period = end + market.deadlines.grace_period;
    run_to_block(grace_period + 1);
    assert_ok!(PredictionMarkets::report(
        RuntimeOrigin::signed(BOB),
        0,
        OutcomeReport::Scalar(reported_value)
    ));
    let market_after_report = MarketCommons::market(&0).unwrap();
    assert!(market_after_report.report.is_some());
    let report = market_after_report.report.unwrap();
    assert_eq!(report.at, grace_period + 1);
    assert_eq!(report.by, BOB);
    assert_eq!(report.outcome, OutcomeReport::Scalar(reported_value));

    run_blocks(market.deadlines.dispute_duration);
    let market_after_resolve = MarketCommons::market(&0).unwrap();
    assert_eq!(market_after_resolve.status, MarketStatus::Resolved);

    // Check balances before redeeming (just to make sure that our tests are based on correct
    // assumptions)!
    assert_eq!(AssetManager::free_balance(base_asset, &CHARLIE), 900 * BASE);
    assert_eq!(AssetManager::free_balance(base_asset, &EVE), 1000 * BASE);

    assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(CHARLIE), 0));
    assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(EVE), 0));
    let assets = PredictionMarkets::outcome_assets(0, &MarketCommons::market(&0).unwrap());
    for asset in assets.iter() {
        assert_eq!(AssetManager::free_balance(*asset, &CHARLIE), 0);
        assert_eq!(AssetManager::free_balance(*asset, &EVE), 0);
    }
}
