#![cfg(all(feature = "mock", test))]

use crate::{
    events::{CommonPoolEventParams, PoolAssetEvent, PoolAssetsEvent, SwapEvent},
    mock::*,
    SubsidyProviders,
};
use frame_support::{assert_noop, assert_ok, assert_storage_noop, error::BadOrigin};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_runtime::SaturatedConversion;
use zeitgeist_primitives::{
    constants::BASE,
    traits::Swaps as _,
    types::{
        AccountIdTest, Asset, MarketId, MarketType, OutcomeReport, PoolId, PoolStatus, ScoringRule,
    },
};
use zrml_rikiddo::traits::RikiddoMVPallet;

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
        assert_eq!(pool.base_asset.unwrap(), ASSET_D);
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
        assert_eq!(Currencies::reserved_balance(ASSET_D, &ALICE), _25);
        assert_ok!(Swaps::destroy_pool_in_subsidy_phase(pool_id));
        // Rserved balanced was returned and all storage cleared.
        assert_eq!(Currencies::reserved_balance(ASSET_D, &ALICE), 0);
        assert!(!crate::SubsidyProviders::<Runtime>::contains_key(pool_id, ALICE));
        assert!(!crate::Pools::<Runtime>::contains_key(pool_id));
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
        let base_asset = Swaps::pool_by_id(pool_id).unwrap().base_asset.unwrap();
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
        create_initial_pool(ScoringRule::CPMM, true);
        assert_noop!(Swaps::end_subsidy_phase(0), crate::Error::<Runtime>::InvalidStateTransition);
        assert_noop!(Swaps::end_subsidy_phase(1), crate::Error::<Runtime>::PoolDoesNotExist);
        create_initial_pool_with_funds_for_alice(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
        let pool_id = 1;
        assert_storage_noop!(Swaps::end_subsidy_phase(pool_id).unwrap());

        // Reserve some funds for subsidy
        let min_subsidy = <Runtime as crate::Config>::MinSubsidy::get();
        subsidize_and_start_rikiddo_pool(pool_id, &ALICE, 0);

        // Check that subsidy was deposited, shares were distributed in exchange, the initial
        // outstanding event outcome assets are assigned to the pool account and pool is active.
        assert_eq!(Currencies::reserved_balance(ASSET_D, &ALICE), 0);

        let pool_shares_id = Swaps::pool_shares_id(pool_id);
        assert_eq!(Currencies::total_balance(pool_shares_id, &ALICE), min_subsidy);

        let pool_account_id = Swaps::pool_account_id(pool_id);
        let total_subsidy = Currencies::total_balance(ASSET_D, &pool_account_id);
        assert_eq!(total_subsidy, min_subsidy);
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
        assert_ok!(Swaps::pool_join(alice_signed(), 0, _1, vec!(_1, _1, _1, _1),));

        assert_ok!(Swaps::set_pool_as_stale(
            &MarketType::Categorical(0),
            0,
            &OutcomeReport::Categorical(if let Asset::CategoricalOutcome(_, idx) = ASSET_A {
                idx
            } else {
                0
            }),
            &Default::default()
        ));

        assert_ok!(Swaps::pool_exit(alice_signed(), 0, _1, vec!(_1, _1)));
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
    });
}

