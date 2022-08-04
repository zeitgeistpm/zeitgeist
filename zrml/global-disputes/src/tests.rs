#![cfg(test)]

use crate::{
    global_disputes_pallet_api::GlobalDisputesPalletApi,
    mock::{
        Balances, ExtBuilder, GlobalDisputes, Origin, Runtime, ALICE, BOB, CHARLIE, EVE, POOR_PAUL,
    },
    Error, LockInfoOf, OutcomeInfo, Outcomes, Winners,
};
use frame_support::{assert_noop, assert_ok, traits::ReservableCurrency, BoundedVec};
use pallet_balances::BalanceLock;
use zeitgeist_primitives::{
    constants::{MinOutcomeVoteAmount, VoteLockIdentifier, BASE},
    types::OutcomeReport,
};

fn the_lock(amount: u128) -> BalanceLock<u128> {
    BalanceLock { id: VoteLockIdentifier::get(), amount, reasons: pallet_balances::Reasons::Misc }
}

#[test]
fn vote_fails_if_insufficient_amount() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(0),
            &ALICE,
            10 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(20),
            &ALICE,
            20 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(40),
            &ALICE,
            30 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(60),
            &ALICE,
            40 * BASE,
        );

        assert_noop!(
            GlobalDisputes::vote_on_outcome(
                Origin::signed(ALICE),
                market_id,
                OutcomeReport::Scalar(40),
                MinOutcomeVoteAmount::get() - 1,
            ),
            Error::<Runtime>::AmountTooLow
        );
    });
}

#[test]
fn get_voting_winner_sets_the_last_outcome_for_same_vote_balances_as_the_canonical_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(0),
            &ALICE,
            10 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(20),
            &ALICE,
            10 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(40),
            &ALICE,
            10 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(60),
            &ALICE,
            10 * BASE,
        );

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(0),
            42 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            market_id,
            OutcomeReport::Scalar(20),
            42 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(CHARLIE),
            market_id,
            OutcomeReport::Scalar(40),
            42 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(EVE),
            market_id,
            OutcomeReport::Scalar(60),
            42 * BASE
        ));
        assert_eq!(
            &GlobalDisputes::get_voting_winner(&market_id).unwrap(),
            &OutcomeReport::Scalar(60)
        );
    });
}

#[test]
fn reserve_before_init_vote_outcome_is_not_allowed_for_voting() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;

        let disputor = &ALICE;
        let free_balance_disputor_before = Balances::free_balance(disputor);
        let arbitrary_amount = 42 * BASE;
        let reserved_balance_disputor = free_balance_disputor_before - arbitrary_amount;

        assert_ok!(Balances::reserve(disputor, reserved_balance_disputor));
        assert_eq!(
            Balances::free_balance(disputor),
            free_balance_disputor_before - reserved_balance_disputor
        );

        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(0),
            &ALICE,
            reserved_balance_disputor,
        );

        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(20),
            &ALICE,
            reserved_balance_disputor * 2,
        );

        assert_noop!(
            GlobalDisputes::vote_on_outcome(
                Origin::signed(*disputor),
                market_id,
                OutcomeReport::Scalar(0),
                arbitrary_amount + 1
            ),
            Error::<Runtime>::InsufficientAmount
        );

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(*disputor),
            market_id,
            OutcomeReport::Scalar(0),
            arbitrary_amount
        ));

        assert_eq!(
            Balances::free_balance(disputor),
            free_balance_disputor_before - reserved_balance_disputor
        );
        assert_eq!(Balances::reserved_balance(disputor), reserved_balance_disputor);
        assert_eq!(Balances::locks(*disputor), vec![the_lock(arbitrary_amount)]);
    });
}

#[test]
fn transfer_fails_with_fully_locked_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;

        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(0),
            &ALICE,
            10 * BASE,
        );

        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(20),
            &ALICE,
            20 * BASE,
        );

        let disputor = &ALICE;
        let free_balance_disputor_before = Balances::free_balance(disputor);
        let arbitrary_amount = 42 * BASE;

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(*disputor),
            market_id,
            OutcomeReport::Scalar(0),
            free_balance_disputor_before - arbitrary_amount
        ));

        assert_eq!(Balances::free_balance(disputor), free_balance_disputor_before);
        assert_eq!(
            Balances::locks(*disputor),
            vec![the_lock(free_balance_disputor_before - arbitrary_amount)]
        );

        assert_noop!(
            Balances::transfer(Origin::signed(*disputor), BOB, arbitrary_amount + 1),
            pallet_balances::Error::<Runtime>::LiquidityRestrictions
        );
        assert_ok!(Balances::transfer(Origin::signed(*disputor), BOB, arbitrary_amount));
    });
}

