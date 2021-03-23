use crate::{mock::*, CommonPoolEventParams, PoolAssetEvent, PoolAssetsEvent, BASE, SwapEvent};
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;
use zrml_traits::shares::Shares as SharesTrait;

pub const ASSET_A: H256 = H256::repeat_byte(65);
pub const ASSET_B: H256 = H256::repeat_byte(66);
pub const ASSET_C: H256 = H256::repeat_byte(67);
pub const ASSET_D: H256 = H256::repeat_byte(68);
pub const ASSET_E: H256 = H256::repeat_byte(69);

pub const ASSETS: [H256; 4] = [ASSET_A, ASSET_B, ASSET_C, ASSET_D];

const _1: u128 = BASE;
const _2: u128 = 2 * BASE;
const _3: u128 = 3 * BASE;
const _4 u128 = 4 * BASE;
const _5: u128 = 5 * BASE;
const _8: u128 = 8 * _BASE;
const _20: u128 = 20 * BASE;
const _25: u128 = 25 * BASE;
const _100: u128 = 100 * BASE;
const _105: u128 = 105 * BASE;

#[test]
fn allows_the_full_user_lifecycle() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice();

        assert_ok!(Swaps::pool_join(
            alice_signed(),
            0,
            _5,
            vec!(_25, _25, _25, _25),
        ));

        let asset_a_bal = Shares::free_balance(ASSET_A, &ALICE);
        let asset_b_bal = Shares::free_balance(ASSET_B, &ALICE);

        // swap_exact_amount_in
        let spot_price = Swaps::get_spot_price(0, ASSET_A, ASSET_B).unwrap();
        assert_eq!(spot_price, _1);

        let pool_account = Swaps::pool_account_id(0);

        let in_balance = Shares::free_balance(ASSET_A, &pool_account);
        assert_eq!(in_balance, _105);

        let expected = crate::math::calc_out_given_in(
            in_balance,
            _2,
            Shares::free_balance(ASSET_B, &pool_account),
            _2,
            _1,
            0,
        ).unwrap();

        assert_ok!(Swaps::swap_exact_amount_in(
            alice_signed(),
            0,
            ASSET_A,
            _1,
            ASSET_B,
            _1 / 2,
            _2,
        ));

        let asset_a_bal_after = Shares::free_balance(ASSET_A, &ALICE);
        assert_eq!(asset_a_bal_after, asset_a_bal - _1);

        let asset_b_bal_after = Shares::free_balance(ASSET_B, &ALICE);
        assert_eq!(asset_b_bal_after - asset_b_bal, expected);

        assert_eq!(expected, 9_905_660_415);

        // swap_exact_amount_out
        let expected_in = crate::math::calc_in_given_out(
            Shares::free_balance(ASSET_A, &pool_account),
            _2,
            Shares::free_balance(ASSET_B, &pool_account),
            _2,
            _1,
            0,
        ).unwrap();

        assert_eq!(expected_in, 10_290_319_622);

        assert_ok!(Swaps::swap_exact_amount_out(
            alice_signed(),
            0,
            ASSET_A,
            _2,
            ASSET_B,
            _1,
            _3,
        ));

        let asset_a_bal_after_2 = Shares::free_balance(ASSET_A, &ALICE);
        assert_eq!(asset_a_bal_after_2, asset_a_bal_after - expected_in);

        let asset_b_bal_after_2 = Shares::free_balance(ASSET_B, &ALICE);
        assert_eq!(asset_b_bal_after_2 - asset_b_bal_after, _1);
    });
}

#[test]
fn assets_must_be_bounded() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool();
        assert_noop!(
            Swaps::swap_exact_amount_in(alice_signed(), 0, ASSET_A, 1, ASSET_E, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );
        assert_noop!(
            Swaps::swap_exact_amount_in(alice_signed(), 0, ASSET_E, 1, ASSET_A, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );

        assert_noop!(
            Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_A, 1, ASSET_E, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );
        assert_noop!(
            Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_E, 1, ASSET_A, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );

        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(alice_signed(), 0, ASSET_E, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );
        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(alice_signed(), 0, ASSET_E, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );

        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(alice_signed(), 0, ASSET_E, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );
        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(alice_signed(), 0, ASSET_E, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );
    });
}

#[test]
fn create_pool_generates_a_new_pool_with_correct_parameters() {
    ExtBuilder::default().build().execute_with(|| {
        let next_pool_before = Swaps::next_pool_id();
        assert_eq!(next_pool_before, 0);

        create_initial_pool();

        let next_pool_after = Swaps::next_pool_id();
        assert_eq!(next_pool_after, 1);

        let pool = Swaps::pools(0).unwrap();

        assert_eq!(pool.assets, ASSETS.iter().cloned().collect::<Vec<_>>());
        assert_eq!(pool.swap_fee, 0);
        assert_eq!(pool.total_weight, _8);

        assert_eq!(*pool.weights.get(&ASSET_A).unwrap(), _2);
        assert_eq!(*pool.weights.get(&ASSET_B).unwrap(), _2);
        assert_eq!(*pool.weights.get(&ASSET_C).unwrap(), _2);
        assert_eq!(*pool.weights.get(&ASSET_D).unwrap(), _2);
    });
}

