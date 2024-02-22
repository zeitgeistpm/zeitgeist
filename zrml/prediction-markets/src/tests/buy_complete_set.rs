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

// TODO(#1239) buy_complete_set fails if market doesn't exist

#[test]
fn buy_complete_set_works() {
    let test = |base_asset: BaseAsset| {
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::Lmsr,
        );

        let market_id = 0;
        let who = BOB;
        let amount = CENT;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(who),
            market_id,
            amount
        ));

        let market = MarketCommons::market(&market_id).unwrap();

        let assets = market.outcome_assets(market_id);
        for asset in assets.iter() {
            let bal = AssetManager::free_balance((*asset).into(), &who);
            assert_eq!(bal, amount);
        }

        let bal = AssetManager::free_balance(base_asset.into(), &who);
        assert_eq!(bal, 1_000 * BASE - amount);

        let market_account = PredictionMarkets::market_account(market_id);
        let market_bal = AssetManager::free_balance(base_asset.into(), &market_account);
        assert_eq!(market_bal, amount);
        System::assert_last_event(Event::BoughtCompleteSet(market_id, amount, who).into());
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
fn buy_complete_fails_on_zero_amount() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::Lmsr,
        );
        assert_noop!(
            PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(BOB), 0, 0),
            Error::<Runtime>::ZeroAmount
        );
    });
}

#[test]
fn buy_complete_set_fails_on_insufficient_balance() {
    let test = |base_asset: BaseAsset| {
        simple_create_categorical_market(
            base_asset,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::Lmsr,
        );
        assert_noop!(
            PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(BOB), 0, 10000 * BASE),
            Error::<Runtime>::NotEnoughBalance
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

#[test_case(MarketStatus::Proposed)]
#[test_case(MarketStatus::Closed)]
#[test_case(MarketStatus::Reported)]
#[test_case(MarketStatus::Disputed)]
#[test_case(MarketStatus::Resolved)]
fn buy_complete_set_fails_if_market_is_not_active(status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            BaseAsset::Ztg,
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
            BaseAsset::Ztg,
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
