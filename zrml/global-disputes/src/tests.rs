#![cfg(test)]

use crate::{
    global_disputes_pallet_api::GlobalDisputesPalletApi,
    mock::{
        Balances, ExtBuilder, GlobalDisputes, MarketCommons, Origin, Runtime, ALICE, BOB, CHARLIE,
        DAVE, EVE,
    },
    DisputeVotes, Error, LockInfoOf,
};
use frame_support::{assert_noop, assert_ok};
use pallet_balances::BalanceLock;
use test_case::test_case;
use zeitgeist_primitives::{
    constants::{MinDisputeVoteAmount, VoteLockIdentifier, BASE},
    traits::DisputeApi,
    types::{
        Market, MarketCreation, MarketDispute, MarketDisputeMechanism, MarketPeriod, MarketStatus,
        MarketType, OutcomeReport, ScoringRule,
    },
};
use zrml_market_commons::MarketCommonsPalletApi;

const DEFAULT_MARKET: Market<u128, u64, u64> = Market {
    creation: MarketCreation::Permissionless,
    creator_fee: 0,
    creator: 0,
    market_type: MarketType::Scalar(0..=100),
    dispute_mechanism: MarketDisputeMechanism::GlobalDisputes,
    metadata: vec![],
    oracle: 0,
    period: MarketPeriod::Block(0..100),
    report: None,
    resolved_outcome: None,
    scoring_rule: ScoringRule::CPMM,
    status: MarketStatus::Disputed,
};

fn the_lock(amount: u128) -> BalanceLock<u128> {
    BalanceLock { id: VoteLockIdentifier::get(), amount, reasons: pallet_balances::Reasons::Misc }
}

#[test]
fn vote_fails_if_insufficient_amount() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = MarketStatus::Disputed;
        assert_ok!(MarketCommons::push_market(market.clone()));
        let market_id = MarketCommons::latest_market_id().unwrap();
        GlobalDisputes::init_dispute_vote(&market_id, 0, 10 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 1, 20 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 2, 30 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 3, 40 * BASE);

        assert_noop!(
            GlobalDisputes::vote(
                Origin::signed(ALICE),
                market_id,
                2u32,
                MinDisputeVoteAmount::get() - 1,
            ),
            Error::<Runtime>::InsufficientAmount
        );
    });
}

#[test_case(MarketDisputeMechanism::Court; "court")]
#[test_case(MarketDisputeMechanism::SimpleDisputes; "simple disputes")]
#[test_case(MarketDisputeMechanism::Authorized(0); "authorized")]
fn on_dispute_denies_non_global_disputes_markets(dispute_mechanism: MarketDisputeMechanism<u128>) {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.dispute_mechanism = dispute_mechanism;
        assert_noop!(
            GlobalDisputes::on_dispute(&[], &0, &market),
            Error::<Runtime>::MarketDoesNotHaveGlobalDisputesMechanism
        );
    });
}

#[test_case(MarketDisputeMechanism::Court; "court")]
#[test_case(MarketDisputeMechanism::SimpleDisputes; "simple disputes")]
#[test_case(MarketDisputeMechanism::Authorized(0); "authorized")]
fn on_resolution_denies_non_global_disputes_markets(
    dispute_mechanism: MarketDisputeMechanism<u128>,
) {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.dispute_mechanism = dispute_mechanism;
        assert_noop!(
            GlobalDisputes::on_resolution(&[], &0, &market),
            Error::<Runtime>::MarketDoesNotHaveGlobalDisputesMechanism
        );
    });
}

#[test_case(MarketStatus::Closed; "closed")]
#[test_case(MarketStatus::Reported; "reported")]
#[test_case(MarketStatus::Active; "active")]
#[test_case(MarketStatus::Resolved; "resolved")]
#[test_case(MarketStatus::Proposed; "proposed")]
#[test_case(MarketStatus::CollectingSubsidy; "collecting subsidy")]
#[test_case(MarketStatus::InsufficientSubsidy; "insufficient subsidy")]
fn on_resolution_denies_non_disputed_markets(status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = status;
        assert_noop!(
            GlobalDisputes::on_resolution(&[], &0, &market),
            Error::<Runtime>::InvalidMarketStatus
        );
    });
}

#[test]
fn on_resolution_sets_the_last_dispute_for_same_vote_balances_as_the_canonical_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = MarketStatus::Disputed;
        let disputes = [
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(0) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(20) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(40) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(60) },
        ];
        assert_ok!(MarketCommons::push_market(market.clone()));
        let market_id = MarketCommons::latest_market_id().unwrap();

        GlobalDisputes::init_dispute_vote(&market_id, 0, 100 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 1, 100 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 2, 100 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 3, 100 * BASE);

        assert_eq!(
            &GlobalDisputes::on_resolution(&disputes, &0, &market).unwrap().unwrap(),
            &disputes.get(3).unwrap().outcome
        );
    });
}

