// Copyright 2022-2023 Forecasting Technologies LTD.
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

#![cfg(test)]

use crate::{
    global_disputes_pallet_api::GlobalDisputesPalletApi,
    mock::*,
    types::{GdStatus, GlobalDisputeInfo, InitialItem, OutcomeInfo, Possession},
    utils::market_mock,
    BalanceOf, Error, Event, GlobalDisputesInfo, InitialItemOf, Locks, MarketIdOf, Outcomes,
};
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, ReservableCurrency},
    BoundedVec,
};
use pallet_balances::{BalanceLock, Error as BalancesError};
use sp_runtime::traits::Zero;
use test_case::test_case;
use zeitgeist_primitives::{
    constants::mock::{
        GlobalDisputeLockId, MinOutcomeVoteAmount, RemoveKeysLimit, VotingOutcomeFee, BASE,
    },
    types::{BlockNumber, OutcomeReport},
};
use zrml_market_commons::{Error as MarketError, Markets};

const SETUP_AMOUNT: u128 = 100 * BASE;

fn the_lock(amount: u128) -> BalanceLock<u128> {
    BalanceLock { id: GlobalDisputeLockId::get(), amount, reasons: pallet_balances::Reasons::Misc }
}

fn get_initial_items() -> Vec<InitialItemOf<Runtime>> {
    vec![
        InitialItem { outcome: OutcomeReport::Scalar(0), owner: ALICE, amount: SETUP_AMOUNT },
        InitialItem { outcome: OutcomeReport::Scalar(20), owner: ALICE, amount: SETUP_AMOUNT },
        InitialItem { outcome: OutcomeReport::Scalar(40), owner: ALICE, amount: SETUP_AMOUNT },
        InitialItem { outcome: OutcomeReport::Scalar(60), owner: ALICE, amount: SETUP_AMOUNT },
    ]
}

fn set_vote_period() {
    let now = <frame_system::Pallet<Runtime>>::block_number();
    <frame_system::Pallet<Runtime>>::set_block_number(
        now + <Runtime as crate::Config>::AddOutcomePeriod::get() + 1,
    );
}

fn check_outcome_sum(
    market_id: &MarketIdOf<Runtime>,
    outcome: OutcomeReport,
    post_setup_amount: u128,
) {
    assert_eq!(
        <Outcomes<Runtime>>::get(market_id, outcome).unwrap(),
        OutcomeInfo {
            outcome_sum: SETUP_AMOUNT + post_setup_amount,
            possession: Possession::Shared { owners: BoundedVec::try_from(vec![ALICE]).unwrap() }
        }
    );
}

#[test]
fn add_vote_outcome_works() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        let free_balance_alice_before = Balances::free_balance(&ALICE);
        let free_balance_reward_account =
            Balances::free_balance(GlobalDisputes::reward_account(&market_id));
        assert_ok!(GlobalDisputes::add_vote_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(80),
        ));
        System::assert_last_event(
            Event::<Runtime>::AddedVotingOutcome {
                market_id,
                owner: ALICE,
                outcome: OutcomeReport::Scalar(80),
            }
            .into(),
        );
        assert_eq!(
            Balances::free_balance(&ALICE),
            free_balance_alice_before - VotingOutcomeFee::get()
        );
        assert_eq!(
            Balances::free_balance(GlobalDisputes::reward_account(&market_id)),
            free_balance_reward_account + VotingOutcomeFee::get()
        );
    });
}

#[test_case(GdStatus::Finished; "finished")]
#[test_case(GdStatus::Destroyed; "destroyed")]
fn is_active_works(status: GdStatus<BlockNumber>) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        assert!(!GlobalDisputes::is_active(&market_id));

        let outcome_info = OutcomeInfo {
            outcome_sum: 0,
            possession: Possession::Shared { owners: BoundedVec::try_from(vec![ALICE]).unwrap() },
        };
        <GlobalDisputesInfo<Runtime>>::insert(
            market_id,
            GlobalDisputeInfo {
                winner_outcome: OutcomeReport::Scalar(0),
                outcome_info: outcome_info.clone(),
                status,
            },
        );

        assert!(!GlobalDisputes::is_active(&market_id));

        <GlobalDisputesInfo<Runtime>>::insert(
            market_id,
            GlobalDisputeInfo {
                winner_outcome: OutcomeReport::Scalar(0),
                outcome_info,
                status: GdStatus::Active { add_outcome_end: 0, vote_end: 0 },
            },
        );

        assert!(GlobalDisputes::is_active(&market_id));
    });
}

