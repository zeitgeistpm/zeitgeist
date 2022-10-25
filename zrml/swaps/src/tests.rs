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

use crate::{
    events::{CommonPoolEventParams, PoolAssetEvent, PoolAssetsEvent, SwapEvent},
    mock::*,
    BalanceOf, Config, Event, MarketIdOf, PoolsCachedForArbitrage, SubsidyProviders,
};
use frame_support::{
    assert_err, assert_noop, assert_ok, assert_storage_noop, error::BadOrigin, traits::Hooks,
    weights::Weight,
};
use more_asserts::{assert_ge, assert_le};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_runtime::SaturatedConversion;
#[allow(unused_imports)]
use test_case::test_case;
use zeitgeist_primitives::{
    constants::BASE,
    traits::Swaps as _,
    types::{
        AccountIdTest, Asset, BlockNumber, Deadlines, Market, MarketCreation,
        MarketDisputeMechanism, MarketId, MarketPeriod, MarketStatus, MarketType, Moment,
        OutcomeReport, PoolId, PoolStatus, ScoringRule,
    },
};
use zrml_market_commons::MarketCommonsPalletApi;
use zrml_rikiddo::traits::RikiddoMVPallet;

pub const ASSET_A: Asset<MarketId> = Asset::CategoricalOutcome(0, 65);
pub const ASSET_B: Asset<MarketId> = Asset::CategoricalOutcome(0, 66);
pub const ASSET_C: Asset<MarketId> = Asset::CategoricalOutcome(0, 67);
pub const ASSET_D: Asset<MarketId> = Asset::CategoricalOutcome(0, 68);
pub const ASSET_E: Asset<MarketId> = Asset::CategoricalOutcome(0, 69);

pub const ASSETS: [Asset<MarketId>; 4] = [ASSET_A, ASSET_B, ASSET_C, ASSET_D];

pub const SENTINEL_AMOUNT: u128 = 123456789;

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
fn create_pool_fails_with_duplicate_assets(assets: Vec<Asset<MarketIdOf<Runtime>>>) {
    ExtBuilder::default().build().execute_with(|| {
        assets.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, _10000);
        });
        let asset_count = assets.len();
        assert_noop!(
            Swaps::create_pool(
                BOB,
                assets,
                ASSET_A,
                0,
                ScoringRule::CPMM,
                Some(0),
                Some(<Runtime as crate::Config>::MinLiquidity::get()),
                Some(vec![_2; asset_count]),
            ),
            crate::Error::<Runtime>::SomeIdenticalAssets
        );
    });
}

#[test]
fn destroy_pool_fails_if_pool_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, Some(0), true);
        assert_noop!(Swaps::destroy_pool(42), crate::Error::<Runtime>::PoolDoesNotExist);
    });
}

#[test]
fn destroy_pool_correctly_cleans_up_pool() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        let pool_id = 0;
        let alice_balance_before = [
            Currencies::free_balance(ASSET_A, &ALICE),
            Currencies::free_balance(ASSET_B, &ALICE),
            Currencies::free_balance(ASSET_C, &ALICE),
            Currencies::free_balance(ASSET_D, &ALICE),
        ];
        assert_ok!(Swaps::destroy_pool(pool_id));
        assert_err!(Swaps::pool(pool_id), crate::Error::<Runtime>::PoolDoesNotExist);
        // Ensure that funds _outside_ of the pool are not impacted!
        // TODO(#792): Remove pool shares.
        let total_pool_shares = Currencies::total_issuance(Swaps::pool_shares_id(0));
        assert_all_parameters(alice_balance_before, 0, [0, 0, 0, 0], total_pool_shares);
    });
}

#[test]
fn destroy_pool_emits_correct_event() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool(ScoringRule::CPMM, Some(0), true);
        let pool_id = 0;
        assert_ok!(Swaps::destroy_pool(pool_id));
        System::assert_last_event(Event::PoolDestroyed(pool_id).into());
    });
}

#[test]
fn allows_the_full_user_lifecycle() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);

        assert_ok!(Swaps::pool_join(alice_signed(), 0, _5, vec!(_25, _25, _25, _25),));

        let asset_a_bal = Currencies::free_balance(ASSET_A, &ALICE);
        let asset_b_bal = Currencies::free_balance(ASSET_B, &ALICE);

        // swap_exact_amount_in
        let spot_price = Swaps::get_spot_price(&0, &ASSET_A, &ASSET_B).unwrap();
        assert_eq!(spot_price, _1);

        let pool_account = Swaps::pool_account_id(&0);

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
            _2,
            Currencies::free_balance(ASSET_B, &pool_account),
            _2,
            _1,
            0,
        )
        .unwrap();

        assert_eq!(expected_in, 10_290_319_622);

        assert_ok!(Swaps::swap_exact_amount_out(
            alice_signed(),
            0,
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        assert_ok!(Swaps::mutate_pool(0, |pool| {
            pool.weights.as_mut().unwrap().remove(&ASSET_B);
            Ok(())
        }));

        assert_noop!(
            Swaps::swap_exact_amount_in(alice_signed(), 0, ASSET_A, 1, ASSET_B, Some(1), Some(1)),
            crate::Error::<Runtime>::AssetNotBound
        );
        assert_noop!(
            Swaps::swap_exact_amount_in(alice_signed(), 0, ASSET_B, 1, ASSET_A, Some(1), Some(1)),
            crate::Error::<Runtime>::AssetNotBound
        );

        assert_noop!(
            Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_A, Some(1), ASSET_B, 1, Some(1)),
            crate::Error::<Runtime>::AssetNotBound
        );
        assert_noop!(
            Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_B, Some(1), ASSET_A, 1, Some(1)),
            crate::Error::<Runtime>::AssetNotBound
        );

        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(alice_signed(), 0, ASSET_B, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
        );
        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(alice_signed(), 0, ASSET_B, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
        );

        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(alice_signed(), 0, ASSET_B, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
        );
        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(alice_signed(), 0, ASSET_B, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
        );
    });
}

#[test]
fn create_pool_generates_a_new_pool_with_correct_parameters_for_cpmm() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);

        let next_pool_before = Swaps::next_pool_id();
        assert_eq!(next_pool_before, 0);

        let amount = <Runtime as crate::Config>::MinLiquidity::get();
        let base_asset = ASSETS.last().unwrap();
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, amount));
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            *base_asset,
            0,
            ScoringRule::CPMM,
            Some(1),
            Some(amount),
            Some(vec!(_4, _3, _2, _1)),
        ));

        let next_pool_after = Swaps::next_pool_id();
        assert_eq!(next_pool_after, 1);

        let pool = Swaps::pools(0).unwrap();

        assert_eq!(pool.assets, ASSETS);
        assert_eq!(pool.base_asset, *base_asset);
        assert_eq!(pool.market_id, 0);
        assert_eq!(pool.pool_status, PoolStatus::Initialized);
        assert_eq!(pool.scoring_rule, ScoringRule::CPMM);
        assert_eq!(pool.swap_fee, Some(1));
        assert_eq!(pool.total_subsidy, None);
        assert_eq!(pool.total_weight.unwrap(), _10);

        assert_eq!(*pool.weights.as_ref().unwrap().get(&ASSET_A).unwrap(), _4);
        assert_eq!(*pool.weights.as_ref().unwrap().get(&ASSET_B).unwrap(), _3);
        assert_eq!(*pool.weights.as_ref().unwrap().get(&ASSET_C).unwrap(), _2);
        assert_eq!(*pool.weights.as_ref().unwrap().get(&ASSET_D).unwrap(), _1);

        let pool_account = Swaps::pool_account_id(&0);
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

#[test]
fn create_pool_generates_a_new_pool_with_correct_parameters_for_rikiddo() {
    ExtBuilder::default().build().execute_with(|| {
        let next_pool_before = Swaps::next_pool_id();
        assert_eq!(next_pool_before, 0);

        create_initial_pool(ScoringRule::RikiddoSigmoidFeeMarketEma, None, false);

        let next_pool_after = Swaps::next_pool_id();
        assert_eq!(next_pool_after, 1);
        let pool = Swaps::pools(0).unwrap();

        assert_eq!(pool.assets, ASSETS.to_vec());
        assert_eq!(pool.base_asset, ASSET_D);
        assert_eq!(pool.pool_status, PoolStatus::CollectingSubsidy);
        assert_eq!(pool.scoring_rule, ScoringRule::RikiddoSigmoidFeeMarketEma);
        assert_eq!(pool.swap_fee, None);
        assert_eq!(pool.total_subsidy, Some(0));
        assert_eq!(pool.total_weight, None);
        assert_eq!(pool.weights, None);
    });
}

#[test]
fn destroy_pool_in_subsidy_phase_returns_subsidy_and_closes_pool() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        // Errors trigger correctly.
        assert_noop!(
            Swaps::destroy_pool_in_subsidy_phase(0),
            crate::Error::<Runtime>::PoolDoesNotExist
        );
        create_initial_pool(ScoringRule::CPMM, Some(0), true);
        assert_noop!(
            Swaps::destroy_pool_in_subsidy_phase(0),
            crate::Error::<Runtime>::InvalidStateTransition
        );

        create_initial_pool_with_funds_for_alice(
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            None,
            false,
        );
        let pool_id = 1;
        // Reserve some funds for subsidy
        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, _25));
        assert_ok!(Currencies::deposit(ASSET_D, &BOB, _26));
        assert_ok!(Swaps::pool_join_subsidy(Origin::signed(BOB), pool_id, _26));
        assert_eq!(Currencies::reserved_balance(ASSET_D, &ALICE), _25);
        assert_eq!(Currencies::reserved_balance(ASSET_D, &BOB), _26);

        assert_ok!(Swaps::destroy_pool_in_subsidy_phase(pool_id));
        // Rserved balanced was returned and all storage cleared.
        assert_eq!(Currencies::reserved_balance(ASSET_D, &ALICE), 0);
        assert_eq!(Currencies::reserved_balance(ASSET_D, &BOB), 0);
        assert!(!crate::SubsidyProviders::<Runtime>::contains_key(pool_id, ALICE));
        assert!(!crate::Pools::<Runtime>::contains_key(pool_id));
        System::assert_last_event(
            Event::PoolDestroyedInSubsidyPhase(pool_id, vec![(BOB, _26), (ALICE, _25)]).into(),
        );
    });
}