#[test]
fn on_resolution_sets_the_highest_vote_of_disputed_markets_as_the_canonical_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = MarketStatus::Disputed;
        let disputes = [
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(0) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(20) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(40) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(60) },
        ];
        assert_ok!(MarketCommons::push_market(market.clone()));
        let market_id = MarketCommons::latest_market_id().unwrap();

        let reinitialize_disputes = || {
            GlobalDisputes::init_dispute_vote(&market_id, 0, 100 * BASE);
            GlobalDisputes::init_dispute_vote(&market_id, 1, 100 * BASE);
            GlobalDisputes::init_dispute_vote(&market_id, 2, 100 * BASE);
            GlobalDisputes::init_dispute_vote(&market_id, 3, 100 * BASE);
        };

        reinitialize_disputes();
        assert_ok!(GlobalDisputes::vote(Origin::signed(ALICE), market_id, 0u32, 10 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(BOB), market_id, 1u32, 10 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(CHARLIE), market_id, 2u32, 11 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(EVE), market_id, 3u32, 10 * BASE));

        assert_eq!(
            &GlobalDisputes::on_resolution(&disputes, &0, &market).unwrap().unwrap(),
            &disputes.get(2).unwrap().outcome
        );

        reinitialize_disputes();
        assert_ok!(GlobalDisputes::vote(Origin::signed(ALICE), market_id, 3u32, 10 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(BOB), market_id, 1u32, 50 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(CHARLIE), market_id, 3u32, 20 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(EVE), market_id, 3u32, 21 * BASE));

        assert_eq!(<DisputeVotes<Runtime>>::get(market_id, 0u32).unwrap(), 100 * BASE);
        assert_eq!(<DisputeVotes<Runtime>>::get(market_id, 1u32).unwrap(), 150 * BASE);
        assert_eq!(<DisputeVotes<Runtime>>::get(market_id, 2u32).unwrap(), 100 * BASE);
        assert_eq!(<DisputeVotes<Runtime>>::get(market_id, 3u32).unwrap(), 151 * BASE);

        assert_eq!(
            &GlobalDisputes::on_resolution(&disputes, &0, &market).unwrap().unwrap(),
            &disputes.get(3).unwrap().outcome
        );

        reinitialize_disputes();
        assert_ok!(GlobalDisputes::vote(Origin::signed(ALICE), market_id, 1u32, 30 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(BOB), market_id, 2u32, 50 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(CHARLIE), market_id, 0u32, 10 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(EVE), market_id, 0u32, 41 * BASE));

        assert_eq!(
            &GlobalDisputes::on_resolution(&disputes, &0, &market).unwrap().unwrap(),
            &disputes.get(0).unwrap().outcome
        );

        reinitialize_disputes();
        assert_ok!(GlobalDisputes::vote(Origin::signed(ALICE), market_id, 1u32, 1 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(CHARLIE), market_id, 0u32, 10 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(ALICE), market_id, 1u32, 10 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(EVE), market_id, 0u32, 40 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(ALICE), market_id, 1u32, 40 * BASE));

        assert_eq!(
            &GlobalDisputes::on_resolution(&disputes, &0, &market).unwrap().unwrap(),
            &disputes.get(1).unwrap().outcome
        );
    });
}

#[test]
fn on_resolution_clears_dispute_votes() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = MarketStatus::Disputed;
        let disputes = [
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(0) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(20) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(40) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(60) },
        ];
        assert_ok!(MarketCommons::push_market(market.clone()));
        let market_id = MarketCommons::latest_market_id().unwrap();
        GlobalDisputes::init_dispute_vote(&market_id, 0, 10 * BASE);

        let mut dispute_votes = <DisputeVotes<Runtime>>::iter();
        assert_eq!(dispute_votes.next(), Some((market_id, 0u32, 10 * BASE)));
        assert_eq!(dispute_votes.next(), None);

        assert_ok!(GlobalDisputes::on_resolution(&disputes, &0, &market));

        assert_eq!(<DisputeVotes<Runtime>>::iter_keys().next(), None);
    });
}

#[test]
fn unlock_clears_lock_info() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = MarketStatus::Disputed;
        let disputes = [
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(0) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(20) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(40) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(60) },
        ];
        assert_ok!(MarketCommons::push_market(market.clone()));
        let market_id = MarketCommons::latest_market_id().unwrap();
        GlobalDisputes::init_dispute_vote(&market_id, 0, 10 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 1, 20 * BASE);

        assert_ok!(GlobalDisputes::vote(Origin::signed(ALICE), market_id, 0u32, 50 * BASE));

        assert_ok!(GlobalDisputes::on_resolution(&disputes, &market_id, &market));

        assert!(<LockInfoOf<Runtime>>::get(ALICE, market_id).is_some());

        assert_ok!(GlobalDisputes::unlock(Origin::signed(ALICE)));

        assert!(<LockInfoOf<Runtime>>::get(ALICE, market_id).is_none());
    });
}

