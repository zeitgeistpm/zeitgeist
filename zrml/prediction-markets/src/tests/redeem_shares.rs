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

use zeitgeist_primitives::types::{OutcomeReport, ScalarPosition};

// TODO(#1239) MarketIsNotResolved
// TODO(#1239) NoWinningBalance
// TODO(#1239) MarketDoesNotExist

#[test]
fn it_allows_to_redeem_shares() {
    let test = |base_asset: BaseAsset| {
        let end = 2;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(CHARLIE), 0, CENT));
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
        run_blocks(market.deadlines.dispute_duration);
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Resolved);

        assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(CHARLIE), 0));
        let bal = Balances::free_balance(CHARLIE);
        assert_eq!(bal, 1_000 * BASE);
        System::assert_last_event(
            Event::TokensRedeemed(0, Asset::CategoricalOutcome(0, 1), CENT, CENT, CHARLIE).into(),
        );
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

#[test_case(ScoringRule::Parimutuel; "parimutuel")]
fn redeem_shares_fails_if_invalid_resolution_mechanism(scoring_rule: ScoringRule) {
    let test = |base_asset: BaseAsset| {
        let end = 2;
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..end,
            scoring_rule,
        );

        assert_ok!(MarketCommons::mutate_market(&0, |market_inner| {
            market_inner.status = MarketStatus::Resolved;
            Ok(())
        }));

        assert_noop!(
            PredictionMarkets::redeem_shares(RuntimeOrigin::signed(CHARLIE), 0),
            Error::<Runtime>::InvalidResolutionMechanism
        );
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
fn scalar_market_correctly_resolves_on_out_of_range_outcomes_below_threshold() {
    let test = |base_asset: BaseAsset| {
        scalar_market_correctly_resolves_common(base_asset, 50);
        assert_eq!(AssetManager::free_balance(base_asset.into(), &CHARLIE), 900 * BASE);
        assert_eq!(AssetManager::free_balance(base_asset.into(), &EVE), 1100 * BASE);
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
fn scalar_market_correctly_resolves_on_out_of_range_outcomes_above_threshold() {
    let test = |base_asset: BaseAsset| {
        scalar_market_correctly_resolves_common(base_asset, 250);
        assert_eq!(AssetManager::free_balance(base_asset.into(), &CHARLIE), 1000 * BASE);
        assert_eq!(AssetManager::free_balance(base_asset.into(), &EVE), 1000 * BASE);
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

// Common code of `scalar_market_correctly_resolves_*`
fn scalar_market_correctly_resolves_common(base_asset: BaseAsset, reported_value: u128) {
    let end = 100;
    simple_create_scalar_market(
        base_asset,
        MarketCreation::Permissionless,
        0..end,
        ScoringRule::AmmCdaHybrid,
    );
    assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(CHARLIE), 0, 100 * BASE));
    assert_ok!(AssetManager::transfer(
        RuntimeOrigin::signed(CHARLIE),
        EVE,
        Asset::ScalarOutcome(0, ScalarPosition::Short),
        100 * BASE
    ));
    // (Eve now has 100 SHORT, Charlie has 100 LONG)

    let market = MarketCommons::market(&0).unwrap();
    let grace_period = end + market.deadlines.grace_period;
    run_to_block(grace_period + 1);
    assert_ok!(PredictionMarkets::report(
        RuntimeOrigin::signed(BOB),
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
    assert_eq!(AssetManager::free_balance(base_asset.into(), &CHARLIE), 900 * BASE);
    assert_eq!(AssetManager::free_balance(base_asset.into(), &EVE), 1000 * BASE);

    assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(CHARLIE), 0));
    assert_ok!(PredictionMarkets::redeem_shares(RuntimeOrigin::signed(EVE), 0));
    let market = &MarketCommons::market(&0).unwrap();
    let assets = market.outcome_assets();
    for asset in assets.iter() {
        assert_eq!(AssetManager::free_balance((*asset).into(), &CHARLIE), 0);
        assert_eq!(AssetManager::free_balance((*asset).into(), &EVE), 0);
    }
}