#[test]
fn destroy_global_dispute_works() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        assert_ok!(GlobalDisputes::destroy_global_dispute(&market_id));

        assert_eq!(
            <GlobalDisputesInfo<Runtime>>::get(market_id).unwrap().status,
            GdStatus::Destroyed
        );
    });
}

#[test]
fn start_global_dispute_works() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        let outcome_info = OutcomeInfo {
            outcome_sum: SETUP_AMOUNT,
            possession: Possession::Shared { owners: BoundedVec::try_from(vec![ALICE]).unwrap() },
        };
        assert_eq!(
            <GlobalDisputesInfo<Runtime>>::get(market_id).unwrap(),
            GlobalDisputeInfo {
                winner_outcome: OutcomeReport::Scalar(60),
                outcome_info,
                status: GdStatus::Active { add_outcome_end: 21, vote_end: 161 },
            }
        );
    });
}

#[test]
fn start_global_dispute_fails_if_outcome_mismatch() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = vec![
            InitialItem { outcome: OutcomeReport::Scalar(0), owner: ALICE, amount: SETUP_AMOUNT },
            InitialItem { outcome: OutcomeReport::Scalar(20), owner: ALICE, amount: SETUP_AMOUNT },
            // categorical outcome mismatch
            InitialItem {
                outcome: OutcomeReport::Categorical(40),
                owner: ALICE,
                amount: SETUP_AMOUNT,
            },
            InitialItem { outcome: OutcomeReport::Scalar(60), owner: ALICE, amount: SETUP_AMOUNT },
        ];
        assert_eq!(
            GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()),
            Err(Error::<Runtime>::OutcomeMismatch.into())
        );
    });
}

#[test]
fn start_global_dispute_fails_if_less_than_two_outcomes() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = vec![InitialItem {
            outcome: OutcomeReport::Scalar(0),
            owner: ALICE,
            amount: SETUP_AMOUNT,
        }];
        assert_eq!(
            GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()),
            Err(Error::<Runtime>::AtLeastTwoUniqueOutcomesRequired.into())
        );
    });
}

#[test]
fn start_global_dispute_fails_if_already_exists() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()).unwrap();
        assert_eq!(
            GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()),
            Err(Error::<Runtime>::GlobalDisputeAlreadyExists.into())
        );
    });
}

#[test]
fn start_global_dispute_fails_if_max_owner_reached() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let mut initial_items = Vec::new();
        for i in 0..MaxOwners::get() + 1 {
            initial_items.push(InitialItem {
                outcome: OutcomeReport::Scalar(0),
                owner: i.into(),
                amount: SETUP_AMOUNT,
            });
        }

        assert_eq!(
            GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()),
            Err(Error::<Runtime>::MaxOwnersReached.into())
        );
    });
}

#[test]
fn add_vote_outcome_fails_with_outcome_mismatch() {
    ExtBuilder::default().build().execute_with(|| {
        // create scalar market
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        assert_noop!(
            GlobalDisputes::add_vote_outcome(
                Origin::signed(ALICE),
                market_id,
                OutcomeReport::Categorical(0u16),
            ),
            Error::<Runtime>::OutcomeMismatch
        );
    });
}

#[test]
fn add_vote_outcome_fails_with_non_existing_market() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        assert_noop!(
            GlobalDisputes::add_vote_outcome(
                Origin::signed(ALICE),
                market_id,
                OutcomeReport::Scalar(80),
            ),
            MarketError::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn add_vote_outcome_fails_if_no_global_dispute_present() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);
        assert_noop!(
            GlobalDisputes::add_vote_outcome(
                Origin::signed(ALICE),
                market_id,
                OutcomeReport::Scalar(20),
            ),
            Error::<Runtime>::GlobalDisputeNotFound
        );
    });
}

