// Copyright 2023 Forecasting Technologies LTD.
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

#![cfg(all(feature = "mock", test))]

use crate::{
    assert_approx,
    consts::*,
    mock::*,
    traits::{liquidity_shares_manager::LiquiditySharesManager, PoolOperations},
    BalanceOf, Config, Error, Event, Pallet, Pools, MAX_SPOT_PRICE, MIN_SPOT_PRICE, MIN_SWAP_FEE,
};
use frame_support::{assert_noop, assert_ok};
use orml_traits::MultiCurrency;
#[allow(unused_imports)]
use test_case::test_case;
use zeitgeist_primitives::{
    constants::CENT,
    math::fixed::{bdiv, bmul},
    types::{
        AccountIdTest, Asset, Deadlines, MarketCreation, MarketDisputeMechanism, MarketId,
        MarketPeriod, MarketStatus, MarketType, MultiHash, ScalarPosition, ScoringRule,
    },
};
use zrml_market_commons::{MarketCommonsPalletApi, Markets};

#[cfg(not(feature = "parachain"))]
const BASE_ASSET: Asset<MarketId> = Asset::Ztg;
#[cfg(feature = "parachain")]
const BASE_ASSET: Asset<MarketId> = FOREIGN_ASSET;

#[test]
fn deploy_pool_works_with_binary_markets() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let alice_before = AssetManager::free_balance(BASE_ASSET, &ALICE);
        let amount = _10;
        let spot_prices = vec![_1_2, _1_2];
        let swap_fee = CENT;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(2),
            amount,
            spot_prices.clone(),
            swap_fee,
        );
        let assets =
            vec![Asset::CategoricalOutcome(market_id, 0), Asset::CategoricalOutcome(market_id, 1)];
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let expected_liquidity = 144269504088;
        assert_eq!(pool.assets(), assets);
        assert_approx!(pool.liquidity_parameter, expected_liquidity, 1);
        assert_eq!(pool.collateral, BASE_ASSET);
        assert_eq!(pool.liquidity_shares_manager.owner, ALICE);
        assert_eq!(pool.liquidity_shares_manager.total_shares, amount);
        assert_eq!(pool.liquidity_shares_manager.fees, 0);
        assert_eq!(pool.swap_fee, swap_fee);
        assert_eq!(AssetManager::free_balance(assets[0], &pool.account_id), amount);
        assert_eq!(AssetManager::free_balance(assets[1], &pool.account_id), amount);
        assert_eq!(pool.reserve_of(&assets[0]).unwrap(), amount);
        assert_eq!(pool.reserve_of(&assets[1]).unwrap(), amount);
        assert_eq!(pool.calculate_spot_price(assets[0]).unwrap(), spot_prices[0]);
        assert_eq!(pool.calculate_spot_price(assets[1]).unwrap(), spot_prices[1]);
        assert_eq!(AssetManager::free_balance(BASE_ASSET, &ALICE), alice_before - amount);
        assert_eq!(AssetManager::free_balance(assets[0], &ALICE), 0);
        assert_eq!(AssetManager::free_balance(assets[1], &ALICE), 0);
        System::assert_last_event(
            Event::PoolDeployed {
                who: ALICE,
                market_id,
                pool_shares_amount: amount,
                amounts_in: vec![amount, amount],
                liquidity_parameter: pool.liquidity_parameter,
            }
            .into(),
        );
    });
}

#[test]
fn deploy_pool_works_with_scalar_marktes() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let alice_before = AssetManager::free_balance(BASE_ASSET, &ALICE);
        let amount = _100;
        let spot_prices = vec![_1_6, _5_6 + 1];
        let swap_fee = CENT;
        let market_id: MarketId = 0;
        let assets = vec![
            Asset::ScalarOutcome(market_id, ScalarPosition::Long),
            Asset::ScalarOutcome(market_id, ScalarPosition::Short),
        ];
        // Deploy some funds in the pool account to ensure that rogue funds don't screw up price
        // calculatings.
        let rogue_funds = _100;
        assert_ok!(AssetManager::deposit(
            assets[0],
            &Pallet::<Runtime>::pool_account_id(&market_id),
            rogue_funds
        ));
        let _ = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            amount,
            spot_prices.clone(),
            swap_fee,
        );
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let expected_liquidity = 558110626551;
        let expected_amounts = vec![amount, 101755598229];
        assert_eq!(pool.assets(), assets);
        assert_approx!(pool.liquidity_parameter, expected_liquidity, 1_000);
        assert_eq!(pool.collateral, BASE_ASSET);
        assert_eq!(pool.liquidity_shares_manager.owner, ALICE);
        assert_eq!(pool.liquidity_shares_manager.total_shares, amount);
        assert_eq!(pool.liquidity_shares_manager.fees, 0);
        assert_eq!(pool.swap_fee, swap_fee);
        assert_eq!(
            AssetManager::free_balance(assets[0], &pool.account_id),
            expected_amounts[0] + rogue_funds
        );
        assert_eq!(AssetManager::free_balance(assets[1], &pool.account_id), expected_amounts[1]);
        assert_eq!(pool.reserve_of(&assets[0]).unwrap(), expected_amounts[0]);
        assert_eq!(pool.reserve_of(&assets[1]).unwrap(), expected_amounts[1]);
        assert_eq!(pool.calculate_spot_price(assets[0]).unwrap(), spot_prices[0]);
        assert_eq!(pool.calculate_spot_price(assets[1]).unwrap(), spot_prices[1]);
        assert_eq!(AssetManager::free_balance(BASE_ASSET, &ALICE), alice_before - amount);
        assert_eq!(AssetManager::free_balance(assets[0], &ALICE), 0);
        assert_eq!(AssetManager::free_balance(assets[1], &ALICE), amount - expected_amounts[1]);
        let price_sum =
            pool.assets().iter().map(|&a| pool.calculate_spot_price(a).unwrap()).sum::<u128>();
        assert_eq!(price_sum, _1);
        System::assert_last_event(
            Event::PoolDeployed {
                who: ALICE,
                market_id,
                pool_shares_amount: amount,
                amounts_in: expected_amounts,
                liquidity_parameter: pool.liquidity_parameter,
            }
            .into(),
        );
    });
}

