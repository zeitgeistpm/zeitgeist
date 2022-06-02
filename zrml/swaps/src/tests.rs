#![cfg(all(feature = "mock", test))]

use crate::{
    events::{CommonPoolEventParams, PoolAssetEvent, PoolAssetsEvent, SwapEvent},
    mock::*,
    Config, Event, SubsidyProviders,
};
use frame_support::{assert_err, assert_noop, assert_ok, assert_storage_noop, error::BadOrigin};
use more_asserts::{assert_ge, assert_le};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_runtime::SaturatedConversion;
use zeitgeist_primitives::{
    constants::BASE,
    traits::Swaps as _,
    types::{
        AccountIdTest, Asset, BlockNumber, Market, MarketCreation, MarketDisputeMechanism,
        MarketId, MarketPeriod, MarketStatus, MarketType, Moment, OutcomeReport, PoolId,
        PoolStatus, ScoringRule,
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

const _1_2: u128 = BASE / 2;
const _1: u128 = BASE;
const _2: u128 = 2 * BASE;
const _3: u128 = 3 * BASE;
const _4: u128 = 4 * BASE;
const _5: u128 = 5 * BASE;
const _8: u128 = 8 * BASE;
const _9: u128 = 9 * BASE;
const _10: u128 = 10 * BASE;
const _20: u128 = 20 * BASE;
const _24: u128 = 24 * BASE;
const _25: u128 = 25 * BASE;
const _26: u128 = 26 * BASE;
const _90: u128 = 90 * BASE;
const _99: u128 = 99 * BASE;
const _100: u128 = 100 * BASE;
const _101: u128 = 101 * BASE;
const _105: u128 = 105 * BASE;
const _1234: u128 = 1234 * BASE;
const _10000: u128 = 10000 * BASE;

#[test]
fn destroy_pool_fails_if_pool_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, true);
        assert_noop!(Swaps::destroy_pool(42), crate::Error::<Runtime>::PoolDoesNotExist);
    });
}

#[test]
fn destroy_pool_correctly_cleans_up_pool() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
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
        assert_all_parameters(alice_balance_before, 0, [0, 0, 0, 0], 0);
    });
}

#[test]
fn destroy_pool_emits_correct_event() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool(ScoringRule::CPMM, true);
        let pool_id = 0;
        assert_ok!(Swaps::destroy_pool(pool_id));
        System::assert_last_event(Event::PoolDestroyed(pool_id).into());
    });
}

