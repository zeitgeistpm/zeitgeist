// Copyright 2022-2023 Forecasting Technologies LTD.
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

extern crate alloc;
use crate::{
    mock::{
        run_blocks, run_to_block, Balances, Court, ExtBuilder, MarketCommons, Origin, Runtime,
        System, ALICE, BOB, CHARLIE, DAVE, EVE, FERDIE, GINA, HARRY, IAN, INITIAL_BALANCE,
        POOR_PAUL,
    },
    mock_storage::pallet::MarketIdsPerDisputeBlock,
    types::{CourtStatus, Draw, Vote},
    AppealInfo, BalanceOf, Courts, Draws, Error, Event, JurorInfo, JurorInfoOf, JurorPool,
    JurorPoolItem, JurorPoolOf, Jurors, MarketIdOf, MarketOf, NegativeImbalanceOf, RequestBlock,
};
use alloc::collections::BTreeMap;
use frame_support::{
    assert_noop, assert_ok,
    traits::{fungible::Balanced, tokens::imbalance::Imbalance, Currency, NamedReservableCurrency},
};
use pallet_balances::{BalanceLock, NegativeImbalance};
use rand::seq::SliceRandom;
use sp_runtime::traits::{BlakeTwo256, Hash, Zero};
use test_case::test_case;
use zeitgeist_primitives::{
    constants::{
        mock::{
            AppealBond, CourtAggregationPeriod, CourtAppealPeriod, CourtLockId, CourtVotePeriod,
            InflationPeriod, MaxAppeals, MaxJurors, MinJurorStake, RequestInterval,
        },
        BASE,
    },
    traits::DisputeApi,
    types::{
        AccountIdTest, Asset, Deadlines, Market, MarketBonds, MarketCreation,
        MarketDisputeMechanism, MarketPeriod, MarketStatus, MarketType, OutcomeReport, Report,
        ScoringRule,
    },
};
use zrml_market_commons::{Error as MError, MarketCommonsPalletApi};

const ORACLE_REPORT: OutcomeReport = OutcomeReport::Scalar(u128::MAX);

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

fn initialize_court() -> MarketIdOf<Runtime> {
    let now = <frame_system::Pallet<Runtime>>::block_number();
    <RequestBlock<Runtime>>::put(now + RequestInterval::get());
    let amount_alice = 2 * BASE;
    let amount_bob = 3 * BASE;
    let amount_charlie = 4 * BASE;
    let amount_dave = 5 * BASE;
    let amount_eve = 6 * BASE;
    Court::join_court(Origin::signed(ALICE), amount_alice).unwrap();
    Court::join_court(Origin::signed(BOB), amount_bob).unwrap();
    Court::join_court(Origin::signed(CHARLIE), amount_charlie).unwrap();
    Court::join_court(Origin::signed(DAVE), amount_dave).unwrap();
    Court::join_court(Origin::signed(EVE), amount_eve).unwrap();
    let market_id = MarketCommons::push_market(DEFAULT_MARKET).unwrap();
    MarketCommons::mutate_market(&market_id, |market| {
        market.report = Some(Report { at: 1, by: BOB, outcome: ORACLE_REPORT });
        Ok(())
    })
    .unwrap();
    Court::on_dispute(&market_id, &DEFAULT_MARKET).unwrap();
    market_id
}

fn fill_juror_pool() {
    for i in 0..MaxJurors::get() {
        let amount = MinJurorStake::get() + i as u128;
        let juror = (i + 1000) as u128;
        let _ = Balances::deposit(&juror, amount).unwrap();
        assert_ok!(Court::join_court(Origin::signed(juror), amount));
    }
}

fn fill_appeals(market_id: &MarketIdOf<Runtime>, appeal_number: usize) {
    assert!(appeal_number <= MaxAppeals::get() as usize);
    let mut court = Courts::<Runtime>::get(market_id).unwrap();
    let mut number = 0u128;
    while (number as usize) < appeal_number {
        let appealed_outcome = OutcomeReport::Scalar(number);
        court
            .appeals
            .try_push(AppealInfo {
                backer: number,
                bond: crate::get_appeal_bond::<Runtime>(court.appeals.len()),
                appealed_outcome,
            })
            .unwrap();
        number += 1;
    }
    Courts::<Runtime>::insert(market_id, court);
}

fn put_alice_in_draw(market_id: MarketIdOf<Runtime>, stake: BalanceOf<Runtime>) {
    // trick a little bit to let alice be part of the ("random") selection
    let mut draws = <Draws<Runtime>>::get(market_id);
    assert!(!draws.is_empty());
    let slashable = MinJurorStake::get();
    draws[0] = Draw { juror: ALICE, weight: 1, vote: Vote::Drawn, slashable };
    <Draws<Runtime>>::insert(market_id, draws);
    <Jurors<Runtime>>::insert(
        ALICE,
        JurorInfo { stake, active_lock: slashable, prepare_exit_at: None },
    );
}

fn set_alice_after_vote(
    outcome: OutcomeReport,
) -> (
    MarketIdOf<Runtime>,
    <Runtime as frame_system::Config>::Hash,
    <Runtime as frame_system::Config>::Hash,
) {
    fill_juror_pool();
    let market_id = initialize_court();

    let amount = MinJurorStake::get() * 100;
    assert_ok!(Court::join_court(Origin::signed(ALICE), amount));

    put_alice_in_draw(market_id, amount);

    run_to_block(<RequestBlock<Runtime>>::get() + 1);

    let salt = <Runtime as frame_system::Config>::Hash::default();
    let commitment = BlakeTwo256::hash_of(&(ALICE, outcome, salt));
    assert_ok!(Court::vote(Origin::signed(ALICE), market_id, commitment));

    (market_id, commitment, salt)
}

const DEFAULT_SET_OF_JURORS: &[JurorPoolItem<AccountIdTest, u128>] = &[
    JurorPoolItem { stake: 9, juror: HARRY, consumed_stake: 0 },
    JurorPoolItem { stake: 8, juror: IAN, consumed_stake: 0 },
    JurorPoolItem { stake: 7, juror: ALICE, consumed_stake: 0 },
    JurorPoolItem { stake: 6, juror: BOB, consumed_stake: 0 },
    JurorPoolItem { stake: 5, juror: CHARLIE, consumed_stake: 0 },
    JurorPoolItem { stake: 4, juror: DAVE, consumed_stake: 0 },
    JurorPoolItem { stake: 3, juror: EVE, consumed_stake: 0 },
    JurorPoolItem { stake: 2, juror: FERDIE, consumed_stake: 0 },
    JurorPoolItem { stake: 1, juror: GINA, consumed_stake: 0 },
];

fn the_lock(amount: u128) -> BalanceLock<u128> {
    BalanceLock { id: CourtLockId::get(), amount, reasons: pallet_balances::Reasons::All }
}

#[test]
fn exit_court_successfully_removes_a_juror_and_frees_balances() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert_ok!(Court::prepare_exit_court(Origin::signed(ALICE)));
        run_blocks(InflationPeriod::get());
        assert_ok!(Court::exit_court(Origin::signed(ALICE), ALICE));
        assert_eq!(Jurors::<Runtime>::iter().count(), 0);
        assert_eq!(Balances::free_balance(ALICE), INITIAL_BALANCE);
        assert_eq!(Balances::locks(ALICE), vec![]);
    });
}

#[test]
fn join_court_successfully_stores_required_data() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        let alice_free_balance_before = Balances::free_balance(ALICE);
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        System::assert_last_event(Event::JurorJoined { juror: ALICE, stake: amount }.into());
        assert_eq!(
            Jurors::<Runtime>::iter().next().unwrap(),
            (ALICE, JurorInfo { stake: amount, active_lock: 0u128, prepare_exit_at: None })
        );
        assert_eq!(Balances::free_balance(ALICE), alice_free_balance_before);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(amount)]);
        assert_eq!(
            JurorPool::<Runtime>::get().into_inner(),
            vec![JurorPoolItem { stake: amount, juror: ALICE, consumed_stake: 0 }]
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
            vec![JurorPoolItem { stake: amount, juror: ALICE, consumed_stake: 0 }]
        );
        assert_eq!(
            Jurors::<Runtime>::iter().collect::<Vec<(AccountIdTest, JurorInfoOf<Runtime>)>>(),
            vec![(ALICE, JurorInfo { stake: amount, active_lock: 0u128, prepare_exit_at: None })]
        );

        assert_ok!(Court::join_court(Origin::signed(BOB), amount));
        assert_eq!(Balances::locks(BOB), vec![the_lock(amount)]);
        assert_eq!(
            JurorPool::<Runtime>::get().into_inner(),
            vec![
                JurorPoolItem { stake: amount, juror: ALICE, consumed_stake: 0 },
                JurorPoolItem { stake: amount, juror: BOB, consumed_stake: 0 }
            ]
        );
        assert_eq!(Jurors::<Runtime>::iter().count(), 2);
        assert_eq!(
            Jurors::<Runtime>::get(ALICE).unwrap(),
            JurorInfo { stake: amount, active_lock: 0u128, prepare_exit_at: None }
        );
        assert_eq!(
            Jurors::<Runtime>::get(BOB).unwrap(),
            JurorInfo { stake: amount, active_lock: 0u128, prepare_exit_at: None }
        );

        let higher_amount = amount + 1;
        assert_ok!(Court::join_court(Origin::signed(ALICE), higher_amount));
        assert_eq!(Balances::locks(BOB), vec![the_lock(amount)]);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(higher_amount)]);
        assert_eq!(
            JurorPool::<Runtime>::get().into_inner(),
            vec![
                JurorPoolItem { stake: amount, juror: BOB, consumed_stake: 0 },
                JurorPoolItem { stake: higher_amount, juror: ALICE, consumed_stake: 0 },
            ]
        );
        assert_eq!(Jurors::<Runtime>::iter().count(), 2);
        assert_eq!(
            Jurors::<Runtime>::get(BOB).unwrap(),
            JurorInfo { stake: amount, active_lock: 0u128, prepare_exit_at: None }
        );
        assert_eq!(
            Jurors::<Runtime>::get(ALICE).unwrap(),
            JurorInfo { stake: higher_amount, active_lock: 0u128, prepare_exit_at: None }
        );
    });
}

