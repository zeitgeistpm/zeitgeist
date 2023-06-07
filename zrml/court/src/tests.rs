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
        run_blocks, run_to_block, Balances, Court, ExtBuilder, MarketCommons, Runtime,
        RuntimeOrigin, System, ALICE, BOB, CHARLIE, DAVE, EVE, INITIAL_BALANCE, POOR_PAUL,
    },
    mock_storage::pallet::MarketIdsPerDisputeBlock,
    types::{CourtStatus, Draw, Vote, VoteItem},
    AppealInfo, BalanceOf, CourtId, CourtIdToMarketId, CourtParticipantInfo,
    CourtParticipantInfoOf, CourtPool, CourtPoolItem, CourtPoolOf, Courts, Error, Event,
    MarketIdToCourtId, MarketOf, NegativeImbalanceOf, Participants, RequestBlock, SelectedDraws,
};
use alloc::collections::BTreeMap;
use frame_support::{
    assert_noop, assert_ok,
    traits::{fungible::Balanced, tokens::imbalance::Imbalance, Currency, NamedReservableCurrency},
};
use pallet_balances::{BalanceLock, NegativeImbalance};
use rand::seq::SliceRandom;
use sp_runtime::{
    traits::{BlakeTwo256, Hash, Zero},
    Perquintill,
};
use test_case::test_case;
use zeitgeist_primitives::{
    constants::{
        mock::{
            AggregationPeriod, AppealBond, AppealPeriod, InflationPeriod, LockId, MaxAppeals,
            MaxCourtParticipants, MinJurorStake, RequestInterval, VotePeriod,
        },
        BASE,
    },
    traits::DisputeApi,
    types::{
        AccountIdTest, Asset, Deadlines, GlobalDisputeItem, Market, MarketBonds, MarketCreation,
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

fn initialize_court() -> CourtId {
    let now = <frame_system::Pallet<Runtime>>::block_number();
    <RequestBlock<Runtime>>::put(now + RequestInterval::get());
    let amount_alice = 2 * BASE;
    let amount_bob = 3 * BASE;
    let amount_charlie = 4 * BASE;
    let amount_dave = 5 * BASE;
    let amount_eve = 6 * BASE;
    Court::join_court(RuntimeOrigin::signed(ALICE), amount_alice).unwrap();
    Court::join_court(RuntimeOrigin::signed(BOB), amount_bob).unwrap();
    Court::join_court(RuntimeOrigin::signed(CHARLIE), amount_charlie).unwrap();
    Court::join_court(RuntimeOrigin::signed(DAVE), amount_dave).unwrap();
    Court::join_court(RuntimeOrigin::signed(EVE), amount_eve).unwrap();
    let market_id = MarketCommons::push_market(DEFAULT_MARKET).unwrap();
    MarketCommons::mutate_market(&market_id, |market| {
        market.report = Some(Report { at: 1, by: BOB, outcome: ORACLE_REPORT });
        Ok(())
    })
    .unwrap();
    Court::on_dispute(&market_id, &DEFAULT_MARKET).unwrap();
    <MarketIdToCourtId<Runtime>>::get(market_id).unwrap()
}

fn fill_juror_pool(jurors_len: u32) {
    for i in 0..jurors_len {
        let amount = MinJurorStake::get() + i as u128;
        let juror = (i + 1000) as u128;
        let _ = Balances::deposit(&juror, amount).unwrap();
        assert_ok!(Court::join_court(RuntimeOrigin::signed(juror), amount));
    }
}

fn fill_appeals(court_id: CourtId, appeal_number: usize) {
    assert!(appeal_number <= MaxAppeals::get() as usize);
    let mut court = Courts::<Runtime>::get(court_id).unwrap();
    let mut number = 0u128;
    while (number as usize) < appeal_number {
        let appealed_vote_item: VoteItem = VoteItem::Outcome(OutcomeReport::Scalar(number));
        court
            .appeals
            .try_push(AppealInfo {
                backer: number,
                bond: crate::get_appeal_bond::<Runtime>(court.appeals.len()),
                appealed_vote_item,
            })
            .unwrap();
        number += 1;
    }
    Courts::<Runtime>::insert(court_id, court);
}

fn put_alice_in_draw(court_id: CourtId, stake: BalanceOf<Runtime>) {
    // trick a little bit to let alice be part of the ("random") selection
    let mut draws = <SelectedDraws<Runtime>>::get(court_id);
    assert!(!draws.is_empty());
    let slashable = MinJurorStake::get();
    draws[0] = Draw { court_participant: ALICE, weight: 1, vote: Vote::Drawn, slashable };
    <SelectedDraws<Runtime>>::insert(court_id, draws);
    <Participants<Runtime>>::insert(
        ALICE,
        CourtParticipantInfo {
            stake,
            active_lock: slashable,
            prepare_exit_at: None,
            delegations: Default::default(),
        },
    );
}

fn set_alice_after_vote(
    outcome: OutcomeReport,
) -> (CourtId, <Runtime as frame_system::Config>::Hash, <Runtime as frame_system::Config>::Hash) {
    fill_juror_pool(MaxCourtParticipants::get());
    let court_id = initialize_court();

    let amount = MinJurorStake::get() * 100;
    assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));

    put_alice_in_draw(court_id, amount);

    run_to_block(<RequestBlock<Runtime>>::get() + 1);

    let salt = <Runtime as frame_system::Config>::Hash::default();
    let vote_item = VoteItem::Outcome(outcome);
    let commitment = BlakeTwo256::hash_of(&(ALICE, vote_item, salt));
    assert_ok!(Court::vote(RuntimeOrigin::signed(ALICE), court_id, commitment));

    (court_id, commitment, salt)
}

fn the_lock(amount: u128) -> BalanceLock<u128> {
    BalanceLock { id: LockId::get(), amount, reasons: pallet_balances::Reasons::All }
}

#[test]
fn exit_court_successfully_removes_a_juror_and_frees_balances() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));
        assert_ok!(Court::prepare_exit_court(RuntimeOrigin::signed(ALICE)));
        run_blocks(InflationPeriod::get());
        assert_ok!(Court::exit_court(RuntimeOrigin::signed(ALICE), ALICE));
        assert_eq!(Participants::<Runtime>::iter().count(), 0);
        assert_eq!(Balances::free_balance(ALICE), INITIAL_BALANCE);
        assert_eq!(Balances::locks(ALICE), vec![]);
    });
}

#[test]
fn join_court_successfully_stores_required_data() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        let alice_free_balance_before = Balances::free_balance(ALICE);
        let joined_at = <frame_system::Pallet<Runtime>>::block_number();
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));
        System::assert_last_event(Event::JurorJoined { juror: ALICE, stake: amount }.into());
        assert_eq!(
            Participants::<Runtime>::iter().next().unwrap(),
            (
                ALICE,
                CourtParticipantInfo {
                    stake: amount,
                    active_lock: 0u128,
                    prepare_exit_at: None,
                    delegations: Default::default()
                }
            )
        );
        assert_eq!(Balances::free_balance(ALICE), alice_free_balance_before);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(amount)]);
        assert_eq!(
            CourtPool::<Runtime>::get().into_inner(),
            vec![CourtPoolItem {
                stake: amount,
                court_participant: ALICE,
                consumed_stake: 0,
                joined_at
            }]
        );
    });
}

#[test]
fn join_court_works_multiple_joins() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let amount = 2 * min;
        let joined_at_0 = <frame_system::Pallet<Runtime>>::block_number();
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));
        assert_eq!(Balances::locks(ALICE), vec![the_lock(amount)]);
        assert_eq!(
            CourtPool::<Runtime>::get().into_inner(),
            vec![CourtPoolItem {
                stake: amount,
                court_participant: ALICE,
                consumed_stake: 0,
                joined_at: joined_at_0
            }]
        );
        assert_eq!(
            Participants::<Runtime>::iter()
                .collect::<Vec<(AccountIdTest, CourtParticipantInfoOf<Runtime>)>>(),
            vec![(
                ALICE,
                CourtParticipantInfo {
                    stake: amount,
                    active_lock: 0u128,
                    prepare_exit_at: None,
                    delegations: Default::default()
                }
            )]
        );

        let joined_at_1 = <frame_system::Pallet<Runtime>>::block_number();
        assert_ok!(Court::join_court(RuntimeOrigin::signed(BOB), amount));
        assert_eq!(Balances::locks(BOB), vec![the_lock(amount)]);
        assert_eq!(
            CourtPool::<Runtime>::get().into_inner(),
            vec![
                CourtPoolItem {
                    stake: amount,
                    court_participant: ALICE,
                    consumed_stake: 0,
                    joined_at: joined_at_0
                },
                CourtPoolItem {
                    stake: amount,
                    court_participant: BOB,
                    consumed_stake: 0,
                    joined_at: joined_at_1
                }
            ]
        );
        assert_eq!(Participants::<Runtime>::iter().count(), 2);
        assert_eq!(
            Participants::<Runtime>::get(ALICE).unwrap(),
            CourtParticipantInfo {
                stake: amount,
                active_lock: 0u128,
                prepare_exit_at: None,
                delegations: Default::default()
            }
        );
        assert_eq!(
            Participants::<Runtime>::get(BOB).unwrap(),
            CourtParticipantInfo {
                stake: amount,
                active_lock: 0u128,
                prepare_exit_at: None,
                delegations: Default::default()
            }
        );

        let higher_amount = amount + 1;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), higher_amount));
        assert_eq!(Balances::locks(BOB), vec![the_lock(amount)]);
        assert_eq!(Balances::locks(ALICE), vec![the_lock(higher_amount)]);
        assert_eq!(
            CourtPool::<Runtime>::get().into_inner(),
            vec![
                CourtPoolItem {
                    stake: amount,
                    court_participant: BOB,
                    consumed_stake: 0,
                    joined_at: joined_at_1
                },
                CourtPoolItem {
                    stake: higher_amount,
                    court_participant: ALICE,
                    consumed_stake: 0,
                    joined_at: joined_at_0
                },
            ]
        );
        assert_eq!(Participants::<Runtime>::iter().count(), 2);
        assert_eq!(
            Participants::<Runtime>::get(BOB).unwrap(),
            CourtParticipantInfo {
                stake: amount,
                active_lock: 0u128,
                prepare_exit_at: None,
                delegations: Default::default()
            }
        );
        assert_eq!(
            Participants::<Runtime>::get(ALICE).unwrap(),
            CourtParticipantInfo {
                stake: higher_amount,
                active_lock: 0u128,
                prepare_exit_at: None,
                delegations: Default::default()
            }
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
        Participants::<Runtime>::insert(
            ALICE,
            CourtParticipantInfo {
                stake: amount,
                active_lock,
                prepare_exit_at: None,
                delegations: Default::default(),
            },
        );
        let joined_at = <frame_system::Pallet<Runtime>>::block_number();
        let juror_pool = vec![CourtPoolItem {
            stake: amount,
            court_participant: ALICE,
            consumed_stake,
            joined_at,
        }];
        CourtPool::<Runtime>::put::<CourtPoolOf<Runtime>>(juror_pool.try_into().unwrap());

        let higher_amount = amount + 1;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), higher_amount));
        assert_eq!(CourtPool::<Runtime>::get().into_inner()[0].consumed_stake, consumed_stake);
        assert_eq!(Participants::<Runtime>::get(ALICE).unwrap().active_lock, active_lock);
    });
}