#[test_case(MarketStatus::Closed; "closed")]
#[test_case(MarketStatus::Reported; "reported")]
#[test_case(MarketStatus::Active; "active")]
#[test_case(MarketStatus::Resolved; "resolved")]
#[test_case(MarketStatus::Proposed; "proposed")]
#[test_case(MarketStatus::CollectingSubsidy; "collecting subsidy")]
#[test_case(MarketStatus::InsufficientSubsidy; "insufficient subsidy")]
fn vote_denies_non_disputed_markets(status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = status;
        assert_ok!(MarketCommons::push_market(market));
        let market_id = MarketCommons::latest_market_id().unwrap();
        GlobalDisputes::init_dispute_vote(&market_id, 0, 10 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 1, 20 * BASE);
        assert_noop!(
            GlobalDisputes::vote(Origin::signed(ALICE), market_id, 0u32, 50 * BASE),
            Error::<Runtime>::InvalidMarketStatus
        );
    });
}

#[test]
fn vote_fails_if_dispute_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = MarketStatus::Disputed;
        assert_ok!(MarketCommons::push_market(market));
        let market_id = MarketCommons::latest_market_id().unwrap();
        GlobalDisputes::init_dispute_vote(&market_id, 0, 10 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 1, 20 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 2, 30 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 3, 40 * BASE);

        assert_noop!(
            GlobalDisputes::vote(Origin::signed(ALICE), market_id, 42u32, 50 * BASE),
            Error::<Runtime>::DisputeDoesNotExist
        );
    });
}

#[test]
fn vote_fails_for_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = MarketStatus::Disputed;
        assert_ok!(MarketCommons::push_market(market));
        let market_id = MarketCommons::latest_market_id().unwrap();
        GlobalDisputes::init_dispute_vote(&market_id, 0, 10 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 1, 20 * BASE);

        assert_noop!(
            GlobalDisputes::vote(Origin::signed(DAVE), market_id, 0u32, 50 * BASE),
            Error::<Runtime>::InsufficientFundsForVote
        );
    });
}

#[test]
fn vote_fails_if_dispute_len_below_two() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = MarketStatus::Disputed;
        assert_ok!(MarketCommons::push_market(DEFAULT_MARKET));
        let market_id = MarketCommons::latest_market_id().unwrap();

        assert_noop!(
            GlobalDisputes::vote(Origin::signed(ALICE), market_id, 0u32, 50 * BASE),
            Error::<Runtime>::NotEnoughDisputes
        );

        GlobalDisputes::init_dispute_vote(&market_id, 0u32, 10 * BASE);

        assert_noop!(
            GlobalDisputes::vote(Origin::signed(ALICE), market_id, 0u32, 50 * BASE),
            Error::<Runtime>::NotEnoughDisputes
        );
    });
}

#[test]
fn locking_works_for_one_market() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = MarketStatus::Disputed;
        let disputes = [
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(0) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(20) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(40) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(60) },
        ];
        assert_ok!(MarketCommons::push_market(market.clone()));
        let market_id = MarketCommons::latest_market_id().unwrap();
        GlobalDisputes::init_dispute_vote(&market_id, 0, 10 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 1, 20 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 2, 30 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id, 3, 40 * BASE);

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id), None);
        assert!(Balances::locks(BOB).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, market_id), None);
        assert!(Balances::locks(CHARLIE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, market_id), None);
        assert!(Balances::locks(EVE).is_empty());

        assert_ok!(GlobalDisputes::vote(Origin::signed(ALICE), market_id, 0u32, 50 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(BOB), market_id, 1u32, 40 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(CHARLIE), market_id, 2u32, 30 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(EVE), market_id, 3u32, 20 * BASE));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id), Some(50 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id), Some(40 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, market_id), Some(30 * BASE));
        assert_eq!(Balances::locks(CHARLIE), vec![the_lock(30 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, market_id), Some(20 * BASE));
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::on_resolution(&disputes, &market_id, &market));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id), Some(50 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);

        assert_ok!(GlobalDisputes::unlock(Origin::signed(ALICE)));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id), None);
        assert!(Balances::locks(ALICE).is_empty());

        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id), Some(40 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, market_id), Some(30 * BASE));
        assert_eq!(Balances::locks(CHARLIE), vec![the_lock(30 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, market_id), Some(20 * BASE));
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock(Origin::signed(BOB)));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id), None);
        assert!(Balances::locks(BOB).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, market_id), Some(30 * BASE));
        assert_eq!(Balances::locks(CHARLIE), vec![the_lock(30 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, market_id), Some(20 * BASE));
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock(Origin::signed(CHARLIE)));
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, market_id), None);
        assert!(Balances::locks(CHARLIE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, market_id), Some(20 * BASE));
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock(Origin::signed(EVE)));
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, market_id), None);
        assert!(Balances::locks(EVE).is_empty());
    });
}

