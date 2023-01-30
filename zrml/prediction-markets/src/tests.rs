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

#![cfg(all(feature = "mock", test))]
#![allow(clippy::reversed_empty_ranges)]

use crate::{
    default_dispute_bond, mock::*, Config, Disputes, Error, Event, LastTimeFrame, MarketIdsForEdit,
    MarketIdsPerCloseBlock, MarketIdsPerDisputeBlock, MarketIdsPerOpenBlock,
    MarketIdsPerReportBlock, TimeFrame,
};
use core::ops::{Range, RangeInclusive};
use frame_support::{
    assert_err, assert_noop, assert_ok,
    dispatch::{DispatchError, DispatchResultWithPostInfo},
    traits::{NamedReservableCurrency, OnInitialize},
};
use test_case::test_case;

use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_runtime::traits::{AccountIdConversion, SaturatedConversion, Zero};
use zeitgeist_primitives::{
    constants::mock::{DisputeFactor, OutsiderBond, BASE, CENT, MILLISECS_PER_BLOCK},
    traits::Swaps as SwapsPalletApi,
    types::{
        AccountIdTest, Asset, Balance, BlockNumber, Bond, Deadlines, Market, MarketBonds,
        MarketCreation, MarketDisputeMechanism, MarketId, MarketPeriod, MarketStatus, MarketType,
        Moment, MultiHash, OutcomeReport, PoolStatus, ScalarPosition, ScoringRule,
    },
};
use zrml_authorized::Error as AuthorizedError;
use zrml_market_commons::MarketCommonsPalletApi;
use zrml_swaps::Pools;

const SENTINEL_AMOUNT: u128 = BASE;

fn get_deadlines() -> Deadlines<<Runtime as frame_system::Config>::BlockNumber> {
    Deadlines {
        grace_period: 1_u32.into(),
        oracle_duration: <Runtime as crate::Config>::MinOracleDuration::get(),
        dispute_duration: <Runtime as crate::Config>::MinDisputeDuration::get(),
    }
}

fn gen_metadata(byte: u8) -> MultiHash {
    let mut metadata = [byte; 50];
    metadata[0] = 0x15;
    metadata[1] = 0x30;
    MultiHash::Sha3_384(metadata)
}

fn reserve_sentinel_amounts() {
    // Reserve a sentinel amount to check that we don't unreserve too much.
    assert_ok!(Balances::reserve_named(&PredictionMarkets::reserve_id(), &ALICE, SENTINEL_AMOUNT));
    assert_ok!(Balances::reserve_named(&PredictionMarkets::reserve_id(), &BOB, SENTINEL_AMOUNT));
    assert_ok!(Balances::reserve_named(
        &PredictionMarkets::reserve_id(),
        &CHARLIE,
        SENTINEL_AMOUNT
    ));
    assert_ok!(Balances::reserve_named(&PredictionMarkets::reserve_id(), &DAVE, SENTINEL_AMOUNT));
    assert_ok!(Balances::reserve_named(&PredictionMarkets::reserve_id(), &EVE, SENTINEL_AMOUNT));
    assert_ok!(Balances::reserve_named(&PredictionMarkets::reserve_id(), &FRED, SENTINEL_AMOUNT));
    assert_eq!(Balances::reserved_balance(&ALICE), SENTINEL_AMOUNT);
    assert_eq!(Balances::reserved_balance(&BOB), SENTINEL_AMOUNT);
    assert_eq!(Balances::reserved_balance(&CHARLIE), SENTINEL_AMOUNT);
    assert_eq!(Balances::reserved_balance(&DAVE), SENTINEL_AMOUNT);
    assert_eq!(Balances::reserved_balance(&EVE), SENTINEL_AMOUNT);
    assert_eq!(Balances::reserved_balance(&FRED), SENTINEL_AMOUNT);
}

fn check_reserve(account: &AccountIdTest, expected: Balance) {
    assert_eq!(Balances::reserved_balance(account), SENTINEL_AMOUNT + expected);
}

fn simple_create_categorical_market(
    base_asset: Asset<MarketId>,
    creation: MarketCreation,
    period: Range<u64>,
    scoring_rule: ScoringRule,
) {
    assert_ok!(PredictionMarkets::create_market(
        Origin::signed(ALICE),
        base_asset,
        BOB,
        MarketPeriod::Block(period),
        get_deadlines(),
        gen_metadata(2),
        creation,
        MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
        MarketDisputeMechanism::SimpleDisputes,
        scoring_rule
    ));
}

fn simple_create_scalar_market(
    base_asset: Asset<MarketId>,
    creation: MarketCreation,
    period: Range<u64>,
    scoring_rule: ScoringRule,
) {
    assert_ok!(PredictionMarkets::create_market(
        Origin::signed(ALICE),
        base_asset,
        BOB,
        MarketPeriod::Block(period),
        get_deadlines(),
        gen_metadata(2),
        creation,
        MarketType::Scalar(100..=200),
        MarketDisputeMechanism::SimpleDisputes,
        scoring_rule
    ));
}

#[test]
fn admin_move_market_to_closed_successfully_closes_market_and_sets_end_blocknumber() {
    ExtBuilder::default().build().execute_with(|| {
        run_blocks(7);
        let now = frame_system::Pallet::<Runtime>::block_number();
        let end = 42;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            now..end,
            ScoringRule::CPMM,
        );
        run_blocks(3);
        let market_id = 0;
        assert_ok!(PredictionMarkets::admin_move_market_to_closed(Origin::signed(SUDO), market_id));
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        let new_end = now + 3;
        assert_eq!(market.period, MarketPeriod::Block(now..new_end));
        assert_ne!(new_end, end);
        System::assert_last_event(Event::MarketClosed(market_id).into());
    });
}

#[test]
fn admin_move_market_to_closed_successfully_closes_market_and_sets_end_timestamp() {
    ExtBuilder::default().build().execute_with(|| {
        let start_block = 7;
        set_timestamp_for_on_initialize(start_block * MILLISECS_PER_BLOCK as u64);
        run_blocks(start_block);
        let start = <zrml_market_commons::Pallet<Runtime>>::now();

        let end = start + 42.saturated_into::<TimeFrame>() * MILLISECS_PER_BLOCK as u64;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Timestamp(start..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.period, MarketPeriod::Timestamp(start..end));

        let shift_blocks = 3;
        let shift = shift_blocks * MILLISECS_PER_BLOCK as u64;
        // millisecs per block is substracted inside the function
        set_timestamp_for_on_initialize(start + shift + MILLISECS_PER_BLOCK as u64);
        run_blocks(shift_blocks);

        assert_ok!(PredictionMarkets::admin_move_market_to_closed(Origin::signed(SUDO), market_id));
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        let new_end = start + shift;
        assert_eq!(market.period, MarketPeriod::Timestamp(start..new_end));
        assert_ne!(new_end, end);
        System::assert_last_event(Event::MarketClosed(market_id).into());
    });
}

#[test]
fn admin_move_market_to_closed_fails_if_market_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::admin_move_market_to_closed(Origin::signed(SUDO), 0),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test_case(MarketStatus::Closed; "closed")]
#[test_case(MarketStatus::Reported; "reported")]
#[test_case(MarketStatus::Disputed; "disputed")]
#[test_case(MarketStatus::Resolved; "resolved")]
#[test_case(MarketStatus::Proposed; "proposed")]
#[test_case(MarketStatus::CollectingSubsidy; "collecting subsidy")]
#[test_case(MarketStatus::InsufficientSubsidy; "insufficient subsidy")]
fn admin_move_market_to_closed_fails_if_market_is_not_active(market_status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        let market_id = 0;
        assert_ok!(MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        }));
        assert_noop!(
            PredictionMarkets::admin_move_market_to_closed(Origin::signed(SUDO), market_id),
            Error::<Runtime>::MarketIsNotActive,
        );
    });
}

#[test]
fn admin_move_market_to_closed_correctly_clears_auto_open_and_close_blocks() {
    ExtBuilder::default().build().execute_with(|| {
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Block(22..66),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            0,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Block(33..66),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            0,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Block(22..33),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            0,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));
        assert_ok!(PredictionMarkets::admin_move_market_to_closed(Origin::signed(SUDO), 0));

        let auto_close = MarketIdsPerCloseBlock::<Runtime>::get(66);
        assert_eq!(auto_close.len(), 1);
        assert_eq!(auto_close[0], 1);

        let auto_open = MarketIdsPerOpenBlock::<Runtime>::get(22);
        assert_eq!(auto_open.len(), 1);
        assert_eq!(auto_open[0], 2);
    });
}

#[test_case(654..=321; "empty range")]
#[test_case(555..=555; "one element as range")]
fn create_scalar_market_fails_on_invalid_range(range: RangeInclusive<u128>) {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::Ztg,
                BOB,
                MarketPeriod::Block(123..456),
                get_deadlines(),
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Scalar(range),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM,
            ),
            Error::<Runtime>::InvalidOutcomeRange
        );
    });
}

#[test]
fn create_market_fails_on_min_dispute_period() {
    ExtBuilder::default().build().execute_with(|| {
        let deadlines = Deadlines {
            grace_period: <Runtime as crate::Config>::MaxGracePeriod::get(),
            oracle_duration: <Runtime as crate::Config>::MaxOracleDuration::get(),
            dispute_duration: <Runtime as crate::Config>::MinDisputeDuration::get() - 1,
        };
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::Ztg,
                BOB,
                MarketPeriod::Block(123..456),
                deadlines,
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Categorical(2),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM,
            ),
            Error::<Runtime>::DisputeDurationSmallerThanMinDisputeDuration
        );
    });
}

#[test]
fn create_market_fails_on_min_oracle_duration() {
    ExtBuilder::default().build().execute_with(|| {
        let deadlines = Deadlines {
            grace_period: <Runtime as crate::Config>::MaxGracePeriod::get(),
            oracle_duration: <Runtime as crate::Config>::MinOracleDuration::get() - 1,
            dispute_duration: <Runtime as crate::Config>::MinDisputeDuration::get(),
        };
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::Ztg,
                BOB,
                MarketPeriod::Block(123..456),
                deadlines,
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Categorical(2),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM,
            ),
            Error::<Runtime>::OracleDurationSmallerThanMinOracleDuration
        );
    });
}

#[test]
fn create_market_fails_on_max_dispute_period() {
    ExtBuilder::default().build().execute_with(|| {
        let deadlines = Deadlines {
            grace_period: <Runtime as crate::Config>::MaxGracePeriod::get(),
            oracle_duration: <Runtime as crate::Config>::MaxOracleDuration::get(),
            dispute_duration: <Runtime as crate::Config>::MaxDisputeDuration::get() + 1,
        };
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::Ztg,
                BOB,
                MarketPeriod::Block(123..456),
                deadlines,
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Categorical(2),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM,
            ),
            Error::<Runtime>::DisputeDurationGreaterThanMaxDisputeDuration
        );
    });
}

#[test]
fn create_market_fails_on_max_grace_period() {
    ExtBuilder::default().build().execute_with(|| {
        let deadlines = Deadlines {
            grace_period: <Runtime as crate::Config>::MaxGracePeriod::get() + 1,
            oracle_duration: <Runtime as crate::Config>::MaxOracleDuration::get(),
            dispute_duration: <Runtime as crate::Config>::MaxDisputeDuration::get(),
        };
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::Ztg,
                BOB,
                MarketPeriod::Block(123..456),
                deadlines,
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Categorical(2),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM,
            ),
            Error::<Runtime>::GracePeriodGreaterThanMaxGracePeriod
        );
    });
}

#[test]
fn create_market_fails_on_max_oracle_duration() {
    ExtBuilder::default().build().execute_with(|| {
        let deadlines = Deadlines {
            grace_period: <Runtime as crate::Config>::MaxGracePeriod::get(),
            oracle_duration: <Runtime as crate::Config>::MaxOracleDuration::get() + 1,
            dispute_duration: <Runtime as crate::Config>::MaxDisputeDuration::get(),
        };
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::Ztg,
                BOB,
                MarketPeriod::Block(123..456),
                deadlines,
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Categorical(2),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM,
            ),
            Error::<Runtime>::OracleDurationGreaterThanMaxOracleDuration
        );
    });
}