// Example taken from
// https://docs.gnosis.io/conditionaltokens/docs/introduction3/#an-example-with-lmsr
#[test]
fn buy_works() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let liquidity = _10;
        let spot_prices = vec![_1_2, _1_2];
        let swap_fee = CENT;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(2),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let total_fee_percentage = swap_fee + EXTERNAL_FEES;
        let amount_in_minus_fees = _10;
        let amount_in = bdiv(amount_in_minus_fees, _1 - total_fee_percentage).unwrap(); // This is exactly _10 after deducting fees.
        let expected_fees = amount_in - amount_in_minus_fees;
        let expected_swap_fee_amount = expected_fees / 2;
        let expected_external_fee_amount = expected_fees / 2;
        let pool_outcomes_before: Vec<_> =
            pool.assets().iter().map(|a| pool.reserve_of(a).unwrap()).collect();
        let pool_liquidity_before = pool.liquidity_parameter;
        let asset_out = pool.assets()[0];
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));
        // Deposit some stuff in the pool account to check that the pools `reserves` fields tracks
        // the reserve correctly.
        assert_ok!(AssetManager::deposit(asset_out, &pool.account_id, _100));
        assert_ok!(NeoSwaps::buy(
            RuntimeOrigin::signed(BOB),
            market_id,
            2,
            asset_out,
            amount_in,
            0,
        ));
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        assert_eq!(pool.liquidity_parameter, pool_liquidity_before);
        assert_eq!(pool.liquidity_shares_manager.owner, ALICE);
        assert_eq!(pool.liquidity_shares_manager.total_shares, liquidity);
        assert_eq!(pool.liquidity_shares_manager.fees, expected_swap_fee_amount);
        let pool_outcomes_after: Vec<_> = pool
            .assets()
            .iter()
            .map(|a| pool.reserve_of(a).unwrap())
            .collect();
        let expected_swap_amount_out = 58496250072;
        let expected_amount_in_minus_fees = _10 + 1; // Note: This is 1 Pennock of the correct result.
        let expected_amount_out = expected_swap_amount_out + expected_amount_in_minus_fees;
        assert_eq!(AssetManager::free_balance(BASE_ASSET, &BOB), 0);
        assert_eq!(AssetManager::free_balance(asset_out, &BOB), expected_amount_out);
        assert_eq!(pool_outcomes_after[0], pool_outcomes_before[0] - expected_swap_amount_out);
        assert_eq!(
            pool_outcomes_after[1],
            pool_outcomes_before[0] + expected_amount_in_minus_fees,
        );
        let expected_pool_account_balance = if pool.collateral == Asset::Ztg {
            expected_swap_fee_amount + AssetManager::minimum_balance(pool.collateral)
        } else {
            expected_swap_fee_amount
        };
        assert_eq!(
            AssetManager::free_balance(BASE_ASSET, &pool.account_id),
            expected_pool_account_balance
        );
        assert_eq!(
            AssetManager::free_balance(BASE_ASSET, &FEE_ACCOUNT),
            expected_external_fee_amount
        );
        let price_sum = pool
            .assets()
            .iter()
            .map(|&a| pool.calculate_spot_price(a).unwrap())
            .sum::<u128>();
        assert_eq!(price_sum, _1);
        System::assert_last_event(
            Event::BuyExecuted {
                who: BOB,
                market_id,
                asset_out,
                amount_in,
                amount_out: expected_amount_out,
                swap_fee_amount: expected_swap_fee_amount,
                external_fee_amount: expected_external_fee_amount,
            }
            .into(),
        );
    });
}