#[test]
fn distribute_pool_share_rewards() {
    ExtBuilder::default().build().execute_with(|| {
        // Create Rikiddo pool
        create_initial_pool(ScoringRule::RikiddoSigmoidFeeMarketEma, None, false);
        let pool_id = 0;
        let subsidy_per_acc = <Runtime as crate::Config>::MinSubsidy::get();
        let asset_per_acc = subsidy_per_acc / 10;
        let base_asset = Swaps::pool_by_id(pool_id).unwrap().base_asset;
        let winning_asset = ASSET_A;

        // Join subsidy with some providers
        let subsidy_providers: Vec<AccountIdTest> = (1000..1010).collect();
        subsidy_providers.iter().for_each(|provider| {
            assert_ok!(Currencies::deposit(base_asset, provider, subsidy_per_acc));
            assert_ok!(Swaps::pool_join_subsidy(
                Origin::signed(*provider),
                pool_id,
                subsidy_per_acc
            ));
        });

        // End subsidy phase
        assert_ok!(Swaps::end_subsidy_phase(pool_id));

        // Buy some winning outcome assets with other accounts and remember how many
        let asset_holders: Vec<AccountIdTest> = (1010..1020).collect();
        asset_holders.iter().for_each(|asset_holder| {
            assert_ok!(Currencies::deposit(base_asset, asset_holder, asset_per_acc + 20));
            assert_ok!(Swaps::swap_exact_amount_out(
                Origin::signed(*asset_holder),
                pool_id,
                base_asset,
                Some(asset_per_acc + 20),
                winning_asset,
                asset_per_acc,
                Some(_5),
            ));
        });
        let total_winning_assets = asset_holders.len().saturated_into::<u128>() * asset_per_acc;

        // Distribute pool share rewards
        let pool = Swaps::pool(pool_id).unwrap();
        let winner_payout_account: AccountIdTest = 1337;
        Swaps::distribute_pool_share_rewards(
            &pool,
            pool_id,
            base_asset,
            winning_asset,
            &winner_payout_account,
        );

        // Check if every subsidy provider got their fair share (percentage)
        assert_ne!(Currencies::total_balance(base_asset, &subsidy_providers[0]), 0);

        for idx in 1..subsidy_providers.len() {
            assert_eq!(
                Currencies::total_balance(base_asset, &subsidy_providers[idx - 1]),
                Currencies::total_balance(base_asset, &subsidy_providers[idx])
            );
        }

        // Check if the winning asset holders can be paid out.
        let winner_payout_acc_balance =
            Currencies::total_balance(base_asset, &winner_payout_account);
        assert!(total_winning_assets <= winner_payout_acc_balance);
        // Ensure the remaining "dust" is tiny
        assert!(winner_payout_acc_balance - total_winning_assets < BASE / 1_000_000);
    });
}

#[test]
fn end_subsidy_phase_distributes_shares_and_outcome_assets() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool(ScoringRule::CPMM, Some(0), true);
        assert_noop!(Swaps::end_subsidy_phase(0), crate::Error::<Runtime>::InvalidStateTransition);
        assert_noop!(Swaps::end_subsidy_phase(1), crate::Error::<Runtime>::PoolDoesNotExist);
        create_initial_pool_with_funds_for_alice(
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            None,
            false,
        );
        let pool_id = 1;
        assert_storage_noop!(Swaps::end_subsidy_phase(pool_id).unwrap());

        // Reserve some funds for subsidy
        let min_subsidy = <Runtime as crate::Config>::MinSubsidy::get();
        let subsidy_alice = min_subsidy;
        let subsidy_bob = min_subsidy + _25;
        assert_ok!(Currencies::deposit(ASSET_D, &ALICE, subsidy_alice));
        assert_ok!(Currencies::deposit(ASSET_D, &BOB, subsidy_bob));
        assert_ok!(Swaps::pool_join_subsidy(Origin::signed(ALICE), pool_id, min_subsidy));
        assert_ok!(Swaps::pool_join_subsidy(Origin::signed(BOB), pool_id, subsidy_bob));
        assert!(Swaps::end_subsidy_phase(pool_id).unwrap().result);

        // Check that subsidy was deposited, shares were distributed in exchange, the initial
        // outstanding event outcome assets are assigned to the pool account and pool is active.
        assert_eq!(Currencies::reserved_balance(ASSET_D, &ALICE), 0);
        assert_eq!(Currencies::reserved_balance(ASSET_D, &BOB), 0);

        let pool_shares_id = Swaps::pool_shares_id(pool_id);
        assert_eq!(Currencies::total_balance(pool_shares_id, &ALICE), subsidy_alice);
        assert_eq!(Currencies::total_balance(pool_shares_id, &BOB), subsidy_bob);

        let pool_account_id = Swaps::pool_account_id(&pool_id);
        let total_subsidy = Currencies::total_balance(ASSET_D, &pool_account_id);
        let total_subsidy_expected = subsidy_alice + subsidy_bob;
        assert_eq!(total_subsidy, total_subsidy_expected);
        System::assert_last_event(
            Event::SubsidyCollected(
                pool_id,
                vec![(BOB, subsidy_bob), (ALICE, subsidy_alice)],
                total_subsidy_expected,
            )
            .into(),
        );
        let initial_outstanding_assets = RikiddoSigmoidFeeMarketEma::initial_outstanding_assets(
            pool_id,
            (ASSETS.len() - 1).saturated_into::<u32>(),
            total_subsidy,
        )
        .unwrap();
        let balance_asset_a = Currencies::total_balance(ASSET_A, &pool_account_id);
        let balance_asset_b = Currencies::total_balance(ASSET_B, &pool_account_id);
        let balance_asset_c = Currencies::total_balance(ASSET_C, &pool_account_id);
        assert!(balance_asset_a == initial_outstanding_assets);
        assert!(balance_asset_a == balance_asset_b && balance_asset_b == balance_asset_c);
        assert_eq!(Swaps::pool_by_id(pool_id).unwrap().pool_status, PoolStatus::Active);
    });
}

#[test_case(PoolStatus::Initialized; "Initialized")]
#[test_case(PoolStatus::Closed; "Closed")]
#[test_case(PoolStatus::Clean; "Clean")]
fn single_asset_operations_and_swaps_fail_on_invalid_status_before_clean(status: PoolStatus) {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        let pool_id = 0;
        // For this test, we need to give Alice some pool shares, as well. We don't do this in
        // `create_initial_pool_...` so that there are exacly 100 pool shares, making computations
        // in other tests easier.
        assert_ok!(Currencies::deposit(Swaps::pool_shares_id(0), &ALICE, _25));
        assert_ok!(Swaps::mutate_pool(pool_id, |pool| {
            pool.pool_status = status;
            Ok(())
        }));

        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(alice_signed(), pool_id, ASSET_A, _1, _2),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(alice_signed(), pool_id, ASSET_A, _1, _1_2),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(alice_signed(), pool_id, ASSET_E, 1, 1),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(alice_signed(), pool_id, ASSET_E, 1, 1),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_ok!(Currencies::deposit(ASSET_A, &ALICE, u64::MAX.into()));
        assert_noop!(
            Swaps::swap_exact_amount_in(
                alice_signed(),
                pool_id,
                ASSET_A,
                u64::MAX.into(),
                ASSET_B,
                Some(_1),
                Some(_1),
            ),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::swap_exact_amount_out(
                alice_signed(),
                pool_id,
                ASSET_A,
                Some(u64::MAX.into()),
                ASSET_B,
                _1,
                Some(_1),
            ),
            crate::Error::<Runtime>::PoolIsNotActive
        );
    });
}

#[test]
fn pool_join_fails_if_pool_is_closed() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        let pool_id = 0;
        assert_ok!(Swaps::close_pool(pool_id));
        assert_noop!(
            Swaps::pool_join(Origin::signed(ALICE), pool_id, _1, vec![_1, _1, _1, _1]),
            crate::Error::<Runtime>::InvalidPoolStatus,
        );
    });
}

#[test]
fn most_operations_fail_if_pool_is_clean() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        let pool_id = 0;
        assert_ok!(Swaps::close_pool(pool_id));
        assert_ok!(Swaps::clean_up_pool(
            &MarketType::Categorical(0),
            pool_id,
            &OutcomeReport::Categorical(if let Asset::CategoricalOutcome(_, idx) = ASSET_A {
                idx
            } else {
                0
            }),
            &Default::default()
        ));

        assert_noop!(
            Swaps::pool_join(Origin::signed(ALICE), pool_id, _1, vec![_10]),
            crate::Error::<Runtime>::InvalidPoolStatus,
        );
        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(alice_signed(), pool_id, ASSET_A, _1, _2),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(alice_signed(), pool_id, ASSET_A, _1, _1_2),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(alice_signed(), pool_id, ASSET_E, 1, 1),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(alice_signed(), pool_id, ASSET_E, 1, 1),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_ok!(Currencies::deposit(ASSET_A, &ALICE, u64::MAX.into()));
        assert_noop!(
            Swaps::swap_exact_amount_in(
                alice_signed(),
                pool_id,
                ASSET_A,
                u64::MAX.into(),
                ASSET_B,
                Some(_1),
                Some(_1),
            ),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::swap_exact_amount_out(
                alice_signed(),
                pool_id,
                ASSET_A,
                Some(u64::MAX.into()),
                ASSET_B,
                _1,
                Some(_1),
            ),
            crate::Error::<Runtime>::PoolIsNotActive
        );
    });
}

#[test_case(_3, _3, _100, _100, 0, 10_000_000_000)]
#[test_case(_3, _3, _100, _150, 0, 6_666_666_667)]
#[test_case(_3, _4, _100, _100, 0, 13_333_333_333)]
#[test_case(_3, _4, _100, _150, 0, 8_888_888_889)]
#[test_case(_3, _6, _125, _150, 0, 16_666_666_667)]
#[test_case(_3, _6, _125, _100, 0, 25_000_000_000)]
#[test_case(_3, _3, _100, _100, _1_10, 11_111_111_111)]
#[test_case(_3, _3, _100, _150, _1_10, 7_407_407_408)]
#[test_case(_3, _4, _100, _100, _1_10, 14_814_814_814)]
#[test_case(_3, _4, _100, _150, _1_10, 9_876_543_210)]
#[test_case(_3, _6, _125, _150, _1_10, 18_518_518_519)]
#[test_case(_3, _6, _125, _100, _1_10, 27_777_777_778)]
fn get_spot_price_returns_correct_results_cpmm(
    weight_in: u128,
    weight_out: u128,
    balance_in: BalanceOf<Runtime>,
    balance_out: BalanceOf<Runtime>,
    swap_fee: BalanceOf<Runtime>,
    expected_spot_price: BalanceOf<Runtime>,
) {
    ExtBuilder::default().build().execute_with(|| {
        // We always swap ASSET_A for ASSET_B, but we vary the weights, balances and swap fees.
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, _100));
        });
        let amount_in_pool = <Runtime as crate::Config>::MinLiquidity::get();
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            *ASSETS.last().unwrap(),
            0,
            ScoringRule::CPMM,
            Some(swap_fee),
            Some(amount_in_pool),
            Some(vec!(weight_in, weight_out, _2, _3))
        ));
        let pool_id = 0;
        let pool_account = Swaps::pool_account_id(&pool_id);

        // Modify pool balances according to test data.
        assert_ok!(Currencies::deposit(ASSET_A, &pool_account, balance_in - amount_in_pool));
        assert_ok!(Currencies::deposit(ASSET_B, &pool_account, balance_out - amount_in_pool));

        let abs_tol = 100;
        assert_approx!(
            Swaps::get_spot_price(&pool_id, &ASSET_A, &ASSET_B).unwrap(),
            expected_spot_price,
            abs_tol,
        );
    });
}