#[test]
fn allows_the_full_user_lifecycle() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);

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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
        assert_ok!(Swaps::mutate_pool(0, |pool| {
            pool.weights.as_mut().unwrap().remove(&ASSET_B);
            Ok(())
        }));

        assert_noop!(
            Swaps::swap_exact_amount_in(alice_signed(), 0, ASSET_A, 1, ASSET_B, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
        );
        assert_noop!(
            Swaps::swap_exact_amount_in(alice_signed(), 0, ASSET_B, 1, ASSET_A, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
        );

        assert_noop!(
            Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_A, 1, ASSET_B, 1, 1),
            crate::Error::<Runtime>::AssetNotBound
        );
        assert_noop!(
            Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_B, 1, ASSET_A, 1, 1),
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

        create_initial_pool(ScoringRule::CPMM, true);

        let next_pool_after = Swaps::next_pool_id();
        assert_eq!(next_pool_after, 1);

        let pool = Swaps::pools(0).unwrap();

        assert_eq!(pool.assets, ASSETS.iter().cloned().collect::<Vec<_>>());
        assert_eq!(pool.scoring_rule, ScoringRule::CPMM);
        assert_eq!(pool.swap_fee.unwrap(), 0);
        assert_eq!(pool.total_subsidy, None);
        assert_eq!(pool.total_weight.unwrap(), _8);

        assert_eq!(*pool.weights.as_ref().unwrap().get(&ASSET_A).unwrap(), _2);
        assert_eq!(*pool.weights.as_ref().unwrap().get(&ASSET_B).unwrap(), _2);
        assert_eq!(*pool.weights.as_ref().unwrap().get(&ASSET_C).unwrap(), _2);
        assert_eq!(*pool.weights.as_ref().unwrap().get(&ASSET_D).unwrap(), _2);

        let pool_account = Swaps::pool_account_id(0);
        System::assert_last_event(
            Event::PoolCreate(
                CommonPoolEventParams { pool_id: next_pool_before, who: BOB },
                pool,
                <Runtime as Config>::MinLiquidity::get(),
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

        create_initial_pool(ScoringRule::RikiddoSigmoidFeeMarketEma, false);

        let next_pool_after = Swaps::next_pool_id();
        assert_eq!(next_pool_after, 1);
        let pool = Swaps::pools(0).unwrap();

        assert_eq!(pool.assets, ASSETS.iter().cloned().collect::<Vec<_>>());
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
        create_initial_pool(ScoringRule::CPMM, true);
        assert_noop!(
            Swaps::destroy_pool_in_subsidy_phase(0),
            crate::Error::<Runtime>::InvalidStateTransition
        );

        create_initial_pool_with_funds_for_alice(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
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
        create_initial_pool(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
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
                asset_per_acc + 20,
                winning_asset,
                asset_per_acc,
                _5
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
        create_initial_pool(ScoringRule::CPMM, true);
        assert_noop!(Swaps::end_subsidy_phase(0), crate::Error::<Runtime>::InvalidStateTransition);
        assert_noop!(Swaps::end_subsidy_phase(1), crate::Error::<Runtime>::PoolDoesNotExist);
        create_initial_pool_with_funds_for_alice(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
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
        assert_eq!(Swaps::end_subsidy_phase(pool_id).unwrap().result, true);

        // Check that subsidy was deposited, shares were distributed in exchange, the initial
        // outstanding event outcome assets are assigned to the pool account and pool is active.
        assert_eq!(Currencies::reserved_balance(ASSET_D, &ALICE), 0);
        assert_eq!(Currencies::reserved_balance(ASSET_D, &BOB), 0);

        let pool_shares_id = Swaps::pool_shares_id(pool_id);
        assert_eq!(Currencies::total_balance(pool_shares_id, &ALICE), subsidy_alice);
        assert_eq!(Currencies::total_balance(pool_shares_id, &BOB), subsidy_bob);

        let pool_account_id = Swaps::pool_account_id(pool_id);
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

#[test]
fn ensure_which_operations_can_be_called_depending_on_the_pool_status() {
    ExtBuilder::default().build().execute_with(|| {
        use zeitgeist_primitives::traits::Swaps as _;
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
        // For this test, we need to give Alice some pool shares, as well. We don't do this in
        // `create_initial_pool_...` so that there are exacly 100 pool shares, making computations
        // in other tests easier.
        let _ = Currencies::deposit(Swaps::pool_shares_id(0), &ALICE, _25);
        assert_ok!(Swaps::pool_join(alice_signed(), 0, _1, vec!(_1, _1, _1, _1),));

        assert_ok!(Swaps::set_pool_to_stale(
            &MarketType::Categorical(0),
            0,
            &OutcomeReport::Categorical(if let Asset::CategoricalOutcome(_, idx) = ASSET_A {
                idx
            } else {
                0
            }),
            &Default::default()
        ));

        assert_ok!(Swaps::pool_exit(alice_signed(), 0, _1, vec!(_1_2, _1_2)));
        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(alice_signed(), 0, ASSET_A, _1, _2),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(alice_signed(), 0, ASSET_A, _1, _1_2),
            crate::Error::<Runtime>::PoolIsNotActive
        );
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
        assert_ok!(Currencies::deposit(ASSET_A, &ALICE, u64::MAX.into()));
        assert_noop!(
            Swaps::swap_exact_amount_in(
                alice_signed(),
                0,
                ASSET_A,
                u64::MAX.into(),
                ASSET_B,
                _1,
                _1
            ),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        assert_noop!(
            Swaps::swap_exact_amount_out(
                alice_signed(),
                0,
                ASSET_A,
                u64::MAX.into(),
                ASSET_B,
                _1,
                _1
            ),
            crate::Error::<Runtime>::PoolIsNotActive
        );
    });
}

#[test]
fn get_spot_price_returns_correct_results() {
    ExtBuilder::default().build().execute_with(|| {
        // CPMM.
        create_initial_pool(ScoringRule::CPMM, true);
        assert_eq!(Swaps::get_spot_price(0, ASSETS[0], ASSETS[1]), Ok(BASE));

        // Rikiddo.
        create_initial_pool(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
        let pool_id = 1;
        assert_noop!(
            Swaps::get_spot_price(pool_id, ASSETS[0], ASSETS[0]),
            crate::Error::<Runtime>::PoolIsNotActive
        );
        subsidize_and_start_rikiddo_pool(pool_id, &ALICE, 0);

        // Asset out, base currency in. Should receive about 1/3 -> price about 3
        let price_base_in =
            Swaps::get_spot_price(pool_id, ASSETS[0], *ASSETS.last().unwrap()).unwrap();
        // Between 0.3 and 0.4
        assert!(price_base_in > 28 * BASE / 10 && price_base_in < 31 * BASE / 10);
        // Base currency in, asset out. Price about 3.
        let price_base_out =
            Swaps::get_spot_price(pool_id, *ASSETS.last().unwrap(), ASSETS[0]).unwrap();
        // Between 2.9 and 3.1
        assert!(price_base_out > 3 * BASE / 10 && price_base_out < 4 * BASE / 10);
        // Asset in, asset out. Price about 1.
        let price_asset_in_out = Swaps::get_spot_price(pool_id, ASSETS[0], ASSETS[1]).unwrap();
        // Between 0.9 and 1.1
        assert!(price_asset_in_out > 9 * BASE / 10 && price_asset_in_out < 11 * BASE / 10);
    });
}

#[test]
fn in_amount_must_be_equal_or_less_than_max_in_ratio() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, true);

        assert_ok!(Currencies::deposit(ASSET_A, &ALICE, u64::MAX.into()));

        assert_noop!(
            Swaps::swap_exact_amount_in(
                alice_signed(),
                0,
                ASSET_A,
                u64::MAX.into(),
                ASSET_B,
                _1,
                _1,
            ),
            crate::Error::<Runtime>::MaxInRatio
        );

        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(
                alice_signed(),
                0,
                ASSET_A,
                u64::MAX.into(),
                1
            ),
            crate::Error::<Runtime>::MaxInRatio
        );

        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(alice_signed(), 0, ASSET_A, _100, 0),
            crate::Error::<Runtime>::MaxInRatio
        );
    });
}

#[test]
fn pool_join_amount_satisfies_max_in_ratio_constraints() {
    ExtBuilder::default().build().execute_with(|| {
        // We want a special set of weights for this test!
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, _100);
        });
        assert_ok!(Swaps::create_pool(
            BOB,
            ASSETS.iter().cloned().collect(),
            ASSETS.last().unwrap().clone(),
            0,
            ScoringRule::CPMM,
            Some(0),
            Some(<Runtime as crate::Config>::MinLiquidity::get()),
            Some(vec!(_2, _2, _2, _5)) // Asset weights don't divide total weight.
        ));

        assert_ok!(Currencies::deposit(ASSET_D, &ALICE, u64::MAX.into()));

        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(
                alice_signed(),
                0,
                ASSET_A,
                _100,
                _10000 // Don't care how much we have to pay!
            ),
            crate::Error::<Runtime>::MaxOutRatio
        );
    });
}