#[test_case(GdStatus::Finished; "finished")]
#[test_case(GdStatus::Destroyed; "destroyed")]
fn add_vote_outcome_fails_if_global_dispute_is_in_wrong_state(status: GdStatus<BlockNumber>) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);
        let possession = Possession::Shared { owners: BoundedVec::try_from(vec![ALICE]).unwrap() };
        let mut gd_info = GlobalDisputeInfo::new(OutcomeReport::Scalar(0), possession, 10 * BASE);
        gd_info.status = status;
        <GlobalDisputesInfo<Runtime>>::insert(market_id, gd_info);

        assert_noop!(
            GlobalDisputes::add_vote_outcome(
                Origin::signed(ALICE),
                market_id,
                OutcomeReport::Scalar(20),
            ),
            Error::<Runtime>::InvalidGlobalDisputeStatus
        );
    });
}

#[test]
fn add_vote_outcome_fails_if_outcome_already_exists() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));
        <Outcomes<Runtime>>::insert(
            market_id,
            OutcomeReport::Scalar(20),
            OutcomeInfo {
                outcome_sum: Zero::zero(),
                possession: Possession::Shared { owners: Default::default() },
            },
        );
        assert_noop!(
            GlobalDisputes::add_vote_outcome(
                Origin::signed(ALICE),
                market_id,
                OutcomeReport::Scalar(20),
            ),
            Error::<Runtime>::OutcomeAlreadyExists
        );
    });
}

#[test]
fn add_vote_outcome_fails_if_balance_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        assert_noop!(
            GlobalDisputes::add_vote_outcome(
                Origin::signed(POOR_PAUL),
                market_id,
                OutcomeReport::Scalar(80),
            ),
            BalancesError::<Runtime>::InsufficientBalance
        );
    });
}

#[test]
fn reward_outcome_owner_works_for_multiple_owners() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        <Outcomes<Runtime>>::insert(
            market_id,
            OutcomeReport::Scalar(20),
            OutcomeInfo {
                outcome_sum: Zero::zero(),
                possession: Possession::Shared {
                    owners: BoundedVec::try_from(vec![ALICE, BOB, CHARLIE]).unwrap(),
                },
            },
        );
        let _ = Balances::deposit_creating(
            &GlobalDisputes::reward_account(&market_id),
            3 * VotingOutcomeFee::get(),
        );
        let gd_info = GlobalDisputeInfo {
            winner_outcome: OutcomeReport::Scalar(20),
            status: GdStatus::Finished,
            outcome_info: OutcomeInfo {
                outcome_sum: 10 * BASE,
                possession: Possession::Shared { owners: Default::default() },
            },
        };
        <GlobalDisputesInfo<Runtime>>::insert(market_id, gd_info);

        let free_balance_alice_before = Balances::free_balance(&ALICE);
        let free_balance_bob_before = Balances::free_balance(&BOB);
        let free_balance_charlie_before = Balances::free_balance(&CHARLIE);

        assert_ok!(GlobalDisputes::purge_outcomes(Origin::signed(ALICE), market_id,));

        System::assert_last_event(Event::<Runtime>::OutcomesFullyCleaned { market_id }.into());

        assert_ok!(GlobalDisputes::reward_outcome_owner(Origin::signed(ALICE), market_id,));

        System::assert_last_event(
            Event::<Runtime>::OutcomeOwnersRewarded {
                market_id,
                owners: vec![ALICE, BOB, CHARLIE],
            }
            .into(),
        );
        assert_eq!(
            Balances::free_balance(&ALICE),
            free_balance_alice_before + VotingOutcomeFee::get()
        );
        assert_eq!(Balances::free_balance(&BOB), free_balance_bob_before + VotingOutcomeFee::get());
        assert_eq!(
            Balances::free_balance(&CHARLIE),
            free_balance_charlie_before + VotingOutcomeFee::get()
        );
        assert!(Balances::free_balance(GlobalDisputes::reward_account(&market_id)).is_zero());
        assert!(<Outcomes<Runtime>>::iter_prefix(market_id).next().is_none());
    });
}

