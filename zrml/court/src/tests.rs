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

#![cfg(test)]

use crate::{
    mock::{
        run_blocks, run_to_block, Balances, Court, ExtBuilder, MarketCommons, Origin, Runtime,
        System, ALICE, BOB, CHARLIE, DAVE, EVE, FERDIE, GINA, HARRY, IAN, INITIAL_BALANCE,
        POOR_PAUL,
    },
    types::{Draw, Vote},
    AccountIdLookupOf, Draws, Error, Event, ExitRequests, JurorInfo, JurorInfoOf, JurorPool,
    JurorPoolItem, JurorPoolOf, Jurors, MarketOf,
};
use frame_support::{assert_noop, assert_ok, traits::fungible::Balanced};
use pallet_balances::BalanceLock;
use rand::seq::SliceRandom;
use zeitgeist_primitives::{
    constants::{
        mock::{CourtLockId, IterationLimit, MinJurorStake},
        BASE,
    },
    traits::DisputeApi,
    types::{
        AccountIdTest, Asset, Deadlines, Market, MarketBonds, MarketCreation,
        MarketDisputeMechanism, MarketPeriod, MarketStatus, MarketType, ScoringRule,
    },
};
use zrml_market_commons::MarketCommonsPalletApi;

const DEFAULT_MARKET: MarketOf<Runtime> = Market {
    base_asset: Asset::Ztg,
    creation: MarketCreation::Permissionless,
    creator_fee: 0,
    creator: 0,
    market_type: MarketType::Scalar(0..=100),
    dispute_mechanism: MarketDisputeMechanism::Court,
    metadata: vec![],
    oracle: 0,
    period: MarketPeriod::Block(0..100),
    deadlines: Deadlines { grace_period: 1_u64, oracle_duration: 1_u64, dispute_duration: 1_u64 },
    report: None,
    resolved_outcome: None,
    status: MarketStatus::Disputed,
    scoring_rule: ScoringRule::CPMM,
    bonds: MarketBonds { creation: None, oracle: None, outsider: None, dispute: None },
};

const DEFAULT_SET_OF_JURORS: &[JurorPoolItem<AccountIdTest, u128>] = &[
    JurorPoolItem { stake: 9, juror: HARRY, slashed: 0 },
    JurorPoolItem { stake: 8, juror: IAN, slashed: 0 },
    JurorPoolItem { stake: 7, juror: ALICE, slashed: 0 },
    JurorPoolItem { stake: 6, juror: BOB, slashed: 0 },
    JurorPoolItem { stake: 5, juror: CHARLIE, slashed: 0 },
    JurorPoolItem { stake: 4, juror: DAVE, slashed: 0 },
    JurorPoolItem { stake: 3, juror: EVE, slashed: 0 },
    JurorPoolItem { stake: 2, juror: FERDIE, slashed: 0 },
    JurorPoolItem { stake: 1, juror: GINA, slashed: 0 },
];

fn the_lock(amount: u128) -> BalanceLock<u128> {
    BalanceLock { id: CourtLockId::get(), amount, reasons: pallet_balances::Reasons::All }
}

#[test]
fn exit_court_successfully_removes_a_juror_and_frees_balances() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert_eq!(Jurors::<Runtime>::iter().count(), 1);
        assert_eq!(Balances::free_balance(ALICE), INITIAL_BALANCE);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(amount)]);
        assert_ok!(Court::prepare_exit_court(Origin::signed(ALICE)));
        assert_ok!(Court::exit_court(Origin::signed(ALICE), ALICE));
        assert_eq!(Jurors::<Runtime>::iter().count(), 0);
        assert_eq!(Balances::free_balance(ALICE), INITIAL_BALANCE);
        assert_eq!(Balances::locks(ALICE), vec![]);
    });
}

#[test]
fn prepare_exit_court_will_not_remove_an_unknown_juror() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Court::prepare_exit_court(Origin::signed(ALICE)),
            Error::<Runtime>::JurorDoesNotExist
        );
    });
}

#[test]
fn join_court_successfully_stores_required_data() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        let alice_free_balance_before = Balances::free_balance(ALICE);
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        System::assert_last_event(Event::JurorJoined { juror: ALICE }.into());
        assert_eq!(Jurors::<Runtime>::iter().next().unwrap(), (ALICE, JurorInfo { stake: amount }));
        assert_eq!(Balances::free_balance(ALICE), alice_free_balance_before);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(amount)]);
        assert_eq!(
            JurorPool::<Runtime>::get().into_inner(),
            vec![JurorPoolItem { stake: amount, juror: ALICE, slashed: 0 }]
        );
    });
}