#[test]
fn sell_works() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let liquidity = _10;
        let spot_prices = vec![_1_4, _3_4];
        let swap_fee = CENT;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let amount_in = _10;
        let pool_outcomes_before: Vec<_> =
            pool.assets().iter().map(|a| pool.reserve_of(a).unwrap()).collect();
        let pool_liquidity_before = pool.liquidity_parameter;
        AssetManager::deposit(BASE_ASSET, &BOB, amount_in).unwrap();
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(BOB),
            market_id,
            amount_in,
        ));
        let asset_in = pool.assets()[1];
        assert_ok!(NeoSwaps::sell(
            RuntimeOrigin::signed(BOB),
            market_id,
            2,
            asset_in,
            amount_in,
            0,
        ));
        let total_fee_percentage = swap_fee + EXTERNAL_FEES;
        let expected_amount_out = 59632253897u128;
        let expected_fees = bmul(total_fee_percentage, expected_amount_out).unwrap();
        let expected_swap_fee_amount = expected_fees / 2;
        let expected_external_fee_amount = expected_fees - expected_swap_fee_amount;
        let expected_amount_out_minus_fees = expected_amount_out - expected_fees;
        assert_eq!(AssetManager::free_balance(BASE_ASSET, &BOB), expected_amount_out_minus_fees);
        assert_eq!(AssetManager::free_balance(asset_in, &BOB), 0);
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        assert_eq!(pool.liquidity_parameter, pool_liquidity_before);
        assert_eq!(pool.liquidity_shares_manager.owner, ALICE);
        assert_eq!(pool.liquidity_shares_manager.total_shares, liquidity);
        assert_eq!(pool.liquidity_shares_manager.fees, expected_swap_fee_amount);
        let pool_outcomes_after: Vec<_> =
            pool.assets().iter().map(|a| pool.reserve_of(a).unwrap()).collect();
        assert_eq!(pool_outcomes_after[0], pool_outcomes_before[0] - expected_amount_out);
        assert_eq!(
            pool_outcomes_after[1],
            pool_outcomes_before[1] + (amount_in - expected_amount_out)
        );
        let expected_pool_account_balance = if pool.collateral == Asset::Ztg {
            expected_swap_fee_amount + AssetManager::minimum_balance(Asset::Ztg)
        } else {
            expected_swap_fee_amount
        };
        assert_eq!(
            AssetManager::free_balance(BASE_ASSET, &pool.account_id),
            expected_pool_account_balance
        );
        assert_eq!(
            AssetManager::free_balance(BASE_ASSET, &FEE_ACCOUNT),
            expected_external_fee_amount
        );
        assert_eq!(
            AssetManager::total_issuance(pool.assets()[0]),
            liquidity + amount_in - expected_amount_out
        );
        assert_eq!(
            AssetManager::total_issuance(pool.assets()[1]),
            liquidity + amount_in - expected_amount_out
        );
        let price_sum =
            pool.assets().iter().map(|&a| pool.calculate_spot_price(a).unwrap()).sum::<u128>();
        assert_eq!(price_sum, _1);
        System::assert_last_event(
            Event::SellExecuted {
                who: BOB,
                market_id,
                asset_in,
                amount_in,
                amount_out: expected_amount_out,
                swap_fee_amount: expected_swap_fee_amount,
                external_fee_amount: expected_external_fee_amount,
            }
            .into(),
        );
    });
}

#[test]
fn join_works() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let liquidity = _10;
        let spot_prices = vec![_1_6, _5_6 + 1];
        let swap_fee = CENT;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        let pool_shares_amount = _4; // Add 40% to the pool.
        assert_ok!(AssetManager::deposit(BASE_ASSET, &ALICE, pool_shares_amount));
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            pool_shares_amount,
        ));
        let pool_before = Pools::<Runtime>::get(market_id).unwrap();
        let alice_long_before = AssetManager::free_balance(pool_before.assets()[1], &ALICE);
        let pool_outcomes_before: Vec<_> =
            pool_before.assets().iter().map(|a| pool_before.reserve_of(a).unwrap()).collect();
        assert_ok!(NeoSwaps::join(
            RuntimeOrigin::signed(ALICE),
            market_id,
            pool_shares_amount,
            vec![u128::MAX, u128::MAX],
        ));
        let pool_after = Pools::<Runtime>::get(market_id).unwrap();
        let ratio = bdiv(liquidity + pool_shares_amount, liquidity).unwrap();
        let pool_outcomes_after: Vec<_> =
            pool_after.assets().iter().map(|a| pool_after.reserve_of(a).unwrap()).collect();
        assert_eq!(pool_outcomes_after[0], bmul(ratio, pool_outcomes_before[0]).unwrap());
        assert_eq!(pool_outcomes_after[1], bmul(ratio, pool_outcomes_before[1]).unwrap());
        let long_diff = pool_outcomes_after[1] - pool_outcomes_before[1];
        assert_eq!(AssetManager::free_balance(pool_after.assets()[0], &ALICE), 0);
        assert_eq!(
            AssetManager::free_balance(pool_after.assets()[1], &ALICE),
            alice_long_before - long_diff
        );
        assert_eq!(
            pool_after.liquidity_parameter,
            bmul(ratio, pool_before.liquidity_parameter).unwrap()
        );
        System::assert_last_event(
            Event::JoinExecuted {
                who: ALICE,
                market_id,
                pool_shares_amount,
                amounts_in: vec![pool_shares_amount, long_diff],
                new_liquidity_parameter: pool_after.liquidity_parameter,
            }
            .into(),
        );
    });
}

