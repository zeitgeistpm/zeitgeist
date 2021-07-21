#![cfg(all(feature = "mock", test))]

use crate::{
    events::{CommonPoolEventParams, PoolAssetEvent, PoolAssetsEvent, SwapEvent},
    mock::*,
};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};
use orml_traits::MultiCurrency;
use zeitgeist_primitives::{
    constants::BASE,
    types::{Asset, MarketId, MarketType, OutcomeReport},
};

pub const ASSET_A: Asset<MarketId> = Asset::CategoricalOutcome(0, 65);
pub const ASSET_B: Asset<MarketId> = Asset::CategoricalOutcome(0, 66);
pub const ASSET_C: Asset<MarketId> = Asset::CategoricalOutcome(0, 67);
pub const ASSET_D: Asset<MarketId> = Asset::CategoricalOutcome(0, 68);
pub const ASSET_E: Asset<MarketId> = Asset::CategoricalOutcome(0, 69);

pub const ASSETS: [Asset<MarketId>; 4] = [ASSET_A, ASSET_B, ASSET_C, ASSET_D];

const _1: u128 = BASE;
const _2: u128 = 2 * BASE;
const _3: u128 = 3 * BASE;
const _4: u128 = 4 * BASE;
const _5: u128 = 5 * BASE;
const _8: u128 = 8 * BASE;
const _20: u128 = 20 * BASE;
const _24: u128 = 24 * BASE;
const _25: u128 = 25 * BASE;
const _26: u128 = 26 * BASE;
const _99: u128 = 99 * BASE;
const _100: u128 = 100 * BASE;
const _101: u128 = 101 * BASE;
const _105: u128 = 105 * BASE;

#[test]
fn allows_the_full_user_lifecycle() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice();

        assert_ok!(Swaps::pool_join(alice_signed(), 0, _5, vec!(_25, _25, _25, _25),));

        let asset_a_bal = Currencies::free_balance(ASSET_A, &ALICE);
        let asset_b_bal = Currencies::free_balance(ASSET_B, &ALICE);

        // swap_exact_amount_in
        let spot_price = Swaps::get_spot_price(0, ASSET_A, ASSET_B).unwrap();
        assert_eq!(spot_price, _1);

        let pool_account = Swaps::pool_account_id(0);

        let in_balance = Currencies::free_balance(ASSET_A, &pool_account);
        assert_eq!(in_balance, _105);

        let expected = crate::math::calc_out_given_in(
            in_balance,
            _2,
            Currencies::free_balance(ASSET_B, &pool_account),
            _2,
            _1,
            0,
        )
        .unwrap();

        assert_ok!(Swaps::swap_exact_amount_in(
            alice_signed(),
            0,
            ASSET_A,
            _1,
            ASSET_B,
            _1 / 2,
            _2,
        ));

        let asset_a_bal_after = Currencies::free_balance(ASSET_A, &ALICE);
        assert_eq!(asset_a_bal_after, asset_a_bal - _1);

        let asset_b_bal_after = Currencies::free_balance(ASSET_B, &ALICE);
        assert_eq!(asset_b_bal_after - asset_b_bal, expected);

        assert_eq!(expected, 9_905_660_415);

        // swap_exact_amount_out
        let expected_in = crate::math::calc_in_given_out(
            Currencies::free_balance(ASSET_A, &pool_account),
            _2,
            Currencies::free_balance(ASSET_B, &pool_account),
            _2,
            _1,
            0,
        )
        .unwrap();

        assert_eq!(expected_in, 10_290_319_622);

        assert_ok!(Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_A, _2, ASSET_B, _1, _3,));

        let asset_a_bal_after_2 = Currencies::free_balance(ASSET_A, &ALICE);
        assert_eq!(asset_a_bal_after_2, asset_a_bal_after - expected_in);

        let asset_b_bal_after_2 = Currencies::free_balance(ASSET_B, &ALICE);
        assert_eq!(asset_b_bal_after_2 - asset_b_bal_after, _1);
    });
}