#[test]
fn join_court_works_multiple_joins() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let amount = 2 * min;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(amount)]);
        assert_eq!(
            JurorPool::<Runtime>::get().into_inner(),
            vec![JurorPoolItem { stake: amount, juror: ALICE, slashed: 0 }]
        );
        assert_eq!(
            Jurors::<Runtime>::iter().collect::<Vec<(AccountIdTest, JurorInfoOf<Runtime>)>>(),
            vec![(ALICE, JurorInfo { stake: amount })]
        );

        assert_ok!(Court::join_court(Origin::signed(BOB), amount));
        assert_eq!(Balances::locks(BOB), vec![the_lock(amount)]);
        assert_eq!(
            JurorPool::<Runtime>::get().into_inner(),
            vec![
                JurorPoolItem { stake: amount, juror: ALICE, slashed: 0 },
                JurorPoolItem { stake: amount, juror: BOB, slashed: 0 }
            ]
        );
        assert_eq!(
            Jurors::<Runtime>::iter().collect::<Vec<(AccountIdTest, JurorInfoOf<Runtime>)>>(),
            vec![(BOB, JurorInfo { stake: amount }), (ALICE, JurorInfo { stake: amount })]
        );

        let higher_amount = amount + 1;
        assert_ok!(Court::join_court(Origin::signed(ALICE), higher_amount));
        assert_eq!(Balances::locks(BOB), vec![the_lock(amount)]);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(higher_amount)]);
        assert_eq!(
            JurorPool::<Runtime>::get().into_inner(),
            vec![
                JurorPoolItem { stake: amount, juror: BOB, slashed: 0 },
                JurorPoolItem { stake: higher_amount, juror: ALICE, slashed: 0 },
            ]
        );
        assert_eq!(
            Jurors::<Runtime>::iter().collect::<Vec<(AccountIdTest, JurorInfoOf<Runtime>)>>(),
            vec![(BOB, JurorInfo { stake: amount }), (ALICE, JurorInfo { stake: higher_amount })]
        );
    });
}

#[test]
fn join_court_saves_slashed_for_double_join() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let amount = 2 * min;

        let slashed = min;
        Jurors::<Runtime>::insert(ALICE, JurorInfo { stake: amount });
        let juror_pool = vec![JurorPoolItem { stake: amount, juror: ALICE, slashed }];
        JurorPool::<Runtime>::put::<JurorPoolOf<Runtime>>(juror_pool.try_into().unwrap());

        let higher_amount = amount + 1;
        assert_ok!(Court::join_court(Origin::signed(ALICE), higher_amount));
        assert_eq!(JurorPool::<Runtime>::get().into_inner()[0].slashed, slashed);
    });
}

#[test]
fn join_court_fails_below_min_juror_stake() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let amount = min - 1;
        assert_noop!(
            Court::join_court(Origin::signed(ALICE), amount),
            Error::<Runtime>::BelowMinJurorStake
        );
    });
}

#[test]
fn join_court_fails_insufficient_amount() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let amount = min + 1;
        assert_noop!(
            Court::join_court(Origin::signed(POOR_PAUL), amount),
            Error::<Runtime>::InsufficientAmount
        );
    });
}

#[test]
fn join_court_fails_amount_below_last_join() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let last_join_amount = 2 * min;
        assert_ok!(Court::join_court(Origin::signed(ALICE), last_join_amount));

        assert_noop!(
            Court::join_court(Origin::signed(ALICE), last_join_amount - 1),
            Error::<Runtime>::AmountBelowLastJoin
        );
    });
}

#[test]
fn join_court_fails_juror_needs_to_exit() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let amount = 2 * min;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));

        assert_ok!(Court::prepare_exit_court(Origin::signed(ALICE)));

        assert_noop!(
            Court::join_court(Origin::signed(ALICE), amount + 1),
            Error::<Runtime>::JurorNeedsToExit
        );
    });
}