#[test]
fn reward_outcome_owner_has_dust() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        <Outcomes<Runtime>>::insert(
            market_id,
            OutcomeReport::Scalar(20),
            OutcomeInfo {
                outcome_sum: Zero::zero(),
                possession: Possession::Shared {
                    owners: BoundedVec::try_from(vec![ALICE, BOB, CHARLIE, EVE, POOR_PAUL, DAVE])
                        .unwrap(),
                },
            },
        );
        let _ = Balances::deposit_creating(&GlobalDisputes::reward_account(&market_id), 100 * BASE);
        let gd_info = GlobalDisputeInfo {
            winner_outcome: OutcomeReport::Scalar(20),
            status: GdStatus::Finished,
            outcome_info: OutcomeInfo {
                outcome_sum: 10 * BASE,
                possession: Possession::Shared { owners: Default::default() },
            },
        };
        <GlobalDisputesInfo<Runtime>>::insert(market_id, gd_info);

        assert_ok!(GlobalDisputes::purge_outcomes(Origin::signed(ALICE), market_id,));

        assert_ok!(GlobalDisputes::reward_outcome_owner(Origin::signed(ALICE), market_id,));

        // 100 * BASE = 1_000_000_000_000 checked_div 6 = 166_666_666_666
        // 166_666_666_666 * 6 = 999_999_999_996 so 4 left
        assert_eq!(Balances::free_balance(GlobalDisputes::reward_account(&market_id)), 4);
    });
}

#[test]
fn reward_outcome_owner_works_for_one_owner() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        <Outcomes<Runtime>>::insert(
            market_id,
            OutcomeReport::Scalar(20),
            OutcomeInfo {
                outcome_sum: Zero::zero(),
                possession: Possession::Shared {
                    owners: BoundedVec::try_from(vec![ALICE]).unwrap(),
                },
            },
        );
        let _ = Balances::deposit_creating(
            &GlobalDisputes::reward_account(&market_id),
            3 * VotingOutcomeFee::get(),
        );
        let gd_info = GlobalDisputeInfo {
            winner_outcome: OutcomeReport::Scalar(20),
            status: GdStatus::Finished,
            outcome_info: OutcomeInfo {
                outcome_sum: 10 * BASE,
                possession: Possession::Shared { owners: BoundedVec::try_from(vec![]).unwrap() },
            },
        };
        <GlobalDisputesInfo<Runtime>>::insert(market_id, gd_info);

        assert_ok!(GlobalDisputes::purge_outcomes(Origin::signed(ALICE), market_id,));

        System::assert_last_event(Event::<Runtime>::OutcomesFullyCleaned { market_id }.into());

        let free_balance_alice_before = Balances::free_balance(&ALICE);

        assert_ok!(GlobalDisputes::reward_outcome_owner(Origin::signed(ALICE), market_id));

        System::assert_last_event(
            Event::<Runtime>::OutcomeOwnersRewarded { market_id, owners: vec![ALICE] }.into(),
        );

        assert_eq!(
            Balances::free_balance(&ALICE),
            free_balance_alice_before + 3 * VotingOutcomeFee::get()
        );
        assert!(Balances::free_balance(GlobalDisputes::reward_account(&market_id)).is_zero());
        assert!(<Outcomes<Runtime>>::iter_prefix(market_id).next().is_none());
    });
}

#[test]
fn vote_fails_if_amount_below_min_outcome_vote_amount() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

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
fn vote_fails_for_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        // Paul does not have 50 * BASE
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
fn determine_voting_winner_sets_the_last_outcome_for_same_vote_balances_as_the_canonical_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        set_vote_period();

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
            &GlobalDisputes::determine_voting_winner(&market_id).unwrap(),
            &OutcomeReport::Scalar(60)
        );
        System::assert_last_event(
            Event::<Runtime>::GlobalDisputeWinnerDetermined { market_id }.into(),
        );
    });
}

