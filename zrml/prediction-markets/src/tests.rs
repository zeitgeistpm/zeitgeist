#![cfg(all(feature = "mock", test))]

use crate::{
    mock::*, Config, Error, Event, MarketIdsPerDisputeBlock, MarketIdsPerReportBlock, RESERVE_ID,
};
use core::ops::{Range, RangeInclusive};
use frame_support::{
    assert_err, assert_noop, assert_ok,
    dispatch::{DispatchError, DispatchResult},
    traits::{Get, NamedReservableCurrency},
};
use more_asserts::assert_le;
use test_case::test_case;

use orml_traits::MultiCurrency;
use sp_runtime::traits::AccountIdConversion;
use zeitgeist_primitives::{
    constants::{DisputeFactor, BASE, CENT, MILLISECS_PER_BLOCK},
    types::{
        Asset, BlockNumber, Market, MarketCreation, MarketDisputeMechanism, MarketPeriod,
        MarketStatus, MarketType, Moment, MultiHash, OutcomeReport, ScalarPosition, ScoringRule,
    },
};
use zrml_market_commons::MarketCommonsPalletApi;

const SENTINEL_AMOUNT: u128 = BASE;

fn gen_metadata(byte: u8) -> MultiHash {
    let mut metadata = [byte; 50];
    metadata[0] = 0x15;
    metadata[1] = 0x30;
    MultiHash::Sha3_384(metadata)
}

fn simple_create_categorical_market<T: crate::Config>(
    creation: MarketCreation,
    period: Range<u64>,
    scoring_rule: ScoringRule,
) {
    assert_ok!(PredictionMarkets::create_market(
        Origin::signed(ALICE),
        BOB,
        MarketPeriod::Block(period),
        gen_metadata(2),
        creation,
        MarketType::Categorical(T::MinCategories::get()),
        MarketDisputeMechanism::SimpleDisputes,
        scoring_rule
    ));
}

fn simple_create_scalar_market<T: crate::Config>(
    creation: MarketCreation,
    period: Range<u64>,
    scoring_rule: ScoringRule,
) {
    assert_ok!(PredictionMarkets::create_market(
        Origin::signed(ALICE),
        BOB,
        MarketPeriod::Block(period),
        gen_metadata(2),
        creation,
        MarketType::Scalar(100..=200),
        MarketDisputeMechanism::SimpleDisputes,
        scoring_rule
    ));
}

#[test_case(654..=321; "empty range")]
#[test_case(555..=555; "one element as range")]
fn create_scalar_market_fails_on_invalid_range(range: RangeInclusive<u128>) {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                BOB,
                MarketPeriod::Block(123..456),
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Scalar(range),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM,
            ),
            crate::Error::<Runtime>::InvalidOutcomeRange
        );
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_permissionless_market_active() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        assert_ok!(Currencies::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), SENTINEL_AMOUNT);
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_permissionless_market_reported() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        run_to_block(2);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(Currencies::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), SENTINEL_AMOUNT);
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_permissionless_market_disputed() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        run_to_block(2);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(Currencies::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), SENTINEL_AMOUNT);
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_permissionless_market_resolved() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        run_to_block(2);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_to_block(9000); // Wait until market resolves
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), 0);
        assert_ok!(Currencies::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), SENTINEL_AMOUNT);
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_advised_market_proposed() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );
        assert_ok!(Currencies::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), SENTINEL_AMOUNT);
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_advised_market_active() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        assert_ok!(Currencies::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), SENTINEL_AMOUNT);
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_advised_market_reported() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        run_to_block(2);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(Currencies::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), SENTINEL_AMOUNT);
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_advised_market_disputed() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        run_to_block(2);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(Currencies::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), SENTINEL_AMOUNT);
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_advised_market_resolved() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        run_to_block(2);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_to_block(9000); // Wait until market resolves
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), 0);
        assert_ok!(Currencies::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), SENTINEL_AMOUNT);
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    });
}

#[test]
fn admin_destroy_market_correctly_cleans_up_accounts() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = <Runtime as zrml_swaps::Config>::MinLiquidity::get();
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            ALICE,
            MarketPeriod::Block(0..42),
            gen_metadata(50),
            MarketType::Categorical(3),
            MarketDisputeMechanism::SimpleDisputes,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![amount, amount, amount],
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); 4],
        ));
        // Buy some outcome tokens for Alice so that we can check that they get destroyed.
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(ALICE), 0, BASE));
        let market_id = 0;
        let pool_id = 0;
        let pool_account = Swaps::pool_account_id(pool_id);
        let market_account = PredictionMarkets::market_account(market_id);
        let alice_ztg_before = Currencies::free_balance(Asset::Ztg, &ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(Currencies::free_balance(Asset::CategoricalOutcome(0, 0), &pool_account), 0);
        assert_eq!(Currencies::free_balance(Asset::CategoricalOutcome(0, 1), &pool_account), 0);
        assert_eq!(Currencies::free_balance(Asset::CategoricalOutcome(0, 2), &pool_account), 0);
        assert_eq!(Currencies::free_balance(Asset::Ztg, &pool_account), 0);
        assert_eq!(Currencies::free_balance(Asset::CategoricalOutcome(0, 0), &market_account), 0);
        assert_eq!(Currencies::free_balance(Asset::CategoricalOutcome(0, 1), &market_account), 0);
        assert_eq!(Currencies::free_balance(Asset::CategoricalOutcome(0, 2), &market_account), 0);
        assert_eq!(Currencies::free_balance(Asset::Ztg, &market_account), 0);
        assert_eq!(Currencies::free_balance(Asset::CategoricalOutcome(0, 0), &ALICE), 0);
        assert_eq!(Currencies::free_balance(Asset::CategoricalOutcome(0, 1), &ALICE), 0);
        assert_eq!(Currencies::free_balance(Asset::CategoricalOutcome(0, 2), &ALICE), 0);
        assert_eq!(Currencies::free_balance(Asset::Ztg, &ALICE), alice_ztg_before);
    });
}

