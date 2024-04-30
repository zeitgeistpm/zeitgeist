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

#![cfg(test)]

use super::*;
use orml_traits::MultiCurrency;
use test_case::test_case;
use zeitgeist_primitives::types::{Amount, Balance};

fn test_helper(
    asset: Assets,
    initial_amount: <Runtime as crate::Config>::Balance,
    min_balance: <Runtime as crate::Config>::Balance,
) {
    assert_eq!(AssetRouter::minimum_balance(asset), min_balance);
    assert_ok!(AssetRouter::deposit(asset, &ALICE, initial_amount));
    assert_eq!(AssetRouter::total_issuance(asset), initial_amount);
    assert_eq!(AssetRouter::total_balance(asset, &ALICE), initial_amount);
    assert_eq!(AssetRouter::free_balance(asset, &ALICE), initial_amount);
    assert_ok!(AssetRouter::ensure_can_withdraw(asset, &ALICE, initial_amount));
    assert!(AssetRouter::ensure_can_withdraw(asset, &ALICE, initial_amount + 1).is_err());
    assert_ok!(AssetRouter::transfer(asset, &ALICE, &BOB, min_balance));
    assert_eq!(AssetRouter::free_balance(asset, &BOB), min_balance);
    assert_eq!(AssetRouter::free_balance(asset, &ALICE), initial_amount - min_balance);
    assert_ok!(AssetRouter::withdraw(asset, &ALICE, 1));
    assert_eq!(AssetRouter::free_balance(asset, &ALICE), initial_amount - min_balance - 1);
    assert!(AssetRouter::can_slash(asset, &ALICE, 1));
    assert_eq!(AssetRouter::slash(asset, &ALICE, 1), 0);
    assert_eq!(AssetRouter::free_balance(asset, &ALICE), initial_amount - min_balance - 2);
    assert_ok!(AssetRouter::update_balance(
        asset,
        &ALICE,
        <AssetRouter as MultiCurrencyExtended<AccountId>>::Amount::from(1u8)
            - <AssetRouter as MultiCurrencyExtended<AccountId>>::Amount::from(2u8)
    ));
    assert_eq!(AssetRouter::free_balance(asset, &ALICE), initial_amount - min_balance - 3);
}

#[test]
fn routes_campaign_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        use frame_support::traits::tokens::fungibles::Inspect;

        assert_ok!(AssetRouter::create(CAMPAIGN_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));

        test_helper(CAMPAIGN_ASSET, CAMPAIGN_ASSET_INITIAL_AMOUNT, CAMPAIGN_ASSET_MIN_BALANCE);

        assert_eq!(<CustomAssets as Inspect<AccountId>>::total_issuance(CUSTOM_ASSET_INTERNAL), 0);
        assert_eq!(<MarketAssets as Inspect<AccountId>>::total_issuance(MARKET_ASSET_INTERNAL), 0);
        assert_eq!(<Tokens as MultiCurrency<AccountId>>::total_issuance(CURRENCY_INTERNAL), 0);
    });
}

#[test]
fn routes_custom_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        use frame_support::traits::tokens::fungibles::Inspect;

        assert_ok!(AssetRouter::create(CUSTOM_ASSET, ALICE, true, CUSTOM_ASSET_MIN_BALANCE));

        test_helper(CUSTOM_ASSET, CUSTOM_ASSET_INITIAL_AMOUNT, CUSTOM_ASSET_MIN_BALANCE);

        assert_eq!(
            <CampaignAssets as Inspect<AccountId>>::total_issuance(CAMPAIGN_ASSET_INTERNAL),
            0
        );
        assert_eq!(<MarketAssets as Inspect<AccountId>>::total_issuance(MARKET_ASSET_INTERNAL), 0);
        assert_eq!(<Tokens as MultiCurrency<AccountId>>::total_issuance(CURRENCY_INTERNAL), 0);
    });
}

#[test]
fn routes_market_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        use frame_support::traits::tokens::fungibles::Inspect;

        assert_ok!(AssetRouter::create(MARKET_ASSET, ALICE, true, MARKET_ASSET_MIN_BALANCE));

        test_helper(MARKET_ASSET, MARKET_ASSET_INITIAL_AMOUNT, MARKET_ASSET_MIN_BALANCE);

        assert_eq!(
            <CampaignAssets as Inspect<AccountId>>::total_issuance(CAMPAIGN_ASSET_INTERNAL),
            0
        );
        assert_eq!(<CustomAssets as Inspect<AccountId>>::total_issuance(CUSTOM_ASSET_INTERNAL), 0);
        assert_eq!(<Tokens as MultiCurrency<AccountId>>::total_issuance(CURRENCY_INTERNAL), 0);
    });
}

#[test]
fn routes_currencies_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        use frame_support::traits::tokens::fungibles::Inspect;

        test_helper(CURRENCY, CURRENCY_INITIAL_AMOUNT, CURRENCY_MIN_BALANCE);
        test_helper(CURRENCY_OLD_OUTCOME, CURRENCY_INITIAL_AMOUNT, CURRENCY_MIN_BALANCE);

        assert_eq!(
            <CampaignAssets as Inspect<AccountId>>::total_issuance(CAMPAIGN_ASSET_INTERNAL),
            0
        );
        assert_eq!(<CustomAssets as Inspect<AccountId>>::total_issuance(CUSTOM_ASSET_INTERNAL), 0);
        assert_eq!(<MarketAssets as Inspect<AccountId>>::total_issuance(MARKET_ASSET_INTERNAL), 0);
    });
}

#[test_case(0, Some(0); "zero")]
#[test_case(Amount::MAX, Some(Amount::MAX.unsigned_abs() as Balance); "max")]
#[test_case(Amount::MIN, None; "min")]
#[test_case(Amount::MIN + 1, Some((Amount::MIN + 1).unsigned_abs() as Balance); "min_plus_one")]
fn update_balance_handles_overflows_correctly(update: Amount, expected: Option<Balance>) {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CAMPAIGN_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));

        if update.is_negative() {
            assert_ok!(AssetRouter::update_balance(CAMPAIGN_ASSET, &ALICE, Amount::MAX));
        }

        if let Some(expected_inner) = expected {
            assert_ok!(AssetRouter::update_balance(CAMPAIGN_ASSET, &ALICE, update));

            if update.is_negative() {
                assert_eq!(
                    AssetRouter::free_balance(CAMPAIGN_ASSET, &ALICE),
                    Amount::MAX as Balance - expected_inner
                );
            } else {
                assert_eq!(AssetRouter::free_balance(CAMPAIGN_ASSET, &ALICE), expected_inner);
            }
        } else {
            assert_noop!(
                AssetRouter::update_balance(CAMPAIGN_ASSET, &ALICE, update),
                Error::<Runtime>::AmountIntoBalanceFailed
            );
        }
    });
}