#[test]
fn exit_works() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let liquidity = _10;
        let spot_prices = vec![_1_6, _5_6 + 1];
        let swap_fee = CENT;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        let pool_shares_amount = _4; // Remove 40% to the pool.
        let pool_before = Pools::<Runtime>::get(market_id).unwrap();
        let alice_outcomes_before = [
            AssetManager::free_balance(pool_before.assets()[0], &ALICE),
            AssetManager::free_balance(pool_before.assets()[1], &ALICE),
        ];
        let pool_outcomes_before: Vec<_> =
            pool_before.assets().iter().map(|a| pool_before.reserve_of(a).unwrap()).collect();
        assert_ok!(NeoSwaps::exit(
            RuntimeOrigin::signed(ALICE),
            market_id,
            pool_shares_amount,
            vec![0, 0],
        ));
        let pool_after = Pools::<Runtime>::get(market_id).unwrap();
        let ratio = bdiv(pool_shares_amount, liquidity).unwrap();
        let pool_outcomes_after: Vec<_> =
            pool_after.assets().iter().map(|a| pool_after.reserve_of(a).unwrap()).collect();
        let expected_pool_diff = vec![
            bmul(ratio, pool_outcomes_before[0]).unwrap(),
            bmul(ratio, pool_outcomes_before[1]).unwrap(),
        ];
        let alice_outcomes_after = [
            AssetManager::free_balance(pool_after.assets()[0], &ALICE),
            AssetManager::free_balance(pool_after.assets()[1], &ALICE),
        ];
        assert_eq!(pool_outcomes_after[0], pool_outcomes_before[0] - expected_pool_diff[0]);
        assert_eq!(pool_outcomes_after[1], pool_outcomes_before[1] - expected_pool_diff[1]);
        assert_eq!(alice_outcomes_after[0], alice_outcomes_before[0] + expected_pool_diff[0]);
        assert_eq!(alice_outcomes_after[1], alice_outcomes_before[1] + expected_pool_diff[1]);
        assert_eq!(
            pool_after.liquidity_parameter,
            bmul(_1 - ratio, pool_before.liquidity_parameter).unwrap()
        );
        System::assert_last_event(
            Event::ExitExecuted {
                who: ALICE,
                market_id,
                pool_shares_amount,
                amounts_out: expected_pool_diff,
                new_liquidity_parameter: pool_after.liquidity_parameter,
            }
            .into(),
        );
    });
}

#[test]
fn exit_destroys_pool() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let liquidity = _10;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            liquidity,
            vec![_1_6, _5_6 + 1],
            CENT,
        );
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let amounts_out = vec![
            pool.reserve_of(&pool.assets()[0]).unwrap(),
            pool.reserve_of(&pool.assets()[1]).unwrap(),
        ];
        let alice_before = [
            AssetManager::free_balance(pool.assets()[0], &ALICE),
            AssetManager::free_balance(pool.assets()[1], &ALICE),
        ];
        assert_ok!(NeoSwaps::exit(RuntimeOrigin::signed(ALICE), market_id, liquidity, vec![0, 0]));
        assert!(!Pools::<Runtime>::contains_key(market_id));
        assert_eq!(AssetManager::free_balance(pool.assets()[0], &pool.account_id), 0);
        assert_eq!(AssetManager::free_balance(pool.assets()[1], &pool.account_id), 0);
        assert_eq!(
            AssetManager::free_balance(pool.assets()[0], &ALICE),
            alice_before[0] + amounts_out[0]
        );
        assert_eq!(
            AssetManager::free_balance(pool.assets()[1], &ALICE),
            alice_before[1] + amounts_out[1]
        );
        System::assert_last_event(
            Event::PoolDestroyed {
                who: ALICE,
                market_id,
                pool_shares_amount: liquidity,
                amounts_out,
            }
            .into(),
        );
    });
}

#[test]
fn withdraw_fees_works() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let liquidity = _10;
        let spot_prices = vec![_1_6, _5_6 + 1];
        let swap_fee = CENT;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        // Mock up some fees for Alice to withdraw.
        let mut pool = Pools::<Runtime>::get(market_id).unwrap();
        let fees = 123456789;
        assert_ok!(AssetManager::deposit(pool.collateral, &pool.account_id, fees));
        pool.liquidity_shares_manager.fees = fees;
        Pools::<Runtime>::insert(market_id, pool.clone());
        let alice_before = AssetManager::free_balance(pool.collateral, &ALICE);
        assert_ok!(NeoSwaps::withdraw_fees(RuntimeOrigin::signed(ALICE), market_id));
        let expected_pool_account_balance = if pool.collateral == Asset::Ztg {
            AssetManager::minimum_balance(pool.collateral)
        } else {
            0
        };
        assert_eq!(
            AssetManager::free_balance(pool.collateral, &pool.account_id),
            expected_pool_account_balance
        );
        assert_eq!(AssetManager::free_balance(pool.collateral, &ALICE), alice_before + fees);
        let pool_after = Pools::<Runtime>::get(market_id).unwrap();
        assert_eq!(pool_after.liquidity_shares_manager.fees, 0);
        System::assert_last_event(
            Event::FeesWithdrawn { who: ALICE, market_id, amount: fees }.into(),
        );
    });
}

#[test]
fn buy_fails_on_incorrect_asset_count() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        assert_noop!(
            NeoSwaps::buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                1,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                _1,
                0
            ),
            Error::<Runtime>::IncorrectAssetCount
        );
    });
}

#[test]
fn sell_fails_on_incorrect_asset_count() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        assert_noop!(
            NeoSwaps::sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                1,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                _1,
                0
            ),
            Error::<Runtime>::IncorrectAssetCount
        );
    });
}

#[test]
fn join_fails_on_incorrect_vec_len() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        assert_noop!(
            NeoSwaps::join(RuntimeOrigin::signed(ALICE), market_id, _1, vec![0]),
            Error::<Runtime>::IncorrectVecLen
        );
    });
}

#[test]
fn exit_fails_on_incorrect_vec_len() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        assert_noop!(
            NeoSwaps::exit(RuntimeOrigin::signed(ALICE), market_id, _1, vec![0]),
            Error::<Runtime>::IncorrectVecLen
        );
    });
}