#[test]
fn join_court_saves_consumed_stake_and_active_lock_for_double_join() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let amount = 2 * min;

        let consumed_stake = min;
        let active_lock = min + 1;
        Jurors::<Runtime>::insert(
            ALICE,
            JurorInfo { stake: amount, active_lock, prepare_exit_at: None },
        );
        let juror_pool = vec![JurorPoolItem { stake: amount, juror: ALICE, consumed_stake }];
        JurorPool::<Runtime>::put::<JurorPoolOf<Runtime>>(juror_pool.try_into().unwrap());

        let higher_amount = amount + 1;
        assert_ok!(Court::join_court(Origin::signed(ALICE), higher_amount));
        assert_eq!(JurorPool::<Runtime>::get().into_inner()[0].consumed_stake, consumed_stake);
        assert_eq!(Jurors::<Runtime>::get(ALICE).unwrap().active_lock, active_lock);
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
fn join_court_fails_if_amount_exceeds_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let amount = min + 1;
        assert_noop!(
            Court::join_court(Origin::signed(POOR_PAUL), amount),
            Error::<Runtime>::AmountExceedsBalance
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
            vec![JurorPoolItem { stake: amount, juror: ALICE, consumed_stake: 0 }]
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

        for i in 0..JurorPoolOf::<Runtime>::bound() {
            let amount = min_amount + i as u128;
            let juror = i as u128;
            let _ = Balances::deposit(&juror, amount).unwrap();
            assert_ok!(Court::join_court(Origin::signed(juror), amount));
        }

        for r in 0..JurorPoolOf::<Runtime>::bound() {
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
fn join_court_binary_search_sorted_insert_works() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let min_amount = 2 * min;

        let max_accounts = JurorPoolOf::<Runtime>::bound();
        let mut rng = rand::thread_rng();
        let mut random_numbers: Vec<u32> = (0u32..max_accounts as u32).collect();
        random_numbers.shuffle(&mut rng);
        let max_amount = min_amount + max_accounts as u128;
        for i in random_numbers {
            let amount = max_amount - i as u128;
            let juror = i as u128;
            let _ = Balances::deposit(&juror, amount).unwrap();
            assert_ok!(Court::join_court(Origin::signed(juror), amount));
        }

        let mut last_stake = 0;
        for pool_item in JurorPool::<Runtime>::get().into_inner().iter() {
            assert!(pool_item.stake >= last_stake);
            last_stake = pool_item.stake;
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
            vec![JurorPoolItem { stake: amount, juror: ALICE, consumed_stake: 0 }]
        );

        assert_ok!(Court::prepare_exit_court(Origin::signed(ALICE)));

        assert_noop!(
            Court::prepare_exit_court(Origin::signed(ALICE)),
            Error::<Runtime>::JurorAlreadyPreparedToExit
        );
    });
}

#[test]
fn prepare_exit_court_fails_juror_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(Jurors::<Runtime>::iter().next().is_none());

        assert_noop!(
            Court::prepare_exit_court(Origin::signed(ALICE)),
            Error::<Runtime>::JurorDoesNotExist
        );
    });
}

#[test]
fn exit_court_works_without_active_lock() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let amount = 2 * min;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert!(!JurorPool::<Runtime>::get().into_inner().is_empty());
        assert_ok!(Court::prepare_exit_court(Origin::signed(ALICE)));
        assert!(JurorPool::<Runtime>::get().into_inner().is_empty());
        assert!(Jurors::<Runtime>::get(ALICE).is_some());

        run_blocks(InflationPeriod::get());

        assert_eq!(Balances::locks(ALICE), vec![the_lock(amount)]);
        assert_ok!(Court::exit_court(Origin::signed(ALICE), ALICE));
        System::assert_last_event(
            Event::JurorExited { juror: ALICE, exit_amount: amount, active_lock: 0u128 }.into(),
        );
        assert!(Jurors::<Runtime>::iter().next().is_none());
        assert!(Balances::locks(ALICE).is_empty());
    });
}

#[test]
fn exit_court_works_with_active_lock() {
    ExtBuilder::default().build().execute_with(|| {
        let active_lock = MinJurorStake::get();
        let amount = 3 * active_lock;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert!(!JurorPool::<Runtime>::get().into_inner().is_empty());

        assert_eq!(
            <Jurors<Runtime>>::get(ALICE).unwrap(),
            JurorInfo { stake: amount, active_lock: 0, prepare_exit_at: None }
        );
        // assume that `choose_multiple_weighted` has set the active_lock
        <Jurors<Runtime>>::insert(
            ALICE,
            JurorInfo { stake: amount, active_lock, prepare_exit_at: None },
        );

        assert_eq!(Balances::locks(ALICE), vec![the_lock(amount)]);

        let now = <frame_system::Pallet<Runtime>>::block_number();
        assert_ok!(Court::prepare_exit_court(Origin::signed(ALICE)));
        assert!(JurorPool::<Runtime>::get().into_inner().is_empty());

        run_blocks(InflationPeriod::get());

        assert_ok!(Court::exit_court(Origin::signed(ALICE), ALICE));
        System::assert_last_event(
            Event::JurorExited { juror: ALICE, exit_amount: amount - active_lock, active_lock }
                .into(),
        );
        assert_eq!(
            Jurors::<Runtime>::get(ALICE).unwrap(),
            JurorInfo { stake: active_lock, active_lock, prepare_exit_at: Some(now) }
        );
        assert_eq!(Balances::locks(ALICE), vec![the_lock(active_lock)]);
    });
}

#[test]
fn exit_court_fails_juror_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Court::exit_court(Origin::signed(ALICE), ALICE),
            Error::<Runtime>::JurorDoesNotExist
        );
    });
}

#[test]
fn exit_court_fails_juror_not_prepared_to_exit() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));

        run_blocks(InflationPeriod::get());

        assert_noop!(
            Court::exit_court(Origin::signed(ALICE), ALICE),
            Error::<Runtime>::JurorNotPreparedToExit
        );
    });
}

#[test]
fn exit_court_fails_if_inflation_period_not_over() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));

        assert_ok!(Court::prepare_exit_court(Origin::signed(ALICE)));

        run_blocks(InflationPeriod::get() - 1);

        assert_noop!(
            Court::exit_court(Origin::signed(ALICE), ALICE),
            Error::<Runtime>::WaitFullInflationPeriod
        );
    });
}

#[test]
fn vote_works() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool();
        let market_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));

        // trick a little bit to let alice be part of the ("random") selection
        let mut draws = <Draws<Runtime>>::get(market_id);
        assert_eq!(draws.len(), 5usize);
        let slashable = MinJurorStake::get();
        let alice_index = 3usize;
        draws[alice_index] = Draw { juror: ALICE, weight: 1, vote: Vote::Drawn, slashable };
        <Draws<Runtime>>::insert(market_id, draws);
        <Jurors<Runtime>>::insert(
            ALICE,
            JurorInfo { stake: amount, active_lock: slashable, prepare_exit_at: None },
        );

        let old_draws = <Draws<Runtime>>::get(market_id);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        let outcome = OutcomeReport::Scalar(42u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome, salt));
        assert_ok!(Court::vote(Origin::signed(ALICE), market_id, commitment));
        System::assert_last_event(Event::JurorVoted { juror: ALICE, market_id, commitment }.into());

        let new_draws = <Draws<Runtime>>::get(market_id);
        for (i, (old_draw, new_draw)) in old_draws.iter().zip(new_draws.iter()).enumerate() {
            if i == alice_index {
                continue;
            } else {
                assert_eq!(old_draw, new_draw);
            }
        }
        assert_eq!(old_draws[alice_index].juror, new_draws[alice_index].juror);
        assert_eq!(old_draws[alice_index].weight, new_draws[alice_index].weight);
        assert_eq!(old_draws[alice_index].slashable, new_draws[alice_index].slashable);
        assert_eq!(old_draws[alice_index].vote, Vote::Drawn);
        assert_eq!(new_draws[alice_index].vote, Vote::Secret { commitment });
    });
}

#[test]
fn vote_overwrite_works() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool();
        let market_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));

        put_alice_in_draw(market_id, amount);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        let wrong_outcome = OutcomeReport::Scalar(69u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let wrong_commitment = BlakeTwo256::hash_of(&(ALICE, wrong_outcome, salt));
        assert_ok!(Court::vote(Origin::signed(ALICE), market_id, wrong_commitment));
        assert_eq!(
            <Draws<Runtime>>::get(market_id)[0].vote,
            Vote::Secret { commitment: wrong_commitment }
        );

        run_blocks(1);

        let right_outcome = OutcomeReport::Scalar(42u128);
        let new_commitment = BlakeTwo256::hash_of(&(ALICE, right_outcome, salt));
        assert_ok!(Court::vote(Origin::signed(ALICE), market_id, new_commitment));
        assert_ne!(wrong_commitment, new_commitment);
        assert_eq!(
            <Draws<Runtime>>::get(market_id)[0].vote,
            Vote::Secret { commitment: new_commitment }
        );
    });
}

