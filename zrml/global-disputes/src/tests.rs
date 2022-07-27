#![cfg(test)]

use crate::{
    global_disputes_pallet_api::GlobalDisputesPalletApi,
    mock::{
        Balances, ExtBuilder, GlobalDisputes, Origin, Runtime, ALICE, BOB, CHARLIE, EVE, POOR_PAUL,
    },
    Error, LockInfoOf, OutcomeVotes, Outcomes,
};
use frame_support::{assert_noop, assert_ok, traits::ReservableCurrency};
use pallet_balances::BalanceLock;
use sp_runtime::SaturatedConversion;
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
        let vote_id = GlobalDisputes::get_next_vote_id().unwrap();
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 10 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(20), 20 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(40), 30 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(60), 40 * BASE));

        assert_noop!(
            GlobalDisputes::vote_on_outcome(
                Origin::signed(ALICE),
                vote_id,
                2u128,
                MinOutcomeVoteAmount::get() - 1,
            ),
            Error::<Runtime>::AmountTooLow
        );
    });
}

#[test]
fn get_voting_winner_sets_the_last_outcome_for_same_vote_balances_as_the_canonical_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        let vote_id = GlobalDisputes::get_next_vote_id().unwrap();
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 10 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(20), 10 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(40), 10 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(60), 10 * BASE));

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            vote_id,
            0u128,
            42 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(Origin::signed(BOB), vote_id, 1u128, 42 * BASE));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(CHARLIE),
            vote_id,
            2u128,
            42 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(Origin::signed(EVE), vote_id, 3u128, 42 * BASE));
        assert_eq!(
            &GlobalDisputes::get_voting_winner(vote_id).unwrap(),
            &OutcomeReport::Scalar(60)
        );
    });
}