#[test]
fn deploy_pool_fails_on_incorrect_vec_len() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::Lmsr);
        assert_noop!(
            NeoSwaps::deploy_pool(RuntimeOrigin::signed(ALICE), market_id, _10, vec![_1_3], CENT),
            Error::<Runtime>::IncorrectVecLen
        );
    });
}

#[test]
fn buy_fails_on_market_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        Markets::<Runtime>::remove(market_id);
        assert_noop!(
            NeoSwaps::buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                _1,
                0
            ),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist,
        );
    });
}

#[test]
fn sell_fails_on_market_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        Markets::<Runtime>::remove(market_id);
        assert_noop!(
            NeoSwaps::sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                _1,
                0
            ),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist,
        );
    });
}

#[test]
fn join_fails_on_market_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        Markets::<Runtime>::remove(market_id);
        assert_noop!(
            NeoSwaps::join(RuntimeOrigin::signed(ALICE), market_id, _1, vec![u128::MAX, u128::MAX]),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn exit_fails_on_market_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        Markets::<Runtime>::remove(market_id);
        assert_noop!(
            NeoSwaps::exit(RuntimeOrigin::signed(ALICE), market_id, _1, vec![0, 0]),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn deploy_pool_fails_on_market_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            NeoSwaps::deploy_pool(RuntimeOrigin::signed(ALICE), 0, _10, vec![_1_4, _3_4], CENT),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist,
        );
    });
}

#[test_case(MarketStatus::Proposed)]
#[test_case(MarketStatus::Suspended)]
#[test_case(MarketStatus::Closed)]
#[test_case(MarketStatus::CollectingSubsidy)]
#[test_case(MarketStatus::InsufficientSubsidy)]
#[test_case(MarketStatus::Reported)]
#[test_case(MarketStatus::Disputed)]
#[test_case(MarketStatus::Resolved)]
fn buy_fails_on_inactive_market(market_status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        })
        .unwrap();
        assert_noop!(
            NeoSwaps::buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                _1,
                0
            ),
            Error::<Runtime>::MarketNotActive,
        );
    });
}

#[test_case(MarketStatus::Proposed)]
#[test_case(MarketStatus::Suspended)]
#[test_case(MarketStatus::Closed)]
#[test_case(MarketStatus::CollectingSubsidy)]
#[test_case(MarketStatus::InsufficientSubsidy)]
#[test_case(MarketStatus::Reported)]
#[test_case(MarketStatus::Disputed)]
#[test_case(MarketStatus::Resolved)]
fn sell_fails_on_inactive_market(market_status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        })
        .unwrap();
        assert_noop!(
            NeoSwaps::sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                _1,
                0
            ),
            Error::<Runtime>::MarketNotActive,
        );
    });
}

#[test_case(MarketStatus::Proposed)]
#[test_case(MarketStatus::Suspended)]
#[test_case(MarketStatus::Closed)]
#[test_case(MarketStatus::CollectingSubsidy)]
#[test_case(MarketStatus::InsufficientSubsidy)]
#[test_case(MarketStatus::Reported)]
#[test_case(MarketStatus::Disputed)]
#[test_case(MarketStatus::Resolved)]
fn join_fails_on_inactive_market(market_status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        })
        .unwrap();
        assert_noop!(
            NeoSwaps::join(RuntimeOrigin::signed(BOB), market_id, _1, vec![u128::MAX, u128::MAX]),
            Error::<Runtime>::MarketNotActive,
        );
    });
}

#[test_case(MarketStatus::Proposed)]
#[test_case(MarketStatus::Suspended)]
#[test_case(MarketStatus::Closed)]
#[test_case(MarketStatus::CollectingSubsidy)]
#[test_case(MarketStatus::InsufficientSubsidy)]
#[test_case(MarketStatus::Reported)]
#[test_case(MarketStatus::Disputed)]
#[test_case(MarketStatus::Resolved)]
fn deploy_pool_fails_on_inactive_market(market_status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::Lmsr);
        MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        })
        .unwrap();
        assert_noop!(
            NeoSwaps::deploy_pool(
                RuntimeOrigin::signed(ALICE),
                market_id,
                _1,
                vec![_1_2, _1_2],
                CENT,
            ),
            Error::<Runtime>::MarketNotActive,
        );
    });
}

#[test]
fn buy_fails_on_pool_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::Lmsr);
        assert_noop!(
            NeoSwaps::buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                _1,
                0
            ),
            Error::<Runtime>::PoolNotFound,
        );
    });
}

#[test]
fn sell_fails_on_pool_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::Lmsr);
        assert_noop!(
            NeoSwaps::sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                _1,
                0
            ),
            Error::<Runtime>::PoolNotFound,
        );
    });
}

#[test]
fn join_fails_on_pool_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::Lmsr);
        assert_noop!(
            NeoSwaps::join(
                RuntimeOrigin::signed(ALICE),
                market_id,
                _1,
                vec![u128::MAX, u128::MAX],
            ),
            Error::<Runtime>::PoolNotFound,
        );
    });
}

#[test]
fn exit_fails_on_pool_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::Lmsr);
        assert_noop!(
            NeoSwaps::exit(RuntimeOrigin::signed(ALICE), market_id, _1, vec![0, 0]),
            Error::<Runtime>::PoolNotFound,
        );
    });
}

#[test]
fn withdraw_fees_fails_on_pool_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::Lmsr);
        assert_noop!(
            NeoSwaps::withdraw_fees(RuntimeOrigin::signed(ALICE), market_id),
            Error::<Runtime>::PoolNotFound
        );
    });
}