#[test]
fn get_spot_price_returns_correct_results_rikiddo() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::RikiddoSigmoidFeeMarketEma, None, false);
        let pool_id = 0;
        assert_noop!(
            Swaps::get_spot_price(&pool_id, &ASSETS[0], &ASSETS[0]),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        subsidize_and_start_rikiddo_pool(pool_id, &ALICE, 0);

        // Asset out, base currency in. Should receive about 1/3 -> price about 3
        let price_base_in =
            Swaps::get_spot_price(&pool_id, &ASSETS[0], ASSETS.last().unwrap()).unwrap();
        // Between 0.3 and 0.4
        assert!(price_base_in > 28 * BASE / 10 && price_base_in < 31 * BASE / 10);
        // Base currency in, asset out. Price about 3.
        let price_base_out =
            Swaps::get_spot_price(&pool_id, ASSETS.last().unwrap(), &ASSETS[0]).unwrap();
        // Between 2.9 and 3.1
        assert!(price_base_out > 3 * BASE / 10 && price_base_out < 4 * BASE / 10);
        // Asset in, asset out. Price about 1.
        let price_asset_in_out = Swaps::get_spot_price(&pool_id, &ASSETS[0], &ASSETS[1]).unwrap();
        // Between 0.9 and 1.1
        assert!(price_asset_in_out > 9 * BASE / 10 && price_asset_in_out < 11 * BASE / 10);
    });
}

#[test]
fn in_amount_must_be_equal_or_less_than_max_in_ratio() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, Some(0), true);

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
            crate::Error::<Runtime>::MaxInRatio
        );
    });
}

#[test]
fn pool_exit_with_exact_asset_amount_satisfies_max_out_ratio_constraints() {
    ExtBuilder::default().build().execute_with(|| {
        // We make sure that the individual asset weights don't divide total weight so we trigger
        // the calculation of exp using the binomial series.
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, _100));
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            *ASSETS.last().unwrap(),
            0,
            ScoringRule::CPMM,
            Some(0),
            Some(<Runtime as crate::Config>::MinLiquidity::get()),
            Some(vec!(_2, _2, _2, _5)),
        ));
        let pool_id = 0;
        assert_ok!(Swaps::open_pool(pool_id));

        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(
                Origin::signed(BOB),
                pool_id,
                ASSET_A,
                _50,
                _10000,
            ),
            crate::Error::<Runtime>::MaxOutRatio,
        );
    });
}

#[test]
fn pool_exit_with_exact_pool_amount_satisfies_max_in_ratio_constraints() {
    ExtBuilder::default().build().execute_with(|| {
        // We make sure that the individual asset weights don't divide total weight so we trigger
        // the calculation of exp using the binomial series.
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, _100));
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            *ASSETS.last().unwrap(),
            0,
            ScoringRule::CPMM,
            Some(0),
            Some(<Runtime as crate::Config>::MinLiquidity::get()),
            Some(vec!(_2, _2, _2, _5)),
        ));
        let pool_id = 0;
        assert_ok!(Swaps::open_pool(pool_id));

        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(
                Origin::signed(BOB),
                pool_id,
                ASSET_A,
                _50,
                _10000,
            ),
            crate::Error::<Runtime>::MaxInRatio,
        );
    });
}

#[test]
fn pool_join_with_exact_asset_amount_satisfies_max_in_ratio_constraints() {
    ExtBuilder::default().build().execute_with(|| {
        // We make sure that the individual asset weights don't divide total weight so we trigger
        // the calculation of exp using the binomial series.
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, _100));
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            *ASSETS.last().unwrap(),
            0,
            ScoringRule::CPMM,
            Some(0),
            Some(<Runtime as crate::Config>::MinLiquidity::get()),
            Some(vec!(_2, _2, _2, _5)),
        ));
        let pool_id = 0;
        assert_ok!(Swaps::open_pool(pool_id));
        let asset_amount = _100;
        assert_ok!(Currencies::deposit(ASSET_A, &ALICE, asset_amount));

        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(
                alice_signed(),
                pool_id,
                ASSET_A,
                asset_amount,
                0,
            ),
            crate::Error::<Runtime>::MaxInRatio,
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
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, _100));
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.to_vec(),
            *ASSETS.last().unwrap(),
            0,
            ScoringRule::CPMM,
            Some(0),
            Some(<Runtime as crate::Config>::MinLiquidity::get()),
            Some(vec!(_2, _2, _2, _5)),
        ));
        let pool_id = 0;
        assert_ok!(Swaps::open_pool(pool_id));
        let max_asset_amount = _10000;
        assert_ok!(Currencies::deposit(ASSET_A, &ALICE, max_asset_amount));

        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(
                alice_signed(),
                pool_id,
                ASSET_A,
                _100,
                max_asset_amount,
            ),
            crate::Error::<Runtime>::MaxOutRatio,
        );
    });
}

#[test]
fn admin_clean_up_pool_fails_if_origin_is_not_root() {
    ExtBuilder::default().build().execute_with(|| {
        let idx = if let Asset::CategoricalOutcome(_, idx) = ASSET_A { idx } else { 0 };
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        assert_ok!(MarketCommons::push_market(mock_market(69)));
        assert_ok!(MarketCommons::insert_market_pool(0, 0));
        assert_noop!(
            Swaps::admin_clean_up_pool(alice_signed(), 0, OutcomeReport::Categorical(idx)),
            BadOrigin
        );
    });
}

#[test]
fn out_amount_must_be_equal_or_less_than_max_out_ratio() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, Some(0), true);

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
            crate::Error::<Runtime>::MaxOutRatio
        );
    });
}

#[test]
fn pool_join_or_exit_raises_on_zero_value() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);

        assert_noop!(
            Swaps::pool_join(alice_signed(), 0, 0, vec!(_1, _1, _1, _1)),
            crate::Error::<Runtime>::ZeroAmount
        );

        assert_noop!(
            Swaps::pool_exit(alice_signed(), 0, 0, vec!(_1, _1, _1, _1)),
            crate::Error::<Runtime>::ZeroAmount
        );

        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(alice_signed(), 0, ASSET_A, 0, 0),
            crate::Error::<Runtime>::ZeroAmount
        );

        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(alice_signed(), 0, ASSET_A, 0, 0),
            crate::Error::<Runtime>::ZeroAmount
        );

        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(alice_signed(), 0, ASSET_A, 0, 0),
            crate::Error::<Runtime>::ZeroAmount
        );

        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(alice_signed(), 0, ASSET_A, 0, 0),
            crate::Error::<Runtime>::ZeroAmount
        );
    });
}

#[test]
fn pool_exit_decreases_correct_pool_parameters() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);

        assert_ok!(Swaps::pool_join(alice_signed(), 0, _1, vec!(_1, _1, _1, _1),));

        assert_ok!(Swaps::pool_exit(alice_signed(), 0, _1, vec!(_1, _1, _1, _1),));

        System::assert_last_event(
            Event::PoolExit(PoolAssetsEvent {
                assets: vec![ASSET_A, ASSET_B, ASSET_C, ASSET_D],
                bounds: vec![_1, _1, _1, _1],
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                transferred: vec![_1 + 1, _1 + 1, _1 + 1, _1 + 1],
                pool_amount: _1,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 + 1, _25 + 1, _25 + 1, _25 + 1],
            0,
            [_100 - 1, _100 - 1, _100 - 1, _100 - 1],
            _100,
        );
    })
}

#[test]
fn pool_exit_emits_correct_events() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        assert_ok!(Swaps::pool_exit(Origin::signed(BOB), 0, _1, vec!(1, 2, 3, 4),));
        let amount = _1 - BASE / 10; // Subtract 10% fees!
        System::assert_last_event(
            Event::PoolExit(PoolAssetsEvent {
                assets: vec![ASSET_A, ASSET_B, ASSET_C, ASSET_D],
                bounds: vec![1, 2, 3, 4],
                cpep: CommonPoolEventParams { pool_id: 0, who: BOB },
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);

        assert_ok!(Swaps::pool_exit(Origin::signed(BOB), 0, _10, vec!(_1, _1, _1, _1),));

        let pool_account = Swaps::pool_account_id(&0);
        let pool_shares_id = Swaps::pool_shares_id(0);
        assert_eq!(Currencies::free_balance(ASSET_A, &BOB), _9);
        assert_eq!(Currencies::free_balance(ASSET_B, &BOB), _9);
        assert_eq!(Currencies::free_balance(ASSET_C, &BOB), _9);
        assert_eq!(Currencies::free_balance(ASSET_D, &BOB), _9);
        assert_eq!(Currencies::free_balance(pool_shares_id, &BOB), _100 - _10);
        assert_eq!(Currencies::free_balance(ASSET_A, &pool_account), _100 - _9);
        assert_eq!(Currencies::free_balance(ASSET_B, &pool_account), _100 - _9);
        assert_eq!(Currencies::free_balance(ASSET_C, &pool_account), _100 - _9);
        assert_eq!(Currencies::free_balance(ASSET_D, &pool_account), _100 - _9);
        assert_eq!(Currencies::total_issuance(pool_shares_id), _100 - _10);

        System::assert_last_event(
            Event::PoolExit(PoolAssetsEvent {
                assets: vec![ASSET_A, ASSET_B, ASSET_C, ASSET_D],
                bounds: vec![_1, _1, _1, _1],
                cpep: CommonPoolEventParams { pool_id: 0, who: BOB },
                transferred: vec![_9, _9, _9, _9],
                pool_amount: _10,
            })
            .into(),
        );
    })
}

