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
    types::{CourtStatus, Draw, Vote},
    AccountIdLookupOf, AppealInfo, Courts, Draws, Error, Event, JurorInfo, JurorInfoOf, JurorPool,
    JurorPoolItem, JurorPoolOf, Jurors, MarketOf, RequestBlock,
};
use frame_support::{assert_noop, assert_ok, traits::fungible::Balanced};
use pallet_balances::BalanceLock;
use rand::seq::SliceRandom;
use sp_runtime::traits::{BlakeTwo256, Hash};
use zeitgeist_primitives::{
    constants::{
        mock::{
            CourtAggregationPeriod, CourtAppealPeriod, CourtLockId, CourtVotePeriod, MaxAppeals,
            MaxJurors, MinJurorStake, RequestInterval,
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
use zrml_market_commons::MarketCommonsPalletApi;

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

fn initialize_court() -> crate::MarketIdOf<Runtime> {
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
fn join_court_successfully_stores_required_data() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        let alice_free_balance_before = Balances::free_balance(ALICE);
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        System::assert_last_event(Event::JurorJoined { juror: ALICE }.into());
        assert_eq!(
            Jurors::<Runtime>::iter().next().unwrap(),
            (ALICE, JurorInfo { stake: amount, active_lock: 0u128 })
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
            vec![(ALICE, JurorInfo { stake: amount, active_lock: 0u128 })]
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
        assert_eq!(
            Jurors::<Runtime>::iter().collect::<Vec<(AccountIdTest, JurorInfoOf<Runtime>)>>(),
            vec![
                (BOB, JurorInfo { stake: amount, active_lock: 0u128 }),
                (ALICE, JurorInfo { stake: amount, active_lock: 0u128 })
            ]
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
        assert_eq!(
            Jurors::<Runtime>::iter().collect::<Vec<(AccountIdTest, JurorInfoOf<Runtime>)>>(),
            vec![
                (BOB, JurorInfo { stake: amount, active_lock: 0u128 }),
                (ALICE, JurorInfo { stake: higher_amount, active_lock: 0u128 })
            ]
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
        Jurors::<Runtime>::insert(ALICE, JurorInfo { stake: amount, active_lock });
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
            vec![JurorPoolItem { stake: amount, juror: ALICE, consumed_stake: 0 }]
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
fn exit_court_works_without_active_lock() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let amount = 2 * min;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert!(!JurorPool::<Runtime>::get().into_inner().is_empty());
        assert_ok!(Court::prepare_exit_court(Origin::signed(ALICE)));
        assert!(JurorPool::<Runtime>::get().into_inner().is_empty());
        assert!(Jurors::<Runtime>::get(ALICE).is_some());

        assert_eq!(Balances::locks(ALICE), vec![the_lock(amount)]);
        let alice_lookup: AccountIdLookupOf<Runtime> = ALICE.into();
        assert_ok!(Court::exit_court(Origin::signed(ALICE), alice_lookup));
        System::assert_last_event(
            Event::JurorExited { juror: ALICE, exit_amount: amount, active_lock: 0u128 }.into(),
        );
        assert!(
            Jurors::<Runtime>::iter()
                .collect::<Vec<(AccountIdTest, JurorInfoOf<Runtime>)>>()
                .is_empty()
        );
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
        assert_ok!(Court::prepare_exit_court(Origin::signed(ALICE)));
        assert!(JurorPool::<Runtime>::get().into_inner().is_empty());

        assert_eq!(
            <Jurors<Runtime>>::get(ALICE).unwrap(),
            JurorInfo { stake: amount, active_lock: 0 }
        );
        // assume that `choose_multiple_weighted` has set the active_lock
        <Jurors<Runtime>>::insert(ALICE, JurorInfo { stake: amount, active_lock });

        assert_eq!(Balances::locks(ALICE), vec![the_lock(amount)]);
        let alice_lookup: AccountIdLookupOf<Runtime> = ALICE.into();
        assert_ok!(Court::exit_court(Origin::signed(ALICE), alice_lookup));
        System::assert_last_event(
            Event::JurorExited { juror: ALICE, exit_amount: amount - active_lock, active_lock }
                .into(),
        );
        assert_eq!(
            Jurors::<Runtime>::get(ALICE).unwrap(),
            JurorInfo { stake: active_lock, active_lock }
        );
        assert_eq!(Balances::locks(ALICE), vec![the_lock(active_lock)]);
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
fn choose_multiple_weighted_works() {
    ExtBuilder::default().build().execute_with(|| {
        let necessary_jurors_weight = Court::necessary_jurors_weight(5usize);
        let mut rng = Court::rng();
        for i in 0..necessary_jurors_weight {
            let amount = MinJurorStake::get() + i as u128;
            let juror = i as u128;
            let _ = Balances::deposit(&juror, amount).unwrap();
            assert_ok!(Court::join_court(Origin::signed(juror), amount));
        }
        let mut jurors = JurorPool::<Runtime>::get();
        let random_jurors =
            Court::choose_multiple_weighted(&mut jurors, necessary_jurors_weight, &mut rng)
                .unwrap();
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
        for i in 0..MaxJurors::get() {
            let amount = MinJurorStake::get() + i as u128;
            let juror = (i + 1000) as u128;
            let _ = Balances::deposit(&juror, amount).unwrap();
            assert_ok!(Court::join_court(Origin::signed(juror), amount));
        }
        // the last appeal is reserved for global dispute backing
        let appeal_number = (MaxAppeals::get() - 1) as usize;
        let mut court = Courts::<Runtime>::get(market_id).unwrap();
        let mut number = 0u128;
        while (number as usize) < appeal_number {
            let appealed_outcome = OutcomeReport::Scalar(number);
            court
                .appeals
                .try_push(AppealInfo {
                    backer: number,
                    bond: crate::default_appeal_bond::<Runtime>(court.appeals.len()),
                    appealed_outcome,
                })
                .unwrap();
            number += 1;
        }
        Courts::<Runtime>::insert(market_id, court);

        let jurors = JurorPool::<Runtime>::get();
        let consumed_stake_before = jurors.iter().map(|juror| juror.consumed_stake).sum::<u128>();

        Court::select_jurors(&market_id, appeal_number).unwrap();

        let draws = <Draws<Runtime>>::get(market_id);
        let total_draw_slashable = draws.iter().map(|draw| draw.slashable).sum::<u128>();
        let jurors = JurorPool::<Runtime>::get();
        let consumed_stake_after = jurors.iter().map(|juror| juror.consumed_stake).sum::<u128>();
        assert_ne!(consumed_stake_before, consumed_stake_after);
        assert_eq!(consumed_stake_before + total_draw_slashable, consumed_stake_after);
    });
}

#[test]
fn on_dispute_creates_correct_court_info() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let court = <Courts<Runtime>>::get(market_id).unwrap();
        let periods = court.periods;
        let request_block = <RequestBlock<Runtime>>::get();
        assert_eq!(periods.pre_vote_end, request_block);
        assert_eq!(periods.vote_end, periods.pre_vote_end + CourtVotePeriod::get());
        assert_eq!(periods.aggregation_end, periods.vote_end + CourtAggregationPeriod::get());
        assert_eq!(periods.appeal_end, periods.aggregation_end + CourtAppealPeriod::get());
        assert_eq!(court.status, CourtStatus::Open);
        assert!(court.appeals.is_empty());
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
        let aggregation_end = court.periods.aggregation_end;
        run_to_block(aggregation_end + 1);
        assert!(Court::has_failed(&market_id, &market).unwrap());
    });
}

#[test]
fn has_failed_returns_true_for_appealable_court_appeals_full() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let market = MarketCommons::market(&market_id).unwrap();
        let mut court = <Courts<Runtime>>::get(market_id).unwrap();
        let mut number = 0u128;
        while !court.appeals.is_full() {
            let appealed_outcome = OutcomeReport::Scalar(number);
            court
                .appeals
                .try_push(AppealInfo {
                    backer: number,
                    bond: crate::default_appeal_bond::<Runtime>(court.appeals.len()),
                    appealed_outcome,
                })
                .unwrap();
            number += 1;
        }
        <Courts<Runtime>>::insert(market_id, court);
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
        assert_eq!(Court::necessary_jurors_weight(4usize), 95usize);
        assert_eq!(Court::necessary_jurors_weight(5usize), 191usize);
    });
}

fn prepare_draws(market_id: &crate::MarketIdOf<Runtime>, outcomes_with_weights: Vec<(u128, u32)>) {
    let mut draws: crate::DrawsOf<Runtime> = vec![].try_into().unwrap();
    for (i, (outcome_index, weight)) in outcomes_with_weights.iter().enumerate() {
        // offset to not conflict with other jurors
        let offset_i = (i + 1000) as u128;
        let juror = offset_i as u128;
        let salt = BlakeTwo256::hash_of(&offset_i);
        let outcome = OutcomeReport::Scalar(*outcome_index);
        let secret = BlakeTwo256::hash_of(&(juror.clone(), outcome.clone(), salt));
        draws
            .try_push(Draw {
                juror,
                weight: *weight,
                vote: Vote::Revealed { secret, outcome, salt },
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
        let winner = Court::get_winner(&draws.as_slice(), None).unwrap();
        assert_eq!(winner, OutcomeReport::Scalar(1002u128));

        let outcomes_and_weights = vec![(1000u128, 2), (1000u128, 4), (1001u128, 4), (1001u128, 3)];
        prepare_draws(&market_id, outcomes_and_weights);

        let draws = <Draws<Runtime>>::get(market_id);
        let winner = Court::get_winner(&draws.as_slice(), None).unwrap();
        assert_eq!(winner, OutcomeReport::Scalar(1001u128));
    });
}

#[test]
fn get_winner_returns_none_for_no_revealed_draws() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let draws = <Draws<Runtime>>::get(market_id);
        let winner = Court::get_winner(&draws.as_slice(), None);
        assert_eq!(winner, None);
    });
}