#[test]
fn reserve_fails_with_fully_locked_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;

        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(0),
            &ALICE,
            10 * BASE,
        );

        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(20),
            &ALICE,
            20 * BASE,
        );

        let disputor = &ALICE;
        let free_balance_disputor_before = Balances::free_balance(disputor);
        let arbitrary_amount = 42 * BASE;

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(*disputor),
            market_id,
            OutcomeReport::Scalar(0),
            free_balance_disputor_before - arbitrary_amount
        ));

        assert_eq!(Balances::free_balance(disputor), free_balance_disputor_before);
        assert_eq!(
            Balances::locks(*disputor),
            vec![the_lock(free_balance_disputor_before - arbitrary_amount)]
        );

        assert_noop!(
            Balances::reserve(disputor, arbitrary_amount + 1),
            pallet_balances::Error::<Runtime>::LiquidityRestrictions
        );
        assert_ok!(Balances::reserve(disputor, arbitrary_amount));
    });
}

#[test]
fn get_voting_winner_sets_the_highest_vote_of_outcome_markets_as_the_canonical_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market_id = 0u128;
        let reinitialize_outcomes = |market_id| {
            GlobalDisputes::push_voting_outcome(
                &market_id,
                OutcomeReport::Scalar(0),
                &ALICE,
                100 * BASE,
            );

            GlobalDisputes::push_voting_outcome(
                &market_id,
                OutcomeReport::Scalar(20),
                &ALICE,
                100 * BASE,
            );
            GlobalDisputes::push_voting_outcome(
                &market_id,
                OutcomeReport::Scalar(40),
                &ALICE,
                100 * BASE,
            );

            GlobalDisputes::push_voting_outcome(
                &market_id,
                OutcomeReport::Scalar(60),
                &ALICE,
                100 * BASE,
            );
        };

        market_id += 1;
        reinitialize_outcomes(market_id);
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(0),
            10 * BASE
        ));

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            market_id,
            OutcomeReport::Scalar(20),
            10 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(CHARLIE),
            market_id,
            OutcomeReport::Scalar(40),
            11 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(EVE),
            market_id,
            OutcomeReport::Scalar(60),
            10 * BASE
        ));

        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(0)).unwrap(),
            OutcomeInfo {
                outcome_sum: 110 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );
        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(20)).unwrap(),
            OutcomeInfo {
                outcome_sum: 110 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );
        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(40)).unwrap(),
            OutcomeInfo {
                outcome_sum: 111 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );
        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(60)).unwrap(),
            OutcomeInfo {
                outcome_sum: 110 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );

        assert_eq!(
            GlobalDisputes::get_voting_winner(&market_id).unwrap(),
            OutcomeReport::Scalar(40)
        );

        assert!(<Winners<Runtime>>::get(market_id).unwrap().is_finished);

        market_id += 1;
        reinitialize_outcomes(market_id);
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(60),
            10 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            market_id,
            OutcomeReport::Scalar(20),
            50 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(CHARLIE),
            market_id,
            OutcomeReport::Scalar(60),
            20 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(EVE),
            market_id,
            OutcomeReport::Scalar(60),
            21 * BASE
        ));

        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(0)).unwrap(),
            OutcomeInfo {
                outcome_sum: 100 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );
        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(20)).unwrap(),
            OutcomeInfo {
                outcome_sum: 150 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );
        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(40)).unwrap(),
            OutcomeInfo {
                outcome_sum: 100 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );
        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(60)).unwrap(),
            OutcomeInfo {
                outcome_sum: 151 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );

        assert_eq!(
            GlobalDisputes::get_voting_winner(&market_id).unwrap(),
            OutcomeReport::Scalar(60)
        );

        market_id += 1;
        reinitialize_outcomes(market_id);
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(20),
            30 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            market_id,
            OutcomeReport::Scalar(40),
            50 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(CHARLIE),
            market_id,
            OutcomeReport::Scalar(0),
            10 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(EVE),
            market_id,
            OutcomeReport::Scalar(0),
            41 * BASE
        ));

        assert_eq!(
            GlobalDisputes::get_voting_winner(&market_id).unwrap(),
            OutcomeReport::Scalar(0)
        );

        market_id += 1;
        reinitialize_outcomes(market_id);
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(20),
            BASE
        ));

        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(20)).unwrap(),
            OutcomeInfo {
                outcome_sum: 101 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(CHARLIE),
            market_id,
            OutcomeReport::Scalar(0),
            10 * BASE
        ));

        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(0)).unwrap(),
            OutcomeInfo {
                outcome_sum: 110 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(20),
            10 * BASE
        ));

        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(20)).unwrap(),
            OutcomeInfo {
                outcome_sum: 110 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(EVE),
            market_id,
            OutcomeReport::Scalar(0),
            40 * BASE
        ));
        // Eve and Charlie have more together
        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(0)).unwrap(),
            OutcomeInfo {
                outcome_sum: 150 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(20),
            40 * BASE
        ));
        // Alice updates here voting balance (instead of accumulating)
        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(20)).unwrap(),
            OutcomeInfo {
                outcome_sum: 140 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );

        assert_eq!(
            GlobalDisputes::get_voting_winner(&market_id).unwrap(),
            OutcomeReport::Scalar(0)
        );
    });
}

