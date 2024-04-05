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

use crate::{MarketIdsForEdit, MarketIdsPerCloseBlock};

// TODO(#1239) MarketDoesNotExist
// TODO(#1239) Fails if market is not proposed

#[test]
fn it_allows_the_advisory_origin_to_reject_markets() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Advised,
            4..6,
            ScoringRule::Lmsr,
        );

        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let reject_reason: Vec<u8> =
            vec![0; <Runtime as Config>::MaxRejectReasonLen::get() as usize];
        assert_ok!(PredictionMarkets::reject_market(
            RuntimeOrigin::signed(RejectOrigin::get()),
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
            BaseAsset::Ztg,
            MarketCreation::Advised,
            0..2,
            ScoringRule::Lmsr,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let reject_reason: Vec<u8> =
            vec![0; <Runtime as Config>::MaxRejectReasonLen::get() as usize + 1];
        assert_noop!(
            PredictionMarkets::reject_market(
                RuntimeOrigin::signed(RejectOrigin::get()),
                0,
                reject_reason
            ),
            Error::<Runtime>::RejectReasonLengthExceedsMaxRejectReasonLen
        );
    });
}

#[test]
fn it_allows_the_advisory_origin_to_reject_markets_with_edit_request() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Advised,
            0..2,
            ScoringRule::Lmsr,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];

        let reject_reason = vec![0_u8; <Runtime as Config>::MaxRejectReasonLen::get() as usize];
        assert_ok!(PredictionMarkets::request_edit(
            RuntimeOrigin::signed(RequestEditOrigin::get()),
            0,
            edit_reason
        ));
        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));
        assert_ok!(PredictionMarkets::reject_market(
            RuntimeOrigin::signed(RejectOrigin::get()),
            0,
            reject_reason
        ));
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
    let test = |base_asset: BaseAsset| {
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Advised,
            0..2,
            ScoringRule::Lmsr,
        );

        // Give ALICE `SENTINEL_AMOUNT` free and reserved ZTG; we record the free balance to check
        // that the AdvisoryBond gets slashed but the OracleBond gets unreserved.
        assert_ok!(AssetManager::deposit(Asset::Ztg, &ALICE, 2 * SENTINEL_AMOUNT));
        assert_ok!(Balances::reserve_named(
            &PredictionMarkets::reserve_id(),
            &ALICE,
            SENTINEL_AMOUNT,
        ));
        assert_eq!(Balances::free_balance(Treasury::account_id()), 0);

        let balance_free_before_alice = Balances::free_balance(ALICE);
        let balance_reserved_before_alice =
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE);

        let reject_reason: Vec<u8> =
            vec![0; <Runtime as Config>::MaxRejectReasonLen::get() as usize];
        assert_ok!(PredictionMarkets::reject_market(
            RuntimeOrigin::signed(RejectOrigin::get()),
            0,
            reject_reason
        ));

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
        let balance_free_after_alice = Balances::free_balance(ALICE);
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
fn reject_market_clears_auto_close_blocks() {
    // We don't have to check that reject market clears the cache for opening pools, since Cpmm pools
    // can not be deployed on pending advised pools.
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Advised,
            33..66,
            ScoringRule::Lmsr,
        );
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Advised,
            22..66,
            ScoringRule::Lmsr,
        );
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Advised,
            22..33,
            ScoringRule::Lmsr,
        );
        let reject_reason: Vec<u8> =
            vec![0; <Runtime as Config>::MaxRejectReasonLen::get() as usize];
        assert_ok!(PredictionMarkets::reject_market(
            RuntimeOrigin::signed(RejectOrigin::get()),
            0,
            reject_reason
        ));

        let auto_close = MarketIdsPerCloseBlock::<Runtime>::get(66);
        assert_eq!(auto_close.len(), 1);
        assert_eq!(auto_close[0], 1);
    });
}

#[test]
fn reject_market_fails_on_permissionless_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::Lmsr,
        );
        let reject_reason: Vec<u8> =
            vec![0; <Runtime as Config>::MaxRejectReasonLen::get() as usize];
        assert_noop!(
            PredictionMarkets::reject_market(
                RuntimeOrigin::signed(RejectOrigin::get()),
                0,
                reject_reason
            ),
            Error::<Runtime>::InvalidMarketStatus
        );
    });
}

#[test]
fn reject_market_fails_on_approved_market() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Advised,
            0..2,
            ScoringRule::Lmsr,
        );
        assert_ok!(PredictionMarkets::approve_market(
            RuntimeOrigin::signed(ApproveOrigin::get()),
            0
        ));
        let reject_reason: Vec<u8> =
            vec![0; <Runtime as Config>::MaxRejectReasonLen::get() as usize];
        assert_noop!(
            PredictionMarkets::reject_market(
                RuntimeOrigin::signed(RejectOrigin::get()),
                0,
                reject_reason
            ),
            Error::<Runtime>::InvalidMarketStatus
        );
    });
}
