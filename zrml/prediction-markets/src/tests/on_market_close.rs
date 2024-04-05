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

use super::*;

use crate::{LastTimeFrame, MarketIdsForEdit};
use zeitgeist_primitives::constants::MILLISECS_PER_BLOCK;

#[test]
fn on_market_close_auto_rejects_expired_advised_market() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: BaseAsset| {
        // Give ALICE `SENTINEL_AMOUNT` free and reserved ZTG; we record the free balance to check
        // that the AdvisoryBond and the OracleBond gets unreserved, when the advised market expires.
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before_alice = Balances::free_balance(ALICE);
        let balance_reserved_before_alice =
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE);

        let end = 33;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Advised,
            0..end,
            ScoringRule::Lmsr,
        );
        let market_id = 0;

        run_to_block(end);

        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            balance_reserved_before_alice
        );
        assert_eq!(Balances::free_balance(ALICE), balance_free_before_alice);
        assert_noop!(
            MarketCommons::market(&market_id),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist,
        );
        System::assert_has_event(Event::MarketExpired(market_id).into());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::CampaignAsset(100));
    });
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::ForeignAsset(100));
    });
}

#[test]
fn on_market_close_auto_rejects_expired_advised_market_with_edit_request() {
    let test = |base_asset: BaseAsset| {
        // Give ALICE `SENTINEL_AMOUNT` free and reserved ZTG; we record the free balance to check
        // that the AdvisoryBond and the OracleBond gets unreserved, when the advised market expires.
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT
        ));
        let balance_free_before_alice = Balances::free_balance(ALICE);
        let balance_reserved_before_alice =
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE);

        let end = 33;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Advised,
            0..end,
            ScoringRule::Lmsr,
        );
        run_to_block(2);
        let market_id = 0;
        let market = MarketCommons::market(&market_id);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];

        assert_ok!(PredictionMarkets::request_edit(
            RuntimeOrigin::signed(RequestEditOrigin::get()),
            market_id,
            edit_reason
        ));

        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));
        run_blocks(end);
        assert!(!MarketIdsForEdit::<Runtime>::contains_key(0));

        assert_eq!(
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE),
            balance_reserved_before_alice
        );
        assert_eq!(Balances::free_balance(ALICE), balance_free_before_alice);
        assert_noop!(
            MarketCommons::market(&market_id),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist,
        );
        System::assert_has_event(Event::MarketExpired(market_id).into());
    };
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::CampaignAsset(100));
    });
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::Ztg);
    });
    #[cfg(feature = "parachain")]
    ExtBuilder::default().build().execute_with(|| {
        test(BaseAsset::ForeignAsset(100));
    });
}

#[test]
fn on_market_close_successfully_auto_closes_market_with_blocks() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 33;
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            ALICE,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        let market_id = 0;

        run_to_block(end - 1);
        let market_before_close = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market_before_close.status, MarketStatus::Active);

        run_to_block(end);
        let market_after_close = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market_after_close.status, MarketStatus::Closed);

        System::assert_last_event(Event::MarketClosed(market_id).into());
    });
}

#[test]
fn on_market_close_successfully_auto_closes_market_with_timestamps() {
    ExtBuilder::default().build().execute_with(|| {
        let end = (2 * MILLISECS_PER_BLOCK) as u64;
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            ALICE,
            MarketPeriod::Timestamp(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        let market_id = 0;

        // (Check that the market doesn't close too soon)
        set_timestamp_for_on_initialize(end - 1);
        run_to_block(2); // Trigger `on_initialize`; must be at least block #2!
        let market_before_close = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market_before_close.status, MarketStatus::Active);

        set_timestamp_for_on_initialize(end);
        run_blocks(1);
        let market_after_close = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market_after_close.status, MarketStatus::Closed);

        System::assert_last_event(Event::MarketClosed(market_id).into());
    });
}

#[test]
fn on_market_close_successfully_auto_closes_multiple_markets_after_stall() {
    // We check that `on_market_close` works correctly even if a block takes much longer than 12sec
    // to be produced and multiple markets are involved.
    ExtBuilder::default().build().execute_with(|| {
        // Mock last time frame to prevent it from defaulting.
        LastTimeFrame::<Runtime>::set(Some(0));

        let end = (5 * MILLISECS_PER_BLOCK) as u64;
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            ALICE,
            MarketPeriod::Timestamp(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            ALICE,
            MarketPeriod::Timestamp(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));

        // This block takes much longer than 12sec, but markets and pools still close correctly.
        set_timestamp_for_on_initialize(9 * MILLISECS_PER_BLOCK as u64);
        run_to_block(2); // Trigger `on_initialize`; must be at least block #2!

        let market_after_close = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after_close.status, MarketStatus::Closed);
        System::assert_has_event(Event::MarketClosed(0).into());

        let market_after_close = MarketCommons::market(&1).unwrap();
        assert_eq!(market_after_close.status, MarketStatus::Closed);
        System::assert_has_event(Event::MarketClosed(1).into());
    });
}

#[test]
fn on_market_close_market_status_manager_exceeds_max_recovery_time_frames_after_stall() {
    // We check that `on_market_close` works correctly even if a block takes much longer than 12sec
    // to be produced and multiple markets are involved.
    ExtBuilder::default().build().execute_with(|| {
        // Mock last time frame to prevent it from defaulting.
        LastTimeFrame::<Runtime>::set(Some(0));

        let end = (5 * MILLISECS_PER_BLOCK) as u64;
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            ALICE,
            MarketPeriod::Timestamp(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            ALICE,
            MarketPeriod::Timestamp(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));

        set_timestamp_for_on_initialize(
            end + (crate::MAX_RECOVERY_TIME_FRAMES + 1) * MILLISECS_PER_BLOCK as u64,
        );
        run_to_block(2); // Trigger `on_initialize`; must be at least block #2!

        System::assert_last_event(
            Event::RecoveryLimitReached { last_time_frame: 0, limit_time_frame: 6 }.into(),
        );

        // still active, not closed, because recovery limit reached
        let market_after_close = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after_close.status, MarketStatus::Active);

        let market_after_close = MarketCommons::market(&1).unwrap();
        assert_eq!(market_after_close.status, MarketStatus::Active);
    });
}
