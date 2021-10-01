#![cfg(all(feature = "mock", test))]

use crate::{
    mock::*, BalanceOf, Config, Error, MarketIdOf, MarketIdsPerDisputeBlock,
    MarketIdsPerReportBlock,
};
use core::{cell::RefCell, ops::Range};
use frame_support::{assert_err, assert_noop, assert_ok, dispatch::{DispatchError, DispatchResult}, storage_root, traits::Get};

use orml_traits::MultiCurrency;
use sp_runtime::traits::AccountIdConversion;
use zeitgeist_primitives::{
    constants::{AdvisoryBond, DisputeBond, DisputeFactor, OracleBond, ValidityBond, BASE, CENT},
    types::{
        Asset, Market, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus,
        MarketType, MultiHash, OutcomeReport, ScalarPosition, ScoringRule,
    },
};
use zrml_market_commons::MarketCommonsPalletApi;

fn gen_metadata(byte: u8) -> MultiHash {
    let mut metadata = [byte; 50];
    metadata[0] = 0x15;
    metadata[1] = 0x30;
    MultiHash::Sha3_384(metadata)
}

fn simple_create_categorical_market<T: crate::Config>(
    creation: MarketCreation,
    period: Range<u64>,
    scoring_rule: ScoringRule
) {
    assert_ok!(PredictionMarkets::create_categorical_market(
        Origin::signed(ALICE),
        BOB,
        MarketPeriod::Block(period),
        gen_metadata(2),
        creation,
        T::MinCategories::get(),
        MarketDisputeMechanism::SimpleDisputes,
        scoring_rule
    ));
}

#[test]
fn it_creates_binary_markets() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market::<Runtime>(MarketCreation::Permissionless, 0..1, ScoringRule::CPMM);

        // check the correct amount was reserved
        let alice_reserved = Balances::reserved_balance(&ALICE);
        assert_eq!(alice_reserved, ValidityBond::get() + OracleBond::get());

        // Creates an advised market.
        simple_create_categorical_market::<Runtime>(MarketCreation::Advised, 0..1, ScoringRule::CPMM);

        let new_alice_reserved = Balances::reserved_balance(&ALICE);
        assert_eq!(new_alice_reserved, AdvisoryBond::get() + OracleBond::get() + alice_reserved);

        // Make sure that the market id has been incrementing
        let market_id = MarketCommons::latest_market_id().unwrap();
        assert_eq!(market_id, 1);
    });
}

#[test]
fn it_does_not_create_market_with_too_few_categories() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_categorical_market(
                Origin::signed(ALICE),
                BOB,
                MarketPeriod::Block(0..100),
                gen_metadata(2),
                MarketCreation::Advised,
                <Runtime as Config>::MinCategories::get() - 1,
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
            PredictionMarkets::create_categorical_market(
                Origin::signed(ALICE),
                BOB,
                MarketPeriod::Block(0..100),
                gen_metadata(2),
                MarketCreation::Advised,
                <Runtime as Config>::MaxCategories::get() + 1,
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
        simple_create_categorical_market::<Runtime>(MarketCreation::Advised, 0..1, ScoringRule::CPMM);

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
        simple_create_categorical_market::<Runtime>(MarketCreation::Advised, 0..1, ScoringRule::CPMM);

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
        simple_create_categorical_market::<Runtime>(MarketCreation::Advised, 0..1, ScoringRule::CPMM);

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
fn it_allows_to_buy_a_complete_set() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(MarketCreation::Permissionless, 0..1, ScoringRule::CPMM);

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
    });
}

#[test]
fn it_allows_to_deploy_a_pool() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(MarketCreation::Permissionless, 0..1, ScoringRule::CPMM);

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, 100 * BASE,));

        assert_ok!(Balances::transfer(
            Origin::signed(BOB),
            <Runtime as crate::Config>::PalletId::get().into_account(),
            100 * BASE
        ));
        assert_ok!(Tokens::deposit(Asset::Ztg, &BOB, 100 * BASE));

        assert_ok!(PredictionMarkets::deploy_swap_pool_for_market(
            Origin::signed(BOB),
            0,
            vec![BASE, BASE, BASE]
        ));
    });
}