#[test]
fn get_latest_resolved_outcome_selects_last_appealed_outcome_for_tie() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = initialize_court();
        let mut court = <Courts<Runtime>>::get(&market_id).unwrap();
        // create a tie of two best outcomes
        let weights = vec![(1000u128, 42), (1001u128, 42)];
        let appealed_outcome = OutcomeReport::Scalar(weights.len() as u128);
        prepare_draws(&market_id, weights);
        court
            .appeals
            .try_push(AppealInfo {
                backer: CHARLIE,
                bond: crate::default_appeal_bond::<Runtime>(1usize),
                appealed_outcome: appealed_outcome.clone(),
            })
            .unwrap();
        <Courts<Runtime>>::insert(&market_id, court);

        let latest = Court::get_latest_resolved_outcome(&market_id).unwrap();
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
        assert_eq!(Court::get_latest_resolved_outcome(&market_id).unwrap(), ORACLE_REPORT);
    });
}

#[test]
fn random_jurors_returns_an_unique_different_subset_of_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        run_to_block(123);

        let mut jurors = <JurorPool<Runtime>>::get();
        for pool_item in DEFAULT_SET_OF_JURORS.iter() {
            <Jurors<Runtime>>::insert(
                pool_item.juror,
                JurorInfo { stake: pool_item.stake, active_lock: 0u128 },
            );
            jurors.try_push(pool_item.clone()).unwrap();
        }

        let mut rng = Court::rng();
        let random_jurors = Court::choose_multiple_weighted(&mut jurors, 3, &mut rng).unwrap();
        let mut at_least_one_set_is_different = false;

        for _ in 0..100 {
            run_blocks(1);

            let another_set_of_random_jurors =
                Court::choose_multiple_weighted(&mut jurors, 3, &mut rng).unwrap();
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
        let mut jurors = <JurorPool<Runtime>>::get();
        for pool_item in DEFAULT_SET_OF_JURORS.iter() {
            <Jurors<Runtime>>::insert(
                pool_item.juror,
                JurorInfo { stake: pool_item.stake, active_lock: 0u128 },
            );
            jurors.try_push(pool_item.clone()).unwrap();
        }

        let mut rng = Court::rng();
        let random_jurors = Court::choose_multiple_weighted(&mut jurors, 2, &mut rng).unwrap();
        for draw in random_jurors {
            assert!(DEFAULT_SET_OF_JURORS.iter().any(|el| el.juror == draw.juror));
        }
    });
}