#[test]
fn join_court_fails_amount_below_lowest_juror() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let min_amount = 2 * min;

        let max_accounts = JurorPoolOf::<Runtime>::bound();
        let max_amount = min_amount + max_accounts as u128;
        for i in 1..=max_accounts {
            let amount = max_amount - i as u128;
            let _ = Balances::deposit(&(i as u128), amount).unwrap();
            assert_ok!(Court::join_court(Origin::signed(i as u128), amount));
        }

        assert!(JurorPool::<Runtime>::get().is_full());

        assert_noop!(
            Court::join_court(Origin::signed(0u128), min_amount - 1),
            Error::<Runtime>::AmountBelowLowestJuror
        );
    });
}

#[test]
fn prepare_exit_court_works() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert_eq!(
            JurorPool::<Runtime>::get().into_inner(),
            vec![JurorPoolItem { stake: amount, juror: ALICE, slashed: 0 }]
        );

        assert_ok!(Court::prepare_exit_court(Origin::signed(ALICE)));
        System::assert_last_event(Event::JurorPreparedExit { juror: ALICE }.into());
        assert!(JurorPool::<Runtime>::get().into_inner().is_empty());
    });
}

#[test]
fn prepare_exit_court_removes_correct_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let min_amount = 2 * min;

        let max_accounts = JurorPoolOf::<Runtime>::bound();
        let mut rng = rand::thread_rng();
        let mut random_numbers: Vec<u32> = (0u32..max_accounts as u32).collect();
        random_numbers.shuffle(&mut rng);
        let mut random_jurors = random_numbers.clone();
        random_jurors.shuffle(&mut rng);
        let max_amount = min_amount + max_accounts as u128;
        for i in random_numbers {
            let amount = max_amount - i as u128;
            let juror = random_jurors.remove(0) as u128;
            let _ = Balances::deposit(&juror, amount).unwrap();
            assert_ok!(Court::join_court(Origin::signed(juror), amount));
        }

        // println!("JurorPool: {:?}", JurorPool::<Runtime>::get().into_inner());

        for r in 0..max_accounts {
            let len = JurorPool::<Runtime>::get().into_inner().len();
            assert!(
                JurorPool::<Runtime>::get().into_inner().iter().any(|item| item.juror == r as u128)
            );
            assert_ok!(Court::prepare_exit_court(Origin::signed(r as u128)));
            assert_eq!(JurorPool::<Runtime>::get().into_inner().len(), len - 1);
            JurorPool::<Runtime>::get().into_inner().iter().for_each(|item| {
                assert_ne!(item.juror, r as u128);
            });
        }
    });
}

#[test]
fn prepare_exit_court_fails_juror_already_prepared_to_exit() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert_eq!(
            JurorPool::<Runtime>::get().into_inner(),
            vec![JurorPoolItem { stake: amount, juror: ALICE, slashed: 0 }]
        );

        assert_ok!(Court::prepare_exit_court(Origin::signed(ALICE)));
        assert!(JurorPool::<Runtime>::get().into_inner().is_empty());

        assert_noop!(
            Court::prepare_exit_court(Origin::signed(ALICE)),
            Error::<Runtime>::JurorAlreadyPreparedToExit
        );
    });
}

#[test]
fn prepare_exit_court_fails_juror_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(
            Jurors::<Runtime>::iter()
                .collect::<Vec<(AccountIdTest, JurorInfoOf<Runtime>)>>()
                .is_empty()
        );

        assert_noop!(
            Court::prepare_exit_court(Origin::signed(ALICE)),
            Error::<Runtime>::JurorDoesNotExist
        );
    });
}

#[test]
fn exit_court_works() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert_ok!(Court::prepare_exit_court(Origin::signed(ALICE)));
        assert!(JurorPool::<Runtime>::get().into_inner().is_empty());
        assert!(Jurors::<Runtime>::get(ALICE).is_some());

        assert_eq!(Balances::locks(ALICE), vec![the_lock(amount)]);
        let alice_lookup: AccountIdLookupOf<Runtime> = ALICE.into();
        assert_ok!(Court::exit_court(Origin::signed(ALICE), alice_lookup));
        System::assert_last_event(Event::JurorExited { juror: ALICE }.into());
        assert!(
            Jurors::<Runtime>::iter()
                .collect::<Vec<(AccountIdTest, JurorInfoOf<Runtime>)>>()
                .is_empty()
        );
        assert!(!ExitRequests::<Runtime>::contains_key(ALICE));
        assert!(Balances::locks(ALICE).is_empty());
    });
}