#[test]
fn vote_fails_if_court_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let commitment = <Runtime as frame_system::Config>::Hash::default();
        assert_noop!(
            Court::vote(Origin::signed(ALICE), market_id, commitment),
            Error::<Runtime>::CourtNotFound
        );
    });
}

#[test_case(
    Vote::Revealed {
        commitment: <Runtime as frame_system::Config>::Hash::default(),
        outcome: OutcomeReport::Scalar(1u128),
        salt: <Runtime as frame_system::Config>::Hash::default(),
    }; "revealed"
)]
#[test_case(
    Vote::Denounced {
        commitment: <Runtime as frame_system::Config>::Hash::default(),
        outcome: OutcomeReport::Scalar(1u128),
        salt: <Runtime as frame_system::Config>::Hash::default(),
    }; "denounced"
)]
fn vote_fails_if_vote_state_incorrect(vote: crate::Vote<<Runtime as frame_system::Config>::Hash>) {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool();
        let market_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));

        let mut draws = <Draws<Runtime>>::get(market_id);
        assert!(!draws.is_empty());
        draws[0] = Draw { juror: ALICE, weight: 101, vote, slashable: 42u128 };
        <Draws<Runtime>>::insert(market_id, draws);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        let outcome = OutcomeReport::Scalar(42u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome, salt));
        assert_noop!(
            Court::vote(Origin::signed(ALICE), market_id, commitment),
            Error::<Runtime>::InvalidVoteState
        );
    });
}

#[test]
fn vote_fails_if_caller_not_in_draws() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool();
        let market_id = initialize_court();

        let mut draws = <Draws<Runtime>>::get(market_id);
        draws.retain(|draw| draw.juror != ALICE);
        <Draws<Runtime>>::insert(market_id, draws);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        let outcome = OutcomeReport::Scalar(42u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome, salt));
        assert_noop!(
            Court::vote(Origin::signed(ALICE), market_id, commitment),
            Error::<Runtime>::CallerNotInDraws
        );
    });
}

#[test]
fn vote_fails_if_not_in_voting_period() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool();
        let market_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));

        put_alice_in_draw(market_id, amount);

        run_to_block(<RequestBlock<Runtime>>::get() + CourtVotePeriod::get() + 1);

        let outcome = OutcomeReport::Scalar(42u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome, salt));
        assert_noop!(
            Court::vote(Origin::signed(ALICE), market_id, commitment),
            Error::<Runtime>::NotInVotingPeriod
        );
    });
}

#[test]
fn reveal_vote_works() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool();
        let market_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));

        // trick a little bit to let alice be part of the ("random") selection
        let mut draws = <Draws<Runtime>>::get(market_id);
        assert_eq!(draws.len(), 5usize);
        let slashable = MinJurorStake::get();
        let alice_index = 3usize;
        draws[alice_index] = Draw { juror: ALICE, weight: 1, vote: Vote::Drawn, slashable };
        <Draws<Runtime>>::insert(market_id, draws);
        <Jurors<Runtime>>::insert(
            ALICE,
            JurorInfo { stake: amount, active_lock: slashable, prepare_exit_at: None },
        );

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        let outcome = OutcomeReport::Scalar(42u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome.clone(), salt));
        assert_ok!(Court::vote(Origin::signed(ALICE), market_id, commitment));

        let old_draws = <Draws<Runtime>>::get(market_id);

        run_blocks(CourtVotePeriod::get() + 1);

        assert_ok!(Court::reveal_vote(Origin::signed(ALICE), market_id, outcome.clone(), salt,));
        System::assert_last_event(
            Event::JurorRevealedVote { juror: ALICE, market_id, outcome: outcome.clone(), salt }
                .into(),
        );

        let new_draws = <Draws<Runtime>>::get(market_id);
        for (i, (old_draw, new_draw)) in old_draws.iter().zip(new_draws.iter()).enumerate() {
            if i == alice_index {
                continue;
            }
            assert_eq!(old_draw, new_draw);
        }
        assert_eq!(old_draws[alice_index].juror, new_draws[alice_index].juror);
        assert_eq!(old_draws[alice_index].weight, new_draws[alice_index].weight);
        assert_eq!(old_draws[alice_index].slashable, new_draws[alice_index].slashable);
        assert_eq!(old_draws[alice_index].vote, Vote::Secret { commitment });
        assert_eq!(new_draws[alice_index].vote, Vote::Revealed { commitment, outcome, salt });
    });
}

#[test]
fn reveal_vote_fails_if_caller_not_juror() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());

        run_blocks(CourtVotePeriod::get() + 1);

        <Jurors<Runtime>>::remove(ALICE);

        assert_noop!(
            Court::reveal_vote(Origin::signed(ALICE), market_id, outcome, salt),
            Error::<Runtime>::OnlyJurorsCanReveal
        );
    });
}

#[test]
fn reveal_vote_fails_if_court_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());
        run_blocks(CourtVotePeriod::get() + 1);

        <Courts<Runtime>>::remove(market_id);

        assert_noop!(
            Court::reveal_vote(Origin::signed(ALICE), market_id, outcome, salt),
            Error::<Runtime>::CourtNotFound
        );
    });
}

#[test]
fn reveal_vote_fails_if_not_in_aggregation_period() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        assert_noop!(
            Court::reveal_vote(Origin::signed(ALICE), market_id, outcome, salt),
            Error::<Runtime>::NotInAggregationPeriod
        );
    });
}

#[test]
fn reveal_vote_fails_if_juror_not_drawn() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());

        run_blocks(CourtVotePeriod::get() + 1);

        <Draws<Runtime>>::mutate(market_id, |draws| {
            draws.retain(|draw| draw.juror != ALICE);
        });

        assert_noop!(
            Court::reveal_vote(Origin::signed(ALICE), market_id, outcome, salt),
            Error::<Runtime>::JurorNotDrawn
        );
    });
}

#[test]
fn reveal_vote_fails_for_invalid_reveal() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome);

        run_blocks(CourtVotePeriod::get() + 1);

        let invalid_outcome = OutcomeReport::Scalar(43u128);
        assert_noop!(
            Court::reveal_vote(Origin::signed(ALICE), market_id, invalid_outcome, salt),
            Error::<Runtime>::InvalidReveal
        );
    });
}

#[test]
fn reveal_vote_fails_for_invalid_salt() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, correct_salt) = set_alice_after_vote(outcome.clone());

        run_blocks(CourtVotePeriod::get() + 1);

        let incorrect_salt: <Runtime as frame_system::Config>::Hash = [42; 32].into();
        assert_ne!(correct_salt, incorrect_salt);
        assert_noop!(
            Court::reveal_vote(Origin::signed(ALICE), market_id, outcome, incorrect_salt),
            Error::<Runtime>::InvalidReveal
        );
    });
}

#[test]
fn reveal_vote_fails_if_juror_not_voted() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());

        run_blocks(CourtVotePeriod::get() + 1);

        <Draws<Runtime>>::mutate(market_id, |draws| {
            draws.iter_mut().for_each(|draw| {
                if draw.juror == ALICE {
                    draw.vote = Vote::Drawn;
                }
            });
        });

        assert_noop!(
            Court::reveal_vote(Origin::signed(ALICE), market_id, outcome, salt),
            Error::<Runtime>::JurorNotVoted
        );
    });
}

#[test]
fn reveal_vote_fails_if_already_revealed() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());

        run_blocks(CourtVotePeriod::get() + 1);

        assert_ok!(Court::reveal_vote(Origin::signed(ALICE), market_id, outcome.clone(), salt));

        assert_noop!(
            Court::reveal_vote(Origin::signed(ALICE), market_id, outcome, salt),
            Error::<Runtime>::VoteAlreadyRevealed
        );
    });
}

#[test]
fn reveal_vote_fails_if_already_denounced() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());

        assert_ok!(Court::denounce_vote(
            Origin::signed(BOB),
            market_id,
            ALICE,
            outcome.clone(),
            salt
        ));

        run_blocks(CourtVotePeriod::get() + 1);

        assert_noop!(
            Court::reveal_vote(Origin::signed(ALICE), market_id, outcome, salt),
            Error::<Runtime>::VoteAlreadyDenounced
        );
    });
}

#[test]
fn denounce_vote_works() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, commitment, salt) = set_alice_after_vote(outcome.clone());

        let old_draws = <Draws<Runtime>>::get(market_id);
        assert!(
            old_draws
                .iter()
                .any(|draw| { draw.juror == ALICE && matches!(draw.vote, Vote::Secret { .. }) })
        );

        let free_alice_before = Balances::free_balance(ALICE);
        let pot_balance_before = Balances::free_balance(&Court::reward_pot(&market_id));

        assert_ok!(Court::denounce_vote(
            Origin::signed(BOB),
            market_id,
            ALICE,
            outcome.clone(),
            salt
        ));
        System::assert_last_event(
            Event::DenouncedJurorVote {
                denouncer: BOB,
                juror: ALICE,
                market_id,
                outcome: outcome.clone(),
                salt,
            }
            .into(),
        );

        let new_draws = <Draws<Runtime>>::get(market_id);
        assert_eq!(old_draws[1..], new_draws[1..]);
        assert_eq!(old_draws[0].juror, ALICE);
        assert_eq!(old_draws[0].juror, new_draws[0].juror);
        assert_eq!(old_draws[0].weight, new_draws[0].weight);
        assert_eq!(old_draws[0].slashable, new_draws[0].slashable);
        assert_eq!(old_draws[0].vote, Vote::Secret { commitment });
        assert_eq!(new_draws[0].vote, Vote::Denounced { commitment, outcome, salt });

        let free_alice_after = Balances::free_balance(ALICE);
        let slash = old_draws[0].slashable;
        assert!(!slash.is_zero());
        // slash happens in `reassign_juror_stakes`
        // see `reassign_juror_stakes_slashes_tardy_jurors_and_rewards_winners`
        assert_eq!(free_alice_after, free_alice_before);

        let pot_balance_after = Balances::free_balance(&Court::reward_pot(&market_id));
        assert_eq!(pot_balance_after, pot_balance_before);
    });
}

