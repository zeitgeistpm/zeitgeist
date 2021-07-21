#![cfg(test)]

use crate::{
    mock::{
        Balances, Court, ExtBuilder, Origin, RandomnessCollectiveFlip, Runtime, System, ALICE, BOB,
    },
    Error, Juror, JurorStatus, Jurors,
};
use core::ops::Range;
use frame_support::{assert_noop, assert_ok, traits::Hooks};
use sp_runtime::traits::Header;
use zeitgeist_primitives::constants::BASE;

#[test]
fn exit_court_successfully_removes_a_juror() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_eq!(Jurors::<Runtime>::iter().count(), 1);
        assert_ok!(Court::exit_court(Origin::signed(ALICE)));
        assert_eq!(Jurors::<Runtime>::iter().count(), 0);
    });
}

#[test]
fn exit_court_will_not_remove_an_unknown_juror() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Court::exit_court(Origin::signed(ALICE)),
            Error::<Runtime>::JurorDoesNotExists
        );
    });
}

#[test]
fn join_court_reserves_balance_according_to_the_number_of_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Balances::free_balance(ALICE), 1000 * BASE);
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_eq!(Balances::free_balance(ALICE), 998 * BASE);
        assert_eq!(Balances::reserved_balance(ALICE), 2 * BASE);

        assert_eq!(Balances::free_balance(BOB), 1000 * BASE);
        assert_ok!(Court::join_court(Origin::signed(BOB)));
        assert_eq!(Balances::free_balance(BOB), 996 * BASE);
        assert_eq!(Balances::reserved_balance(BOB), 4 * BASE);
    });
}

#[test]
fn join_court_successfully_stores_a_juror() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_eq!(
            Jurors::<Runtime>::iter().next().unwrap(),
            (ALICE, Juror { staked: 2 * BASE, status: JurorStatus::Ok })
        );
    });
}

#[test]
fn join_court_will_not_insert_an_already_stored_juror() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_noop!(
            Court::join_court(Origin::signed(ALICE)),
            Error::<Runtime>::JurorAlreadyExists
        );
    });
}

#[test]
fn random_jurors_return_a_subset_of_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        const START_BLOCK: u64 = 123;
        setup_blocks(1..START_BLOCK);
        let jurors = [
            (0, Juror { staked: 1, status: JurorStatus::Ok }),
            (0, Juror { staked: 2, status: JurorStatus::Tardy }),
            (0, Juror { staked: 3, status: JurorStatus::Ok }),
            (0, Juror { staked: 4, status: JurorStatus::Tardy }),
            (0, Juror { staked: 5, status: JurorStatus::Ok }),
            (0, Juror { staked: 6, status: JurorStatus::Ok }),
            (0, Juror { staked: 7, status: JurorStatus::Ok }),
        ];
        let random_jurors = Court::random_jurors(&jurors, 2);
        let mut at_least_one_is_different = false;
        for n in 0..1000 {
            setup_blocks(START_BLOCK..START_BLOCK + n);
            if random_jurors != Court::random_jurors(&jurors, 2) {
                at_least_one_is_different = true;
                break;
            }
        }
        assert_eq!(at_least_one_is_different, true);
    });
}

fn setup_blocks(range: Range<u64>) {
    let mut parent_hash = System::parent_hash();

    for i in range {
        System::initialize(&i, &parent_hash, &Default::default(), frame_system::InitKind::Full);
        RandomnessCollectiveFlip::on_initialize(i);

        let header = System::finalize();
        parent_hash = header.hash();
        System::set_block_number(*header.number());
    }
}