#[test_case(MarketPeriod::Block(0..100); "market period block")]
#[test_case(MarketPeriod::Timestamp(0..100); "market period timestamp")]
fn admin_move_market_moves_active_market_to_closed(
    market_period: MarketPeriod<BlockNumber, Moment>,
) {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            market_period,
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        let market_id = 0;
        let block = 50;
        let timestamp = 50;
        run_to_block(block);
        Timestamp::set_timestamp(timestamp);
        assert_ok!(PredictionMarkets::admin_move_market_to_closed(Origin::signed(SUDO), market_id));
        let market = MarketCommons::market(&market_id).unwrap();
        // Verify that the market is _closed_ in the sense that the status is `Active` and the
        // market period has ended.
        assert_eq!(market.status, MarketStatus::Active);
        match market.period {
            MarketPeriod::Block(range) => assert_le!(range.end, block),
            MarketPeriod::Timestamp(range) => assert_le!(range.end, timestamp),
        };
    });
}

#[test]
fn admin_move_market_fails_if_market_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::admin_move_market_to_closed(Origin::signed(SUDO), 0),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test_case(MarketStatus::Active, MarketPeriod::Block(0..1); "closed with block")]
#[test_case(MarketStatus::Active, MarketPeriod::Timestamp(0..1); "closed with timestamp")]
#[test_case(MarketStatus::Reported, MarketPeriod::Block(0..1); "reported")]
#[test_case(MarketStatus::Disputed, MarketPeriod::Block(0..1); "disputed")]
#[test_case(MarketStatus::Resolved, MarketPeriod::Block(0..1); "resolved")]
#[test_case(MarketStatus::Proposed, MarketPeriod::Block(0..1); "proposed")]
#[test_case(MarketStatus::CollectingSubsidy, MarketPeriod::Block(0..1); "collecting subsidy")]
#[test_case(MarketStatus::InsufficientSubsidy, MarketPeriod::Block(0..1); "insufficient subsidy")]
fn admin_move_market_to_closed_fails_if_market_is_not_active(
    market_status: MarketStatus,
    market_period: MarketPeriod<BlockNumber, Moment>,
) {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            market_period,
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        let _ = MarketCommons::mutate_market(&0, |market| {
            market.status = market_status;
            Ok(())
        });
        run_to_block(2);
        Timestamp::set_timestamp(2);
        assert_noop!(
            PredictionMarkets::admin_move_market_to_closed(Origin::signed(SUDO), 0),
            Error::<Runtime>::MarketIsNotActive,
        );
    });
}

#[test]
fn it_creates_binary_markets() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );

        // check the correct amount was reserved
        let alice_reserved = Balances::reserved_balance(&ALICE);
        assert_eq!(alice_reserved, ValidityBond::get() + OracleBond::get());

        // Creates an advised market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        let new_alice_reserved = Balances::reserved_balance(&ALICE);
        assert_eq!(new_alice_reserved, AdvisoryBond::get() + OracleBond::get() + alice_reserved);

        // Make sure that the market id has been incrementing
        let market_id = MarketCommons::latest_market_id().unwrap();
        assert_eq!(market_id, 1);
    });
}

#[test]
fn create_categorical_market_deposits_the_correct_event() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            1..2,
            ScoringRule::CPMM,
        );
        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        let market_account = PredictionMarkets::market_account(market_id);
        System::assert_last_event(Event::MarketCreated(0, market_account, market).into());
    });
}

#[test]
fn create_scalar_market_deposits_the_correct_event() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        simple_create_scalar_market::<Runtime>(
            MarketCreation::Permissionless,
            1..2,
            ScoringRule::CPMM,
        );
        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        let market_account = PredictionMarkets::market_account(market_id);
        System::assert_last_event(Event::MarketCreated(0, market_account, market).into());
    });
}

#[test]
fn it_does_not_create_market_with_too_few_categories() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                BOB,
                MarketPeriod::Block(0..100),
                gen_metadata(2),
                MarketCreation::Advised,
                MarketType::Categorical(<Runtime as Config>::MinCategories::get() - 1),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM
            ),
            Error::<Runtime>::NotEnoughCategories
        );
    });
}

#[test]
fn it_does_not_create_market_with_too_many_categories() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                BOB,
                MarketPeriod::Block(0..100),
                gen_metadata(2),
                MarketCreation::Advised,
                MarketType::Categorical(<Runtime as Config>::MaxCategories::get() + 1),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM
            ),
            Error::<Runtime>::TooManyCategories
        );
    });
}

#[test]
fn it_allows_sudo_to_destroy_markets() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        // destroy the market
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));

        assert_noop!(
            MarketCommons::market(&0),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn it_allows_advisory_origin_to_approve_markets() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        // Make sure it fails from the random joe
        assert_noop!(
            PredictionMarkets::approve_market(Origin::signed(BOB), 0),
            DispatchError::BadOrigin
        );

        // Now it should work from SUDO
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));

        let after_market = MarketCommons::market(&0);
        assert_eq!(after_market.unwrap().status, MarketStatus::Active);
    });
}

#[test]
fn it_allows_the_advisory_origin_to_reject_markets() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        // Now it should work from SUDO
        assert_ok!(PredictionMarkets::reject_market(Origin::signed(SUDO), 0));

        assert_noop!(
            MarketCommons::market(&0),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn reject_market_unreserves_oracle_bond_and_slashes_advisory_bond() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        // Give ALICE `SENTINEL_AMOUNT` free and reserved ZTG; we record the free balance to check
        // that the AdvisoryBond gets slashed but the OracleBond gets unreserved.
        assert_ok!(Currencies::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        let balance_free_before_alice = Balances::free_balance(&ALICE);

        let balance_reserved_before_alice = Balances::reserved_balance_named(&RESERVE_ID, &ALICE);

        assert_ok!(PredictionMarkets::reject_market(Origin::signed(SUDO), 0));

        // AdvisoryBond gets slashed after reject_market
        // OracleBond gets unreserved after reject_market
        let balance_reserved_after_alice = Balances::reserved_balance_named(&RESERVE_ID, &ALICE);
        assert_eq!(
            balance_reserved_after_alice,
            balance_reserved_before_alice
                - <Runtime as Config>::OracleBond::get()
                - <Runtime as Config>::AdvisoryBond::get()
        );
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(
            balance_free_after_alice,
            balance_free_before_alice + <Runtime as Config>::OracleBond::get()
        );
    });
}