#[test]
fn set_pool_to_stale_fails_if_origin_is_not_root() {
    ExtBuilder::default().build().execute_with(|| {
        let idx = if let Asset::CategoricalOutcome(_, idx) = ASSET_A { idx } else { 0 };
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
        assert_ok!(MarketCommons::push_market(mock_market(69)));
        MarketCommons::insert_market_pool(0, 0);
        assert_noop!(
            Swaps::admin_set_pool_to_stale(alice_signed(), 0, OutcomeReport::Categorical(idx)),
            BadOrigin
        );
    });
}

#[test]
fn out_amount_must_be_equal_or_less_than_max_out_ratio() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, true);

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
fn pool_join_or_exit_raises_on_zero_value() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);

        assert_noop!(
            Swaps::pool_join(alice_signed(), 0, 0, vec!(_1, _1, _1, _1)),
            crate::Error::<Runtime>::MathApproximation
        );

        assert_noop!(
            Swaps::pool_exit(alice_signed(), 0, 0, vec!(_1, _1, _1, _1)),
            crate::Error::<Runtime>::MathApproximation
        );

        assert_noop!(
            Swaps::pool_join_with_exact_pool_amount(alice_signed(), 0, ASSET_A, 0, 0),
            crate::Error::<Runtime>::MathApproximation
        );

        assert_noop!(
            Swaps::pool_join_with_exact_asset_amount(alice_signed(), 0, ASSET_A, 0, 0),
            crate::Error::<Runtime>::MathApproximation
        );

        assert_noop!(
            Swaps::pool_exit_with_exact_pool_amount(alice_signed(), 0, ASSET_A, 0, 0),
            crate::Error::<Runtime>::MathApproximation
        );

        assert_noop!(
            Swaps::pool_exit_with_exact_asset_amount(alice_signed(), 0, ASSET_A, 0, 0),
            crate::Error::<Runtime>::MathApproximation
        );
    });
}