#[test]
fn vote_on_outcome_check_event() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        set_vote_period();

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(EVE),
            market_id,
            OutcomeReport::Scalar(60),
            42 * BASE
        ));

        System::assert_last_event(
            Event::<Runtime>::VotedOnOutcome {
                voter: EVE,
                market_id,
                outcome: OutcomeReport::Scalar(60),
                vote_amount: 42 * BASE,
            }
            .into(),
        );
    });
}

#[test]
fn reserve_before_init_vote_outcome_is_not_allowed_for_voting() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let disputor = &ALICE;
        let free_balance_disputor_before = Balances::free_balance(disputor);
        let arbitrary_amount = 42 * BASE;
        let reserved_balance_disputor = free_balance_disputor_before - arbitrary_amount;

        assert_ok!(Balances::reserve(disputor, reserved_balance_disputor));
        assert_eq!(
            Balances::free_balance(disputor),
            free_balance_disputor_before - reserved_balance_disputor
        );

        let initial_items = vec![
            InitialItem {
                outcome: OutcomeReport::Scalar(0),
                owner: ALICE,
                amount: reserved_balance_disputor,
            },
            InitialItem {
                outcome: OutcomeReport::Scalar(20),
                owner: ALICE,
                amount: reserved_balance_disputor * 2,
            },
        ];

        GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()).unwrap();

        set_vote_period();

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
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        set_vote_period();

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
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        set_vote_period();

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
fn determine_voting_winner_works_four_outcome_votes() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        set_vote_period();

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

        check_outcome_sum(&market_id, OutcomeReport::Scalar(0), 10 * BASE);
        check_outcome_sum(&market_id, OutcomeReport::Scalar(20), 10 * BASE);
        check_outcome_sum(&market_id, OutcomeReport::Scalar(40), 11 * BASE);
        check_outcome_sum(&market_id, OutcomeReport::Scalar(60), 10 * BASE);

        assert_eq!(
            GlobalDisputes::determine_voting_winner(&market_id).unwrap(),
            OutcomeReport::Scalar(40)
        );

        assert_eq!(
            <GlobalDisputesInfo<Runtime>>::get(market_id).unwrap().status,
            GdStatus::Finished
        );
    });
}

#[test]
fn determine_voting_winner_works_three_outcome_votes() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        set_vote_period();

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

        check_outcome_sum(&market_id, OutcomeReport::Scalar(0), 51 * BASE);
        check_outcome_sum(&market_id, OutcomeReport::Scalar(20), 30 * BASE);
        check_outcome_sum(&market_id, OutcomeReport::Scalar(40), 50 * BASE);
        check_outcome_sum(&market_id, OutcomeReport::Scalar(60), 0);

        assert_eq!(
            GlobalDisputes::determine_voting_winner(&market_id).unwrap(),
            OutcomeReport::Scalar(0)
        );
    });
}

#[test]
fn determine_voting_winner_works_two_outcome_votes() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        set_vote_period();

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

        check_outcome_sum(&market_id, OutcomeReport::Scalar(0), 0);
        check_outcome_sum(&market_id, OutcomeReport::Scalar(20), 50 * BASE);
        check_outcome_sum(&market_id, OutcomeReport::Scalar(40), 0);
        check_outcome_sum(&market_id, OutcomeReport::Scalar(60), 51 * BASE);

        assert_eq!(
            GlobalDisputes::determine_voting_winner(&market_id).unwrap(),
            OutcomeReport::Scalar(60)
        );
    });
}

#[test]
fn determine_voting_winner_works_with_accumulated_votes_for_alice() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        set_vote_period();

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(20),
            BASE
        ));
        check_outcome_sum(&market_id, OutcomeReport::Scalar(20), BASE);

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(CHARLIE),
            market_id,
            OutcomeReport::Scalar(0),
            10 * BASE
        ));
        check_outcome_sum(&market_id, OutcomeReport::Scalar(0), 10 * BASE);

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(20),
            10 * BASE
        ));
        check_outcome_sum(&market_id, OutcomeReport::Scalar(20), 11 * BASE);

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(EVE),
            market_id,
            OutcomeReport::Scalar(0),
            40 * BASE
        ));
        // Eve and Charlie have more together currently
        check_outcome_sum(&market_id, OutcomeReport::Scalar(0), 50 * BASE);

        // Now Alice wins again
        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(20),
            40 * BASE
        ));
        // votes accumulate
        check_outcome_sum(&market_id, OutcomeReport::Scalar(20), 51 * BASE);

        // 51 BASE for outcome 20 against 50 BASE for outcome 0
        assert_eq!(
            GlobalDisputes::determine_voting_winner(&market_id).unwrap(),
            OutcomeReport::Scalar(20)
        );
    });
}