#[test]
fn it_allows_to_buy_a_complete_set() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::CPMM,
        );

        // Allows someone to generate a complete set
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, CENT));

        let market = MarketCommons::market(&0).unwrap();

        // Check the outcome balances
        let assets = PredictionMarkets::outcome_assets(0, &market);
        for asset in assets.iter() {
            let bal = Tokens::free_balance(*asset, &BOB);
            assert_eq!(bal, CENT);
        }

        // also check native balance
        let bal = Balances::free_balance(&BOB);
        assert_eq!(bal, 1_000 * BASE - CENT);

        let market_account = PredictionMarkets::market_account(0);
        let market_bal = Balances::free_balance(market_account);
        assert_eq!(market_bal, CENT);
        System::assert_last_event(Event::BoughtCompleteSet(0, CENT, BOB).into());
    });
}

#[test]
fn it_does_not_allow_to_buy_a_complete_set_on_pending_advised_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );
        assert_noop!(
            PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, CENT),
            Error::<Runtime>::MarketIsNotActive,
        );
    });
}

#[test]
fn create_categorical_market_fails_if_market_begin_is_equal_to_end() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                BOB,
                MarketPeriod::Block(3..3),
                gen_metadata(0),
                MarketCreation::Permissionless,
                MarketType::Categorical(3),
                MarketDisputeMechanism::Authorized(CHARLIE),
                ScoringRule::CPMM,
            ),
            crate::Error::<Runtime>::InvalidMarketPeriod,
        );
    });
}

#[test_case(MarketPeriod::Block(2..1); "block start greater than end")]
#[test_case(MarketPeriod::Block(3..3); "block start equal to end")]
#[test_case(
    MarketPeriod::Block(0..<Runtime as Config>::MaxMarketPeriod::get() + 1);
    "block end greater than max market period"
)]
#[test_case(MarketPeriod::Timestamp(2..1); "timestamp start greater than end")]
#[test_case(MarketPeriod::Timestamp(3..3); "timestamp start equal to end")]
#[test_case(
    MarketPeriod::Timestamp(0..<Runtime as Config>::MaxMarketPeriod::get() + 1);
    "timestamp end greater than max market period"
)]
fn create_categorical_market_fails_if_market_period_is_invalid(
    period: MarketPeriod<BlockNumber, Moment>,
) {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                BOB,
                period,
                gen_metadata(0),
                MarketCreation::Permissionless,
                MarketType::Categorical(3),
                MarketDisputeMechanism::Authorized(CHARLIE),
                ScoringRule::CPMM,
            ),
            crate::Error::<Runtime>::InvalidMarketPeriod,
        );
    });
}

#[test_case(MarketPeriod::Block(2..1); "block start greater than end")]
#[test_case(MarketPeriod::Block(3..3); "block start equal to end")]
#[test_case(
    MarketPeriod::Block(0..<Runtime as Config>::MaxMarketPeriod::get() + 1);
    "block end greater than max market period"
)]
#[test_case(MarketPeriod::Timestamp(2..1); "timestamp start greater than end")]
#[test_case(MarketPeriod::Timestamp(3..3); "timestamp start equal to end")]
#[test_case(
    MarketPeriod::Timestamp(0..<Runtime as Config>::MaxMarketPeriod::get() + 1);
    "timestamp end greater than max market period"
)]
fn create_scalar_market_fails_if_market_period_is_invalid(
    period: MarketPeriod<BlockNumber, Moment>,
) {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                BOB,
                period,
                gen_metadata(0),
                MarketCreation::Permissionless,
                MarketType::Scalar(123..=456),
                MarketDisputeMechanism::Authorized(CHARLIE),
                ScoringRule::CPMM,
            ),
            crate::Error::<Runtime>::InvalidMarketPeriod,
        );
    });
}

#[test]
fn it_does_not_allow_zero_amounts_in_buy_complete_set() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        assert_noop!(
            PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, 0),
            Error::<Runtime>::ZeroAmount
        );
    });
}

#[test]
fn it_does_not_allow_buying_complete_sets_with_insufficient_balance() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        assert_noop!(
            PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, 10000 * BASE),
            Error::<Runtime>::NotEnoughBalance
        );
    });
}

#[test]
fn it_allows_to_deploy_a_pool() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, 100 * BASE));

        assert_ok!(Balances::transfer(
            Origin::signed(BOB),
            <Runtime as crate::Config>::PalletId::get().into_account(),
            100 * BASE
        ));

        assert_ok!(PredictionMarkets::deploy_swap_pool_for_market(
            Origin::signed(BOB),
            0,
            vec![BASE, BASE, BASE]
        ));
    });
}

#[test]
fn it_does_not_allow_to_deploy_a_pool_on_pending_advised_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        assert_noop!(
            PredictionMarkets::deploy_swap_pool_for_market(
                Origin::signed(BOB),
                0,
                vec![BASE, BASE, BASE]
            ),
            Error::<Runtime>::MarketIsNotActive,
        );
    });
}

#[test]
fn it_allows_to_sell_a_complete_set() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::CPMM,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, CENT));

        assert_ok!(PredictionMarkets::sell_complete_set(Origin::signed(BOB), 0, CENT));

        let market = MarketCommons::market(&0).unwrap();

        // Check the outcome balances
        let assets = PredictionMarkets::outcome_assets(0, &market);
        for asset in assets.iter() {
            let bal = Tokens::free_balance(*asset, &BOB);
            assert_eq!(bal, 0);
        }

        // also check native balance
        let bal = Balances::free_balance(&BOB);
        assert_eq!(bal, 1_000 * BASE);

        System::assert_last_event(Event::SoldCompleteSet(0, CENT, BOB).into());
    });
}

#[test]
fn it_does_not_allow_zero_amounts_in_sell_complete_set() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        assert_noop!(
            PredictionMarkets::sell_complete_set(Origin::signed(BOB), 0, 0),
            Error::<Runtime>::ZeroAmount
        );
    });
}