#[cfg(feature = "parachain")]
#[test]
fn create_market_with_foreign_assets() {
    ExtBuilder::default().build().execute_with(|| {
        let deadlines = Deadlines {
            grace_period: <Runtime as crate::Config>::MaxGracePeriod::get(),
            oracle_duration: <Runtime as crate::Config>::MaxOracleDuration::get(),
            dispute_duration: <Runtime as crate::Config>::MaxDisputeDuration::get(),
        };

        // As per Mock asset_registry genesis ForeignAsset(420) has allow_as_base_asset set to false.

        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::ForeignAsset(420),
                BOB,
                MarketPeriod::Block(123..456),
                deadlines,
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Categorical(2),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM,
            ),
            Error::<Runtime>::InvalidBaseAsset,
        );
        // As per Mock asset_registry genesis ForeignAsset(50) is not registered in asset_registry.
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::ForeignAsset(50),
                BOB,
                MarketPeriod::Block(123..456),
                deadlines,
                gen_metadata(2),
                MarketCreation::Permissionless,
                MarketType::Categorical(2),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM,
            ),
            Error::<Runtime>::UnregisteredForeignAsset,
        );
        // As per Mock asset_registry genesis ForeignAsset(100) has allow_as_base_asset set to true.
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::ForeignAsset(100),
            BOB,
            MarketPeriod::Block(123..456),
            deadlines,
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.base_asset, Asset::ForeignAsset(100));
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_permissionless_market_active() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::CPMM,
        );
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            SENTINEL_AMOUNT
        );
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_permissionless_market_reported() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        let end = 2_u64;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );
        let market = MarketCommons::market(&0).unwrap();
        run_to_block(end + market.deadlines.grace_period);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            SENTINEL_AMOUNT
        );
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_permissionless_market_disputed() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        let end = 2;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        assert_ne!(grace_period, 0);
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_to_block(grace_period + 2);
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            SENTINEL_AMOUNT
        );
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn admin_destroy_market_correctly_unreserves_dispute_bonds() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );
        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        assert_ne!(grace_period, 0);
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_to_block(grace_period + 2);
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(DAVE),
            0,
            OutcomeReport::Categorical(1)
        ));
        let set_up_account = |account| {
            assert_ok!(AssetManager::deposit(Asset::Ztg, account, SENTINEL_AMOUNT));
            assert_ok!(Balances::reserve_named(
                &PredictionMarkets::reserve_id(),
                account,
                SENTINEL_AMOUNT,
            ));
        };
        set_up_account(&CHARLIE);
        set_up_account(&DAVE);

        let balance_free_before_charlie = Balances::free_balance(&CHARLIE);
        let balance_free_before_dave = Balances::free_balance(&DAVE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), market_id));
        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &CHARLIE),
            SENTINEL_AMOUNT,
        );
        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &DAVE),
            SENTINEL_AMOUNT,
        );
        assert_eq!(
            Balances::free_balance(CHARLIE),
            balance_free_before_charlie + default_dispute_bond::<Runtime>(0)
        );
        assert_eq!(
            Balances::free_balance(DAVE),
            balance_free_before_dave + default_dispute_bond::<Runtime>(1),
        );
        assert!(Disputes::<Runtime>::get(market_id).is_empty());
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_permissionless_market_resolved() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        let end = 2;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        assert_eq!(Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE), 0);
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            SENTINEL_AMOUNT
        );
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_advised_market_proposed() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            SENTINEL_AMOUNT
        );
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_advised_market_proposed_with_edit_request() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];

        assert_ok!(PredictionMarkets::request_edit(Origin::signed(SUDO), 0, edit_reason));
        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            SENTINEL_AMOUNT
        );
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
        assert!(!MarketIdsForEdit::<Runtime>::contains_key(0));
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_advised_market_active() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            SENTINEL_AMOUNT
        );
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_advised_market_reported() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        let end = 2;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Advised,
            0..end,
            ScoringRule::CPMM,
        );
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            SENTINEL_AMOUNT
        );
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_advised_market_disputed() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        let end = 2;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Advised,
            0..end,
            ScoringRule::CPMM,
        );
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            SENTINEL_AMOUNT
        );
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn admin_destroy_market_correctly_slashes_advised_market_resolved() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        let end = 2;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Advised,
            0..end,
            ScoringRule::CPMM,
        );
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        assert_eq!(Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE), 0);
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            SENTINEL_AMOUNT
        );
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        assert_eq!(balance_free_before_alice, balance_free_after_alice);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}
#[test]
fn admin_destroy_market_correctly_cleans_up_accounts() {
    let test = |base_asset: Asset<MarketId>| {
        let alice_ztg_before_market_creation = AssetManager::free_balance(Asset::Ztg, &ALICE);
        let alice_base_asset_before_market_creation =
            AssetManager::free_balance(base_asset, &ALICE);
        let swap_fee = <Runtime as zrml_swaps::Config>::MaxSwapFee::get();
        let min_liquidity = <Runtime as zrml_swaps::Config>::MinLiquidity::get();
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            base_asset,
            ALICE,
            MarketPeriod::Block(0..42),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(3),
            MarketDisputeMechanism::SimpleDisputes,
            swap_fee,
            min_liquidity,
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); 3],
        ));
        // Buy some outcome tokens for Alice so that we can check that they get destroyed.
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(ALICE), 0, BASE));
        let market_id = 0;
        let pool_id = 0;
        let pool_account = Swaps::pool_account_id(&pool_id);
        let market_account = MarketCommons::market_account(market_id);
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));
        assert_eq!(AssetManager::free_balance(Asset::CategoricalOutcome(0, 0), &pool_account), 0);
        assert_eq!(AssetManager::free_balance(Asset::CategoricalOutcome(0, 1), &pool_account), 0);
        assert_eq!(AssetManager::free_balance(Asset::CategoricalOutcome(0, 2), &pool_account), 0);
        assert_eq!(AssetManager::free_balance(base_asset, &pool_account), 0);
        assert_eq!(AssetManager::free_balance(Asset::CategoricalOutcome(0, 0), &market_account), 0);
        assert_eq!(AssetManager::free_balance(Asset::CategoricalOutcome(0, 1), &market_account), 0);
        assert_eq!(AssetManager::free_balance(Asset::CategoricalOutcome(0, 2), &market_account), 0);
        assert_eq!(AssetManager::free_balance(base_asset, &market_account), 0);
        // premissionless market so using ValidityBond
        let creation_bond = <Runtime as Config>::ValidityBond::get();
        let oracle_bond = <Runtime as Config>::OracleBond::get();
        // substract min_liquidity twice, one for buy_complete_set() in
        // create_cpmm_market_and_deploy_assets() and one in swaps::create_pool()
        // then again substract BASE as buy_complete_set() above
        let expected_base_asset_value =
            alice_base_asset_before_market_creation - min_liquidity - min_liquidity - BASE;
        if base_asset == Asset::Ztg {
            let alice_base_asset_balance = AssetManager::free_balance(base_asset, &ALICE);
            assert_eq!(
                alice_base_asset_balance,
                expected_base_asset_value - creation_bond - oracle_bond
            );
        } else {
            let alice_base_asset_balance = AssetManager::free_balance(base_asset, &ALICE);
            assert_eq!(alice_base_asset_balance, expected_base_asset_value);
            let alice_ztg_balance = AssetManager::free_balance(Asset::Ztg, &ALICE);
            assert_eq!(
                alice_ztg_balance,
                alice_ztg_before_market_creation - creation_bond - oracle_bond
            );
        }
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn admin_destroy_market_correctly_clears_auto_open_and_close_blocks() {
    ExtBuilder::default().build().execute_with(|| {
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Block(22..66),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            0,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Block(33..66),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            0,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Block(22..33),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            0,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));

        let auto_close = MarketIdsPerCloseBlock::<Runtime>::get(66);
        assert_eq!(auto_close.len(), 1);
        assert_eq!(auto_close[0], 1);

        let auto_open = MarketIdsPerOpenBlock::<Runtime>::get(22);
        assert_eq!(auto_open.len(), 1);
        assert_eq!(auto_open[0], 2);
    });
}

#[test]
fn admin_move_market_to_resolved_resolves_reported_market() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        let end = 33;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );
        let market_id = 0;

        // Give ALICE `SENTINEL_AMOUNT` free and reserved ZTG; we record the free balance to check
        // that the correct bonds are unreserved!
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before = Balances::free_balance(&ALICE);
        let balance_reserved_before =
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE);

        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        let category = 1;
        let outcome_report = OutcomeReport::Categorical(category);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            market_id,
            outcome_report.clone()
        ));
        assert_ok!(PredictionMarkets::admin_move_market_to_resolved(
            Origin::signed(SUDO),
            market_id
        ));

        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Resolved);
        assert_eq!(market.report.unwrap().outcome, outcome_report);
        assert_eq!(market.resolved_outcome.unwrap(), outcome_report);
        System::assert_last_event(
            Event::MarketResolved(market_id, MarketStatus::Resolved, outcome_report).into(),
        );

        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            balance_reserved_before
                - <Runtime as Config>::OracleBond::get()
                - <Runtime as Config>::ValidityBond::get()
        );
        assert_eq!(
            Balances::free_balance(&ALICE),
            balance_free_before
                + <Runtime as Config>::OracleBond::get()
                + <Runtime as Config>::ValidityBond::get()
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test_case(MarketStatus::Active; "Active")]
#[test_case(MarketStatus::Closed; "Closed")]
#[test_case(MarketStatus::CollectingSubsidy; "CollectingSubsidy")]
#[test_case(MarketStatus::InsufficientSubsidy; "InsufficientSubsidy")]
fn admin_move_market_to_resovled_fails_if_market_is_not_reported_or_disputed(
    market_status: MarketStatus,
) {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..33,
            ScoringRule::CPMM,
        );
        let market_id = 0;
        assert_ok!(MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        }));
        assert_noop!(
            PredictionMarkets::admin_move_market_to_resolved(Origin::signed(SUDO), market_id,),
            Error::<Runtime>::InvalidMarketStatus,
        );
    });
}

#[test]
fn it_creates_binary_markets() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::CPMM,
        );

        // check the correct amount was reserved
        let alice_reserved = Balances::reserved_balance(&ALICE);
        assert_eq!(alice_reserved, ValidityBond::get() + OracleBond::get());

        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..2,
            ScoringRule::CPMM,
        );

        let new_alice_reserved = Balances::reserved_balance(&ALICE);
        assert_eq!(new_alice_reserved, AdvisoryBond::get() + OracleBond::get() + alice_reserved);

        // Make sure that the market id has been incrementing
        let market_id = MarketCommons::latest_market_id().unwrap();
        assert_eq!(market_id, 1);
    });
}

#[test]
fn create_categorical_market_deposits_the_correct_event() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            1..2,
            ScoringRule::CPMM,
        );
        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        let market_account = MarketCommons::market_account(market_id);
        System::assert_last_event(Event::MarketCreated(0, market_account, market).into());
    });
}

#[test]
fn create_scalar_market_deposits_the_correct_event() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        simple_create_scalar_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            1..2,
            ScoringRule::CPMM,
        );
        let market_id = 0;
        let market = MarketCommons::market(&market_id).unwrap();
        let market_account = MarketCommons::market_account(market_id);
        System::assert_last_event(Event::MarketCreated(0, market_account, market).into());
    });
}

#[test]
fn it_does_not_create_market_with_too_few_categories() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::Ztg,
                BOB,
                MarketPeriod::Block(0..100),
                get_deadlines(),
                gen_metadata(2),
                MarketCreation::Advised,
                MarketType::Categorical(<Runtime as Config>::MinCategories::get() - 1),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM
            ),
            Error::<Runtime>::NotEnoughCategories
        );
    });
}

#[test]
fn it_does_not_create_market_with_too_many_categories() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::Ztg,
                BOB,
                MarketPeriod::Block(0..100),
                get_deadlines(),
                gen_metadata(2),
                MarketCreation::Advised,
                MarketType::Categorical(<Runtime as Config>::MaxCategories::get() + 1),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM
            ),
            Error::<Runtime>::TooManyCategories
        );
    });
}

#[test]
fn it_allows_sudo_to_destroy_markets() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        // destroy the market
        assert_ok!(PredictionMarkets::admin_destroy_market(Origin::signed(SUDO), 0));

        assert_noop!(
            MarketCommons::market(&0),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn it_allows_advisory_origin_to_approve_markets() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        // Make sure it fails from the random joe
        assert_noop!(
            PredictionMarkets::approve_market(Origin::signed(BOB), 0),
            DispatchError::BadOrigin
        );

        // Now it should work from SUDO
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));

        let after_market = MarketCommons::market(&0);
        assert_eq!(after_market.unwrap().status, MarketStatus::Active);
    });
}

#[test]
fn it_allows_request_edit_origin_to_request_edits_for_markets() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            2..4,
            ScoringRule::CPMM,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];
        // Make sure it fails from the random joe
        assert_noop!(
            PredictionMarkets::request_edit(Origin::signed(BOB), 0, edit_reason.clone()),
            DispatchError::BadOrigin
        );

        // Now it should work from SUDO
        assert_ok!(PredictionMarkets::request_edit(Origin::signed(SUDO), 0, edit_reason.clone()));
        System::assert_last_event(
            Event::MarketRequestedEdit(
                0,
                edit_reason.try_into().expect("Conversion to BoundedVec failed"),
            )
            .into(),
        );

        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));
    });
}

#[test]
fn request_edit_fails_on_bad_origin() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            2..4,
            ScoringRule::CPMM,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];
        // Make sure it fails from the random joe
        assert_noop!(
            PredictionMarkets::request_edit(Origin::signed(BOB), 0, edit_reason),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn edit_request_fails_if_edit_reason_is_too_long() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize + 1];

        assert_noop!(
            PredictionMarkets::request_edit(Origin::signed(SUDO), 0, edit_reason),
            Error::<Runtime>::EditReasonLengthExceedsMaxEditReasonLen
        );
    });
}