#[test]
fn pool_exit_with_exact_asset_amount_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Module::<Test>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            alice_signed(),
            0,
            ASSET_A,
            _5,
            0
        ));
        let pool_shares = Shares::free_balance(Swaps::pool_shares_id(0), &ALICE);
        assert_ok!(Swaps::pool_exit_with_exact_pool_amount(
            alice_signed(),
            0,
            ASSET_A,
            pool_shares,
            _4
        ));
        assert_eq!(Shares::free_balance(ASSET_A, &ALICE), _25 - 335); //ok
    });
}

#[test]
fn pool_exit_with_exact_pool_amount_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Module::<Test>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();
        assert_ok!(Swaps::pool_join_with_exact_pool_amount(
            alice_signed(),
            0,
            ASSET_A,
            _1,
            _1 / 4 // calculated
        ));
        let asset_after = Shares::free_balance(ASSET_A, &ALICE);
        assert_eq!(asset_after, 25 * BASE - BASE / 4 + 9320700);
        assert_ok!(Swaps::pool_exit_with_exact_pool_amount(
            alice_signed(),
            0,
            ASSET_A,
            BASE,
            BASE / 4
        ));
        // assert!(event_exists(crate::RawEvent::PoolExitWithExactPoolAmount(
        //     PoolAssetEvent {
        //         bound: BASE,
        //         cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
        //         transferred: BASE
        //     }
        // )));

        assert_eq!(Shares::free_balance(Swaps::pool_shares_id(0), &ALICE), 0);
        assert_eq!(Shares::free_balance(ASSET_A, &ALICE), _25);
    });
}

#[test]
fn in_amount_must_be_equal_or_less_than_max_in_ratio() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool();

        assert_noop!(
            Swaps::swap_exact_amount_in(
                alice_signed(),
                0,
                ASSET_A,
                u128::MAX,
                ASSET_B,
                _1,
                _1,
            ),
            crate::Error::<Test>::MaxInRatio
        );

        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(alice_signed(), 0, ASSET_A, u128::MAX, 1),
            crate::Error::<Test>::MaxInRatio
        );
    });
}

#[test]
fn pool_join_with_exact_asset_amount_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Module::<Test>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            alice_signed(),
            0,
            ASSET_A,
            BASE,
            0
        ));
        assert!(event_exists(crate::RawEvent::PoolJoinWithExactAssetAmount(
            PoolAssetEvent {
                bound: 0,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                transferred: BASE,
            }
        )));
        assert_eq!(Shares::free_balance(ASSET_A, &ALICE), 240000000000);
    });
}

#[test]
fn pool_join_with_exact_pool_amount_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Module::<Test>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();
        assert_ok!(Swaps::pool_join_with_exact_pool_amount(
            alice_signed(),
            0,
            ASSET_A,
            BASE,
            BASE
        ));
        // assert!(event_exists(crate::RawEvent::PoolJoinWithExactPoolAmount(
        //     PoolAssetEvent {
        //         bound: BASE,
        //         cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
        //         transferred: 2500000000,
        //     }
        // )));
        assert_eq!(Shares::free_balance(ASSET_A, &ALICE), 247500000000);
    });
}

#[test]
fn out_amount_must_be_equal_or_less_than_max_out_ratio() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool();

        assert_noop!(
            Swaps::swap_exact_amount_out(
                alice_signed(),
                0,
                ASSET_A,
                _1,
                ASSET_B,
                u128::MAX,
                _1,
            ),
            crate::Error::<Test>::MaxOutRatio
        );

        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(alice_signed(), 0, ASSET_A, u128::MAX, 1),
            crate::Error::<Test>::MaxOutRatio
        );
    });
}

#[test]
fn pool_amount_must_not_be_zero() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice();

        assert_noop!(
            Swaps::pool_join(alice_signed(), 0, 0, vec!(_1, _1, _1, _1)),
            crate::Error::<Test>::MathApproximation
        );

        assert_noop!(
            Swaps::pool_exit(alice_signed(), 0, 0, vec!(_1, _1, _1, _1)),
            crate::Error::<Test>::MathApproximation
        );
    });
}