#[test]
fn it_does_not_allow_to_sell_complete_sets_with_insufficient_balance() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, 2 * CENT));
        assert_eq!(Currencies::slash(Asset::CategoricalOutcome(0, 1), &BOB, CENT), 0);
        assert_noop!(
            PredictionMarkets::sell_complete_set(Origin::signed(BOB), 0, 2 * CENT),
            Error::<Runtime>::InsufficientShareBalance
        );
    });
}

#[test]
fn it_allows_to_report_the_outcome_of_a_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..100,
            ScoringRule::CPMM,
        );

        run_to_block(100);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Active);
        assert_eq!(market.report.is_none(), true);

        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market_after = MarketCommons::market(&0).unwrap();
        let report = market_after.report.unwrap();
        assert_eq!(market_after.status, MarketStatus::Reported);
        assert_eq!(report.outcome, OutcomeReport::Categorical(1));
        assert_eq!(report.by, market_after.oracle);

        // Reset and report again as approval origin
        let _ = MarketCommons::mutate_market(&0, |market| {
            market.status = MarketStatus::Active;
            market.report = None;
            Ok(())
        });

        assert_ok!(PredictionMarkets::report(
            Origin::signed(SUDO),
            0,
            OutcomeReport::Categorical(1)
        ));
    });
}

#[test]
fn report_fails_on_mismatched_outcome_for_categorical_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..100,
            ScoringRule::CPMM,
        );
        run_to_block(100);
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Scalar(123)),
            Error::<Runtime>::OutcomeMismatch,
        );
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Active);
        assert_eq!(market.report.is_none(), true);
    });
}

#[test]
fn report_fails_on_out_of_range_outcome_for_categorical_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..100,
            ScoringRule::CPMM,
        );
        run_to_block(100);
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(2)),
            Error::<Runtime>::OutcomeMismatch,
        );
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Active);
        assert_eq!(market.report.is_none(), true);
    });
}

#[test]
fn report_fails_on_mismatched_outcome_for_scalar_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_scalar_market::<Runtime>(
            MarketCreation::Permissionless,
            0..100,
            ScoringRule::CPMM,
        );
        run_to_block(100);
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(0)),
            Error::<Runtime>::OutcomeMismatch,
        );
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Active);
        assert_eq!(market.report.is_none(), true);
    });
}

#[test]
fn it_allows_to_dispute_the_outcome_of_a_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );

        // Run to the end of the trading phase.
        run_to_block(100);

        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        // Dispute phase is 10 blocks... so only run 5 of them.
        run_to_block(105);

        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        let disputes = crate::Disputes::<Runtime>::get(&0);
        assert_eq!(disputes.len(), 1);
        let dispute = &disputes[0];
        assert_eq!(dispute.at, 105);
        assert_eq!(dispute.by, CHARLIE);
        assert_eq!(dispute.outcome, OutcomeReport::Categorical(0));

        let market_ids = MarketIdsPerDisputeBlock::<Runtime>::get(&105);
        assert_eq!(market_ids.len(), 1);
        assert_eq!(market_ids[0], 0);
    });
}

#[test]
fn it_allows_anyone_to_report_an_unreported_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );

        // Just skip to waaaay overdue.
        run_to_block(9000);

        assert_ok!(PredictionMarkets::report(
            Origin::signed(ALICE), // alice reports her own market now
            0,
            OutcomeReport::Categorical(1),
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Reported);
        assert_eq!(market.report.unwrap().by, ALICE);
        // but oracle was bob
        assert_eq!(market.oracle, BOB);

        // make sure it still resolves
        run_to_block(9011);

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);
    });
}

#[test]
fn it_correctly_resolves_a_market_that_was_reported_on() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, CENT));

        run_to_block(100);

        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let reported_ids = MarketIdsPerReportBlock::<Runtime>::get(&100);
        assert_eq!(reported_ids.len(), 1);
        let id = reported_ids[0];
        assert_eq!(id, 0);

        run_to_block(111);

        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Resolved);

        // check to make sure all but the winning share was deleted
        let share_a = Asset::CategoricalOutcome(0, 0);
        let share_a_total = Tokens::total_issuance(share_a);
        assert_eq!(share_a_total, 0);
        let share_a_bal = Tokens::free_balance(share_a, &CHARLIE);
        assert_eq!(share_a_bal, 0);

        let share_b = Asset::CategoricalOutcome(0, 1);
        let share_b_total = Tokens::total_issuance(share_b);
        assert_eq!(share_b_total, CENT);
        let share_b_bal = Tokens::free_balance(share_b, &CHARLIE);
        assert_eq!(share_b_bal, CENT);

        let share_c = Asset::CategoricalOutcome(0, 2);
        let share_c_total = Tokens::total_issuance(share_c);
        assert_eq!(share_c_total, 0);
        let share_c_bal = Tokens::free_balance(share_c, &CHARLIE);
        assert_eq!(share_c_bal, 0);
    });
}