#[test]
fn pool_exit_decreases_correct_pool_parameters() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);

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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);

        assert_ok!(Swaps::pool_exit(Origin::signed(BOB), 0, _10, vec!(_1, _1, _1, _1),));

        let pool_account = Swaps::pool_account_id(0);
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
fn pool_exit_decreases_correct_pool_parameters_on_stale_pool() {
    // Test is the same as
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
        assert_ok!(MarketCommons::push_market(mock_market(69)));
        MarketCommons::insert_market_pool(0, 0);

        assert_ok!(Swaps::pool_join(alice_signed(), 0, _1, vec!(_1, _1, _1, _1),));
        assert_ok!(Swaps::admin_set_pool_to_stale(
            Origin::root(),
            0,
            OutcomeReport::Categorical(65),
        ));
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

        create_initial_pool_with_funds_for_alice(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
        let pool_id = 0;

        // Add some subsidy
        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, _25));
        let mut reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        let mut noted = <SubsidyProviders<Runtime>>::get(pool_id, &ALICE).unwrap();
        let mut total_subsidy = Swaps::pool_by_id(pool_id).unwrap().total_subsidy.unwrap();
        assert_eq!(reserved, _25);
        assert_eq!(reserved, noted);
        assert_eq!(reserved, total_subsidy);

        // Exit 5 subsidy and see if the storage is consistent
        assert_ok!(Swaps::pool_exit_subsidy(alice_signed(), pool_id, _5));
        reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        noted = <SubsidyProviders<Runtime>>::get(pool_id, &ALICE).unwrap();
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
        assert!(<SubsidyProviders<Runtime>>::get(pool_id, &ALICE).is_none());
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
        assert!(<SubsidyProviders<Runtime>>::get(pool_id, &ALICE).is_none());
        total_subsidy = Swaps::pool_by_id(pool_id).unwrap().total_subsidy.unwrap();
        assert_eq!(reserved, 0);
        assert_eq!(reserved, total_subsidy);
    });
}

#[test]
fn pool_exit_subsidy_fails_if_no_subsidy_is_provided() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
        assert_noop!(
            Swaps::pool_exit_subsidy(alice_signed(), 0, _1),
            crate::Error::<Runtime>::NoSubsidyProvided
        );
    });
}

#[test]
fn pool_exit_subsidy_fails_if_amount_is_zero() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
        assert_noop!(
            Swaps::pool_exit_subsidy(alice_signed(), 0, _1),
            crate::Error::<Runtime>::InvalidScoringRule
        );
    });
}

#[test]
fn pool_exit_with_exact_pool_amount_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(alice_signed(), 0, ASSET_A, _5, 0));
        let pool_amount = Currencies::free_balance(Swaps::pool_shares_id(0), &ALICE);
        assert_ok!(Swaps::pool_exit_with_exact_pool_amount(
            alice_signed(),
            0,
            ASSET_A,
            pool_amount,
            _4
        ));
        System::assert_last_event(
            Event::PoolExitWithExactPoolAmount(PoolAssetEvent {
                asset: ASSET_A,
                bound: _4,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                transferred: _5 - 335,
                pool_amount,
            })
            .into(),
        );
        assert_all_parameters([_25 - 335, _25, _25, _25], 0, [_100 + 335, _100, _100, _100], _100)
    });
}