#[test]
fn pool_exit_decreases_correct_pool_parameters_on_cleaned_up_pool() {
    // Test is the same as
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        assert_ok!(MarketCommons::push_market(mock_market(69)));
        assert_ok!(MarketCommons::insert_market_pool(0, 0));

        assert_ok!(Swaps::pool_join(alice_signed(), 0, _1, vec!(_1, _1, _1, _1),));
        assert_ok!(Swaps::close_pool(0));
        assert_ok!(Swaps::admin_clean_up_pool(Origin::root(), 0, OutcomeReport::Categorical(65),));
        assert_ok!(Swaps::pool_exit(alice_signed(), 0, _1, vec!(_1, _1),));

        System::assert_last_event(
            Event::PoolExit(PoolAssetsEvent {
                assets: vec![ASSET_A, ASSET_D],
                bounds: vec![_1, _1],
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                transferred: vec![_1 + 1, _1 + 1],
                pool_amount: _1,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 + 1, _24, _24, _25 + 1],
            0,
            // Note: Although the asset is deleted from the pool, the assets B/C still remain on the
            // pool account.
            [_100 - 1, _101, _101, _100 - 1],
            _100,
        );
    })
}

#[test]
fn pool_exit_subsidy_unreserves_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        // Events cannot be emitted on block zero...
        frame_system::Pallet::<Runtime>::set_block_number(1);

        create_initial_pool_with_funds_for_alice(
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            None,
            false,
        );
        let pool_id = 0;

        // Add some subsidy
        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, _25));
        let mut reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        let mut noted = <SubsidyProviders<Runtime>>::get(pool_id, ALICE).unwrap();
        let mut total_subsidy = Swaps::pool_by_id(pool_id).unwrap().total_subsidy.unwrap();
        assert_eq!(reserved, _25);
        assert_eq!(reserved, noted);
        assert_eq!(reserved, total_subsidy);

        // Exit 5 subsidy and see if the storage is consistent
        assert_ok!(Swaps::pool_exit_subsidy(alice_signed(), pool_id, _5));
        reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        noted = <SubsidyProviders<Runtime>>::get(pool_id, ALICE).unwrap();
        total_subsidy = Swaps::pool_by_id(pool_id).unwrap().total_subsidy.unwrap();
        assert_eq!(reserved, noted);
        assert_eq!(reserved, total_subsidy);
        System::assert_last_event(
            Event::PoolExitSubsidy(ASSET_D, _5, CommonPoolEventParams { pool_id, who: ALICE }, _5)
                .into(),
        );

        // Exit the remaining subsidy (in fact, we attempt to exit with more than remaining!) and
        // see if the storage is consistent
        assert_ok!(Swaps::pool_exit_subsidy(alice_signed(), pool_id, _25));
        reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        assert!(<SubsidyProviders<Runtime>>::get(pool_id, ALICE).is_none());
        total_subsidy = Swaps::pool_by_id(pool_id).unwrap().total_subsidy.unwrap();
        assert_eq!(reserved, 0);
        assert_eq!(reserved, total_subsidy);
        System::assert_last_event(
            Event::PoolExitSubsidy(
                ASSET_D,
                _25,
                CommonPoolEventParams { pool_id, who: ALICE },
                _20,
            )
            .into(),
        );

        // Add some subsidy, manually remove some reserved balance (create inconsistency)
        // and check if the internal values are adjusted to the inconsistency.
        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, _25));
        assert_eq!(Currencies::unreserve(ASSET_D, &ALICE, _20), 0);
        assert_ok!(Swaps::pool_exit_subsidy(alice_signed(), pool_id, _20));
        reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        assert!(<SubsidyProviders<Runtime>>::get(pool_id, ALICE).is_none());
        total_subsidy = Swaps::pool_by_id(pool_id).unwrap().total_subsidy.unwrap();
        assert_eq!(reserved, 0);
        assert_eq!(reserved, total_subsidy);
    });
}

#[test]
fn pool_exit_subsidy_fails_if_no_subsidy_is_provided() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            None,
            false,
        );
        assert_noop!(
            Swaps::pool_exit_subsidy(alice_signed(), 0, _1),
            crate::Error::<Runtime>::NoSubsidyProvided
        );
    });
}

#[test]
fn pool_exit_subsidy_fails_if_amount_is_zero() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            None,
            false,
        );
        assert_noop!(
            Swaps::pool_exit_subsidy(alice_signed(), 0, 0),
            crate::Error::<Runtime>::ZeroAmount
        );
    });
}

#[test]
fn pool_exit_subsidy_fails_if_pool_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Swaps::pool_exit_subsidy(alice_signed(), 0, _1),
            crate::Error::<Runtime>::PoolDoesNotExist
        );
    });
}

#[test]
fn pool_exit_subsidy_fails_if_scoring_rule_is_not_rikiddo() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        assert_noop!(
            Swaps::pool_exit_subsidy(alice_signed(), 0, _1),
            crate::Error::<Runtime>::InvalidScoringRule
        );
    });
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(swap_fee), true);
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            alice_signed(),
            0,
            ASSET_A,
            asset_amount_joined,
            0
        ));
        let pool_amount = Currencies::free_balance(Swaps::pool_shares_id(0), &ALICE);
        assert_eq!(pool_amount, pool_amount_expected); // (This is just a sanity check)

        assert_ok!(Swaps::pool_exit_with_exact_pool_amount(
            alice_signed(),
            0,
            ASSET_A,
            pool_amount,
            bound,
        ));
        System::assert_last_event(
            Event::PoolExitWithExactPoolAmount(PoolAssetEvent {
                asset: ASSET_A,
                bound,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                transferred: asset_amount_expected,
                pool_amount,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 - asset_amount_joined + asset_amount_expected, _25, _25, _25],
            0,
            [_100 + asset_amount_joined - asset_amount_expected, _100, _100, _100],
            _100,
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(swap_fee), true);
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            alice_signed(),
            0,
            ASSET_A,
            asset_amount_joined,
            0
        ));

        // (Sanity check for dust size)
        let pool_amount = Currencies::free_balance(Swaps::pool_shares_id(0), &ALICE);
        let abs_diff = |x, y| {
            if x < y { y - x } else { x - y }
        };
        let dust = abs_diff(pool_amount, pool_amount_expected);
        assert_le!(dust, 100);

        assert_ok!(Swaps::pool_exit_with_exact_asset_amount(
            alice_signed(),
            0,
            ASSET_A,
            asset_amount,
            bound,
        ));
        System::assert_last_event(
            Event::PoolExitWithExactAssetAmount(PoolAssetEvent {
                asset: ASSET_A,
                bound,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                transferred: asset_amount,
                pool_amount: pool_amount_expected,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 - asset_amount_joined + asset_amount, _25, _25, _25],
            dust,
            [_100 + asset_amount_joined - asset_amount, _100, _100, _100],
            _100 + dust,
        )
    });
}

#[test]
fn pool_exit_is_not_allowed_with_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);

        // Alice has no pool shares!
        assert_noop!(
            Swaps::pool_exit(alice_signed(), 0, _1, vec!(0, 0, 0, 0)),
            crate::Error::<Runtime>::InsufficientBalance,
        );

        // Now Alice has 25 pool shares!
        let _ = Currencies::deposit(Swaps::pool_shares_id(0), &ALICE, _25);
        assert_noop!(
            Swaps::pool_exit(alice_signed(), 0, _26, vec!(0, 0, 0, 0)),
            crate::Error::<Runtime>::InsufficientBalance,
        );
    })
}

#[test]
fn pool_join_increases_correct_pool_parameters() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);

        assert_ok!(Swaps::pool_join(alice_signed(), 0, _5, vec!(_25, _25, _25, _25),));
        System::assert_last_event(
            Event::PoolJoin(PoolAssetsEvent {
                assets: vec![ASSET_A, ASSET_B, ASSET_C, ASSET_D],
                bounds: vec![_25, _25, _25, _25],
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        assert_ok!(Swaps::pool_join(alice_signed(), 0, _1, vec!(_1, _1, _1, _1),));
        System::assert_last_event(
            Event::PoolJoin(PoolAssetsEvent {
                assets: vec![ASSET_A, ASSET_B, ASSET_C, ASSET_D],
                bounds: vec![_1, _1, _1, _1],
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                transferred: vec![_1, _1, _1, _1],
                pool_amount: _1,
            })
            .into(),
        );
    });
}

#[test]
fn pool_join_subsidy_reserves_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        // Events cannot be emitted on block zero...
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            None,
            false,
        );
        let pool_id = 0;
        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, _20));
        let mut reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        let mut noted = <SubsidyProviders<Runtime>>::get(pool_id, ALICE).unwrap();
        assert_eq!(reserved, _20);
        assert_eq!(reserved, noted);
        assert_eq!(reserved, Swaps::pool_by_id(pool_id).unwrap().total_subsidy.unwrap());
        System::assert_last_event(
            Event::PoolJoinSubsidy(ASSET_D, _20, CommonPoolEventParams { pool_id, who: ALICE })
                .into(),
        );

        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, _5));
        reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        noted = <SubsidyProviders<Runtime>>::get(pool_id, ALICE).unwrap();
        assert_eq!(reserved, _25);
        assert_eq!(reserved, noted);
        assert_eq!(reserved, Swaps::pool_by_id(pool_id).unwrap().total_subsidy.unwrap());
        assert_storage_noop!(Swaps::pool_join_subsidy(alice_signed(), pool_id, _5).unwrap_or(()));
        System::assert_last_event(
            Event::PoolJoinSubsidy(ASSET_D, _5, CommonPoolEventParams { pool_id, who: ALICE })
                .into(),
        );
    });
}

#[test]
fn pool_join_subsidy_fails_if_amount_is_zero() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            None,
            false,
        );
        assert_noop!(
            Swaps::pool_join_subsidy(alice_signed(), 0, 0),
            crate::Error::<Runtime>::ZeroAmount
        );
    });
}

#[test]
fn pool_join_subsidy_fails_if_pool_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Swaps::pool_join_subsidy(alice_signed(), 0, _1),
            crate::Error::<Runtime>::PoolDoesNotExist
        );
    });
}

#[test]
fn pool_join_subsidy_fails_if_scoring_rule_is_not_rikiddo() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        assert_noop!(
            Swaps::pool_join_subsidy(alice_signed(), 0, _1),
            crate::Error::<Runtime>::InvalidScoringRule
        );
    });
}

#[test]
fn pool_join_subsidy_fails_if_subsidy_is_below_min_per_account() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            None,
            false,
        );
        assert_noop!(
            Swaps::pool_join_subsidy(
                alice_signed(),
                0,
                <Runtime as Config>::MinSubsidyPerAccount::get() - 1
            ),
            crate::Error::<Runtime>::InvalidSubsidyAmount,
        );
    });
}

#[test]
fn pool_join_subsidy_with_small_amount_is_ok_if_account_is_already_a_provider() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            None,
            false,
        );
        let pool_id = 0;
        let large_amount = <Runtime as Config>::MinSubsidyPerAccount::get();
        let small_amount = 1;
        let total_amount = large_amount + small_amount;
        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, large_amount));
        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, small_amount));
        let reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        let noted = <SubsidyProviders<Runtime>>::get(pool_id, ALICE).unwrap();
        let total_subsidy = Swaps::pool_by_id(pool_id).unwrap().total_subsidy.unwrap();
        assert_eq!(reserved, total_amount);
        assert_eq!(noted, total_amount);
        assert_eq!(total_subsidy, total_amount);
    });
}