#[test]
fn reserve_before_init_vote_outcome_is_not_allowed_for_voting() {
    ExtBuilder::default().build().execute_with(|| {
        let vote_id = GlobalDisputes::get_next_vote_id().unwrap();

        let disputor = &ALICE;
        let free_balance_disputor_before = Balances::free_balance(disputor);
        let arbitrary_amount = 42 * BASE;
        let reserved_balance_disputor = free_balance_disputor_before - arbitrary_amount;

        assert_ok!(Balances::reserve(disputor, reserved_balance_disputor));
        assert_eq!(
            Balances::free_balance(disputor),
            free_balance_disputor_before - reserved_balance_disputor
        );

        assert_ok!(GlobalDisputes::push_voting_outcome(
            OutcomeReport::Scalar(0),
            reserved_balance_disputor
        ));

        assert_ok!(GlobalDisputes::push_voting_outcome(
            OutcomeReport::Scalar(20),
            reserved_balance_disputor * 2
        ));

        assert_noop!(
            GlobalDisputes::vote_on_outcome(
                Origin::signed(*disputor),
                vote_id,
                0u128,
                arbitrary_amount + 1
            ),
            Error::<Runtime>::InsufficientAmount
        );

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(*disputor),
            vote_id,
            0u128,
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
        let vote_id = GlobalDisputes::get_next_vote_id().unwrap();

        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 10 * BASE));

        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(20), 20 * BASE));

        let disputor = &ALICE;
        let free_balance_disputor_before = Balances::free_balance(disputor);
        let arbitrary_amount = 42 * BASE;

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(*disputor),
            vote_id,
            0u128,
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
        let vote_id = GlobalDisputes::get_next_vote_id().unwrap();

        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 10 * BASE));

        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(20), 20 * BASE));

        let disputor = &ALICE;
        let free_balance_disputor_before = Balances::free_balance(disputor);
        let arbitrary_amount = 42 * BASE;

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(*disputor),
            vote_id,
            0u128,
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
        let reinitialize_outcomes = || {
            GlobalDisputes::get_next_vote_id().unwrap();

            assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 100 * BASE));

            assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(20), 100 * BASE));
            assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(40), 100 * BASE));

            assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(60), 100 * BASE));
        };

        reinitialize_outcomes();
        let latest_vote_id = GlobalDisputes::get_latest_vote_id();
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            latest_vote_id,
            0u128,
            10 * BASE
        ));

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            latest_vote_id,
            1u128,
            10 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(CHARLIE),
            latest_vote_id,
            2u128,
            11 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(EVE),
            latest_vote_id,
            3u128,
            10 * BASE
        ));

        assert_eq!(<OutcomeVotes<Runtime>>::get(latest_vote_id, 0u128).unwrap(), 110 * BASE);
        assert_eq!(<OutcomeVotes<Runtime>>::get(latest_vote_id, 1u128).unwrap(), 110 * BASE);
        assert_eq!(<OutcomeVotes<Runtime>>::get(latest_vote_id, 2u128).unwrap(), 111 * BASE);
        assert_eq!(<OutcomeVotes<Runtime>>::get(latest_vote_id, 3u128).unwrap(), 110 * BASE);

        assert_eq!(
            GlobalDisputes::get_voting_winner(latest_vote_id).unwrap(),
            OutcomeReport::Scalar(40)
        );
        
        assert_eq!(<Outcomes<Runtime>>::get(latest_vote_id), vec![]);

        reinitialize_outcomes();
        let latest_vote_id = GlobalDisputes::get_latest_vote_id();
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            latest_vote_id,
            3u128,
            10 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            latest_vote_id,
            1u128,
            50 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(CHARLIE),
            latest_vote_id,
            3u128,
            20 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(EVE),
            latest_vote_id,
            3u128,
            21 * BASE
        ));

        assert_eq!(<OutcomeVotes<Runtime>>::get(latest_vote_id, 0u128).unwrap(), 100 * BASE);
        assert_eq!(<OutcomeVotes<Runtime>>::get(latest_vote_id, 1u128).unwrap(), 150 * BASE);
        assert_eq!(<OutcomeVotes<Runtime>>::get(latest_vote_id, 2u128).unwrap(), 100 * BASE);
        assert_eq!(<OutcomeVotes<Runtime>>::get(latest_vote_id, 3u128).unwrap(), 151 * BASE);

        assert_eq!(
            GlobalDisputes::get_voting_winner(latest_vote_id).unwrap(),
            OutcomeReport::Scalar(60)
        );

        reinitialize_outcomes();
        let latest_vote_id = GlobalDisputes::get_latest_vote_id();
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            latest_vote_id,
            1u128,
            30 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            latest_vote_id,
            2u128,
            50 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(CHARLIE),
            latest_vote_id,
            0u128,
            10 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(EVE),
            latest_vote_id,
            0u128,
            41 * BASE
        ));

        assert_eq!(
            GlobalDisputes::get_voting_winner(latest_vote_id).unwrap(),
            OutcomeReport::Scalar(0)
        );

        reinitialize_outcomes();
        let latest_vote_id = GlobalDisputes::get_latest_vote_id();
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            latest_vote_id,
            1u128,
            1 * BASE
        ));
        assert_eq!(<OutcomeVotes<Runtime>>::get(latest_vote_id, 1u128).unwrap(), 101 * BASE);

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(CHARLIE),
            latest_vote_id,
            0u128,
            10 * BASE
        ));
        assert_eq!(<OutcomeVotes<Runtime>>::get(latest_vote_id, 0u128).unwrap(), 110 * BASE);

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            latest_vote_id,
            1u128,
            10 * BASE
        ));
        assert_eq!(<OutcomeVotes<Runtime>>::get(latest_vote_id, 1u128).unwrap(), 110 * BASE);

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(EVE),
            latest_vote_id,
            0u128,
            40 * BASE
        ));
        // Eve and Charlie have more together
        assert_eq!(<OutcomeVotes<Runtime>>::get(latest_vote_id, 0u128).unwrap(), 150 * BASE);

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            latest_vote_id,
            1u128,
            40 * BASE
        ));
        // Alice updates here voting balance (instead of accumulating)
        assert_eq!(<OutcomeVotes<Runtime>>::get(latest_vote_id, 1u128).unwrap(), 140 * BASE);

        assert_eq!(
            GlobalDisputes::get_voting_winner(latest_vote_id).unwrap(),
            OutcomeReport::Scalar(0)
        );
    });
}