#[test]
fn pool_exit_with_exact_pool_amount_exchanges_correct_values_with_fee() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&(BASE / 10));
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(alice_signed(), 0, ASSET_A, _5, 0));
        let pool_amount = Currencies::free_balance(Swaps::pool_shares_id(0), &ALICE);
        assert_ok!(Swaps::pool_exit_with_exact_pool_amount(
            alice_signed(),
            0,
            ASSET_A,
            pool_amount,
            _4
        ));
        assert_all_parameters(
            [245_082_061_850, _25, _25, _25],
            0,
            [1_004_917_938_150, _100, _100, _100],
            _100,
        );
        System::assert_last_event(
            Event::PoolExitWithExactPoolAmount(PoolAssetEvent {
                asset: ASSET_A,
                bound: _4,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                transferred: 45_082_061_850,
                pool_amount,
            })
            .into(),
        );
    });
}

#[test]
fn pool_exit_with_exact_asset_amount_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&0u128);
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
        let asset_before_join = Currencies::free_balance(ASSET_A, &ALICE);
        assert_ok!(Swaps::pool_join_with_exact_pool_amount(alice_signed(), 0, ASSET_A, _1, _5));
        let pool_amount_before_exit = Currencies::free_balance(Swaps::pool_shares_id(0), &ALICE);
        let asset_after_join = asset_before_join - Currencies::free_balance(ASSET_A, &ALICE);
        assert_ok!(Swaps::pool_exit_with_exact_asset_amount(
            alice_signed(),
            0,
            ASSET_A,
            asset_after_join - 1000,
            _1
        ));
        let pool_amount_after_exit = Currencies::free_balance(Swaps::pool_shares_id(0), &ALICE);
        let pool_amount = pool_amount_before_exit - pool_amount_after_exit;
        System::assert_last_event(
            Event::PoolExitWithExactAssetAmount(PoolAssetEvent {
                asset: ASSET_A,
                bound: _1,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                transferred: asset_after_join - 1000,
                pool_amount,
            })
            .into(),
        );
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
fn pool_exit_with_exact_asset_amount_exchanges_correct_values_with_fee() {
    ExtBuilder::default().build().execute_with(|| {
        <Runtime as Config>::ExitFee::set(&(BASE / 10));
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
        let asset_before_join = Currencies::free_balance(ASSET_A, &ALICE);
        assert_ok!(Swaps::pool_join_with_exact_pool_amount(alice_signed(), 0, ASSET_A, _1, _5));
        let asset_after_join = asset_before_join - Currencies::free_balance(ASSET_A, &ALICE);
        let exit_amount = (asset_after_join * 9) / 10;
        let amount_left_behind_in_pool = asset_after_join - exit_amount;
        assert_ok!(Swaps::pool_exit_with_exact_asset_amount(
            alice_signed(),
            0,
            ASSET_A,
            exit_amount,
            _1
        ));
        let pool_amount = 9_984_935_413;
        System::assert_last_event(
            Event::PoolExitWithExactAssetAmount(PoolAssetEvent {
                asset: ASSET_A,
                bound: _1,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                transferred: exit_amount,
                pool_amount,
            })
            .into(),
        );
        assert_eq!(asset_after_join, 40604010000);
        let shares_remaining = _1 - pool_amount; // shares_after_join - pool_amount_in
        assert_all_parameters(
            [_25 - amount_left_behind_in_pool, _25, _25, _25],
            shares_remaining,
            [_100 + amount_left_behind_in_pool, _100, _100, _100],
            _100 + shares_remaining,
        )
    });
}