#[test]
fn pool_exit_subsidy_unreserves_remaining_subsidy_if_below_min_per_account() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            None,
            false,
        );
        let pool_id = 0;
        let large_amount = <Runtime as Config>::MinSubsidyPerAccount::get();
        let small_amount = 1;
        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, large_amount));
        assert_ok!(Swaps::pool_exit_subsidy(alice_signed(), pool_id, small_amount));
        let reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        let noted = <SubsidyProviders<Runtime>>::get(pool_id, ALICE);
        let total_subsidy = Swaps::pool_by_id(pool_id).unwrap().total_subsidy.unwrap();
        assert_eq!(reserved, 0);
        assert!(noted.is_none());
        assert_eq!(total_subsidy, 0);
        System::assert_last_event(
            Event::PoolExitSubsidy(
                ASSET_D,
                small_amount,
                CommonPoolEventParams { pool_id, who: ALICE },
                large_amount,
            )
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(swap_fee), true);
        let bound = 0;
        let alice_sent = _1;
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            alice_signed(),
            0,
            ASSET_A,
            asset_amount,
            bound,
        ));
        System::assert_last_event(
            Event::PoolJoinWithExactAssetAmount(PoolAssetEvent {
                asset: ASSET_A,
                bound,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                transferred: asset_amount,
                pool_amount: pool_amount_expected,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 - asset_amount, _25, _25, _25],
            pool_amount_expected,
            [_100 + alice_sent, _100, _100, _100],
            _100 + pool_amount_expected,
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(swap_fee), true);
        let bound = _5;
        assert_ok!(Swaps::pool_join_with_exact_pool_amount(
            alice_signed(),
            0,
            ASSET_A,
            pool_amount,
            bound,
        ));
        System::assert_last_event(
            Event::PoolJoinWithExactPoolAmount(PoolAssetEvent {
                asset: ASSET_A,
                bound,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                transferred: asset_amount_expected,
                pool_amount,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 - asset_amount_expected, _25, _25, _25],
            pool_amount,
            [_100 + asset_amount_expected, _100, _100, _100],
            _100 + pool_amount,
        );
    });
}

#[test]
fn provided_values_len_must_equal_assets_len() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, Some(0), true);
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
fn clean_up_pool_leaves_only_correct_assets() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool(ScoringRule::CPMM, Some(0), true);
        let pool_id = 0;
        assert_ok!(Swaps::close_pool(pool_id));
        let cat_idx = if let Asset::CategoricalOutcome(_, cidx) = ASSET_A { cidx } else { 0 };
        assert_ok!(Swaps::clean_up_pool(
            &MarketType::Categorical(4),
            pool_id,
            &OutcomeReport::Categorical(cat_idx),
            &Default::default()
        ));
        let pool = Swaps::pool(pool_id).unwrap();
        assert_eq!(pool.pool_status, PoolStatus::Clean);
        assert_eq!(Swaps::pool_by_id(pool_id).unwrap().assets, vec![ASSET_A, ASSET_D]);
        System::assert_last_event(Event::PoolCleanedUp(pool_id).into());
    });
}

#[test]
fn clean_up_pool_handles_rikiddo_pools_properly() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::RikiddoSigmoidFeeMarketEma, None, false);
        let pool_id = 0;
        let cat_idx = if let Asset::CategoricalOutcome(_, cidx) = ASSET_A { cidx } else { 0 };

        // We need to forcefully close the pool (Rikiddo pools are not allowed to be cleaned
        // up when CollectingSubsidy).
        assert_ok!(Swaps::mutate_pool(pool_id, |pool| {
            pool.pool_status = PoolStatus::Closed;
            Ok(())
        }));

        assert_ok!(Swaps::clean_up_pool(
            &MarketType::Categorical(4),
            pool_id,
            &OutcomeReport::Categorical(cat_idx),
            &Default::default()
        ));

        // Rikiddo instance does not exist anymore.
        assert_storage_noop!(RikiddoSigmoidFeeMarketEma::clear(pool_id).unwrap_or(()));
    });
}

#[test_case(PoolStatus::Active; "active")]
#[test_case(PoolStatus::Clean; "clean")]
#[test_case(PoolStatus::CollectingSubsidy; "collecting_subsidy")]
#[test_case(PoolStatus::Initialized; "initialized")]
fn clean_up_pool_fails_if_pool_is_not_closed(pool_status: PoolStatus) {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::RikiddoSigmoidFeeMarketEma, None, false);
        let pool_id = 0;
        assert_ok!(Swaps::mutate_pool(pool_id, |pool| {
            pool.pool_status = pool_status;
            Ok(())
        }));
        let pool_id = 0;
        let cat_idx = if let Asset::CategoricalOutcome(_, cidx) = ASSET_A { cidx } else { 0 };
        assert_noop!(
            Swaps::clean_up_pool(
                &MarketType::Categorical(4),
                pool_id,
                &OutcomeReport::Categorical(cat_idx),
                &Default::default()
            ),
            crate::Error::<Runtime>::InvalidStateTransition
        );
    });
}

#[test]
fn clean_up_pool_fails_if_winning_asset_is_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, Some(0), true);
        let pool_id = 0;
        assert_ok!(Swaps::close_pool(pool_id));
        assert_noop!(
            Swaps::clean_up_pool(
                &MarketType::Categorical(1337),
                pool_id,
                &OutcomeReport::Categorical(1337),
                &Default::default()
            ),
            crate::Error::<Runtime>::WinningAssetNotFound
        );
    });
}

#[test]
fn swap_exact_amount_in_exchanges_correct_values_with_cpmm() {
    ExtBuilder::default().build().execute_with(|| {
        let asset_bound = Some(_1 / 2);
        let max_price = Some(_2);
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        assert_ok!(Swaps::swap_exact_amount_in(
            alice_signed(),
            0,
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
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                max_price,
            })
            .into(),
        );
        assert_all_parameters(
            [_24, _25 + 9900990100, _25, _25],
            0,
            [_101, _99 + 99009900, _100, _100],
            _100,
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
            *ASSETS.last().unwrap(),
            0,
            ScoringRule::CPMM,
            Some(BASE / 10),
            Some(<Runtime as crate::Config>::MinLiquidity::get()),
            Some(vec!(_2, _2, _2, _2)),
        ));
        let pool_id = 0;
        assert_ok!(Swaps::open_pool(pool_id));

        let asset_bound = Some(_1 / 2);
        let max_price = Some(_2);
        // ALICE swaps in BASE / 0.9; this results in adjusted_in ≈ BASE in
        // `math::calc_out_given_in` so we can use the same numbers as in the test above!
        let asset_amount_in = 11_111_111_111;
        let asset_amount_out = 9_900_990_100;
        assert_ok!(Swaps::swap_exact_amount_in(
            alice_signed(),
            pool_id,
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
                cpep: CommonPoolEventParams { pool_id, who: 0 },
                max_price,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 - asset_amount_in, _25 + asset_amount_out, _25, _25],
            0,
            [_100 + asset_amount_in, _100 - asset_amount_out, _100, _100],
            _100,
        );
    });
}

#[test]
fn swap_exact_amount_in_fails_if_no_limit_is_specified() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(1), true);
        assert_noop!(
            Swaps::swap_exact_amount_in(alice_signed(), 0, ASSET_A, _1, ASSET_B, None, None,),
            crate::Error::<Runtime>::LimitMissing
        );
    });
}

#[test]
fn swap_exact_amount_in_fails_if_min_asset_amount_out_is_not_satisfied_with_cpmm() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
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
            crate::Error::<Runtime>::LimitOut,
        );
    });
}

#[test]
fn swap_exact_amount_in_fails_if_max_price_is_not_satisfied_with_cpmm() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        // We're swapping 1:1, but due to slippage the price will exceed _1, so this should raise an
        // error:
        assert_noop!(
            Swaps::swap_exact_amount_in(alice_signed(), 0, ASSET_A, _1, ASSET_B, None, Some(_1)),
            crate::Error::<Runtime>::BadLimitPrice,
        );
    });
}

#[test]
fn swap_exact_amount_in_exchanges_correct_values_with_rikiddo() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::RikiddoSigmoidFeeMarketEma, None, true);
        let pool_id = 0;

        // Generate funds, add subsidy and start pool.
        subsidize_and_start_rikiddo_pool(pool_id, &ALICE, _1);
        assert_ok!(Currencies::deposit(ASSET_A, &ALICE, _1));

        // Check if unsupport trades are catched (base_asset in || asset_in == asset_out).
        assert_noop!(
            Swaps::swap_exact_amount_in(
                alice_signed(),
                pool_id,
                ASSET_D,
                _1,
                ASSET_B,
                Some(_1 / 2),
                Some(_2),
            ),
            crate::Error::<Runtime>::UnsupportedTrade
        );
        assert_noop!(
            Swaps::swap_exact_amount_in(
                alice_signed(),
                pool_id,
                ASSET_D,
                _1,
                ASSET_D,
                Some(_1 / 2),
                Some(_2),
            ),
            crate::Error::<Runtime>::UnsupportedTrade
        );
        assert_ok!(Currencies::withdraw(ASSET_D, &ALICE, _1));

        // Check if the trade is executed.
        let asset_a_issuance = Currencies::total_issuance(ASSET_A);
        assert_ok!(Swaps::swap_exact_amount_in(
            alice_signed(),
            pool_id,
            ASSET_A,
            _1,
            ASSET_D,
            Some(0),
            Some(_20),
        ));

        // Check if the balances were updated accordingly.
        let asset_a_issuance_after = Currencies::total_issuance(ASSET_A);
        let alice_balance_a_after = Currencies::total_balance(ASSET_A, &ALICE);
        let alice_balance_d_after = Currencies::total_balance(ASSET_D, &ALICE);
        assert_eq!(asset_a_issuance - asset_a_issuance_after, _1);
        assert_eq!(alice_balance_a_after, 0);

        // Received base_currency greater than 0.3 and smaller than 0.4
        assert!(alice_balance_d_after > 3 * BASE / 10 && alice_balance_d_after < 4 * BASE / 10);
    });
}