#[test]
fn join_court_fails_below_min_juror_stake() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let amount = min - 1;
        assert_noop!(
            Court::join_court(RuntimeOrigin::signed(ALICE), amount),
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
            Court::join_court(RuntimeOrigin::signed(POOR_PAUL), amount),
            Error::<Runtime>::AmountExceedsBalance
        );
    });
}

#[test]
fn join_court_fails_amount_below_last_join() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let last_join_amount = 2 * min;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), last_join_amount));

        assert_noop!(
            Court::join_court(RuntimeOrigin::signed(ALICE), last_join_amount - 1),
            Error::<Runtime>::AmountBelowLastJoin
        );
    });
}

#[test]
fn join_court_after_prepare_exit_court() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let amount = 2 * min;
        let now = <frame_system::Pallet<Runtime>>::block_number();
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));

        assert_ok!(Court::prepare_exit_court(RuntimeOrigin::signed(ALICE)));

        let p_info = <Participants<Runtime>>::get(ALICE).unwrap();
        assert_eq!(Some(now), p_info.prepare_exit_at);

        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount + 1));

        let p_info = <Participants<Runtime>>::get(ALICE).unwrap();
        assert_eq!(None, p_info.prepare_exit_at);
    });
}

#[test]
fn join_court_fails_amount_below_lowest_juror() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let min_amount = 2 * min;

        let max_accounts = CourtPoolOf::<Runtime>::bound();
        let max_amount = min_amount + max_accounts as u128;
        for i in 1..=max_accounts {
            let amount = max_amount - i as u128;
            let _ = Balances::deposit(&(i as u128), amount).unwrap();
            assert_ok!(Court::join_court(RuntimeOrigin::signed(i as u128), amount));
        }

        assert!(CourtPool::<Runtime>::get().is_full());

        assert_noop!(
            Court::join_court(RuntimeOrigin::signed(0u128), min_amount - 1),
            Error::<Runtime>::AmountBelowLowestJuror
        );
    });
}

#[test]
fn prepare_exit_court_works() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        let joined_at = <frame_system::Pallet<Runtime>>::block_number();
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));
        assert_eq!(
            CourtPool::<Runtime>::get().into_inner(),
            vec![CourtPoolItem {
                stake: amount,
                court_participant: ALICE,
                consumed_stake: 0,
                joined_at
            }]
        );

        assert_ok!(Court::prepare_exit_court(RuntimeOrigin::signed(ALICE)));
        System::assert_last_event(Event::ExitPrepared { court_participant: ALICE }.into());
        assert!(CourtPool::<Runtime>::get().into_inner().is_empty());
    });
}

#[test]
fn prepare_exit_court_removes_lowest_staked_juror() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let min_amount = 2 * min;

        for i in 0..CourtPoolOf::<Runtime>::bound() {
            let amount = min_amount + i as u128;
            let juror = i as u128;
            let _ = Balances::deposit(&juror, amount).unwrap();
            assert_ok!(Court::join_court(RuntimeOrigin::signed(juror), amount));
        }

        let len = CourtPool::<Runtime>::get().into_inner().len();
        assert!(
            CourtPool::<Runtime>::get()
                .into_inner()
                .iter()
                .any(|item| item.court_participant == 0u128)
        );
        assert_ok!(Court::prepare_exit_court(RuntimeOrigin::signed(0u128)));
        assert_eq!(CourtPool::<Runtime>::get().into_inner().len(), len - 1);
        CourtPool::<Runtime>::get().into_inner().iter().for_each(|item| {
            assert_ne!(item.court_participant, 0u128);
        });
    });
}

#[test]
fn prepare_exit_court_removes_middle_staked_juror() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let min_amount = 2 * min;

        for i in 0..CourtPoolOf::<Runtime>::bound() {
            let amount = min_amount + i as u128;
            let juror = i as u128;
            let _ = Balances::deposit(&juror, amount).unwrap();
            assert_ok!(Court::join_court(RuntimeOrigin::signed(juror), amount));
        }

        let middle_index = (CourtPoolOf::<Runtime>::bound() / 2) as u128;

        let len = CourtPool::<Runtime>::get().into_inner().len();
        assert!(
            CourtPool::<Runtime>::get()
                .into_inner()
                .iter()
                .any(|item| item.court_participant == middle_index)
        );
        assert_ok!(Court::prepare_exit_court(RuntimeOrigin::signed(middle_index)));
        assert_eq!(CourtPool::<Runtime>::get().into_inner().len(), len - 1);
        CourtPool::<Runtime>::get().into_inner().iter().for_each(|item| {
            assert_ne!(item.court_participant, middle_index);
        });
    });
}

#[test]
fn prepare_exit_court_removes_highest_staked_juror() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let min_amount = 2 * min;

        for i in 0..CourtPoolOf::<Runtime>::bound() {
            let amount = min_amount + i as u128;
            let juror = i as u128;
            let _ = Balances::deposit(&juror, amount).unwrap();
            assert_ok!(Court::join_court(RuntimeOrigin::signed(juror), amount));
        }

        let last_index = (CourtPoolOf::<Runtime>::bound() - 1) as u128;

        let len = CourtPool::<Runtime>::get().into_inner().len();
        assert!(
            CourtPool::<Runtime>::get()
                .into_inner()
                .iter()
                .any(|item| item.court_participant == last_index)
        );
        assert_ok!(Court::prepare_exit_court(RuntimeOrigin::signed(last_index)));
        assert_eq!(CourtPool::<Runtime>::get().into_inner().len(), len - 1);
        CourtPool::<Runtime>::get().into_inner().iter().for_each(|item| {
            assert_ne!(item.court_participant, last_index);
        });
    });
}

#[test]
fn join_court_binary_search_sorted_insert_works() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let min_amount = 2 * min;

        let max_accounts = CourtPoolOf::<Runtime>::bound();
        let mut rng = rand::thread_rng();
        let mut random_numbers: Vec<u32> = (0u32..max_accounts as u32).collect();
        random_numbers.shuffle(&mut rng);
        let max_amount = min_amount + max_accounts as u128;
        for i in random_numbers {
            let amount = max_amount - i as u128;
            let juror = i as u128;
            let _ = Balances::deposit(&juror, amount).unwrap();
            assert_ok!(Court::join_court(RuntimeOrigin::signed(juror), amount));
        }

        let mut last_stake = 0;
        for pool_item in CourtPool::<Runtime>::get().into_inner().iter() {
            assert!(pool_item.stake >= last_stake);
            last_stake = pool_item.stake;
        }
    });
}

#[test]
fn prepare_exit_court_fails_juror_already_prepared_to_exit() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        let joined_at = <frame_system::Pallet<Runtime>>::block_number();
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));
        assert_eq!(
            CourtPool::<Runtime>::get().into_inner(),
            vec![CourtPoolItem {
                stake: amount,
                court_participant: ALICE,
                consumed_stake: 0,
                joined_at
            }]
        );

        assert_ok!(Court::prepare_exit_court(RuntimeOrigin::signed(ALICE)));

        assert_noop!(
            Court::prepare_exit_court(RuntimeOrigin::signed(ALICE)),
            Error::<Runtime>::AlreadyPreparedExit
        );
    });
}

#[test]
fn prepare_exit_court_fails_juror_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(Participants::<Runtime>::iter().next().is_none());

        assert_noop!(
            Court::prepare_exit_court(RuntimeOrigin::signed(ALICE)),
            Error::<Runtime>::JurorDoesNotExist
        );
    });
}

#[test]
fn exit_court_works_without_active_lock() {
    ExtBuilder::default().build().execute_with(|| {
        let min = MinJurorStake::get();
        let amount = 2 * min;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));
        assert!(!CourtPool::<Runtime>::get().into_inner().is_empty());
        assert_ok!(Court::prepare_exit_court(RuntimeOrigin::signed(ALICE)));
        assert!(CourtPool::<Runtime>::get().into_inner().is_empty());
        assert!(Participants::<Runtime>::get(ALICE).is_some());

        run_blocks(InflationPeriod::get());

        assert_eq!(Balances::locks(ALICE), vec![the_lock(amount)]);
        assert_ok!(Court::exit_court(RuntimeOrigin::signed(ALICE), ALICE));
        System::assert_last_event(
            Event::ExitedCourt {
                court_participant: ALICE,
                exit_amount: amount,
                active_lock: 0u128,
            }
            .into(),
        );
        assert!(Participants::<Runtime>::iter().next().is_none());
        assert!(Balances::locks(ALICE).is_empty());
    });
}

#[test]
fn exit_court_works_with_active_lock() {
    ExtBuilder::default().build().execute_with(|| {
        let active_lock = MinJurorStake::get();
        let amount = 3 * active_lock;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));
        assert!(!CourtPool::<Runtime>::get().into_inner().is_empty());

        assert_eq!(
            <Participants<Runtime>>::get(ALICE).unwrap(),
            CourtParticipantInfo {
                stake: amount,
                active_lock: 0,
                prepare_exit_at: None,
                delegations: Default::default()
            }
        );
        // assume that `choose_multiple_weighted` has set the active_lock
        <Participants<Runtime>>::insert(
            ALICE,
            CourtParticipantInfo {
                stake: amount,
                active_lock,
                prepare_exit_at: None,
                delegations: Default::default(),
            },
        );

        assert_eq!(Balances::locks(ALICE), vec![the_lock(amount)]);

        let now = <frame_system::Pallet<Runtime>>::block_number();
        assert_ok!(Court::prepare_exit_court(RuntimeOrigin::signed(ALICE)));
        assert!(CourtPool::<Runtime>::get().into_inner().is_empty());

        run_blocks(InflationPeriod::get());

        assert_ok!(Court::exit_court(RuntimeOrigin::signed(ALICE), ALICE));
        System::assert_last_event(
            Event::ExitedCourt {
                court_participant: ALICE,
                exit_amount: amount - active_lock,
                active_lock,
            }
            .into(),
        );
        assert_eq!(
            Participants::<Runtime>::get(ALICE).unwrap(),
            CourtParticipantInfo {
                stake: active_lock,
                active_lock,
                prepare_exit_at: Some(now),
                delegations: Default::default()
            }
        );
        assert_eq!(Balances::locks(ALICE), vec![the_lock(active_lock)]);
    });
}

#[test]
fn exit_court_fails_juror_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Court::exit_court(RuntimeOrigin::signed(ALICE), ALICE),
            Error::<Runtime>::JurorDoesNotExist
        );
    });
}

#[test]
fn exit_court_fails_juror_not_prepared_to_exit() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));

        run_blocks(InflationPeriod::get());

        assert_noop!(
            Court::exit_court(RuntimeOrigin::signed(ALICE), ALICE),
            Error::<Runtime>::PrepareExitAtNotPresent
        );
    });
}

#[test]
fn exit_court_fails_if_inflation_period_not_over() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = 2 * BASE;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));

        assert_ok!(Court::prepare_exit_court(RuntimeOrigin::signed(ALICE)));

        run_blocks(InflationPeriod::get() - 1);

        assert_noop!(
            Court::exit_court(RuntimeOrigin::signed(ALICE), ALICE),
            Error::<Runtime>::PrematureExit
        );
    });
}