#[test]
fn it_allows_to_sell_a_complete_set() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(MarketCreation::Permissionless, 0..1, ScoringRule::CPMM);

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
    });
}

#[test]
fn it_allows_to_report_the_outcome_of_a_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(MarketCreation::Permissionless, 0..100, ScoringRule::CPMM);

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
            market.status = MarketStatus::Closed;
            market.report = None;
            Ok(())
        });

        assert_ok!(PredictionMarkets::report(
            Origin::signed(SUDO),
            0,
            OutcomeReport::Categorical(1)
        ));

        // Reset and report again as unauthorized origin
        let _ = MarketCommons::mutate_market(&0, |market| {
            market.status = MarketStatus::Closed;
            market.report = None;
            Ok(())
        });

        assert_noop!(
            PredictionMarkets::report(Origin::signed(CHARLIE), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::ReporterNotOracle
        );
    });
}

#[test]
fn it_allows_to_dispute_the_outcome_of_a_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market::<Runtime>(MarketCreation::Permissionless, 0..1, ScoringRule::CPMM);

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
        simple_create_categorical_market::<Runtime>(MarketCreation::Permissionless, 0..1, ScoringRule::CPMM);

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
        simple_create_categorical_market::<Runtime>(MarketCreation::Permissionless, 0..1, ScoringRule::CPMM);

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
        simple_create_categorical_market::<Runtime>(MarketCreation::Permissionless, 0..1, ScoringRule::CPMM);

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

        // make sure rewards are right
        //
        // slashed amounts
        // ---------------------------
        // - OracleBond: 50 * CENT
        // - Dave's reserve: DisputeBond::get() + DisputeFactor::get()
        // Total: 50 * CENT + DisputeBond::get() + DisputeFactor::get()
        // Per each: 25 * CENT + (DisputeBond::get() + DisputeFactor::get()) / 2

        let dave_reserved = DisputeBond::get() + DisputeFactor::get();

        let charlie_balance = Balances::free_balance(&CHARLIE);
        assert_eq!(charlie_balance, 1_000 * BASE + 25 * CENT + dave_reserved / 2);
        let charlie_reserved_2 = Balances::reserved_balance(&CHARLIE);
        assert_eq!(charlie_reserved_2, 0);
        let eve_balance = Balances::free_balance(&EVE);
        assert_eq!(eve_balance, 1_000 * BASE + 25 * CENT + dave_reserved / 2);

        let dave_balance = Balances::free_balance(&DAVE);
        assert_eq!(dave_balance, 1_000 * BASE - dave_reserved);

        let alice_balance = Balances::free_balance(&ALICE);
        assert_eq!(alice_balance, 1_000 * BASE - 50 * CENT);

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
        simple_create_categorical_market::<Runtime>(MarketCreation::Permissionless, 0..1, ScoringRule::CPMM);

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
    });
}

