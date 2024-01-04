// Copyright 2022-2024 Forecasting Technologies LTD.
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
//
// This file incorporates work covered by the license above but
// published without copyright notice by Balancer Labs
// (<https://balancer.finance>, contact@balancer.finance) in the
// balancer-core repository
// <https://github.com/balancer-labs/balancer-core>.

#![cfg(all(feature = "mock", test))]
#![allow(clippy::too_many_arguments)]

use crate::{
    events::{CommonPoolEventParams, PoolAssetEvent, PoolAssetsEvent, SwapEvent},
    math::calc_out_given_in,
    mock::*,
    types::PoolStatus,
    AssetOf, BalanceOf, Config, Error, Event,
};
use frame_support::{assert_err, assert_noop, assert_ok};
use more_asserts::{assert_ge, assert_le};
use orml_traits::MultiCurrency;
#[allow(unused_imports)]
use test_case::test_case;
use zeitgeist_primitives::{
    constants::BASE,
    traits::Swaps as _,
    types::{Asset, MarketId, PoolId},
};

const _1_2: u128 = BASE / 2;
const _1_10: u128 = BASE / 10;
const _1_20: u128 = BASE / 20;
const _1: u128 = BASE;
const _2: u128 = 2 * BASE;
const _3: u128 = 3 * BASE;
const _4: u128 = 4 * BASE;
const _5: u128 = 5 * BASE;
const _6: u128 = 6 * BASE;
const _8: u128 = 8 * BASE;
const _9: u128 = 9 * BASE;
const _10: u128 = 10 * BASE;
const _20: u128 = 20 * BASE;
const _24: u128 = 24 * BASE;
const _25: u128 = 25 * BASE;
const _26: u128 = 26 * BASE;
const _40: u128 = 40 * BASE;
const _50: u128 = 50 * BASE;
const _90: u128 = 90 * BASE;
const _99: u128 = 99 * BASE;
const _100: u128 = 100 * BASE;
const _101: u128 = 101 * BASE;
const _105: u128 = 105 * BASE;
const _110: u128 = 110 * BASE;
const _125: u128 = 125 * BASE;
const _150: u128 = 150 * BASE;
const _165: u128 = 165 * BASE;
const _900: u128 = 900 * BASE;
const _1234: u128 = 1234 * BASE;
const _10000: u128 = 10000 * BASE;

const DEFAULT_POOL_ID: PoolId = 0;
const DEFAULT_LIQUIDITY: u128 = _100;
const DEFAULT_WEIGHT: u128 = _2;

// Macro for comparing fixed point u128.
#[allow(unused_macros)]
macro_rules! assert_approx {
    ($left:expr, $right:expr, $precision:expr $(,)?) => {
        match (&$left, &$right, &$precision) {
            (left_val, right_val, precision_val) => {
                let diff = if *left_val > *right_val {
                    *left_val - *right_val
                } else {
                    *right_val - *left_val
                };
                if diff > $precision {
                    panic!("{} is not {}-close to {}", *left_val, *precision_val, *right_val);
                }
            }
        }
    };
}

#[test_case(vec![ASSET_A, ASSET_A]; "short vector")]
#[test_case(vec![ASSET_A, ASSET_B, ASSET_C, ASSET_D, ASSET_E, ASSET_A]; "start and end")]
#[test_case(vec![ASSET_A, ASSET_B, ASSET_C, ASSET_D, ASSET_E, ASSET_E]; "successive at end")]
#[test_case(vec![ASSET_A, ASSET_B, ASSET_C, ASSET_A, ASSET_E, ASSET_D]; "start and middle")]
fn create_pool_fails_with_duplicate_assets(assets: Vec<AssetOf<Runtime>>) {
    ExtBuilder::default().build().execute_with(|| {
        assets.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, _10000);
        });
        let asset_count = assets.len();
        assert_noop!(
            Swaps::create_pool(
                BOB,
                assets,
                0,
                DEFAULT_LIQUIDITY,
                vec![DEFAULT_WEIGHT; asset_count],
            ),
            Error::<Runtime>::SomeIdenticalAssets
        );
    });
}

#[test]
fn destroy_pool_fails_if_pool_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(0, true);
        assert_noop!(Swaps::destroy_pool(42), Error::<Runtime>::PoolDoesNotExist);
    });
}

#[test]
fn destroy_pool_correctly_cleans_up_pool() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);
        let alice_balance_before = [
            Currencies::free_balance(ASSET_A, &ALICE),
            Currencies::free_balance(ASSET_B, &ALICE),
            Currencies::free_balance(ASSET_C, &ALICE),
            Currencies::free_balance(ASSET_D, &ALICE),
        ];
        assert_ok!(Swaps::destroy_pool(DEFAULT_POOL_ID));
        assert_err!(Swaps::pool_by_id(DEFAULT_POOL_ID), Error::<Runtime>::PoolDoesNotExist);
        // Ensure that funds _outside_ of the pool are not impacted!
        // TODO(#792): Remove pool shares.
        let total_pool_shares = Currencies::total_issuance(Swaps::pool_shares_id(DEFAULT_POOL_ID));
        assert_all_parameters(alice_balance_before, 0, [0, 0, 0, 0], total_pool_shares);
    });
}

#[test]
fn destroy_pool_emits_correct_event() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool(0, true);
        assert_ok!(Swaps::destroy_pool(DEFAULT_POOL_ID));
        System::assert_last_event(Event::PoolDestroyed(DEFAULT_POOL_ID).into());
    });
}

#[test]
fn allows_the_full_user_lifecycle() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);

        assert_ok!(
            Swaps::pool_join(alice_signed(), DEFAULT_POOL_ID, _5, vec!(_25, _25, _25, _25),)
        );

        let asset_a_bal = Currencies::free_balance(ASSET_A, &ALICE);
        let asset_b_bal = Currencies::free_balance(ASSET_B, &ALICE);

        // swap_exact_amount_in
        let spot_price = Swaps::get_spot_price(&DEFAULT_POOL_ID, &ASSET_A, &ASSET_B, true).unwrap();
        assert_eq!(spot_price, _1);

        let pool_account = Swaps::pool_account_id(&DEFAULT_POOL_ID);

        let in_balance = Currencies::free_balance(ASSET_A, &pool_account);
        assert_eq!(in_balance, _105);

        let expected = calc_out_given_in(
            in_balance,
            DEFAULT_WEIGHT,
            Currencies::free_balance(ASSET_B, &pool_account),
            DEFAULT_WEIGHT,
            _1,
            0,
        )
        .unwrap();

        assert_ok!(Swaps::swap_exact_amount_in(
            alice_signed(),
            DEFAULT_POOL_ID,
            ASSET_A,
            _1,
            ASSET_B,
            Some(_1 / 2),
            Some(_2),
        ));

        let asset_a_bal_after = Currencies::free_balance(ASSET_A, &ALICE);
        assert_eq!(asset_a_bal_after, asset_a_bal - _1);

        let asset_b_bal_after = Currencies::free_balance(ASSET_B, &ALICE);
        assert_eq!(asset_b_bal_after - asset_b_bal, expected);

        assert_eq!(expected, 9_905_660_415);

        // swap_exact_amount_out
        let expected_in = crate::math::calc_in_given_out(
            Currencies::free_balance(ASSET_A, &pool_account),
            DEFAULT_WEIGHT,
            Currencies::free_balance(ASSET_B, &pool_account),
            DEFAULT_WEIGHT,
            _1,
            0,
        )
        .unwrap();

        assert_eq!(expected_in, 10_290_319_622);

        assert_ok!(Swaps::swap_exact_amount_out(
            alice_signed(),
            DEFAULT_POOL_ID,
            ASSET_A,
            Some(_2),
            ASSET_B,
            _1,
            Some(_3),
        ));

        let asset_a_bal_after_2 = Currencies::free_balance(ASSET_A, &ALICE);
        assert_eq!(asset_a_bal_after_2, asset_a_bal_after - expected_in);

        let asset_b_bal_after_2 = Currencies::free_balance(ASSET_B, &ALICE);
        assert_eq!(asset_b_bal_after_2 - asset_b_bal_after, _1);
    });
}