#[test]
fn vote_works() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool(MaxCourtParticipants::get());
        let court_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));

        // trick a little bit to let alice be part of the ("random") selection
        let mut draws = <SelectedDraws<Runtime>>::get(court_id);
        assert_eq!(
            draws.iter().map(|draw| draw.weight).sum::<u32>() as usize,
            Court::necessary_draws_weight(0usize)
        );
        let slashable = MinJurorStake::get();
        let alice_index =
            draws.binary_search_by_key(&ALICE, |draw| draw.court_participant).unwrap_or_else(|j| j);
        draws[alice_index] =
            Draw { court_participant: ALICE, weight: 1, vote: Vote::Drawn, slashable };
        <SelectedDraws<Runtime>>::insert(court_id, draws);
        <Participants<Runtime>>::insert(
            ALICE,
            CourtParticipantInfo {
                stake: amount,
                active_lock: slashable,
                prepare_exit_at: None,
                delegations: Default::default(),
            },
        );

        let old_draws = <SelectedDraws<Runtime>>::get(court_id);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        let outcome = OutcomeReport::Scalar(42u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome, salt));
        assert_ok!(Court::vote(RuntimeOrigin::signed(ALICE), court_id, commitment));
        System::assert_last_event(Event::JurorVoted { court_id, juror: ALICE, commitment }.into());

        let new_draws = <SelectedDraws<Runtime>>::get(court_id);
        for (i, (old_draw, new_draw)) in old_draws.iter().zip(new_draws.iter()).enumerate() {
            if i == alice_index {
                continue;
            } else {
                assert_eq!(old_draw, new_draw);
            }
        }
        assert_eq!(
            old_draws[alice_index].court_participant,
            new_draws[alice_index].court_participant
        );
        assert_eq!(old_draws[alice_index].weight, new_draws[alice_index].weight);
        assert_eq!(old_draws[alice_index].slashable, new_draws[alice_index].slashable);
        assert_eq!(old_draws[alice_index].vote, Vote::Drawn);
        assert_eq!(new_draws[alice_index].vote, Vote::Secret { commitment });
    });
}

#[test]
fn vote_overwrite_works() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool(MaxCourtParticipants::get());
        let court_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));

        put_alice_in_draw(court_id, amount);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        let wrong_outcome = OutcomeReport::Scalar(69u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let wrong_commitment = BlakeTwo256::hash_of(&(ALICE, wrong_outcome, salt));
        assert_ok!(Court::vote(RuntimeOrigin::signed(ALICE), court_id, wrong_commitment));
        assert_eq!(
            <SelectedDraws<Runtime>>::get(court_id)[0].vote,
            Vote::Secret { commitment: wrong_commitment }
        );

        run_blocks(1);

        let right_outcome = OutcomeReport::Scalar(42u128);
        let new_commitment = BlakeTwo256::hash_of(&(ALICE, right_outcome, salt));
        assert_ok!(Court::vote(RuntimeOrigin::signed(ALICE), court_id, new_commitment));
        assert_ne!(wrong_commitment, new_commitment);
        assert_eq!(
            <SelectedDraws<Runtime>>::get(court_id)[0].vote,
            Vote::Secret { commitment: new_commitment }
        );
    });
}

#[test]
fn vote_fails_if_court_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = 0;
        let commitment = <Runtime as frame_system::Config>::Hash::default();
        assert_noop!(
            Court::vote(RuntimeOrigin::signed(ALICE), court_id, commitment),
            Error::<Runtime>::CourtNotFound
        );
    });
}

#[test_case(
    Vote::Revealed {
        commitment: <Runtime as frame_system::Config>::Hash::default(),
        vote_item: VoteItem::Outcome(OutcomeReport::Scalar(1u128)),
        salt: <Runtime as frame_system::Config>::Hash::default(),
    }; "revealed"
)]
#[test_case(
    Vote::Denounced {
        commitment: <Runtime as frame_system::Config>::Hash::default(),
        vote_item: VoteItem::Outcome(OutcomeReport::Scalar(1u128)),
        salt: <Runtime as frame_system::Config>::Hash::default(),
    }; "denounced"
)]
fn vote_fails_if_vote_state_incorrect(
    vote: crate::Vote<<Runtime as frame_system::Config>::Hash, crate::DelegatedStakesOf<Runtime>>,
) {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool(MaxCourtParticipants::get());
        let court_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));

        let mut draws = <SelectedDraws<Runtime>>::get(court_id);
        assert!(!draws.is_empty());
        draws[0] = Draw { court_participant: ALICE, weight: 101, vote, slashable: 42u128 };
        <SelectedDraws<Runtime>>::insert(court_id, draws);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        let outcome = OutcomeReport::Scalar(42u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome, salt));
        assert_noop!(
            Court::vote(RuntimeOrigin::signed(ALICE), court_id, commitment),
            Error::<Runtime>::InvalidVoteState
        );
    });
}

#[test]
fn vote_fails_if_caller_not_in_draws() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool(MaxCourtParticipants::get());
        let court_id = initialize_court();

        let mut draws = <SelectedDraws<Runtime>>::get(court_id);
        draws.retain(|draw| draw.court_participant != ALICE);
        <SelectedDraws<Runtime>>::insert(court_id, draws);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        let outcome = OutcomeReport::Scalar(42u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome, salt));
        assert_noop!(
            Court::vote(RuntimeOrigin::signed(ALICE), court_id, commitment),
            Error::<Runtime>::CallerNotInSelectedDraws
        );
    });
}

#[test]
fn vote_fails_if_not_in_voting_period() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool(MaxCourtParticipants::get());
        let court_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));

        put_alice_in_draw(court_id, amount);

        run_to_block(<RequestBlock<Runtime>>::get() + VotePeriod::get() + 1);

        let outcome = OutcomeReport::Scalar(42u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome, salt));
        assert_noop!(
            Court::vote(RuntimeOrigin::signed(ALICE), court_id, commitment),
            Error::<Runtime>::NotInVotingPeriod
        );
    });
}

#[test]
fn reveal_vote_works() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool(MaxCourtParticipants::get());
        let court_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));

        // trick a little bit to let alice be part of the ("random") selection
        let mut draws = <SelectedDraws<Runtime>>::get(court_id);
        assert_eq!(
            draws.iter().map(|draw| draw.weight).sum::<u32>() as usize,
            Court::necessary_draws_weight(0usize)
        );
        let slashable = MinJurorStake::get();
        let alice_index =
            draws.binary_search_by_key(&ALICE, |draw| draw.court_participant).unwrap_or_else(|j| j);
        draws[alice_index] =
            Draw { court_participant: ALICE, weight: 1, vote: Vote::Drawn, slashable };
        <SelectedDraws<Runtime>>::insert(court_id, draws);
        <Participants<Runtime>>::insert(
            ALICE,
            CourtParticipantInfo {
                stake: amount,
                active_lock: slashable,
                prepare_exit_at: None,
                delegations: Default::default(),
            },
        );

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        let outcome = OutcomeReport::Scalar(42u128);
        let vote_item = VoteItem::Outcome(outcome);

        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, vote_item.clone(), salt));
        assert_ok!(Court::vote(RuntimeOrigin::signed(ALICE), court_id, commitment));

        let old_draws = <SelectedDraws<Runtime>>::get(court_id);

        run_blocks(VotePeriod::get() + 1);

        assert_ok!(Court::reveal_vote(
            RuntimeOrigin::signed(ALICE),
            court_id,
            vote_item.clone(),
            salt,
        ));
        System::assert_last_event(
            Event::JurorRevealedVote { juror: ALICE, court_id, vote_item: vote_item.clone(), salt }
                .into(),
        );

        let new_draws = <SelectedDraws<Runtime>>::get(court_id);
        for (i, (old_draw, new_draw)) in old_draws.iter().zip(new_draws.iter()).enumerate() {
            if i == alice_index {
                continue;
            }
            assert_eq!(old_draw, new_draw);
        }
        assert_eq!(
            old_draws[alice_index].court_participant,
            new_draws[alice_index].court_participant
        );
        assert_eq!(old_draws[alice_index].weight, new_draws[alice_index].weight);
        assert_eq!(old_draws[alice_index].slashable, new_draws[alice_index].slashable);
        assert_eq!(old_draws[alice_index].vote, Vote::Secret { commitment });
        assert_eq!(new_draws[alice_index].vote, Vote::Revealed { commitment, vote_item, salt });
    });
}

#[test]
fn reveal_vote_fails_if_caller_not_juror() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());

        run_blocks(VotePeriod::get() + 1);

        <Participants<Runtime>>::remove(ALICE);

        let vote_item = VoteItem::Outcome(outcome);

        assert_noop!(
            Court::reveal_vote(RuntimeOrigin::signed(ALICE), court_id, vote_item, salt),
            Error::<Runtime>::CallerIsNotACourtParticipant
        );
    });
}

#[test]
fn reveal_vote_fails_if_court_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());
        run_blocks(VotePeriod::get() + 1);

        <Courts<Runtime>>::remove(court_id);

        let vote_item = VoteItem::Outcome(outcome);

        assert_noop!(
            Court::reveal_vote(RuntimeOrigin::signed(ALICE), court_id, vote_item, salt),
            Error::<Runtime>::CourtNotFound
        );
    });
}

#[test]
fn reveal_vote_fails_if_not_in_aggregation_period() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        let vote_item = VoteItem::Outcome(outcome);

        assert_noop!(
            Court::reveal_vote(RuntimeOrigin::signed(ALICE), court_id, vote_item, salt),
            Error::<Runtime>::NotInAggregationPeriod
        );
    });
}

#[test]
fn reveal_vote_fails_if_juror_not_drawn() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());

        run_blocks(VotePeriod::get() + 1);

        <SelectedDraws<Runtime>>::mutate(court_id, |draws| {
            draws.retain(|draw| draw.court_participant != ALICE);
        });

        let vote_item = VoteItem::Outcome(outcome);

        assert_noop!(
            Court::reveal_vote(RuntimeOrigin::signed(ALICE), court_id, vote_item, salt),
            Error::<Runtime>::CallerNotInSelectedDraws
        );
    });
}

#[test]
fn reveal_vote_fails_for_invalid_reveal() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome);

        run_blocks(VotePeriod::get() + 1);

        let invalid_outcome = OutcomeReport::Scalar(43u128);
        let invalid_vote_item = VoteItem::Outcome(invalid_outcome);
        assert_noop!(
            Court::reveal_vote(RuntimeOrigin::signed(ALICE), court_id, invalid_vote_item, salt),
            Error::<Runtime>::CommitmentHashMismatch
        );
    });
}

#[test]
fn reveal_vote_fails_for_invalid_salt() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, correct_salt) = set_alice_after_vote(outcome.clone());

        run_blocks(VotePeriod::get() + 1);

        let incorrect_salt: <Runtime as frame_system::Config>::Hash = [42; 32].into();
        assert_ne!(correct_salt, incorrect_salt);

        let vote_item = VoteItem::Outcome(outcome);
        assert_noop!(
            Court::reveal_vote(RuntimeOrigin::signed(ALICE), court_id, vote_item, incorrect_salt),
            Error::<Runtime>::CommitmentHashMismatch
        );
    });
}

#[test]
fn reveal_vote_fails_if_juror_not_voted() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());

        run_blocks(VotePeriod::get() + 1);

        <SelectedDraws<Runtime>>::mutate(court_id, |draws| {
            draws.iter_mut().for_each(|draw| {
                if draw.court_participant == ALICE {
                    draw.vote = Vote::Drawn;
                }
            });
        });

        let vote_item = VoteItem::Outcome(outcome);

        assert_noop!(
            Court::reveal_vote(RuntimeOrigin::signed(ALICE), court_id, vote_item, salt),
            Error::<Runtime>::JurorDidNotVote
        );
    });
}

#[test]
fn reveal_vote_fails_if_already_revealed() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());

        run_blocks(VotePeriod::get() + 1);

        let vote_item = VoteItem::Outcome(outcome);

        assert_ok!(Court::reveal_vote(
            RuntimeOrigin::signed(ALICE),
            court_id,
            vote_item.clone(),
            salt
        ));

        assert_noop!(
            Court::reveal_vote(RuntimeOrigin::signed(ALICE), court_id, vote_item, salt),
            Error::<Runtime>::VoteAlreadyRevealed
        );
    });
}