#[test]
fn purge_outcomes_fully_cleaned_works() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        set_vote_period();

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

        check_outcome_sum(&market_id, OutcomeReport::Scalar(0), 10 * BASE);

        check_outcome_sum(&market_id, OutcomeReport::Scalar(20), 10 * BASE);

        assert!(GlobalDisputes::determine_voting_winner(&market_id).is_some());

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));

        assert_ok!(GlobalDisputes::purge_outcomes(Origin::signed(ALICE), market_id,));

        System::assert_last_event(Event::<Runtime>::OutcomesFullyCleaned { market_id }.into());

        assert_eq!(<Outcomes<Runtime>>::iter_prefix(market_id).next(), None);
    });
}

#[test]
fn purge_outcomes_partially_cleaned_works() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let mut initial_items = Vec::new();
        for i in 0..(2 * RemoveKeysLimit::get()) {
            initial_items.push(InitialItem {
                owner: ALICE,
                outcome: OutcomeReport::Scalar(i.into()),
                amount: SETUP_AMOUNT,
            });
        }

        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        assert!(GlobalDisputes::determine_voting_winner(&market_id).is_some());

        assert_ok!(GlobalDisputes::purge_outcomes(Origin::signed(ALICE), market_id,));

        System::assert_last_event(Event::<Runtime>::OutcomesPartiallyCleaned { market_id }.into());

        assert!(<Outcomes<Runtime>>::iter_prefix(market_id).next().is_some());

        assert_ok!(GlobalDisputes::purge_outcomes(Origin::signed(ALICE), market_id,));

        System::assert_last_event(Event::<Runtime>::OutcomesFullyCleaned { market_id }.into());

        assert_eq!(<Outcomes<Runtime>>::iter_prefix(market_id).next(), None);
    });
}

#[test]
fn refund_vote_fees_works() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let pushed_outcome_1 = 0;
        let pushed_outcome_2 = 20;

        let initial_items = vec![
            InitialItem {
                owner: ALICE,
                outcome: OutcomeReport::Scalar(pushed_outcome_1),
                amount: SETUP_AMOUNT,
            },
            InitialItem {
                owner: ALICE,
                outcome: OutcomeReport::Scalar(pushed_outcome_2),
                amount: SETUP_AMOUNT,
            },
        ];

        let offset = pushed_outcome_1.max(pushed_outcome_2) + 1;

        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        let mut overall_fees = <BalanceOf<Runtime>>::zero();
        // minus 2 because of the above push_vote_outcome calls
        for i in 0..(2 * RemoveKeysLimit::get() - 2) {
            assert_ok!(GlobalDisputes::add_vote_outcome(
                Origin::signed(ALICE),
                market_id,
                // offset to not conflict with pushed outcomes
                OutcomeReport::Scalar(offset + i as u128),
            ));
            overall_fees = overall_fees.saturating_add(VotingOutcomeFee::get());
        }

        assert_ok!(GlobalDisputes::destroy_global_dispute(&market_id));

        let alice_free_balance_before = Balances::free_balance(&ALICE);
        assert_ok!(GlobalDisputes::refund_vote_fees(Origin::signed(ALICE), market_id,));

        System::assert_last_event(Event::<Runtime>::OutcomesPartiallyCleaned { market_id }.into());

        assert!(<Outcomes<Runtime>>::iter_prefix(market_id).next().is_some());

        assert_ok!(GlobalDisputes::refund_vote_fees(Origin::signed(ALICE), market_id,));

        System::assert_last_event(Event::<Runtime>::OutcomesFullyCleaned { market_id }.into());

        assert_eq!(<Outcomes<Runtime>>::iter_prefix(market_id).next(), None);

        assert_eq!(
            Balances::free_balance(&ALICE),
            alice_free_balance_before.saturating_add(overall_fees)
        );
    });
}