#[test]
fn assets_must_be_bounded() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);
        assert_ok!(Swaps::mutate_pool(0, |pool| {
            pool.weights.remove(&ASSET_B);
            Ok(())
        }));

        assert_noop!(
            Swaps::swap_exact_amount_in(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_A,
                1,
                ASSET_B,
                Some(1),
                Some(1)
            ),
            Error::<Runtime>::AssetNotBound
        );
        assert_noop!(
            Swaps::swap_exact_amount_in(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_B,
                1,
                ASSET_A,
                Some(1),
                Some(1)
            ),
            Error::<Runtime>::AssetNotBound
        );

        assert_noop!(
            Swaps::swap_exact_amount_out(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_A,
                Some(1),
                ASSET_B,
                1,
                Some(1)
            ),
            Error::<Runtime>::AssetNotBound
        );
        assert_noop!(
            Swaps::swap_exact_amount_out(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_B,
                Some(1),
                ASSET_A,
                1,
                Some(1)
            ),
            Error::<Runtime>::AssetNotBound
        );

        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_B,
                1,
                1
            ),
            Error::<Runtime>::AssetNotBound
        );
        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(alice_signed(), DEFAULT_POOL_ID, ASSET_B, 1, 1),
            Error::<Runtime>::AssetNotBound
        );

        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(alice_signed(), DEFAULT_POOL_ID, ASSET_B, 1, 1),
            Error::<Runtime>::AssetNotBound
        );
        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_B,
                1,
                1
            ),
            Error::<Runtime>::AssetNotBound
        );
    });
}

#[test]
fn create_pool_generates_a_new_pool_with_correct_parameters_for_cpmm() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);

        let next_pool_before = Swaps::next_pool_id();
        assert_eq!(next_pool_before, 0);

        let amount = DEFAULT_LIQUIDITY;
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, amount));
        });
        assert_ok!(Swaps::create_pool(BOB, ASSETS.to_vec(), 1, amount, vec!(_4, _3, _2, _1),));

        let next_pool_after = Swaps::next_pool_id();
        assert_eq!(next_pool_after, 1);

        let pool = Swaps::pools(DEFAULT_POOL_ID).unwrap();

        assert_eq!(pool.assets.clone().into_inner(), ASSETS);
        assert_eq!(pool.swap_fee, 1);
        assert_eq!(pool.total_weight, _10);

        assert_eq!(*pool.weights.get(&ASSET_A).unwrap(), _4);
        assert_eq!(*pool.weights.get(&ASSET_B).unwrap(), _3);
        assert_eq!(*pool.weights.get(&ASSET_C).unwrap(), _2);
        assert_eq!(*pool.weights.get(&ASSET_D).unwrap(), _1);

        let pool_account = Swaps::pool_account_id(&DEFAULT_POOL_ID);
        System::assert_last_event(
            Event::PoolCreate(
                CommonPoolEventParams { pool_id: next_pool_before, who: BOB },
                pool,
                amount,
                pool_account,
            )
            .into(),
        );
    });
}

#[test_case(PoolStatus::Closed; "Closed")]
fn single_asset_operations_and_swaps_fail_on_invalid_status_before_clean(status: PoolStatus) {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);
        // For this test, we need to give Alice some pool shares, as well. We don't do this in
        // `create_initial_pool_...` so that there are exacly 100 pool shares, making computations
        // in other tests easier.
        assert_ok!(Currencies::deposit(Swaps::pool_shares_id(DEFAULT_POOL_ID), &ALICE, _25));
        assert_ok!(Swaps::mutate_pool(DEFAULT_POOL_ID, |pool| {
            pool.status = status;
            Ok(())
        }));

        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_A,
                _1,
                _2
            ),
            Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_A,
                _1,
                _1_2
            ),
            Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_E,
                1,
                1
            ),
            Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(alice_signed(), DEFAULT_POOL_ID, ASSET_E, 1, 1),
            Error::<Runtime>::PoolIsNotActive
        );
        assert_ok!(Currencies::deposit(ASSET_A, &ALICE, u64::MAX.into()));
        assert_noop!(
            Swaps::swap_exact_amount_in(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_A,
                u64::MAX.into(),
                ASSET_B,
                Some(_1),
                Some(_1),
            ),
            Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::swap_exact_amount_out(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_A,
                Some(u64::MAX.into()),
                ASSET_B,
                _1,
                Some(_1),
            ),
            Error::<Runtime>::PoolIsNotActive
        );
    });
}

#[test]
fn pool_join_fails_if_pool_is_closed() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);
        assert_ok!(Swaps::close_pool(DEFAULT_POOL_ID));
        assert_noop!(
            Swaps::pool_join(
                RuntimeOrigin::signed(ALICE),
                DEFAULT_POOL_ID,
                _1,
                vec![_1, _1, _1, _1]
            ),
            Error::<Runtime>::InvalidPoolStatus,
        );
    });
}

#[test_case(_3, _3, _100, _100, 0, 10_000_000_000, 10_000_000_000)]
#[test_case(_3, _3, _100, _150, 0, 6_666_666_667, 6_666_666_667)]
#[test_case(_3, _4, _100, _100, 0, 13_333_333_333, 13_333_333_333)]
#[test_case(_3, _4, _100, _150, 0, 8_888_888_889, 8_888_888_889)]
#[test_case(_3, _6, _125, _150, 0, 16_666_666_667, 16_666_666_667)]
#[test_case(_3, _6, _125, _100, 0, 25_000_000_000, 25_000_000_000)]
#[test_case(_3, _3, _100, _100, _1_10, 11_111_111_111, 10_000_000_000)]
#[test_case(_3, _3, _100, _150, _1_10, 7_407_407_408, 6_666_666_667)]
#[test_case(_3, _4, _100, _100, _1_10, 14_814_814_814, 13_333_333_333)]
#[test_case(_3, _4, _100, _150, _1_10, 9_876_543_210, 8_888_888_889)]
#[test_case(_3, _6, _125, _150, _1_10, 18_518_518_519, 16_666_666_667)]
#[test_case(_3, _6, _125, _100, _1_10, 27_777_777_778, 25_000_000_000)]
fn get_spot_price_returns_correct_results_cpmm(
    weight_in: u128,
    weight_out: u128,
    balance_in: BalanceOf<Runtime>,
    balance_out: BalanceOf<Runtime>,
    swap_fee: BalanceOf<Runtime>,
    expected_spot_price_with_fees: BalanceOf<Runtime>,
    expected_spot_price_without_fees: BalanceOf<Runtime>,
) {
    ExtBuilder::default().build().execute_with(|| {
        // We always swap ASSET_A for ASSET_B, but we vary the weights, balances and swap fees.
        let amount_in_pool = DEFAULT_LIQUIDITY;
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, amount_in_pool));
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            swap_fee,
            amount_in_pool,
            vec!(weight_in, weight_out, _2, _3),
        ));
        let pool_account = Swaps::pool_account_id(&DEFAULT_POOL_ID);

        // Modify pool balances according to test data.
        assert_ok!(Currencies::deposit(ASSET_A, &pool_account, balance_in - amount_in_pool));
        assert_ok!(Currencies::deposit(ASSET_B, &pool_account, balance_out - amount_in_pool));

        let abs_tol = 100;
        assert_approx!(
            Swaps::get_spot_price(&DEFAULT_POOL_ID, &ASSET_A, &ASSET_B, true).unwrap(),
            expected_spot_price_with_fees,
            abs_tol,
        );
        assert_approx!(
            Swaps::get_spot_price(&DEFAULT_POOL_ID, &ASSET_A, &ASSET_B, false).unwrap(),
            expected_spot_price_without_fees,
            abs_tol,
        );
    });
}

#[test]
fn in_amount_must_be_equal_or_less_than_max_in_ratio() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(0, true);

        assert_ok!(Currencies::deposit(ASSET_A, &ALICE, u64::MAX.into()));

        assert_noop!(
            Swaps::swap_exact_amount_in(
                alice_signed(),
                0,
                ASSET_A,
                u64::MAX.into(),
                ASSET_B,
                Some(_1),
                Some(_1),
            ),
            Error::<Runtime>::MaxInRatio
        );
    });
}

#[test]
fn pool_exit_with_exact_asset_amount_satisfies_max_out_ratio_constraints() {
    ExtBuilder::default().build().execute_with(|| {
        // We make sure that the individual asset weights don't divide total weight so we trigger
        // the calculation of exp using the binomial series.
        let amount_in_pool = DEFAULT_LIQUIDITY;
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, amount_in_pool));
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            0,
            amount_in_pool,
            vec!(_2, _2, _2, _5),
        ));
        assert_ok!(Swaps::open_pool(DEFAULT_POOL_ID));

        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(
                RuntimeOrigin::signed(BOB),
                DEFAULT_POOL_ID,
                ASSET_A,
                _50,
                _10000,
            ),
            Error::<Runtime>::MaxOutRatio,
        );
    });
}