#[test]
fn exit_court_fails_juror_still_drawn() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert_ok!(Court::prepare_exit_court(Origin::signed(ALICE)));

        let mut draws = <Draws<Runtime>>::get(0);
        draws
            .try_push(Draw { juror: ALICE, weight: 0u32, vote: Vote::Drawn, slashable: 0u128 })
            .unwrap();
        <Draws<Runtime>>::insert(0, draws);
        let alice_lookup: AccountIdLookupOf<Runtime> = ALICE.into();
        assert_noop!(
            Court::exit_court(Origin::signed(ALICE), alice_lookup),
            Error::<Runtime>::JurorStillDrawn
        );
    });
}

#[test]
fn exit_court_works_over_iteration_limit() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert_ok!(Court::prepare_exit_court(Origin::signed(ALICE)));

        let limit = IterationLimit::get();
        for i in 0..(2 * limit) {
            let mut draws = <Draws<Runtime>>::get(i as u128);
            draws
                .try_push(Draw {
                    juror: CHARLIE,
                    weight: 0u32,
                    vote: Vote::Drawn,
                    slashable: 0u128,
                })
                .unwrap();
            <Draws<Runtime>>::insert(i as u128, draws);
        }
        let alice_lookup: AccountIdLookupOf<Runtime> = ALICE.into();
        assert_ok!(Court::exit_court(Origin::signed(ALICE), alice_lookup));
        System::assert_last_event(Event::JurorMayStillBeDrawn { juror: ALICE }.into());
        let exit_request = <ExitRequests<Runtime>>::get(ALICE);

        let last_query = <Draws<Runtime>>::iter().skip(limit as usize).next().unwrap().0;
        assert_eq!(exit_request.unwrap().last_market_id, Some(last_query));

        assert_ok!(Court::exit_court(Origin::signed(ALICE), alice_lookup));
        System::assert_last_event(Event::JurorExited { juror: ALICE }.into());
    });
}

#[test]
fn check_draws_iter_new_inserts_only_after_previous() {
    ExtBuilder::default().build().execute_with(|| {
        let limit = IterationLimit::get();
        let excess = 2 * limit;
        for i in 0..excess {
            let mut draws = <Draws<Runtime>>::get(i as u128);
            draws
                .try_push(Draw {
                    juror: CHARLIE,
                    weight: 0u32,
                    vote: Vote::Drawn,
                    slashable: 0u128,
                })
                .unwrap();
            <Draws<Runtime>>::insert(i as u128, draws);
        }

        let draws = <Draws<Runtime>>::iter().map(|(key, _)| key).collect::<Vec<_>>();

        let mut numbers: Vec<u32> = (excess..(excess + limit)).collect();
        let mut rng = rand::thread_rng();
        numbers.shuffle(&mut rng);
        for i in numbers {
            let mut draws = <Draws<Runtime>>::get(i as u128);
            draws
                .try_push(Draw {
                    juror: CHARLIE,
                    weight: 0u32,
                    vote: Vote::Drawn,
                    slashable: 0u128,
                })
                .unwrap();
            <Draws<Runtime>>::insert(i as u128, draws);
        }

        let first_key = <Draws<Runtime>>::iter().next().unwrap().0;
        let hashed_key = <Draws<Runtime>>::hashed_key_for(first_key);
        let new_draws = <Draws<Runtime>>::iter_from(hashed_key)
            .map(|(key, _)| key)
            .take(excess as usize)
            .collect::<Vec<_>>();
        assert_eq!(draws, new_draws);
    });
}

#[test]
fn exit_court_fails_juror_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_lookup: AccountIdLookupOf<Runtime> = ALICE.into();
        assert_noop!(
            Court::exit_court(Origin::signed(ALICE), alice_lookup),
            Error::<Runtime>::JurorDoesNotExist
        );
    });
}

#[test]
fn exit_court_fails_juror_not_prepared_to_exit() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));

        let alice_lookup: AccountIdLookupOf<Runtime> = ALICE.into();
        assert_noop!(
            Court::exit_court(Origin::signed(ALICE), alice_lookup),
            Error::<Runtime>::JurorNotPreparedToExit
        );
    });
}