#[test]
fn unlock_clears_lock_info() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        set_vote_period();

        assert_ok!(GlobalDisputes::vote_on_outcome(
            Origin::signed(ALICE),
            market_id,
            OutcomeReport::Scalar(0),
            50 * BASE
        ));

        assert!(GlobalDisputes::determine_voting_winner(&market_id).is_some());

        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![(market_id, 50 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));

        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![]);
    });
}

#[test]
fn vote_fails_if_outcome_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = vec![
            InitialItem { owner: ALICE, outcome: OutcomeReport::Scalar(0), amount: 10 * BASE },
            InitialItem { owner: ALICE, outcome: OutcomeReport::Scalar(20), amount: 20 * BASE },
            InitialItem { owner: ALICE, outcome: OutcomeReport::Scalar(40), amount: 30 * BASE },
            InitialItem { owner: ALICE, outcome: OutcomeReport::Scalar(60), amount: 40 * BASE },
        ];

        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        set_vote_period();

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
fn locking_works_for_one_market() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, &market);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id, initial_items.as_slice()));

        set_vote_period();

        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![]);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(<Locks<Runtime>>::get(BOB), vec![]);
        assert!(Balances::locks(BOB).is_empty());
        assert_eq!(<Locks<Runtime>>::get(CHARLIE), vec![]);
        assert!(Balances::locks(CHARLIE).is_empty());
        assert_eq!(<Locks<Runtime>>::get(EVE), vec![]);
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

        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![(market_id, 50 * BASE)]);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(<Locks<Runtime>>::get(BOB), vec![(market_id, 40 * BASE)]);
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);
        assert_eq!(<Locks<Runtime>>::get(CHARLIE), vec![(market_id, 30 * BASE)]);
        assert_eq!(Balances::locks(CHARLIE), vec![the_lock(30 * BASE)]);
        assert_eq!(<Locks<Runtime>>::get(EVE), vec![(market_id, 20 * BASE)]);
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert!(GlobalDisputes::determine_voting_winner(&market_id).is_some());

        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![(market_id, 50 * BASE)]);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![]);
        assert!(Balances::locks(ALICE).is_empty());

        assert_eq!(<Locks<Runtime>>::get(BOB), vec![(market_id, 40 * BASE)]);
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);
        assert_eq!(<Locks<Runtime>>::get(CHARLIE), vec![(market_id, 30 * BASE)]);
        assert_eq!(Balances::locks(CHARLIE), vec![the_lock(30 * BASE)]);
        assert_eq!(<Locks<Runtime>>::get(EVE), vec![(market_id, 20 * BASE)]);
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(<Locks<Runtime>>::get(BOB), vec![]);
        assert!(Balances::locks(BOB).is_empty());
        assert_eq!(<Locks<Runtime>>::get(CHARLIE), vec![(market_id, 30 * BASE)]);
        assert_eq!(Balances::locks(CHARLIE), vec![the_lock(30 * BASE)]);
        assert_eq!(<Locks<Runtime>>::get(EVE), vec![(market_id, 20 * BASE)]);
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(CHARLIE), CHARLIE));
        assert_eq!(<Locks<Runtime>>::get(CHARLIE), vec![]);
        assert!(Balances::locks(CHARLIE).is_empty());
        assert_eq!(<Locks<Runtime>>::get(EVE), vec![(market_id, 20 * BASE)]);
        assert_eq!(Balances::locks(EVE), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(EVE), EVE));
        assert_eq!(<Locks<Runtime>>::get(EVE), vec![]);
        assert!(Balances::locks(EVE).is_empty());
    });
}