#[test]
fn denounce_vote_fails_if_self_denounce() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());

        assert_noop!(
            Court::denounce_vote(Origin::signed(ALICE), market_id, ALICE, outcome, salt),
            Error::<Runtime>::SelfDenounceDisallowed
        );
    });
}

#[test]
fn denounce_vote_fails_if_juror_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());

        <Jurors<Runtime>>::remove(ALICE);

        assert_noop!(
            Court::denounce_vote(Origin::signed(BOB), market_id, ALICE, outcome, salt),
            Error::<Runtime>::JurorDoesNotExist
        );
    });
}

#[test]
fn denounce_vote_fails_if_court_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());

        <Courts<Runtime>>::remove(market_id);

        assert_noop!(
            Court::denounce_vote(Origin::signed(BOB), market_id, ALICE, outcome, salt),
            Error::<Runtime>::CourtNotFound
        );
    });
}

#[test]
fn denounce_vote_fails_if_not_in_voting_period() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());

        run_blocks(CourtVotePeriod::get() + 1);

        assert_noop!(
            Court::denounce_vote(Origin::signed(BOB), market_id, ALICE, outcome, salt),
            Error::<Runtime>::NotInVotingPeriod
        );
    });
}

#[test]
fn denounce_vote_fails_if_juror_not_drawn() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());

        <Draws<Runtime>>::mutate(market_id, |draws| {
            draws.retain(|draw| draw.juror != ALICE);
        });

        assert_noop!(
            Court::denounce_vote(Origin::signed(BOB), market_id, ALICE, outcome, salt),
            Error::<Runtime>::JurorNotDrawn
        );
    });
}

#[test]
fn denounce_vote_fails_if_invalid_reveal() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome);

        let invalid_outcome = OutcomeReport::Scalar(69u128);
        assert_noop!(
            Court::denounce_vote(Origin::signed(BOB), market_id, ALICE, invalid_outcome, salt),
            Error::<Runtime>::InvalidReveal
        );
    });
}

#[test]
fn denounce_vote_fails_if_juror_not_voted() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());

        <Draws<Runtime>>::mutate(market_id, |draws| {
            draws.iter_mut().for_each(|draw| {
                if draw.juror == ALICE {
                    draw.vote = Vote::Drawn;
                }
            });
        });

        assert_noop!(
            Court::denounce_vote(Origin::signed(BOB), market_id, ALICE, outcome, salt),
            Error::<Runtime>::JurorNotVoted
        );
    });
}

#[test]
fn denounce_vote_fails_if_vote_already_revealed() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());

        run_blocks(CourtVotePeriod::get() + 1);

        assert_ok!(Court::reveal_vote(Origin::signed(ALICE), market_id, outcome.clone(), salt));

        assert_noop!(
            Court::reveal_vote(Origin::signed(ALICE), market_id, outcome, salt),
            Error::<Runtime>::VoteAlreadyRevealed
        );
    });
}

#[test]
fn denounce_vote_fails_if_vote_already_denounced() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, salt) = set_alice_after_vote(outcome.clone());

        assert_ok!(Court::denounce_vote(
            Origin::signed(BOB),
            market_id,
            ALICE,
            outcome.clone(),
            salt
        ));

        assert_noop!(
            Court::denounce_vote(Origin::signed(CHARLIE), market_id, ALICE, outcome, salt),
            Error::<Runtime>::VoteAlreadyDenounced
        );
    });
}

#[test]
fn appeal_updates_cycle_ends() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        let last_court = <Courts<Runtime>>::get(market_id).unwrap();

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(Origin::signed(CHARLIE), market_id));

        let now = <frame_system::Pallet<Runtime>>::block_number();
        let court = <Courts<Runtime>>::get(market_id).unwrap();

        let request_block = <RequestBlock<Runtime>>::get();
        assert!(now < request_block);
        assert_eq!(court.cycle_ends.pre_vote, request_block);
        assert_eq!(court.cycle_ends.vote, request_block + CourtVotePeriod::get());
        assert_eq!(
            court.cycle_ends.aggregation,
            request_block + CourtVotePeriod::get() + CourtAggregationPeriod::get()
        );
        assert_eq!(
            court.cycle_ends.appeal,
            request_block
                + CourtVotePeriod::get()
                + CourtAggregationPeriod::get()
                + CourtAppealPeriod::get()
        );

        assert!(last_court.cycle_ends.pre_vote < court.cycle_ends.pre_vote);
        assert!(last_court.cycle_ends.vote < court.cycle_ends.vote);
        assert!(last_court.cycle_ends.aggregation < court.cycle_ends.aggregation);
        assert!(last_court.cycle_ends.appeal < court.cycle_ends.appeal);
    });
}

#[test]
fn appeal_reserves_get_appeal_bond() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        let free_charlie_before = Balances::free_balance(CHARLIE);
        assert_ok!(Court::appeal(Origin::signed(CHARLIE), market_id));

        let free_charlie_after = Balances::free_balance(CHARLIE);
        let bond = crate::get_appeal_bond::<Runtime>(1usize);
        assert!(!bond.is_zero());
        assert_eq!(free_charlie_after, free_charlie_before - bond);
        assert_eq!(Balances::reserved_balance(CHARLIE), bond);
    });
}

#[test]
fn appeal_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(Origin::signed(CHARLIE), market_id));

        System::assert_last_event(Event::MarketAppealed { market_id, appeal_number: 1u32 }.into());
    });
}

#[test]
fn appeal_shifts_auto_resolve() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        let resolve_at_0 = <Courts<Runtime>>::get(market_id).unwrap().cycle_ends.appeal;
        assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at_0), vec![0]);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(Origin::signed(CHARLIE), market_id));

        let resolve_at_1 = <Courts<Runtime>>::get(market_id).unwrap().cycle_ends.appeal;
        assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at_1), vec![0]);
        assert_ne!(resolve_at_0, resolve_at_1);
        assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at_0), vec![]);
    });
}

#[test]
fn appeal_overrides_last_draws() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        let last_draws = <Draws<Runtime>>::get(market_id);
        assert!(!last_draws.len().is_zero());

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(Origin::signed(CHARLIE), market_id));

        let draws = <Draws<Runtime>>::get(market_id);
        assert_ne!(draws, last_draws);
    });
}

#[test]
fn appeal_draws_total_weight_is_correct() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        let last_draws = <Draws<Runtime>>::get(market_id);
        let last_draws_total_weight = last_draws.iter().map(|draw| draw.weight).sum::<u32>();
        assert_eq!(last_draws_total_weight, Court::necessary_jurors_weight(0usize) as u32);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(Origin::signed(CHARLIE), market_id));

        let neccessary_juror_weight = Court::necessary_jurors_weight(1usize) as u32;
        let draws = <Draws<Runtime>>::get(market_id);
        let draws_total_weight = draws.iter().map(|draw| draw.weight).sum::<u32>();
        assert_eq!(draws_total_weight, neccessary_juror_weight);
    });
}

#[test]
fn appeal_get_latest_resolved_outcome_changes() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(Origin::signed(CHARLIE), market_id));

        let last_appealed_outcome = <Courts<Runtime>>::get(market_id)
            .unwrap()
            .appeals
            .last()
            .unwrap()
            .appealed_outcome
            .clone();

        let request_block = <RequestBlock<Runtime>>::get();
        run_to_block(request_block + 1);
        let outcome = OutcomeReport::Scalar(69u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome.clone(), salt));

        // cheat a little to get alice in the draw for the new appeal
        put_alice_in_draw(market_id, MinJurorStake::get());
        assert_ok!(Court::vote(Origin::signed(ALICE), market_id, commitment));

        run_blocks(CourtVotePeriod::get() + 1);

        assert_ok!(Court::reveal_vote(Origin::signed(ALICE), market_id, outcome.clone(), salt));

        run_blocks(CourtAggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(Origin::signed(CHARLIE), market_id));

        let new_appealed_outcome = <Courts<Runtime>>::get(market_id)
            .unwrap()
            .appeals
            .last()
            .unwrap()
            .appealed_outcome
            .clone();

        // if the new appealed outcome were the last appealed outcome,
        // then the wrong appealed outcome was added in `appeal`
        assert_eq!(new_appealed_outcome, outcome);
        assert_ne!(last_appealed_outcome, new_appealed_outcome);
    });
}

#[test]
fn appeal_fails_if_court_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(Court::appeal(Origin::signed(CHARLIE), 0), Error::<Runtime>::CourtNotFound);
    });
}

#[test]
fn appeal_fails_if_appeal_bond_exceeds_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        assert_noop!(
            Court::appeal(Origin::signed(POOR_PAUL), market_id),
            Error::<Runtime>::AppealBondExceedsBalance
        );
    });
}

#[test]
fn appeal_fails_if_max_appeals_reached() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        fill_appeals(&market_id, MaxAppeals::get() as usize);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        assert_noop!(
            Court::appeal(Origin::signed(CHARLIE), market_id),
            Error::<Runtime>::MaxAppealsReached
        );
    });
}