#[test]
fn reveal_vote_fails_if_already_denounced() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());

        let vote_item = VoteItem::Outcome(outcome);

        assert_ok!(Court::denounce_vote(
            RuntimeOrigin::signed(BOB),
            court_id,
            ALICE,
            vote_item.clone(),
            salt
        ));

        run_blocks(VotePeriod::get() + 1);

        assert_noop!(
            Court::reveal_vote(RuntimeOrigin::signed(ALICE), court_id, vote_item, salt),
            Error::<Runtime>::VoteAlreadyDenounced
        );
    });
}

#[test]
fn denounce_vote_works() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, commitment, salt) = set_alice_after_vote(outcome.clone());

        let old_draws = <SelectedDraws<Runtime>>::get(court_id);
        assert!(old_draws.iter().any(|draw| {
            draw.court_participant == ALICE && matches!(draw.vote, Vote::Secret { .. })
        }));

        let free_alice_before = Balances::free_balance(ALICE);
        let pot_balance_before = Balances::free_balance(Court::reward_pot(court_id));

        let vote_item = VoteItem::Outcome(outcome);

        assert_ok!(Court::denounce_vote(
            RuntimeOrigin::signed(BOB),
            court_id,
            ALICE,
            vote_item.clone(),
            salt
        ));
        System::assert_last_event(
            Event::DenouncedJurorVote {
                denouncer: BOB,
                juror: ALICE,
                court_id,
                vote_item: vote_item.clone(),
                salt,
            }
            .into(),
        );

        let new_draws = <SelectedDraws<Runtime>>::get(court_id);
        assert_eq!(old_draws[1..], new_draws[1..]);
        assert_eq!(old_draws[0].court_participant, ALICE);
        assert_eq!(old_draws[0].court_participant, new_draws[0].court_participant);
        assert_eq!(old_draws[0].weight, new_draws[0].weight);
        assert_eq!(old_draws[0].slashable, new_draws[0].slashable);
        assert_eq!(old_draws[0].vote, Vote::Secret { commitment });
        assert_eq!(new_draws[0].vote, Vote::Denounced { commitment, vote_item, salt });

        let free_alice_after = Balances::free_balance(ALICE);
        let slash = old_draws[0].slashable;
        assert!(!slash.is_zero());
        // slash happens in `reassign_court_stakes`
        // see `reassign_court_stakes_slashes_tardy_jurors_and_rewards_winners`
        assert_eq!(free_alice_after, free_alice_before);

        let pot_balance_after = Balances::free_balance(Court::reward_pot(court_id));
        assert_eq!(pot_balance_after, pot_balance_before);
    });
}

#[test]
fn denounce_vote_fails_if_self_denounce() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());

        let vote_item = VoteItem::Outcome(outcome);

        assert_noop!(
            Court::denounce_vote(RuntimeOrigin::signed(ALICE), court_id, ALICE, vote_item, salt),
            Error::<Runtime>::CallerDenouncedItself
        );
    });
}

#[test]
fn denounce_vote_fails_if_juror_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());

        <Participants<Runtime>>::remove(ALICE);

        let vote_item = VoteItem::Outcome(outcome);

        assert_noop!(
            Court::denounce_vote(RuntimeOrigin::signed(BOB), court_id, ALICE, vote_item, salt),
            Error::<Runtime>::JurorDoesNotExist
        );
    });
}

#[test]
fn denounce_vote_fails_if_court_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());

        <Courts<Runtime>>::remove(court_id);

        let vote_item = VoteItem::Outcome(outcome);

        assert_noop!(
            Court::denounce_vote(RuntimeOrigin::signed(BOB), court_id, ALICE, vote_item, salt),
            Error::<Runtime>::CourtNotFound
        );
    });
}

#[test]
fn denounce_vote_fails_if_not_in_voting_period() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());

        run_blocks(VotePeriod::get() + 1);

        let vote_item = VoteItem::Outcome(outcome);

        assert_noop!(
            Court::denounce_vote(RuntimeOrigin::signed(BOB), court_id, ALICE, vote_item, salt),
            Error::<Runtime>::NotInVotingPeriod
        );
    });
}

#[test]
fn denounce_vote_fails_if_juror_not_drawn() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());

        <SelectedDraws<Runtime>>::mutate(court_id, |draws| {
            draws.retain(|draw| draw.court_participant != ALICE);
        });

        let vote_item = VoteItem::Outcome(outcome);

        assert_noop!(
            Court::denounce_vote(RuntimeOrigin::signed(BOB), court_id, ALICE, vote_item, salt),
            Error::<Runtime>::JurorNotDrawn
        );
    });
}

#[test]
fn denounce_vote_fails_if_invalid_reveal() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome);

        let invalid_outcome = OutcomeReport::Scalar(69u128);
        let invalid_vote_item = VoteItem::Outcome(invalid_outcome);
        assert_noop!(
            Court::denounce_vote(
                RuntimeOrigin::signed(BOB),
                court_id,
                ALICE,
                invalid_vote_item,
                salt
            ),
            Error::<Runtime>::CommitmentHashMismatch
        );
    });
}

#[test]
fn denounce_vote_fails_if_juror_not_voted() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());

        <SelectedDraws<Runtime>>::mutate(court_id, |draws| {
            draws.iter_mut().for_each(|draw| {
                if draw.court_participant == ALICE {
                    draw.vote = Vote::Drawn;
                }
            });
        });

        let vote_item = VoteItem::Outcome(outcome);

        assert_noop!(
            Court::denounce_vote(RuntimeOrigin::signed(BOB), court_id, ALICE, vote_item, salt),
            Error::<Runtime>::JurorDidNotVote
        );
    });
}

#[test]
fn denounce_vote_fails_if_vote_already_revealed() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());

        run_blocks(VotePeriod::get() + 1);

        let vote_item = VoteItem::Outcome(outcome);

        assert_ok!(Court::reveal_vote(
            RuntimeOrigin::signed(ALICE),
            court_id,
            vote_item.clone(),
            salt
        ));

        assert_noop!(
            Court::reveal_vote(RuntimeOrigin::signed(ALICE), court_id, vote_item, salt),
            Error::<Runtime>::VoteAlreadyRevealed
        );
    });
}

#[test]
fn denounce_vote_fails_if_vote_already_denounced() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, salt) = set_alice_after_vote(outcome.clone());

        let vote_item = VoteItem::Outcome(outcome);

        assert_ok!(Court::denounce_vote(
            RuntimeOrigin::signed(BOB),
            court_id,
            ALICE,
            vote_item.clone(),
            salt
        ));

        assert_noop!(
            Court::denounce_vote(RuntimeOrigin::signed(CHARLIE), court_id, ALICE, vote_item, salt),
            Error::<Runtime>::VoteAlreadyDenounced
        );
    });
}

#[test]
fn appeal_updates_round_ends() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        let last_court = <Courts<Runtime>>::get(court_id).unwrap();

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(RuntimeOrigin::signed(CHARLIE), court_id));

        let now = <frame_system::Pallet<Runtime>>::block_number();
        let court = <Courts<Runtime>>::get(court_id).unwrap();

        let request_block = <RequestBlock<Runtime>>::get();
        assert!(now < request_block);
        assert_eq!(court.round_ends.pre_vote, request_block);
        assert_eq!(court.round_ends.vote, request_block + VotePeriod::get());
        assert_eq!(
            court.round_ends.aggregation,
            request_block + VotePeriod::get() + AggregationPeriod::get()
        );
        assert_eq!(
            court.round_ends.appeal,
            request_block + VotePeriod::get() + AggregationPeriod::get() + AppealPeriod::get()
        );

        assert!(last_court.round_ends.pre_vote < court.round_ends.pre_vote);
        assert!(last_court.round_ends.vote < court.round_ends.vote);
        assert!(last_court.round_ends.aggregation < court.round_ends.aggregation);
        assert!(last_court.round_ends.appeal < court.round_ends.appeal);
    });
}

#[test]
fn appeal_reserves_get_appeal_bond() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        let free_charlie_before = Balances::free_balance(CHARLIE);
        assert_ok!(Court::appeal(RuntimeOrigin::signed(CHARLIE), court_id));

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
        let (court_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(RuntimeOrigin::signed(CHARLIE), court_id));

        System::assert_last_event(Event::CourtAppealed { court_id, appeal_number: 1u32 }.into());
    });
}

#[test]
fn appeal_shifts_auto_resolve() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        let resolve_at_0 = <Courts<Runtime>>::get(court_id).unwrap().round_ends.appeal;
        assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at_0), vec![0]);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(RuntimeOrigin::signed(CHARLIE), court_id));

        let resolve_at_1 = <Courts<Runtime>>::get(court_id).unwrap().round_ends.appeal;
        assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at_1), vec![0]);
        assert_ne!(resolve_at_0, resolve_at_1);
        assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at_0), vec![]);
    });
}

#[test]
fn appeal_overrides_last_draws() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        let last_draws = <SelectedDraws<Runtime>>::get(court_id);
        assert!(!last_draws.len().is_zero());

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(RuntimeOrigin::signed(CHARLIE), court_id));

        let draws = <SelectedDraws<Runtime>>::get(court_id);
        assert_ne!(draws, last_draws);
    });
}

#[test]
fn appeal_draws_total_weight_is_correct() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        let last_draws = <SelectedDraws<Runtime>>::get(court_id);
        let last_draws_total_weight = last_draws.iter().map(|draw| draw.weight).sum::<u32>();
        assert_eq!(last_draws_total_weight, Court::necessary_draws_weight(0usize) as u32);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(RuntimeOrigin::signed(CHARLIE), court_id));

        let neccessary_juror_weight = Court::necessary_draws_weight(1usize) as u32;
        let draws = <SelectedDraws<Runtime>>::get(court_id);
        let draws_total_weight = draws.iter().map(|draw| draw.weight).sum::<u32>();
        assert_eq!(draws_total_weight, neccessary_juror_weight);
    });
}

#[test]
fn appeal_get_latest_resolved_outcome_changes() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(RuntimeOrigin::signed(CHARLIE), court_id));

        let last_appealed_vote_item = <Courts<Runtime>>::get(court_id)
            .unwrap()
            .appeals
            .last()
            .unwrap()
            .appealed_vote_item
            .clone();

        let request_block = <RequestBlock<Runtime>>::get();
        run_to_block(request_block + 1);
        let outcome = OutcomeReport::Scalar(69u128);
        let vote_item = VoteItem::Outcome(outcome.clone());
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, vote_item, salt));

        // cheat a little to get alice in the draw for the new appeal
        put_alice_in_draw(court_id, MinJurorStake::get());
        assert_ok!(Court::vote(RuntimeOrigin::signed(ALICE), court_id, commitment));

        run_blocks(VotePeriod::get() + 1);

        let vote_item = VoteItem::Outcome(outcome);

        assert_ok!(Court::reveal_vote(
            RuntimeOrigin::signed(ALICE),
            court_id,
            vote_item.clone(),
            salt
        ));

        run_blocks(AggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(RuntimeOrigin::signed(CHARLIE), court_id));

        let new_appealed_vote_item = <Courts<Runtime>>::get(court_id)
            .unwrap()
            .appeals
            .last()
            .unwrap()
            .appealed_vote_item
            .clone();

        // if the new appealed outcome were the last appealed outcome,
        // then the wrong appealed outcome was added in `appeal`
        assert_eq!(new_appealed_vote_item, vote_item);
        assert_ne!(last_appealed_vote_item, new_appealed_vote_item);
    });
}