#[test]
fn pool_exit_decreases_correct_pool_parameters() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Module::<Test>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();

        assert_ok!(Swaps::pool_join(
            alice_signed(),
            0,
            _1,
            vec!(_1, _1, _1, _1),
        ));

        assert_ok!(Swaps::pool_exit(
            alice_signed(),
            0,
            _1,
            vec!(_1, _1, _1, _1),
        ));

        assert!(event_exists(crate::RawEvent::PoolExit(PoolAssetsEvent {
            bounds: vec!(_1, _1, _1, _1),
            cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
            transferred: vec!(_1 + 1, _1 + 1, _1 + 1, _1 + 1),
        })));
        assert_eq!(Shares::free_balance(Swaps::pool_shares_id(0), &ALICE), 0);
        assert_eq!(Shares::free_balance(ASSET_A, &ALICE), 25 * BASE + 1);
        assert_eq!(Shares::free_balance(ASSET_B, &ALICE), 25 * BASE + 1);
        assert_eq!(Shares::free_balance(ASSET_C, &ALICE), 25 * BASE + 1);
        assert_eq!(Shares::free_balance(ASSET_D, &ALICE), 25 * BASE + 1);
    })
}

#[test]
fn pool_join_increases_correct_pool_parameters() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Module::<Test>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();

        assert_ok!(Swaps::pool_join(
            alice_signed(),
            0,
            _5,
            vec!(_25, _25, _25, _25),
        ));
        assert!(event_exists(crate::RawEvent::PoolJoin(PoolAssetsEvent {
            bounds: vec!(_25, _25, _25, _25),
            cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
            transferred: vec!(_5, _5, _5, _5),
        })));
        assert_eq!(
            Shares::free_balance(Swaps::pool_shares_id(0), &ALICE),
            _5
        );
        assert_eq!(Shares::free_balance(ASSET_A, &ALICE), _20);
        assert_eq!(Shares::free_balance(ASSET_B, &ALICE), _20);
        assert_eq!(Shares::free_balance(ASSET_C, &ALICE), _20);
        assert_eq!(Shares::free_balance(ASSET_D, &ALICE), _20);
    })
}

#[test]
fn provided_values_len_must_equal_assets_len() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool();
        assert_noop!(
            Swaps::pool_join(alice_signed(), 0, _5, vec![]),
            crate::Error::<Test>::ProvidedValuesLenMustEqualAssetsLen
        );
        assert_noop!(
            Swaps::pool_exit(alice_signed(), 0, _5, vec![]),
            crate::Error::<Test>::ProvidedValuesLenMustEqualAssetsLen
        );
    });
}

#[test]
fn swap_exact_amount_in_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Module::<Test>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();
        assert_ok!(Swaps::swap_exact_amount_in(
            alice_signed(),
            0,
            ASSET_A,
            _1,
            ASSET_B,
            _1 / 2,
            _2,
        ));
        assert!(event_exists(crate::RawEvent::SwapExactAmountIn(
            SwapEvent {
                asset_amount_in: _1,
                asset_amount_out: 9900990100,
                asset_bound: _1 / 2,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                max_price: _2,
            }
        )));
        assert_eq!(Shares::free_balance(ASSET_A, &ALICE), 240000000000);
        assert_eq!(Shares::free_balance(ASSET_B, &ALICE), 259900990100);
    });
}

#[test]
fn swap_exact_amount_out_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Module::<Test>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();
        assert_ok!(Swaps::swap_exact_amount_out(
            alice_signed(),
            0,
            ASSET_A,
            _2,
            ASSET_B,
            _1,
            _3,
        ));
        assert!(event_exists(crate::RawEvent::SwapExactAmountOut(
            SwapEvent {
                asset_amount_in: 10101010100,
                asset_amount_out: _1,
                asset_bound: _2,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                max_price: _3,
            }
        )));
        assert_eq!(Shares::free_balance(ASSET_A, &ALICE), 239898989900);
        assert_eq!(Shares::free_balance(ASSET_B, &ALICE), 260000000000);
    });
}

#[inline]
fn alice_signed() -> Origin {
    Origin::signed(ALICE)
}

fn create_initial_pool() {
    ASSETS.iter().cloned().for_each(|asset| {
        let _ = Shares::generate(asset, &BOB, _100);
    });
    assert_ok!(Swaps::create_pool(
        Origin::signed(BOB),
        ASSETS.iter().cloned().collect(),
        vec!(_2, _2, _2, _2),
    ));
}

fn create_initial_pool_with_funds_for_alice() {
    create_initial_pool();
    let _ = Shares::generate(ASSET_A, &ALICE, _25);
    let _ = Shares::generate(ASSET_B, &ALICE, _25);
    let _ = Shares::generate(ASSET_C, &ALICE, _25);
    let _ = Shares::generate(ASSET_D, &ALICE, _25);
}

fn event_exists(raw_evt: crate::RawEvent<AccountId, Balance>) -> bool {
    let evt = TestEvent::zrml_swaps(raw_evt);
    frame_system::Module::<Test>::events()
        .iter()
        .any(|e| e.event == evt)
}
