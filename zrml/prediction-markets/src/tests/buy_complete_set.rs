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
fn it_allows_to_buy_a_complete_set() {
    let test = |base_asset: Asset<MarketId>| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        // Creates a permissionless market.
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::Lmsr,
        );

        // Allows someone to generate a complete set
        assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(BOB), 0, CENT));

        let market = MarketCommons::market(&0).unwrap();

        // Check the outcome balances
        let assets = PredictionMarkets::outcome_assets(0, &market);
        for asset in assets.iter() {
            let bal = Tokens::free_balance(*asset, &BOB);
            assert_eq!(bal, CENT);
        }

        let market_account = PredictionMarkets::market_account(0);
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
            ScoringRule::Lmsr,
        );
        assert_noop!(
            PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(BOB), 0, CENT),
            Error::<Runtime>::MarketIsNotActive,
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
            ScoringRule::Lmsr,
        );
        assert_noop!(
            PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(BOB), 0, 0),
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
            ScoringRule::Lmsr,
        );
        assert_noop!(
            PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(BOB), 0, 10000 * BASE),
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

#[test_case(MarketStatus::Proposed)]
#[test_case(MarketStatus::Closed)]
#[test_case(MarketStatus::Reported)]
#[test_case(MarketStatus::Disputed)]
#[test_case(MarketStatus::Resolved)]
fn buy_complete_set_fails_if_market_is_not_active(status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::Lmsr,
        );
        let market_id = 0;
        assert_ok!(MarketCommons::mutate_market(&market_id, |market| {
            market.status = status;
            Ok(())
        }));
        assert_noop!(
            PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(FRED), market_id, 1),
            Error::<Runtime>::MarketIsNotActive,
        );
    });
}

#[test_case(ScoringRule::Parimutuel)]
fn buy_complete_set_fails_if_market_has_wrong_scoring_rule(scoring_rule: ScoringRule) {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..2,
            scoring_rule,
        );
        let market_id = 0;
        assert_noop!(
            PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(FRED), market_id, 1),
            Error::<Runtime>::InvalidScoringRule,
        );
    });
}