#[test]
fn it_resolves_a_disputed_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, CENT));

        run_to_block(100);

        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));

        run_to_block(102);

        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));

        run_to_block(103);

        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(DAVE),
            0,
            OutcomeReport::Categorical(0)
        ));

        run_to_block(104);

        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        // check everyone's deposits
        let charlie_reserved = Balances::reserved_balance(&CHARLIE);
        assert_eq!(charlie_reserved, DisputeBond::get());

        let dave_reserved = Balances::reserved_balance(&DAVE);
        assert_eq!(dave_reserved, DisputeBond::get() + DisputeFactor::get());

        let eve_reserved = Balances::reserved_balance(&EVE);
        assert_eq!(eve_reserved, DisputeBond::get() + 2 * DisputeFactor::get());

        // check disputes length
        let disputes = crate::Disputes::<Runtime>::get(&0);
        assert_eq!(disputes.len(), 3);

        // make sure the old mappings of market id per dispute block are erased
        let market_ids_1 = MarketIdsPerDisputeBlock::<Runtime>::get(&102);
        assert_eq!(market_ids_1.len(), 0);

        let market_ids_2 = MarketIdsPerDisputeBlock::<Runtime>::get(&103);
        assert_eq!(market_ids_2.len(), 0);

        let market_ids_3 = MarketIdsPerDisputeBlock::<Runtime>::get(&104);
        assert_eq!(market_ids_3.len(), 1);

        run_to_block(115);

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);

        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(CHARLIE), 0));

        // Make sure rewards are right:
        //
        // Slashed amounts:
        //     - Dave's reserve: DisputeBond::get() + DisputeFactor::get()
        //     - Alice's oracle bond: OracleBond::get()
        // Total: OracleBond::get() + DisputeBond::get() + DisputeFactor::get()
        //
        // Charlie and Eve each receive half of the total slashed amount as bounty.
        let dave_reserved = DisputeBond::get() + DisputeFactor::get();
        let total_slashed = OracleBond::get() + dave_reserved;

        let charlie_balance = Balances::free_balance(&CHARLIE);
        assert_eq!(charlie_balance, 1_000 * BASE + total_slashed / 2);
        let charlie_reserved_2 = Balances::reserved_balance(&CHARLIE);
        assert_eq!(charlie_reserved_2, 0);
        let eve_balance = Balances::free_balance(&EVE);
        assert_eq!(eve_balance, 1_000 * BASE + total_slashed / 2);

        let dave_balance = Balances::free_balance(&DAVE);
        assert_eq!(dave_balance, 1_000 * BASE - dave_reserved);

        let alice_balance = Balances::free_balance(&ALICE);
        assert_eq!(alice_balance, 1_000 * BASE - OracleBond::get());

        // bob kinda gets away scot-free since Alice is held responsible
        // for her designated reporter
        let bob_balance = Balances::free_balance(&BOB);
        assert_eq!(bob_balance, 1_000 * BASE);
    });
}

#[test]
fn it_resolves_a_disputed_market_to_default_if_mdm_failed() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Block(0..1),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            MarketDisputeMechanism::Authorized(ALICE),
            ScoringRule::CPMM,
        ));
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, CENT));

        run_to_block(100);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_to_block(102);
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_to_block(103);
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(DAVE),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_to_block(104);
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));

        let charlie_reserved = Balances::reserved_balance(&CHARLIE);
        let eve_reserved = Balances::reserved_balance(&EVE);

        run_to_block(115);
        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);
        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(CHARLIE), 0));

        // make sure rewards are right
        //
        // slashed amounts
        // ---------------------------
        // - Charlie's reserve: DisputeBond::get()
        // - Eve's reserve: DisputeBond::get() + 2 * DisputeFactor::get()
        //
        // All goes to Dave (because Bob is - strictly speaking - not a disputor).
        assert_eq!(Balances::free_balance(&CHARLIE), 1_000 * BASE - charlie_reserved);
        assert_eq!(Balances::free_balance(&EVE), 1_000 * BASE - eve_reserved);
        let total_slashed = charlie_reserved + eve_reserved;
        assert_eq!(Balances::free_balance(&DAVE), 1_000 * BASE + total_slashed);

        // The oracle report was accepted, so Alice is not slashed.
        assert_eq!(Balances::free_balance(&ALICE), 1_000 * BASE);
        assert_eq!(Balances::free_balance(&BOB), 1_000 * BASE);
    });
}

#[test]
fn it_allows_to_redeem_shares() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, CENT));
        run_to_block(100);

        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_to_block(111);
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Resolved);

        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(CHARLIE), 0));
        let bal = Balances::free_balance(&CHARLIE);
        assert_eq!(bal, 1_000 * BASE);
        System::assert_last_event(
            Event::TokensRedeemed(0, Asset::CategoricalOutcome(0, 1), CENT, CENT, CHARLIE).into(),
        );
    });
}

#[test]
fn create_market_and_deploy_assets_results_in_expected_balances() {
    let oracle = ALICE;
    let period = MarketPeriod::Block(0..42);
    let metadata = gen_metadata(42);
    let category_count = 4;
    let assets = MarketType::Categorical(category_count);
    let extra_amount = 10 * BASE;
    let min_liqudity = <Runtime as zrml_swaps::Config>::MinLiquidity::get();
    let amount = min_liqudity + 2 * extra_amount;
    let weights = vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); 5];
    let amount_base_asset = amount;
    let amounts = vec![amount - extra_amount, amount, amount, amount];
    let pool_id = 0;

    // Execute the combined convenience function
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            oracle,
            period,
            metadata,
            assets,
            MarketDisputeMechanism::SimpleDisputes,
            amount_base_asset,
            amounts,
            weights,
        ));

        let pool_account = Swaps::pool_account_id(pool_id);
        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 0), &ALICE), extra_amount);
        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 1), &ALICE), 0);
        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 2), &ALICE), 0);
        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 3), &ALICE), 0);

        assert_eq!(
            Tokens::free_balance(Asset::CategoricalOutcome(0, 0), &pool_account),
            amount - extra_amount
        );
        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 1), &pool_account), amount);
        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 2), &pool_account), amount);
        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 3), &pool_account), amount);
        assert_eq!(System::account(&pool_account).data.free, amount);
    });
}

#[test]
fn process_subsidy_activates_market_with_sufficient_subsidy() {
    ExtBuilder::default().build().execute_with(|| {
        let min_sub_period =
            <Runtime as crate::Config>::MinSubsidyPeriod::get() / (MILLISECS_PER_BLOCK as u64);
        let max_sub_period =
            <Runtime as crate::Config>::MaxSubsidyPeriod::get() / (MILLISECS_PER_BLOCK as u64);

        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            min_sub_period..max_sub_period,
            ScoringRule::RikiddoSigmoidFeeMarketEma,
        );
        let min_subsidy = <Runtime as zrml_swaps::Config>::MinSubsidy::get();
        assert_ok!(Swaps::pool_join_subsidy(Origin::signed(ALICE), 0, min_subsidy));
        run_to_block(min_sub_period);
        let subsidy_queue = crate::MarketsCollectingSubsidy::<Runtime>::get();
        assert_eq!(subsidy_queue.len(), 0);
        assert_eq!(MarketCommons::market(&0).unwrap().status, MarketStatus::Active);
    });
}