#[test]
fn pool_exit_with_exact_pool_amount_satisfies_max_in_ratio_constraints() {
    ExtBuilder::default().build().execute_with(|| {
        // We make sure that the individual asset weights don't divide total weight so we trigger
        // the calculation of exp using the binomial series.
        let amount_in_pool = DEFAULT_LIQUIDITY;
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, amount_in_pool));
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            0,
            amount_in_pool,
            vec!(_2, _2, _2, _5),
        ));
        assert_ok!(Swaps::open_pool(DEFAULT_POOL_ID));

        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(
                RuntimeOrigin::signed(BOB),
                DEFAULT_POOL_ID,
                ASSET_A,
                _50,
                _10000,
            ),
            Error::<Runtime>::MaxInRatio,
        );
    });
}

#[test]
fn pool_join_with_exact_asset_amount_satisfies_max_in_ratio_constraints() {
    ExtBuilder::default().build().execute_with(|| {
        // We make sure that the individual asset weights don't divide total weight so we trigger
        // the calculation of exp using the binomial series.
        let amount_in_pool = DEFAULT_LIQUIDITY;
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, amount_in_pool));
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            0,
            amount_in_pool,
            vec!(_2, _2, _2, _5),
        ));
        assert_ok!(Swaps::open_pool(DEFAULT_POOL_ID));
        let asset_amount = DEFAULT_LIQUIDITY;
        assert_ok!(Currencies::deposit(ASSET_A, &ALICE, asset_amount));

        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_A,
                asset_amount,
                0,
            ),
            Error::<Runtime>::MaxInRatio,
        );
    });
}

#[test]
fn pool_join_with_exact_pool_amount_satisfies_max_out_ratio_constraints() {
    // If `MaxInRatio` and `MaxOutRatio` are the same, then `MaxInRatio` will always trigger if
    // `MaxOutRatio` triggers, since the ratio pool_share_amount / total_issuance is less or equal
    // to asset_amount_out / asset_balance. But `MaxOutRatio` is verified first, so it will trigger
    // in this test.
    ExtBuilder::default().build().execute_with(|| {
        // We make sure that the individual asset weights don't divide total weight so we trigger
        // the calculation of exp using the binomial series.
        let amount_in_pool = DEFAULT_LIQUIDITY;
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, amount_in_pool));
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            0,
            amount_in_pool,
            vec!(_2, _2, _2, _5),
        ));
        assert_ok!(Swaps::open_pool(DEFAULT_POOL_ID));
        let max_asset_amount = _10000;
        assert_ok!(Currencies::deposit(ASSET_A, &ALICE, max_asset_amount));

        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_A,
                _100,
                max_asset_amount,
            ),
            Error::<Runtime>::MaxOutRatio,
        );
    });
}

#[test]
fn out_amount_must_be_equal_or_less_than_max_out_ratio() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(0, true);

        assert_noop!(
            Swaps::swap_exact_amount_out(
                alice_signed(),
                0,
                ASSET_A,
                Some(_1),
                ASSET_B,
                _50,
                Some(_1),
            ),
            Error::<Runtime>::MaxOutRatio
        );
    });
}

#[test]
fn pool_join_or_exit_raises_on_zero_value() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);

        assert_noop!(
            Swaps::pool_join(alice_signed(), DEFAULT_POOL_ID, 0, vec!(_1, _1, _1, _1)),
            Error::<Runtime>::ZeroAmount
        );

        assert_noop!(
            Swaps::pool_exit(alice_signed(), DEFAULT_POOL_ID, 0, vec!(_1, _1, _1, _1)),
            Error::<Runtime>::ZeroAmount
        );

        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(alice_signed(), DEFAULT_POOL_ID, ASSET_A, 0, 0),
            Error::<Runtime>::ZeroAmount
        );

        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_A,
                0,
                0
            ),
            Error::<Runtime>::ZeroAmount
        );

        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(alice_signed(), DEFAULT_POOL_ID, ASSET_A, 0, 0),
            Error::<Runtime>::ZeroAmount
        );

        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_A,
                0,
                0
            ),
            Error::<Runtime>::ZeroAmount
        );
    });
}

#[test]
fn pool_exit_decreases_correct_pool_parameters() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(0, true);

        assert_ok!(Swaps::pool_join(alice_signed(), DEFAULT_POOL_ID, _1, vec!(_1, _1, _1, _1),));

        assert_ok!(Swaps::pool_exit(alice_signed(), DEFAULT_POOL_ID, _1, vec!(_1, _1, _1, _1),));

        System::assert_last_event(
            Event::PoolExit(PoolAssetsEvent {
                assets: vec![ASSET_A, ASSET_B, ASSET_C, ASSET_D],
                bounds: vec![_1, _1, _1, _1],
                cpep: CommonPoolEventParams { pool_id: DEFAULT_POOL_ID, who: 0 },
                transferred: vec![_1 + 1, _1 + 1, _1 + 1, _1 + 1],
                pool_amount: _1,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 + 1, _25 + 1, _25 + 1, _25 + 1],
            0,
            [
                DEFAULT_LIQUIDITY - 1,
                DEFAULT_LIQUIDITY - 1,
                DEFAULT_LIQUIDITY - 1,
                DEFAULT_LIQUIDITY - 1,
            ],
            DEFAULT_LIQUIDITY,
        );
    })
}

#[test]
fn pool_exit_emits_correct_events() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(0, true);
        assert_ok!(Swaps::pool_exit(
            RuntimeOrigin::signed(BOB),
            DEFAULT_POOL_ID,
            _1,
            vec!(1, 2, 3, 4),
        ));
        let amount = _1 - BASE / 10; // Subtract 10% fees!
        System::assert_last_event(
            Event::PoolExit(PoolAssetsEvent {
                assets: vec![ASSET_A, ASSET_B, ASSET_C, ASSET_D],
                bounds: vec![1, 2, 3, 4],
                cpep: CommonPoolEventParams { pool_id: DEFAULT_POOL_ID, who: BOB },
                transferred: vec![amount; 4],
                pool_amount: _1,
            })
            .into(),
        );
    });
}

#[test]
fn pool_exit_decreases_correct_pool_parameters_with_exit_fee() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(0, true);

        assert_ok!(Swaps::pool_exit(
            RuntimeOrigin::signed(BOB),
            DEFAULT_POOL_ID,
            _10,
            vec!(_1, _1, _1, _1),
        ));

        let pool_account = Swaps::pool_account_id(&DEFAULT_POOL_ID);
        let pool_shares_id = Swaps::pool_shares_id(DEFAULT_POOL_ID);
        assert_eq!(Currencies::free_balance(ASSET_A, &BOB), _9);
        assert_eq!(Currencies::free_balance(ASSET_B, &BOB), _9);
        assert_eq!(Currencies::free_balance(ASSET_C, &BOB), _9);
        assert_eq!(Currencies::free_balance(ASSET_D, &BOB), _9);
        assert_eq!(Currencies::free_balance(pool_shares_id, &BOB), DEFAULT_LIQUIDITY - _10);
        assert_eq!(Currencies::free_balance(ASSET_A, &pool_account), DEFAULT_LIQUIDITY - _9);
        assert_eq!(Currencies::free_balance(ASSET_B, &pool_account), DEFAULT_LIQUIDITY - _9);
        assert_eq!(Currencies::free_balance(ASSET_C, &pool_account), DEFAULT_LIQUIDITY - _9);
        assert_eq!(Currencies::free_balance(ASSET_D, &pool_account), DEFAULT_LIQUIDITY - _9);
        assert_eq!(Currencies::total_issuance(pool_shares_id), DEFAULT_LIQUIDITY - _10);

        System::assert_last_event(
            Event::PoolExit(PoolAssetsEvent {
                assets: vec![ASSET_A, ASSET_B, ASSET_C, ASSET_D],
                bounds: vec![_1, _1, _1, _1],
                cpep: CommonPoolEventParams { pool_id: DEFAULT_POOL_ID, who: BOB },
                transferred: vec![_9, _9, _9, _9],
                pool_amount: _10,
            })
            .into(),
        );
    })
}