#[test]
fn appeal_fails_if_court_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Court::appeal(RuntimeOrigin::signed(CHARLIE), 0),
            Error::<Runtime>::CourtNotFound
        );
    });
}

#[test]
fn appeal_fails_if_appeal_bond_exceeds_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        assert_noop!(
            Court::appeal(RuntimeOrigin::signed(POOR_PAUL), court_id),
            Error::<Runtime>::AppealBondExceedsBalance
        );
    });
}

#[test]
fn appeal_fails_if_max_appeals_reached() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        fill_appeals(court_id, MaxAppeals::get() as usize);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        assert_noop!(
            Court::appeal(RuntimeOrigin::signed(CHARLIE), court_id),
            Error::<Runtime>::MaxAppealsReached
        );
    });
}

#[test]
fn check_appealable_market_fails_if_market_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let now = <frame_system::Pallet<Runtime>>::block_number();
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        let court = <Courts<Runtime>>::get(court_id).unwrap();
        MarketCommons::remove_market(&court_id).unwrap();

        assert_noop!(
            Court::check_appealable_market(court_id, &court, now),
            MError::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn check_appealable_market_fails_if_dispute_mechanism_wrong() {
    ExtBuilder::default().build().execute_with(|| {
        let now = <frame_system::Pallet<Runtime>>::block_number();
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        let court = <Courts<Runtime>>::get(court_id).unwrap();

        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        MarketCommons::mutate_market(&market_id, |market| {
            market.dispute_mechanism = MarketDisputeMechanism::SimpleDisputes;
            Ok(())
        })
        .unwrap();

        assert_noop!(
            Court::check_appealable_market(court_id, &court, now),
            Error::<Runtime>::MarketDoesNotHaveCourtMechanism
        );
    });
}

#[test]
fn check_appealable_market_fails_if_not_in_appeal_period() {
    ExtBuilder::default().build().execute_with(|| {
        let now = <frame_system::Pallet<Runtime>>::block_number();
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(VotePeriod::get());

        let court = <Courts<Runtime>>::get(court_id).unwrap();

        assert_noop!(
            Court::check_appealable_market(court_id, &court, now),
            Error::<Runtime>::NotInAppealPeriod
        );
    });
}

#[test]
fn appeal_last_appeal_just_removes_auto_resolve() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        fill_appeals(court_id, (MaxAppeals::get() - 1) as usize);

        let court = <Courts<Runtime>>::get(court_id).unwrap();
        let resolve_at = court.round_ends.appeal;

        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at), vec![market_id]);

        assert_ok!(Court::appeal(RuntimeOrigin::signed(CHARLIE), court_id));

        assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at), vec![]);
    });
}

#[test]
fn appeal_adds_last_appeal() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        fill_appeals(court_id, (MaxAppeals::get() - 1) as usize);

        let last_draws = <SelectedDraws<Runtime>>::get(court_id);
        let appealed_vote_item =
            Court::get_latest_winner_vote_item(court_id, last_draws.as_slice()).unwrap();

        assert_ok!(Court::appeal(RuntimeOrigin::signed(CHARLIE), court_id));

        let court = <Courts<Runtime>>::get(court_id).unwrap();
        assert!(court.appeals.is_full());

        let last_appeal = court.appeals.last().unwrap();
        assert_eq!(last_appeal.appealed_vote_item, appealed_vote_item);
    });
}

#[test]
fn reassign_court_stakes_slashes_tardy_jurors_and_rewards_winners() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool(MaxCourtParticipants::get());
        let court_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(BOB), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(CHARLIE), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(DAVE), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(EVE), amount));

        let outcome = OutcomeReport::Scalar(42u128);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome.clone(), salt));

        let vote_item = VoteItem::Outcome(outcome);

        let draws: crate::SelectedDrawsOf<Runtime> = vec![
            Draw {
                court_participant: ALICE,
                weight: 1,
                vote: Vote::Drawn,
                slashable: MinJurorStake::get(),
            },
            Draw {
                court_participant: BOB,
                weight: 1,
                vote: Vote::Secret { commitment },
                slashable: 2 * MinJurorStake::get(),
            },
            Draw {
                court_participant: CHARLIE,
                weight: 1,
                vote: Vote::Revealed { commitment, vote_item: vote_item.clone(), salt },
                slashable: 3 * MinJurorStake::get(),
            },
            Draw {
                court_participant: DAVE,
                weight: 1,
                vote: Vote::Drawn,
                slashable: 4 * MinJurorStake::get(),
            },
            Draw {
                court_participant: EVE,
                weight: 1,
                vote: Vote::Denounced { commitment, vote_item, salt },
                slashable: 5 * MinJurorStake::get(),
            },
        ]
        .try_into()
        .unwrap();
        let old_draws = draws.clone();
        <SelectedDraws<Runtime>>::insert(court_id, draws);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + AppealPeriod::get() + 1);

        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        let _ = Court::on_resolution(&market_id, &market).unwrap();

        let free_alice_before = Balances::free_balance(ALICE);
        let free_bob_before = Balances::free_balance(BOB);
        let free_charlie_before = Balances::free_balance(CHARLIE);
        let free_dave_before = Balances::free_balance(DAVE);
        let free_eve_before = Balances::free_balance(EVE);

        assert_ok!(Court::reassign_court_stakes(RuntimeOrigin::signed(EVE), court_id));

        let free_alice_after = Balances::free_balance(ALICE);
        assert_ne!(free_alice_after, free_alice_before);
        assert_eq!(free_alice_after, free_alice_before - old_draws[ALICE as usize].slashable);

        let free_bob_after = Balances::free_balance(BOB);
        assert_ne!(free_bob_after, free_bob_before);
        assert_eq!(free_bob_after, free_bob_before - old_draws[BOB as usize].slashable);

        let free_charlie_after = Balances::free_balance(CHARLIE);
        let full_slashes = old_draws[ALICE as usize].slashable
            + old_draws[BOB as usize].slashable
            + old_draws[DAVE as usize].slashable
            + old_draws[EVE as usize].slashable;
        assert_eq!(free_charlie_after, free_charlie_before + full_slashes);

        let free_dave_after = Balances::free_balance(DAVE);
        assert_ne!(free_dave_after, free_dave_before);
        assert_eq!(free_dave_after, free_dave_before - old_draws[DAVE as usize].slashable);

        let free_eve_after = Balances::free_balance(EVE);
        assert_ne!(free_eve_after, free_eve_before);
        assert_eq!(free_eve_after, free_eve_before - old_draws[EVE as usize].slashable);
    });
}

#[test]
fn reassign_court_stakes_fails_if_court_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Court::reassign_court_stakes(RuntimeOrigin::signed(EVE), 0),
            Error::<Runtime>::CourtNotFound
        );
    });
}

#[test]
fn reassign_court_stakes_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        let _ = Court::on_resolution(&market_id, &market).unwrap().result.unwrap();

        assert_ok!(Court::reassign_court_stakes(RuntimeOrigin::signed(EVE), court_id));
        System::assert_last_event(Event::StakesReassigned { court_id }.into());
    });
}

#[test]
fn reassign_court_stakes_fails_if_juror_stakes_already_reassigned() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        let _ = Court::on_resolution(&market_id, &market).unwrap().result.unwrap();

        assert_ok!(Court::reassign_court_stakes(RuntimeOrigin::signed(EVE), court_id));

        assert_noop!(
            Court::reassign_court_stakes(RuntimeOrigin::signed(EVE), court_id),
            Error::<Runtime>::CourtAlreadyReassigned
        );
    });
}

#[test]
fn reassign_court_stakes_updates_court_status() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        let resolution_outcome = Court::on_resolution(&market_id, &market).unwrap().result.unwrap();

        let court = <Courts<Runtime>>::get(court_id).unwrap();
        let resolution_vote_item = VoteItem::Outcome(resolution_outcome);
        assert_eq!(court.status, CourtStatus::Closed { winner: resolution_vote_item });

        assert_ok!(Court::reassign_court_stakes(RuntimeOrigin::signed(EVE), court_id));

        let court = <Courts<Runtime>>::get(court_id).unwrap();
        assert_eq!(court.status, CourtStatus::Reassigned);
    });
}

#[test]
fn reassign_court_stakes_removes_draws() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        let _ = Court::on_resolution(&market_id, &market).unwrap().result.unwrap();

        let draws = <SelectedDraws<Runtime>>::get(court_id);
        assert!(!draws.is_empty());

        assert_ok!(Court::reassign_court_stakes(RuntimeOrigin::signed(EVE), court_id));

        let draws = <SelectedDraws<Runtime>>::get(court_id);
        assert!(draws.is_empty());
    });
}

#[test]
fn reassign_court_stakes_fails_if_court_not_closed() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = initialize_court();
        assert_noop!(
            Court::reassign_court_stakes(RuntimeOrigin::signed(EVE), court_id),
            Error::<Runtime>::CourtNotClosed
        );
    });
}

#[test]
fn reassign_court_stakes_decreases_active_lock() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool(MaxCourtParticipants::get());
        let court_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(BOB), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(CHARLIE), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(DAVE), amount));

        let outcome = OutcomeReport::Scalar(42u128);
        let vote_item = VoteItem::Outcome(outcome.clone());
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, outcome, salt));

        let alice_slashable = MinJurorStake::get();
        <Participants<Runtime>>::mutate(ALICE, |p_info| {
            if let Some(ref mut info) = p_info {
                info.active_lock = alice_slashable;
            }
        });
        let bob_slashable = 2 * MinJurorStake::get();
        <Participants<Runtime>>::mutate(BOB, |p_info| {
            if let Some(ref mut p_info) = p_info {
                p_info.active_lock = bob_slashable;
            }
        });
        let charlie_slashable = 3 * MinJurorStake::get();
        <Participants<Runtime>>::mutate(CHARLIE, |p_info| {
            if let Some(ref mut p_info) = p_info {
                p_info.active_lock = charlie_slashable;
            }
        });
        let dave_slashable = 4 * MinJurorStake::get();
        <Participants<Runtime>>::mutate(DAVE, |p_info| {
            if let Some(ref mut p_info) = p_info {
                p_info.active_lock = dave_slashable;
            }
        });

        let draws: crate::SelectedDrawsOf<Runtime> = vec![
            Draw {
                court_participant: ALICE,
                weight: 1,
                vote: Vote::Drawn,
                slashable: alice_slashable,
            },
            Draw {
                court_participant: BOB,
                weight: 1,
                vote: Vote::Secret { commitment },
                slashable: bob_slashable,
            },
            Draw {
                court_participant: CHARLIE,
                weight: 1,
                vote: Vote::Revealed { commitment, vote_item: vote_item.clone(), salt },
                slashable: charlie_slashable,
            },
            Draw {
                court_participant: DAVE,
                weight: 1,
                vote: Vote::Denounced { commitment, vote_item, salt },
                slashable: dave_slashable,
            },
        ]
        .try_into()
        .unwrap();
        <SelectedDraws<Runtime>>::insert(court_id, draws);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + AppealPeriod::get() + 1);

        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        let _ = Court::on_resolution(&market_id, &market).unwrap();

        assert_ok!(Court::reassign_court_stakes(RuntimeOrigin::signed(EVE), court_id));
        assert!(<Participants<Runtime>>::get(ALICE).unwrap().active_lock.is_zero());
        assert!(<Participants<Runtime>>::get(BOB).unwrap().active_lock.is_zero());
        assert!(<Participants<Runtime>>::get(CHARLIE).unwrap().active_lock.is_zero());
        assert!(<Participants<Runtime>>::get(DAVE).unwrap().active_lock.is_zero());
    });
}