#[test]
fn process_subsidy_blocks_market_with_insufficient_subsidy() {
    ExtBuilder::default().build().execute_with(|| {
        let min_sub_period =
            <Runtime as crate::Config>::MinSubsidyPeriod::get() / (MILLISECS_PER_BLOCK as u64);
        let max_sub_period =
            <Runtime as crate::Config>::MaxSubsidyPeriod::get() / (MILLISECS_PER_BLOCK as u64);

        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            min_sub_period..max_sub_period,
            ScoringRule::RikiddoSigmoidFeeMarketEma,
        );
        let subsidy = <Runtime as zrml_swaps::Config>::MinSubsidy::get() / 3;
        assert_ok!(Swaps::pool_join_subsidy(Origin::signed(ALICE), 0, subsidy));
        assert_ok!(Swaps::pool_join_subsidy(Origin::signed(BOB), 0, subsidy));
        run_to_block(min_sub_period);
        let subsidy_queue = crate::MarketsCollectingSubsidy::<Runtime>::get();
        assert_eq!(subsidy_queue.len(), 0);
        assert_eq!(MarketCommons::market(&0).unwrap().status, MarketStatus::InsufficientSubsidy);

        // Check that the balances are correctly unreserved.
        assert_eq!(Balances::reserved_balance(&ALICE), 0);
        assert_eq!(Balances::reserved_balance(&BOB), 0);
    });
}

#[test]
fn process_subsidy_keeps_market_in_subsidy_queue_until_end_of_subsidy_phase() {
    ExtBuilder::default().build().execute_with(|| {
        let min_sub_period =
            <Runtime as crate::Config>::MinSubsidyPeriod::get() / (MILLISECS_PER_BLOCK as u64);
        let max_sub_period =
            <Runtime as crate::Config>::MaxSubsidyPeriod::get() / (MILLISECS_PER_BLOCK as u64);

        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            min_sub_period + 42..max_sub_period,
            ScoringRule::RikiddoSigmoidFeeMarketEma,
        );

        // Run to block where 2 markets are ready and process all markets.
        run_to_block(min_sub_period);
        let subsidy_queue = crate::MarketsCollectingSubsidy::<Runtime>::get();
        assert!(subsidy_queue.len() == 1);
        assert!(subsidy_queue[0].market_id == 0);
        assert!(MarketCommons::market(&0).unwrap().status == MarketStatus::CollectingSubsidy);
    });
}

#[test]
fn start_subsidy_creates_pool_and_starts_subsidy() {
    ExtBuilder::default().build().execute_with(|| {
        // Create advised categorical market using Rikiddo.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Advised,
            1337..1338,
            ScoringRule::RikiddoSigmoidFeeMarketEma,
        );
        let market_id = 0;
        let mut market = MarketCommons::market(&market_id).unwrap();

        // Ensure and set correct market status.
        assert_err!(
            PredictionMarkets::start_subsidy(&market, market_id),
            crate::Error::<Runtime>::MarketIsNotCollectingSubsidy
        );
        assert_ok!(MarketCommons::mutate_market(&market_id, |market_inner| {
            market_inner.status = MarketStatus::CollectingSubsidy;
            market = market_inner.clone();
            Ok(())
        }));

        // Pool was created and market was registered for state transition into active.
        assert_ok!(PredictionMarkets::start_subsidy(&market, market_id));
        assert_ok!(MarketCommons::market_pool(&market_id));
        let mut inserted = false;

        for market in crate::MarketsCollectingSubsidy::<Runtime>::get() {
            if market.market_id == market_id {
                inserted = true;
            }
        }

        assert!(inserted);
    });
}

#[test]
fn the_entire_market_lifecycle_works_with_timestamps() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));

        // is ok
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, CENT));

        // set the timestamp
        Timestamp::set_timestamp(123_456_789);

        assert_noop!(
            PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, CENT),
            Error::<Runtime>::MarketIsNotActive,
        );

        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
    });
}

#[test]
fn full_scalar_market_lifecycle() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            gen_metadata(3),
            MarketCreation::Permissionless,
            MarketType::Scalar(10..=30),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, 100 * BASE));

        // check balances
        let assets = PredictionMarkets::outcome_assets(0, &MarketCommons::market(&0).unwrap());
        assert_eq!(assets.len(), 2);
        for asset in assets.iter() {
            let bal = Tokens::free_balance(*asset, &CHARLIE);
            assert_eq!(bal, 100 * BASE);
        }

        Timestamp::set_timestamp(123_456_789);
        run_to_block(100);

        // report
        assert_ok!(PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Scalar(100)));

        let market_after_report = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after_report.report.is_some(), true);
        let report = market_after_report.report.unwrap();
        assert_eq!(report.at, 100);
        assert_eq!(report.by, BOB);
        assert_eq!(report.outcome, OutcomeReport::Scalar(100));

        // dispute
        assert_ok!(PredictionMarkets::dispute(Origin::signed(DAVE), 0, OutcomeReport::Scalar(25)));
        let disputes = crate::Disputes::<Runtime>::get(&0);
        assert_eq!(disputes.len(), 1);

        run_to_block(150);

        let market_after_resolve = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after_resolve.status, MarketStatus::Resolved);

        // give EVE some shares
        assert_ok!(Tokens::transfer(
            Origin::signed(CHARLIE),
            EVE,
            Asset::ScalarOutcome(0, ScalarPosition::Short),
            50 * BASE
        ));

        assert_eq!(
            Tokens::free_balance(Asset::ScalarOutcome(0, ScalarPosition::Short), &CHARLIE),
            50 * BASE
        );

        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(CHARLIE), 0));
        for asset in assets.iter() {
            let bal = Tokens::free_balance(*asset, &CHARLIE);
            assert_eq!(bal, 0);
        }

        // check payouts is right for each CHARLIE and EVE
        let ztg_bal_charlie = Balances::free_balance(&CHARLIE);
        let ztg_bal_eve = Balances::free_balance(&EVE);
        assert_eq!(ztg_bal_charlie, 98750 * CENT); // 75 (LONG) + 12.5 (SHORT) + 900 (balance)
        assert_eq!(ztg_bal_eve, 1000 * BASE);
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

        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(EVE), 0));
        let ztg_bal_eve_after = Balances::free_balance(&EVE);
        assert_eq!(ztg_bal_eve_after, 101250 * CENT); // 12.5 (SHORT) + 1000 (balance)
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
    })
}