#[test_case(49_999_999_665, 12_272_234_300, 0, 0; "no_fees")]
#[test_case(45_082_061_850, 12_272_234_300, _1_10, 0; "with_exit_fees")]
#[test_case(46_403_174_924, 11_820_024_200, 0, _1_20; "with_swap_fees")]
#[test_case(41_836_235_739, 11_820_024_200, _1_10, _1_20; "with_both_fees")]
fn pool_exit_with_exact_pool_amount_exchanges_correct_values(
    asset_amount_expected: BalanceOf<Runtime>,
    pool_amount_expected: BalanceOf<Runtime>,
    exit_fee: BalanceOf<Runtime>,
    swap_fee: BalanceOf<Runtime>,
) {
    ExtBuilder::default().build().execute_with(|| {
        let bound = _4;
        let asset_amount_joined = _5;
        <Runtime as Config>::ExitFee::set(&exit_fee);
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(swap_fee, true);
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            alice_signed(),
            DEFAULT_POOL_ID,
            ASSET_A,
            asset_amount_joined,
            0
        ));
        let pool_amount = Currencies::free_balance(Swaps::pool_shares_id(DEFAULT_POOL_ID), &ALICE);
        assert_eq!(pool_amount, pool_amount_expected); // (This is just a sanity check)

        assert_ok!(Swaps::pool_exit_with_exact_pool_amount(
            alice_signed(),
            DEFAULT_POOL_ID,
            ASSET_A,
            pool_amount,
            bound,
        ));
        System::assert_last_event(
            Event::PoolExitWithExactPoolAmount(PoolAssetEvent {
                asset: ASSET_A,
                bound,
                cpep: CommonPoolEventParams { pool_id: DEFAULT_POOL_ID, who: 0 },
                transferred: asset_amount_expected,
                pool_amount,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 - asset_amount_joined + asset_amount_expected, _25, _25, _25],
            0,
            [
                DEFAULT_LIQUIDITY + asset_amount_joined - asset_amount_expected,
                DEFAULT_LIQUIDITY,
                DEFAULT_LIQUIDITY,
                DEFAULT_LIQUIDITY,
            ],
            DEFAULT_LIQUIDITY,
        )
    });
}

#[test_case(49_999_999_297, 12_272_234_248, 0, 0; "no_fees")]
#[test_case(45_082_061_850, 12_272_234_293, _1_10, 0; "with_exit_fees")]
#[test_case(46_403_174_873, 11_820_024_153, 0, _1_20; "with_swap_fees")]
#[test_case(41_836_235_739, 11_820_024_187, _1_10, _1_20; "with_both_fees")]
fn pool_exit_with_exact_asset_amount_exchanges_correct_values(
    asset_amount: BalanceOf<Runtime>,
    pool_amount_expected: BalanceOf<Runtime>,
    exit_fee: BalanceOf<Runtime>,
    swap_fee: BalanceOf<Runtime>,
) {
    // This test is based on `pool_exit_with_exact_pool_amount_exchanges_correct_values`. Due to
    // rounding errors, the numbers aren't _exactly_ the same, which results in this test ending up
    // with a little bit of dust in some accounts.
    ExtBuilder::default().build().execute_with(|| {
        let bound = _2;
        let asset_amount_joined = _5;
        <Runtime as Config>::ExitFee::set(&exit_fee);
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(swap_fee, true);
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            alice_signed(),
            DEFAULT_POOL_ID,
            ASSET_A,
            asset_amount_joined,
            0
        ));

        // (Sanity check for dust size)
        let pool_amount = Currencies::free_balance(Swaps::pool_shares_id(DEFAULT_POOL_ID), &ALICE);
        let abs_diff = |x, y| {
            if x < y { y - x } else { x - y }
        };
        let dust = abs_diff(pool_amount, pool_amount_expected);
        assert_le!(dust, 100);

        assert_ok!(Swaps::pool_exit_with_exact_asset_amount(
            alice_signed(),
            DEFAULT_POOL_ID,
            ASSET_A,
            asset_amount,
            bound,
        ));
        System::assert_last_event(
            Event::PoolExitWithExactAssetAmount(PoolAssetEvent {
                asset: ASSET_A,
                bound,
                cpep: CommonPoolEventParams { pool_id: DEFAULT_POOL_ID, who: 0 },
                transferred: asset_amount,
                pool_amount: pool_amount_expected,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 - asset_amount_joined + asset_amount, _25, _25, _25],
            dust,
            [
                DEFAULT_LIQUIDITY + asset_amount_joined - asset_amount,
                DEFAULT_LIQUIDITY,
                DEFAULT_LIQUIDITY,
                DEFAULT_LIQUIDITY,
            ],
            DEFAULT_LIQUIDITY + dust,
        )
    });
}

#[test]
fn pool_exit_is_not_allowed_with_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(0, true);

        // Alice has no pool shares!
        assert_noop!(
            Swaps::pool_exit(alice_signed(), DEFAULT_POOL_ID, _1, vec!(0, 0, 0, 0)),
            Error::<Runtime>::InsufficientBalance,
        );

        // Now Alice has 25 pool shares!
        let _ = Currencies::deposit(Swaps::pool_shares_id(DEFAULT_POOL_ID), &ALICE, _25);
        assert_noop!(
            Swaps::pool_exit(alice_signed(), DEFAULT_POOL_ID, _26, vec!(0, 0, 0, 0)),
            Error::<Runtime>::InsufficientBalance,
        );
    })
}

#[test]
fn pool_join_increases_correct_pool_parameters() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(0, true);

        assert_ok!(
            Swaps::pool_join(alice_signed(), DEFAULT_POOL_ID, _5, vec!(_25, _25, _25, _25),)
        );
        System::assert_last_event(
            Event::PoolJoin(PoolAssetsEvent {
                assets: vec![ASSET_A, ASSET_B, ASSET_C, ASSET_D],
                bounds: vec![_25, _25, _25, _25],
                cpep: CommonPoolEventParams { pool_id: DEFAULT_POOL_ID, who: 0 },
                transferred: vec![_5, _5, _5, _5],
                pool_amount: _5,
            })
            .into(),
        );
        assert_all_parameters([_20, _20, _20, _20], _5, [_105, _105, _105, _105], _105);
    })
}

#[test]
fn pool_join_emits_correct_events() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(0, true);
        assert_ok!(Swaps::pool_join(alice_signed(), DEFAULT_POOL_ID, _1, vec!(_1, _1, _1, _1),));
        System::assert_last_event(
            Event::PoolJoin(PoolAssetsEvent {
                assets: vec![ASSET_A, ASSET_B, ASSET_C, ASSET_D],
                bounds: vec![_1, _1, _1, _1],
                cpep: CommonPoolEventParams { pool_id: DEFAULT_POOL_ID, who: 0 },
                transferred: vec![_1, _1, _1, _1],
                pool_amount: _1,
            })
            .into(),
        );
    });
}

#[test_case(_1, 2_490_679_300, 0; "without_swap_fee")]
#[test_case(_1, 2_304_521_500, _1_10; "with_swap_fee")]
fn pool_join_with_exact_asset_amount_exchanges_correct_values(
    asset_amount: BalanceOf<Runtime>,
    pool_amount_expected: BalanceOf<Runtime>,
    swap_fee: BalanceOf<Runtime>,
) {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(swap_fee, true);
        let bound = 0;
        let alice_sent = _1;
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            alice_signed(),
            DEFAULT_POOL_ID,
            ASSET_A,
            asset_amount,
            bound,
        ));
        System::assert_last_event(
            Event::PoolJoinWithExactAssetAmount(PoolAssetEvent {
                asset: ASSET_A,
                bound,
                cpep: CommonPoolEventParams { pool_id: DEFAULT_POOL_ID, who: 0 },
                transferred: asset_amount,
                pool_amount: pool_amount_expected,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 - asset_amount, _25, _25, _25],
            pool_amount_expected,
            [
                DEFAULT_LIQUIDITY + alice_sent,
                DEFAULT_LIQUIDITY,
                DEFAULT_LIQUIDITY,
                DEFAULT_LIQUIDITY,
            ],
            DEFAULT_LIQUIDITY + pool_amount_expected,
        );
    });
}

#[test_case(_1, 40_604_010_000, 0; "without_swap_fee")]
#[test_case(_1, 43_896_227_027, _1_10; "with_swap_fee")]
fn pool_join_with_exact_pool_amount_exchanges_correct_values(
    pool_amount: BalanceOf<Runtime>,
    asset_amount_expected: BalanceOf<Runtime>,
    swap_fee: BalanceOf<Runtime>,
) {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(swap_fee, true);
        let bound = _5;
        assert_ok!(Swaps::pool_join_with_exact_pool_amount(
            alice_signed(),
            DEFAULT_POOL_ID,
            ASSET_A,
            pool_amount,
            bound,
        ));
        System::assert_last_event(
            Event::PoolJoinWithExactPoolAmount(PoolAssetEvent {
                asset: ASSET_A,
                bound,
                cpep: CommonPoolEventParams { pool_id: DEFAULT_POOL_ID, who: 0 },
                transferred: asset_amount_expected,
                pool_amount,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 - asset_amount_expected, _25, _25, _25],
            pool_amount,
            [
                DEFAULT_LIQUIDITY + asset_amount_expected,
                DEFAULT_LIQUIDITY,
                DEFAULT_LIQUIDITY,
                DEFAULT_LIQUIDITY,
            ],
            DEFAULT_LIQUIDITY + pool_amount,
        );
    });
}