#[test]
fn deploy_pool_fails_on_duplicate_pool() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        assert_noop!(
            NeoSwaps::deploy_pool(
                RuntimeOrigin::signed(ALICE),
                market_id,
                _2,
                vec![_1_2, _1_2],
                CENT,
            ),
            Error::<Runtime>::DuplicatePool,
        );
    });
}

#[test_case(MarketType::Categorical(2))]
#[test_case(MarketType::Scalar(0..=1))]
fn buy_fails_on_asset_not_found(market_type: MarketType) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            market_type,
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        assert_noop!(
            NeoSwaps::buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::CategoricalOutcome(market_id, 2),
                _1,
                0
            ),
            Error::<Runtime>::AssetNotFound,
        );
    });
}

#[test_case(MarketType::Categorical(2))]
#[test_case(MarketType::Scalar(0..=1))]
fn sell_fails_on_asset_not_found(market_type: MarketType) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            market_type,
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        assert_noop!(
            NeoSwaps::sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::CategoricalOutcome(market_id, 2),
                _1,
                u128::MAX,
            ),
            Error::<Runtime>::AssetNotFound,
        );
    });
}

#[test]
fn buy_fails_on_numerical_limits() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let amount_in = 100 * pool.liquidity_parameter;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));
        assert_noop!(
            NeoSwaps::buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                amount_in,
                0,
            ),
            Error::<Runtime>::NumericalLimits,
        );
    });
}

#[test]
fn sell_fails_on_numerical_limits() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let asset_in = Asset::ScalarOutcome(market_id, ScalarPosition::Long);
        let amount_in = 100 * pool.liquidity_parameter;
        assert_ok!(AssetManager::deposit(asset_in, &BOB, amount_in));
        assert_noop!(
            NeoSwaps::buy(RuntimeOrigin::signed(BOB), market_id, 2, asset_in, amount_in, 0),
            Error::<Runtime>::NumericalLimits,
        );
    });
}

#[test]
fn buy_fails_on_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        let amount_in = _10;
        #[cfg(not(feature = "parachain"))]
        let expected_error = pallet_balances::Error::<Runtime>::InsufficientBalance;
        #[cfg(feature = "parachain")]
        let expected_error = orml_tokens::Error::<Runtime>::BalanceTooLow;
        println!("{:?}", expected_error);
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in - 1));
        assert_noop!(
            NeoSwaps::buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                amount_in,
                0,
            ),
            expected_error,
        );
    });
}

#[test]
fn sell_fails_on_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        let amount_in = _10;
        let asset_in = Asset::ScalarOutcome(market_id, ScalarPosition::Long);
        assert_ok!(AssetManager::deposit(asset_in, &BOB, amount_in - 1));
        assert_noop!(
            NeoSwaps::sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                asset_in,
                amount_in,
                u128::MAX,
            ),
            orml_tokens::Error::<Runtime>::BalanceTooLow,
        );
    });
}

#[test]
fn join_fails_on_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        assert_noop!(
            NeoSwaps::join(
                RuntimeOrigin::signed(ALICE),
                market_id,
                _100,
                vec![u128::MAX, u128::MAX]
            ),
            orml_tokens::Error::<Runtime>::BalanceTooLow
        );
    });
}

#[test]
fn exit_fails_on_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            liquidity,
            vec![_1_2, _1_2],
            CENT,
        );
        assert_noop!(
            NeoSwaps::exit(
                RuntimeOrigin::signed(ALICE),
                market_id,
                liquidity + 1, // One more than Alice has.
                vec![0, 0]
            ),
            Error::<Runtime>::InsufficientPoolShares,
        );
    });
}

#[test]
fn buy_fails_on_amount_out_below_min() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        let amount_in = _1;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));
        // Buying 1 at price of .5 will return less than 2 outcomes due to slippage.
        assert_noop!(
            NeoSwaps::buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                amount_in,
                _2,
            ),
            Error::<Runtime>::AmountOutBelowMin,
        );
    });
}

#[test]
fn sell_fails_on_amount_out_below_min() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _100,
            vec![_1_2, _1_2],
            CENT,
        );
        let amount_in = _20;
        let asset_in = Asset::ScalarOutcome(market_id, ScalarPosition::Long);
        assert_ok!(AssetManager::deposit(asset_in, &BOB, amount_in));
        // Selling 20 at price of .5 will return less than 10 dollars due to slippage.
        assert_noop!(
            NeoSwaps::sell(RuntimeOrigin::signed(BOB), market_id, 2, asset_in, amount_in, _10),
            Error::<Runtime>::AmountOutBelowMin,
        );
    });
}

#[test]
fn join_fails_on_amount_in_above_max() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _20,
            vec![_1_2, _1_2],
            CENT,
        );
        let pool_shares_amount = _10;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            pool_shares_amount,
        ));
        assert_noop!(
            NeoSwaps::join(
                RuntimeOrigin::signed(ALICE),
                market_id,
                pool_shares_amount,
                vec![pool_shares_amount - 1, pool_shares_amount]
            ),
            Error::<Runtime>::AmountInAboveMax
        );
    });
}

#[test]
fn exit_fails_on_amount_out_below_min() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _20,
            vec![_1_2, _1_2],
            CENT,
        );
        let pool_shares_amount = _5;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            pool_shares_amount,
        ));
        assert_noop!(
            NeoSwaps::exit(
                RuntimeOrigin::signed(ALICE),
                market_id,
                pool_shares_amount,
                vec![pool_shares_amount + 1, pool_shares_amount]
            ),
            Error::<Runtime>::AmountOutBelowMin
        );
    });
}

