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