#[test]
fn only_root_can_call_admin_set_pool_as_stale() {
    ExtBuilder::default().build().execute_with(|| {
        let idx = if let Asset::CategoricalOutcome(_, idx) = ASSET_A { idx } else { 0 };
        assert_noop!(
            Swaps::admin_set_pool_as_stale(
                alice_signed(),
                MarketType::Categorical(0),
                0,
                OutcomeReport::Categorical(idx)
            ),
            BadOrigin
        );

        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
        assert_ok!(Swaps::pool_join(alice_signed(), 0, _1, vec!(_1, _1, _1, _1),));
        assert_ok!(Swaps::admin_set_pool_as_stale(
            Origin::root(),
            MarketType::Categorical(0),
            0,
            OutcomeReport::Categorical(idx)
        ),);
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
fn pool_amount_must_not_be_zero() {
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
    });
}

#[test]
fn pool_exit_decreases_correct_pool_parameters() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);

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
fn pool_exit_subsidy_unreserves_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, true);
        assert_noop!(
            Swaps::pool_exit_subsidy(alice_signed(), 0, 42),
            crate::Error::<Runtime>::InvalidScoringRule
        );
        assert_noop!(
            Swaps::pool_exit_subsidy(alice_signed(), 1, 42),
            crate::Error::<Runtime>::PoolDoesNotExist
        );
        create_initial_pool_with_funds_for_alice(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
        let pool_id = 1;
        assert_noop!(
            Swaps::pool_exit_subsidy(alice_signed(), pool_id, 42),
            crate::Error::<Runtime>::NoSubsidyProvided
        );

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

        // Exit the remaining subsidy and see if the storage is consistent
        assert_ok!(Swaps::pool_exit_subsidy(alice_signed(), pool_id, _20));
        reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        assert!(<SubsidyProviders<Runtime>>::get(pool_id, &ALICE).is_none());
        total_subsidy = Swaps::pool_by_id(pool_id).unwrap().total_subsidy.unwrap();
        assert_eq!(reserved, 0);
        assert_eq!(reserved, total_subsidy);

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
fn pool_exit_with_exact_pool_amount_exchanges_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);
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
        create_initial_pool_with_funds_for_alice(ScoringRule::CPMM, true);

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
fn pool_join_subsidy_reserves_correct_values() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, true);
        assert_noop!(
            Swaps::pool_join_subsidy(alice_signed(), 0, 42),
            crate::Error::<Runtime>::InvalidScoringRule
        );
        create_initial_pool_with_funds_for_alice(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
        let pool_id = 1;
        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, _20));
        let mut reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        let mut noted = <SubsidyProviders<Runtime>>::get(pool_id, &ALICE).unwrap();
        assert_eq!(reserved, _20);
        assert_eq!(reserved, noted);
        assert_eq!(reserved, Swaps::pool_by_id(pool_id).unwrap().total_subsidy.unwrap());
        assert_ok!(Swaps::pool_join_subsidy(alice_signed(), pool_id, _5));
        reserved = Currencies::reserved_balance(ASSET_D, &ALICE);
        noted = <SubsidyProviders<Runtime>>::get(pool_id, &ALICE).unwrap();
        assert_eq!(reserved, _25);
        assert_eq!(reserved, noted);
        assert_eq!(reserved, Swaps::pool_by_id(pool_id).unwrap().total_subsidy.unwrap());
        assert_storage_noop!(Swaps::pool_join_subsidy(alice_signed(), pool_id, _5).unwrap_or(()));
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
fn set_pool_as_stale_leaves_only_correct_assets() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::CPMM, true);
        let pool_id = 0;

        assert_noop!(
            Swaps::set_pool_as_stale(
                &MarketType::Categorical(1337),
                pool_id,
                &OutcomeReport::Categorical(1337),
                &Default::default()
            ),
            crate::Error::<Runtime>::WinningAssetNotFound
        );

        let cat_idx = if let Asset::CategoricalOutcome(_, cidx) = ASSET_A { cidx } else { 0 };

        assert_ok!(Swaps::set_pool_as_stale(
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
fn set_pool_as_stale_handles_rikiddo_pools_properly() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool(ScoringRule::RikiddoSigmoidFeeMarketEma, false);
        let pool_id = 0;

        let cat_idx = if let Asset::CategoricalOutcome(_, cidx) = ASSET_A { cidx } else { 0 };

        assert_noop!(
            Swaps::set_pool_as_stale(
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

        assert_ok!(Swaps::set_pool_as_stale(
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
        Some(ASSETS.last().unwrap().clone()),
        0,
        scoring_rule,
        if scoring_rule == ScoringRule::CPMM { Some(0) } else { None },
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