#[test]
fn swap_exact_amount_out_exchanges_correct_values_with_cpmm() {
    let asset_bound = Some(_2);
    let max_price = Some(_3);
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        assert_ok!(Swaps::swap_exact_amount_out(
            alice_signed(),
            0,
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
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                max_price,
            })
            .into(),
        );
        assert_all_parameters(
            [239898989900, _26, _25, _25],
            0,
            [_101 + 101010100, _99, _100, _100],
            _100,
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
            *ASSETS.last().unwrap(),
            0,
            ScoringRule::CPMM,
            Some(BASE / 10),
            Some(<Runtime as crate::Config>::MinLiquidity::get()),
            Some(vec!(_2, _2, _2, _2)),
        ));
        let pool_id = 0;
        assert_ok!(Swaps::open_pool(pool_id));

        let asset_amount_out = _1;
        let asset_amount_in = 11223344556; // 10101010100 / 0.9
        let asset_bound = Some(_2);
        let max_price = Some(_3);
        assert_ok!(Swaps::swap_exact_amount_out(
            alice_signed(),
            pool_id,
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
                cpep: CommonPoolEventParams { pool_id, who: 0 },
                max_price,
            })
            .into(),
        );
        assert_all_parameters(
            [_25 - asset_amount_in, _25 + asset_amount_out, _25, _25],
            0,
            [_100 + asset_amount_in, _100 - asset_amount_out, _100, _100],
            _100,
        );
    });
}

#[test]
fn swap_exact_amount_out_fails_if_no_limit_is_specified() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(1), true);
        assert_noop!(
            Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_A, None, ASSET_B, _1, None,),
            crate::Error::<Runtime>::LimitMissing
        );
    });
}

#[test]
fn swap_exact_amount_out_fails_if_min_asset_amount_out_is_not_satisfied_with_cpmm() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
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
            crate::Error::<Runtime>::LimitIn,
        );
    });
}

#[test]
fn swap_exact_amount_out_fails_if_max_price_is_not_satisfied_with_cpmm() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        // We're swapping 1:1, but due to slippage the price will exceed 1, so this should raise an
        // error:
        assert_noop!(
            Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_A, None, ASSET_B, _1, Some(_1)),
            crate::Error::<Runtime>::BadLimitPrice,
        );
    });
}

#[test]
fn swap_exact_amount_out_exchanges_correct_values_with_rikiddo() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool(ScoringRule::RikiddoSigmoidFeeMarketEma, None, true);
        let pool_id = 0;

        // Generate funds, add subsidy and start pool.
        subsidize_and_start_rikiddo_pool(pool_id, &ALICE, (BASE * 4) / 10);

        // Check if unsupport trades are catched (base_asset out || asset_in == asset_out).
        assert_noop!(
            Swaps::swap_exact_amount_out(
                alice_signed(),
                pool_id,
                ASSET_B,
                Some(_20),
                ASSET_D,
                _1,
                Some(_20),
            ),
            crate::Error::<Runtime>::UnsupportedTrade
        );
        assert_noop!(
            Swaps::swap_exact_amount_out(
                alice_signed(),
                pool_id,
                ASSET_D,
                Some(_2),
                ASSET_D,
                _1,
                Some(_2),
            ),
            crate::Error::<Runtime>::UnsupportedTrade
        );

        // Check if the trade is executed.
        let asset_a_issuance = Currencies::total_issuance(ASSET_A);
        assert_ok!(Swaps::swap_exact_amount_out(
            alice_signed(),
            pool_id,
            ASSET_D,
            Some(_1),
            ASSET_A,
            _1,
            Some(_20),
        ));

        // Check if the balances were updated accordingly.
        let asset_a_issuance_after = Currencies::total_issuance(ASSET_A);
        let alice_balance_a_after = Currencies::total_balance(ASSET_A, &ALICE);
        let alice_balance_d_after = Currencies::total_balance(ASSET_D, &ALICE);
        assert_eq!(asset_a_issuance_after - asset_a_issuance, _1);
        assert_eq!(alice_balance_a_after, _1);

        // Left over base currency must be less than 0.1
        assert!(alice_balance_d_after < BASE / 10);
    });
}

#[test]
fn create_pool_fails_on_too_many_assets() {
    ExtBuilder::default().build().execute_with(|| {
        let length = <Runtime as crate::Config>::MaxAssets::get();
        let assets: Vec<Asset<MarketId>> =
            (0..=length).map(|x| Asset::CategoricalOutcome(0, x)).collect::<Vec<_>>();
        let weights = vec![_2; length.into()];

        assets.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, _100);
        });

        assert_noop!(
            Swaps::create_pool(
                BOB,
                assets.clone(),
                *assets.last().unwrap(),
                0,
                ScoringRule::CPMM,
                Some(0),
                Some(<Runtime as crate::Config>::MinLiquidity::get()),
                Some(weights),
            ),
            crate::Error::<Runtime>::TooManyAssets
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
                ASSET_A,
                0,
                ScoringRule::CPMM,
                Some(0),
                Some(<Runtime as crate::Config>::MinLiquidity::get()),
                Some(vec!(_2, _2, _2, _2)),
            ),
            crate::Error::<Runtime>::TooFewAssets
        );
    });
}

#[test]
fn create_pool_fails_if_base_asset_is_not_in_asset_vector() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Swaps::create_pool(
                BOB,
                vec!(ASSET_A, ASSET_B, ASSET_C),
                ASSET_D,
                0,
                ScoringRule::CPMM,
                Some(0),
                Some(<Runtime as crate::Config>::MinLiquidity::get()),
                Some(vec!(_2, _2, _2)),
            ),
            crate::Error::<Runtime>::BaseAssetNotFound
        );
    });
}

#[test]
fn create_pool_fails_if_swap_fee_is_too_high() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = <Runtime as crate::Config>::MinLiquidity::get();
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, amount);
        });
        assert_noop!(
            Swaps::create_pool(
                BOB,
                ASSETS.to_vec(),
                ASSET_D,
                0,
                ScoringRule::CPMM,
                Some(<Runtime as crate::Config>::MaxSwapFee::get() + 1),
                Some(amount),
                Some(vec!(_2, _2, _2)),
            ),
            crate::Error::<Runtime>::SwapFeeTooHigh
        );
    });
}

#[test]
fn create_pool_fails_if_swap_fee_is_unspecified_for_cpmm() {
    ExtBuilder::default().build().execute_with(|| {
        let amount = <Runtime as crate::Config>::MinLiquidity::get();
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, amount);
        });
        assert_noop!(
            Swaps::create_pool(
                BOB,
                ASSETS.to_vec(),
                ASSET_D,
                0,
                ScoringRule::CPMM,
                None,
                Some(amount),
                Some(vec!(_2, _2, _2)),
            ),
            crate::Error::<Runtime>::InvalidFeeArgument
        );
    });
}

#[test]
fn join_pool_exit_pool_does_not_create_extra_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);

        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &CHARLIE, _100);
        });

        let amount = 123_456_789_123; // Strange number to force rounding errors!
        assert_ok!(Swaps::pool_join(
            Origin::signed(CHARLIE),
            0,
            amount,
            vec![_10000, _10000, _10000, _10000]
        ));
        assert_ok!(Swaps::pool_exit(Origin::signed(CHARLIE), 0, amount, vec![0, 0, 0, 0]));

        // Check that the pool retains more tokens than before, and that Charlie loses some tokens
        // due to fees.
        let pool_account_id = Swaps::pool_account_id(&0);
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
                *ASSETS.last().unwrap(),
                0,
                ScoringRule::CPMM,
                Some(0),
                Some(<Runtime as crate::Config>::MinLiquidity::get()),
                Some(vec!(_2, <Runtime as crate::Config>::MinWeight::get() - 1, _2, _2)),
            ),
            crate::Error::<Runtime>::BelowMinimumWeight,
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
                *ASSETS.last().unwrap(),
                0,
                ScoringRule::CPMM,
                Some(0),
                Some(<Runtime as crate::Config>::MinLiquidity::get()),
                Some(vec!(_2, <Runtime as crate::Config>::MaxWeight::get() + 1, _2, _2)),
            ),
            crate::Error::<Runtime>::AboveMaximumWeight,
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
            Swaps::create_pool(
                BOB,
                ASSETS.to_vec(),
                *ASSETS.last().unwrap(),
                0,
                ScoringRule::CPMM,
                Some(0),
                Some(<Runtime as crate::Config>::MinLiquidity::get()),
                Some(vec![weight; 4]),
            ),
            crate::Error::<Runtime>::MaxTotalWeight,
        );
    });
}

#[test]
fn create_pool_fails_on_insufficient_liquidity() {
    ExtBuilder::default().build().execute_with(|| {
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, _100);
        });
        assert_noop!(
            Swaps::create_pool(
                BOB,
                ASSETS.to_vec(),
                *ASSETS.last().unwrap(),
                0,
                ScoringRule::CPMM,
                Some(0),
                Some(<Runtime as crate::Config>::MinLiquidity::get() - 1),
                Some(vec!(_2, _2, _2, _2)),
            ),
            crate::Error::<Runtime>::InsufficientLiquidity,
        );
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
            *ASSETS.last().unwrap(),
            0,
            ScoringRule::CPMM,
            Some(0),
            Some(_1234),
            Some(vec!(_2, _2, _2, _2)),
        ));

        let pool_shares_id = Swaps::pool_shares_id(0);
        assert_eq!(Currencies::free_balance(pool_shares_id, &BOB), _1234);
        assert_eq!(Currencies::free_balance(ASSET_A, &BOB), _10000 - _1234);
        assert_eq!(Currencies::free_balance(ASSET_B, &BOB), _10000 - _1234);
        assert_eq!(Currencies::free_balance(ASSET_C, &BOB), _10000 - _1234);
        assert_eq!(Currencies::free_balance(ASSET_D, &BOB), _10000 - _1234);

        let pool_account_id = Swaps::pool_account_id(&0);
        assert_eq!(Currencies::free_balance(ASSET_A, &pool_account_id), _1234);
        assert_eq!(Currencies::free_balance(ASSET_B, &pool_account_id), _1234);
        assert_eq!(Currencies::free_balance(ASSET_C, &pool_account_id), _1234);
        assert_eq!(Currencies::free_balance(ASSET_D, &pool_account_id), _1234);
    });
}

#[test]
fn close_pool_fails_if_pool_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(Swaps::close_pool(0), crate::Error::<Runtime>::PoolDoesNotExist);
    });
}

#[test_case(PoolStatus::Closed; "closed")]
#[test_case(PoolStatus::Clean; "clean")]
#[test_case(PoolStatus::CollectingSubsidy; "collecting_subsidy")]
fn close_pool_fails_if_pool_is_not_active_or_initialized(pool_status: PoolStatus) {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, Some(0), true);
        let pool_id = 0;
        assert_ok!(Swaps::mutate_pool(pool_id, |pool| {
            pool.pool_status = pool_status;
            Ok(())
        }));
        assert_noop!(Swaps::close_pool(0), crate::Error::<Runtime>::InvalidStateTransition);
    });
}