#[test]
fn assets_must_be_bounded() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool();
        assert_noop!(
            Swaps::swap_exact_amount_in(alice_signed(), 0, ASSET_A, 1, ASSET_E, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
        );
        assert_noop!(
            Swaps::swap_exact_amount_in(alice_signed(), 0, ASSET_E, 1, ASSET_A, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
        );

        assert_noop!(
            Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_A, 1, ASSET_E, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
        );
        assert_noop!(
            Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_E, 1, ASSET_A, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
        );

        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(alice_signed(), 0, ASSET_E, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
        );
        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(alice_signed(), 0, ASSET_E, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
        );

        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(alice_signed(), 0, ASSET_E, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
        );
        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(alice_signed(), 0, ASSET_E, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
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
fn ensure_which_operations_can_be_called_depending_on_the_pool_status() {
    ExtBuilder::default().build().execute_with(|| {
        use zeitgeist_primitives::traits::Swaps as _;
        create_initial_pool_with_funds_for_alice();
        assert_ok!(Swaps::pool_join(alice_signed(), 0, _1, vec!(_1, _1, _1, _1),));

        assert_ok!(Swaps::set_pool_as_stale(
            &MarketType::Categorical(0),
            0,
            &OutcomeReport::Scalar(0)
        ));

        assert_ok!(Swaps::pool_exit(alice_signed(), 0, _1, vec!(_1, _1, _1, _1)));
        assert_ok!(Swaps::pool_exit_with_exact_asset_amount(alice_signed(), 0, ASSET_A, _1, _1));
        assert_ok!(Swaps::pool_exit_with_exact_pool_amount(alice_signed(), 0, ASSET_A, _1, _1));
        assert_noop!(
            Swaps::pool_join(alice_signed(), 0, 0, vec!(_1, _1, _1, _1)),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(alice_signed(), 0, ASSET_E, 1, 1),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(alice_signed(), 0, ASSET_E, 1, 1),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::swap_exact_amount_in(alice_signed(), 0, ASSET_A, u128::MAX, ASSET_B, _1, _1),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_A, u128::MAX, ASSET_B, _1, _1),
            crate::Error::<Runtime>::PoolIsNotActive
        );
    });
}

#[test]
fn in_amount_must_be_equal_or_less_than_max_in_ratio() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool();

        assert_noop!(
            Swaps::swap_exact_amount_in(alice_signed(), 0, ASSET_A, u128::MAX, ASSET_B, _1, _1,),
            crate::Error::<Runtime>::MaxInRatio
        );

        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(alice_signed(), 0, ASSET_A, u128::MAX, 1),
            crate::Error::<Runtime>::MaxInRatio
        );
    });
}

#[test]
fn only_root_can_call_admin_set_pool_as_stale() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Swaps::admin_set_pool_as_stale(
                alice_signed(),
                MarketType::Scalar((0, 0)),
                0,
                OutcomeReport::Scalar(1)
            ),
            BadOrigin
        );

        create_initial_pool_with_funds_for_alice();
        assert_ok!(Swaps::pool_join(alice_signed(), 0, _1, vec!(_1, _1, _1, _1),));
        assert_ok!(Swaps::admin_set_pool_as_stale(
            Origin::root(),
            MarketType::Scalar((0, 0)),
            0,
            OutcomeReport::Scalar(1)
        ),);
    });
}

#[test]
fn out_amount_must_be_equal_or_less_than_max_out_ratio() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool();

        assert_noop!(
            Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_A, _1, ASSET_B, u128::MAX, _1,),
            crate::Error::<Runtime>::MaxOutRatio
        );

        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(alice_signed(), 0, ASSET_A, u128::MAX, 1),
            crate::Error::<Runtime>::MaxOutRatio
        );
    });
}

#[test]
fn pool_amount_must_not_be_zero() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice();

        assert_noop!(
            Swaps::pool_join(alice_signed(), 0, 0, vec!(_1, _1, _1, _1)),
            crate::Error::<Runtime>::MathApproximation
        );

        assert_noop!(
            Swaps::pool_exit(alice_signed(), 0, 0, vec!(_1, _1, _1, _1)),
            crate::Error::<Runtime>::MathApproximation
        );
    });
}