#[test]
fn provided_values_len_must_equal_assets_len() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(0, true);
        assert_noop!(
            Swaps::pool_join(alice_signed(), DEFAULT_POOL_ID, _5, vec![]),
            Error::<Runtime>::ProvidedValuesLenMustEqualAssetsLen
        );
        assert_noop!(
            Swaps::pool_exit(alice_signed(), DEFAULT_POOL_ID, _5, vec![]),
            Error::<Runtime>::ProvidedValuesLenMustEqualAssetsLen
        );
    });
}

#[test]
fn swap_exact_amount_in_exchanges_correct_values_with_cpmm() {
    ExtBuilder::default().build().execute_with(|| {
        let asset_bound = Some(_1 / 2);
        let max_price = Some(_2);
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(0, true);
        assert_ok!(Swaps::swap_exact_amount_in(
            alice_signed(),
            DEFAULT_POOL_ID,
            ASSET_A,
            _1,
            ASSET_B,
            asset_bound,
            max_price,
        ));
        System::assert_last_event(
            Event::SwapExactAmountIn(SwapEvent {
                asset_amount_in: _1,
                asset_amount_out: 9900990100,
                asset_bound,
                asset_in: ASSET_A,
                asset_out: ASSET_B,
                cpep: CommonPoolEventParams { pool_id: DEFAULT_POOL_ID, who: 0 },
                max_price,
            })
            .into(),
        );
        assert_all_parameters(
            [_24, _25 + 9900990100, _25, _25],
            0,
            [
                DEFAULT_LIQUIDITY + _1,
                DEFAULT_LIQUIDITY - 9900990100,
                DEFAULT_LIQUIDITY,
                DEFAULT_LIQUIDITY,
            ],
            DEFAULT_LIQUIDITY,
        );
    });
}

#[test]
fn swap_exact_amount_in_exchanges_correct_values_with_cpmm_with_fees() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &ALICE, _25);
            let _ = Currencies::deposit(asset, &BOB, _10000);
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            BASE / 10,
            DEFAULT_LIQUIDITY,
            vec!(DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT),
        ));
        assert_ok!(Swaps::open_pool(DEFAULT_POOL_ID));

        let asset_bound = Some(_1 / 2);
        let max_price = Some(_2);
        // ALICE swaps in BASE / 0.9; this results in adjusted_in ≈ BASE in
        // `math::calc_out_given_in` so we can use the same numbers as in the test above!
        let asset_amount_in = 11_111_111_111;
        let asset_amount_out = 9_900_990_100;
        assert_ok!(Swaps::swap_exact_amount_in(
            alice_signed(),
            DEFAULT_POOL_ID,
            ASSET_A,
            asset_amount_in,
            ASSET_B,
            asset_bound,
            max_price,
        ));
        System::assert_last_event(
            Event::SwapExactAmountIn(SwapEvent {
                asset_amount_in,
                asset_amount_out,
                asset_bound,
                asset_in: ASSET_A,
                asset_out: ASSET_B,
                cpep: CommonPoolEventParams { pool_id: DEFAULT_POOL_ID, who: 0 },
                max_price,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 - asset_amount_in, _25 + asset_amount_out, _25, _25],
            0,
            [
                DEFAULT_LIQUIDITY + asset_amount_in,
                DEFAULT_LIQUIDITY - asset_amount_out,
                DEFAULT_LIQUIDITY,
                DEFAULT_LIQUIDITY,
            ],
            DEFAULT_LIQUIDITY,
        );
    });
}

#[test]
fn swap_exact_amount_in_fails_if_no_limit_is_specified() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(1, true);
        assert_noop!(
            Swaps::swap_exact_amount_in(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_A,
                _1,
                ASSET_B,
                None,
                None,
            ),
            Error::<Runtime>::LimitMissing
        );
    });
}

#[test]
fn swap_exact_amount_in_fails_if_min_asset_amount_out_is_not_satisfied_with_cpmm() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);
        // Expected amount to receive from trading BASE of A for B. See
        // swap_exact_amount_in_exchanges_correct_values_with_cpmm for details.
        let expected_amount = 9900990100;
        assert_noop!(
            Swaps::swap_exact_amount_in(
                alice_signed(),
                0,
                ASSET_A,
                _1,
                ASSET_B,
                Some(expected_amount + 1), // We expect 1 more than we will actually receive!
                None,
            ),
            Error::<Runtime>::LimitOut,
        );
    });
}

#[test]
fn swap_exact_amount_in_fails_if_max_price_is_not_satisfied_with_cpmm() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);
        // We're swapping 1:1, but due to slippage the price will exceed _1, so this should raise an
        // error:
        assert_noop!(
            Swaps::swap_exact_amount_in(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_A,
                _1,
                ASSET_B,
                None,
                Some(_1)
            ),
            Error::<Runtime>::BadLimitPrice,
        );
    });
}

#[test]
fn swap_exact_amount_out_exchanges_correct_values_with_cpmm() {
    let asset_bound = Some(_2);
    let max_price = Some(_3);
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(0, true);
        assert_ok!(Swaps::swap_exact_amount_out(
            alice_signed(),
            DEFAULT_POOL_ID,
            ASSET_A,
            asset_bound,
            ASSET_B,
            _1,
            max_price,
        ));
        System::assert_last_event(
            Event::SwapExactAmountOut(SwapEvent {
                asset_amount_in: 10101010100,
                asset_amount_out: _1,
                asset_bound,
                asset_in: ASSET_A,
                asset_out: ASSET_B,
                cpep: CommonPoolEventParams { pool_id: DEFAULT_POOL_ID, who: 0 },
                max_price,
            })
            .into(),
        );
        assert_all_parameters(
            [239898989900, _26, _25, _25],
            0,
            [
                DEFAULT_LIQUIDITY + _1 + 101010100,
                DEFAULT_LIQUIDITY - _1,
                DEFAULT_LIQUIDITY,
                DEFAULT_LIQUIDITY,
            ],
            DEFAULT_LIQUIDITY,
        );
    });
}

#[test]
fn swap_exact_amount_out_exchanges_correct_values_with_cpmm_with_fees() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &ALICE, _25);
            let _ = Currencies::deposit(asset, &BOB, _10000);
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            BASE / 10,
            DEFAULT_LIQUIDITY,
            vec!(DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT),
        ));
        assert_ok!(Swaps::open_pool(DEFAULT_POOL_ID));

        let asset_amount_out = _1;
        let asset_amount_in = 11223344556; // 10101010100 / 0.9
        let asset_bound = Some(_2);
        let max_price = Some(_3);
        assert_ok!(Swaps::swap_exact_amount_out(
            alice_signed(),
            DEFAULT_POOL_ID,
            ASSET_A,
            asset_bound,
            ASSET_B,
            asset_amount_out,
            max_price,
        ));
        System::assert_last_event(
            Event::SwapExactAmountOut(SwapEvent {
                asset_amount_in,
                asset_amount_out,
                asset_bound,
                asset_in: ASSET_A,
                asset_out: ASSET_B,
                cpep: CommonPoolEventParams { pool_id: DEFAULT_POOL_ID, who: 0 },
                max_price,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 - asset_amount_in, _25 + asset_amount_out, _25, _25],
            0,
            [
                DEFAULT_LIQUIDITY + asset_amount_in,
                DEFAULT_LIQUIDITY - asset_amount_out,
                DEFAULT_LIQUIDITY,
                DEFAULT_LIQUIDITY,
            ],
            DEFAULT_LIQUIDITY,
        );
    });
}

#[test]
fn swap_exact_amount_out_fails_if_no_limit_is_specified() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(1, true);
        assert_noop!(
            Swaps::swap_exact_amount_out(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_A,
                None,
                ASSET_B,
                _1,
                None,
            ),
            Error::<Runtime>::LimitMissing
        );
    });
}