#[test]
fn get_voting_winner_clears_outcome_votes() {
    ExtBuilder::default().build().execute_with(|| {
        let vote_id = GlobalDisputes::get_next_vote_id().unwrap();
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 0 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(20), 0 * BASE));

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            vote_id,
            0u128,
            10 * BASE
        ));

        assert_ok!(GlobalDisputes::vote_on_outcome(Origin::signed(BOB), vote_id, 1u128, 10 * BASE));

        assert_eq!(<OutcomeVotes<Runtime>>::get(vote_id, 0u128), Some(10 * BASE));
        assert_eq!(<OutcomeVotes<Runtime>>::get(vote_id, 1u128), Some(10 * BASE));

        assert_ok!(GlobalDisputes::get_voting_winner(vote_id));

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        // TODO if nobody would have voted on 1u128, then the OutcomeVotes would never be deleted
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));

        assert_eq!(<OutcomeVotes<Runtime>>::iter_keys().next(), None);
    });
}

#[test]
fn unlock_clears_lock_info() {
    ExtBuilder::default().build().execute_with(|| {
        let vote_id = GlobalDisputes::get_next_vote_id().unwrap();

        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 10 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(20), 20 * BASE));

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            vote_id,
            0u128,
            50 * BASE
        ));

        assert_ok!(GlobalDisputes::get_voting_winner(vote_id));

        assert!(<LockInfoOf<Runtime>>::get(ALICE, vote_id).is_some());

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));

        assert!(<LockInfoOf<Runtime>>::get(ALICE, vote_id).is_none());
    });
}

#[test]
fn vote_fails_if_outcome_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        let vote_id = GlobalDisputes::get_next_vote_id().unwrap();
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 10 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(20), 20 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(40), 30 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(60), 40 * BASE));

        assert_noop!(
            GlobalDisputes::vote_on_outcome(Origin::signed(ALICE), vote_id, 42u128, 50 * BASE),
            Error::<Runtime>::OutcomeDoesNotExist
        );
    });
}

#[test]
fn vote_fails_for_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let vote_id = GlobalDisputes::get_next_vote_id().unwrap();
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 10 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(20), 20 * BASE));
        assert_noop!(
            GlobalDisputes::vote_on_outcome(Origin::signed(POOR_PAUL), vote_id, 0u128, 50 * BASE),
            Error::<Runtime>::InsufficientAmount
        );
    });
}

#[test]
fn vote_fails_if_outcome_len_below_min_outcomes() {
    ExtBuilder::default().build().execute_with(|| {
        let vote_id = GlobalDisputes::get_next_vote_id().unwrap();

        let mut i: u128 = 1;
        while i < crate::mock::MinOutcomes::get().saturated_into::<u128>() {
            assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(i), 42 * BASE));

            assert_noop!(
                GlobalDisputes::vote_on_outcome(Origin::signed(ALICE), vote_id, 0u128, 50 * BASE),
                Error::<Runtime>::NotEnoughOutcomes
            );

            i += 1;
        }

        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 42 * BASE));

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            vote_id,
            0u128,
            50 * BASE
        ));
    });
}

#[test]
fn locking_works_for_one_market() {
    ExtBuilder::default().build().execute_with(|| {
        let vote_id = GlobalDisputes::get_next_vote_id().unwrap();
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 10 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(20), 20 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(40), 30 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(60), 40 * BASE));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id), None);
        assert!(Balances::locks(BOB).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, vote_id), None);
        assert!(Balances::locks(CHARLIE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, vote_id), None);
        assert!(Balances::locks(EVE).is_empty());

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            vote_id,
            0u128,
            50 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(Origin::signed(BOB), vote_id, 1u128, 40 * BASE));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(CHARLIE),
            vote_id,
            2u128,
            30 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(Origin::signed(EVE), vote_id, 3u128, 20 * BASE));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id), Some((0u128, 50 * BASE)));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id), Some((1u128, 40 * BASE)));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, vote_id), Some((2u128, 30 * BASE)));
        assert_eq!(Balances::locks(CHARLIE), vec![the_lock(30 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, vote_id), Some((3u128, 20 * BASE)));
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::get_voting_winner(vote_id));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id), Some((0u128, 50 * BASE)));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id), None);
        assert!(Balances::locks(ALICE).is_empty());

        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id), Some((1u128, 40 * BASE)));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, vote_id), Some((2u128, 30 * BASE)));
        assert_eq!(Balances::locks(CHARLIE), vec![the_lock(30 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, vote_id), Some((3u128, 20 * BASE)));
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id), None);
        assert!(Balances::locks(BOB).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, vote_id), Some((2u128, 30 * BASE)));
        assert_eq!(Balances::locks(CHARLIE), vec![the_lock(30 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, vote_id), Some((3u128, 20 * BASE)));
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(CHARLIE), CHARLIE));
        assert_eq!(LockInfoOf::<Runtime>::get(CHARLIE, vote_id), None);
        assert!(Balances::locks(CHARLIE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, vote_id), Some((3u128, 20 * BASE)));
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(EVE), EVE));
        assert_eq!(LockInfoOf::<Runtime>::get(EVE, vote_id), None);
        assert!(Balances::locks(EVE).is_empty());
    });
}