#[test]
fn scalar_market_correctly_resolves_on_out_of_range_outcomes_below_threshold() {
    ExtBuilder::default().build().execute_with(|| {
        scalar_market_correctly_resolves_common(50);
        assert_eq!(Balances::free_balance(&CHARLIE), 900 * BASE);
        assert_eq!(Balances::free_balance(&EVE), 1100 * BASE);
    })
}

#[test]
fn scalar_market_correctly_resolves_on_out_of_range_outcomes_above_threshold() {
    ExtBuilder::default().build().execute_with(|| {
        scalar_market_correctly_resolves_common(250);
        assert_eq!(Balances::free_balance(&CHARLIE), 1000 * BASE);
        assert_eq!(Balances::free_balance(&EVE), 1000 * BASE);
    })
}

#[test]
fn reject_market_fails_on_permissionless_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        assert_noop!(
            PredictionMarkets::reject_market(Origin::signed(SUDO), 0),
            Error::<Runtime>::InvalidMarketStatus
        );
    });
}

#[test]
fn reject_market_fails_on_approved_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market::<Runtime>(
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        assert_noop!(
            PredictionMarkets::reject_market(Origin::signed(SUDO), 0),
            Error::<Runtime>::InvalidMarketStatus
        );
    });
}

#[test]
fn market_resolve_does_not_hold_liquidity_withdraw() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Block(0..100),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(3),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        deploy_swap_pool(MarketCommons::market(&0).unwrap(), 0).unwrap();
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(ALICE), 0, 1 * BASE));
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, 2 * BASE));
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, 3 * BASE));

        run_to_block(100);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(2)
        ));

        run_to_block(150);
        assert_ok!(Swaps::pool_exit(Origin::signed(FRED), 0, BASE * 100, vec![0, 0]));
        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(BOB), 0));
    })
}

#[test]
fn authorized_correctly_resolves_disputed_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Block(0..1),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            MarketDisputeMechanism::Authorized(FRED),
            ScoringRule::CPMM,
        ));
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, CENT));

        run_to_block(100);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_to_block(102);
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));

        // Fred authorizses an outcome, but fat-fingers it on the first try.
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(FRED),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(FRED),
            0,
            OutcomeReport::Categorical(1)
        ));

        run_to_block(103);
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(DAVE),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_to_block(104);
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        // check everyone's deposits
        let charlie_reserved = Balances::reserved_balance(&CHARLIE);
        assert_eq!(charlie_reserved, DisputeBond::get());

        let dave_reserved = Balances::reserved_balance(&DAVE);
        assert_eq!(dave_reserved, DisputeBond::get() + DisputeFactor::get());

        let eve_reserved = Balances::reserved_balance(&EVE);
        assert_eq!(eve_reserved, DisputeBond::get() + 2 * DisputeFactor::get());

        // check disputes length
        let disputes = crate::Disputes::<Runtime>::get(&0);
        assert_eq!(disputes.len(), 3);

        // make sure the old mappings of market id per dispute block are erased
        let market_ids_1 = MarketIdsPerDisputeBlock::<Runtime>::get(&102);
        assert_eq!(market_ids_1.len(), 0);

        let market_ids_2 = MarketIdsPerDisputeBlock::<Runtime>::get(&103);
        assert_eq!(market_ids_2.len(), 0);

        let market_ids_3 = MarketIdsPerDisputeBlock::<Runtime>::get(&104);
        assert_eq!(market_ids_3.len(), 1);

        run_to_block(115);

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);

        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(CHARLIE), 0));

        // Make sure rewards are right:
        //
        // Slashed amounts:
        //     - Dave's reserve: DisputeBond::get() + DisputeFactor::get()
        //     - Alice's oracle bond: OracleBond::get()
        // Total: OracleBond::get() + DisputeBond::get() + DisputeFactor::get()
        //
        // Charlie and Eve each receive half of the total slashed amount as bounty.
        let dave_reserved = DisputeBond::get() + DisputeFactor::get();
        let total_slashed = OracleBond::get() + dave_reserved;

        let charlie_balance = Balances::free_balance(&CHARLIE);
        assert_eq!(charlie_balance, 1_000 * BASE + total_slashed / 2);
        let charlie_reserved_2 = Balances::reserved_balance(&CHARLIE);
        assert_eq!(charlie_reserved_2, 0);
        let eve_balance = Balances::free_balance(&EVE);
        assert_eq!(eve_balance, 1_000 * BASE + total_slashed / 2);

        let dave_balance = Balances::free_balance(&DAVE);
        assert_eq!(dave_balance, 1_000 * BASE - dave_reserved);

        let alice_balance = Balances::free_balance(&ALICE);
        assert_eq!(alice_balance, 1_000 * BASE - OracleBond::get());

        // bob kinda gets away scot-free since Alice is held responsible
        // for her designated reporter
        let bob_balance = Balances::free_balance(&BOB);
        assert_eq!(bob_balance, 1_000 * BASE);
    });
}