#[test]
fn market_with_edit_request_cannot_be_approved() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];

        assert_ok!(PredictionMarkets::request_edit(Origin::signed(SUDO), 0, edit_reason));

        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));
        assert_noop!(
            PredictionMarkets::approve_market(Origin::signed(SUDO), 0),
            Error::<Runtime>::MarketEditRequestAlreadyInProgress
        );
    });
}

#[test]
fn it_allows_the_advisory_origin_to_reject_markets() {
    ExtBuilder::default().build().execute_with(|| {
        run_to_block(2);
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            4..6,
            ScoringRule::CPMM,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let reject_reason: Vec<u8> =
            vec![0; <Runtime as Config>::MaxRejectReasonLen::get() as usize];
        // Now it should work from SUDO
        assert_ok!(PredictionMarkets::reject_market(
            Origin::signed(SUDO),
            0,
            reject_reason.clone()
        ));
        let reject_reason = reject_reason.try_into().expect("BoundedVec conversion failed");
        System::assert_has_event(Event::MarketRejected(0, reject_reason).into());

        assert_noop!(
            MarketCommons::market(&0),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn reject_errors_if_reject_reason_is_too_long() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let reject_reason: Vec<u8> =
            vec![0; <Runtime as Config>::MaxRejectReasonLen::get() as usize + 1];
        assert_noop!(
            PredictionMarkets::reject_market(Origin::signed(SUDO), 0, reject_reason),
            Error::<Runtime>::RejectReasonLengthExceedsMaxRejectReasonLen
        );
    });
}

#[test]
fn it_allows_the_advisory_origin_to_reject_markets_with_edit_request() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];

        let reject_reason = vec![0_u8; <Runtime as Config>::MaxRejectReasonLen::get() as usize];
        assert_ok!(PredictionMarkets::request_edit(Origin::signed(SUDO), 0, edit_reason));
        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));
        assert_ok!(PredictionMarkets::reject_market(Origin::signed(SUDO), 0, reject_reason));
        assert!(!MarketIdsForEdit::<Runtime>::contains_key(0));

        assert_noop!(
            MarketCommons::market(&0),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn reject_market_unreserves_oracle_bond_and_slashes_advisory_bond() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        // Give ALICE `SENTINEL_AMOUNT` free and reserved ZTG; we record the free balance to check
        // that the AdvisoryBond gets slashed but the OracleBond gets unreserved.
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT,
        ));
        assert!(Balances::free_balance(Treasury::account_id()).is_zero());

        let balance_free_before_alice = Balances::free_balance(&ALICE);
        let balance_reserved_before_alice =
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE);

        let reject_reason: Vec<u8> =
            vec![0; <Runtime as Config>::MaxRejectReasonLen::get() as usize];
        assert_ok!(PredictionMarkets::reject_market(Origin::signed(SUDO), 0, reject_reason));

        // AdvisoryBond gets slashed after reject_market
        // OracleBond gets unreserved after reject_market
        let balance_reserved_after_alice =
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE);
        assert_eq!(
            balance_reserved_after_alice,
            balance_reserved_before_alice
                - <Runtime as Config>::OracleBond::get()
                - <Runtime as Config>::AdvisoryBond::get(),
        );
        let balance_free_after_alice = Balances::free_balance(&ALICE);
        let slash_amount_advisory_bond = <Runtime as Config>::AdvisoryBondSlashPercentage::get()
            .mul_floor(<Runtime as Config>::AdvisoryBond::get());
        let advisory_bond_remains =
            <Runtime as Config>::AdvisoryBond::get() - slash_amount_advisory_bond;
        assert_eq!(
            balance_free_after_alice,
            balance_free_before_alice
                + <Runtime as Config>::OracleBond::get()
                + advisory_bond_remains,
        );

        // AdvisoryBond is transferred to the treasury
        let balance_treasury_after = Balances::free_balance(Treasury::account_id());
        assert_eq!(balance_treasury_after, slash_amount_advisory_bond);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn reject_market_clears_auto_close_blocks() {
    // We don't have to check that reject market clears the cache for opening pools, since Cpmm pools
    // can not be deployed on pending advised pools.
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            33..66,
            ScoringRule::CPMM,
        );
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            22..66,
            ScoringRule::CPMM,
        );
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            22..33,
            ScoringRule::CPMM,
        );
        let reject_reason: Vec<u8> =
            vec![0; <Runtime as Config>::MaxRejectReasonLen::get() as usize];
        assert_ok!(PredictionMarkets::reject_market(Origin::signed(SUDO), 0, reject_reason));

        let auto_close = MarketIdsPerCloseBlock::<Runtime>::get(66);
        assert_eq!(auto_close.len(), 1);
        assert_eq!(auto_close[0], 1);
    });
}

#[test]
fn on_market_close_auto_rejects_expired_advised_market() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        // Give ALICE `SENTINEL_AMOUNT` free and reserved ZTG; we record the free balance to check
        // that the AdvisoryBond and the OracleBond gets unreserved, when the advised market expires.
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        let balance_reserved_before_alice =
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE);

        let end = 33;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Advised,
            0..end,
            ScoringRule::CPMM,
        );
        let market_id = 0;

        run_to_block(end);

        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            balance_reserved_before_alice
        );
        assert_eq!(Balances::free_balance(&ALICE), balance_free_before_alice);
        assert_noop!(
            MarketCommons::market(&market_id),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist,
        );
        System::assert_has_event(Event::MarketExpired(market_id).into());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_market_close_auto_rejects_expired_advised_market_with_edit_request() {
    let test = |base_asset: Asset<MarketId>| {
        // Give ALICE `SENTINEL_AMOUNT` free and reserved ZTG; we record the free balance to check
        // that the AdvisoryBond and the OracleBond gets unreserved, when the advised market expires.
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before_alice = Balances::free_balance(&ALICE);
        let balance_reserved_before_alice =
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE);

        let end = 33;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Advised,
            0..end,
            ScoringRule::CPMM,
        );
        run_to_block(2);
        let market_id = 0;
        let market = MarketCommons::market(&market_id);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];

        assert_ok!(PredictionMarkets::request_edit(Origin::signed(SUDO), market_id, edit_reason));

        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));
        run_blocks(end);
        assert!(!MarketIdsForEdit::<Runtime>::contains_key(0));

        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            balance_reserved_before_alice
        );
        assert_eq!(Balances::free_balance(&ALICE), balance_free_before_alice);
        assert_noop!(
            MarketCommons::market(&market_id),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist,
        );
        System::assert_has_event(Event::MarketExpired(market_id).into());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_market_open_successfully_auto_opens_market_pool_with_blocks() {
    ExtBuilder::default().build().execute_with(|| {
        let start = 33;
        let end = 66;
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Block(start..end),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            0,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));
        let market_id = 0;
        let pool_id = MarketCommons::market_pool(&market_id).unwrap();

        run_to_block(start - 1);
        let pool_before_open = Swaps::pool(pool_id).unwrap();
        assert_eq!(pool_before_open.pool_status, PoolStatus::Initialized);

        run_to_block(start);
        let pool_after_open = Swaps::pool(pool_id).unwrap();
        assert_eq!(pool_after_open.pool_status, PoolStatus::Active);
    });
}

#[test]
fn on_market_close_successfully_auto_closes_market_with_blocks() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 33;
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            0,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));
        let market_id = 0;
        let pool_id = MarketCommons::market_pool(&market_id).unwrap();

        run_to_block(end - 1);
        let market_before_close = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market_before_close.status, MarketStatus::Active);
        let pool_before_close = Swaps::pool(pool_id).unwrap();
        assert_eq!(pool_before_close.pool_status, PoolStatus::Active);

        run_to_block(end);
        let market_after_close = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market_after_close.status, MarketStatus::Closed);
        let pool_after_close = Swaps::pool(pool_id).unwrap();
        assert_eq!(pool_after_close.pool_status, PoolStatus::Closed);

        System::assert_last_event(Event::MarketClosed(market_id).into());
    });
}

#[test]
fn on_market_open_successfully_auto_opens_market_with_timestamps() {
    ExtBuilder::default().build().execute_with(|| {
        let start: Moment = (33 * MILLISECS_PER_BLOCK).into();
        let end: Moment = (66 * MILLISECS_PER_BLOCK).into();
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Timestamp(start..end),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            0,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));
        let market_id = 0;
        let pool_id = MarketCommons::market_pool(&market_id).unwrap();

        // (Check that the market doesn't close too soon)
        set_timestamp_for_on_initialize(start - 1);
        run_blocks(1); // Trigger hook!
        let pool_before_close = Swaps::pool(pool_id).unwrap();
        assert_eq!(pool_before_close.pool_status, PoolStatus::Initialized);

        set_timestamp_for_on_initialize(start);
        run_blocks(1); // Trigger hook!
        let pool_after_close = Swaps::pool(pool_id).unwrap();
        assert_eq!(pool_after_close.pool_status, PoolStatus::Active);
    });
}

#[test]
fn on_market_close_successfully_auto_closes_market_with_timestamps() {
    ExtBuilder::default().build().execute_with(|| {
        let end: Moment = (2 * MILLISECS_PER_BLOCK).into();
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Timestamp(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            0,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));
        let market_id = 0;
        let pool_id = MarketCommons::market_pool(&market_id).unwrap();

        // (Check that the market doesn't close too soon)
        set_timestamp_for_on_initialize(end - 1);
        run_to_block(2); // Trigger `on_initialize`; must be at least block #2!
        let market_before_close = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market_before_close.status, MarketStatus::Active);
        let pool_before_close = Swaps::pool(pool_id).unwrap();
        assert_eq!(pool_before_close.pool_status, PoolStatus::Active);

        set_timestamp_for_on_initialize(end);
        run_blocks(1);
        let market_after_close = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market_after_close.status, MarketStatus::Closed);
        let pool_after_close = Swaps::pool(pool_id).unwrap();
        assert_eq!(pool_after_close.pool_status, PoolStatus::Closed);

        System::assert_last_event(Event::MarketClosed(market_id).into());
    });
}

#[test]
fn on_market_open_successfully_auto_opens_multiple_markets_after_stall() {
    // We check that `on_market_open` works correctly even if a block takes much longer than 12sec
    // to be produced and multiple markets are involved.
    ExtBuilder::default().build().execute_with(|| {
        // Mock last time frame to prevent it from defaulting.
        LastTimeFrame::<Runtime>::set(Some(0));

        let start: Moment = (33 * MILLISECS_PER_BLOCK).into();
        let end: Moment = (666 * MILLISECS_PER_BLOCK).into();
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Timestamp(start..end),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            0,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Timestamp(start..end),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            0,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));

        // This block takes much longer than 12sec, but markets and pools still close correctly.
        set_timestamp_for_on_initialize(end / 2);
        run_to_block(2); // Trigger `on_initialize`; must be at least block #2!
        assert_eq!(Swaps::pool(0).unwrap().pool_status, PoolStatus::Active);
        assert_eq!(Swaps::pool(1).unwrap().pool_status, PoolStatus::Active);
    });
}

#[test]
fn on_market_close_successfully_auto_closes_multiple_markets_after_stall() {
    // We check that `on_market_close` works correctly even if a block takes much longer than 12sec
    // to be produced and multiple markets are involved.
    ExtBuilder::default().build().execute_with(|| {
        // Mock last time frame to prevent it from defaulting.
        LastTimeFrame::<Runtime>::set(Some(0));

        let end: Moment = (5 * MILLISECS_PER_BLOCK).into();
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Timestamp(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            0,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Timestamp(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            0,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));

        // This block takes much longer than 12sec, but markets and pools still close correctly.
        set_timestamp_for_on_initialize(10 * end);
        run_to_block(2); // Trigger `on_initialize`; must be at least block #2!

        let market_after_close = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after_close.status, MarketStatus::Closed);
        let pool_after_close = Swaps::pool(0).unwrap();
        assert_eq!(pool_after_close.pool_status, PoolStatus::Closed);
        System::assert_has_event(Event::MarketClosed(0).into());

        let market_after_close = MarketCommons::market(&1).unwrap();
        assert_eq!(market_after_close.status, MarketStatus::Closed);
        let pool_after_close = Swaps::pool(1).unwrap();
        assert_eq!(pool_after_close.pool_status, PoolStatus::Closed);
        System::assert_has_event(Event::MarketClosed(1).into());
    });
}

#[test]
fn on_initialize_skips_the_genesis_block() {
    // We ensure that a timestamp of zero will not be stored at genesis into LastTimeFrame storage.
    let blocks = 5;
    let end: Moment = (blocks * MILLISECS_PER_BLOCK).into();
    ExtBuilder::default().build().execute_with(|| {
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Timestamp(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            123,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); category_count.into()],
        ));

        // Blocknumber = 0
        assert_eq!(Timestamp::get(), 0);
        PredictionMarkets::on_initialize(0);
        assert_eq!(LastTimeFrame::<Runtime>::get(), None);

        // Blocknumber = 1
        assert_eq!(Timestamp::get(), 0);
        PredictionMarkets::on_initialize(1);
        assert_eq!(LastTimeFrame::<Runtime>::get(), None);

        // Blocknumer != 0, 1
        set_timestamp_for_on_initialize(end);
        PredictionMarkets::on_initialize(2);
        assert_eq!(LastTimeFrame::<Runtime>::get(), Some(blocks.into()));
    });
}