#[test]
fn create_market_and_deploy_assets_is_identical_to_sequential_calls() {
    let oracle = ALICE;
    let period = MarketPeriod::Block(0..42);
    let metadata = gen_metadata(42);
    let creation = MarketCreation::Permissionless;
    let category_count = 4;
    let assets = MarketType::Categorical(category_count);
    let extra_amount = 50 * BASE;
    let amount = <Runtime as zrml_swaps::Config>::MinLiquidity::get() + extra_amount;
    let weights = vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); 5];
    let add_additional: Vec<(Asset<MarketIdOf<Runtime>>, BalanceOf<Runtime>, BalanceOf<Runtime>)> = vec![
        (Asset::CategoricalOutcome(0, 0), extra_amount, 0),
        (Asset::CategoricalOutcome(0, 1), extra_amount, 0),
        (Asset::CategoricalOutcome(0, 3), extra_amount, 0),
    ];

    let first_state = RefCell::new(vec![]);
    let second_state = RefCell::new(vec![]);

    // Execute the combined convenience function
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            oracle,
            period.clone(),
            metadata.clone(),
            creation.clone(),
            assets,
            amount,
            weights.clone(),
            add_additional.clone(),
            MarketDisputeMechanism::SimpleDisputes
        ));

        *first_state.borrow_mut() = storage_root();
    });

    // Execute every command included in the convencience function one-by-one
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_categorical_market(
            Origin::signed(ALICE),
            oracle,
            period.clone(),
            metadata,
            creation,
            category_count,
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(ALICE), 0, amount));
        assert_ok!(PredictionMarkets::deploy_swap_pool_for_market(
            Origin::signed(ALICE),
            0,
            weights
        ));

        for (asset_in, asset_amount, min_pool_amount) in add_additional {
            assert_ok!(Swaps::pool_join_with_exact_asset_amount(
                Origin::signed(ALICE),
                0,
                asset_in,
                asset_amount,
                min_pool_amount
            ));
        }

        *second_state.borrow_mut() = storage_root();
    });

    // Compare resulting state
    assert_eq!(*first_state.borrow(), *second_state.borrow());
}

#[test]
fn start_subsidy_creates_pool_and_starts_subsidy() {
    ExtBuilder::default().build().execute_with(|| {
        // Create advised categorical market using Rikiddo.
        simple_create_categorical_market::<Runtime>(MarketCreation::Advised, 1337..1337, ScoringRule::RikiddoSigmoidFeeMarketEma);
        let market_id = 0;
        let mut market = MarketCommons::market(&market_id).unwrap();
        
        // Ensure and set correct market status.
        assert_err!(PredictionMarkets::start_subsidy(&market, market_id), crate::Error::<Runtime>::MarketIsNotCollectingSubsidy);
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
        assert_ok!(PredictionMarkets::create_categorical_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            gen_metadata(2),
            MarketCreation::Permissionless,
            2,
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
        assert_ok!(PredictionMarkets::create_scalar_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            gen_metadata(3),
            MarketCreation::Permissionless,
            10..=30,
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, 100 * BASE,));

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
        assert_ok!(PredictionMarkets::dispute(Origin::signed(DAVE), 0, OutcomeReport::Scalar(20)));
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
            100 * BASE
        ));

        assert_eq!(
            Tokens::free_balance(Asset::ScalarOutcome(0, ScalarPosition::Short), &CHARLIE),
            0
        );

        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(CHARLIE), 0));
        for asset in assets.iter() {
            let bal = Tokens::free_balance(*asset, &CHARLIE);
            assert_eq!(bal, 0);
        }

        // check payouts is right for each CHARLIE and EVE
        let ztg_bal_charlie = Balances::free_balance(&CHARLIE);
        let ztg_bal_eve = Balances::free_balance(&EVE);
        assert_eq!(ztg_bal_charlie, 950 * BASE);
        assert_eq!(ztg_bal_eve, 1000 * BASE);

        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(EVE), 0));
        let ztg_bal_eve_after = Balances::free_balance(&EVE);
        assert_eq!(ztg_bal_eve_after, 1050 * BASE);
    })
}

#[test]
fn market_resolve_does_not_hold_liquidity_withdraw() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_categorical_market(
            Origin::signed(ALICE),
            BOB,
            MarketPeriod::Block(0..100),
            gen_metadata(2),
            MarketCreation::Permissionless,
            3,
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
        assert_ok!(Swaps::pool_exit(Origin::signed(BOB), 0, BASE * 100, vec![0, 0]));
        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(BOB), 0));
    })
}

fn deploy_swap_pool(market: Market<u128, u64, u64>, market_id: u128) -> DispatchResult {
    assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(FRED), 0, 100 * BASE,));
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
