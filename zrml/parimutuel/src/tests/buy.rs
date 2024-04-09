// Copyright 2023-2024 Forecasting Technologies LTD.
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

use crate::{mock::*, utils::*, *};
use core::ops::RangeInclusive;
use frame_support::{assert_noop, assert_ok};
use orml_traits::MultiCurrency;
use test_case::test_case;
use zeitgeist_primitives::types::{MarketStatus, MarketType, ParimutuelAsset, ScoringRule};
use zrml_market_commons::{Error as MError, Markets};

#[test]
fn buy_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset = ParimutuelAsset::Share(market_id, 0u16);
        let amount = 10 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount));

        let amount_minus_fees = 99000000000;
        let fees = 1000000000;
        assert_eq!(amount, amount_minus_fees + fees);

        System::assert_last_event(
            Event::OutcomeBought {
                market_id,
                buyer: ALICE,
                asset: asset.into(),
                amount_minus_fees,
                fees,
            }
            .into(),
        );
    });
}

#[test]
fn buy_balances_change_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market.clone());

        let base_asset = market.base_asset;

        let free_alice_before = AssetManager::free_balance(base_asset.into(), &ALICE);
        let free_creator_before = AssetManager::free_balance(base_asset.into(), &market.creator);
        let free_pot_before =
            AssetManager::free_balance(base_asset.into(), &Parimutuel::pot_account(market_id));

        let asset = ParimutuelAsset::Share(market_id, 0u16);
        let amount = 10 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount));

        let amount_minus_fees = 99000000000;
        let fees = 1000000000;
        assert_eq!(amount, amount_minus_fees + fees);

        assert_eq!(
            AssetManager::free_balance(base_asset.into(), &ALICE),
            free_alice_before - amount
        );
        assert_eq!(
            AssetManager::free_balance(base_asset.into(), &Parimutuel::pot_account(market_id))
                - free_pot_before,
            amount_minus_fees
        );
        assert_eq!(AssetManager::free_balance(asset.into(), &ALICE), amount_minus_fees);
        assert_eq!(
            AssetManager::free_balance(base_asset.into(), &market.creator) - free_creator_before,
            fees
        );
    });
}

#[test_case(ScoringRule::AmmCdaHybrid; "amm_cda_hybrid")]
fn buy_fails_if_invalid_scoring_rule(scoring_rule: ScoringRule) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Active;
        market.scoring_rule = scoring_rule;

        Markets::<Runtime>::insert(market_id, market.clone());

        let asset = ParimutuelAsset::Share(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get();
        assert_noop!(
            Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount),
            Error::<Runtime>::InvalidScoringRule
        );
    });
}

#[test_case(MarketStatus::Proposed; "proposed")]
#[test_case(MarketStatus::Closed; "closed")]
#[test_case(MarketStatus::Reported; "reported")]
#[test_case(MarketStatus::Disputed; "disputed")]
fn buy_fails_if_market_status_is_not_active(status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = status;
        market.scoring_rule = ScoringRule::Parimutuel;

        Markets::<Runtime>::insert(market_id, market.clone());

        let asset = ParimutuelAsset::Share(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get();
        assert_noop!(
            Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount),
            Error::<Runtime>::MarketIsNotActive
        );
    });
}

#[test]
fn buy_fails_if_market_type_is_scalar() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        let range: RangeInclusive<u128> = 0..=100;
        market.market_type = MarketType::Scalar(range);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset = ParimutuelAsset::Share(market_id, 0u16);
        let amount =
            <Runtime as Config>::MinBetSize::get() + <Runtime as Config>::MinBetSize::get();
        assert_noop!(
            Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount),
            Error::<Runtime>::NotCategorical
        );
    });
}

#[test]
fn buy_fails_if_insufficient_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market.clone());

        let free_alice = AssetManager::free_balance(market.base_asset.into(), &ALICE);
        AssetManager::slash(market.base_asset.into(), &ALICE, free_alice);

        let asset = ParimutuelAsset::Share(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get();
        assert_noop!(
            Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount),
            Error::<Runtime>::InsufficientBalance
        );
    });
}

#[test]
fn buy_fails_if_below_minimum_bet_size() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market.clone());

        let asset = ParimutuelAsset::Share(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get() - 1;
        assert_noop!(
            Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount),
            Error::<Runtime>::AmountBelowMinimumBetSize
        );
    });
}

#[test]
fn buy_fails_if_market_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;

        let asset = ParimutuelAsset::Share(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get();
        assert_noop!(
            Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount),
            MError::<Runtime>::MarketDoesNotExist
        );
    });
}
