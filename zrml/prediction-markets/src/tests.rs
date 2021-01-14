use crate::{market::*, mock::*, Error};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use sp_core::H256;
use zrml_traits::shares::Shares as SharesTrait;

#[test]
fn it_creates_binary_markets() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(ALICE),
            BOB,
            MarketType::Binary,
            100,
            H256::repeat_byte(2).to_fixed_bytes().to_vec(),
            MarketCreation::Permissionless,
        ));

        // check the correct amount was reserved
        let reserved = Balances::reserved_balance(&ALICE);
        assert_eq!(reserved, 300);

        // Creates an advised market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(BOB),
            ALICE,
            MarketType::Binary,
            1000,
            H256::repeat_byte(3).to_fixed_bytes().to_vec(),
            MarketCreation::Advised,
        ));

        let bob_reserved = Balances::reserved_balance(&BOB);
        assert_eq!(bob_reserved, 150);

        // Make sure that the market id has been incrementing
        let market_id = PredictionMarkets::market_count();
        assert_eq!(market_id, 2);
    });
}

#[test]
fn it_allows_sudo_to_destroy_markets() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(BOB),
            ALICE,
            MarketType::Binary,
            1000,
            H256::repeat_byte(3).to_fixed_bytes().to_vec(),
            MarketCreation::Advised,
        ));

        // destroy the market
        assert_ok!(PredictionMarkets::destroy_market(Origin::signed(SUDO), 0));

        assert_eq!(PredictionMarkets::markets(0).is_none(), true);
    });
}

#[test]
fn it_allows_advisory_origin_to_approve_markets() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(BOB),
            ALICE,
            MarketType::Binary,
            1000,
            H256::repeat_byte(3).to_fixed_bytes().to_vec(),
            MarketCreation::Advised,
        ));

        // make sure it's in status proposed
        let market = PredictionMarkets::markets(0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        // Make sure it fails from the random joe
        assert_noop!(
            PredictionMarkets::approve_market(Origin::signed(BOB), 0),
            DispatchError::BadOrigin
        );

        // Now it should work from SUDO
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));

        let after_market = PredictionMarkets::markets(0);
        assert_eq!(after_market.unwrap().status, MarketStatus::Active);
    });
}

#[test]
fn it_allows_the_advisory_origin_to_reject_markets() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(BOB),
            ALICE,
            MarketType::Binary,
            1000,
            H256::repeat_byte(3).to_fixed_bytes().to_vec(),
            MarketCreation::Advised,
        ));

        // make sure it's in status proposed
        let market = PredictionMarkets::markets(0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        // Now it should work from SUDO
        assert_ok!(PredictionMarkets::reject_market(Origin::signed(SUDO), 0));

        let after_market = PredictionMarkets::markets(0);
        assert_eq!(after_market.is_none(), true);
    });
}

#[test]
fn it_allows_to_buy_a_complete_set() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(ALICE),
            BOB,
            MarketType::Binary,
            100,
            H256::repeat_byte(2).to_fixed_bytes().to_vec(),
            MarketCreation::Permissionless,
        ));

        // Allows someone to generate a complete set
        assert_ok!(PredictionMarkets::buy_complete_set(
            Origin::signed(BOB),
            0,
            100,
        ));

        // Check the outcome balances
        for i in 0..=2 {
            let share_id = PredictionMarkets::market_outcome_share_id(0, i);
            let bal = Shares::free_balance(share_id, &BOB);
            assert_eq!(bal, 100);
        }

        // also check native balance
        let bal = Balances::free_balance(&BOB);
        assert_eq!(bal, 1_000 * BASE - 100);

        let market_account = PredictionMarkets::market_account(0);
        let market_bal = Balances::free_balance(market_account);
        assert_eq!(market_bal, 100);
    });
}

#[test]
fn it_allows_to_deploy_a_pool() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(ALICE),
            BOB,
            MarketType::Binary,
            100,
            H256::repeat_byte(2).to_fixed_bytes().to_vec(),
            MarketCreation::Permissionless,
        ));

        assert_ok!(PredictionMarkets::buy_complete_set(
            Origin::signed(BOB),
            0,
            100 * BASE,
        ));

        assert_ok!(Shares::wrap_native_currency(
            Origin::signed(BOB),
            100 * BASE
        ));

        assert_ok!(PredictionMarkets::deploy_swap_pool_for_market(
            Origin::signed(BOB),
            0,
            vec![
                10_000_000_000,
                10_000_000_000,
                10_000_000_000,
                10_000_000_000
            ]
        ));
    });
}