#[test]
fn it_allows_to_buy_a_complete_set() {
    let test = |base_asset: Asset<MarketId>| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        // Creates a permissionless market.
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::CPMM,
        );

        // Allows someone to generate a complete set
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, CENT));

        let market = MarketCommons::market(&0).unwrap();

        // Check the outcome balances
        let assets = PredictionMarkets::outcome_assets(0, &market);
        for asset in assets.iter() {
            let bal = Tokens::free_balance(*asset, &BOB);
            assert_eq!(bal, CENT);
        }

        let market_account = MarketCommons::market_account(0);
        let bal = AssetManager::free_balance(base_asset, &BOB);
        assert_eq!(bal, 1_000 * BASE - CENT);

        let market_bal = AssetManager::free_balance(base_asset, &market_account);
        assert_eq!(market_bal, CENT);
        System::assert_last_event(Event::BoughtCompleteSet(0, CENT, BOB).into());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn it_does_not_allow_to_buy_a_complete_set_on_pending_advised_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );
        assert_noop!(
            PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, CENT),
            Error::<Runtime>::MarketIsNotActive,
        );
    });
}

#[test]
fn create_categorical_market_fails_if_market_begin_is_equal_to_end() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::Ztg,
                BOB,
                MarketPeriod::Block(3..3),
                get_deadlines(),
                gen_metadata(0),
                MarketCreation::Permissionless,
                MarketType::Categorical(3),
                MarketDisputeMechanism::Authorized,
                ScoringRule::CPMM,
            ),
            Error::<Runtime>::InvalidMarketPeriod,
        );
    });
}

#[test_case(MarketPeriod::Block(2..1); "block start greater than end")]
#[test_case(MarketPeriod::Block(3..3); "block start equal to end")]
#[test_case(MarketPeriod::Timestamp(2..1); "timestamp start greater than end")]
#[test_case(MarketPeriod::Timestamp(3..3); "timestamp start equal to end")]
#[test_case(
    MarketPeriod::Timestamp(0..(MILLISECS_PER_BLOCK - 1).into());
    "range shorter than block time"
)]
fn create_categorical_market_fails_if_market_period_is_invalid(
    period: MarketPeriod<BlockNumber, Moment>,
) {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::Ztg,
                BOB,
                period,
                get_deadlines(),
                gen_metadata(0),
                MarketCreation::Permissionless,
                MarketType::Categorical(3),
                MarketDisputeMechanism::Authorized,
                ScoringRule::CPMM,
            ),
            Error::<Runtime>::InvalidMarketPeriod,
        );
    });
}

#[test]
fn create_categorical_market_fails_if_end_is_not_far_enough_ahead() {
    ExtBuilder::default().build().execute_with(|| {
        let end_block = 33;
        run_to_block(end_block);
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::Ztg,
                BOB,
                MarketPeriod::Block(0..end_block),
                get_deadlines(),
                gen_metadata(0),
                MarketCreation::Permissionless,
                MarketType::Categorical(3),
                MarketDisputeMechanism::Authorized,
                ScoringRule::CPMM,
            ),
            Error::<Runtime>::InvalidMarketPeriod,
        );

        let end_time = MILLISECS_PER_BLOCK as u64 / 2;
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::Ztg,
                BOB,
                MarketPeriod::Timestamp(0..end_time),
                get_deadlines(),
                gen_metadata(0),
                MarketCreation::Permissionless,
                MarketType::Categorical(3),
                MarketDisputeMechanism::Authorized,
                ScoringRule::CPMM,
            ),
            Error::<Runtime>::InvalidMarketPeriod,
        );
    });
}

#[test]
fn it_does_not_allow_zero_amounts_in_buy_complete_set() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        assert_noop!(
            PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, 0),
            Error::<Runtime>::ZeroAmount
        );
    });
}

#[test]
fn it_does_not_allow_buying_complete_sets_with_insufficient_balance() {
    let test = |base_asset: Asset<MarketId>| {
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        assert_noop!(
            PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, 10000 * BASE),
            Error::<Runtime>::NotEnoughBalance
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}
#[test]
fn it_allows_to_deploy_a_pool() {
    let test = |base_asset: Asset<MarketId>| {
        // Creates a permissionless market.
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, 100 * BASE));

        assert_ok!(Balances::transfer(
            Origin::signed(BOB),
            <Runtime as crate::Config>::PalletId::get().into_account_truncating(),
            100 * BASE
        ));

        assert_ok!(PredictionMarkets::deploy_swap_pool_for_market(
            Origin::signed(BOB),
            0,
            <Runtime as zrml_swaps::Config>::MaxSwapFee::get(),
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); 2],
        ));
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn deploy_swap_pool_for_market_fails_if_market_has_a_pool() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, 200 * BASE));
        assert_ok!(PredictionMarkets::deploy_swap_pool_for_market(
            Origin::signed(BOB),
            0,
            <Runtime as zrml_swaps::Config>::MaxSwapFee::get(),
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); 2],
        ));
        assert_noop!(
            PredictionMarkets::deploy_swap_pool_for_market(
                Origin::signed(BOB),
                0,
                <Runtime as zrml_swaps::Config>::MaxSwapFee::get(),
                <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
                vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); 2],
            ),
            zrml_market_commons::Error::<Runtime>::PoolAlreadyExists,
        );
    });
}

#[test]
fn it_does_not_allow_to_deploy_a_pool_on_pending_advised_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        assert_noop!(
            PredictionMarkets::deploy_swap_pool_for_market(
                Origin::signed(BOB),
                0,
                <Runtime as zrml_swaps::Config>::MaxSwapFee::get(),
                <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
                vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); 2],
            ),
            Error::<Runtime>::MarketIsNotActive,
        );
    });
}

#[test]
fn it_allows_to_sell_a_complete_set() {
    let test = |base_asset: Asset<MarketId>| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        // Creates a permissionless market.
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::CPMM,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, CENT));

        assert_ok!(PredictionMarkets::sell_complete_set(Origin::signed(BOB), 0, CENT));

        let market = MarketCommons::market(&0).unwrap();

        // Check the outcome balances
        let assets = PredictionMarkets::outcome_assets(0, &market);
        for asset in assets.iter() {
            let bal = Tokens::free_balance(*asset, &BOB);
            assert_eq!(bal, 0);
        }

        // also check native balance
        let bal = Balances::free_balance(&BOB);
        assert_eq!(bal, 1_000 * BASE);

        System::assert_last_event(Event::SoldCompleteSet(0, CENT, BOB).into());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn it_does_not_allow_zero_amounts_in_sell_complete_set() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        assert_noop!(
            PredictionMarkets::sell_complete_set(Origin::signed(BOB), 0, 0),
            Error::<Runtime>::ZeroAmount
        );
    });
}

#[test]
fn it_does_not_allow_to_sell_complete_sets_with_insufficient_balance() {
    let test = |base_asset: Asset<MarketId>| {
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, 2 * CENT));
        assert_eq!(AssetManager::slash(Asset::CategoricalOutcome(0, 1), &BOB, CENT), 0);
        assert_noop!(
            PredictionMarkets::sell_complete_set(Origin::signed(BOB), 0, 2 * CENT),
            Error::<Runtime>::InsufficientShareBalance
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn it_allows_to_report_the_outcome_of_a_market() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );

        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        assert!(market.report.is_none());

        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market_after = MarketCommons::market(&0).unwrap();
        let report = market_after.report.unwrap();
        assert_eq!(market_after.status, MarketStatus::Reported);
        assert_eq!(report.outcome, OutcomeReport::Categorical(1));
        assert_eq!(report.by, market_after.oracle);

        // Reset and report again as approval origin
        let _ = MarketCommons::mutate_market(&0, |market| {
            market.status = MarketStatus::Closed;
            market.report = None;
            Ok(())
        });

        assert_ok!(PredictionMarkets::report(
            Origin::signed(SUDO),
            0,
            OutcomeReport::Categorical(1)
        ));
    });
}

#[test]
fn report_fails_before_grace_period_is_over() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );

        run_to_block(end);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        assert!(market.report.is_none());

        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::NotAllowedToReportYet
        );
    });
}

#[test]
fn it_allows_only_oracle_to_report_the_outcome_of_a_market_during_oracle_duration_blocks() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );

        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        assert!(market.report.is_none());

        assert_noop!(
            PredictionMarkets::report(Origin::signed(CHARLIE), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::ReporterNotOracle
        );

        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market_after = MarketCommons::market(&0).unwrap();
        let report = market_after.report.unwrap();
        assert_eq!(market_after.status, MarketStatus::Reported);
        assert_eq!(report.outcome, OutcomeReport::Categorical(1));
        assert_eq!(report.by, market_after.oracle);
    });
}

#[test]
fn it_allows_only_oracle_to_report_the_outcome_of_a_market_during_oracle_duration_moment() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, CENT));

        // set the timestamp
        let market = MarketCommons::market(&0).unwrap();
        // set the timestamp

        set_timestamp_for_on_initialize(100_000_000);
        run_to_block(2); // Trigger `on_initialize`; must be at least block #2.
        let grace_period: u64 = market.deadlines.grace_period * MILLISECS_PER_BLOCK as u64;
        Timestamp::set_timestamp(100_000_000 + grace_period);

        assert_noop!(
            PredictionMarkets::report(Origin::signed(EVE), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::ReporterNotOracle
        );
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
    });
}

#[test]
fn report_fails_on_mismatched_outcome_for_categorical_market() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Scalar(123)),
            Error::<Runtime>::OutcomeMismatch,
        );
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        assert!(market.report.is_none());
    });
}

#[test]
fn report_fails_on_out_of_range_outcome_for_categorical_market() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(2)),
            Error::<Runtime>::OutcomeMismatch,
        );
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        assert!(market.report.is_none());
    });
}

#[test]
fn report_fails_on_mismatched_outcome_for_scalar_market() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_scalar_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(0)),
            Error::<Runtime>::OutcomeMismatch,
        );
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        assert!(market.report.is_none());
    });
}

#[test]
fn it_allows_to_dispute_the_outcome_of_a_market() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );

        // Run to the end of the trading phase.
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let dispute_at = grace_period + 2;
        run_to_block(dispute_at);

        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        let disputes = crate::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 1);
        let dispute = &disputes[0];
        assert_eq!(dispute.at, dispute_at);
        assert_eq!(dispute.by, CHARLIE);
        assert_eq!(dispute.outcome, OutcomeReport::Categorical(0));

        let dispute_ends_at = dispute_at + market.deadlines.dispute_duration;
        let market_ids = MarketIdsPerDisputeBlock::<Runtime>::get(dispute_ends_at);
        assert_eq!(market_ids.len(), 1);
        assert_eq!(market_ids[0], 0);
    });
}

#[test]
fn dispute_fails_authority_reported_already() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            MarketDisputeMechanism::Authorized,
            ScoringRule::CPMM,
        ));

        // Run to the end of the trading phase.
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let dispute_at = grace_period + 2;
        run_to_block(dispute_at);

        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));

        assert_noop!(
            PredictionMarkets::dispute(Origin::signed(CHARLIE), 0, OutcomeReport::Categorical(1)),
            AuthorizedError::<Runtime>::OnlyOneDisputeAllowed
        );
    });
}

#[test]
fn it_allows_anyone_to_report_an_unreported_market() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );

        let market = MarketCommons::market(&0).unwrap();
        // Just skip to waaaay overdue.
        run_to_block(end + market.deadlines.grace_period + market.deadlines.oracle_duration + 1);

        assert_ok!(PredictionMarkets::report(
            Origin::signed(ALICE), // alice reports her own market now
            0,
            OutcomeReport::Categorical(1),
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Reported);
        assert_eq!(market.report.unwrap().by, ALICE);
        // but oracle was bob
        assert_eq!(market.oracle, BOB);

        // make sure it still resolves
        run_to_block(
            frame_system::Pallet::<Runtime>::block_number() + market.deadlines.dispute_duration,
        );

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);
    });
}

#[test]
fn it_correctly_resolves_a_market_that_was_reported_on() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, CENT));

        let market = MarketCommons::market(&0).unwrap();
        let report_at = end + market.deadlines.grace_period + 1;
        run_to_block(report_at);

        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let reported_ids =
            MarketIdsPerReportBlock::<Runtime>::get(report_at + market.deadlines.dispute_duration);
        assert_eq!(reported_ids.len(), 1);
        let id = reported_ids[0];
        assert_eq!(id, 0);

        run_blocks(market.deadlines.dispute_duration);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Resolved);

        // Check balance of winning outcome asset.
        let share_b = Asset::CategoricalOutcome(0, 1);
        let share_b_total = Tokens::total_issuance(share_b);
        assert_eq!(share_b_total, CENT);
        let share_b_bal = Tokens::free_balance(share_b, &CHARLIE);
        assert_eq!(share_b_bal, CENT);

        // TODO(#792): Remove other assets.
        let share_a = Asset::CategoricalOutcome(0, 0);
        let share_a_total = Tokens::total_issuance(share_a);
        assert_eq!(share_a_total, CENT);
        let share_a_bal = Tokens::free_balance(share_a, &CHARLIE);
        assert_eq!(share_a_bal, CENT);

        let share_c = Asset::CategoricalOutcome(0, 2);
        let share_c_total = Tokens::total_issuance(share_c);
        assert_eq!(share_c_total, 0);
        let share_c_bal = Tokens::free_balance(share_c, &CHARLIE);
        assert_eq!(share_c_bal, 0);

        assert!(market.bonds.creation.unwrap().is_settled);
        assert!(market.bonds.oracle.unwrap().is_settled);
    });
}