#[test]
fn check_appealable_market_fails_if_market_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let now = <frame_system::Pallet<Runtime>>::block_number();
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        let court = <Courts<Runtime>>::get(market_id).unwrap();
        MarketCommons::remove_market(&market_id).unwrap();

        assert_noop!(
            Court::check_appealable_market(&0, &court, now),
            MError::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn check_appealable_market_fails_if_dispute_mechanism_wrong() {
    ExtBuilder::default().build().execute_with(|| {
        let now = <frame_system::Pallet<Runtime>>::block_number();
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        let court = <Courts<Runtime>>::get(market_id).unwrap();

        MarketCommons::mutate_market(&market_id, |market| {
            market.dispute_mechanism = MarketDisputeMechanism::SimpleDisputes;
            Ok(())
        })
        .unwrap();

        assert_noop!(
            Court::check_appealable_market(&0, &court, now),
            Error::<Runtime>::MarketDoesNotHaveCourtMechanism
        );
    });
}

#[test]
fn check_appealable_market_fails_if_not_in_appeal_period() {
    ExtBuilder::default().build().execute_with(|| {
        let now = <frame_system::Pallet<Runtime>>::block_number();
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(CourtVotePeriod::get());

        let court = <Courts<Runtime>>::get(market_id).unwrap();

        assert_noop!(
            Court::check_appealable_market(&0, &court, now),
            Error::<Runtime>::NotInAppealPeriod
        );
    });
}

#[test]
fn appeal_last_appeal_just_removes_auto_resolve() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        fill_appeals(&market_id, (MaxAppeals::get() - 1) as usize);

        let court = <Courts<Runtime>>::get(market_id).unwrap();
        let resolve_at = court.cycle_ends.appeal;
        assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at), vec![market_id]);

        assert_ok!(Court::appeal(Origin::signed(CHARLIE), market_id));

        assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at), vec![]);
    });
}

#[test]
fn appeal_adds_last_appeal() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        fill_appeals(&market_id, (MaxAppeals::get() - 1) as usize);

        let last_draws = <Draws<Runtime>>::get(market_id);
        let appealed_outcome =
            Court::get_latest_resolved_outcome(&market_id, last_draws.as_slice()).unwrap();

        assert_ok!(Court::appeal(Origin::signed(CHARLIE), market_id));

        let court = <Courts<Runtime>>::get(market_id).unwrap();
        assert!(court.appeals.is_full());

        let last_appeal = court.appeals.last().unwrap();
        assert_eq!(last_appeal.appealed_outcome, appealed_outcome);
    });
}

#[test]
fn reassign_juror_stakes_slashes_tardy_jurors_and_rewards_winners() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool();
        let market_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert_ok!(Court::join_court(Origin::signed(BOB), amount));
        assert_ok!(Court::join_court(Origin::signed(CHARLIE), amount));
        assert_ok!(Court::join_court(Origin::signed(DAVE), amount));
        assert_ok!(Court::join_court(Origin::signed(EVE), amount));

        let outcome = OutcomeReport::Scalar(42u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome.clone(), salt));

        let draws: crate::DrawsOf<Runtime> = vec![
            Draw { juror: ALICE, weight: 1, vote: Vote::Drawn, slashable: MinJurorStake::get() },
            Draw {
                juror: BOB,
                weight: 1,
                vote: Vote::Secret { commitment },
                slashable: 2 * MinJurorStake::get(),
            },
            Draw {
                juror: CHARLIE,
                weight: 1,
                vote: Vote::Revealed { commitment, outcome: outcome.clone(), salt },
                slashable: 3 * MinJurorStake::get(),
            },
            Draw { juror: DAVE, weight: 1, vote: Vote::Drawn, slashable: 4 * MinJurorStake::get() },
            Draw {
                juror: EVE,
                weight: 1,
                vote: Vote::Denounced { commitment, outcome, salt },
                slashable: 5 * MinJurorStake::get(),
            },
        ]
        .try_into()
        .unwrap();
        let old_draws = draws.clone();
        <Draws<Runtime>>::insert(market_id, draws);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        run_blocks(
            CourtVotePeriod::get() + CourtAggregationPeriod::get() + CourtAppealPeriod::get() + 1,
        );

        let market = MarketCommons::market(&market_id).unwrap();
        let _ = Court::on_resolution(&market_id, &market).unwrap();

        let free_alice_before = Balances::free_balance(&ALICE);
        let free_bob_before = Balances::free_balance(&BOB);
        let free_charlie_before = Balances::free_balance(&CHARLIE);
        let free_dave_before = Balances::free_balance(&DAVE);
        let free_eve_before = Balances::free_balance(&EVE);

        assert_ok!(Court::reassign_juror_stakes(Origin::signed(EVE), market_id));

        let free_alice_after = Balances::free_balance(&ALICE);
        assert_ne!(free_alice_after, free_alice_before);
        assert_eq!(free_alice_after, free_alice_before - old_draws[ALICE as usize].slashable);

        let free_bob_after = Balances::free_balance(&BOB);
        assert_ne!(free_bob_after, free_bob_before);
        assert_eq!(free_bob_after, free_bob_before - old_draws[BOB as usize].slashable);

        let free_charlie_after = Balances::free_balance(&CHARLIE);
        let full_slashes = old_draws[ALICE as usize].slashable
            + old_draws[BOB as usize].slashable
            + old_draws[DAVE as usize].slashable
            + old_draws[EVE as usize].slashable;
        assert_eq!(free_charlie_after, free_charlie_before + full_slashes);

        let free_dave_after = Balances::free_balance(&DAVE);
        assert_ne!(free_dave_after, free_dave_before);
        assert_eq!(free_dave_after, free_dave_before - old_draws[DAVE as usize].slashable);

        let free_eve_after = Balances::free_balance(&EVE);
        assert_ne!(free_eve_after, free_eve_before);
        assert_eq!(free_eve_after, free_eve_before - old_draws[EVE as usize].slashable);
    });
}

#[test]
fn reassign_juror_stakes_fails_if_court_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Court::reassign_juror_stakes(Origin::signed(EVE), 0),
            Error::<Runtime>::CourtNotFound
        );
    });
}

#[test]
fn reassign_juror_stakes_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        let market = MarketCommons::market(&market_id).unwrap();
        let _ = Court::on_resolution(&market_id, &market).unwrap().unwrap();

        assert_ok!(Court::reassign_juror_stakes(Origin::signed(EVE), market_id));
        System::assert_last_event(Event::JurorStakesReassigned { market_id }.into());
    });
}

#[test]
fn reassign_juror_stakes_fails_if_juror_stakes_already_reassigned() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        let market = MarketCommons::market(&market_id).unwrap();
        let _ = Court::on_resolution(&market_id, &market).unwrap().unwrap();

        assert_ok!(Court::reassign_juror_stakes(Origin::signed(EVE), market_id));

        assert_noop!(
            Court::reassign_juror_stakes(Origin::signed(EVE), market_id),
            Error::<Runtime>::CourtAlreadyReassigned
        );
    });
}

#[test]
fn reassign_juror_stakes_updates_court_status() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        let market = MarketCommons::market(&market_id).unwrap();
        let resolution_outcome = Court::on_resolution(&market_id, &market).unwrap().unwrap();

        let court = <Courts<Runtime>>::get(market_id).unwrap();
        assert_eq!(court.status, CourtStatus::Closed { winner: resolution_outcome });

        assert_ok!(Court::reassign_juror_stakes(Origin::signed(EVE), market_id));

        let court = <Courts<Runtime>>::get(market_id).unwrap();
        assert_eq!(court.status, CourtStatus::Reassigned);
    });
}

#[test]
fn reassign_juror_stakes_removes_draws() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        let market = MarketCommons::market(&market_id).unwrap();
        let _ = Court::on_resolution(&market_id, &market).unwrap().unwrap();

        let draws = <Draws<Runtime>>::get(market_id);
        assert!(!draws.is_empty());

        assert_ok!(Court::reassign_juror_stakes(Origin::signed(EVE), market_id));

        let draws = <Draws<Runtime>>::get(market_id);
        assert!(draws.is_empty());
    });
}

#[test]
fn reassign_juror_stakes_fails_if_court_not_closed() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        assert_noop!(
            Court::reassign_juror_stakes(Origin::signed(EVE), market_id),
            Error::<Runtime>::CourtNotClosed
        );
    });
}

#[test]
fn reassign_juror_stakes_decreases_active_lock() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool();
        let market_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert_ok!(Court::join_court(Origin::signed(BOB), amount));
        assert_ok!(Court::join_court(Origin::signed(CHARLIE), amount));
        assert_ok!(Court::join_court(Origin::signed(DAVE), amount));

        let outcome = OutcomeReport::Scalar(42u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome.clone(), salt));

        let alice_slashable = MinJurorStake::get();
        <Jurors<Runtime>>::mutate(ALICE, |juror_info| {
            if let Some(ref mut info) = juror_info {
                info.active_lock = alice_slashable;
            }
        });
        let bob_slashable = 2 * MinJurorStake::get();
        <Jurors<Runtime>>::mutate(BOB, |juror_info| {
            if let Some(ref mut juror_info) = juror_info {
                juror_info.active_lock = bob_slashable;
            }
        });
        let charlie_slashable = 3 * MinJurorStake::get();
        <Jurors<Runtime>>::mutate(CHARLIE, |juror_info| {
            if let Some(ref mut juror_info) = juror_info {
                juror_info.active_lock = charlie_slashable;
            }
        });
        let dave_slashable = 4 * MinJurorStake::get();
        <Jurors<Runtime>>::mutate(DAVE, |juror_info| {
            if let Some(ref mut juror_info) = juror_info {
                juror_info.active_lock = dave_slashable;
            }
        });

        let draws: crate::DrawsOf<Runtime> = vec![
            Draw { juror: ALICE, weight: 1, vote: Vote::Drawn, slashable: alice_slashable },
            Draw {
                juror: BOB,
                weight: 1,
                vote: Vote::Secret { commitment },
                slashable: bob_slashable,
            },
            Draw {
                juror: CHARLIE,
                weight: 1,
                vote: Vote::Revealed { commitment, outcome: outcome.clone(), salt },
                slashable: charlie_slashable,
            },
            Draw {
                juror: DAVE,
                weight: 1,
                vote: Vote::Denounced { commitment, outcome, salt },
                slashable: dave_slashable,
            },
        ]
        .try_into()
        .unwrap();
        <Draws<Runtime>>::insert(market_id, draws);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        run_blocks(
            CourtVotePeriod::get() + CourtAggregationPeriod::get() + CourtAppealPeriod::get() + 1,
        );

        let market = MarketCommons::market(&market_id).unwrap();
        let _ = Court::on_resolution(&market_id, &market).unwrap();

        assert_ok!(Court::reassign_juror_stakes(Origin::signed(EVE), market_id));
        assert!(<Jurors<Runtime>>::get(ALICE).unwrap().active_lock.is_zero());
        assert!(<Jurors<Runtime>>::get(BOB).unwrap().active_lock.is_zero());
        assert!(<Jurors<Runtime>>::get(CHARLIE).unwrap().active_lock.is_zero());
        assert!(<Jurors<Runtime>>::get(DAVE).unwrap().active_lock.is_zero());
    });
}

