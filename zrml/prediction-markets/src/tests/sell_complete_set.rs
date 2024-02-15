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

// TODO(#1239) MarketDoesNotExist

#[test_case(ScoringRule::Lmsr)]
#[test_case(ScoringRule::Orderbook)]
fn sell_complete_set_works(scoring_rule: ScoringRule) {
    let test = |base_asset: AssetOf<Runtime>| {
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..2,
            scoring_rule,
        );
        let market_id = 0;
        let buy_amount = 5 * CENT;
        let sell_amount = 3 * CENT;
        let expected_amount = 2 * CENT;
        let who = BOB;

        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(who),
            market_id,
            buy_amount
        ));

        assert_ok!(PredictionMarkets::sell_complete_set(
            RuntimeOrigin::signed(who),
            market_id,
            sell_amount
        ));

        let market = MarketCommons::market(&market_id).unwrap();
        let assets = PredictionMarkets::outcome_assets(market_id, &market);
        for asset in assets.iter() {
            let bal = AssetManager::free_balance(*asset, &who);
            assert_eq!(bal, expected_amount);
        }

        let bal = AssetManager::free_balance(base_asset, &who);
        assert_eq!(bal, 1_000 * BASE - expected_amount);

        System::assert_last_event(Event::SoldCompleteSet(market_id, sell_amount, who).into());
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
fn sell_complete_set_fails_on_zero_amount() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::Lmsr,
        );
        assert_noop!(
            PredictionMarkets::sell_complete_set(RuntimeOrigin::signed(BOB), 0, 0),
            Error::<Runtime>::ZeroAmount
        );
    });
}

#[test]
fn sell_complete_set_fails_on_insufficient_share_balance() {
    let test = |base_asset: AssetOf<Runtime>| {
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::Lmsr,
        );
        let market_id = 0;
        let amount = 2 * CENT;
        let who = BOB;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(who),
            market_id,
            amount
        ));
        assert_eq!(AssetManager::slash(Asset::CategoricalOutcome(market_id, 1), &who, 1), 0);
        assert_noop!(
            PredictionMarkets::sell_complete_set(RuntimeOrigin::signed(who), market_id, amount),
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

#[test_case(ScoringRule::Parimutuel; "parimutuel")]
fn sell_complete_set_fails_if_market_has_wrong_scoring_rule(scoring_rule: ScoringRule) {
    let test = |base_asset: AssetOf<Runtime>| {
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..2,
            scoring_rule,
        );
        assert_noop!(
            PredictionMarkets::sell_complete_set(RuntimeOrigin::signed(BOB), 0, 2 * CENT),
            Error::<Runtime>::InvalidScoringRule
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