#[test]
fn pool_exit_decreases_correct_pool_parameters() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();

        assert_ok!(Swaps::pool_join(alice_signed(), 0, _1, vec!(_1, _1, _1, _1),));

        assert_ok!(Swaps::pool_exit(alice_signed(), 0, _1, vec!(_1, _1, _1, _1),));

        assert!(event_exists(crate::Event::PoolExit(PoolAssetsEvent {
            bounds: vec!(_1, _1, _1, _1),
            cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
            transferred: vec!(_1 + 1, _1 + 1, _1 + 1, _1 + 1),
        })));
        assert_all_parameters(
            [_25 + 1, _25 + 1, _25 + 1, _25 + 1],
            0,
            [_100 - 1, _100 - 1, _100 - 1, _100 - 1],
            _100,
        );
    })
}

#[test]
fn pool_exit_with_exact_pool_amount_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(alice_signed(), 0, ASSET_A, _5, 0));
        let pool_shares = Currencies::free_balance(Swaps::pool_shares_id(0), &ALICE);
        assert_ok!(Swaps::pool_exit_with_exact_pool_amount(
            alice_signed(),
            0,
            ASSET_A,
            pool_shares,
            _4
        ));
        assert!(event_exists(crate::Event::PoolExitWithExactPoolAmount(PoolAssetEvent {
            bound: _4,
            cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
            transferred: _5 - 335,
        })));
        assert_all_parameters([_25 - 335, _25, _25, _25], 0, [_100 + 335, _100, _100, _100], _100)
    });
}

#[test]
fn pool_exit_with_exact_asset_amount_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();
        let asset_before_join = Currencies::free_balance(ASSET_A, &ALICE);
        assert_ok!(Swaps::pool_join_with_exact_pool_amount(alice_signed(), 0, ASSET_A, _1, _5));
        let asset_after_join = asset_before_join - Currencies::free_balance(ASSET_A, &ALICE);
        assert_ok!(Swaps::pool_exit_with_exact_asset_amount(
            alice_signed(),
            0,
            ASSET_A,
            asset_after_join - 1000,
            _1
        ));
        assert!(event_exists(crate::Event::PoolExitWithExactAssetAmount(PoolAssetEvent {
            bound: _1,
            cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
            transferred: asset_after_join - 1000,
        })));
        assert_eq!(asset_after_join, 40604010000);
        assert_all_parameters(
            [_25 - 1000, _25, _25, _25],
            100,
            [_100 + 1000, _100, _100, _100],
            1000000000100,
        )
    });
}

#[test]
fn pool_join_increases_correct_pool_parameters() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();

        assert_ok!(Swaps::pool_join(alice_signed(), 0, _5, vec!(_25, _25, _25, _25),));
        assert!(event_exists(crate::Event::PoolJoin(PoolAssetsEvent {
            bounds: vec!(_25, _25, _25, _25),
            cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
            transferred: vec!(_5, _5, _5, _5),
        })));
        assert_all_parameters([_20, _20, _20, _20], _5, [_105, _105, _105, _105], _105);
    })
}

#[test]
fn pool_join_with_exact_asset_amount_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();
        let alice_sent = _1;
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            alice_signed(),
            0,
            ASSET_A,
            alice_sent,
            0
        ));
        assert!(event_exists(crate::Event::PoolJoinWithExactAssetAmount(PoolAssetEvent {
            bound: 0,
            cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
            transferred: alice_sent
        })));
        let alice_received = Currencies::free_balance(Swaps::pool_shares_id(0), &ALICE);
        assert_all_parameters(
            [_25 - alice_sent, _25, _25, _25],
            alice_received,
            [_100 + alice_sent, _100, _100, _100],
            _100 + alice_received,
        );
    });
}

#[test]
fn pool_join_with_exact_pool_amount_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();
        let alice_initial = Currencies::free_balance(ASSET_A, &ALICE);
        let alice_sent = _1;
        assert_ok!(Swaps::pool_join_with_exact_pool_amount(
            alice_signed(),
            0,
            ASSET_A,
            alice_sent,
            _5
        ));
        let asset_amount = alice_initial - Currencies::free_balance(ASSET_A, &ALICE);
        assert!(event_exists(crate::Event::PoolJoinWithExactPoolAmount(PoolAssetEvent {
            bound: _5,
            cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
            transferred: asset_amount,
        })));
        let alice_received = alice_initial - Currencies::free_balance(ASSET_A, &ALICE);
        assert_eq!(alice_received, 40604010000);
        assert_all_parameters(
            [_25 - alice_received, _25, _25, _25],
            alice_sent,
            [_100 + alice_received, _100, _100, _100],
            _100 + alice_sent,
        );
    });
}