#[test]
fn reassign_juror_stakes_slashes_loosers_and_awards_winners() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool();
        let market_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert_ok!(Court::join_court(Origin::signed(BOB), amount));
        assert_ok!(Court::join_court(Origin::signed(CHARLIE), amount));
        assert_ok!(Court::join_court(Origin::signed(DAVE), amount));

        let outcome = OutcomeReport::Scalar(42u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome.clone(), salt));

        let wrong_outcome_0 = OutcomeReport::Scalar(69u128);
        let wrong_outcome_1 = OutcomeReport::Scalar(56u128);

        let draws: crate::DrawsOf<Runtime> = vec![
            Draw {
                juror: ALICE,
                weight: 1,
                vote: Vote::Revealed { commitment, outcome: outcome.clone(), salt },
                slashable: MinJurorStake::get(),
            },
            Draw {
                juror: BOB,
                weight: 1,
                vote: Vote::Revealed { commitment, outcome: wrong_outcome_0, salt },
                slashable: 2 * MinJurorStake::get(),
            },
            Draw {
                juror: CHARLIE,
                weight: 1,
                vote: Vote::Revealed { commitment, outcome: outcome.clone(), salt },
                slashable: 3 * MinJurorStake::get(),
            },
            Draw {
                juror: DAVE,
                weight: 1,
                vote: Vote::Revealed { commitment, outcome: wrong_outcome_1, salt },
                slashable: 4 * MinJurorStake::get(),
            },
        ]
        .try_into()
        .unwrap();
        let last_draws = draws.clone();
        <Draws<Runtime>>::insert(market_id, draws);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        run_blocks(
            CourtVotePeriod::get() + CourtAggregationPeriod::get() + CourtAppealPeriod::get() + 1,
        );

        let market = MarketCommons::market(&market_id).unwrap();
        let resolution_outcome = Court::on_resolution(&market_id, &market).unwrap().unwrap();
        assert_eq!(resolution_outcome, outcome);

        let free_alice_before = Balances::free_balance(ALICE);
        let free_bob_before = Balances::free_balance(BOB);
        let free_charlie_before = Balances::free_balance(CHARLIE);
        let free_dave_before = Balances::free_balance(DAVE);

        let reward_pot = Court::reward_pot(&market_id);
        let tardy_or_denounced_value = 5 * MinJurorStake::get();
        let _ = Balances::deposit(&reward_pot, tardy_or_denounced_value).unwrap();

        assert_ok!(Court::reassign_juror_stakes(Origin::signed(EVE), market_id));

        let bob_slashed = last_draws[BOB as usize].slashable;
        let dave_slashed = last_draws[DAVE as usize].slashable;
        let slashed = bob_slashed + dave_slashed + tardy_or_denounced_value;
        let free_alice_after = Balances::free_balance(ALICE);
        assert_eq!(free_alice_after, free_alice_before + slashed / 2);

        let free_bob_after = Balances::free_balance(BOB);
        assert_eq!(free_bob_after, free_bob_before - bob_slashed);

        let free_charlie_after = Balances::free_balance(CHARLIE);
        assert_eq!(free_charlie_after, free_charlie_before + slashed / 2);

        let free_dave_after = Balances::free_balance(DAVE);
        assert_eq!(free_dave_after, free_dave_before - dave_slashed);

        assert!(Balances::free_balance(&reward_pot).is_zero());
    });
}

#[test]
fn reassign_juror_stakes_rewards_treasury_if_no_winner() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool();
        let market_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert_ok!(Court::join_court(Origin::signed(BOB), amount));
        assert_ok!(Court::join_court(Origin::signed(CHARLIE), amount));
        assert_ok!(Court::join_court(Origin::signed(DAVE), amount));

        let outcome = OutcomeReport::Scalar(42u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome.clone(), salt));

        let wrong_outcome_0 = OutcomeReport::Scalar(69u128);
        let wrong_outcome_1 = OutcomeReport::Scalar(56u128);

        let draws: crate::DrawsOf<Runtime> = vec![
            Draw {
                juror: ALICE,
                weight: 1,
                vote: Vote::Revealed { commitment, outcome: wrong_outcome_1.clone(), salt },
                slashable: MinJurorStake::get(),
            },
            Draw {
                juror: BOB,
                weight: 1,
                vote: Vote::Revealed { commitment, outcome: wrong_outcome_0.clone(), salt },
                slashable: 2 * MinJurorStake::get(),
            },
            Draw {
                juror: CHARLIE,
                weight: 1,
                vote: Vote::Revealed { commitment, outcome: wrong_outcome_0, salt },
                slashable: 3 * MinJurorStake::get(),
            },
            Draw {
                juror: DAVE,
                weight: 1,
                vote: Vote::Revealed { commitment, outcome: wrong_outcome_1, salt },
                slashable: 4 * MinJurorStake::get(),
            },
        ]
        .try_into()
        .unwrap();
        let last_draws = draws.clone();
        <Draws<Runtime>>::insert(market_id, draws);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        run_blocks(
            CourtVotePeriod::get() + CourtAggregationPeriod::get() + CourtAppealPeriod::get() + 1,
        );

        let mut court = <Courts<Runtime>>::get(market_id).unwrap();
        court.status = CourtStatus::Closed { winner: outcome };
        <Courts<Runtime>>::insert(market_id, court);

        let free_alice_before = Balances::free_balance(ALICE);
        let free_bob_before = Balances::free_balance(BOB);
        let free_charlie_before = Balances::free_balance(CHARLIE);
        let free_dave_before = Balances::free_balance(DAVE);

        let treasury_account = Court::treasury_account_id();
        let free_treasury_before = Balances::free_balance(&treasury_account);

        assert_ok!(Court::reassign_juror_stakes(Origin::signed(EVE), market_id));

        let alice_slashed = last_draws[ALICE as usize].slashable;
        let bob_slashed = last_draws[BOB as usize].slashable;
        let charlie_slashed = last_draws[CHARLIE as usize].slashable;
        let dave_slashed = last_draws[DAVE as usize].slashable;

        let slashed = bob_slashed + dave_slashed + alice_slashed + charlie_slashed;

        let free_alice_after = Balances::free_balance(ALICE);
        assert_eq!(free_alice_after, free_alice_before - alice_slashed);

        let free_bob_after = Balances::free_balance(BOB);
        assert_eq!(free_bob_after, free_bob_before - bob_slashed);

        let free_charlie_after = Balances::free_balance(CHARLIE);
        assert_eq!(free_charlie_after, free_charlie_before - charlie_slashed);

        let free_dave_after = Balances::free_balance(DAVE);
        assert_eq!(free_dave_after, free_dave_before - dave_slashed);

        assert_eq!(Balances::free_balance(&treasury_account), free_treasury_before + slashed);
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
fn on_resolution_sets_court_status() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.report.as_ref().unwrap().outcome, ORACLE_REPORT);

        assert_eq!(Court::on_resolution(&market_id, &market), Ok(Some(ORACLE_REPORT)));
        let court = <Courts<Runtime>>::get(market_id).unwrap();
        assert_eq!(court.status, CourtStatus::Closed { winner: ORACLE_REPORT });
    });
}

#[test]
fn on_resolution_fails_if_court_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = MarketCommons::push_market(DEFAULT_MARKET).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();

        assert_noop!(Court::on_resolution(&market_id, &market), Error::<Runtime>::CourtNotFound);
    });
}

#[test]
fn on_resolution_denies_non_court_markets() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.dispute_mechanism = MarketDisputeMechanism::SimpleDisputes;
        assert_noop!(
            Court::on_resolution(&0, &market),
            Error::<Runtime>::MarketDoesNotHaveCourtMechanism
        );
    });
}

#[test]
fn exchange_fails_if_non_court_markets() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.dispute_mechanism = MarketDisputeMechanism::SimpleDisputes;
        assert_noop!(
            Court::exchange(&0, &market, &ORACLE_REPORT, NegativeImbalance::<Runtime>::zero()),
            Error::<Runtime>::MarketDoesNotHaveCourtMechanism
        );
    });
}