#[test]
fn reassign_court_stakes_slashes_loosers_and_awards_winners() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool(MaxCourtParticipants::get());
        let court_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(BOB), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(CHARLIE), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(DAVE), amount));

        let outcome = OutcomeReport::Scalar(42u128);
        let vote_item = VoteItem::Outcome(outcome.clone());
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, vote_item.clone(), salt));

        let wrong_outcome_0 = OutcomeReport::Scalar(69u128);
        let wrong_vote_item_0 = VoteItem::Outcome(wrong_outcome_0);
        let wrong_outcome_1 = OutcomeReport::Scalar(56u128);
        let wrong_vote_item_1 = VoteItem::Outcome(wrong_outcome_1);

        let alice_slashable = MinJurorStake::get();
        let bob_slashable = 2 * MinJurorStake::get();
        let charlie_slashable = 3 * MinJurorStake::get();
        let dave_slashable = 4 * MinJurorStake::get();

        let draws: crate::SelectedDrawsOf<Runtime> = vec![
            Draw {
                court_participant: ALICE,
                weight: 1,
                vote: Vote::Revealed { commitment, vote_item: vote_item.clone(), salt },
                slashable: alice_slashable,
            },
            Draw {
                court_participant: BOB,
                weight: 1,
                vote: Vote::Revealed { commitment, vote_item: wrong_vote_item_0, salt },
                slashable: bob_slashable,
            },
            Draw {
                court_participant: CHARLIE,
                weight: 1,
                vote: Vote::Revealed { commitment, vote_item, salt },
                slashable: charlie_slashable,
            },
            Draw {
                court_participant: DAVE,
                weight: 1,
                vote: Vote::Revealed { commitment, vote_item: wrong_vote_item_1, salt },
                slashable: dave_slashable,
            },
        ]
        .try_into()
        .unwrap();
        let last_draws = draws.clone();
        <SelectedDraws<Runtime>>::insert(court_id, draws);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + AppealPeriod::get() + 1);

        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        let resolution_outcome = Court::on_resolution(&market_id, &market).unwrap().result.unwrap();
        assert_eq!(resolution_outcome, outcome);

        let free_alice_before = Balances::free_balance(ALICE);
        let free_bob_before = Balances::free_balance(BOB);
        let free_charlie_before = Balances::free_balance(CHARLIE);
        let free_dave_before = Balances::free_balance(DAVE);

        let reward_pot = Court::reward_pot(court_id);
        let tardy_or_denounced_value = 5 * MinJurorStake::get();
        let _ = Balances::deposit(&reward_pot, tardy_or_denounced_value).unwrap();

        assert_ok!(Court::reassign_court_stakes(RuntimeOrigin::signed(EVE), court_id));

        let bob_slashed = last_draws[BOB as usize].slashable;
        let dave_slashed = last_draws[DAVE as usize].slashable;
        let slashed = bob_slashed + dave_slashed + tardy_or_denounced_value;

        let winners_risked_amount = charlie_slashable + alice_slashable;

        let alice_share = Perquintill::from_rational(alice_slashable, winners_risked_amount);
        let free_alice_after = Balances::free_balance(ALICE);
        assert_eq!(free_alice_after, free_alice_before + alice_share * slashed);

        let free_bob_after = Balances::free_balance(BOB);
        assert_eq!(free_bob_after, free_bob_before - bob_slashed);

        let charlie_share = Perquintill::from_rational(charlie_slashable, winners_risked_amount);
        let free_charlie_after = Balances::free_balance(CHARLIE);
        assert_eq!(free_charlie_after, free_charlie_before + charlie_share * slashed);

        let free_dave_after = Balances::free_balance(DAVE);
        assert_eq!(free_dave_after, free_dave_before - dave_slashed);

        assert!(Balances::free_balance(reward_pot).is_zero());
    });
}

#[test]
fn reassign_court_stakes_works_for_delegations() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool(MaxCourtParticipants::get());
        let court_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(BOB), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(CHARLIE), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(DAVE), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(EVE), amount));

        let outcome = OutcomeReport::Scalar(42u128);
        let vote_item = VoteItem::Outcome(outcome.clone());
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, vote_item.clone(), salt));

        let wrong_outcome = OutcomeReport::Scalar(69u128);
        let wrong_vote_item = VoteItem::Outcome(wrong_outcome);

        let alice_slashable = MinJurorStake::get();
        let bob_slashable = 2 * MinJurorStake::get();
        let charlie_slashable = 3 * MinJurorStake::get();
        let dave_slashable = 3 * MinJurorStake::get();
        let eve_slashable = 5 * MinJurorStake::get();

        let delegated_stakes_charlie: crate::DelegatedStakesOf<Runtime> =
            vec![(ALICE, 2 * MinJurorStake::get()), (BOB, MinJurorStake::get())]
                .try_into()
                .unwrap();

        let delegated_stakes_dave: crate::DelegatedStakesOf<Runtime> =
            vec![(ALICE, 2 * MinJurorStake::get()), (BOB, MinJurorStake::get())]
                .try_into()
                .unwrap();

        let draws: crate::SelectedDrawsOf<Runtime> = vec![
            Draw {
                court_participant: ALICE,
                weight: 1,
                vote: Vote::Revealed { commitment, vote_item: vote_item.clone(), salt },
                slashable: alice_slashable,
            },
            Draw {
                court_participant: EVE,
                weight: 1,
                vote: Vote::Revealed { commitment, vote_item, salt },
                slashable: eve_slashable,
            },
            Draw {
                court_participant: BOB,
                weight: 1,
                vote: Vote::Revealed { commitment, vote_item: wrong_vote_item, salt },
                slashable: bob_slashable,
            },
            Draw {
                court_participant: CHARLIE,
                weight: 1,
                vote: Vote::Delegated { delegated_stakes: delegated_stakes_charlie.clone() },
                slashable: charlie_slashable,
            },
            Draw {
                court_participant: DAVE,
                weight: 1,
                vote: Vote::Delegated { delegated_stakes: delegated_stakes_dave.clone() },
                slashable: dave_slashable,
            },
        ]
        .try_into()
        .unwrap();
        let last_draws = draws.clone();
        <SelectedDraws<Runtime>>::insert(court_id, draws);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + AppealPeriod::get() + 1);

        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        let resolution_outcome = Court::on_resolution(&market_id, &market).unwrap().result.unwrap();
        assert_eq!(resolution_outcome, outcome);

        let free_alice_before = Balances::free_balance(ALICE);
        let free_bob_before = Balances::free_balance(BOB);
        let free_charlie_before = Balances::free_balance(CHARLIE);
        let free_dave_before = Balances::free_balance(DAVE);
        let free_eve_before = Balances::free_balance(EVE);

        let reward_pot = Court::reward_pot(court_id);
        let tardy_or_denounced_value = 5 * MinJurorStake::get();
        let _ = Balances::deposit(&reward_pot, tardy_or_denounced_value).unwrap();

        assert_ok!(Court::reassign_court_stakes(RuntimeOrigin::signed(EVE), court_id));

        let bob_slashed =
            last_draws.iter().find(|draw| draw.court_participant == BOB).unwrap().slashable;
        let charlie_delegated_bob_slashed =
            delegated_stakes_charlie.iter().find(|(acc, _)| *acc == BOB).unwrap().1;
        let dave_delegated_bob_slashed =
            delegated_stakes_dave.iter().find(|(acc, _)| *acc == BOB).unwrap().1;
        let slashed = bob_slashed
            + charlie_delegated_bob_slashed
            + dave_delegated_bob_slashed
            + tardy_or_denounced_value;

        let charlie_delegated_alice_slashable =
            delegated_stakes_charlie.iter().find(|(acc, _)| *acc == ALICE).unwrap().1;
        let dave_delegated_alice_slashable =
            delegated_stakes_dave.iter().find(|(acc, _)| *acc == ALICE).unwrap().1;
        let winners_risked_amount = charlie_delegated_alice_slashable
            + dave_delegated_alice_slashable
            + alice_slashable
            + eve_slashable;

        let alice_share = Perquintill::from_rational(alice_slashable, winners_risked_amount);
        let free_alice_after = Balances::free_balance(ALICE);
        assert_eq!(free_alice_after, free_alice_before + alice_share * slashed);

        let eve_share = Perquintill::from_rational(eve_slashable, winners_risked_amount);
        let free_eve_after = Balances::free_balance(EVE);
        assert_eq!(free_eve_after, free_eve_before + eve_share * slashed);

        let free_bob_after = Balances::free_balance(BOB);
        assert_eq!(free_bob_after, free_bob_before - bob_slashed);

        let charlie_share =
            Perquintill::from_rational(charlie_delegated_alice_slashable, winners_risked_amount);
        let free_charlie_after = Balances::free_balance(CHARLIE);
        let charlie_rewarded = charlie_share * slashed;
        assert_eq!(
            free_charlie_after,
            free_charlie_before + charlie_rewarded - charlie_delegated_bob_slashed
        );

        let dave_share =
            Perquintill::from_rational(dave_delegated_alice_slashable, winners_risked_amount);
        let free_dave_after = Balances::free_balance(DAVE);
        let dave_rewarded = dave_share * slashed;
        assert_eq!(free_dave_after, free_dave_before + dave_rewarded - dave_delegated_bob_slashed);

        assert!(Balances::free_balance(reward_pot).is_zero());
    });
}

#[test]
fn reassign_court_stakes_rewards_treasury_if_no_winner() {
    ExtBuilder::default().build().execute_with(|| {
        fill_juror_pool(MaxCourtParticipants::get());
        let court_id = initialize_court();

        let amount = MinJurorStake::get() * 100;
        assert_ok!(Court::join_court(RuntimeOrigin::signed(ALICE), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(BOB), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(CHARLIE), amount));
        assert_ok!(Court::join_court(RuntimeOrigin::signed(DAVE), amount));

        let outcome = OutcomeReport::Scalar(42u128);
        let vote_item = VoteItem::Outcome(outcome);
        let salt = <Runtime as frame_system::Config>::Hash::default();
        let commitment = BlakeTwo256::hash_of(&(ALICE, vote_item.clone(), salt));

        let wrong_outcome_0 = OutcomeReport::Scalar(69u128);
        let wrong_vote_item_0 = VoteItem::Outcome(wrong_outcome_0);
        let wrong_outcome_1 = OutcomeReport::Scalar(56u128);
        let wrong_vote_item_1 = VoteItem::Outcome(wrong_outcome_1);

        let draws: crate::SelectedDrawsOf<Runtime> = vec![
            Draw {
                court_participant: ALICE,
                weight: 1,
                vote: Vote::Revealed { commitment, vote_item: wrong_vote_item_1.clone(), salt },
                slashable: MinJurorStake::get(),
            },
            Draw {
                court_participant: BOB,
                weight: 1,
                vote: Vote::Revealed { commitment, vote_item: wrong_vote_item_0.clone(), salt },
                slashable: 2 * MinJurorStake::get(),
            },
            Draw {
                court_participant: CHARLIE,
                weight: 1,
                vote: Vote::Revealed { commitment, vote_item: wrong_vote_item_0, salt },
                slashable: 3 * MinJurorStake::get(),
            },
            Draw {
                court_participant: DAVE,
                weight: 1,
                vote: Vote::Revealed { commitment, vote_item: wrong_vote_item_1, salt },
                slashable: 4 * MinJurorStake::get(),
            },
        ]
        .try_into()
        .unwrap();
        let last_draws = draws.clone();
        <SelectedDraws<Runtime>>::insert(court_id, draws);

        run_to_block(<RequestBlock<Runtime>>::get() + 1);

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + AppealPeriod::get() + 1);

        let mut court = <Courts<Runtime>>::get(court_id).unwrap();
        court.status = CourtStatus::Closed { winner: vote_item };
        <Courts<Runtime>>::insert(court_id, court);

        let free_alice_before = Balances::free_balance(ALICE);
        let free_bob_before = Balances::free_balance(BOB);
        let free_charlie_before = Balances::free_balance(CHARLIE);
        let free_dave_before = Balances::free_balance(DAVE);

        let treasury_account = Court::treasury_account_id();
        let free_treasury_before = Balances::free_balance(treasury_account);

        assert_ok!(Court::reassign_court_stakes(RuntimeOrigin::signed(EVE), court_id));

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

        assert_eq!(Balances::free_balance(treasury_account), free_treasury_before + slashed);
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
        let court_id = initialize_court();
        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.report.as_ref().unwrap().outcome, ORACLE_REPORT);

        assert_eq!(Court::on_resolution(&market_id, &market).unwrap().result, Some(ORACLE_REPORT));
        let court = <Courts<Runtime>>::get(court_id).unwrap();
        assert_eq!(court.status, CourtStatus::Closed { winner: VoteItem::Outcome(ORACLE_REPORT) });
    });
}