#[test]
fn approve_market_correctly_unreserves_advisory_bond() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Block(0..100),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        // Reserve a sentinel amount to check that we don't unreserve too much.
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        let alice_balance_before = Balances::free_balance(&ALICE);
        assert_eq!(
            Balances::reserved_balance(&ALICE),
            SENTINEL_AMOUNT + AdvisoryBond::get() + OracleBond::get()
        );
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        assert_eq!(Balances::reserved_balance(&ALICE), SENTINEL_AMOUNT + OracleBond::get());
        assert_eq!(Balances::free_balance(&ALICE), alice_balance_before + AdvisoryBond::get());
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_on_oracle_report()
 {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Block(0..100),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        // Reserve a sentinel amount to check that we don't unreserve too much.
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        let alice_balance_before = Balances::free_balance(&ALICE);
        assert_eq!(
            Balances::reserved_balance(&ALICE),
            SENTINEL_AMOUNT + ValidityBond::get() + OracleBond::get()
        );
        run_to_block(100);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_to_block(150);
        assert_eq!(Balances::reserved_balance(&ALICE), SENTINEL_AMOUNT);
        assert_eq!(
            Balances::free_balance(&ALICE),
            alice_balance_before + ValidityBond::get() + OracleBond::get()
        );
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_on_outsider_report()
 {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Block(0..100),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        // Reserve a sentinel amount to check that we don't unreserve too much.
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        let alice_balance_before = Balances::free_balance(&ALICE);
        assert_eq!(
            Balances::reserved_balance(&ALICE),
            SENTINEL_AMOUNT + ValidityBond::get() + OracleBond::get()
        );
        run_to_block(9000);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_to_block(9100);
        assert_eq!(Balances::reserved_balance(&ALICE), SENTINEL_AMOUNT);
        // Check that validity bond didn't get slashed, but oracle bond did
        assert_eq!(Balances::free_balance(&ALICE), alice_balance_before + ValidityBond::get());
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_approved_advised_market_on_oracle_report()
 {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Block(0..100),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        // Reserve a sentinel amount to check that we don't unreserve too much.
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        let alice_balance_before = Balances::free_balance(&ALICE);
        assert_eq!(Balances::reserved_balance(&ALICE), SENTINEL_AMOUNT + OracleBond::get());
        run_to_block(100);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_to_block(150);
        assert_eq!(Balances::reserved_balance(&ALICE), SENTINEL_AMOUNT);
        // Check that nothing got slashed
        assert_eq!(Balances::free_balance(&ALICE), alice_balance_before + OracleBond::get());
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_approved_advised_market_on_outsider_report()
 {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Block(0..100),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        // Reserve a sentinel amount to check that we don't unreserve too much.
        assert_ok!(Balances::reserve_named(&RESERVE_ID, &ALICE, SENTINEL_AMOUNT));
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        let alice_balance_before = Balances::free_balance(&ALICE);
        assert_eq!(Balances::reserved_balance(&ALICE), SENTINEL_AMOUNT + OracleBond::get());
        run_to_block(9000);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_to_block(9100);
        // Check that oracle bond got slashed
        assert_eq!(Balances::reserved_balance(&ALICE), SENTINEL_AMOUNT);
        assert_eq!(Balances::free_balance(&ALICE), alice_balance_before);
    });
}

#[test]
fn report_fails_on_market_state_proposed() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_closed_for_advised_market() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        Timestamp::set_timestamp(123_456_789); // Market is now "closed".
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_collecting_subsidy() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Timestamp(100_000_000..200_000_000),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::RikiddoSigmoidFeeMarketEma
        ));
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_insufficient_subsidy() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Timestamp(100_000_000..200_000_000),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::RikiddoSigmoidFeeMarketEma
        ));
        let _ = MarketCommons::mutate_market(&0, |market| {
            market.status = MarketStatus::InsufficientSubsidy;
            Ok(())
        });
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_active() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_suspended() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        let _ = MarketCommons::mutate_market(&0, |market| {
            market.status = MarketStatus::Suspended;
            Ok(())
        });
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_resolved() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        Timestamp::set_timestamp(123_456_789);
        let _ = MarketCommons::mutate_market(&0, |market| {
            market.status = MarketStatus::Resolved;
            Ok(())
        });
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_if_reporter_is_not_the_oracle() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        Timestamp::set_timestamp(123_456_789);
        assert_noop!(
            PredictionMarkets::report(Origin::signed(CHARLIE), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::ReporterNotOracle,
        );
    });
}

fn deploy_swap_pool(market: Market<u128, u64, u64>, market_id: u128) -> DispatchResult {
    assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(FRED), 0, 100 * BASE));
    assert_ok!(Balances::transfer(
        Origin::signed(FRED),
        <Runtime as crate::Config>::PalletId::get().into_account(),
        100 * BASE
    ));
    let outcome_assets_len = PredictionMarkets::outcome_assets(market_id, &market).len();
    PredictionMarkets::deploy_swap_pool_for_market(
        Origin::signed(FRED),
        0,
        (0..outcome_assets_len + 1).map(|_| BASE).collect(),
    )
}

// Common code of `scalar_market_correctly_resolves_*`
fn scalar_market_correctly_resolves_common(reported_value: u128) {
    simple_create_scalar_market::<Runtime>(
        MarketCreation::Permissionless,
        0..100,
        ScoringRule::CPMM,
    );
    assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, 100 * BASE));
    assert_ok!(Tokens::transfer(
        Origin::signed(CHARLIE),
        EVE,
        Asset::ScalarOutcome(0, ScalarPosition::Short),
        100 * BASE
    ));
    // (Eve now has 100 SHORT, Charlie has 100 LONG)

    run_to_block(100);
    assert_ok!(PredictionMarkets::report(
        Origin::signed(BOB),
        0,
        OutcomeReport::Scalar(reported_value)
    ));
    let market_after_report = MarketCommons::market(&0).unwrap();
    assert_eq!(market_after_report.report.is_some(), true);
    let report = market_after_report.report.unwrap();
    assert_eq!(report.at, 100);
    assert_eq!(report.by, BOB);
    assert_eq!(report.outcome, OutcomeReport::Scalar(reported_value));

    run_to_block(150);
    let market_after_resolve = MarketCommons::market(&0).unwrap();
    assert_eq!(market_after_resolve.status, MarketStatus::Resolved);

    // Check balances before redeeming (just to make sure that our tests are based on correct
    // assumptions)!
    assert_eq!(Balances::free_balance(&CHARLIE), 900 * BASE);
    assert_eq!(Balances::free_balance(&EVE), 1000 * BASE);

    assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(CHARLIE), 0));
    assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(EVE), 0));
    let assets = PredictionMarkets::outcome_assets(0, &MarketCommons::market(&0).unwrap());
    for asset in assets.iter() {
        assert_eq!(Tokens::free_balance(*asset, &CHARLIE), 0);
        assert_eq!(Tokens::free_balance(*asset, &EVE), 0);
    }
}