#[test]
fn exchange_slashes_unjustified_and_unreserves_justified_appealers() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let market = MarketCommons::market(&market_id).unwrap();

        let resolved_outcome = OutcomeReport::Scalar(1);
        let other_outcome = OutcomeReport::Scalar(2);

        let mut court = <Courts<Runtime>>::get(market_id).unwrap();
        let mut free_balances_before = BTreeMap::new();
        let mut number = 0u128;
        let mut slashed_bonds = <BalanceOf<Runtime>>::zero();
        while (number as usize) < MaxAppeals::get() as usize {
            let bond = crate::get_appeal_bond::<Runtime>(court.appeals.len());
            let appealed_outcome = if number % 2 == 0 {
                // The appeals are not justified,
                // because the appealed outcomes are equal to the resolved outcome.
                // it is punished to appeal the right outcome
                slashed_bonds += bond;
                resolved_outcome.clone()
            } else {
                other_outcome.clone()
            };

            let backer = number;
            let _ = Balances::deposit(&backer, bond).unwrap();
            assert_ok!(Balances::reserve_named(&Court::reserve_id(), &backer, bond));
            let free_balance = Balances::free_balance(&backer);
            free_balances_before.insert(backer, free_balance);
            court.appeals.try_push(AppealInfo { backer, bond, appealed_outcome }).unwrap();
            number += 1;
        }
        Courts::<Runtime>::insert(market_id, court);

        let imbalance: NegativeImbalanceOf<Runtime> =
            <pallet_balances::Pallet<Runtime> as Currency<crate::AccountIdOf<Runtime>>>::issue(
                42_000_000_000,
            );
        let prev_balance = imbalance.peek();
        let imb_remainder =
            Court::exchange(&market_id, &market, &resolved_outcome, imbalance).unwrap();
        assert_eq!(imb_remainder.peek(), prev_balance + slashed_bonds);

        let court = <Courts<Runtime>>::get(market_id).unwrap();
        let appeals = court.appeals;
        for AppealInfo { backer, bond, appealed_outcome } in appeals {
            assert_eq!(Balances::reserved_balance_named(&Court::reserve_id(), &backer), 0);
            let free_balance_after = Balances::free_balance(&backer);
            let free_balance_before = free_balances_before.get(&backer).unwrap();

            if appealed_outcome == resolved_outcome {
                assert_eq!(free_balance_after, *free_balance_before);
            } else {
                assert_eq!(free_balance_after, *free_balance_before + bond);
            }
        }
    });
}

#[test]
fn get_auto_resolve_works() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let market = MarketCommons::market(&market_id).unwrap();
        let court = <Courts<Runtime>>::get(market_id).unwrap();
        let appeal_end = court.cycle_ends.appeal;
        assert_eq!(Court::get_auto_resolve(&market_id, &market).unwrap(), Some(appeal_end));
    });
}

#[test]
fn get_auto_resolve_fails_if_wrong_dispute_mechanism() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.dispute_mechanism = MarketDisputeMechanism::SimpleDisputes;
        assert_noop!(
            Court::get_auto_resolve(&0, &market),
            Error::<Runtime>::MarketDoesNotHaveCourtMechanism
        );
    });
}

#[test]
fn on_global_dispute_removes_court() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let market = MarketCommons::market(&market_id).unwrap();
        assert!(<Courts<Runtime>>::contains_key(market_id));
        assert_ok!(Court::on_global_dispute(&market_id, &market));
        assert!(!<Courts<Runtime>>::contains_key(market_id));
    });
}

#[test]
fn on_global_dispute_removes_draws() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let market = MarketCommons::market(&market_id).unwrap();
        assert!(<Draws<Runtime>>::contains_key(market_id));
        assert_ok!(Court::on_global_dispute(&market_id, &market));
        assert!(!<Draws<Runtime>>::contains_key(market_id));
    });
}

#[test]
fn on_global_dispute_fails_if_wrong_dispute_mechanism() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.dispute_mechanism = MarketDisputeMechanism::SimpleDisputes;
        assert_noop!(
            Court::on_global_dispute(&0, &market),
            Error::<Runtime>::MarketDoesNotHaveCourtMechanism
        );
    });
}

#[test]
fn on_global_dispute_fails_if_court_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market = DEFAULT_MARKET;
        assert_noop!(Court::on_global_dispute(&0, &market), Error::<Runtime>::CourtNotFound);
    });
}

#[test]
fn on_global_dispute_fails_if_market_report_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        MarketCommons::mutate_market(&market_id, |market| {
            market.report = None;
            Ok(())
        })
        .unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        assert_noop!(
            Court::on_global_dispute(&market_id, &market),
            Error::<Runtime>::MarketReportNotFound
        );
    });
}

#[test]
fn on_global_dispute_returns_appealed_outcomes() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let market = MarketCommons::market(&market_id).unwrap();
        let mut court = <Courts<Runtime>>::get(market_id).unwrap();
        let mut gd_outcomes = Vec::new();

        let initial_vote_amount = <BalanceOf<Runtime>>::zero();
        let treasury_account = Court::treasury_account_id();
        for number in 0..MaxAppeals::get() {
            let appealed_outcome = OutcomeReport::Scalar(number as u128);
            let backer = number as u128;
            let bond = crate::get_appeal_bond::<Runtime>(court.appeals.len());
            gd_outcomes.push((appealed_outcome.clone(), treasury_account, initial_vote_amount));
            court.appeals.try_push(AppealInfo { backer, bond, appealed_outcome }).unwrap();
        }
        Courts::<Runtime>::insert(market_id, court);
        assert_eq!(Court::on_global_dispute(&market_id, &market).unwrap(), gd_outcomes);
    });
}

#[test]
fn choose_multiple_weighted_works() {
    ExtBuilder::default().build().execute_with(|| {
        let necessary_jurors_weight = Court::necessary_jurors_weight(5usize);
        for i in 0..necessary_jurors_weight {
            let amount = MinJurorStake::get() + i as u128;
            let juror = i as u128;
            let _ = Balances::deposit(&juror, amount).unwrap();
            assert_ok!(Court::join_court(Origin::signed(juror), amount));
        }
        let mut jurors = JurorPool::<Runtime>::get();
        let random_jurors =
            Court::choose_multiple_weighted(&mut jurors, necessary_jurors_weight).unwrap();
        assert_eq!(
            random_jurors.iter().map(|draw| draw.weight).sum::<u32>() as usize,
            necessary_jurors_weight
        );
    });
}

#[test]
fn select_jurors_updates_juror_consumed_stake() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        fill_juror_pool();
        // the last appeal is reserved for global dispute backing
        let appeal_number = (MaxAppeals::get() - 1) as usize;
        fill_appeals(&market_id, appeal_number);

        let jurors = JurorPool::<Runtime>::get();
        let consumed_stake_before = jurors.iter().map(|juror| juror.consumed_stake).sum::<u128>();

        let new_draws = Court::select_jurors(appeal_number).unwrap();

        let total_draw_slashable = new_draws.iter().map(|draw| draw.slashable).sum::<u128>();
        let jurors = JurorPool::<Runtime>::get();
        let consumed_stake_after = jurors.iter().map(|juror| juror.consumed_stake).sum::<u128>();
        assert_ne!(consumed_stake_before, consumed_stake_after);
        assert_eq!(consumed_stake_before + total_draw_slashable, consumed_stake_after);
    });
}

#[test_case(0usize; "first")]
#[test_case(1usize; "second")]
#[test_case(2usize; "third")]
#[test_case(3usize; "fourth")]
fn select_jurors_fails_if_not_enough_jurors(appeal_number: usize) {
    ExtBuilder::default().build().execute_with(|| {
        let necessary_jurors_weight = Court::necessary_jurors_weight(appeal_number);
        for i in 0..(necessary_jurors_weight - 1usize) {
            let amount = MinJurorStake::get() + i as u128;
            let juror = (i + 1000) as u128;
            let _ = Balances::deposit(&juror, amount).unwrap();
            assert_ok!(Court::join_court(Origin::signed(juror), amount));
        }

        assert_noop!(Court::select_jurors(appeal_number), Error::<Runtime>::NotEnoughJurors);
    });
}

#[test]
fn appeal_reduces_active_lock_from_old_draws() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (market_id, _, _) = set_alice_after_vote(outcome);

        let old_draws = <Draws<Runtime>>::get(market_id);
        assert!(!old_draws.is_empty());
        old_draws.iter().for_each(|draw| {
            let juror = draw.juror;
            let juror_info = <Jurors<Runtime>>::get(juror).unwrap();
            assert_ne!(draw.slashable, 0);
            assert_eq!(juror_info.active_lock, draw.slashable);
        });

        run_blocks(CourtVotePeriod::get() + CourtAggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(Origin::signed(CHARLIE), market_id));

        let new_draws = <Draws<Runtime>>::get(market_id);
        old_draws.iter().for_each(|draw| {
            let juror = draw.juror;
            let juror_info = <Jurors<Runtime>>::get(juror).unwrap();
            if let Some(new_draw) = new_draws.iter().find(|new_draw| new_draw.juror == juror) {
                assert_eq!(new_draw.slashable, juror_info.active_lock);
            } else {
                assert_eq!(juror_info.active_lock, 0);
            }
        });
    });
}

#[test]
fn on_dispute_creates_correct_court_info() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let court = <Courts<Runtime>>::get(market_id).unwrap();
        let cycle_ends = court.cycle_ends;
        let request_block = <RequestBlock<Runtime>>::get();
        assert_eq!(cycle_ends.pre_vote, request_block);
        assert_eq!(cycle_ends.vote, cycle_ends.pre_vote + CourtVotePeriod::get());
        assert_eq!(cycle_ends.aggregation, cycle_ends.vote + CourtAggregationPeriod::get());
        assert_eq!(cycle_ends.appeal, cycle_ends.aggregation + CourtAppealPeriod::get());
        assert_eq!(court.status, CourtStatus::Open);
        assert!(court.appeals.is_empty());
    });
}

#[test]
fn on_dispute_fails_if_court_already_exists() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let market = MarketCommons::market(&market_id).unwrap();
        assert_noop!(Court::on_dispute(&market_id, &market), Error::<Runtime>::CourtAlreadyExists);
    });
}