#[test]
fn reward_outcome_owner_cleans_outcome_info() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        GlobalDisputes::push_voting_outcome(&market_id, OutcomeReport::Scalar(0), &ALICE, 0);
        GlobalDisputes::push_voting_outcome(&market_id, OutcomeReport::Scalar(20), &ALICE, 0);

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(0),
            10 * BASE
        ));

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            market_id,
            OutcomeReport::Scalar(20),
            10 * BASE
        ));

        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(0)).unwrap(),
            OutcomeInfo {
                outcome_sum: 10 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );
        assert_eq!(
            <Outcomes<Runtime>>::get(market_id, OutcomeReport::Scalar(20)).unwrap(),
            OutcomeInfo {
                outcome_sum: 10 * BASE,
                owners: BoundedVec::try_from(vec![ALICE]).unwrap()
            }
        );

        assert!(GlobalDisputes::get_voting_winner(&market_id).is_some());

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));

        assert_ok!(GlobalDisputes::reward_outcome_owner(Origin::signed(BOB), market_id,));

        // figure out why this doesnt emit: System::assert_has_event(PEvent::<Runtime>::OutcomesFullyCleaned(market_id).into());
        // System::assert_last_event(PEvent::<Runtime>::OutcomesFullyCleaned(market_id).into());
        // System::assert_has_event(PEvent::<Runtime>::OutcomesFullyCleaned(market_id).into());

        assert_eq!(<Outcomes<Runtime>>::iter_prefix(market_id).next(), None);
    });
}

#[test]
fn unlock_clears_lock_info() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;

        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(0),
            &ALICE,
            10 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(20),
            &ALICE,
            20 * BASE,
        );

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(0),
            50 * BASE
        ));

        assert!(GlobalDisputes::get_voting_winner(&market_id).is_some());

        assert!(<LockInfoOf<Runtime>>::get(ALICE, market_id).is_some());

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));

        assert!(<LockInfoOf<Runtime>>::get(ALICE, market_id).is_none());
    });
}

#[test]
fn vote_fails_if_outcome_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(0),
            &ALICE,
            10 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(20),
            &ALICE,
            20 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(40),
            &ALICE,
            30 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(60),
            &ALICE,
            40 * BASE,
        );

        assert_noop!(
            GlobalDisputes::vote_on_outcome(
                Origin::signed(ALICE),
                market_id,
                OutcomeReport::Scalar(42),
                50 * BASE
            ),
            Error::<Runtime>::OutcomeDoesNotExist
        );
    });
}

#[test]
fn vote_fails_for_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(0),
            &ALICE,
            10 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(20),
            &ALICE,
            20 * BASE,
        );
        assert_noop!(
            GlobalDisputes::vote_on_outcome(
                Origin::signed(POOR_PAUL),
                market_id,
                OutcomeReport::Scalar(0),
                50 * BASE
            ),
            Error::<Runtime>::InsufficientAmount
        );
    });
}