#[test]
fn pool_exit_is_not_allowed_with_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);

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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);

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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
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
        create_initial_pool_with_funds_for_alice(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
        let pool_id = 0;
        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, _20));
        let mut reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        let mut noted = <SubsidyProviders<Runtime>>::get(pool_id, &ALICE).unwrap();
        assert_eq!(reserved, _20);
        assert_eq!(reserved, noted);
        assert_eq!(reserved, Swaps::pool_by_id(pool_id).unwrap().total_subsidy.unwrap());
        System::assert_last_event(
            Event::PoolJoinSubsidy(ASSET_D, _20, CommonPoolEventParams { pool_id, who: ALICE })
                .into(),
        );

        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, _5));
        reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        noted = <SubsidyProviders<Runtime>>::get(pool_id, &ALICE).unwrap();
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
        create_initial_pool_with_funds_for_alice(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
        assert_noop!(
            Swaps::pool_join_subsidy(alice_signed(), 0, _1),
            crate::Error::<Runtime>::InvalidScoringRule
        );
    });
}

#[test]
fn pool_join_subsidy_fails_if_subsidy_is_below_min_per_account() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
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
        create_initial_pool_with_funds_for_alice(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
        let pool_id = 0;
        let large_amount = <Runtime as Config>::MinSubsidyPerAccount::get();
        let small_amount = 1;
        let total_amount = large_amount + small_amount;
        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, large_amount));
        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, small_amount));
        let reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        let noted = <SubsidyProviders<Runtime>>::get(pool_id, &ALICE).unwrap();
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
        create_initial_pool_with_funds_for_alice(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
        let pool_id = 0;
        let large_amount = <Runtime as Config>::MinSubsidyPerAccount::get();
        let small_amount = 1;
        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, large_amount));
        assert_ok!(Swaps::pool_exit_subsidy(alice_signed(), pool_id, small_amount));
        let reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        let noted = <SubsidyProviders<Runtime>>::get(pool_id, &ALICE);
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

#[test]
fn pool_join_with_exact_asset_amount_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
        let alice_sent = _1;
        assert_ok!(Swaps::pool_join_with_exact_asset_amount(
            alice_signed(),
            0,
            ASSET_A,
            alice_sent,
            0
        ));
        let alice_received = Currencies::free_balance(Swaps::pool_shares_id(0), &ALICE);
        System::assert_last_event(
            Event::PoolJoinWithExactAssetAmount(PoolAssetEvent {
                asset: ASSET_A,
                bound: 0,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                transferred: alice_sent,
                pool_amount: alice_received,
            })
            .into(),
        );
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
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
        System::assert_last_event(
            Event::PoolJoinWithExactPoolAmount(PoolAssetEvent {
                asset: ASSET_A,
                bound: _5,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                transferred: asset_amount,
                pool_amount: alice_sent,
            })
            .into(),
        );
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
        create_initial_pool(ScoringRule::CPMM, true);
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
fn set_pool_to_stale_leaves_only_correct_assets() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, true);
        let pool_id = 0;

        assert_noop!(
            Swaps::set_pool_to_stale(
                &MarketType::Categorical(1337),
                pool_id,
                &OutcomeReport::Categorical(1337),
                &Default::default()
            ),
            crate::Error::<Runtime>::WinningAssetNotFound
        );

        let cat_idx = if let Asset::CategoricalOutcome(_, cidx) = ASSET_A { cidx } else { 0 };

        assert_ok!(Swaps::set_pool_to_stale(
            &MarketType::Categorical(4),
            pool_id,
            &OutcomeReport::Categorical(cat_idx),
            &Default::default()
        ));

        assert_eq!(Swaps::pool_by_id(pool_id).unwrap().pool_status, PoolStatus::Stale);
        assert_eq!(Swaps::pool_by_id(pool_id).unwrap().assets, vec![ASSET_A, ASSET_D]);
    });
}