#[test]
fn swap_exact_amount_out_fails_if_min_asset_amount_out_is_not_satisfied_with_cpmm() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);
        // Expected amount of A to swap in for receiving BASE of B. See
        // swap_exact_amount_out_exchanges_correct_values_with_cpmm for details!
        let expected_amount = 10101010100;
        assert_noop!(
            Swaps::swap_exact_amount_out(
                alice_signed(),
                0,
                ASSET_A,
                Some(expected_amount - 1), // We expect to pay 1 less than we actually have to pay!
                ASSET_B,
                _1,
                None,
            ),
            Error::<Runtime>::LimitIn,
        );
    });
}

#[test]
fn swap_exact_amount_out_fails_if_max_price_is_not_satisfied_with_cpmm() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);
        // We're swapping 1:1, but due to slippage the price will exceed 1, so this should raise an
        // error:
        assert_noop!(
            Swaps::swap_exact_amount_out(
                alice_signed(),
                DEFAULT_POOL_ID,
                ASSET_A,
                None,
                ASSET_B,
                _1,
                Some(_1)
            ),
            Error::<Runtime>::BadLimitPrice,
        );
    });
}

#[test]
fn create_pool_fails_on_too_many_assets() {
    ExtBuilder::default().build().execute_with(|| {
        let length = <Runtime as crate::Config>::MaxAssets::get();
        let assets: Vec<Asset<MarketId>> =
            (0..=length).map(|x| Asset::CategoricalOutcome(0, x)).collect::<Vec<_>>();
        let weights = vec![DEFAULT_WEIGHT; length.into()];

        assets.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, _100);
        });

        assert_noop!(
            Swaps::create_pool(BOB, assets.clone(), 0, DEFAULT_LIQUIDITY, weights,),
            Error::<Runtime>::TooManyAssets
        );
    });
}

#[test]
fn create_pool_fails_on_too_few_assets() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Swaps::create_pool(
                BOB,
                vec!(ASSET_A),
                0,
                DEFAULT_LIQUIDITY,
                vec!(DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT),
            ),
            Error::<Runtime>::TooFewAssets
        );
    });
}

#[test]
fn create_pool_fails_if_swap_fee_is_too_high() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = _100;
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, amount);
        });
        assert_noop!(
            Swaps::create_pool(
                BOB,
                ASSETS.to_vec(),
                <Runtime as crate::Config>::MaxSwapFee::get() + 1,
                amount,
                vec!(DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT),
            ),
            Error::<Runtime>::SwapFeeTooHigh
        );
    });
}

#[test]
fn join_pool_exit_pool_does_not_create_extra_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);

        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &CHARLIE, _100);
        });

        let amount = 123_456_789_123; // Strange number to force rounding errors!
        assert_ok!(Swaps::pool_join(
            RuntimeOrigin::signed(CHARLIE),
            DEFAULT_POOL_ID,
            amount,
            vec![_10000, _10000, _10000, _10000]
        ));
        assert_ok!(Swaps::pool_exit(
            RuntimeOrigin::signed(CHARLIE),
            DEFAULT_POOL_ID,
            amount,
            vec![0, 0, 0, 0]
        ));

        // Check that the pool retains more tokens than before, and that Charlie loses some tokens
        // due to fees.
        let pool_account_id = Swaps::pool_account_id(&DEFAULT_POOL_ID);
        assert_ge!(Currencies::free_balance(ASSET_A, &pool_account_id), _100);
        assert_ge!(Currencies::free_balance(ASSET_B, &pool_account_id), _100);
        assert_ge!(Currencies::free_balance(ASSET_C, &pool_account_id), _100);
        assert_ge!(Currencies::free_balance(ASSET_D, &pool_account_id), _100);
        assert_le!(Currencies::free_balance(ASSET_A, &CHARLIE), _100);
        assert_le!(Currencies::free_balance(ASSET_B, &CHARLIE), _100);
        assert_le!(Currencies::free_balance(ASSET_C, &CHARLIE), _100);
        assert_le!(Currencies::free_balance(ASSET_D, &CHARLIE), _100);
    });
}

#[test]
fn create_pool_fails_on_weight_below_minimum_weight() {
    ExtBuilder::default().build().execute_with(|| {
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, _100);
        });
        assert_noop!(
            Swaps::create_pool(
                BOB,
                ASSETS.to_vec(),
                0,
                DEFAULT_LIQUIDITY,
                vec!(
                    DEFAULT_WEIGHT,
                    <Runtime as crate::Config>::MinWeight::get() - 1,
                    DEFAULT_WEIGHT,
                    DEFAULT_WEIGHT
                ),
            ),
            Error::<Runtime>::BelowMinimumWeight,
        );
    });
}

#[test]
fn create_pool_fails_on_weight_above_maximum_weight() {
    ExtBuilder::default().build().execute_with(|| {
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, _100);
        });
        assert_noop!(
            Swaps::create_pool(
                BOB,
                ASSETS.to_vec(),
                0,
                DEFAULT_LIQUIDITY,
                vec!(
                    DEFAULT_WEIGHT,
                    <Runtime as crate::Config>::MaxWeight::get() + 1,
                    DEFAULT_WEIGHT,
                    DEFAULT_WEIGHT
                ),
            ),
            Error::<Runtime>::AboveMaximumWeight,
        );
    });
}

#[test]
fn create_pool_fails_on_total_weight_above_maximum_total_weight() {
    ExtBuilder::default().build().execute_with(|| {
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, _100);
        });
        let weight = <Runtime as crate::Config>::MaxTotalWeight::get() / 4 + 100;
        assert_noop!(
            Swaps::create_pool(BOB, ASSETS.to_vec(), 0, DEFAULT_LIQUIDITY, vec![weight; 4],),
            Error::<Runtime>::MaxTotalWeight,
        );
    });
}

#[test]
fn create_pool_fails_on_insufficient_liquidity() {
    ExtBuilder::default().build().execute_with(|| {
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, _100);
        });
        let min_balance = Swaps::min_balance_of_pool(0, ASSETS.as_ref());
        assert_noop!(
            Swaps::create_pool(
                BOB,
                ASSETS.to_vec(),
                0,
                min_balance - 1,
                vec!(DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT),
            ),
            Error::<Runtime>::InsufficientLiquidity,
        );
    });
}

#[test]
fn create_pool_succeeds_on_min_liquidity() {
    ExtBuilder::default().build().execute_with(|| {
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, _100);
        });
        // Only got one type of tokens in the pool, so we can sample the minimum balance using one
        // asset.
        let min_balance = Swaps::min_balance_of_pool(0, ASSETS.as_ref());
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            0,
            min_balance,
            vec!(DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT),
        ));
        assert_all_parameters(
            [0; 4],
            0,
            [min_balance, min_balance, min_balance, min_balance],
            min_balance,
        );
        let pool_shares_id = Swaps::pool_shares_id(DEFAULT_POOL_ID);
        assert_eq!(Currencies::free_balance(pool_shares_id, &BOB), min_balance);
    });
}

#[test]
fn create_pool_transfers_the_correct_amount_of_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, _10000);
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            0,
            _1234,
            vec!(DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT),
        ));

        let pool_shares_id = Swaps::pool_shares_id(DEFAULT_POOL_ID);
        assert_eq!(Currencies::free_balance(pool_shares_id, &BOB), _1234);
        assert_eq!(Currencies::free_balance(ASSET_A, &BOB), _10000 - _1234);
        assert_eq!(Currencies::free_balance(ASSET_B, &BOB), _10000 - _1234);
        assert_eq!(Currencies::free_balance(ASSET_C, &BOB), _10000 - _1234);
        assert_eq!(Currencies::free_balance(ASSET_D, &BOB), _10000 - _1234);

        let pool_account_id = Swaps::pool_account_id(&DEFAULT_POOL_ID);
        assert_eq!(Currencies::free_balance(ASSET_A, &pool_account_id), _1234);
        assert_eq!(Currencies::free_balance(ASSET_B, &pool_account_id), _1234);
        assert_eq!(Currencies::free_balance(ASSET_C, &pool_account_id), _1234);
        assert_eq!(Currencies::free_balance(ASSET_D, &pool_account_id), _1234);
    });
}

#[test]
fn close_pool_fails_if_pool_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(Swaps::close_pool(0), Error::<Runtime>::PoolDoesNotExist);
    });
}

#[test_case(PoolStatus::Closed)]
fn close_pool_fails_if_pool_is_not_active_or_initialized(status: PoolStatus) {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(0, true);
        assert_ok!(Swaps::mutate_pool(DEFAULT_POOL_ID, |pool| {
            pool.status = status;
            Ok(())
        }));
        assert_noop!(Swaps::close_pool(0), Error::<Runtime>::InvalidStateTransition);
    });
}