#[test]
fn it_resolves_a_disputed_market() {
    let test = |base_asset: Asset<MarketId>| {
        let end = 2;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, CENT));
        let market = MarketCommons::market(&0).unwrap();

        let report_at = end + market.deadlines.grace_period + 1;
        run_to_block(report_at);

        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));

        let dispute_at_0 = report_at + 1;
        run_to_block(dispute_at_0);

        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));

        let dispute_at_1 = report_at + 2;
        run_to_block(dispute_at_1);

        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(DAVE),
            0,
            OutcomeReport::Categorical(0)
        ));

        let dispute_at_2 = report_at + 3;
        run_to_block(dispute_at_2);

        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        // check everyone's deposits
        let charlie_reserved = Balances::reserved_balance(&CHARLIE);
        assert_eq!(charlie_reserved, DisputeBond::get());

        let dave_reserved = Balances::reserved_balance(&DAVE);
        assert_eq!(dave_reserved, DisputeBond::get() + DisputeFactor::get());

        let eve_reserved = Balances::reserved_balance(&EVE);
        assert_eq!(eve_reserved, DisputeBond::get() + 2 * DisputeFactor::get());

        // check disputes length
        let disputes = crate::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 3);

        // make sure the old mappings of market id per dispute block are erased
        let market_ids_1 = MarketIdsPerDisputeBlock::<Runtime>::get(
            dispute_at_0 + market.deadlines.dispute_duration,
        );
        assert_eq!(market_ids_1.len(), 0);

        let market_ids_2 = MarketIdsPerDisputeBlock::<Runtime>::get(
            dispute_at_1 + market.deadlines.dispute_duration,
        );
        assert_eq!(market_ids_2.len(), 0);

        let market_ids_3 = MarketIdsPerDisputeBlock::<Runtime>::get(
            dispute_at_2 + market.deadlines.dispute_duration,
        );
        assert_eq!(market_ids_3.len(), 1);

        run_blocks(market.deadlines.dispute_duration);

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);
        let disputes = crate::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 0);

        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(CHARLIE), 0));

        // Make sure rewards are right:
        //
        // Slashed amounts:
        //     - Dave's reserve: DisputeBond::get() + DisputeFactor::get()
        //     - Alice's oracle bond: OracleBond::get()
        // Total: OracleBond::get() + DisputeBond::get() + DisputeFactor::get()
        //
        // Charlie and Eve each receive half of the total slashed amount as bounty.
        let dave_reserved = DisputeBond::get() + DisputeFactor::get();
        let total_slashed = OracleBond::get() + dave_reserved;

        let charlie_balance = Balances::free_balance(&CHARLIE);
        assert_eq!(charlie_balance, 1_000 * BASE + total_slashed / 2);
        let charlie_reserved_2 = Balances::reserved_balance(&CHARLIE);
        assert_eq!(charlie_reserved_2, 0);
        let eve_balance = Balances::free_balance(&EVE);
        assert_eq!(eve_balance, 1_000 * BASE + total_slashed / 2);

        let dave_balance = Balances::free_balance(&DAVE);
        assert_eq!(dave_balance, 1_000 * BASE - dave_reserved);

        let alice_balance = Balances::free_balance(&ALICE);
        assert_eq!(alice_balance, 1_000 * BASE - OracleBond::get());

        // bob kinda gets away scot-free since Alice is held responsible
        // for her designated reporter
        let bob_balance = Balances::free_balance(&BOB);
        assert_eq!(bob_balance, 1_000 * BASE);

        assert!(market_after.bonds.creation.unwrap().is_settled);
        assert!(market_after.bonds.oracle.unwrap().is_settled);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test_case(MarketStatus::Active; "active")]
#[test_case(MarketStatus::CollectingSubsidy; "collecting_subsidy")]
#[test_case(MarketStatus::InsufficientSubsidy; "insufficient_subsidy")]
#[test_case(MarketStatus::Closed; "closed")]
#[test_case(MarketStatus::Proposed; "proposed")]
#[test_case(MarketStatus::Resolved; "resolved")]
fn dispute_fails_unless_reported_or_disputed_market(status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );

        assert_ok!(MarketCommons::mutate_market(&0, |market_inner| {
            market_inner.status = status;
            Ok(())
        }));

        assert_noop!(
            PredictionMarkets::dispute(Origin::signed(EVE), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::InvalidMarketStatus
        );
    });
}

#[test]
fn start_global_dispute_works() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Block(0..2),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MaxDisputes::get() + 1),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        let market_id = MarketCommons::latest_market_id().unwrap();

        let market = MarketCommons::market(&market_id).unwrap();
        let grace_period = market.deadlines.grace_period;
        run_to_block(end + grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            market_id,
            OutcomeReport::Categorical(0)
        ));
        let dispute_at_0 = end + grace_period + 2;
        run_to_block(dispute_at_0);
        for i in 1..=<Runtime as Config>::MaxDisputes::get() {
            if i == 1 {
                #[cfg(feature = "with-global-disputes")]
                assert_noop!(
                    PredictionMarkets::start_global_dispute(Origin::signed(CHARLIE), market_id),
                    Error::<Runtime>::InvalidMarketStatus
                );
            } else {
                #[cfg(feature = "with-global-disputes")]
                assert_noop!(
                    PredictionMarkets::start_global_dispute(Origin::signed(CHARLIE), market_id),
                    Error::<Runtime>::MaxDisputesNeeded
                );
            }
            assert_ok!(PredictionMarkets::dispute(
                Origin::signed(CHARLIE),
                market_id,
                OutcomeReport::Categorical(i.saturated_into())
            ));
            run_blocks(1);
            let market = MarketCommons::market(&market_id).unwrap();
            assert_eq!(market.status, MarketStatus::Disputed);
        }

        let disputes = crate::Disputes::<Runtime>::get(market_id);
        assert_eq!(disputes.len(), <Runtime as Config>::MaxDisputes::get() as usize);

        let last_dispute = disputes.last().unwrap();
        let dispute_block = last_dispute.at.saturating_add(market.deadlines.dispute_duration);
        let removable_market_ids = MarketIdsPerDisputeBlock::<Runtime>::get(dispute_block);
        assert_eq!(removable_market_ids.len(), 1);

        #[cfg(feature = "with-global-disputes")]
        {
            use zrml_global_disputes::GlobalDisputesPalletApi;

            let now = <frame_system::Pallet<Runtime>>::block_number();
            assert_ok!(PredictionMarkets::start_global_dispute(Origin::signed(CHARLIE), market_id));

            // report check
            assert_eq!(
                GlobalDisputes::get_voting_outcome_info(&market_id, &OutcomeReport::Categorical(0)),
                Some((Zero::zero(), vec![BOB])),
            );
            for i in 1..=<Runtime as Config>::MaxDisputes::get() {
                let dispute_bond = crate::default_dispute_bond::<Runtime>((i - 1).into());
                assert_eq!(
                    GlobalDisputes::get_voting_outcome_info(
                        &market_id,
                        &OutcomeReport::Categorical(i.saturated_into())
                    ),
                    Some((dispute_bond, vec![CHARLIE])),
                );
            }

            // remove_last_dispute_from_market_ids_per_dispute_block works
            let removable_market_ids = MarketIdsPerDisputeBlock::<Runtime>::get(dispute_block);
            assert_eq!(removable_market_ids.len(), 0);

            let market_ids = MarketIdsPerDisputeBlock::<Runtime>::get(
                now + <Runtime as Config>::GlobalDisputePeriod::get(),
            );
            assert_eq!(market_ids, vec![market_id]);
            assert!(GlobalDisputes::is_started(&market_id));
            System::assert_last_event(Event::GlobalDisputeStarted(market_id).into());

            assert_noop!(
                PredictionMarkets::start_global_dispute(Origin::signed(CHARLIE), market_id),
                Error::<Runtime>::GlobalDisputeAlreadyStarted
            );
        }
    });
}

#[test]
fn start_global_dispute_fails_on_wrong_mdm() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Block(0..2),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MaxDisputes::get() + 1),
            MarketDisputeMechanism::Authorized,
            ScoringRule::CPMM,
        ));
        let market_id = MarketCommons::latest_market_id().unwrap();

        let market = MarketCommons::market(&market_id).unwrap();
        let grace_period = market.deadlines.grace_period;
        run_to_block(end + grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            market_id,
            OutcomeReport::Categorical(0)
        ));
        let dispute_at_0 = end + grace_period + 2;
        run_to_block(dispute_at_0);

        // only one dispute allowed for authorized mdm
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            market_id,
            OutcomeReport::Categorical(1u32.saturated_into())
        ));
        run_blocks(1);
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        #[cfg(feature = "with-global-disputes")]
        assert_noop!(
            PredictionMarkets::start_global_dispute(Origin::signed(CHARLIE), market_id),
            Error::<Runtime>::InvalidDisputeMechanism
        );
    });
}