#[test]
fn close_pool_succeeds_and_emits_correct_event_if_pool_exists() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool(ScoringRule::CPMM, Some(0), true);
        let pool_id = 0;
        assert_ok!(Swaps::close_pool(pool_id));
        let pool = Swaps::pool(pool_id).unwrap();
        assert_eq!(pool.pool_status, PoolStatus::Closed);
        System::assert_last_event(Event::PoolClosed(pool_id).into());
    });
}

#[test]
fn open_pool_fails_if_pool_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(Swaps::open_pool(0), crate::Error::<Runtime>::PoolDoesNotExist);
    });
}

#[test_case(PoolStatus::Active; "active")]
#[test_case(PoolStatus::Clean; "clean")]
#[test_case(PoolStatus::CollectingSubsidy; "collecting_subsidy")]
#[test_case(PoolStatus::Closed; "closed")]
fn open_pool_fails_if_pool_is_not_closed(pool_status: PoolStatus) {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, Some(1), true);
        let pool_id = 0;
        assert_ok!(Swaps::mutate_pool(pool_id, |pool| {
            pool.pool_status = pool_status;
            Ok(())
        }));
        assert_noop!(Swaps::open_pool(pool_id), crate::Error::<Runtime>::InvalidStateTransition);
    });
}

#[test]
fn open_pool_succeeds_and_emits_correct_event_if_pool_exists() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let amount = <Runtime as crate::Config>::MinLiquidity::get();
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, amount));
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            vec![ASSET_D, ASSET_B, ASSET_C, ASSET_A],
            ASSET_A,
            0,
            ScoringRule::CPMM,
            Some(0),
            Some(amount),
            Some(vec!(_1, _2, _3, _4)),
        ));
        let pool_id = 0;
        assert_ok!(Swaps::open_pool(pool_id));
        let pool = Swaps::pool(pool_id).unwrap();
        assert_eq!(pool.pool_status, PoolStatus::Active);
        System::assert_last_event(Event::PoolActive(pool_id).into());
    });
}

#[test]
fn pool_join_fails_if_max_assets_in_is_violated() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        assert_noop!(
            Swaps::pool_join(alice_signed(), 0, _1, vec!(_1, _1, _1 - 1, _1)),
            crate::Error::<Runtime>::LimitIn,
        );
    });
}

#[test]
fn pool_join_with_exact_asset_amount_fails_if_min_pool_tokens_is_violated() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
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
            crate::Error::<Runtime>::LimitOut,
        );
    });
}

#[test]
fn pool_join_with_exact_pool_amount_fails_if_max_asset_amount_is_violated() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
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
            crate::Error::<Runtime>::LimitIn,
        );
    });
}

#[test]
fn pool_exit_fails_if_min_assets_out_is_violated() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        assert_ok!(Swaps::pool_join(alice_signed(), 0, _1, vec!(_1, _1, _1, _1)));
        assert_noop!(
            Swaps::pool_exit(alice_signed(), 0, _1, vec!(_1, _1, _1 + 1, _1)),
            crate::Error::<Runtime>::LimitOut,
        );
    });
}

#[test]
fn pool_exit_with_exact_asset_amount_fails_if_min_pool_amount_is_violated() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&(BASE / 10));
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(alice_signed(), 0, ASSET_A, _5, 0));
        let pool_amount = Currencies::free_balance(Swaps::pool_shares_id(0), &ALICE);
        let expected_amount = 45_082_061_850;
        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(
                alice_signed(),
                0,
                ASSET_A,
                pool_amount,
                expected_amount + 100,
            ),
            crate::Error::<Runtime>::LimitOut,
        );
    });
}

#[test]
fn pool_exit_with_exact_pool_amount_fails_if_max_asset_amount_is_violated() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&(BASE / 10));
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(0), true);
        let asset_before_join = Currencies::free_balance(ASSET_A, &ALICE);
        assert_ok!(Swaps::pool_join_with_exact_pool_amount(alice_signed(), 0, ASSET_A, _1, _5));
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
            crate::Error::<Runtime>::LimitIn,
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
            ASSET_A,
            0,
            ScoringRule::CPMM,
            Some(0),
            Some(<Runtime as crate::Config>::MinLiquidity::get()),
            Some(vec!(_1, _2, _3, _4)),
        ));
        let pool = Swaps::pool(0).unwrap();
        let pool_weights = pool.weights.unwrap();
        assert_eq!(pool_weights[&ASSET_A], _4);
        assert_eq!(pool_weights[&ASSET_B], _2);
        assert_eq!(pool_weights[&ASSET_C], _3);
        assert_eq!(pool_weights[&ASSET_D], _1);
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
        create_initial_pool(ScoringRule::CPMM, Some(0), true);
        let pool_id = 0;
        assert_ok!(Currencies::deposit(asset, &ALICE, amount_in));
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            Origin::signed(ALICE),
            pool_id,
            asset,
            amount_in,
            0,
        ));
        let pool_amount = Currencies::free_balance(Swaps::pool_shares_id(pool_id), &ALICE);
        assert_ok!(Swaps::pool_exit_with_exact_pool_amount(
            Origin::signed(ALICE),
            pool_id,
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
        create_initial_pool(ScoringRule::CPMM, Some(swap_fee), true);
        let pool_id = 0;
        assert_ok!(Currencies::deposit(asset_in, &ALICE, amount_in));
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            Origin::signed(ALICE),
            pool_id,
            asset_in,
            amount_in,
            0,
        ));
        let pool_amount = Currencies::free_balance(Swaps::pool_shares_id(pool_id), &ALICE);
        assert_ok!(Swaps::pool_exit_with_exact_pool_amount(
            Origin::signed(ALICE),
            pool_id,
            asset_out,
            pool_amount,
            0,
        ));
        Currencies::free_balance(asset_out, &ALICE)
    });

    let amount_out_swap = ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, Some(swap_fee), true);
        let pool_id = 0;
        assert_ok!(Currencies::deposit(asset_in, &ALICE, amount_in));
        assert_ok!(Swaps::swap_exact_amount_in(
            Origin::signed(ALICE),
            pool_id,
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(1), true);
        let pool_id = 0;
        let pool_account_id = Swaps::pool_account_id(&pool_id);
        assert_ok!(Currencies::deposit(ASSET_A, &pool_account_id, _50));
        assert_ok!(Swaps::pool_join(Origin::signed(ALICE), pool_id, _10, vec![_100; 4]));
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(1), true);
        let pool_id = 0;
        let pool_account_id = Swaps::pool_account_id(&pool_id);

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
            Swaps::pool_exit(Origin::signed(BOB), pool_id, _10, vec![0; 4]),
            crate::Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn pool_exit_fails_if_liquidity_drops_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        // We drop the liquidity below `Swaps::min_balance(...)`, but balances remains above
        // `Swaps::min_balance(...)`.
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(1), true);
        let pool_id = 0;
        let pool_account_id = Swaps::pool_account_id(&pool_id);

        // There's 1000 left of each asset.
        assert_ok!(Currencies::deposit(ASSET_A, &pool_account_id, _900));
        assert_ok!(Currencies::deposit(ASSET_B, &pool_account_id, _900));
        assert_ok!(Currencies::deposit(ASSET_C, &pool_account_id, _900));
        assert_ok!(Currencies::deposit(ASSET_D, &pool_account_id, _900));

        // We withdraw too much liquidity but leave enough of each asset.
        assert_noop!(
            Swaps::pool_exit(
                Origin::signed(BOB),
                pool_id,
                _100 - Swaps::min_balance(Swaps::pool_shares_id(pool_id)) + 1,
                vec![0; 4]
            ),
            crate::Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn swap_exact_amount_in_fails_if_balances_drop_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(1), true);
        let pool_id = 0;
        let pool_account_id = Swaps::pool_account_id(&pool_id);

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
                Origin::signed(ALICE),
                pool_id,
                ASSET_A,
                Swaps::min_balance(ASSET_A) / 10,
                ASSET_B,
                Some(0),
                None,
            ),
            crate::Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn swap_exact_amount_out_fails_if_balances_drop_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(1), true);
        let pool_id = 0;
        let pool_account_id = Swaps::pool_account_id(&pool_id);

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
                Origin::signed(ALICE),
                pool_id,
                ASSET_A,
                Some(u128::MAX),
                ASSET_B,
                Swaps::min_balance(ASSET_B) / 10,
                None,
            ),
            crate::Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn pool_exit_with_exact_pool_amount_fails_if_balances_drop_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(1), true);
        let pool_id = 0;
        let pool_account_id = Swaps::pool_account_id(&pool_id);

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
            Swaps::pool_exit_with_exact_pool_amount(Origin::signed(BOB), pool_id, ASSET_A, _1, 0),
            crate::Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn pool_exit_with_exact_pool_amount_fails_if_liquidity_drops_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(1), true);
        let pool_id = 0;
        let pool_account_id = Swaps::pool_account_id(&pool_id);

        assert_ok!(Currencies::deposit(ASSET_A, &pool_account_id, _10000));
        assert_ok!(Currencies::deposit(ASSET_B, &pool_account_id, _10000));
        assert_ok!(Currencies::deposit(ASSET_C, &pool_account_id, _10000));
        assert_ok!(Currencies::deposit(ASSET_D, &pool_account_id, _10000));

        // Reduce amount of liquidity so that doing the withdraw doesn't cause a `Min*Ratio` error!
        let pool_shares_id = Swaps::pool_shares_id(pool_id);
        assert_eq!(Currencies::total_issuance(pool_shares_id), _100);
        Currencies::slash(pool_shares_id, &BOB, _100 - Swaps::min_balance(pool_shares_id));

        let ten_percent_of_pool = Swaps::min_balance(pool_shares_id) / 10;
        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(
                Origin::signed(BOB),
                pool_id,
                ASSET_A,
                ten_percent_of_pool,
                0,
            ),
            crate::Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn pool_exit_with_exact_asset_amount_fails_if_balances_drop_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(1), true);
        let pool_id = 0;
        let pool_account_id = Swaps::pool_account_id(&pool_id);

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
                Origin::signed(BOB),
                pool_id,
                ASSET_A,
                ten_percent_of_balance,
                _100,
            ),
            crate::Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn pool_exit_with_exact_asset_amount_fails_if_liquidity_drops_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(1), true);
        let pool_id = 0;

        // Reduce amount of liquidity so that doing the withdraw doesn't cause a `Min*Ratio` error!
        let pool_shares_id = Swaps::pool_shares_id(pool_id);
        assert_eq!(Currencies::total_issuance(pool_shares_id), _100);
        Currencies::slash(pool_shares_id, &BOB, _100 - Swaps::min_balance(pool_shares_id));

        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(
                Origin::signed(BOB),
                pool_id,
                ASSET_A,
                _25,
                _100,
            ),
            crate::Error::<Runtime>::PoolDrain,
        );
    });
}