#[test]
fn on_resolution_fails_if_court_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = MarketCommons::push_market(DEFAULT_MARKET).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();

        <MarketIdToCourtId<Runtime>>::insert(market_id, 0);
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
        let court_id = initialize_court();
        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();

        let resolved_outcome = OutcomeReport::Scalar(1);
        let other_outcome = OutcomeReport::Scalar(2);

        let mut court = <Courts<Runtime>>::get(court_id).unwrap();
        let mut free_balances_before = BTreeMap::new();
        let mut number = 0u128;
        let mut slashed_bonds = <BalanceOf<Runtime>>::zero();
        while (number as usize) < MaxAppeals::get() as usize {
            let bond = crate::get_appeal_bond::<Runtime>(court.appeals.len());
            let appealed_vote_item = if number % 2 == 0 {
                // The appeals are not justified,
                // because the appealed outcomes are equal to the resolved outcome.
                // it is punished to appeal the right outcome
                slashed_bonds += bond;
                VoteItem::Outcome(resolved_outcome.clone())
            } else {
                VoteItem::Outcome(other_outcome.clone())
            };

            let backer = number;
            let _ = Balances::deposit(&backer, bond).unwrap();
            assert_ok!(Balances::reserve_named(&Court::reserve_id(), &backer, bond));
            let free_balance = Balances::free_balance(backer);
            free_balances_before.insert(backer, free_balance);
            court.appeals.try_push(AppealInfo { backer, bond, appealed_vote_item }).unwrap();
            number += 1;
        }
        Courts::<Runtime>::insert(court_id, court);

        let imbalance: NegativeImbalanceOf<Runtime> =
            <pallet_balances::Pallet<Runtime> as Currency<crate::AccountIdOf<Runtime>>>::issue(
                42_000_000_000,
            );
        let prev_balance = imbalance.peek();
        let imb_remainder =
            Court::exchange(&market_id, &market, &resolved_outcome, imbalance).unwrap();
        assert_eq!(imb_remainder.result.peek(), prev_balance + slashed_bonds);

        let court = <Courts<Runtime>>::get(court_id).unwrap();
        let appeals = court.appeals;
        for AppealInfo { backer, bond, appealed_vote_item } in appeals {
            assert_eq!(Balances::reserved_balance_named(&Court::reserve_id(), &backer), 0);
            let free_balance_after = Balances::free_balance(backer);
            let free_balance_before = free_balances_before.get(&backer).unwrap();

            let resolved_vote_item = VoteItem::Outcome(resolved_outcome.clone());

            if appealed_vote_item == resolved_vote_item {
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
        let court_id = initialize_court();
        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        let court = <Courts<Runtime>>::get(court_id).unwrap();
        let appeal_end = court.round_ends.appeal;
        assert_eq!(Court::get_auto_resolve(&market_id, &market).result, Some(appeal_end));
    });
}

#[test]
fn on_global_dispute_removes_court() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = initialize_court();
        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        assert!(<Courts<Runtime>>::contains_key(court_id));
        assert_ok!(Court::on_global_dispute(&market_id, &market));
        assert!(!<Courts<Runtime>>::contains_key(court_id));
    });
}

#[test]
fn on_global_dispute_removes_draws() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = initialize_court();
        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        assert!(<SelectedDraws<Runtime>>::contains_key(court_id));
        assert_ok!(Court::on_global_dispute(&market_id, &market));
        assert!(!<SelectedDraws<Runtime>>::contains_key(court_id));
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
        <MarketIdToCourtId<Runtime>>::insert(0, 0);
        let market = DEFAULT_MARKET;
        assert_noop!(Court::on_global_dispute(&0, &market), Error::<Runtime>::CourtNotFound);
    });
}

#[test]
fn on_global_dispute_fails_if_market_report_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = initialize_court();
        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
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
        let court_id = initialize_court();
        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        let mut court = <Courts<Runtime>>::get(court_id).unwrap();
        let mut gd_outcomes = Vec::new();

        let initial_vote_amount = <BalanceOf<Runtime>>::zero();
        let treasury_account = Court::treasury_account_id();
        for number in 0..MaxAppeals::get() {
            let appealed_vote_item: VoteItem =
                VoteItem::Outcome(OutcomeReport::Scalar(number as u128));
            let backer = number as u128;
            let bond = crate::get_appeal_bond::<Runtime>(court.appeals.len());
            gd_outcomes.push(GlobalDisputeItem {
                outcome: appealed_vote_item.clone().into_outcome().unwrap(),
                owner: treasury_account,
                initial_vote_amount,
            });
            court.appeals.try_push(AppealInfo { backer, bond, appealed_vote_item }).unwrap();
        }
        Courts::<Runtime>::insert(court_id, court);
        assert_eq!(Court::on_global_dispute(&market_id, &market).unwrap().result, gd_outcomes);
    });
}

#[test]
fn choose_multiple_weighted_works() {
    ExtBuilder::default().build().execute_with(|| {
        let necessary_draws_weight = Court::necessary_draws_weight(0usize);
        for i in 0..necessary_draws_weight {
            let amount = MinJurorStake::get() + i as u128;
            let juror = i as u128;
            let _ = Balances::deposit(&juror, amount).unwrap();
            assert_ok!(Court::join_court(RuntimeOrigin::signed(juror), amount));
        }
        let random_jurors = Court::choose_multiple_weighted(necessary_draws_weight).unwrap();
        assert_eq!(
            random_jurors.iter().map(|draw| draw.weight).sum::<u32>() as usize,
            necessary_draws_weight
        );
    });
}

#[test]
fn select_participants_updates_juror_consumed_stake() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = initialize_court();
        fill_juror_pool(MaxCourtParticipants::get());
        // the last appeal is reserved for global dispute backing
        let appeal_number = (MaxAppeals::get() - 1) as usize;
        fill_appeals(court_id, appeal_number);

        let jurors = CourtPool::<Runtime>::get();
        let consumed_stake_before = jurors.iter().map(|juror| juror.consumed_stake).sum::<u128>();

        let new_draws = Court::select_participants(appeal_number).unwrap();

        let total_draw_slashable = new_draws.iter().map(|draw| draw.slashable).sum::<u128>();
        let jurors = CourtPool::<Runtime>::get();
        let consumed_stake_after = jurors.iter().map(|juror| juror.consumed_stake).sum::<u128>();
        assert_ne!(consumed_stake_before, consumed_stake_after);
        assert_eq!(consumed_stake_before + total_draw_slashable, consumed_stake_after);
    });
}

#[test_case(0usize; "first")]
#[test_case(1usize; "second")]
#[test_case(2usize; "third")]
#[test_case(3usize; "fourth")]
fn select_participants_fails_if_not_enough_jurors(appeal_number: usize) {
    ExtBuilder::default().build().execute_with(|| {
        let necessary_draws_weight = Court::necessary_draws_weight(appeal_number);
        for i in 0..(necessary_draws_weight - 1usize) {
            let amount = MinJurorStake::get() + i as u128;
            let juror = (i + 1000) as u128;
            let _ = Balances::deposit(&juror, amount).unwrap();
            assert_ok!(Court::join_court(RuntimeOrigin::signed(juror), amount));
        }

        assert_noop!(
            Court::select_participants(appeal_number),
            Error::<Runtime>::NotEnoughJurorsAndDelegatorsStake
        );
    });
}

#[test]
fn appeal_reduces_active_lock_from_old_draws() {
    ExtBuilder::default().build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(42u128);
        let (court_id, _, _) = set_alice_after_vote(outcome);

        let old_draws = <SelectedDraws<Runtime>>::get(court_id);
        assert!(!old_draws.is_empty());
        old_draws.iter().for_each(|draw| {
            let juror = draw.court_participant;
            let p_info = <Participants<Runtime>>::get(juror).unwrap();
            assert_ne!(draw.slashable, 0);
            assert_eq!(p_info.active_lock, draw.slashable);
        });

        run_blocks(VotePeriod::get() + AggregationPeriod::get() + 1);

        assert_ok!(Court::appeal(RuntimeOrigin::signed(CHARLIE), court_id));

        let new_draws = <SelectedDraws<Runtime>>::get(court_id);
        old_draws.iter().for_each(|draw| {
            let juror = draw.court_participant;
            let p_info = <Participants<Runtime>>::get(juror).unwrap();
            if let Some(new_draw) =
                new_draws.iter().find(|new_draw| new_draw.court_participant == juror)
            {
                assert_eq!(new_draw.slashable, p_info.active_lock);
            } else {
                assert_eq!(p_info.active_lock, 0);
            }
        });
    });
}

#[test]
fn on_dispute_creates_correct_court_info() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = initialize_court();
        let court = <Courts<Runtime>>::get(court_id).unwrap();
        let round_ends = court.round_ends;
        let request_block = <RequestBlock<Runtime>>::get();
        assert_eq!(round_ends.pre_vote, request_block);
        assert_eq!(round_ends.vote, round_ends.pre_vote + VotePeriod::get());
        assert_eq!(round_ends.aggregation, round_ends.vote + AggregationPeriod::get());
        assert_eq!(round_ends.appeal, round_ends.aggregation + AppealPeriod::get());
        assert_eq!(court.status, CourtStatus::Open);
        assert!(court.appeals.is_empty());
    });
}

#[test]
fn on_dispute_inserts_draws() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = initialize_court();
        let draws = <SelectedDraws<Runtime>>::get(court_id);
        assert_eq!(
            draws[0],
            Draw {
                court_participant: ALICE,
                weight: 3,
                vote: Vote::Drawn,
                slashable: 3 * MinJurorStake::get()
            }
        );
        assert_eq!(
            draws[1],
            Draw {
                court_participant: BOB,
                weight: 5,
                vote: Vote::Drawn,
                slashable: 5 * MinJurorStake::get()
            }
        );
        assert_eq!(
            draws[2],
            Draw {
                court_participant: CHARLIE,
                weight: 6,
                vote: Vote::Drawn,
                slashable: 6 * MinJurorStake::get()
            }
        );
        assert_eq!(
            draws[3],
            Draw {
                court_participant: DAVE,
                weight: 7,
                vote: Vote::Drawn,
                slashable: 7 * MinJurorStake::get()
            }
        );
        assert_eq!(
            draws[4],
            Draw {
                court_participant: EVE,
                weight: 10,
                vote: Vote::Drawn,
                slashable: 10 * MinJurorStake::get()
            }
        );
        assert_eq!(draws.len(), 5usize);
    });
}