#[test]
fn on_dispute_denies_non_court_markets() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.dispute_mechanism = MarketDisputeMechanism::SimpleDisputes;
        assert_noop!(
            Court::on_dispute(&0, &market),
            Error::<Runtime>::MarketDoesNotHaveCourtMechanism
        );
    });
}

#[test]
fn get_resolution_outcome_denies_non_court_markets() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.dispute_mechanism = MarketDisputeMechanism::SimpleDisputes;
        assert_noop!(
            Court::get_resolution_outcome(&0, &market),
            Error::<Runtime>::MarketDoesNotHaveCourtMechanism
        );
    });
}

#[test]
fn appeal_stores_jurors_that_should_vote() {
    ExtBuilder::default().build().execute_with(|| {
        run_to_block(123);
    });
}

// Alice is the winner, Bob is tardy and Charlie is the loser
#[test]
fn get_resolution_outcome_awards_winners_and_slashes_losers() {
    ExtBuilder::default().build().execute_with(|| {
        run_to_block(2);
        let amount_alice = 2 * BASE;
        let amount_bob = 3 * BASE;
        let amount_charlie = 4 * BASE;
        Court::join_court(Origin::signed(ALICE), amount_alice).unwrap();
        Court::join_court(Origin::signed(BOB), amount_bob).unwrap();
        Court::join_court(Origin::signed(CHARLIE), amount_charlie).unwrap();
        MarketCommons::push_market(DEFAULT_MARKET).unwrap();
    });
}

#[test]
fn get_resolution_outcome_decides_market_outcome_based_on_the_plurality() {
    ExtBuilder::default().build().execute_with(|| {
        run_to_block(2);
        let amount_alice = 2 * BASE;
        let amount_bob = 3 * BASE;
        let amount_charlie = 4 * BASE;
        Court::join_court(Origin::signed(ALICE), amount_alice).unwrap();
        Court::join_court(Origin::signed(BOB), amount_bob).unwrap();
        Court::join_court(Origin::signed(CHARLIE), amount_charlie).unwrap();
        MarketCommons::push_market(DEFAULT_MARKET).unwrap();
    });
}

#[test]
fn random_jurors_returns_an_unique_different_subset_of_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        run_to_block(123);

        let mut rng = Court::rng();
        let random_jurors =
            Court::choose_multiple_weighted(DEFAULT_SET_OF_JURORS, 2, &mut rng).unwrap();
        let mut at_least_one_set_is_different = false;

        for _ in 0..100 {
            run_blocks(1);

            let another_set_of_random_jurors =
                Court::choose_multiple_weighted(DEFAULT_SET_OF_JURORS, 2, &mut rng).unwrap();
            let mut iter = another_set_of_random_jurors.iter();

            if let Some(juror) = iter.next() {
                at_least_one_set_is_different = random_jurors.iter().all(|el| el != juror);
            } else {
                continue;
            }
            for juror in iter {
                at_least_one_set_is_different &= random_jurors.iter().all(|el| el != juror);
            }

            if at_least_one_set_is_different {
                break;
            }
        }

        assert!(at_least_one_set_is_different);
    });
}

#[test]
fn random_jurors_returns_a_subset_of_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        run_to_block(123);
        let mut rng = Court::rng();
        let random_jurors =
            Court::choose_multiple_weighted(DEFAULT_SET_OF_JURORS, 2, &mut rng).unwrap();
        for draw in random_jurors {
            assert!(DEFAULT_SET_OF_JURORS.iter().any(|el| el.juror == draw.juror));
        }
    });
}

#[test]
fn vote_will_not_accept_unknown_accounts() {
    ExtBuilder::default().build().execute_with(|| {
        run_to_block(123);
        let amount_alice = 2 * BASE;
        let amount_bob = 3 * BASE;
        let amount_charlie = 4 * BASE;
        let amount_eve = 5 * BASE;
        let amount_dave = 6 * BASE;
        Court::join_court(Origin::signed(ALICE), amount_alice).unwrap();
        Court::join_court(Origin::signed(BOB), amount_bob).unwrap();
        Court::join_court(Origin::signed(CHARLIE), amount_charlie).unwrap();
        Court::join_court(Origin::signed(EVE), amount_eve).unwrap();
        Court::join_court(Origin::signed(DAVE), amount_dave).unwrap();
        Court::on_dispute(&0, &DEFAULT_MARKET).unwrap();
    });
}