#[test]
fn locking_works_for_two_markets_with_stronger_first_unlock() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = MarketStatus::Disputed;
        let disputes = [
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(0) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(20) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(40) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(60) },
        ];
        assert_ok!(MarketCommons::push_market(market.clone()));
        let market_id_1 = MarketCommons::latest_market_id().unwrap();

        assert_ok!(MarketCommons::push_market(market.clone()));
        let market_id_2 = MarketCommons::latest_market_id().unwrap();

        GlobalDisputes::init_dispute_vote(&market_id_1, 0, 10 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id_1, 1, 20 * BASE);

        GlobalDisputes::init_dispute_vote(&market_id_2, 0, 10 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id_2, 1, 20 * BASE);

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), None);
        assert!(Balances::locks(BOB).is_empty());

        assert_ok!(GlobalDisputes::vote(Origin::signed(ALICE), market_id_1, 0u32, 50 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(BOB), market_id_1, 1u32, 40 * BASE));

        assert_ok!(GlobalDisputes::vote(Origin::signed(ALICE), market_id_2, 0u32, 30 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(BOB), market_id_2, 1u32, 20 * BASE));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), Some(50 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), Some(30 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), Some(40 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), Some(20 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        // market_id_1 has stronger locks
        assert_ok!(GlobalDisputes::on_resolution(&disputes, &market_id_1, &market));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), Some(50 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), Some(30 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_ok!(GlobalDisputes::unlock(Origin::signed(ALICE)));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), Some(30 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(30 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), Some(40 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), Some(20 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        assert_ok!(GlobalDisputes::unlock(Origin::signed(BOB)));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), Some(20 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(20 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), Some(30 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(30 * BASE)]);

        assert_ok!(GlobalDisputes::on_resolution(&disputes, &market_id_2, &market));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), Some(30 * BASE));
        assert_ok!(GlobalDisputes::unlock(Origin::signed(ALICE)));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), Some(20 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock(Origin::signed(BOB)));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), None);
        assert!(Balances::locks(BOB).is_empty());
    });
}

#[test]
fn locking_works_for_two_markets_with_weaker_first_unlock() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = MarketStatus::Disputed;
        let disputes = [
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(0) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(20) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(40) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(60) },
        ];
        assert_ok!(MarketCommons::push_market(market.clone()));
        let market_id_1 = MarketCommons::latest_market_id().unwrap();

        assert_ok!(MarketCommons::push_market(market.clone()));
        let market_id_2 = MarketCommons::latest_market_id().unwrap();

        GlobalDisputes::init_dispute_vote(&market_id_1, 0, 10 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id_1, 1, 20 * BASE);

        GlobalDisputes::init_dispute_vote(&market_id_2, 0, 10 * BASE);
        GlobalDisputes::init_dispute_vote(&market_id_2, 1, 20 * BASE);

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), None);
        assert!(Balances::locks(BOB).is_empty());

        assert_ok!(GlobalDisputes::vote(Origin::signed(ALICE), market_id_1, 0u32, 50 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(BOB), market_id_1, 1u32, 40 * BASE));

        assert_ok!(GlobalDisputes::vote(Origin::signed(ALICE), market_id_2, 0u32, 30 * BASE));
        assert_ok!(GlobalDisputes::vote(Origin::signed(BOB), market_id_2, 1u32, 20 * BASE));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), Some(50 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), Some(30 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), Some(40 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), Some(20 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        // market_id_2 has weaker locks
        assert_ok!(GlobalDisputes::on_resolution(&disputes, &market_id_2, &market));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), Some(50 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), Some(30 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_ok!(GlobalDisputes::unlock(Origin::signed(ALICE)));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), Some(50 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), None);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), Some(40 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), Some(20 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        assert_ok!(GlobalDisputes::unlock(Origin::signed(BOB)));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), Some(40 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), None);
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), Some(50 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), None);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);

        assert_ok!(GlobalDisputes::on_resolution(&disputes, &market_id_1, &market));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), Some(50 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), None);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_ok!(GlobalDisputes::unlock(Origin::signed(ALICE)));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), Some(40 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), None);
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        assert_ok!(GlobalDisputes::unlock(Origin::signed(BOB)));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), None);
        assert!(Balances::locks(BOB).is_empty());
    });
}