#[test]
fn start_global_dispute_works_without_feature() {
    ExtBuilder::default().build().execute_with(|| {
        let non_market_id = 0;

        #[cfg(not(feature = "with-global-disputes"))]
        assert_noop!(
            PredictionMarkets::start_global_dispute(Origin::signed(CHARLIE), non_market_id),
            Error::<Runtime>::GlobalDisputesDisabled
        );

        #[cfg(feature = "with-global-disputes")]
        assert_noop!(
            PredictionMarkets::start_global_dispute(Origin::signed(CHARLIE), non_market_id),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn it_allows_to_redeem_shares() {
    let test = |base_asset: Asset<MarketId>| {
        let end = 2;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::CPMM,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, CENT));
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Resolved);

        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(CHARLIE), 0));
        let bal = Balances::free_balance(&CHARLIE);
        assert_eq!(bal, 1_000 * BASE);
        System::assert_last_event(
            Event::TokensRedeemed(0, Asset::CategoricalOutcome(0, 1), CENT, CENT, CHARLIE).into(),
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn create_market_and_deploy_assets_results_in_expected_balances_and_pool_params() {
    let test = |base_asset: Asset<MarketId>| {
        let oracle = ALICE;
        let period = MarketPeriod::Block(0..42);
        let metadata = gen_metadata(42);
        let category_count = 4;
        let market_type = MarketType::Categorical(category_count);
        let swap_fee = <Runtime as zrml_swaps::Config>::MaxSwapFee::get();
        let amount = 123 * BASE;
        let pool_id = 0;
        let weight = <Runtime as zrml_swaps::Config>::MinWeight::get();
        let weights = vec![weight; category_count.into()];
        let base_asset_weight = (category_count as u128) * weight;
        let total_weight = 2 * base_asset_weight;
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            base_asset,
            oracle,
            period,
            get_deadlines(),
            metadata,
            market_type,
            MarketDisputeMechanism::SimpleDisputes,
            swap_fee,
            amount,
            weights,
        ));
        let market_id = 0;

        let pool_account = Swaps::pool_account_id(&pool_id);
        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 0), &ALICE), 0);
        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 1), &ALICE), 0);
        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 2), &ALICE), 0);
        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 3), &ALICE), 0);

        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 0), &pool_account), amount);
        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 1), &pool_account), amount);
        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 2), &pool_account), amount);
        assert_eq!(Tokens::free_balance(Asset::CategoricalOutcome(0, 3), &pool_account), amount);
        assert_eq!(AssetManager::free_balance(base_asset, &pool_account), amount);

        let pool = Pools::<Runtime>::get(0).unwrap();
        let assets_expected = vec![
            Asset::CategoricalOutcome(market_id, 0),
            Asset::CategoricalOutcome(market_id, 1),
            Asset::CategoricalOutcome(market_id, 2),
            Asset::CategoricalOutcome(market_id, 3),
            base_asset,
        ];
        assert_eq!(pool.assets, assets_expected);
        assert_eq!(pool.base_asset, base_asset);
        assert_eq!(pool.market_id, market_id);
        assert_eq!(pool.scoring_rule, ScoringRule::CPMM);
        assert_eq!(pool.swap_fee, Some(swap_fee));
        assert_eq!(pool.total_subsidy, None);
        assert_eq!(pool.total_subsidy, None);
        assert_eq!(pool.total_weight, Some(total_weight));
        let pool_weights = pool.weights.unwrap();
        assert_eq!(pool_weights[&Asset::CategoricalOutcome(market_id, 0)], weight);
        assert_eq!(pool_weights[&Asset::CategoricalOutcome(market_id, 1)], weight);
        assert_eq!(pool_weights[&Asset::CategoricalOutcome(market_id, 2)], weight);
        assert_eq!(pool_weights[&Asset::CategoricalOutcome(market_id, 3)], weight);
        assert_eq!(pool_weights[&base_asset], base_asset_weight);
    };

    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn process_subsidy_activates_market_with_sufficient_subsidy() {
    let test = |base_asset: Asset<MarketId>| {
        let min_sub_period =
            <Runtime as crate::Config>::MinSubsidyPeriod::get() / (MILLISECS_PER_BLOCK as u64);
        let max_sub_period =
            <Runtime as crate::Config>::MaxSubsidyPeriod::get() / (MILLISECS_PER_BLOCK as u64);

        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            min_sub_period..max_sub_period,
            ScoringRule::RikiddoSigmoidFeeMarketEma,
        );
        let min_subsidy = <Runtime as zrml_swaps::Config>::MinSubsidy::get();
        assert_ok!(Swaps::pool_join_subsidy(Origin::signed(ALICE), 0, min_subsidy));
        run_to_block(min_sub_period);
        let subsidy_queue = crate::MarketsCollectingSubsidy::<Runtime>::get();
        assert_eq!(subsidy_queue.len(), 0);
        assert_eq!(MarketCommons::market(&0).unwrap().status, MarketStatus::Active);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn process_subsidy_blocks_market_with_insufficient_subsidy() {
    let test = |base_asset: Asset<MarketId>| {
        let min_sub_period =
            <Runtime as crate::Config>::MinSubsidyPeriod::get() / (MILLISECS_PER_BLOCK as u64);
        let max_sub_period =
            <Runtime as crate::Config>::MaxSubsidyPeriod::get() / (MILLISECS_PER_BLOCK as u64);

        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            min_sub_period..max_sub_period,
            ScoringRule::RikiddoSigmoidFeeMarketEma,
        );
        let subsidy = <Runtime as zrml_swaps::Config>::MinSubsidy::get() / 3;
        assert_ok!(Swaps::pool_join_subsidy(Origin::signed(ALICE), 0, subsidy));
        assert_ok!(Swaps::pool_join_subsidy(Origin::signed(BOB), 0, subsidy));
        run_to_block(min_sub_period);
        let subsidy_queue = crate::MarketsCollectingSubsidy::<Runtime>::get();
        assert_eq!(subsidy_queue.len(), 0);
        assert_eq!(MarketCommons::market(&0).unwrap().status, MarketStatus::InsufficientSubsidy);

        // Check that the balances are correctly unreserved.
        assert_eq!(AssetManager::reserved_balance(base_asset, &ALICE), 0);
        assert_eq!(AssetManager::reserved_balance(base_asset, &BOB), 0);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn process_subsidy_keeps_market_in_subsidy_queue_until_end_of_subsidy_phase() {
    let test = |base_asset: Asset<MarketId>| {
        let min_sub_period =
            <Runtime as crate::Config>::MinSubsidyPeriod::get() / (MILLISECS_PER_BLOCK as u64);
        let max_sub_period =
            <Runtime as crate::Config>::MaxSubsidyPeriod::get() / (MILLISECS_PER_BLOCK as u64);

        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            min_sub_period + 42..max_sub_period,
            ScoringRule::RikiddoSigmoidFeeMarketEma,
        );

        // Run to block where 2 markets are ready and process all markets.
        run_to_block(min_sub_period);
        let subsidy_queue = crate::MarketsCollectingSubsidy::<Runtime>::get();
        assert!(subsidy_queue.len() == 1);
        assert!(subsidy_queue[0].market_id == 0);
        assert!(MarketCommons::market(&0).unwrap().status == MarketStatus::CollectingSubsidy);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn start_subsidy_creates_pool_and_starts_subsidy() {
    let test = |base_asset: Asset<MarketId>| {
        // Create advised categorical market using Rikiddo.
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Advised,
            1337..1338,
            ScoringRule::RikiddoSigmoidFeeMarketEma,
        );
        let market_id = 0;
        let mut market = MarketCommons::market(&market_id).unwrap();

        // Ensure and set correct market status.
        assert_err!(
            PredictionMarkets::start_subsidy(&market, market_id),
            Error::<Runtime>::MarketIsNotCollectingSubsidy
        );
        assert_ok!(MarketCommons::mutate_market(&market_id, |market_inner| {
            market_inner.status = MarketStatus::CollectingSubsidy;
            market = market_inner.clone();
            Ok(())
        }));

        // Pool was created and market was registered for state transition into active.
        assert_ok!(PredictionMarkets::start_subsidy(&market, market_id));
        assert_ok!(MarketCommons::market_pool(&market_id));
        let mut inserted = false;

        for market in crate::MarketsCollectingSubsidy::<Runtime>::get() {
            if market.market_id == market_id {
                inserted = true;
            }
        }

        assert!(inserted);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn only_creator_can_edit_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];

        // Now it should work from SUDO
        assert_ok!(PredictionMarkets::request_edit(Origin::signed(SUDO), 0, edit_reason));

        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));

        // ALICE is market creator through simple_create_categorical_market
        assert_noop!(
            PredictionMarkets::edit_market(
                Origin::signed(BOB),
                Asset::Ztg,
                0,
                CHARLIE,
                MarketPeriod::Block(0..1),
                get_deadlines(),
                gen_metadata(2),
                MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM
            ),
            Error::<Runtime>::EditorNotCreator
        );
    });
}

#[test]
fn edit_cycle_for_proposed_markets() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        run_to_block(1);
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            2..4,
            ScoringRule::CPMM,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];

        // Now it should work from SUDO
        assert_ok!(PredictionMarkets::request_edit(Origin::signed(SUDO), 0, edit_reason));

        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));

        // BOB was the oracle before through simple_create_categorical_market
        // After this edit its changed to ALICE
        assert_ok!(PredictionMarkets::edit_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            0,
            CHARLIE,
            MarketPeriod::Block(2..4),
            get_deadlines(),
            gen_metadata(2),
            MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        let edited_market = MarketCommons::market(&0).expect("Market not found");
        System::assert_last_event(Event::MarketEdited(0, edited_market).into());
        assert!(!MarketIdsForEdit::<Runtime>::contains_key(0));
        // verify oracle is CHARLIE
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().oracle, CHARLIE);
    });
}

#[cfg(feature = "parachain")]
#[test]
fn edit_market_with_foreign_asset() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];

        // Now it should work from SUDO
        assert_ok!(PredictionMarkets::request_edit(Origin::signed(SUDO), 0, edit_reason));

        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));

        // ALICE is market creator through simple_create_categorical_market
        // As per Mock asset_registry genesis ForeignAsset(50) is not registered in asset_registry.
        assert_noop!(
            PredictionMarkets::edit_market(
                Origin::signed(ALICE),
                Asset::ForeignAsset(50),
                0,
                CHARLIE,
                MarketPeriod::Block(0..1),
                get_deadlines(),
                gen_metadata(2),
                MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM
            ),
            Error::<Runtime>::UnregisteredForeignAsset
        );
        // As per Mock asset_registry genesis ForeignAsset(420) has allow_as_base_asset set to false.
        assert_noop!(
            PredictionMarkets::edit_market(
                Origin::signed(ALICE),
                Asset::ForeignAsset(420),
                0,
                CHARLIE,
                MarketPeriod::Block(0..1),
                get_deadlines(),
                gen_metadata(2),
                MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
                MarketDisputeMechanism::SimpleDisputes,
                ScoringRule::CPMM
            ),
            Error::<Runtime>::InvalidBaseAsset,
        );
        // As per Mock asset_registry genesis ForeignAsset(100) has allow_as_base_asset set to true.
        assert_ok!(PredictionMarkets::edit_market(
            Origin::signed(ALICE),
            Asset::ForeignAsset(100),
            0,
            CHARLIE,
            MarketPeriod::Block(0..1),
            get_deadlines(),
            gen_metadata(2),
            MarketType::Categorical(<Runtime as crate::Config>::MinCategories::get()),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.base_asset, Asset::ForeignAsset(100));
    });
}

#[test]
fn the_entire_market_lifecycle_works_with_timestamps() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));

        // is ok
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, CENT));
        let market = MarketCommons::market(&0).unwrap();

        // set the timestamp
        set_timestamp_for_on_initialize(100_000_000);
        run_to_block(2); // Trigger `on_initialize`; must be at least block #2.
        let grace_period: u64 = market.deadlines.grace_period * MILLISECS_PER_BLOCK as u64;
        Timestamp::set_timestamp(100_000_000 + grace_period);

        assert_noop!(
            PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, CENT),
            Error::<Runtime>::MarketIsNotActive,
        );

        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
    });
}

#[test]
fn full_scalar_market_lifecycle() {
    let test = |base_asset: Asset<MarketId>| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            base_asset,
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(3),
            MarketCreation::Permissionless,
            MarketType::Scalar(10..=30),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));

        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, 100 * BASE));

        // check balances
        let assets = PredictionMarkets::outcome_assets(0, &MarketCommons::market(&0).unwrap());
        assert_eq!(assets.len(), 2);
        for asset in assets.iter() {
            let bal = Tokens::free_balance(*asset, &CHARLIE);
            assert_eq!(bal, 100 * BASE);
        }
        let market = MarketCommons::market(&0).unwrap();

        set_timestamp_for_on_initialize(100_000_000);
        let report_at = 2;
        run_to_block(report_at); // Trigger `on_initialize`; must be at least block #2.
        let grace_period: u64 = market.deadlines.grace_period * MILLISECS_PER_BLOCK as u64;
        Timestamp::set_timestamp(100_000_000 + grace_period);

        // report
        assert_ok!(PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Scalar(100)));

        let market_after_report = MarketCommons::market(&0).unwrap();
        assert!(market_after_report.report.is_some());
        let report = market_after_report.report.unwrap();
        assert_eq!(report.at, report_at);
        assert_eq!(report.by, BOB);
        assert_eq!(report.outcome, OutcomeReport::Scalar(100));

        // dispute
        assert_ok!(PredictionMarkets::dispute(Origin::signed(DAVE), 0, OutcomeReport::Scalar(25)));
        let disputes = crate::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 1);

        run_blocks(market.deadlines.dispute_duration);

        let market_after_resolve = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after_resolve.status, MarketStatus::Resolved);
        let disputes = crate::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 0);

        // give EVE some shares
        assert_ok!(Tokens::transfer(
            Origin::signed(CHARLIE),
            EVE,
            Asset::ScalarOutcome(0, ScalarPosition::Short),
            50 * BASE
        ));

        assert_eq!(
            Tokens::free_balance(Asset::ScalarOutcome(0, ScalarPosition::Short), &CHARLIE),
            50 * BASE
        );

        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(CHARLIE), 0));
        for asset in assets.iter() {
            let bal = Tokens::free_balance(*asset, &CHARLIE);
            assert_eq!(bal, 0);
        }

        // check payouts is right for each CHARLIE and EVE
        let base_asset_bal_charlie = AssetManager::free_balance(base_asset, &CHARLIE);
        let base_asset_bal_eve = AssetManager::free_balance(base_asset, &EVE);
        assert_eq!(base_asset_bal_charlie, 98750 * CENT); // 75 (LONG) + 12.5 (SHORT) + 900 (balance)
        assert_eq!(base_asset_bal_eve, 1000 * BASE);
        System::assert_has_event(
            Event::TokensRedeemed(
                0,
                Asset::ScalarOutcome(0, ScalarPosition::Long),
                100 * BASE,
                75 * BASE,
                CHARLIE,
            )
            .into(),
        );
        System::assert_has_event(
            Event::TokensRedeemed(
                0,
                Asset::ScalarOutcome(0, ScalarPosition::Short),
                50 * BASE,
                1250 * CENT, // 12.5
                CHARLIE,
            )
            .into(),
        );

        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(EVE), 0));
        let base_asset_bal_eve_after = AssetManager::free_balance(base_asset, &EVE);
        assert_eq!(base_asset_bal_eve_after, 101250 * CENT); // 12.5 (SHORT) + 1000 (balance)
        System::assert_last_event(
            Event::TokensRedeemed(
                0,
                Asset::ScalarOutcome(0, ScalarPosition::Short),
                50 * BASE,
                1250 * CENT, // 12.5
                EVE,
            )
            .into(),
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn scalar_market_correctly_resolves_on_out_of_range_outcomes_below_threshold() {
    let test = |base_asset: Asset<MarketId>| {
        scalar_market_correctly_resolves_common(base_asset, 50);
        assert_eq!(AssetManager::free_balance(base_asset, &CHARLIE), 900 * BASE);
        assert_eq!(AssetManager::free_balance(base_asset, &EVE), 1100 * BASE);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn scalar_market_correctly_resolves_on_out_of_range_outcomes_above_threshold() {
    let test = |base_asset: Asset<MarketId>| {
        scalar_market_correctly_resolves_common(base_asset, 250);
        assert_eq!(AssetManager::free_balance(base_asset, &CHARLIE), 1000 * BASE);
        assert_eq!(AssetManager::free_balance(base_asset, &EVE), 1000 * BASE);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn reject_market_fails_on_permissionless_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::CPMM,
        );
        let reject_reason: Vec<u8> =
            vec![0; <Runtime as Config>::MaxRejectReasonLen::get() as usize];
        assert_noop!(
            PredictionMarkets::reject_market(Origin::signed(SUDO), 0, reject_reason),
            Error::<Runtime>::InvalidMarketStatus
        );
    });
}

#[test]
fn reject_market_fails_on_approved_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..1,
            ScoringRule::CPMM,
        );
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        let reject_reason: Vec<u8> =
            vec![0; <Runtime as Config>::MaxRejectReasonLen::get() as usize];
        assert_noop!(
            PredictionMarkets::reject_market(Origin::signed(SUDO), 0, reject_reason),
            Error::<Runtime>::InvalidMarketStatus
        );
    });
}