#[test]
fn on_dispute_inserts_draws() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let draws = <Draws<Runtime>>::get(market_id);
        assert_eq!(
            draws[0],
            Draw { juror: ALICE, weight: 1, vote: Vote::Drawn, slashable: MinJurorStake::get() }
        );
        assert_eq!(
            draws[1],
            Draw { juror: BOB, weight: 2, vote: Vote::Drawn, slashable: 2 * MinJurorStake::get() }
        );
        assert_eq!(
            draws[2],
            Draw { juror: DAVE, weight: 1, vote: Vote::Drawn, slashable: MinJurorStake::get() }
        );
        assert_eq!(
            draws[3],
            Draw { juror: EVE, weight: 1, vote: Vote::Drawn, slashable: MinJurorStake::get() }
        );
        assert_eq!(draws.len(), 4usize);
    });
}

#[test]
fn on_dispute_adds_auto_resolve() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let court = <Courts<Runtime>>::get(market_id).unwrap();
        let resolve_at = court.cycle_ends.appeal;
        assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at), vec![market_id]);
    });
}

#[test]
fn has_failed_returns_true_for_appealable_court_too_few_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        // force empty jurors pool
        <JurorPool<Runtime>>::kill();
        let market = MarketCommons::market(&market_id).unwrap();
        let court = <Courts<Runtime>>::get(market_id).unwrap();
        let aggregation = court.cycle_ends.aggregation;
        run_to_block(aggregation + 1);
        assert!(Court::has_failed(&market_id, &market).unwrap());
    });
}

#[test]
fn has_failed_returns_true_for_appealable_court_appeals_full() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let market = MarketCommons::market(&market_id).unwrap();

        fill_appeals(&market_id, MaxAppeals::get() as usize);

        assert!(Court::has_failed(&market_id, &market).unwrap());
    });
}

#[test]
fn has_failed_returns_true_for_uninitialized_court() {
    ExtBuilder::default().build().execute_with(|| {
        // force empty jurors pool
        <JurorPool<Runtime>>::kill();
        let market_id = MarketCommons::push_market(DEFAULT_MARKET).unwrap();
        let report_block = 42;
        MarketCommons::mutate_market(&market_id, |market| {
            market.report = Some(Report { at: report_block, by: BOB, outcome: ORACLE_REPORT });
            Ok(())
        })
        .unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        let block_after_dispute_duration = report_block + market.deadlines.dispute_duration;
        run_to_block(block_after_dispute_duration - 1);
        assert!(Court::has_failed(&market_id, &market).unwrap());
    });
}

#[test]
fn check_necessary_jurors_weight() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Court::necessary_jurors_weight(0usize), 5usize);
        assert_eq!(Court::necessary_jurors_weight(1usize), 11usize);
        assert_eq!(Court::necessary_jurors_weight(2usize), 23usize);
        assert_eq!(Court::necessary_jurors_weight(3usize), 47usize);
    });
}

#[test]
fn check_appeal_bond() {
    ExtBuilder::default().build().execute_with(|| {
        let appeal_bond = AppealBond::get();
        assert_eq!(crate::get_appeal_bond::<Runtime>(0usize), appeal_bond);
        assert_eq!(crate::get_appeal_bond::<Runtime>(1usize), 2 * appeal_bond);
        assert_eq!(crate::get_appeal_bond::<Runtime>(2usize), 4 * appeal_bond);
        assert_eq!(crate::get_appeal_bond::<Runtime>(3usize), 8 * appeal_bond);
    });
}

fn prepare_draws(market_id: &MarketIdOf<Runtime>, outcomes_with_weights: Vec<(u128, u32)>) {
    let mut draws: crate::DrawsOf<Runtime> = vec![].try_into().unwrap();
    for (i, (outcome_index, weight)) in outcomes_with_weights.iter().enumerate() {
        // offset to not conflict with other jurors
        let offset_i = (i + 1000) as u128;
        let juror = offset_i as u128;
        let salt = BlakeTwo256::hash_of(&offset_i);
        let outcome = OutcomeReport::Scalar(*outcome_index);
        let commitment = BlakeTwo256::hash_of(&(juror, outcome.clone(), salt));
        draws
            .try_push(Draw {
                juror,
                weight: *weight,
                vote: Vote::Revealed { commitment, outcome, salt },
                slashable: 0u128,
            })
            .unwrap();
    }
    <Draws<Runtime>>::insert(market_id, draws);
}

#[test]
fn get_winner_works() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let outcomes_and_weights =
            vec![(1000u128, 8), (1001u128, 5), (1002u128, 42), (1003u128, 13)];
        prepare_draws(&market_id, outcomes_and_weights);

        let draws = <Draws<Runtime>>::get(market_id);
        let winner = Court::get_winner(draws.as_slice(), None).unwrap();
        assert_eq!(winner, OutcomeReport::Scalar(1002u128));

        let outcomes_and_weights = vec![(1000u128, 2), (1000u128, 4), (1001u128, 4), (1001u128, 3)];
        prepare_draws(&market_id, outcomes_and_weights);

        let draws = <Draws<Runtime>>::get(market_id);
        let winner = Court::get_winner(draws.as_slice(), None).unwrap();
        assert_eq!(winner, OutcomeReport::Scalar(1001u128));
    });
}

#[test]
fn get_winner_returns_none_for_no_revealed_draws() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let draws = <Draws<Runtime>>::get(market_id);
        let winner = Court::get_winner(draws.as_slice(), None);
        assert_eq!(winner, None);
    });
}

#[test]
fn get_latest_resolved_outcome_selects_last_appealed_outcome_for_tie() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let mut court = <Courts<Runtime>>::get(market_id).unwrap();
        // create a tie of two best outcomes
        let weights = vec![(1000u128, 42), (1001u128, 42)];
        let appealed_outcome = OutcomeReport::Scalar(weights.len() as u128);
        prepare_draws(&market_id, weights);
        court
            .appeals
            .try_push(AppealInfo {
                backer: CHARLIE,
                bond: crate::get_appeal_bond::<Runtime>(1usize),
                appealed_outcome: appealed_outcome.clone(),
            })
            .unwrap();
        <Courts<Runtime>>::insert(market_id, court);

        let draws = <Draws<Runtime>>::get(market_id);
        let latest = Court::get_latest_resolved_outcome(&market_id, draws.as_slice()).unwrap();
        assert_eq!(latest, appealed_outcome);
        assert!(latest != ORACLE_REPORT);
    });
}

#[test]
fn get_latest_resolved_outcome_selects_oracle_report() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.report.unwrap().outcome, ORACLE_REPORT);
        let draws = <Draws<Runtime>>::get(market_id);
        assert_eq!(
            Court::get_latest_resolved_outcome(&market_id, draws.as_slice()).unwrap(),
            ORACLE_REPORT
        );
    });
}

#[test]
fn choose_multiple_weighted_returns_different_jurors_with_other_seed() {
    ExtBuilder::default().build().execute_with(|| {
        run_to_block(123);

        let mut jurors = <JurorPool<Runtime>>::get();
        for pool_item in DEFAULT_SET_OF_JURORS.iter() {
            <Jurors<Runtime>>::insert(
                pool_item.juror,
                JurorInfo { stake: pool_item.stake, active_lock: 0u128, prepare_exit_at: None },
            );
            jurors.try_push(pool_item.clone()).unwrap();
        }

        let nonce_0 = 42u64;
        <crate::JurorsSelectionNonce<Runtime>>::put(nonce_0);
        // randomness is mocked and purely based on the nonce
        // thus a different nonce will result in a different seed (disregarding hash collisions)
        let first_random_seed = Court::get_random_seed(nonce_0);
        let first_random_list = Court::choose_multiple_weighted(&mut jurors, 3).unwrap();

        run_blocks(1);

        let nonce_1 = 69u64;
        <crate::JurorsSelectionNonce<Runtime>>::put(nonce_1);
        let second_random_seed = Court::get_random_seed(nonce_1);

        assert_ne!(first_random_seed, second_random_seed);
        let second_random_list = Court::choose_multiple_weighted(&mut jurors, 3).unwrap();

        // the two lists contain different jurors
        for juror in &first_random_list {
            assert!(second_random_list.iter().all(|el| el != juror));
        }
    });
}

#[test]
fn get_random_seed_returns_equal_seeds_with_equal_nonce() {
    ExtBuilder::default().build().execute_with(|| {
        run_to_block(123);

        // this is useful to check that the random seed only depends on the nonce
        // the same nonce always results in the same seed for testing deterministic
        let nonce = 42u64;
        <crate::JurorsSelectionNonce<Runtime>>::put(nonce);
        let first_random_seed = Court::get_random_seed(nonce);

        run_blocks(1);

        <crate::JurorsSelectionNonce<Runtime>>::put(nonce);
        let second_random_seed = Court::get_random_seed(nonce);

        assert_eq!(first_random_seed, second_random_seed);
    });
}

#[test]
fn random_jurors_returns_a_subset_of_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        run_to_block(123);
        let mut jurors = <JurorPool<Runtime>>::get();
        for pool_item in DEFAULT_SET_OF_JURORS.iter() {
            <Jurors<Runtime>>::insert(
                pool_item.juror,
                JurorInfo { stake: pool_item.stake, active_lock: 0u128, prepare_exit_at: None },
            );
            jurors.try_push(pool_item.clone()).unwrap();
        }

        let random_jurors = Court::choose_multiple_weighted(&mut jurors, 2).unwrap();
        for draw in random_jurors {
            assert!(DEFAULT_SET_OF_JURORS.iter().any(|el| el.juror == draw.juror));
        }
    });
}