#[test]
fn it_allows_to_sell_a_complete_set() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(ALICE),
            BOB,
            MarketType::Binary,
            100,
            H256::repeat_byte(2).to_fixed_bytes().to_vec(),
            MarketCreation::Permissionless,
        ));

        assert_ok!(PredictionMarkets::buy_complete_set(
            Origin::signed(BOB),
            0,
            100,
        ));

        assert_ok!(PredictionMarkets::sell_complete_set(
            Origin::signed(BOB),
            0,
            100,
        ));

        // Check the outcome balances
        for i in 0..=2 {
            let share_id = PredictionMarkets::market_outcome_share_id(0, i);
            let bal = Shares::free_balance(share_id, &BOB);
            assert_eq!(bal, 0);
        }

        // also check native balance
        let bal = Balances::free_balance(&BOB);
        assert_eq!(bal, 1_000 * BASE);
    });
}

#[test]
fn it_allows_to_report_the_outcome_of_a_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(ALICE),
            BOB,
            MarketType::Binary,
            100,
            H256::repeat_byte(2).to_fixed_bytes().to_vec(),
            MarketCreation::Permissionless,
        ));

        run_to_block(100);

        let market = PredictionMarkets::markets(0).unwrap();
        assert_eq!(market.status, MarketStatus::Active);
        assert_eq!(market.reporter, None);

        assert_ok!(PredictionMarkets::report(Origin::signed(BOB), 0, 1,));

        let market_after = PredictionMarkets::markets(0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Reported);
        assert_eq!(market_after.reported_outcome.unwrap(), 1);
        assert_eq!(market_after.reporter, Some(market_after.oracle));
    });
}

#[test]
fn it_allows_to_dispute_the_outcome_of_a_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(ALICE),
            BOB,
            MarketType::Binary,
            100,
            H256::repeat_byte(2).to_fixed_bytes().to_vec(),
            MarketCreation::Permissionless,
        ));

        // Run to the end of the trading phase.
        run_to_block(100);

        assert_ok!(PredictionMarkets::report(Origin::signed(BOB), 0, 1,));

        // Dispute phase is 10 blocks... so only run 5 of them.
        run_to_block(105);

        assert_ok!(PredictionMarkets::dispute(Origin::signed(CHARLIE), 0, 0,));

        let market = PredictionMarkets::markets(0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        let disputes = PredictionMarkets::disputes(0);
        assert_eq!(disputes.len(), 1);
        let dispute = &disputes[0];
        assert_eq!(dispute.at, 105);
        assert_eq!(dispute.by, CHARLIE);
        assert_eq!(dispute.outcome, 0);

        let market_ids = PredictionMarkets::market_ids_per_dispute_block(105);
        assert_eq!(market_ids.len(), 1);
        assert_eq!(market_ids[0], 0);
    });
}

#[test]
fn it_allows_anyone_to_report_an_unreported_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(ALICE),
            BOB,
            MarketType::Binary,
            100,
            H256::repeat_byte(2).to_fixed_bytes().to_vec(),
            MarketCreation::Permissionless,
        ));

        // Just skip to waaaay overdue.
        run_to_block(3000);

        assert_ok!(PredictionMarkets::report(
            Origin::signed(ALICE), // alice reports her own market now
            0,
            1,
        ));

        let market = PredictionMarkets::markets(0).unwrap();
        assert_eq!(market.status, MarketStatus::Reported);
        assert_eq!(market.reporter, Some(ALICE));
        // but lol oracle was bob
        assert_eq!(market.oracle, BOB);

        // make sure it still resolves
        run_to_block(3011);

        let market_after = PredictionMarkets::markets(0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);
    });
}

#[test]
fn it_correctly_resolves_a_market_that_was_reported_on() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(ALICE),
            BOB,
            MarketType::Binary,
            100,
            H256::repeat_byte(2).to_fixed_bytes().to_vec(),
            MarketCreation::Permissionless,
        ));

        assert_ok!(PredictionMarkets::buy_complete_set(
            Origin::signed(CHARLIE),
            0,
            100,
        ));

        run_to_block(100);

        assert_ok!(PredictionMarkets::report(Origin::signed(BOB), 0, 1,));

        let reported_ids = PredictionMarkets::market_ids_per_report_block(100);
        assert_eq!(reported_ids.len(), 1);
        let id = reported_ids[0];
        assert_eq!(id, 0);

        run_to_block(111);

        let market = PredictionMarkets::markets(0);
        assert_eq!(market.unwrap().status, MarketStatus::Resolved);

        // check to make sure all but the winning share was deleted
        let share_a = PredictionMarkets::market_outcome_share_id(0, 0);
        let share_a_total = Shares::total_supply(share_a);
        assert_eq!(share_a_total, 0);
        let share_a_bal = Shares::free_balance(share_a, &CHARLIE);
        assert_eq!(share_a_bal, 0);

        let share_b = PredictionMarkets::market_outcome_share_id(0, 1);
        let share_b_total = Shares::total_supply(share_b);
        assert_eq!(share_b_total, 100);
        let share_b_bal = Shares::free_balance(share_b, &CHARLIE);
        assert_eq!(share_b_bal, 100);

        let share_c = PredictionMarkets::market_outcome_share_id(0, 2);
        let share_c_total = Shares::total_supply(share_c);
        assert_eq!(share_c_total, 0);
        let share_c_bal = Shares::free_balance(share_c, &CHARLIE);
        assert_eq!(share_c_bal, 0);
    });
}