#[test]
fn locking_works_for_two_markets_with_stronger_first_unlock() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id_1 = 0u128;
        let market_1 = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id_1, &market_1);

        let market_id_2 = 1u128;
        let market_2 = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id_2, &market_2);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id_1, initial_items.as_slice()));
        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id_2, initial_items.as_slice()));

        set_vote_period();

        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![]);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(<Locks<Runtime>>::get(BOB), vec![]);
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

        assert_eq!(
            <Locks<Runtime>>::get(ALICE),
            vec![(market_id_1, 50 * BASE), (market_id_2, 30 * BASE)]
        );
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(
            <Locks<Runtime>>::get(BOB),
            vec![(market_id_1, 40 * BASE), (market_id_2, 20 * BASE)]
        );
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        // market_id_1 has stronger locks
        assert!(GlobalDisputes::determine_voting_winner(&market_id_1).is_some());

        assert_eq!(
            <Locks<Runtime>>::get(ALICE),
            vec![(market_id_1, 50 * BASE), (market_id_2, 30 * BASE)]
        );
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![(market_id_2, 30 * BASE)]);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(30 * BASE)]);
        assert_eq!(
            <Locks<Runtime>>::get(BOB),
            vec![(market_id_1, 40 * BASE), (market_id_2, 20 * BASE)]
        );
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(<Locks<Runtime>>::get(BOB), vec![(market_id_2, 20 * BASE)]);
        assert_eq!(Balances::locks(BOB), vec![the_lock(20 * BASE)]);
        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![(market_id_2, 30 * BASE)]);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(30 * BASE)]);

        assert!(GlobalDisputes::determine_voting_winner(&market_id_2).is_some());

        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![(market_id_2, 30 * BASE)]);
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));

        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![]);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(<Locks<Runtime>>::get(BOB), vec![(market_id_2, 20 * BASE)]);
        assert_eq!(Balances::locks(BOB), vec![the_lock(20 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(<Locks<Runtime>>::get(BOB), vec![]);
        assert!(Balances::locks(BOB).is_empty());
    });
}

#[test]
fn locking_works_for_two_markets_with_weaker_first_unlock() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id_1 = 0u128;
        let market_1 = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id_1, &market_1);

        let market_id_2 = 1u128;
        let market_2 = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id_2, &market_2);

        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id_1, initial_items.as_slice()));
        let initial_items = get_initial_items();
        assert_ok!(GlobalDisputes::start_global_dispute(&market_id_2, initial_items.as_slice()));

        set_vote_period();

        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![]);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(<Locks<Runtime>>::get(BOB), vec![]);
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

        assert_eq!(
            <Locks<Runtime>>::get(ALICE),
            vec![(market_id_1, 50 * BASE), (market_id_2, 30 * BASE)]
        );
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(
            <Locks<Runtime>>::get(BOB),
            vec![(market_id_1, 40 * BASE), (market_id_2, 20 * BASE)]
        );
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        // market_id_2 has weaker locks
        assert!(GlobalDisputes::determine_voting_winner(&market_id_2).is_some());

        assert_eq!(
            <Locks<Runtime>>::get(ALICE),
            vec![(market_id_1, 50 * BASE), (market_id_2, 30 * BASE)]
        );
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));
        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![(market_id_1, 50 * BASE)]);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_eq!(
            <Locks<Runtime>>::get(BOB),
            vec![(market_id_1, 40 * BASE), (market_id_2, 20 * BASE)]
        );
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(<Locks<Runtime>>::get(BOB), vec![(market_id_1, 40 * BASE)]);
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);
        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![(market_id_1, 50 * BASE)]);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);

        assert!(GlobalDisputes::determine_voting_winner(&market_id_1).is_some());

        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![(market_id_1, 50 * BASE)]);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(50 * BASE)]);
        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(ALICE), ALICE));

        assert_eq!(<Locks<Runtime>>::get(ALICE), vec![]);
        assert!(Balances::locks(ALICE).is_empty());
        assert_eq!(<Locks<Runtime>>::get(BOB), vec![(market_id_1, 40 * BASE)]);
        assert_eq!(Balances::locks(BOB), vec![the_lock(40 * BASE)]);

        assert_ok!(GlobalDisputes::unlock_vote_balance(Origin::signed(BOB), BOB));
        assert_eq!(<Locks<Runtime>>::get(BOB), vec![]);
        assert!(Balances::locks(BOB).is_empty());
    });
}