#[test]
fn close_pool_succeeds_and_emits_correct_event_if_pool_exists() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool(0, true);
        assert_ok!(Swaps::close_pool(DEFAULT_POOL_ID));
        let pool = Swaps::pool_by_id(DEFAULT_POOL_ID).unwrap();
        assert_eq!(pool.status, PoolStatus::Closed);
        System::assert_last_event(Event::PoolClosed(DEFAULT_POOL_ID).into());
    });
}

#[test]
fn open_pool_fails_if_pool_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(Swaps::open_pool(0), Error::<Runtime>::PoolDoesNotExist);
    });
}

#[test_case(PoolStatus::Open)]
fn open_pool_fails_if_pool_is_not_closed(status: PoolStatus) {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(1, true);
        assert_ok!(Swaps::mutate_pool(DEFAULT_POOL_ID, |pool| {
            pool.status = status;
            Ok(())
        }));
        assert_noop!(Swaps::open_pool(DEFAULT_POOL_ID), Error::<Runtime>::InvalidStateTransition);
    });
}

#[test]
fn open_pool_succeeds_and_emits_correct_event_if_pool_exists() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let amount = _100;
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, amount));
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            vec![ASSET_D, ASSET_B, ASSET_C, ASSET_A],
            0,
            amount,
            vec!(_1, _2, _3, _4),
        ));
        assert_ok!(Swaps::open_pool(DEFAULT_POOL_ID));
        let pool = Swaps::pool_by_id(DEFAULT_POOL_ID).unwrap();
        assert_eq!(pool.status, PoolStatus::Open);
        System::assert_last_event(Event::PoolActive(DEFAULT_POOL_ID).into());
    });
}

#[test]
fn pool_join_fails_if_max_assets_in_is_violated() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);
        assert_noop!(
            Swaps::pool_join(alice_signed(), DEFAULT_POOL_ID, _1, vec!(_1, _1, _1 - 1, _1)),
            Error::<Runtime>::LimitIn,
        );
    });
}

#[test]
fn pool_join_with_exact_asset_amount_fails_if_min_pool_tokens_is_violated() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);
        // Expected pool amount when joining with exactly BASE A.
        let expected_pool_amount = 2490679300;
        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(
                alice_signed(),
                0,
                ASSET_A,
                _1,
                expected_pool_amount + 1, // We expect 1 pool share than we will actually receive.
            ),
            Error::<Runtime>::LimitOut,
        );
    });
}

#[test]
fn pool_join_with_exact_pool_amount_fails_if_max_asset_amount_is_violated() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);
        // Expected asset amount required to joining for BASE pool share.
        let expected_asset_amount = 40604010000;
        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(
                alice_signed(),
                0,
                ASSET_A,
                _1,
                expected_asset_amount - 1, // We want to pay 1 less than we actually have to pay.
            ),
            Error::<Runtime>::LimitIn,
        );
    });
}

#[test]
fn pool_exit_fails_if_min_assets_out_is_violated() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(0, true);
        assert_ok!(Swaps::pool_join(alice_signed(), DEFAULT_POOL_ID, _1, vec!(_1, _1, _1, _1)));
        assert_noop!(
            Swaps::pool_exit(alice_signed(), DEFAULT_POOL_ID, _1, vec!(_1, _1, _1 + 1, _1)),
            Error::<Runtime>::LimitOut,
        );
    });
}

#[test]
fn pool_exit_with_exact_asset_amount_fails_if_min_pool_amount_is_violated() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&(BASE / 10));
        create_initial_pool_with_funds_for_alice(0, true);
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            alice_signed(),
            DEFAULT_POOL_ID,
            ASSET_A,
            _5,
            0
        ));
        let pool_amount = Currencies::free_balance(Swaps::pool_shares_id(DEFAULT_POOL_ID), &ALICE);
        let expected_amount = 45_082_061_850;
        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(
                alice_signed(),
                0,
                ASSET_A,
                pool_amount,
                expected_amount + 100,
            ),
            Error::<Runtime>::LimitOut,
        );
    });
}

#[test]
fn pool_exit_with_exact_pool_amount_fails_if_max_asset_amount_is_violated() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&(BASE / 10));
        create_initial_pool_with_funds_for_alice(0, true);
        let asset_before_join = Currencies::free_balance(ASSET_A, &ALICE);
        assert_ok!(Swaps::pool_join_with_exact_pool_amount(
            alice_signed(),
            DEFAULT_POOL_ID,
            ASSET_A,
            _1,
            _5
        ));
        let asset_after_join = asset_before_join - Currencies::free_balance(ASSET_A, &ALICE);
        let exit_amount = (asset_after_join * 9) / 10;
        let expected_amount = 9_984_935_413;
        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(
                alice_signed(),
                0,
                ASSET_A,
                exit_amount,
                expected_amount - 100,
            ),
            Error::<Runtime>::LimitIn,
        );
    });
}

#[test]
fn create_pool_correctly_associates_weights_with_assets() {
    ExtBuilder::default().build().execute_with(|| {
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, _10000);
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            vec![ASSET_D, ASSET_B, ASSET_C, ASSET_A],
            0,
            DEFAULT_LIQUIDITY,
            vec!(_1, _2, _3, _4),
        ));
        let pool = Swaps::pool_by_id(0).unwrap();
        assert_eq!(pool.weights[&ASSET_A], _4);
        assert_eq!(pool.weights[&ASSET_B], _2);
        assert_eq!(pool.weights[&ASSET_C], _3);
        assert_eq!(pool.weights[&ASSET_D], _1);
    });
}

#[test]
fn single_asset_join_and_exit_are_inverse() {
    // Sanity check for verifying that single-asset join/exits are inverse and that the user can't
    // steal tokens from the pool using these functions.
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0);
        let asset = ASSET_B;
        let amount_in = _1;
        create_initial_pool(0, true);
        assert_ok!(Currencies::deposit(asset, &ALICE, amount_in));
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            RuntimeOrigin::signed(ALICE),
            DEFAULT_POOL_ID,
            asset,
            amount_in,
            0,
        ));
        let pool_amount = Currencies::free_balance(Swaps::pool_shares_id(DEFAULT_POOL_ID), &ALICE);
        assert_ok!(Swaps::pool_exit_with_exact_pool_amount(
            RuntimeOrigin::signed(ALICE),
            DEFAULT_POOL_ID,
            asset,
            pool_amount,
            0,
        ));
        let amount_out = Currencies::free_balance(asset, &ALICE);
        assert_le!(amount_out, amount_in);
        assert_approx!(amount_out, amount_in, 1_000);
    });
}

#[test]
fn single_asset_operations_are_equivalent_to_swaps() {
    // This is a sanity test that verifies that performing a single-asset join followed by a
    // single-asset exit is equivalent to a swap provided that no fees are taken. The claim made in
    // the Balancer whitepaper that this is true even if swap fees but no exit fees are taken, is
    // incorrect, except if the pool contains only two assets of equal weight.
    let amount_in = _1;
    let asset_in = ASSET_A;
    let asset_out = ASSET_B;
    let swap_fee = 0;

    let amount_out_single_asset_ops = ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0);
        create_initial_pool(swap_fee, true);
        assert_ok!(Currencies::deposit(asset_in, &ALICE, amount_in));
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            RuntimeOrigin::signed(ALICE),
            DEFAULT_POOL_ID,
            asset_in,
            amount_in,
            0,
        ));
        let pool_amount = Currencies::free_balance(Swaps::pool_shares_id(DEFAULT_POOL_ID), &ALICE);
        assert_ok!(Swaps::pool_exit_with_exact_pool_amount(
            RuntimeOrigin::signed(ALICE),
            DEFAULT_POOL_ID,
            asset_out,
            pool_amount,
            0,
        ));
        Currencies::free_balance(asset_out, &ALICE)
    });

    let amount_out_swap = ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(swap_fee, true);
        assert_ok!(Currencies::deposit(asset_in, &ALICE, amount_in));
        assert_ok!(Swaps::swap_exact_amount_in(
            RuntimeOrigin::signed(ALICE),
            DEFAULT_POOL_ID,
            asset_in,
            amount_in,
            asset_out,
            Some(0),
            None,
        ));
        Currencies::free_balance(asset_out, &ALICE)
    });

    let dust = 1_000;
    assert_approx!(amount_out_single_asset_ops, amount_out_swap, dust);
}