#[test]
fn set_pool_to_stale_handles_rikiddo_pools_properly() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
        let pool_id = 0;

        let cat_idx = if let Asset::CategoricalOutcome(_, cidx) = ASSET_A { cidx } else { 0 };

        assert_noop!(
            Swaps::set_pool_to_stale(
                &MarketType::Categorical(4),
                pool_id,
                &OutcomeReport::Categorical(cat_idx),
                &Default::default()
            ),
            crate::Error::<Runtime>::InvalidStateTransition
        );

        assert_ok!(Swaps::mutate_pool(pool_id, |pool| {
            pool.pool_status = PoolStatus::Active;
            Ok(())
        }));

        assert_ok!(Swaps::set_pool_to_stale(
            &MarketType::Categorical(4),
            pool_id,
            &OutcomeReport::Categorical(cat_idx),
            &Default::default()
        ));

        // Rikiddo instance does not exist anymore.
        assert_storage_noop!(RikiddoSigmoidFeeMarketEma::clear(pool_id).unwrap_or(()));
    });
}

#[test]
fn swap_exact_amount_in_exchanges_correct_values_with_cpmm() {
    ExtBuilder::default().build().execute_with(|| {
        // CPMM
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
        assert_ok!(Swaps::swap_exact_amount_in(
            alice_signed(),
            0,
            ASSET_A,
            _1,
            ASSET_B,
            _1 / 2,
            _2,
        ));
        System::assert_last_event(
            Event::SwapExactAmountIn(SwapEvent {
                asset_amount_in: _1,
                asset_amount_out: 9900990100,
                asset_bound: _1 / 2,
                asset_in: ASSET_A,
                asset_out: ASSET_B,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                max_price: _2,
            })
            .into(),
        );
        assert_all_parameters(
            [_24, _25 + 9900990100, _25, _25],
            0,
            [_101, _99 + 0099009900, _100, _100],
            _100,
        );
    });
}

#[test]
fn swap_exact_amount_in_exchanges_correct_values_with_rikiddo() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::RikiddoSigmoidFeeMarketEma, true);
        let pool_id = 0;

        // Generate funds, add subsidy and start pool.
        subsidize_and_start_rikiddo_pool(pool_id, &ALICE, _1);
        assert_ok!(Currencies::deposit(ASSET_A, &ALICE, _1));

        // Check if unsupport trades are catched (base_asset in || asset_in == asset_out).
        assert_noop!(
            Swaps::swap_exact_amount_in(alice_signed(), pool_id, ASSET_D, _1, ASSET_B, _1 / 2, _2,),
            crate::Error::<Runtime>::UnsupportedTrade
        );
        assert_noop!(
            Swaps::swap_exact_amount_in(alice_signed(), pool_id, ASSET_D, _1, ASSET_D, _1 / 2, _2,),
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
            0,
            _20,
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
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
        assert_ok!(Swaps::swap_exact_amount_out(alice_signed(), 0, ASSET_A, _2, ASSET_B, _1, _3,));
        System::assert_last_event(
            Event::SwapExactAmountOut(SwapEvent {
                asset_amount_in: 10101010100,
                asset_amount_out: _1,
                asset_bound: _2,
                asset_in: ASSET_A,
                asset_out: ASSET_B,
                cpep: CommonPoolEventParams { pool_id: 0, who: 0 },
                max_price: _3,
            })
            .into(),
        );
        assert_all_parameters(
            [239898989900, _26, _25, _25],
            0,
            [_101 + 0101010100, _99, _100, _100],
            _100,
        );
    });
}