#[test]
fn provided_values_len_must_equal_assets_len() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool();
        assert_noop!(
            Swaps::pool_join(alice_signed(), 0, _5, vec![]),
            crate::Error::<Runtime>::ProvidedValuesLenMustEqualAssetsLen
        );
        assert_noop!(
            Swaps::pool_exit(alice_signed(), 0, _5, vec![]),
            crate::Error::<Runtime>::ProvidedValuesLenMustEqualAssetsLen
        );
    });
}

#[test]
fn swap_exact_amount_in_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
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
        assert!(event_exists(crate::Event::SwapExactAmountIn(SwapEvent {
            asset_amount_in: _1,
            asset_amount_out: 9900990100,
            asset_bound: _1 / 2,
            cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
            max_price: _2,
        })));
        assert_all_parameters(
            [_24, _25 + 9900990100, _25, _25],
            0,
            [_101, _99 + 0099009900, _100, _100],
            _100,
        );
    });
}

#[test]
fn swap_exact_amount_out_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice();
        assert_ok!(Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_A, _2, ASSET_B, _1, _3,));
        assert!(event_exists(crate::Event::SwapExactAmountOut(SwapEvent {
            asset_amount_in: 10101010100,
            asset_amount_out: _1,
            asset_bound: _2,
            cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
            max_price: _3,
        })));
        assert_all_parameters(
            [239898989900, _26, _25, _25],
            0,
            [_101 + 0101010100, _99, _100, _100],
            _100,
        );
    });
}

fn alice_signed() -> Origin {
    Origin::signed(ALICE)
}

fn create_initial_pool() {
    ASSETS.iter().cloned().for_each(|asset| {
        let _ = Currencies::deposit(asset, &BOB, _100);
    });
    assert_ok!(Swaps::create_pool(
        Origin::signed(BOB),
        ASSETS.iter().cloned().collect(),
        0,
        vec!(_2, _2, _2, _2),
    ));
}

fn create_initial_pool_with_funds_for_alice() {
    create_initial_pool();
    let _ = Currencies::deposit(ASSET_A, &ALICE, _25);
    let _ = Currencies::deposit(ASSET_B, &ALICE, _25);
    let _ = Currencies::deposit(ASSET_C, &ALICE, _25);
    let _ = Currencies::deposit(ASSET_D, &ALICE, _25);
}

fn event_exists(raw_evt: crate::Event<Runtime>) -> bool {
    let evt = Event::Swaps(raw_evt);
    frame_system::Pallet::<Runtime>::events().iter().any(|e| e.event == evt)
}

fn assert_all_parameters(
    alice_assets: [u128; 4],
    alice_pool_assets: u128,
    pool_assets: [u128; 4],
    total_issuance: u128,
) {
    let pai = Swaps::pool_account_id(0);
    let psi = Swaps::pool_shares_id(0);

    assert_eq!(Currencies::free_balance(ASSET_A, &ALICE), alice_assets[0]);
    assert_eq!(Currencies::free_balance(ASSET_B, &ALICE), alice_assets[1]);
    assert_eq!(Currencies::free_balance(ASSET_C, &ALICE), alice_assets[2]);
    assert_eq!(Currencies::free_balance(ASSET_D, &ALICE), alice_assets[3]);

    assert_eq!(Currencies::free_balance(psi, &ALICE), alice_pool_assets);

    assert_eq!(Currencies::free_balance(ASSET_A, &pai), pool_assets[0]);
    assert_eq!(Currencies::free_balance(ASSET_B, &pai), pool_assets[1]);
    assert_eq!(Currencies::free_balance(ASSET_C, &pai), pool_assets[2]);
    assert_eq!(Currencies::free_balance(ASSET_D, &pai), pool_assets[3]);

    assert_eq!(Currencies::total_issuance(psi), total_issuance);
}