#[test]
fn market_resolve_does_not_hold_liquidity_withdraw() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(3),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        deploy_swap_pool(MarketCommons::market(&0).unwrap(), 0).unwrap();
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(ALICE), 0, BASE));
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(BOB), 0, 2 * BASE));
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, 3 * BASE));
        let market = MarketCommons::market(&0).unwrap();

        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(2)
        ));

        run_to_block(grace_period + market.deadlines.dispute_duration + 2);
        assert_ok!(Swaps::pool_exit(Origin::signed(FRED), 0, BASE * 100, vec![0, 0]));
        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(BOB), 0));
    })
}

#[test]
fn authorized_correctly_resolves_disputed_market() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            base_asset,
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            MarketDisputeMechanism::Authorized,
            ScoringRule::CPMM,
        ));
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, CENT));

        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));

        let charlie_balance = AssetManager::free_balance(base_asset, &CHARLIE);
        assert_eq!(charlie_balance, 1_000 * BASE - CENT);

        let dispute_at = grace_period + 1 + 1;
        run_to_block(dispute_at);
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));

        if base_asset == Asset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT - DisputeBond::get());
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - DisputeBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT);
        }

        // Fred authorizses an outcome, but fat-fingers it on the first try.
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(AuthorizedDisputeResolutionUser::get()),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(AuthorizedDisputeResolutionUser::get()),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        // check everyone's deposits
        let charlie_reserved = Balances::reserved_balance(&CHARLIE);
        assert_eq!(charlie_reserved, DisputeBond::get());

        // check disputes length
        let disputes = crate::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 1);

        let market_ids_1 = MarketIdsPerDisputeBlock::<Runtime>::get(
            dispute_at + <Runtime as zrml_authorized::Config>::CorrectionPeriod::get(),
        );
        assert_eq!(market_ids_1.len(), 1);

        if base_asset == Asset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT - DisputeBond::get());
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - DisputeBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT);
        }

        run_blocks(<Runtime as zrml_authorized::Config>::CorrectionPeriod::get() - 1);

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Disputed);

        if base_asset == Asset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT - DisputeBond::get());
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - DisputeBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT);
        }

        run_blocks(1);

        if base_asset == Asset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT + OracleBond::get());
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE + OracleBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE - CENT);
        }

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);
        let disputes = crate::Disputes::<Runtime>::get(0);
        assert_eq!(disputes.len(), 0);

        assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(CHARLIE), 0));

        if base_asset == Asset::Ztg {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE + OracleBond::get());
        } else {
            let charlie_balance = AssetManager::free_balance(Asset::Ztg, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE + OracleBond::get());
            let charlie_balance = AssetManager::free_balance(base_asset, &CHARLIE);
            assert_eq!(charlie_balance, 1_000 * BASE);
        }
        let charlie_reserved_2 = AssetManager::reserved_balance(Asset::Ztg, &CHARLIE);
        assert_eq!(charlie_reserved_2, 0);

        let alice_balance = AssetManager::free_balance(Asset::Ztg, &ALICE);
        assert_eq!(alice_balance, 1_000 * BASE - OracleBond::get());

        // bob kinda gets away scot-free since Alice is held responsible
        // for her designated reporter
        let bob_balance = AssetManager::free_balance(Asset::Ztg, &BOB);
        assert_eq!(bob_balance, 1_000 * BASE);

        assert!(market_after.bonds.creation.unwrap().is_settled);
        assert!(market_after.bonds.oracle.unwrap().is_settled);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn approve_market_correctly_unreserves_advisory_bond() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            base_asset,
            BOB,
            MarketPeriod::Block(0..100),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        let market_id = 0;
        let alice_balance_before = Balances::free_balance(&ALICE);
        check_reserve(&ALICE, AdvisoryBond::get() + OracleBond::get());
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), market_id));
        check_reserve(&ALICE, OracleBond::get());
        assert_eq!(Balances::free_balance(&ALICE), alice_balance_before + AdvisoryBond::get());
        let market = MarketCommons::market(&market_id).unwrap();
        assert!(market.bonds.creation.unwrap().is_settled);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn deploy_swap_pool_correctly_sets_weight_of_base_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let weights = vec![
            <Runtime as zrml_swaps::Config>::MinWeight::get() + 11,
            <Runtime as zrml_swaps::Config>::MinWeight::get() + 22,
            <Runtime as zrml_swaps::Config>::MinWeight::get() + 33,
        ];
        assert_ok!(PredictionMarkets::create_cpmm_market_and_deploy_assets(
            Origin::signed(ALICE),
            Asset::Ztg,
            ALICE,
            MarketPeriod::Block(0..42),
            get_deadlines(),
            gen_metadata(50),
            MarketType::Categorical(3),
            MarketDisputeMechanism::SimpleDisputes,
            1,
            <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
            weights,
        ));
        let pool = <Pools<Runtime>>::get(0).unwrap();
        let pool_weights = pool.weights.unwrap();
        assert_eq!(
            pool_weights[&Asset::Ztg],
            3 * <Runtime as zrml_swaps::Config>::MinWeight::get() + 66
        );
    });
}

#[test]
fn deploy_swap_pool_for_market_returns_error_if_weights_is_too_short() {
    ExtBuilder::default().build().execute_with(|| {
        let category_count = 5;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Block(0..100),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        let amount = 123 * BASE;
        assert_ok!(Balances::set_balance(Origin::root(), ALICE, 2 * amount, 0));
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(ALICE), 0, amount));
        // Attempt to create a pool with four weights; but we need five instead (base asset not
        // counted).
        assert_noop!(
            PredictionMarkets::deploy_swap_pool_for_market(
                Origin::signed(ALICE),
                0,
                1,
                amount,
                vec![
                    <Runtime as zrml_swaps::Config>::MinWeight::get();
                    (category_count - 1).into()
                ],
            ),
            Error::<Runtime>::WeightsLenMustEqualAssetsLen,
        );
    });
}

#[test]
fn deploy_swap_pool_for_market_returns_error_if_weights_is_too_long() {
    ExtBuilder::default().build().execute_with(|| {
        let category_count = 5;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Block(0..100),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        let amount = 123 * BASE;
        assert_ok!(Balances::set_balance(Origin::root(), ALICE, 2 * amount, 0));
        assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(ALICE), 0, amount));
        // Attempt to create a pool with six weights; but we need five instead (base asset not
        // counted).
        assert_noop!(
            PredictionMarkets::deploy_swap_pool_for_market(
                Origin::signed(ALICE),
                0,
                <Runtime as zrml_swaps::Config>::MaxSwapFee::get(),
                amount,
                vec![
                    <Runtime as zrml_swaps::Config>::MinWeight::get();
                    (category_count + 1).into()
                ],
            ),
            Error::<Runtime>::WeightsLenMustEqualAssetsLen,
        );
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_on_oracle_report()
 {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            base_asset,
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        let alice_balance_before = Balances::free_balance(&ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_to_block(grace_period + market.deadlines.dispute_duration + 1);
        check_reserve(&ALICE, 0);
        assert_eq!(
            Balances::free_balance(&ALICE),
            alice_balance_before + ValidityBond::get() + OracleBond::get()
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_on_outsider_report()
 {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            base_asset,
            BOB,
            MarketPeriod::Block(0..100),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        let alice_balance_before = Balances::free_balance(&ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());

        let charlie_balance_before = Balances::free_balance(&CHARLIE);
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        let report_at = grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(report_at);

        assert!(market.bonds.outsider.is_none());
        assert_ok!(PredictionMarkets::report(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.bonds.outsider, Some(Bond::new(CHARLIE, OutsiderBond::get())));
        check_reserve(&CHARLIE, OutsiderBond::get());
        assert_eq!(Balances::free_balance(&CHARLIE), charlie_balance_before - OutsiderBond::get());
        let charlie_balance_before = Balances::free_balance(&CHARLIE);

        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // Check that validity bond didn't get slashed, but oracle bond did
        assert_eq!(Balances::free_balance(&ALICE), alice_balance_before + ValidityBond::get());

        check_reserve(&CHARLIE, 0);
        // Check that the outsider gets the OracleBond together with the OutsiderBond
        assert_eq!(
            Balances::free_balance(&CHARLIE),
            charlie_balance_before + OracleBond::get() + OutsiderBond::get()
        );
        let market = MarketCommons::market(&0).unwrap();
        assert!(market.bonds.outsider.unwrap().is_settled);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn outsider_reports_wrong_outcome() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();

        let end = 100;
        let alice_balance_before = Balances::free_balance(&ALICE);
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            base_asset,
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));

        let outsider = CHARLIE;

        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        let report_at = grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(report_at);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(outsider),
            0,
            OutcomeReport::Categorical(1)
        ));

        let outsider_balance_before = Balances::free_balance(&outsider);
        check_reserve(&outsider, OutsiderBond::get());

        let dispute_at_0 = report_at + 1;
        run_to_block(dispute_at_0);
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(EVE),
            0,
            OutcomeReport::Categorical(0)
        ));

        let eve_balance_before = Balances::free_balance(&EVE);

        // on_resolution called
        run_blocks(market.deadlines.dispute_duration);

        assert_eq!(Balances::free_balance(&ALICE), alice_balance_before - OracleBond::get());

        check_reserve(&outsider, 0);
        assert_eq!(Balances::free_balance(&outsider), outsider_balance_before);

        let dispute_bond = crate::default_dispute_bond::<Runtime>(0usize);
        // disputor EVE gets the OracleBond and OutsiderBond and dispute bond
        assert_eq!(
            Balances::free_balance(&EVE),
            eve_balance_before + dispute_bond + OutsiderBond::get() + OracleBond::get()
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_approved_advised_market_on_oracle_report()
 {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            base_asset,
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        let alice_balance_before = Balances::free_balance(&ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        let report_at = grace_period + 1;
        run_to_block(report_at);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // Check that nothing got slashed
        assert_eq!(Balances::free_balance(&ALICE), alice_balance_before + OracleBond::get());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_approved_advised_market_on_outsider_report()
 {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            base_asset,
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        let alice_balance_before = Balances::free_balance(&ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        let report_at = grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(report_at);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        // Check that oracle bond got slashed
        check_reserve(&ALICE, 0);
        assert_eq!(Balances::free_balance(&ALICE), alice_balance_before);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_with_correct_disputed_outcome_with_oracle_report()
 {
    // Oracle reports in time but incorrect report, so OracleBond gets slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            base_asset,
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        let alice_balance_before = Balances::free_balance(&ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is slashed
        assert_eq!(Balances::free_balance(&ALICE), alice_balance_before + ValidityBond::get());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_approved_advised_market_with_correct_disputed_outcome_with_oracle_report()
 {
    // Oracle reports in time but incorrect report, so OracleBond gets slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            base_asset,
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        let alice_balance_before = Balances::free_balance(&ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is slashed
        assert_eq!(Balances::free_balance(&ALICE), alice_balance_before);
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_with_wrong_disputed_outcome_with_oracle_report()
 {
    // Oracle reports in time and correct report, so OracleBond does not get slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            base_asset,
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        let alice_balance_before = Balances::free_balance(&ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        // EVE disputes with wrong outcome
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is not slashed
        assert_eq!(
            Balances::free_balance(&ALICE),
            alice_balance_before + ValidityBond::get() + OracleBond::get()
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_advised_approved_market_with_wrong_disputed_outcome_with_oracle_report()
 {
    // Oracle reports in time and correct report, so OracleBond does not get slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            base_asset,
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));
        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        let alice_balance_before = Balances::free_balance(&ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            Origin::signed(BOB),
            0,
            OutcomeReport::Categorical(0)
        ));
        // EVE disputes with wrong outcome
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is not slashed
        assert_eq!(Balances::free_balance(&ALICE), alice_balance_before + OracleBond::get());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_permissionless_market_with_disputed_outcome_with_outsider_report()
 {
    // Oracle does not report in time, so OracleBond gets slashed on resolution
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            base_asset,
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));

        let alice_balance_before = Balances::free_balance(&ALICE);
        check_reserve(&ALICE, ValidityBond::get() + OracleBond::get());

        let outsider = CHARLIE;

        let market = MarketCommons::market(&0).unwrap();
        let after_oracle_duration =
            end + market.deadlines.grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(after_oracle_duration);
        // CHARLIE is not an Oracle
        assert_ok!(PredictionMarkets::report(
            Origin::signed(outsider),
            0,
            OutcomeReport::Categorical(0)
        ));

        let outsider_balance_before = Balances::free_balance(&outsider);
        check_reserve(&outsider, OutsiderBond::get());

        // EVE disputes with wrong outcome
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(FRED),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is slashed
        assert_eq!(Balances::free_balance(&ALICE), alice_balance_before + ValidityBond::get());

        check_reserve(&outsider, 0);
        assert_eq!(
            Balances::free_balance(&outsider),
            outsider_balance_before + OracleBond::get() + OutsiderBond::get()
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn on_resolution_correctly_reserves_and_unreserves_bonds_for_advised_approved_market_with_disputed_outcome_with_outsider_report()
 {
    // Oracle does not report in time, so OracleBond gets slashed on resolution
    // NOTE: Bonds are always in ZTG
    let test = |base_asset: Asset<MarketId>| {
        reserve_sentinel_amounts();
        let end = 100;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            base_asset,
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM,
        ));

        let outsider = CHARLIE;

        assert_ok!(PredictionMarkets::approve_market(Origin::signed(SUDO), 0));
        let alice_balance_before = Balances::free_balance(&ALICE);
        check_reserve(&ALICE, OracleBond::get());
        let market = MarketCommons::market(&0).unwrap();
        let after_oracle_duration =
            end + market.deadlines.grace_period + market.deadlines.oracle_duration + 1;
        run_to_block(after_oracle_duration);
        // CHARLIE is not an Oracle
        assert_ok!(PredictionMarkets::report(
            Origin::signed(outsider),
            0,
            OutcomeReport::Categorical(0)
        ));

        let outsider_balance_before = Balances::free_balance(&outsider);
        check_reserve(&outsider, OutsiderBond::get());

        // EVE disputes with wrong outcome
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));
        assert_ok!(PredictionMarkets::dispute(
            Origin::signed(FRED),
            0,
            OutcomeReport::Categorical(0)
        ));
        run_blocks(market.deadlines.dispute_duration);
        check_reserve(&ALICE, 0);
        // ValidityBond bond is returned but OracleBond is slashed
        assert_eq!(Balances::free_balance(&ALICE), alice_balance_before);

        check_reserve(&outsider, 0);
        assert_eq!(
            Balances::free_balance(&outsider),
            outsider_balance_before + OracleBond::get() + OutsiderBond::get()
        );
    };
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(Asset::ForeignAsset(100));
    });
}

#[test]
fn report_fails_on_market_state_proposed() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_closed_for_advised_market() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_collecting_subsidy() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Timestamp(100_000_000..200_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::RikiddoSigmoidFeeMarketEma
        ));
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_insufficient_subsidy() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Timestamp(100_000_000..200_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::RikiddoSigmoidFeeMarketEma
        ));
        let _ = MarketCommons::mutate_market(&0, |market| {
            market.status = MarketStatus::InsufficientSubsidy;
            Ok(())
        });
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_active() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_suspended() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        let _ = MarketCommons::mutate_market(&0, |market| {
            market.status = MarketStatus::Suspended;
            Ok(())
        });
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_resolved() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        let _ = MarketCommons::mutate_market(&0, |market| {
            market.status = MarketStatus::Resolved;
            Ok(())
        });
        assert_noop!(
            PredictionMarkets::report(Origin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_if_reporter_is_not_the_oracle() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            MarketDisputeMechanism::SimpleDisputes,
            ScoringRule::CPMM
        ));
        let market = MarketCommons::market(&0).unwrap();
        set_timestamp_for_on_initialize(100_000_000);
        // Trigger hooks which close the market.
        run_to_block(2);
        let grace_period: u64 = market.deadlines.grace_period * MILLISECS_PER_BLOCK as u64;
        set_timestamp_for_on_initialize(100_000_000 + grace_period + MILLISECS_PER_BLOCK as u64);
        assert_noop!(
            PredictionMarkets::report(Origin::signed(CHARLIE), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::ReporterNotOracle,
        );
    });
}

