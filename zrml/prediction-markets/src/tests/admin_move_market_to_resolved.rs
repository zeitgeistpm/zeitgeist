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
use test_case::test_case;

use zeitgeist_primitives::types::OutcomeReport;

#[test]
fn admin_move_market_to_resolved_resolves_reported_market() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: AssetOf<Runtime>| {
        let end = 33;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
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
        let balance_free_before = Balances::free_balance(ALICE);
        let balance_reserved_before =
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE);

        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        let category = 1;
        let outcome_report = OutcomeReport::Categorical(category);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            market_id,
            outcome_report.clone()
        ));
        assert_ok!(PredictionMarkets::admin_move_market_to_resolved(
            RuntimeOrigin::signed(ResolveOrigin::get()),
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
            Balances::free_balance(ALICE),
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

#[test]
fn admin_move_market_to_resolved_resolves_disputed_market() {
    // NOTE: Bonds are always in ZTG, irrespective of base_asset.
    let test = |base_asset: AssetOf<Runtime>| {
        let end = 33;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
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
        let balance_free_before = Balances::free_balance(ALICE);
        let balance_reserved_before =
            Balances::reserved_balance_named(&PredictionMarkets::reserve_id(), &ALICE);

        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        let category = 1;
        let outcome_report = OutcomeReport::Categorical(category);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            market_id,
            OutcomeReport::Categorical(0),
        ));
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), market_id));
        assert_ok!(Authorized::authorize_market_outcome(
            RuntimeOrigin::signed(AuthorizedDisputeResolutionUser::get()),
            market_id,
            outcome_report.clone(),
        ));
        assert_ok!(PredictionMarkets::admin_move_market_to_resolved(
            RuntimeOrigin::signed(ResolveOrigin::get()),
            market_id
        ));

        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Resolved);
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
            Balances::free_balance(ALICE),
            balance_free_before + <Runtime as Config>::ValidityBond::get()
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

#[test_case(MarketStatus::Active)]
#[test_case(MarketStatus::Closed)]
#[test_case(MarketStatus::Resolved)]
fn admin_move_market_to_resolved_fails_if_market_is_not_reported_or_disputed(
    market_status: MarketStatus,
) {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..33,
            ScoringRule::AmmCdaHybrid,
        );
        let market_id = 0;
        assert_ok!(MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        }));
        assert_noop!(
            PredictionMarkets::admin_move_market_to_resolved(
                RuntimeOrigin::signed(ResolveOrigin::get()),
                market_id,
            ),
            Error::<Runtime>::InvalidMarketStatus,
        );
    });
}