#[test]
fn on_dispute_adds_auto_resolve() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = initialize_court();
        let court = <Courts<Runtime>>::get(court_id).unwrap();
        let resolve_at = court.round_ends.appeal;
        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        assert_eq!(MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at), vec![market_id]);
    });
}

#[test]
fn has_failed_returns_true_for_appealable_court_too_few_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = initialize_court();
        // force empty jurors pool
        <CourtPool<Runtime>>::kill();
        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        let court = <Courts<Runtime>>::get(court_id).unwrap();
        let aggregation = court.round_ends.aggregation;
        run_to_block(aggregation + 1);
        assert!(Court::has_failed(&market_id, &market).unwrap().result);
    });
}

#[test]
fn has_failed_returns_true_for_appealable_court_appeals_full() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = initialize_court();
        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();

        fill_appeals(court_id, MaxAppeals::get() as usize);

        assert!(Court::has_failed(&market_id, &market).unwrap().result);
    });
}

#[test]
fn has_failed_returns_true_for_uninitialized_court() {
    ExtBuilder::default().build().execute_with(|| {
        // force empty jurors pool
        <CourtPool<Runtime>>::kill();
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
        <MarketIdToCourtId<Runtime>>::insert(market_id, 0);
        assert!(Court::has_failed(&market_id, &market).unwrap().result);
    });
}

#[test]
fn check_necessary_draws_weight() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Court::necessary_draws_weight(0usize), 31usize);
        assert_eq!(Court::necessary_draws_weight(1usize), 63usize);
        assert_eq!(Court::necessary_draws_weight(2usize), 127usize);
        assert_eq!(Court::necessary_draws_weight(3usize), 255usize);
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

fn prepare_draws(court_id: CourtId, outcomes_with_weights: Vec<(u128, u32)>) {
    let mut draws: crate::SelectedDrawsOf<Runtime> = vec![].try_into().unwrap();
    for (i, (outcome_index, weight)) in outcomes_with_weights.iter().enumerate() {
        // offset to not conflict with other jurors
        let offset_i = (i + 1000) as u128;
        let juror = offset_i;
        let salt = BlakeTwo256::hash_of(&offset_i);
        let vote_item: VoteItem = VoteItem::Outcome(OutcomeReport::Scalar(*outcome_index));
        let commitment = BlakeTwo256::hash_of(&(juror, vote_item.clone(), salt));
        draws
            .try_push(Draw {
                court_participant: juror,
                weight: *weight,
                vote: Vote::Revealed { commitment, vote_item, salt },
                slashable: 0u128,
            })
            .unwrap();
    }
    <SelectedDraws<Runtime>>::insert(court_id, draws);
}

#[test]
fn get_winner_works() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = initialize_court();
        let outcomes_and_weights =
            vec![(1000u128, 8), (1001u128, 5), (1002u128, 42), (1003u128, 13)];
        prepare_draws(court_id, outcomes_and_weights);

        let draws = <SelectedDraws<Runtime>>::get(court_id);
        let winner = Court::get_winner(draws.as_slice(), None).unwrap();
        assert_eq!(winner.into_outcome().unwrap(), OutcomeReport::Scalar(1002u128));

        let outcomes_and_weights = vec![(1000u128, 2), (1000u128, 4), (1001u128, 4), (1001u128, 3)];
        prepare_draws(court_id, outcomes_and_weights);

        let draws = <SelectedDraws<Runtime>>::get(court_id);
        let winner = Court::get_winner(draws.as_slice(), None).unwrap();
        assert_eq!(winner.into_outcome().unwrap(), OutcomeReport::Scalar(1001u128));
    });
}

#[test]
fn get_winner_returns_none_for_no_revealed_draws() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = initialize_court();
        let draws = <SelectedDraws<Runtime>>::get(court_id);
        let winner = Court::get_winner(draws.as_slice(), None);
        assert_eq!(winner, None);
    });
}

#[test]
fn get_latest_winner_vote_item_selects_last_appealed_outcome_for_tie() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = initialize_court();
        let mut court = <Courts<Runtime>>::get(court_id).unwrap();
        // create a tie of two best outcomes
        let weights = vec![(1000u128, 42), (1001u128, 42)];
        let appealed_vote_item: VoteItem =
            VoteItem::Outcome(OutcomeReport::Scalar(weights.len() as u128));
        prepare_draws(court_id, weights);
        court
            .appeals
            .try_push(AppealInfo {
                backer: CHARLIE,
                bond: crate::get_appeal_bond::<Runtime>(1usize),
                appealed_vote_item: appealed_vote_item.clone(),
            })
            .unwrap();
        <Courts<Runtime>>::insert(court_id, court);

        let draws = <SelectedDraws<Runtime>>::get(court_id);
        let latest = Court::get_latest_winner_vote_item(court_id, draws.as_slice()).unwrap();
        assert_eq!(latest, appealed_vote_item);
        assert!(latest.into_outcome().unwrap() != ORACLE_REPORT);
    });
}

#[test]
fn get_latest_winner_vote_item_selects_oracle_report() {
    ExtBuilder::default().build().execute_with(|| {
        let court_id = initialize_court();
        let market_id = <CourtIdToMarketId<Runtime>>::get(court_id).unwrap();
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.report.unwrap().outcome, ORACLE_REPORT);
        let draws = <SelectedDraws<Runtime>>::get(court_id);
        assert_eq!(
            Court::get_latest_winner_vote_item(court_id, draws.as_slice())
                .unwrap()
                .into_outcome()
                .unwrap(),
            ORACLE_REPORT
        );
    });
}

#[test]
fn choose_multiple_weighted_returns_different_jurors_with_other_seed() {
    ExtBuilder::default().build().execute_with(|| {
        run_to_block(123);

        fill_juror_pool(MaxCourtParticipants::get());

        let nonce_0 = 42u64;
        <crate::SelectionNonce<Runtime>>::put(nonce_0);
        // randomness is mocked and purely based on the nonce
        // thus a different nonce will result in a different seed (disregarding hash collisions)
        let first_random_seed = Court::get_random_seed(nonce_0);
        let first_random_list = Court::choose_multiple_weighted(3).unwrap();

        run_blocks(1);

        let nonce_1 = 69u64;
        <crate::SelectionNonce<Runtime>>::put(nonce_1);
        let second_random_seed = Court::get_random_seed(nonce_1);

        assert_ne!(first_random_seed, second_random_seed);
        let second_random_list = Court::choose_multiple_weighted(3).unwrap();

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
        <crate::SelectionNonce<Runtime>>::put(nonce);
        let first_random_seed = Court::get_random_seed(nonce);

        run_blocks(1);

        <crate::SelectionNonce<Runtime>>::put(nonce);
        let second_random_seed = Court::get_random_seed(nonce);

        assert_eq!(first_random_seed, second_random_seed);
    });
}

#[test]
fn random_jurors_returns_a_subset_of_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        run_to_block(123);
        fill_juror_pool(MaxCourtParticipants::get());

        let jurors = <CourtPool<Runtime>>::get();

        let random_jurors = Court::choose_multiple_weighted(2).unwrap();
        for draw in random_jurors {
            assert!(jurors.iter().any(|el| el.court_participant == draw.court_participant));
        }
    });
}

#[test]
fn handle_inflation_works() {
    ExtBuilder::default().build().execute_with(|| {
        let mut jurors = <CourtPool<Runtime>>::get();
        let mut free_balances_before = BTreeMap::new();
        let jurors_list = vec![1000, 10_000, 100_000, 1_000_000, 10_000_000];
        run_to_block(InflationPeriod::get());
        let joined_at = <frame_system::Pallet<Runtime>>::block_number();
        for number in jurors_list.iter() {
            let stake = *number;
            let juror = *number;
            let _ = Balances::deposit(&juror, stake).unwrap();
            free_balances_before.insert(juror, stake);
            jurors
                .try_push(CourtPoolItem {
                    stake,
                    court_participant: juror,
                    consumed_stake: 0,
                    joined_at,
                })
                .unwrap();
        }
        <CourtPool<Runtime>>::put(jurors.clone());

        let inflation_period = InflationPeriod::get();
        run_blocks(inflation_period);
        let now = <frame_system::Pallet<Runtime>>::block_number();
        Court::handle_inflation(now);

        let free_balance_after_0 = Balances::free_balance(jurors_list[0]);
        assert_eq!(free_balance_after_0 - free_balances_before[&jurors_list[0]], 43_286_841);

        let free_balance_after_1 = Balances::free_balance(jurors_list[1]);
        assert_eq!(free_balance_after_1 - free_balances_before[&jurors_list[1]], 432_868_409);

        let free_balance_after_2 = Balances::free_balance(jurors_list[2]);
        assert_eq!(free_balance_after_2 - free_balances_before[&jurors_list[2]], 4_328_684_088);

        let free_balance_after_3 = Balances::free_balance(jurors_list[3]);
        assert_eq!(free_balance_after_3 - free_balances_before[&jurors_list[3]], 43_286_840_884);

        let free_balance_after_4 = Balances::free_balance(jurors_list[4]);
        assert_eq!(free_balance_after_4 - free_balances_before[&jurors_list[4]], 432_868_408_838);
    });
}

#[test]
fn handle_inflation_without_waiting_one_inflation_period() {
    ExtBuilder::default().build().execute_with(|| {
        let mut jurors = <CourtPool<Runtime>>::get();
        let mut free_balances_before = BTreeMap::new();
        let jurors_list = vec![1000, 10_000, 100_000, 1_000_000, 10_000_000];
        run_to_block(InflationPeriod::get());
        let joined_at = <frame_system::Pallet<Runtime>>::block_number();
        for number in jurors_list.iter() {
            let stake = *number;
            let juror = *number;
            let _ = Balances::deposit(&juror, stake).unwrap();
            free_balances_before.insert(juror, stake);
            jurors
                .try_push(CourtPoolItem {
                    stake,
                    court_participant: juror,
                    consumed_stake: 0,
                    joined_at,
                })
                .unwrap();
        }
        <CourtPool<Runtime>>::put(jurors.clone());

        let inflation_period = InflationPeriod::get();
        run_blocks(inflation_period.saturating_sub(1));
        let now = <frame_system::Pallet<Runtime>>::block_number();
        Court::handle_inflation(now);

        let free_balance_after_0 = Balances::free_balance(jurors_list[0]);
        assert_eq!(free_balance_after_0 - free_balances_before[&jurors_list[0]], 0);

        let free_balance_after_1 = Balances::free_balance(jurors_list[1]);
        assert_eq!(free_balance_after_1 - free_balances_before[&jurors_list[1]], 0);

        let free_balance_after_2 = Balances::free_balance(jurors_list[2]);
        assert_eq!(free_balance_after_2 - free_balances_before[&jurors_list[2]], 0);

        let free_balance_after_3 = Balances::free_balance(jurors_list[3]);
        assert_eq!(free_balance_after_3 - free_balances_before[&jurors_list[3]], 0);

        let free_balance_after_4 = Balances::free_balance(jurors_list[4]);
        assert_eq!(free_balance_after_4 - free_balances_before[&jurors_list[4]], 0);
    });
}