#[test]
fn locking_works_for_one_market() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(0),
            &ALICE,
            10 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(20),
            &ALICE,
            20 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(40),
            &ALICE,
            30 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id,
            OutcomeReport::Scalar(60),
            &ALICE,
            40 * BASE,
        );

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id), None);
        assert!(Balances::locks(BOB).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, market_id), None);
        assert!(Balances::locks(CHARLIE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, market_id), None);
        assert!(Balances::locks(EVE).is_empty());

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(0),
            50 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            market_id,
            OutcomeReport::Scalar(20),
            40 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(CHARLIE),
            market_id,
            OutcomeReport::Scalar(40),
            30 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(EVE),
            market_id,
            OutcomeReport::Scalar(60),
            20 * BASE
        ));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id), Some(50 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id), Some(40 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, market_id), Some(30 * BASE));
        assert_eq!(Balances::locks(CHARLIE), vec![the_lock(30 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, market_id), Some(20 * BASE));
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert!(GlobalDisputes::get_voting_winner(&market_id).is_some());

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id), Some(50 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id), None);
        assert!(Balances::locks(ALICE).is_empty());

        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id), Some(40 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, market_id), Some(30 * BASE));
        assert_eq!(Balances::locks(CHARLIE), vec![the_lock(30 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, market_id), Some(20 * BASE));
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id), None);
        assert!(Balances::locks(BOB).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, market_id), Some(30 * BASE));
        assert_eq!(Balances::locks(CHARLIE), vec![the_lock(30 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, market_id), Some(20 * BASE));
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(CHARLIE), CHARLIE));
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, market_id), None);
        assert!(Balances::locks(CHARLIE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, market_id), Some(20 * BASE));
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(EVE), EVE));
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, market_id), None);
        assert!(Balances::locks(EVE).is_empty());
    });
}

#[test]
fn locking_works_for_two_markets_with_stronger_first_unlock() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id_1 = 0u128;
        let market_id_2 = 1u128;
        GlobalDisputes::push_voting_outcome(
            &market_id_1,
            OutcomeReport::Scalar(0),
            &ALICE,
            10 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id_1,
            OutcomeReport::Scalar(20),
            &ALICE,
            20 * BASE,
        );

        GlobalDisputes::push_voting_outcome(
            &market_id_2,
            OutcomeReport::Scalar(0),
            &ALICE,
            10 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id_2,
            OutcomeReport::Scalar(20),
            &ALICE,
            20 * BASE,
        );

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), None);
        assert!(Balances::locks(BOB).is_empty());

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id_1,
            OutcomeReport::Scalar(0),
            50 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            market_id_1,
            OutcomeReport::Scalar(20),
            40 * BASE
        ));

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id_2,
            OutcomeReport::Scalar(0),
            30 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            market_id_2,
            OutcomeReport::Scalar(20),
            20 * BASE
        ));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), Some(50 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), Some(30 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), Some(40 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), Some(20 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        // market_id_1 has stronger locks
        assert!(GlobalDisputes::get_voting_winner(&market_id_1).is_some());

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), Some(50 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), Some(30 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), Some(30 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(30 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), Some(40 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), Some(20 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), Some(20 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(20 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), Some(30 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(30 * BASE)]);

        assert!(GlobalDisputes::get_voting_winner(&market_id_2).is_some());

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), Some(30 * BASE));
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), Some(20 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), None);
        assert!(Balances::locks(BOB).is_empty());
    });
}

#[test]
fn locking_works_for_two_markets_with_weaker_first_unlock() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id_1 = 0u128;
        let market_id_2 = 1u128;

        GlobalDisputes::push_voting_outcome(
            &market_id_1,
            OutcomeReport::Scalar(0),
            &ALICE,
            10 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id_1,
            OutcomeReport::Scalar(20),
            &ALICE,
            20 * BASE,
        );

        GlobalDisputes::push_voting_outcome(
            &market_id_2,
            OutcomeReport::Scalar(0),
            &ALICE,
            10 * BASE,
        );
        GlobalDisputes::push_voting_outcome(
            &market_id_2,
            OutcomeReport::Scalar(20),
            &ALICE,
            20 * BASE,
        );

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), None);
        assert!(Balances::locks(BOB).is_empty());

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id_1,
            OutcomeReport::Scalar(0),
            50 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            market_id_1,
            OutcomeReport::Scalar(20),
            40 * BASE
        ));

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id_2,
            OutcomeReport::Scalar(0),
            30 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            market_id_2,
            OutcomeReport::Scalar(20),
            20 * BASE
        ));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), Some(50 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), Some(30 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), Some(40 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), Some(20 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        // market_id_2 has weaker locks
        assert!(GlobalDisputes::get_voting_winner(&market_id_2).is_some());

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), Some(50 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), Some(30 * BASE));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), Some(50 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), None);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), Some(40 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), Some(20 * BASE));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), Some(40 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), None);
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), Some(50 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), None);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);

        assert!(GlobalDisputes::get_voting_winner(&market_id_1).is_some());

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), Some(50 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), None);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, market_id_2), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), Some(40 * BASE));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), None);
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, market_id_2), None);
        assert!(Balances::locks(BOB).is_empty());
    });
}