#[test]
fn swap_exact_amount_out_exchanges_correct_values_with_rikiddo() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool(ScoringRule::RikiddoSigmoidFeeMarketEma, true);
        let pool_id = 0;

        // Generate funds, add subsidy and start pool.
        subsidize_and_start_rikiddo_pool(pool_id, &ALICE, (BASE * 4) / 10);

        // Check if unsupport trades are catched (base_asset out || asset_in == asset_out).
        assert_noop!(
            Swaps::swap_exact_amount_out(alice_signed(), pool_id, ASSET_B, _20, ASSET_D, _1, _20,),
            crate::Error::<Runtime>::UnsupportedTrade
        );
        assert_noop!(
            Swaps::swap_exact_amount_out(alice_signed(), pool_id, ASSET_D, _2, ASSET_D, _1, _2,),
            crate::Error::<Runtime>::UnsupportedTrade
        );

        // Check if the trade is executed.
        let asset_a_issuance = Currencies::total_issuance(ASSET_A);
        assert_ok!(Swaps::swap_exact_amount_out(
            alice_signed(),
            pool_id,
            ASSET_D,
            _1,
            ASSET_A,
            _1,
            _20,
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
                assets.last().unwrap().clone(),
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

// Macro for comparing fixed point u128.
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

#[test]
fn join_pool_exit_pool_does_not_create_extra_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);

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
        let pool_account_id = Swaps::pool_account_id(0);
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
                ASSETS.iter().cloned().collect(),
                ASSETS.last().unwrap().clone(),
                0,
                ScoringRule::CPMM,
                Some(0),
                Some(<Runtime as crate::Config>::MinLiquidity::get()),
                Some(vec!(_2, <Runtime as crate::Config>::MinWeight::get() - 1, _2, _2))
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
                ASSETS.iter().cloned().collect(),
                ASSETS.last().unwrap().clone(),
                0,
                ScoringRule::CPMM,
                Some(0),
                Some(<Runtime as crate::Config>::MinLiquidity::get()),
                Some(vec!(_2, <Runtime as crate::Config>::MaxWeight::get() + 1, _2, _2))
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
                ASSETS.iter().cloned().collect(),
                ASSETS.last().unwrap().clone(),
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
                ASSETS.iter().cloned().collect(),
                ASSETS.last().unwrap().clone(),
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
            ASSETS.iter().cloned().collect(),
            ASSETS.last().unwrap().clone(),
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

        let pool_account_id = Swaps::pool_account_id(0);
        assert_eq!(Currencies::free_balance(ASSET_A, &pool_account_id), _1234);
        assert_eq!(Currencies::free_balance(ASSET_B, &pool_account_id), _1234);
        assert_eq!(Currencies::free_balance(ASSET_C, &pool_account_id), _1234);
        assert_eq!(Currencies::free_balance(ASSET_D, &pool_account_id), _1234);
    });
}

fn alice_signed() -> Origin {
    Origin::signed(ALICE)
}

fn create_initial_pool(scoring_rule: ScoringRule, deposit: bool) {
    if deposit {
        ASSETS.iter().cloned().for_each(|asset| {
            let _ = Currencies::deposit(asset, &BOB, _100);
        });
    }

    assert_ok!(Swaps::create_pool(
        BOB,
        ASSETS.iter().cloned().collect(),
        ASSETS.last().unwrap().clone(),
        0,
        scoring_rule,
        if scoring_rule == ScoringRule::CPMM { Some(0) } else { None },
        if scoring_rule == ScoringRule::CPMM {
            Some(<Runtime as crate::Config>::MinLiquidity::get())
        } else {
            None
        },
        if scoring_rule == ScoringRule::CPMM { Some(vec!(_2, _2, _2, _2)) } else { None },
    ));
}

fn create_initial_pool_with_funds_for_alice(scoring_rule: ScoringRule, deposit: bool) {
    create_initial_pool(scoring_rule, deposit);
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

// Subsidize and start a Rikiddo pool. Extra is the amount of additional base asset added to who.
fn subsidize_and_start_rikiddo_pool(
    pool_id: PoolId,
    who: &<Runtime as frame_system::Config>::AccountId,
    extra: crate::BalanceOf<Runtime>,
) {
    let min_subsidy = <Runtime as crate::Config>::MinSubsidy::get();
    assert_ok!(Currencies::deposit(ASSET_D, who, min_subsidy + extra));
    assert_ok!(Swaps::pool_join_subsidy(Origin::signed(*who), pool_id, min_subsidy));
    assert_eq!(Swaps::end_subsidy_phase(pool_id).unwrap().result, true);
}

fn mock_market(categories: u16) -> Market<AccountIdTest, BlockNumber, Moment> {
    Market {
        creation: MarketCreation::Permissionless,
        creator_fee: 0,
        creator: ALICE,
        market_type: MarketType::Categorical(categories),
        mdm: MarketDisputeMechanism::Authorized(ALICE),
        metadata: vec![0; 50],
        oracle: ALICE,
        period: MarketPeriod::Block(0..1),
        report: None,
        resolved_outcome: None,
        scoring_rule: ScoringRule::CPMM,
        status: MarketStatus::Active,
    }
}
