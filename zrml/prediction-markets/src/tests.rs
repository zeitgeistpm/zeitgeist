use crate::{Error, mock::*, market::*};
use frame_support::{
    assert_noop, assert_ok,
    dispatch::DispatchError,
    traits::{Currency, ReservableCurrency},
};
use sp_core::H256;
use zrml_traits::shares::{Shares as SharesTrait, ReservableShares};

#[test]
fn it_creates_binary_markets() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(
            PredictionMarkets::create(
                Origin::signed(ALICE),
                BOB,
                MarketType::Binary,
                3,
                100,
                H256::repeat_byte(2).to_fixed_bytes(),
                MarketCreation::Permissionless,
            )
        );

        // check the correct amount was reserved
        let reserved = Balances::reserved_balance(&ALICE);
        assert_eq!(reserved, 300);

        // Creates an advised market.
        assert_ok!(
            PredictionMarkets::create(
                Origin::signed(BOB),
                ALICE,
                MarketType::Binary,
                3,
                1000,
                H256::repeat_byte(3).to_fixed_bytes(),
                MarketCreation::Advised,
            )
        );

        let bob_reserved = Balances::reserved_balance(&BOB);
        assert_eq!(bob_reserved, 150);

        // Make sure that the market id has been incrementing
        let market_id = PredictionMarkets::market_count();
        assert_eq!(market_id, 2);
    });
}

#[test]
fn it_allows_advisory_origin_to_approve_markets() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        assert_ok!(
            PredictionMarkets::create(
                Origin::signed(BOB),
                ALICE,
                MarketType::Binary,
                3,
                1000,
                H256::repeat_byte(3).to_fixed_bytes(),
                MarketCreation::Advised,
            )
        );

        // make sure it's in status proposed
        let market = PredictionMarkets::markets(0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        // Make sure it fails from the random joe
        assert_noop!(PredictionMarkets::approve_market(Origin::signed(BOB), 0), DispatchError::BadOrigin);

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
        assert_ok!(
            PredictionMarkets::create(
                Origin::signed(BOB),
                ALICE,
                MarketType::Binary,
                3,
                1000,
                H256::repeat_byte(3).to_fixed_bytes(),
                MarketCreation::Advised,
            )
        );

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
        assert_ok!(
            PredictionMarkets::create(
                Origin::signed(ALICE),
                BOB,
                MarketType::Binary,
                3,
                100,
                H256::repeat_byte(2).to_fixed_bytes(),
                MarketCreation::Permissionless,
            )
        );

        // Allows someone to generate a complete set
        assert_ok!(
            PredictionMarkets::buy_complete_set(
                Origin::signed(BOB),
                0,
                100,
            )
        );

        // Check the outcome balances
        for i in 0..3 {
            let share_id = PredictionMarkets::market_outcome_share_id(0, i);
            let bal = Shares::free_balance(share_id, &BOB);
            assert_eq!(bal, 100);
        }

        // also check native balance
        let bal = Balances::free_balance(&BOB);
        assert_eq!(bal, 1_000 - 100);
    });
}

#[test]
fn it_allows_to_sell_a_complete_set() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(
            PredictionMarkets::create(
                Origin::signed(ALICE),
                BOB,
                MarketType::Binary,
                3,
                100,
                H256::repeat_byte(2).to_fixed_bytes(),
                MarketCreation::Permissionless,
            )
        );

        assert_ok!(
            PredictionMarkets::buy_complete_set(
                Origin::signed(BOB),
                0,
                100,
            )
        );

        assert_ok!(
            PredictionMarkets::sell_complete_set(
                Origin::signed(BOB),
                0,
                100,
            )
        );

        // Check the outcome balances
        for i in 0..3 {
            let share_id = PredictionMarkets::market_outcome_share_id(0, i);
            let bal = Shares::free_balance(share_id, &BOB);
            assert_eq!(bal, 0);
        }

        // also check native balance
        let bal = Balances::free_balance(&BOB);
        assert_eq!(bal, 1_000);
    });
}

#[test]
fn it_allows_to_report_the_outcome_of_a_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(
            PredictionMarkets::create(
                Origin::signed(ALICE),
                BOB,
                MarketType::Binary,
                3,
                100,
                H256::repeat_byte(2).to_fixed_bytes(),
                MarketCreation::Permissionless,
            )
        );

        // TODO
    });
}

#[test]
fn it_allows_to_dispute_the_outcome_of_a_market() {
    ExtBuilder::default().build().execute_with(|| {
        // TODO
    });
}

#[test]
fn it_allows_to_redeem_shares() {
    ExtBuilder::default().build().execute_with(|| {
        // TODO
    });
}