#[test]
fn trading_functions_cache_pool_ids() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, Some(1), true);
        let pool_id = 0;

        assert_ok!(Swaps::pool_join_with_exact_pool_amount(
            Origin::signed(ALICE),
            pool_id,
            ASSET_A,
            _2,
            u128::MAX,
        ));
        assert!(PoolsCachedForArbitrage::<Runtime>::contains_key(pool_id));
        PoolsCachedForArbitrage::<Runtime>::remove(pool_id);

        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            Origin::signed(ALICE),
            pool_id,
            ASSET_A,
            _2,
            0,
        ));
        assert!(PoolsCachedForArbitrage::<Runtime>::contains_key(pool_id));
        PoolsCachedForArbitrage::<Runtime>::remove(pool_id);

        assert_ok!(Swaps::pool_exit_with_exact_asset_amount(
            Origin::signed(ALICE),
            pool_id,
            ASSET_A,
            _1,
            u128::MAX,
        ));
        assert!(PoolsCachedForArbitrage::<Runtime>::contains_key(pool_id));
        PoolsCachedForArbitrage::<Runtime>::remove(pool_id);

        assert_ok!(Swaps::pool_exit_with_exact_pool_amount(
            Origin::signed(ALICE),
            pool_id,
            ASSET_A,
            _1,
            0,
        ));
        assert!(PoolsCachedForArbitrage::<Runtime>::contains_key(pool_id));
        PoolsCachedForArbitrage::<Runtime>::remove(pool_id);

        assert_ok!(Swaps::swap_exact_amount_in(
            Origin::signed(ALICE),
            pool_id,
            ASSET_A,
            _1,
            ASSET_B,
            Some(0),
            None,
        ));
        assert!(PoolsCachedForArbitrage::<Runtime>::contains_key(pool_id));
        PoolsCachedForArbitrage::<Runtime>::remove(pool_id);

        assert_ok!(Swaps::swap_exact_amount_out(
            Origin::signed(ALICE),
            pool_id,
            ASSET_A,
            Some(u128::MAX),
            ASSET_B,
            _1,
            None,
        ));
        assert!(PoolsCachedForArbitrage::<Runtime>::contains_key(pool_id));
    });
}

#[test]
fn on_idle_skips_arbitrage_if_price_does_not_exceed_threshold() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let assets = ASSETS;
        assets.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, _10000));
        });
        // Outcome weights sum to the weight of the base asset, and we create no imbalances, so
        // total spot price is equal to 1.
        assert_ok!(Swaps::create_pool(
            BOB,
            assets.into(),
            ASSET_A,
            0,
            ScoringRule::CPMM,
            Some(0),
            Some(<Runtime as crate::Config>::MinLiquidity::get()),
            Some(vec![_3, _1, _1, _1]),
        ));
        let pool_id = 0;
        // Force the pool into the cache.
        crate::PoolsCachedForArbitrage::<Runtime>::insert(pool_id, ());
        Swaps::on_idle(System::block_number(), Weight::max_value());
        System::assert_has_event(Event::ArbitrageSkipped(pool_id).into());
    });
}

#[test]
fn on_idle_arbitrages_pools_with_mint_sell() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let assets = ASSETS;
        assets.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, _10000));
        });
        let balance = <Runtime as crate::Config>::MinLiquidity::get();
        let base_asset = ASSET_A;
        assert_ok!(Swaps::create_pool(
            BOB,
            assets.into(),
            base_asset,
            0,
            ScoringRule::CPMM,
            Some(0),
            Some(balance),
            Some(vec![_3, _1, _1, _1]),
        ));
        let pool_id = 0;

        // Withdraw a certain amount of outcome tokens to push the total spot price above 1
        // (ASSET_A is the base asset, all other assets are considered outcomes).
        let pool_account_id = Swaps::pool_account_id(&pool_id);
        let amount_removed = _25;
        assert_ok!(Currencies::withdraw(ASSET_B, &pool_account_id, amount_removed));

        // Force arbitrage hook.
        crate::PoolsCachedForArbitrage::<Runtime>::insert(pool_id, ());
        Swaps::on_idle(System::block_number(), Weight::max_value());

        let arbitrage_amount = 49_537_658_690;
        assert_eq!(
            Currencies::free_balance(base_asset, &pool_account_id),
            balance - arbitrage_amount,
        );
        assert_eq!(Currencies::free_balance(ASSET_C, &pool_account_id), balance + arbitrage_amount);
        assert_eq!(Currencies::free_balance(ASSET_D, &pool_account_id), balance + arbitrage_amount);
        assert_eq!(
            Currencies::free_balance(ASSET_B, &pool_account_id),
            balance + arbitrage_amount - amount_removed,
        );
        let market_id = 0;
        let market_account_id = MarketCommons::market_account(market_id);
        assert_eq!(Currencies::free_balance(base_asset, &market_account_id), arbitrage_amount);
        System::assert_has_event(Event::ArbitrageMintSell(pool_id, arbitrage_amount).into());
    });
}

#[test]
fn on_idle_arbitrages_pools_with_buy_burn() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let assets = ASSETS;
        assets.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, _10000));
        });
        let balance = <Runtime as crate::Config>::MinLiquidity::get();
        let base_asset = ASSET_A;
        assert_ok!(Swaps::create_pool(
            BOB,
            assets.into(),
            base_asset,
            0,
            ScoringRule::CPMM,
            Some(0),
            Some(balance),
            Some(vec![_3, _1, _1, _1]),
        ));
        let pool_id = 0;

        // Withdraw a certain amount of base tokens to push the total spot price below 1 (ASSET_A
        // is the base asset, all other assets are considered outcomes).
        let pool_account_id = Swaps::pool_account_id(&pool_id);
        let amount_removed = _25;
        assert_ok!(Currencies::withdraw(base_asset, &pool_account_id, amount_removed));

        // Deposit funds into the prize pool to ensure that the transfers don't fail.
        let market_id = 0;
        let market_account_id = MarketCommons::market_account(market_id);
        let arbitrage_amount = 125_007_629_394; // "Should" be 125_000_000_000.
        assert_ok!(Currencies::deposit(
            base_asset,
            &market_account_id,
            arbitrage_amount + SENTINEL_AMOUNT,
        ));

        // Force arbitrage hook.
        crate::PoolsCachedForArbitrage::<Runtime>::insert(pool_id, ());
        Swaps::on_idle(System::block_number(), Weight::max_value());

        assert_eq!(
            Currencies::free_balance(base_asset, &pool_account_id),
            balance + arbitrage_amount - amount_removed,
        );
        assert_eq!(Currencies::free_balance(ASSET_B, &pool_account_id), balance - arbitrage_amount);
        assert_eq!(Currencies::free_balance(ASSET_C, &pool_account_id), balance - arbitrage_amount);
        assert_eq!(Currencies::free_balance(ASSET_D, &pool_account_id), balance - arbitrage_amount);
        assert_eq!(Currencies::free_balance(base_asset, &market_account_id), SENTINEL_AMOUNT);
        System::assert_has_event(Event::ArbitrageBuyBurn(pool_id, arbitrage_amount).into());
    });
}

#[test]
fn apply_to_cached_pools_only_drains_requested_pools() {
    ExtBuilder::default().build().execute_with(|| {
        let pool_count = 5;
        for pool_id in 0..pool_count {
            // Force the pool into the cache.
            PoolsCachedForArbitrage::<Runtime>::insert(pool_id, ());
        }
        let number_of_pools_to_retain: u32 = 3;
        Swaps::apply_to_cached_pools(
            pool_count.saturated_into::<u32>() - number_of_pools_to_retain,
            |_| Ok(0),
            Weight::max_value(),
        );
        assert_eq!(
            PoolsCachedForArbitrage::<Runtime>::iter().count(),
            number_of_pools_to_retain as usize,
        );
    });
}

fn alice_signed() -> Origin {
    Origin::signed(ALICE)
}

fn create_initial_pool(
    scoring_rule: ScoringRule,
    swap_fee: Option<BalanceOf<Runtime>>,
    deposit: bool,
) {
    if deposit {
        ASSETS.iter().cloned().for_each(|asset| {
            assert_ok!(Currencies::deposit(asset, &BOB, _100));
        });
    }

    let pool_id = Swaps::next_pool_id();
    assert_ok!(Swaps::create_pool(
        BOB,
        ASSETS.to_vec(),
        *ASSETS.last().unwrap(),
        0,
        scoring_rule,
        swap_fee,
        if scoring_rule == ScoringRule::CPMM {
            Some(<Runtime as crate::Config>::MinLiquidity::get())
        } else {
            None
        },
        if scoring_rule == ScoringRule::CPMM { Some(vec!(_2, _2, _2, _2)) } else { None },
    ));
    if scoring_rule == ScoringRule::CPMM {
        assert_ok!(Swaps::open_pool(pool_id));
    }
}

fn create_initial_pool_with_funds_for_alice(
    scoring_rule: ScoringRule,
    swap_fee: Option<BalanceOf<Runtime>>,
    deposit: bool,
) {
    create_initial_pool(scoring_rule, swap_fee, deposit);
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
    let pai = Swaps::pool_account_id(&0);
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

// Subsidize and start a Rikiddo pool. Extra is the amount of additional base asset added to who.
fn subsidize_and_start_rikiddo_pool(
    pool_id: PoolId,
    who: &<Runtime as frame_system::Config>::AccountId,
    extra: crate::BalanceOf<Runtime>,
) {
    let min_subsidy = <Runtime as crate::Config>::MinSubsidy::get();
    assert_ok!(Currencies::deposit(ASSET_D, who, min_subsidy + extra));
    assert_ok!(Swaps::pool_join_subsidy(Origin::signed(*who), pool_id, min_subsidy));
    assert!(Swaps::end_subsidy_phase(pool_id).unwrap().result);
}

fn mock_market(categories: u16) -> Market<AccountIdTest, BlockNumber, Moment> {
    Market {
        creation: MarketCreation::Permissionless,
        creator_fee: 0,
        creator: ALICE,
        market_type: MarketType::Categorical(categories),
        dispute_mechanism: MarketDisputeMechanism::Authorized(ALICE),
        metadata: vec![0; 50],
        oracle: ALICE,
        period: MarketPeriod::Block(0..1),
        deadlines: Deadlines { grace_period: 1, oracle_duration: 1, dispute_duration: 1 },
        report: None,
        resolved_outcome: None,
        scoring_rule: ScoringRule::CPMM,
        status: MarketStatus::Active,
    }
}