#[test]
fn it_resolves_a_disputed_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(ALICE),
            BOB,
            MarketType::Binary,
            100,
            H256::repeat_byte(2).to_fixed_bytes().to_vec(),
            MarketCreation::Permissionless,
        ));

        assert_ok!(PredictionMarkets::buy_complete_set(
            Origin::signed(CHARLIE),
            0,
            100,
        ));

        run_to_block(100);

        assert_ok!(PredictionMarkets::report(Origin::signed(BOB), 0, 1,));

        run_to_block(102);

        assert_ok!(PredictionMarkets::dispute(Origin::signed(CHARLIE), 0, 2,));

        run_to_block(103);

        assert_ok!(PredictionMarkets::dispute(Origin::signed(DAVE), 0, 1,));

        run_to_block(104);

        assert_ok!(PredictionMarkets::dispute(Origin::signed(EVE), 0, 2,));

        let market = PredictionMarkets::markets(0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        // check everyone's deposits
        let charlie_reserved = Balances::reserved_balance(&CHARLIE);
        assert_eq!(charlie_reserved, 100);

        let dave_reserved = Balances::reserved_balance(&DAVE);
        assert_eq!(dave_reserved, 125);

        let eve_reserved = Balances::reserved_balance(&EVE);
        assert_eq!(eve_reserved, 150);

        // check disputes length
        let disputes = PredictionMarkets::disputes(0);
        assert_eq!(disputes.len(), 3);

        // make sure the old mappings of market id per dispute block are erased
        let market_ids_1 = PredictionMarkets::market_ids_per_dispute_block(102);
        assert_eq!(market_ids_1.len(), 0);

        let market_ids_2 = PredictionMarkets::market_ids_per_dispute_block(103);
        assert_eq!(market_ids_2.len(), 0);

        let market_ids_3 = PredictionMarkets::market_ids_per_dispute_block(104);
        assert_eq!(market_ids_3.len(), 1);

        run_to_block(115);

        let market_after = PredictionMarkets::markets(0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);

        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(CHARLIE), 0,));

        // make sure rewards are right
        //
        // slashed amounts
        // ---------------------------
        // - OracleBond: 100
        // - Dave's reserve: 125
        // Total: 225
        // Per each: 112

        let charlie_balance = Balances::free_balance(&CHARLIE);
        assert_eq!(charlie_balance, 1_000 * BASE + 112);
        let eve_balance = Balances::free_balance(&EVE);
        assert_eq!(eve_balance, 1_000 * BASE + 112);

        let dave_balance = Balances::free_balance(&DAVE);
        assert_eq!(dave_balance, 1_000 * BASE - 125);

        let alice_balance = Balances::free_balance(&ALICE);
        assert_eq!(alice_balance, 1_000 * BASE - 100);

        // bob kinda gets away scot-free since Alice is held responsible
        // for her designated reporter
        let bob_balance = Balances::free_balance(&BOB);
        assert_eq!(bob_balance, 1_000 * BASE);
    });
}

#[test]
fn it_allows_to_redeem_shares() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(ALICE),
            BOB,
            MarketType::Binary,
            100,
            H256::repeat_byte(2).to_fixed_bytes().to_vec(),
            MarketCreation::Permissionless,
        ));

        assert_ok!(PredictionMarkets::buy_complete_set(
            Origin::signed(CHARLIE),
            0,
            100,
        ));

        run_to_block(100);

        assert_ok!(PredictionMarkets::report(Origin::signed(BOB), 0, 1,));

        run_to_block(111);

        let market = PredictionMarkets::markets(0).unwrap();
        assert_eq!(market.status, MarketStatus::Resolved);

        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(CHARLIE), 0));
        let bal = Balances::free_balance(&CHARLIE);
        assert_eq!(bal, 1_000 * BASE);
    });
}

#[test]
fn the_entire_market_lifecycle_works_with_timestamps() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create(
            Origin::signed(ALICE),
            BOB,
            MarketType::Binary,
            1_234_567_890_123,
            H256::repeat_byte(2).to_fixed_bytes().to_vec(),
            MarketCreation::Permissionless,
        ));

        // is ok
        assert_ok!(PredictionMarkets::buy_complete_set(
            Origin::signed(BOB),
            0,
            100,
        ));

        // set the timestamp
        Timestamp::set_timestamp(1_234_567_890_124);

        assert_noop!(
            PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, 100,),
            Error::<Test>::MarketNotActive,
        );

        assert_ok!(PredictionMarkets::report(Origin::signed(BOB), 0, 1,));
    });
}
