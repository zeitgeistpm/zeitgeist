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

use super::*;
use crate::liquidity_tree::types::Node;
use alloc::collections::BTreeMap;
use test_case::test_case;

#[test]
fn deploy_pool_works_with_binary_markets() {
    ExtBuilder::default().build().execute_with(|| {
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
        let buffer = AssetManager::minimum_balance(pool.collateral);
        assert_eq!(pool.assets(), assets);
        assert_approx!(pool.liquidity_parameter, expected_liquidity, 1);
        assert_eq!(pool.collateral, BASE_ASSET);
        assert_liquidity_tree_state!(
            pool.liquidity_shares_manager,
            [Node::<Runtime> {
                account: Some(ALICE),
                stake: amount,
                fees: 0u128,
                descendant_stake: 0u128,
                lazy_fees: 0u128,
            }],
            create_b_tree_map!({ ALICE => 0 }),
            Vec::<u32>::new(),
        );
        assert_eq!(pool.swap_fee, swap_fee);
        assert_balance!(pool.account_id, pool.collateral, buffer);
        assert_balance!(pool.account_id, assets[0], amount);
        assert_balance!(pool.account_id, assets[1], amount);
        assert_eq!(pool.reserve_of(&assets[0]).unwrap(), amount);
        assert_eq!(pool.reserve_of(&assets[1]).unwrap(), amount);
        assert_eq!(pool.calculate_spot_price(assets[0]).unwrap(), spot_prices[0]);
        assert_eq!(pool.calculate_spot_price(assets[1]).unwrap(), spot_prices[1]);
        assert_balance!(ALICE, BASE_ASSET, alice_before - amount - buffer);
        assert_balance!(ALICE, assets[0], 0);
        assert_balance!(ALICE, assets[1], 0);
        let mut reserves = BTreeMap::new();
        reserves.insert(assets[0], amount);
        reserves.insert(assets[1], amount);
        System::assert_last_event(
            Event::PoolDeployed {
                who: ALICE,
                market_id,
                account_id: pool.account_id,
                reserves,
                collateral: pool.collateral,
                liquidity_parameter: pool.liquidity_parameter,
                pool_shares_amount: amount,
                swap_fee,
            }
            .into(),
        );
    });
}

#[test]
fn deploy_pool_works_with_scalar_marktes() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_before = AssetManager::free_balance(BASE_ASSET, &ALICE);
        let amount = _100;
        let expected_amounts = [amount, 101755598229];
        let spot_prices = vec![_1_6, _5_6 + 1];
        let swap_fee = CENT;
        let market_id: MarketId = 0;
        let assets = vec![
            Asset::ScalarOutcome(market_id, ScalarPosition::Long),
            Asset::ScalarOutcome(market_id, ScalarPosition::Short),
        ];
        let _ = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            amount,
            spot_prices.clone(),
            swap_fee,
        );
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let mut reserves = BTreeMap::new();
        reserves.insert(assets[0], expected_amounts[0]);
        reserves.insert(assets[1], expected_amounts[1]);
        System::assert_last_event(
            Event::PoolDeployed {
                who: ALICE,
                market_id,
                account_id: pool.account_id,
                reserves,
                collateral: pool.collateral,
                liquidity_parameter: pool.liquidity_parameter,
                pool_shares_amount: amount,
                swap_fee,
            }
            .into(),
        );
        // Deploy some funds in the pool account to ensure that rogue funds don't screw up price
        // calculatings.
        let rogue_funds = _100;
        assert_ok!(AssetManager::deposit(
            assets[0],
            &Pallet::<Runtime>::pool_account_id(&market_id),
            rogue_funds
        ));
        let expected_liquidity = 558110626551;
        let buffer = AssetManager::minimum_balance(pool.collateral);
        assert_eq!(pool.assets(), assets);
        assert_approx!(pool.liquidity_parameter, expected_liquidity, 1_000);
        assert_eq!(pool.collateral, BASE_ASSET);
        assert_liquidity_tree_state!(
            pool.liquidity_shares_manager,
            [Node::<Runtime> {
                account: Some(ALICE),
                stake: amount,
                fees: 0u128,
                descendant_stake: 0u128,
                lazy_fees: 0u128,
            }],
            create_b_tree_map!({ ALICE => 0 }),
            Vec::<u32>::new(),
        );
        assert_eq!(pool.swap_fee, swap_fee);
        assert_balance!(pool.account_id, assets[0], expected_amounts[0] + rogue_funds);
        assert_balance!(pool.account_id, assets[1], expected_amounts[1]);
        assert_eq!(pool.reserve_of(&assets[0]).unwrap(), expected_amounts[0]);
        assert_eq!(pool.reserve_of(&assets[1]).unwrap(), expected_amounts[1]);
        assert_eq!(pool.calculate_spot_price(assets[0]).unwrap(), spot_prices[0]);
        assert_eq!(pool.calculate_spot_price(assets[1]).unwrap(), spot_prices[1]);
        assert_balance!(ALICE, BASE_ASSET, alice_before - amount - buffer);
        assert_balance!(ALICE, assets[0], 0);
        assert_balance!(ALICE, assets[1], amount - expected_amounts[1]);
        let price_sum =
            pool.assets().iter().map(|&a| pool.calculate_spot_price(a).unwrap()).sum::<u128>();
        assert_eq!(price_sum, _1);
    });
}

#[test]
fn deploy_pool_fails_on_incorrect_vec_len() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::AmmCdaHybrid);
        assert_noop!(
            NeoSwaps::deploy_pool(RuntimeOrigin::signed(ALICE), market_id, _10, vec![_1_3], CENT),
            Error::<Runtime>::IncorrectVecLen
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
#[test_case(MarketStatus::Closed)]
#[test_case(MarketStatus::Reported)]
#[test_case(MarketStatus::Disputed)]
#[test_case(MarketStatus::Resolved)]
fn deploy_pool_fails_on_inactive_market(market_status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::AmmCdaHybrid);
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

#[test_case(ScoringRule::Parimutuel)]
fn deploy_pool_fails_on_invalid_trading_mechanism(scoring_rule: ScoringRule) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), scoring_rule);
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
fn deploy_pool_fails_on_asset_count_above_max() {
    ExtBuilder::default().build().execute_with(|| {
        let category_count = MAX_ASSETS + 1;
        let market_id = create_market(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(category_count),
            ScoringRule::AmmCdaHybrid,
        );
        let liquidity = _10;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            liquidity,
        ));
        // Beware! Depending on the value of MAX_ASSETS and price barriers, this `spot_prices`
        // vector might violate some other rules for deploying pools.
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
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::AmmCdaHybrid);
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
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::AmmCdaHybrid);
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
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::AmmCdaHybrid);
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
fn deploy_pool_fails_on_spot_price_below_min() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::AmmCdaHybrid);
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
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::AmmCdaHybrid);
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
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::AmmCdaHybrid);
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
            DispatchError::Arithmetic(ArithmeticError::Underflow)
        );
    });
}

#[test]
fn deploy_pool_fails_on_liquidity_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::AmmCdaHybrid);
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
