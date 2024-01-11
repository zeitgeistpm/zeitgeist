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

#[test]
fn it_allows_to_sell_a_complete_set() {
    let test = |base_asset: AssetOf<Runtime>| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        // Creates a permissionless market.
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::Lmsr,
        );

        assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(BOB), 0, CENT));

        assert_ok!(PredictionMarkets::sell_complete_set(RuntimeOrigin::signed(BOB), 0, CENT));

        let market = MarketCommons::market(&0).unwrap();

        // Check the outcome balances
        let assets = PredictionMarkets::outcome_assets(0, &market);
        for asset in assets.iter() {
            let bal = Tokens::free_balance(*asset, &BOB);
            assert_eq!(bal, 0);
        }

        // also check native balance
        let bal = Balances::free_balance(BOB);
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
            ScoringRule::Lmsr,
        );
        assert_noop!(
            PredictionMarkets::sell_complete_set(RuntimeOrigin::signed(BOB), 0, 0),
            Error::<Runtime>::ZeroAmount
        );
    });
}

#[test]
fn it_does_not_allow_to_sell_complete_sets_with_insufficient_balance() {
    let test = |base_asset: AssetOf<Runtime>| {
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..1,
            ScoringRule::Lmsr,
        );
        assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(BOB), 0, 2 * CENT));
        assert_eq!(AssetManager::slash(Asset::CategoricalOutcome(0, 1), &BOB, CENT), 0);
        assert_noop!(
            PredictionMarkets::sell_complete_set(RuntimeOrigin::signed(BOB), 0, 2 * CENT),
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
            0..1,
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