#[test]
fn join_fails_if_not_allowed() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _20,
            vec![_1_2, _1_2],
            CENT,
        );
        let pool_shares_amount = _5;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, pool_shares_amount));
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(BOB),
            market_id,
            pool_shares_amount,
        ));
        assert_noop!(
            NeoSwaps::join(
                RuntimeOrigin::signed(BOB),
                market_id,
                pool_shares_amount,
                vec![pool_shares_amount, pool_shares_amount]
            ),
            Error::<Runtime>::NotAllowed
        );
    });
}

#[test]
fn exit_fails_if_not_allowed() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _20,
            vec![_1_2, _1_2],
            CENT,
        );
        let pool_shares_amount = _5;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, pool_shares_amount));
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(BOB),
            market_id,
            pool_shares_amount,
        ));
        assert_noop!(
            NeoSwaps::exit(
                RuntimeOrigin::signed(BOB),
                market_id,
                pool_shares_amount,
                vec![pool_shares_amount, pool_shares_amount]
            ),
            Error::<Runtime>::NotAllowed
        );
    });
}

#[test]
fn exit_fails_on_outstanding_fees() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _20,
            vec![_1_2, _1_2],
            CENT,
        );
        let pool_shares_amount = _20;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, pool_shares_amount));
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(BOB),
            market_id,
            pool_shares_amount,
        ));
        assert_ok!(Pools::<Runtime>::try_mutate(market_id, |pool| pool
            .as_mut()
            .unwrap()
            .liquidity_shares_manager
            .deposit_fees(1)));
        assert_noop!(
            NeoSwaps::exit(
                RuntimeOrigin::signed(BOB),
                market_id,
                pool_shares_amount,
                vec![pool_shares_amount, pool_shares_amount]
            ),
            Error::<Runtime>::OutstandingFees
        );
    });
}

#[test]
fn deploy_pool_fails_on_not_allowed() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::Lmsr);
        assert_noop!(
            NeoSwaps::deploy_pool(
                RuntimeOrigin::signed(BOB),
                market_id,
                _10,
                vec![_1_4, _3_4],
                CENT
            ),
            Error::<Runtime>::NotAllowed
        );
    });
}

#[test]
fn deploy_pool_fails_on_invalid_trading_mechanism() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::CPMM);
        assert_noop!(
            NeoSwaps::deploy_pool(
                RuntimeOrigin::signed(ALICE),
                market_id,
                _10,
                vec![_1_4, _3_4],
                CENT
            ),
            Error::<Runtime>::InvalidTradingMechanism
        );
    });
}

#[test]
fn deploy_pool_fails_on_market_is_not_binary_or_scalar() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(3), ScoringRule::Lmsr);
        let liquidity = _10;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            liquidity,
        ));
        assert_noop!(
            NeoSwaps::deploy_pool(
                RuntimeOrigin::signed(ALICE),
                market_id,
                liquidity,
                vec![_1_3, _1_3, _1_3],
                CENT
            ),
            Error::<Runtime>::MarketNotBinaryOrScalar
        );
    });
}

// FIXME This test currently fails because the `ensure!` throwing `AssetCountAboveMax` is
// currently unreachable if the market is not binary/scalar.
#[test]
#[should_panic]
fn deploy_pool_fails_on_asset_count_above_max() {
    ExtBuilder::default().build().execute_with(|| {
        let category_count = MAX_ASSETS + 1;
        let market_id = create_market(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(category_count),
            ScoringRule::Lmsr,
        );
        let liquidity = _10;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            liquidity,
        ));
        // Depending on the value of MAX_ASSETS and PRICE_BARRIER_*, this `spot_prices` vector
        // might violate some other rules for deploying pools.
        let mut spot_prices = vec![_1 / category_count as u128; category_count as usize - 1];
        spot_prices.push(_1 - spot_prices.iter().sum::<u128>());
        assert_noop!(
            NeoSwaps::deploy_pool(
                RuntimeOrigin::signed(ALICE),
                market_id,
                liquidity,
                spot_prices,
                CENT
            ),
            Error::<Runtime>::AssetCountAboveMax
        );
    });
}

#[test]
fn deploy_pool_fails_on_swap_fee_below_min() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::Lmsr);
        let liquidity = _10;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            liquidity,
        ));
        assert_noop!(
            NeoSwaps::deploy_pool(
                RuntimeOrigin::signed(ALICE),
                market_id,
                liquidity,
                vec![_1_4, _3_4],
                MIN_SWAP_FEE - 1,
            ),
            Error::<Runtime>::SwapFeeBelowMin
        );
    });
}

#[test]
fn deploy_pool_fails_on_swap_fee_above_max() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::Lmsr);
        let liquidity = _10;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            liquidity,
        ));
        assert_noop!(
            NeoSwaps::deploy_pool(
                RuntimeOrigin::signed(ALICE),
                market_id,
                liquidity,
                vec![_1_4, _3_4],
                <Runtime as Config>::MaxSwapFee::get() + 1,
            ),
            Error::<Runtime>::SwapFeeAboveMax
        );
    });
}

