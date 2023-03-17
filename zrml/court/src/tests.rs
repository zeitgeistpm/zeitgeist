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
        Balances, Court, ExtBuilder, MarketCommons, Origin, RandomnessCollectiveFlip, Runtime,
        System, ALICE, BOB, CHARLIE, DAVE, EVE, FERDIE, GINA, HARRY, IAN, INITIAL_BALANCE,
    },
    Error, JurorInfo, JurorPool, JurorPoolItem, Jurors, MarketOf,
};
use frame_support::{assert_noop, assert_ok, traits::Hooks};
use pallet_balances::BalanceLock;
use zeitgeist_primitives::{
    constants::{mock::CourtLockId, BASE},
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
    JurorPoolItem { stake: 9, juror: HARRY, total_slashable: 0 },
    JurorPoolItem { stake: 8, juror: IAN, total_slashable: 0 },
    JurorPoolItem { stake: 7, juror: ALICE, total_slashable: 0 },
    JurorPoolItem { stake: 6, juror: BOB, total_slashable: 0 },
    JurorPoolItem { stake: 5, juror: CHARLIE, total_slashable: 0 },
    JurorPoolItem { stake: 4, juror: DAVE, total_slashable: 0 },
    JurorPoolItem { stake: 3, juror: EVE, total_slashable: 0 },
    JurorPoolItem { stake: 2, juror: FERDIE, total_slashable: 0 },
    JurorPoolItem { stake: 1, juror: GINA, total_slashable: 0 },
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
fn join_court_successfully_stores_a_juror() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        assert_ok!(Court::join_court(Origin::signed(ALICE), amount));
        assert_eq!(
            Jurors::<Runtime>::iter().next().unwrap(),
            (ALICE, JurorInfo { stake: amount, active_lock: 0u128 })
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
        setup_blocks(123);
    });
}

// Alice is the winner, Bob is tardy and Charlie is the loser
#[test]
fn get_resolution_outcome_awards_winners_and_slashes_losers() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(2);
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
        setup_blocks(2);
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
        setup_blocks(123);

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
        let mut at_least_one_set_is_different = false;

        for _ in 0..100 {
            setup_blocks(1);

            let another_set_of_random_jurors =
                Court::choose_multiple_weighted(&mut jurors, 2, &mut rng).unwrap();
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
        setup_blocks(123);
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

#[test]
fn vote_will_not_accept_unknown_accounts() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(123);
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

fn setup_blocks(num_blocks: u32) {
    for _ in 0..num_blocks {
        let current_block_number = System::block_number() + 1;
        let parent_block_hash = System::parent_hash();
        let current_digest = System::digest();

        System::initialize(&current_block_number, &parent_block_hash, &current_digest);
        RandomnessCollectiveFlip::on_initialize(current_block_number);
        Court::on_initialize(current_block_number);
        System::finalize();
        System::set_block_number(current_block_number);
    }
}