#[test]
fn create_market_succeeds_if_market_duration_is_maximal_in_blocks() {
    ExtBuilder::default().build().execute_with(|| {
        let now = 1;
        frame_system::Pallet::<Runtime>::set_block_number(now);
        let start = 5;
        let end = now + <Runtime as Config>::MaxMarketLifetime::get();
        assert!(
            end > start,
            "Test failed due to misconfiguration: `MaxMarketLifetime` is too small"
        );
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Block(start..end),
            get_deadlines(),
            gen_metadata(0),
            MarketCreation::Permissionless,
            MarketType::Categorical(3),
            MarketDisputeMechanism::Authorized,
            ScoringRule::CPMM,
        ));
    });
}

#[test]
fn create_market_suceeds_if_market_duration_is_maximal_in_moments() {
    ExtBuilder::default().build().execute_with(|| {
        let now = 12_001u64;
        Timestamp::set_timestamp(now);
        let start = 5 * MILLISECS_PER_BLOCK as u64;
        let end =
            now + <Runtime as Config>::MaxMarketLifetime::get() * (MILLISECS_PER_BLOCK as u64);
        assert!(
            end > start,
            "Test failed due to misconfiguration: `MaxMarketLifetime` is too small"
        );
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(ALICE),
            Asset::Ztg,
            BOB,
            MarketPeriod::Timestamp(start..end),
            get_deadlines(),
            gen_metadata(0),
            MarketCreation::Permissionless,
            MarketType::Categorical(3),
            MarketDisputeMechanism::Authorized,
            ScoringRule::CPMM,
        ));
    });
}

#[test]
fn create_market_fails_if_market_duration_is_too_long_in_blocks() {
    ExtBuilder::default().build().execute_with(|| {
        let now = 1;
        frame_system::Pallet::<Runtime>::set_block_number(now);
        let start = 5;
        let end = now + <Runtime as Config>::MaxMarketLifetime::get() + 1;
        assert!(
            end > start,
            "Test failed due to misconfiguration: `MaxMarketLifetime` is too small"
        );
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::Ztg,
                BOB,
                MarketPeriod::Block(start..end),
                get_deadlines(),
                gen_metadata(0),
                MarketCreation::Permissionless,
                MarketType::Categorical(3),
                MarketDisputeMechanism::Authorized,
                ScoringRule::CPMM,
            ),
            crate::Error::<Runtime>::MarketDurationTooLong,
        );
    });
}

#[test]
fn create_market_fails_if_market_duration_is_too_long_in_moments() {
    ExtBuilder::default().build().execute_with(|| {
        let now = 12_001u64;
        Timestamp::set_timestamp(now);
        let start = 5 * MILLISECS_PER_BLOCK as u64;
        let end = now
            + (<Runtime as Config>::MaxMarketLifetime::get() + 1) * (MILLISECS_PER_BLOCK as u64);
        assert!(
            end > start,
            "Test failed due to misconfiguration: `MaxMarketLifetime` is too small"
        );
        assert_noop!(
            PredictionMarkets::create_market(
                Origin::signed(ALICE),
                Asset::Ztg,
                BOB,
                MarketPeriod::Timestamp(start..end),
                get_deadlines(),
                gen_metadata(0),
                MarketCreation::Permissionless,
                MarketType::Categorical(3),
                MarketDisputeMechanism::Authorized,
                ScoringRule::CPMM,
            ),
            crate::Error::<Runtime>::MarketDurationTooLong,
        );
    });
}

#[test_case(
    MarketCreation::Advised,
    ScoringRule::CPMM,
    MarketStatus::Proposed,
    MarketBonds {
        creation: Some(Bond::new(ALICE, <Runtime as Config>::AdvisoryBond::get())),
        oracle: Some(Bond::new(ALICE, <Runtime as Config>::OracleBond::get())),
        outsider: None,
    }
)]
#[test_case(
    MarketCreation::Permissionless,
    ScoringRule::CPMM,
    MarketStatus::Active,
    MarketBonds {
        creation: Some(Bond::new(ALICE, <Runtime as Config>::ValidityBond::get())),
        oracle: Some(Bond::new(ALICE, <Runtime as Config>::OracleBond::get())),
        outsider: None,
    }
)]
fn create_market_sets_the_correct_market_parameters_and_reserves_the_correct_amount(
    creation: MarketCreation,
    scoring_rule: ScoringRule,
    status: MarketStatus,
    bonds: MarketBonds<AccountIdTest, Balance>,
) {
    ExtBuilder::default().build().execute_with(|| {
        let creator = ALICE;
        let oracle = BOB;
        let period = MarketPeriod::Block(1..2);
        let deadlines = Deadlines {
            grace_period: 1,
            oracle_duration: <Runtime as crate::Config>::MinOracleDuration::get() + 2,
            dispute_duration: <Runtime as crate::Config>::MinDisputeDuration::get() + 3,
        };
        let metadata = gen_metadata(0x99);
        let MultiHash::Sha3_384(multihash) = metadata;
        let market_type = MarketType::Categorical(7);
        let dispute_mechanism = MarketDisputeMechanism::Authorized;
        assert_ok!(PredictionMarkets::create_market(
            Origin::signed(creator),
            Asset::Ztg,
            oracle,
            period.clone(),
            deadlines,
            metadata,
            creation.clone(),
            market_type.clone(),
            dispute_mechanism.clone(),
            scoring_rule,
        ));
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.creator, creator);
        assert_eq!(market.creation, creation);
        assert_eq!(market.creator_fee, 0);
        assert_eq!(market.oracle, oracle);
        assert_eq!(market.metadata, multihash);
        assert_eq!(market.market_type, market_type);
        assert_eq!(market.period, period);
        assert_eq!(market.deadlines, deadlines);
        assert_eq!(market.scoring_rule, scoring_rule);
        assert_eq!(market.status, status);
        assert_eq!(market.report, None);
        assert_eq!(market.resolved_outcome, None);
        assert_eq!(market.dispute_mechanism, dispute_mechanism);
        assert_eq!(market.bonds, bonds);
    });
}

fn deploy_swap_pool(
    market: Market<AccountIdTest, Balance, BlockNumber, Moment, Asset<u128>>,
    market_id: u128,
) -> DispatchResultWithPostInfo {
    assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(FRED), 0, 100 * BASE));
    assert_ok!(Balances::transfer(
        Origin::signed(FRED),
        <Runtime as crate::Config>::PalletId::get().into_account_truncating(),
        100 * BASE
    ));
    let outcome_assets_len = PredictionMarkets::outcome_assets(market_id, &market).len();
    PredictionMarkets::deploy_swap_pool_for_market(
        Origin::signed(FRED),
        0,
        <Runtime as zrml_swaps::Config>::MaxSwapFee::get(),
        <Runtime as zrml_swaps::Config>::MinLiquidity::get(),
        vec![<Runtime as zrml_swaps::Config>::MinWeight::get(); outcome_assets_len],
    )
}

// Common code of `scalar_market_correctly_resolves_*`
fn scalar_market_correctly_resolves_common(base_asset: Asset<MarketId>, reported_value: u128) {
    let end = 100;
    simple_create_scalar_market(
        base_asset,
        MarketCreation::Permissionless,
        0..end,
        ScoringRule::CPMM,
    );
    assert_ok!(PredictionMarkets::buy_complete_set(Origin::signed(CHARLIE), 0, 100 * BASE));
    assert_ok!(Tokens::transfer(
        Origin::signed(CHARLIE),
        EVE,
        Asset::ScalarOutcome(0, ScalarPosition::Short),
        100 * BASE
    ));
    // (Eve now has 100 SHORT, Charlie has 100 LONG)

    let market = MarketCommons::market(&0).unwrap();
    let grace_period = end + market.deadlines.grace_period;
    run_to_block(grace_period + 1);
    assert_ok!(PredictionMarkets::report(
        Origin::signed(BOB),
        0,
        OutcomeReport::Scalar(reported_value)
    ));
    let market_after_report = MarketCommons::market(&0).unwrap();
    assert!(market_after_report.report.is_some());
    let report = market_after_report.report.unwrap();
    assert_eq!(report.at, grace_period + 1);
    assert_eq!(report.by, BOB);
    assert_eq!(report.outcome, OutcomeReport::Scalar(reported_value));

    run_blocks(market.deadlines.dispute_duration);
    let market_after_resolve = MarketCommons::market(&0).unwrap();
    assert_eq!(market_after_resolve.status, MarketStatus::Resolved);

    // Check balances before redeeming (just to make sure that our tests are based on correct
    // assumptions)!
    assert_eq!(AssetManager::free_balance(base_asset, &CHARLIE), 900 * BASE);
    assert_eq!(AssetManager::free_balance(base_asset, &EVE), 1000 * BASE);

    assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(CHARLIE), 0));
    assert_ok!(PredictionMarkets::redeem_shares(Origin::signed(EVE), 0));
    let assets = PredictionMarkets::outcome_assets(0, &MarketCommons::market(&0).unwrap());
    for asset in assets.iter() {
        assert_eq!(AssetManager::free_balance(*asset, &CHARLIE), 0);
        assert_eq!(AssetManager::free_balance(*asset, &EVE), 0);
    }
}