#[test_case(vec![_1_4, _3_4 - 1])]
#[test_case(vec![_1_4 + 1, _3_4])]
fn deploy_pool_fails_on_invalid_spot_prices(spot_prices: Vec<BalanceOf<Runtime>>) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::Lmsr);
        let liquidity = _10;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            liquidity,
        ));
        assert_noop!(
            NeoSwaps::deploy_pool(
                RuntimeOrigin::signed(ALICE),
                market_id,
                liquidity,
                spot_prices,
                CENT
            ),
            Error::<Runtime>::InvalidSpotPrices
        );
    });
}

#[test]
fn buy_fails_on_spot_price_above_max() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(2),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        assert_noop!(
            NeoSwaps::buy(
                RuntimeOrigin::signed(ALICE),
                market_id,
                2,
                Asset::CategoricalOutcome(market_id, 0),
                _70,
                0,
            ),
            Error::<Runtime>::SpotPriceAboveMax
        );
    });
}

#[test]
fn sell_fails_on_spot_price_below_min() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(2),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        let asset_in = Asset::CategoricalOutcome(market_id, 0);
        let amount_in = _80;
        assert_ok!(AssetManager::deposit(asset_in, &BOB, amount_in));
        assert_noop!(
            NeoSwaps::sell(RuntimeOrigin::signed(BOB), market_id, 2, asset_in, amount_in, 0),
            Error::<Runtime>::SpotPriceBelowMin
        );
    });
}

#[test]
fn deploy_pool_fails_on_spot_price_below_min() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::Lmsr);
        let liquidity = _10;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            liquidity,
        ));
        let spot_price = MIN_SPOT_PRICE - 1;
        assert_noop!(
            NeoSwaps::deploy_pool(
                RuntimeOrigin::signed(ALICE),
                market_id,
                liquidity,
                vec![spot_price, _1 - spot_price],
                CENT
            ),
            Error::<Runtime>::SpotPriceBelowMin
        );
    });
}

#[test]
fn deploy_pool_fails_on_spot_price_above_max() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::Lmsr);
        let liquidity = _10;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            liquidity,
        ));
        let spot_price = MAX_SPOT_PRICE + 1;
        assert_noop!(
            NeoSwaps::deploy_pool(
                RuntimeOrigin::signed(ALICE),
                market_id,
                liquidity,
                vec![spot_price, _1 - spot_price],
                CENT
            ),
            Error::<Runtime>::SpotPriceAboveMax
        );
    });
}

#[test]
fn deploy_pool_fails_on_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::Lmsr);
        let liquidity = _10;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            liquidity - 1,
        ));
        assert_noop!(
            NeoSwaps::deploy_pool(
                RuntimeOrigin::signed(ALICE),
                market_id,
                liquidity,
                vec![_3_4, _1_4],
                CENT
            ),
            orml_tokens::Error::<Runtime>::BalanceTooLow
        );
    });
}

#[test]
fn exit_pool_fails_on_liquidity_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        // Will result in liquidity of about 0.7213475204444817.
        assert_noop!(
            NeoSwaps::exit(RuntimeOrigin::signed(ALICE), market_id, _10 - _1_2, vec![0, 0]),
            Error::<Runtime>::LiquidityTooLow
        );
    });
}

#[test]
fn deploy_pool_fails_on_liquidity_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::Lmsr);
        let amount = _1_2;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            amount,
        ));
        assert_noop!(
            NeoSwaps::deploy_pool(
                RuntimeOrigin::signed(ALICE),
                market_id,
                amount,
                vec![_1_2, _1_2],
                CENT
            ),
            Error::<Runtime>::LiquidityTooLow
        );
    });
}

#[test]
fn split_is_noop() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            NeoSwaps::split(RuntimeOrigin::signed(ALICE), 0, BOB, _1),
            Error::<Runtime>::NotImplemented
        );
    });
}

fn create_market(
    creator: AccountIdTest,
    base_asset: Asset<MarketId>,
    market_type: MarketType,
    scoring_rule: ScoringRule,
) -> MarketId {
    let mut metadata = [2u8; 50];
    metadata[0] = 0x15;
    metadata[1] = 0x30;
    assert_ok!(PredictionMarkets::create_market(
        RuntimeOrigin::signed(creator),
        base_asset,
        EVE,
        MarketPeriod::Block(0..2),
        Deadlines {
            grace_period: 0_u32.into(),
            oracle_duration: <Runtime as zrml_prediction_markets::Config>::MinOracleDuration::get(),
            dispute_duration: <Runtime as zrml_prediction_markets::Config>::MinDisputeDuration::get(
            ),
        },
        MultiHash::Sha3_384(metadata),
        MarketCreation::Permissionless,
        market_type,
        MarketDisputeMechanism::SimpleDisputes,
        scoring_rule,
    ));
    MarketCommons::latest_market_id().unwrap()
}

fn create_market_and_deploy_pool(
    creator: AccountIdTest,
    base_asset: Asset<MarketId>,
    market_type: MarketType,
    amount: BalanceOf<Runtime>,
    spot_prices: Vec<BalanceOf<Runtime>>,
    swap_fee: BalanceOf<Runtime>,
) -> MarketId {
    let market_id = create_market(creator, base_asset, market_type, ScoringRule::Lmsr);
    assert_ok!(PredictionMarkets::buy_complete_set(
        RuntimeOrigin::signed(ALICE),
        market_id,
        amount,
    ));
    assert_ok!(NeoSwaps::deploy_pool(
        RuntimeOrigin::signed(ALICE),
        market_id,
        amount,
        spot_prices.clone(),
        swap_fee,
    ));
    market_id
}
