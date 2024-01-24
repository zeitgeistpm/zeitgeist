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

use crate::{MarketIdsPerDisputeBlock, MarketIdsPerReportBlock};
use sp_runtime::{
    traits::{BlakeTwo256, Hash, Zero},
    Perquintill,
};
use zeitgeist_primitives::types::{Bond, OutcomeReport, Report};
use zrml_court::types::{CourtStatus, Draw, Vote, VoteItem};

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
    let test = |base_asset: AssetOf<Runtime>| {
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
        assert_eq!(
            charlie_reserved,
            DisputeBond::get() + <Runtime as zrml_simple_disputes::Config>::OutcomeBond::get()
        );

        let dave_reserved = Balances::reserved_balance(DAVE);
        assert_eq!(
            dave_reserved,
            <Runtime as zrml_simple_disputes::Config>::OutcomeBond::get()
                + <Runtime as zrml_simple_disputes::Config>::OutcomeFactor::get()
        );

        let eve_reserved = Balances::reserved_balance(EVE);
        assert_eq!(
            eve_reserved,
            <Runtime as zrml_simple_disputes::Config>::OutcomeBond::get()
                + 2 * <Runtime as zrml_simple_disputes::Config>::OutcomeFactor::get()
        );

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
        //     - Dave's reserve: <Runtime as Config>::OutcomeBond::get() + <Runtime as zrml_simple_disputes::Config>::OutcomeFactor::get()
        //     - Alice's oracle bond: OracleBond::get()
        // simple-disputes reward: <Runtime as Config>::OutcomeBond::get() + <Runtime as zrml_simple_disputes::Config>::OutcomeFactor::get()
        // Charlie gets OracleBond, because the dispute was justified.
        // A dispute is justified if the oracle's report is different to the final outcome.
        //
        // Charlie and Eve each receive half of the simple-disputes reward as bounty.
        let dave_reserved = <Runtime as zrml_simple_disputes::Config>::OutcomeBond::get()
            + <Runtime as zrml_simple_disputes::Config>::OutcomeFactor::get();
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
    let test = |base_asset: AssetOf<Runtime>| {
        let juror_0 = 1000;
        let juror_1 = 1001;
        let juror_2 = 1002;
        let juror_3 = 1003;
        let juror_4 = 1004;
        let juror_5 = 1005;

        for j in &[juror_0, juror_1, juror_2, juror_3, juror_4, juror_5] {
            let amount = <Runtime as zrml_court::Config>::MinJurorStake::get() + *j;
            assert_ok!(AssetManager::deposit(Asset::Ztg, j, amount + SENTINEL_AMOUNT));
            assert_ok!(Court::join_court(RuntimeOrigin::signed(*j), amount));
        }

        // just to have enough jurors for the dispute
        for j in 1006..(1006 + Court::necessary_draws_weight(0usize) as u32) {
            let juror = j as u128;
            let amount = <Runtime as zrml_court::Config>::MinJurorStake::get() + juror;
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
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
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
                slashable: <Runtime as zrml_court::Config>::MinJurorStake::get(),
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
            zrml_court::Error::<Runtime>::CommitmentHashMismatch
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
            zrml_court::Error::<Runtime>::VoteAlreadyDenounced
        );

        assert_noop!(
            Court::reveal_vote(
                RuntimeOrigin::signed(juror_4),
                market_id,
                vote_item_0.clone(),
                salt
            ),
            zrml_court::Error::<Runtime>::JurorDidNotVote
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

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_approved_advised_market_on_oracle_report()
 {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: AssetOf<Runtime>| {
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
    let test = |base_asset: AssetOf<Runtime>| {
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
    let test = |base_asset: AssetOf<Runtime>| {
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
    let test = |base_asset: AssetOf<Runtime>| {
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
    let test = |base_asset: AssetOf<Runtime>| {
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
    let test = |base_asset: AssetOf<Runtime>| {
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
    let test = |base_asset: AssetOf<Runtime>| {
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
        check_reserve(&outsider, <Runtime as Config>::OutsiderBond::get());

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
            outsider_balance_before + OracleBond::get() + <Runtime as Config>::OutsiderBond::get()
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
    let test = |base_asset: AssetOf<Runtime>| {
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
        check_reserve(&outsider, <Runtime as Config>::OutsiderBond::get());

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
            outsider_balance_before + OracleBond::get() + <Runtime as Config>::OutsiderBond::get()
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
                oracle_duration: <Runtime as Config>::MinOracleDuration::get(),
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
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_on_oracle_report()
 {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: AssetOf<Runtime>| {
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
    let test = |base_asset: AssetOf<Runtime>| {
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
        assert_eq!(
            market.bonds.outsider,
            Some(Bond::new(CHARLIE, <Runtime as Config>::OutsiderBond::get()))
        );
        check_reserve(&CHARLIE, <Runtime as Config>::OutsiderBond::get());
        assert_eq!(
            Balances::free_balance(CHARLIE),
            charlie_balance_before - <Runtime as Config>::OutsiderBond::get()
        );
        let charlie_balance_before = Balances::free_balance(CHARLIE);

        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // Check that validity bond didn't get slashed, but oracle bond did
        assert_eq!(Balances::free_balance(ALICE), alice_balance_before + ValidityBond::get());

        check_reserve(&CHARLIE, 0);
        // Check that the outsider gets the OracleBond together with the OutsiderBond
        assert_eq!(
            Balances::free_balance(CHARLIE),
            charlie_balance_before + OracleBond::get() + <Runtime as Config>::OutsiderBond::get()
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