#[test]
fn locking_works_for_two_markets_with_stronger_first_unlock() {
    ExtBuilder::default().build().execute_with(|| {
        let vote_id_1 = GlobalDisputes::get_next_vote_id().unwrap();
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 10 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(20), 20 * BASE));

        let vote_id_2 = GlobalDisputes::get_next_vote_id().unwrap();
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 10 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(20), 20 * BASE));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_2), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_2), None);
        assert!(Balances::locks(BOB).is_empty());

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            vote_id_1,
            0u128,
            50 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            vote_id_1,
            1u128,
            40 * BASE
        ));

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            vote_id_2,
            0u128,
            30 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            vote_id_2,
            1u128,
            20 * BASE
        ));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_1), Some((0u128, 50 * BASE)));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_2), Some((0u128, 30 * BASE)));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_1), Some((1u128, 40 * BASE)));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_2), Some((1u128, 20 * BASE)));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        // vote_id_1 has stronger locks
        assert_ok!(GlobalDisputes::get_voting_winner(vote_id_1));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_1), Some((0u128, 50 * BASE)));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_2), Some((0u128, 30 * BASE)));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_2), Some((0u128, 30 * BASE)));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(30 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_1), Some((1u128, 40 * BASE)));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_2), Some((1u128, 20 * BASE)));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_2), Some((1u128, 20 * BASE)));
        assert_eq!(Balances::locks(BOB), vec![the_lock(20 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_2), Some((0u128, 30 * BASE)));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(30 * BASE)]);

        assert_ok!(GlobalDisputes::get_voting_winner(vote_id_2));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_2), Some((0u128, 30 * BASE)));
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_2), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_2), Some((1u128, 20 * BASE)));
        assert_eq!(Balances::locks(BOB), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_2), None);
        assert!(Balances::locks(BOB).is_empty());
    });
}

#[test]
fn locking_works_for_two_markets_with_weaker_first_unlock() {
    ExtBuilder::default().build().execute_with(|| {
        let vote_id_1 = GlobalDisputes::get_next_vote_id().unwrap();
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 10 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(20), 20 * BASE));

        let vote_id_2 = GlobalDisputes::get_next_vote_id().unwrap();
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(0), 10 * BASE));
        assert_ok!(GlobalDisputes::push_voting_outcome(OutcomeReport::Scalar(20), 20 * BASE));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_2), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_2), None);
        assert!(Balances::locks(BOB).is_empty());

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            vote_id_1,
            0u128,
            50 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            vote_id_1,
            1u128,
            40 * BASE
        ));

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            vote_id_2,
            0u128,
            30 * BASE
        ));
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(BOB),
            vote_id_2,
            1u128,
            20 * BASE
        ));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_1), Some((0u128, 50 * BASE)));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_2), Some((0u128, 30 * BASE)));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_1), Some((1u128, 40 * BASE)));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_2), Some((1u128, 20 * BASE)));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        // vote_id_2 has weaker locks
        assert_ok!(GlobalDisputes::get_voting_winner(vote_id_2));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_1), Some((0u128, 50 * BASE)));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_2), Some((0u128, 30 * BASE)));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_1), Some((0u128, 50 * BASE)));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_2), None);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_1), Some((1u128, 40 * BASE)));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_2), Some((1u128, 20 * BASE)));
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_1), Some((1u128, 40 * BASE)));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_2), None);
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_1), Some((0u128, 50 * BASE)));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_2), None);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);

        assert_ok!(GlobalDisputes::get_voting_winner(vote_id_1));

        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_1), Some((0u128, 50 * BASE)));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_2), None);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(ALICE, vote_id_2), None);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_1), Some((1u128, 40 * BASE)));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_2), None);
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_1), None);
        assert_eq!(LockInfoOf::<Runtime>::get(BOB, vote_id_2), None);
        assert!(Balances::locks(BOB).is_empty());
    });
}