#[test]
fn pool_join_with_uneven_balances() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(1, true);
        let pool_account_id = Swaps::pool_account_id(&DEFAULT_POOL_ID);
        assert_ok!(Currencies::deposit(ASSET_A, &pool_account_id, _50));
        assert_ok!(Swaps::pool_join(
            RuntimeOrigin::signed(ALICE),
            DEFAULT_POOL_ID,
            _10,
            vec![_100; 4]
        ));
        assert_eq!(Currencies::free_balance(ASSET_A, &pool_account_id), _165);
        assert_eq!(Currencies::free_balance(ASSET_B, &pool_account_id), _110);
        assert_eq!(Currencies::free_balance(ASSET_C, &pool_account_id), _110);
        assert_eq!(Currencies::free_balance(ASSET_D, &pool_account_id), _110);
    });
}

#[test]
fn pool_exit_fails_if_balances_drop_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        // We drop the balances below `Swaps::min_balance(...)`, but liquidity remains above
        // `Swaps::min_balance(...)`.
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(1, true);
        let pool_account_id = Swaps::pool_account_id(&DEFAULT_POOL_ID);

        assert_ok!(Currencies::withdraw(
            ASSET_A,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_A)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_B,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_B)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_C,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_C)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_D,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_D)
        ));

        // We withdraw 99% of it, leaving 0.01 of each asset, which is below minimum balance.
        assert_noop!(
            Swaps::pool_exit(RuntimeOrigin::signed(BOB), DEFAULT_POOL_ID, _10, vec![0; 4]),
            Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn pool_exit_fails_if_liquidity_drops_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        // We drop the liquidity below `Swaps::min_balance(...)`, but balances remains above
        // `Swaps::min_balance(...)`.
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(1, true);
        let pool_account_id = Swaps::pool_account_id(&DEFAULT_POOL_ID);

        // There's 1000 left of each asset.
        assert_ok!(Currencies::deposit(ASSET_A, &pool_account_id, _900));
        assert_ok!(Currencies::deposit(ASSET_B, &pool_account_id, _900));
        assert_ok!(Currencies::deposit(ASSET_C, &pool_account_id, _900));
        assert_ok!(Currencies::deposit(ASSET_D, &pool_account_id, _900));

        // We withdraw too much liquidity but leave enough of each asset.
        assert_noop!(
            Swaps::pool_exit(
                RuntimeOrigin::signed(BOB),
                DEFAULT_POOL_ID,
                _100 - Swaps::min_balance(Swaps::pool_shares_id(DEFAULT_POOL_ID)) + 1,
                vec![0; 4]
            ),
            Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn swap_exact_amount_in_fails_if_balances_drop_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(1, true);
        let pool_account_id = Swaps::pool_account_id(&DEFAULT_POOL_ID);

        // There's only very little left of all assets!
        assert_ok!(Currencies::withdraw(
            ASSET_A,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_A)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_B,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_B)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_C,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_C)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_D,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_D)
        ));

        assert_noop!(
            Swaps::swap_exact_amount_in(
                RuntimeOrigin::signed(ALICE),
                DEFAULT_POOL_ID,
                ASSET_A,
                Swaps::min_balance(ASSET_A) / 10,
                ASSET_B,
                Some(0),
                None,
            ),
            Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn swap_exact_amount_out_fails_if_balances_drop_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(1, true);
        let pool_account_id = Swaps::pool_account_id(&DEFAULT_POOL_ID);

        // There's only very little left of all assets!
        assert_ok!(Currencies::withdraw(
            ASSET_A,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_A)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_B,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_B)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_C,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_C)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_D,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_D)
        ));

        assert_noop!(
            Swaps::swap_exact_amount_out(
                RuntimeOrigin::signed(ALICE),
                DEFAULT_POOL_ID,
                ASSET_A,
                Some(u128::MAX),
                ASSET_B,
                Swaps::min_balance(ASSET_B) / 10,
                None,
            ),
            Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn pool_exit_with_exact_pool_amount_fails_if_balances_drop_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(1, true);
        let pool_account_id = Swaps::pool_account_id(&DEFAULT_POOL_ID);

        // There's only very little left of all assets!
        assert_ok!(Currencies::withdraw(
            ASSET_A,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_A)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_B,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_B)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_C,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_C)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_D,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_D)
        ));

        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(
                RuntimeOrigin::signed(BOB),
                DEFAULT_POOL_ID,
                ASSET_A,
                _1,
                0
            ),
            Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn pool_exit_with_exact_pool_amount_fails_if_liquidity_drops_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(1, true);
        let pool_account_id = Swaps::pool_account_id(&DEFAULT_POOL_ID);

        assert_ok!(Currencies::deposit(ASSET_A, &pool_account_id, _10000));
        assert_ok!(Currencies::deposit(ASSET_B, &pool_account_id, _10000));
        assert_ok!(Currencies::deposit(ASSET_C, &pool_account_id, _10000));
        assert_ok!(Currencies::deposit(ASSET_D, &pool_account_id, _10000));

        // Reduce amount of liquidity so that doing the withdraw doesn't cause a `Min*Ratio` error!
        let pool_shares_id = Swaps::pool_shares_id(DEFAULT_POOL_ID);
        assert_eq!(Currencies::total_issuance(pool_shares_id), _100);
        Currencies::slash(pool_shares_id, &BOB, _100 - Swaps::min_balance(pool_shares_id));

        let ten_percent_of_pool = Swaps::min_balance(pool_shares_id) / 10;
        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(
                RuntimeOrigin::signed(BOB),
                DEFAULT_POOL_ID,
                ASSET_A,
                ten_percent_of_pool,
                0,
            ),
            Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn pool_exit_with_exact_asset_amount_fails_if_balances_drop_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(1, true);
        let pool_account_id = Swaps::pool_account_id(&DEFAULT_POOL_ID);

        // There's only very little left of all assets!
        assert_ok!(Currencies::withdraw(
            ASSET_A,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_A)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_B,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_B)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_C,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_C)
        ));
        assert_ok!(Currencies::withdraw(
            ASSET_D,
            &pool_account_id,
            _100 - Swaps::min_balance(ASSET_D)
        ));

        let ten_percent_of_balance = Swaps::min_balance(ASSET_A) / 10;
        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(
                RuntimeOrigin::signed(BOB),
                DEFAULT_POOL_ID,
                ASSET_A,
                ten_percent_of_balance,
                _100,
            ),
            Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn pool_exit_with_exact_asset_amount_fails_if_liquidity_drops_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(1, true);

        // Reduce amount of liquidity so that doing the withdraw doesn't cause a `Min*Ratio` error!
        let pool_shares_id = Swaps::pool_shares_id(DEFAULT_POOL_ID);
        assert_eq!(Currencies::total_issuance(pool_shares_id), _100);
        Currencies::slash(pool_shares_id, &BOB, _100 - Swaps::min_balance(pool_shares_id));

        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(
                RuntimeOrigin::signed(BOB),
                DEFAULT_POOL_ID,
                ASSET_A,
                _25,
                _100,
            ),
            Error::<Runtime>::PoolDrain,
        );
    });
}

fn alice_signed() -> RuntimeOrigin {
    RuntimeOrigin::signed(ALICE)
}

fn create_initial_pool(swap_fee: BalanceOf<Runtime>, deposit: bool) {
    if deposit {
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, _100));
        });
    }
    let pool_id = Swaps::next_pool_id();
    assert_ok!(Swaps::create_pool(
        BOB,
        ASSETS.to_vec(),
        swap_fee,
        DEFAULT_LIQUIDITY,
        vec![DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT, DEFAULT_WEIGHT],
    ));
    assert_ok!(Swaps::open_pool(pool_id));
}

fn create_initial_pool_with_funds_for_alice(swap_fee: BalanceOf<Runtime>, deposit: bool) {
    create_initial_pool(swap_fee, deposit);
    let _ = Currencies::deposit(ASSET_A, &ALICE, _25);
    let _ = Currencies::deposit(ASSET_B, &ALICE, _25);
    let _ = Currencies::deposit(ASSET_C, &ALICE, _25);
    let _ = Currencies::deposit(ASSET_D, &ALICE, _25);
}

fn assert_all_parameters(
    alice_assets: [u128; 4],
    alice_pool_assets: u128,
    pool_assets: [u128; 4],
    total_issuance: u128,
) {
    let pai = Swaps::pool_account_id(&DEFAULT_POOL_ID);
    let psi = Swaps::pool_shares_id(DEFAULT_POOL_ID);

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
