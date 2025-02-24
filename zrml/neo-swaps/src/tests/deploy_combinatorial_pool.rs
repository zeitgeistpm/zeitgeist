// Copyright 2023-2025 Forecasting Technologies LTD.
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
use zeitgeist_primitives::constants::BASE;

#[test]
fn deploy_combinatorial_pool_works_with_single_market() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_before = AssetManager::free_balance(BASE_ASSET, &ALICE);
        let amount = _10;
        let asset_count = 2usize;
        let spot_prices = vec![BASE / (asset_count as u128); asset_count];
        let swap_fee = CENT;
        let (market_ids, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2)],
            amount,
            spot_prices.clone(),
            swap_fee,
        );
        let pool = Pools::<Runtime>::get(pool_id).unwrap();
        let assets = pool.assets();
        let expected_liquidity = 144_269_504_089;
        let buffer = AssetManager::minimum_balance(pool.collateral);
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

        let mut reserves = BTreeMap::new();
        for (&asset, &price) in assets.iter().zip(spot_prices.iter()) {
            assert_balance!(pool.account_id, asset, amount);
            assert_eq!(pool.reserve_of(&asset).unwrap(), amount);
            assert_eq!(pool.calculate_spot_price(asset).unwrap(), price);
            assert_balance!(ALICE, asset, 0);
            reserves.insert(asset, amount);
        }
        assert_balance!(ALICE, BASE_ASSET, alice_before - amount - buffer);

        System::assert_last_event(
            Event::CombinatorialPoolDeployed {
                who: ALICE,
                market_ids,
                pool_id,
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
fn deploy_combinatorial_pool_works_with_single_market_uneven_spot_prices() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_before = AssetManager::free_balance(BASE_ASSET, &ALICE);
        let amount = _10;
        let spot_prices = vec![_1_4, _3_4];
        let expected_reserves = [_10, 20_751_874_964];
        let swap_fee = CENT;
        let (market_ids, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2)],
            amount,
            spot_prices.clone(),
            swap_fee,
        );
        let pool = Pools::<Runtime>::get(pool_id).unwrap();
        let assets = pool.assets();
        let expected_liquidity = 72_134_752_044;
        let buffer = AssetManager::minimum_balance(pool.collateral);
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

        let mut reserves = BTreeMap::new();
        for ((&asset, &price), &reserve) in
            assets.iter().zip(spot_prices.iter()).zip(expected_reserves.iter())
        {
            assert_balance!(pool.account_id, asset, reserve);
            assert_eq!(pool.reserve_of(&asset).unwrap(), reserve);
            assert_eq!(pool.calculate_spot_price(asset).unwrap(), price);
            assert_balance!(ALICE, asset, amount - reserve);
            reserves.insert(asset, reserve);
        }
        assert_balance!(ALICE, BASE_ASSET, alice_before - amount - buffer);

        System::assert_last_event(
            Event::CombinatorialPoolDeployed {
                who: ALICE,
                market_ids,
                pool_id,
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
fn deploy_combinatorial_pool_works_with_multiple_markets() {
    ExtBuilder::default().build().execute_with(|| {
        let alice_before = AssetManager::free_balance(BASE_ASSET, &ALICE);
        let amount = _10;
        let asset_count = 16usize;
        let spot_prices = vec![BASE / (asset_count as u128); asset_count];
        let swap_fee = CENT;
        let (market_ids, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![
                MarketType::Categorical(2),
                MarketType::Categorical(4),
                MarketType::Scalar(0u128..=1u128),
            ],
            amount,
            spot_prices.clone(),
            swap_fee,
        );
        let pool = Pools::<Runtime>::get(pool_id).unwrap();
        let assets = pool.assets();
        let expected_liquidity = 36_067_376_022;
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

        let mut reserves = BTreeMap::new();
        for (&asset, &price) in assets.iter().zip(spot_prices.iter()) {
            assert_balance!(pool.account_id, asset, amount);
            assert_eq!(pool.reserve_of(&asset).unwrap(), amount);
            assert_eq!(pool.calculate_spot_price(asset).unwrap(), price);
            assert_balance!(ALICE, asset, 0);
            reserves.insert(asset, amount);
        }
        assert_balance!(ALICE, BASE_ASSET, alice_before - amount - buffer);

        System::assert_last_event(
            Event::CombinatorialPoolDeployed {
                who: ALICE,
                market_ids,
                pool_id,
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
fn deploy_combinatorial_pool_fails_on_incorrect_vec_len() {
    ExtBuilder::default().build().execute_with(|| {
        // The following markets will produce 6 collections: LONG & 0, LONG & 1, LONG & 2, SHORT & 0, SHORT & 1, SHORT & 2
        let market_ids = vec![
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::AmmCdaHybrid),
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(3), ScoringRule::AmmCdaHybrid),
        ];
        assert_noop!(
            NeoSwaps::deploy_combinatorial_pool(
                RuntimeOrigin::signed(ALICE),
                6,
                market_ids,
                _10,
                // Here it's five spot prices although the above market ids will have 6 spot prices.
                vec![20 * CENT; 5],
                CENT,
                Fuel::new(16, false),
            ),
            Error::<Runtime>::IncorrectVecLen
        );
    });
}

#[test]
fn deploy_combinatorial_pool_fails_on_market_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let _ =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::AmmCdaHybrid);
        let _ =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(5), ScoringRule::AmmCdaHybrid);
        assert_noop!(
            NeoSwaps::deploy_combinatorial_pool(
                RuntimeOrigin::signed(ALICE),
                10,
                vec![0, 2, 1],
                _10,
                vec![10 * CENT; 10],
                CENT,
                Fuel::new(16, false),
            ),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist,
        );
    });
}

#[test_case(MarketStatus::Proposed)]
#[test_case(MarketStatus::Closed)]
#[test_case(MarketStatus::Reported)]
#[test_case(MarketStatus::Disputed)]
#[test_case(MarketStatus::Resolved)]
fn deploy_combinatorial_pool_fails_on_inactive_market(market_status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let market_ids = vec![
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::AmmCdaHybrid),
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(5), ScoringRule::AmmCdaHybrid),
        ];
        MarketCommons::mutate_market(market_ids.last().unwrap(), |market| {
            market.status = market_status;
            Ok(())
        })
        .unwrap();
        assert_noop!(
            NeoSwaps::deploy_combinatorial_pool(
                RuntimeOrigin::signed(ALICE),
                10,
                market_ids,
                _100,
                vec![10 * CENT; 10],
                CENT,
                Fuel::new(16, false),
            ),
            Error::<Runtime>::MarketNotActive,
        );
    });
}

#[test]
fn deploy_combinatorial_pool_fails_on_invalid_trading_mechanism() {
    ExtBuilder::default().build().execute_with(|| {
        let market_ids = vec![
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::AmmCdaHybrid),
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(5), ScoringRule::Parimutuel),
        ];
        assert_noop!(
            NeoSwaps::deploy_combinatorial_pool(
                RuntimeOrigin::signed(ALICE),
                10,
                market_ids,
                _100,
                vec![10 * CENT; 10],
                CENT,
                Fuel::new(16, false),
            ),
            Error::<Runtime>::InvalidTradingMechanism
        );
    });
}

#[test]
fn deploy_combinatorial_pool_fails_on_max_splits_exceeded() {
    ExtBuilder::default().build().execute_with(|| {
        // log2(MaxSplits + 1)
        let market_count = u16::BITS - <Runtime as Config>::MaxSplits::get().leading_zeros();

        let mut market_ids = vec![];
        for _ in 0..market_count {
            let market_id = create_market(
                ALICE,
                BASE_ASSET,
                MarketType::Categorical(2),
                ScoringRule::AmmCdaHybrid,
            );

            market_ids.push(market_id);
        }
        let liquidity = 1_000 * BASE;

        let asset_count = 2u128.pow(market_count);
        let mut spot_prices = vec![_1 / asset_count; asset_count as usize - 1];
        spot_prices.push(_1 - spot_prices.iter().sum::<u128>());

        assert_noop!(
            NeoSwaps::deploy_combinatorial_pool(
                RuntimeOrigin::signed(ALICE),
                2u16.pow(market_count),
                market_ids,
                liquidity,
                spot_prices,
                CENT,
                Fuel::new(16, false),
            ),
            Error::<Runtime>::MaxSplitsExceeded
        );
    });
}

#[test]
fn deploy_combinatorial_pool_fails_on_swap_fee_below_min() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::AmmCdaHybrid);
        let liquidity = _10;
        assert_noop!(
            NeoSwaps::deploy_combinatorial_pool(
                RuntimeOrigin::signed(ALICE),
                2,
                vec![market_id],
                liquidity,
                vec![_1_4, _3_4],
                MIN_SWAP_FEE - 1,
                Fuel::new(16, false),
            ),
            Error::<Runtime>::SwapFeeBelowMin
        );
    });
}

#[test]
fn deploy_combinatorial_pool_fails_on_swap_fee_above_max() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::AmmCdaHybrid);
        let liquidity = _10;
        assert_noop!(
            NeoSwaps::deploy_combinatorial_pool(
                RuntimeOrigin::signed(ALICE),
                2,
                vec![market_id],
                liquidity,
                vec![_1_4, _3_4],
                <Runtime as Config>::MaxSwapFee::get() + 1,
                Fuel::new(16, false),
            ),
            Error::<Runtime>::SwapFeeAboveMax
        );
    });
}

#[test_case(vec![_1_4, _3_4 - 1])]
#[test_case(vec![_1_4 + 1, _3_4])]
fn deploy_combinatorial_pool_fails_on_invalid_spot_prices(spot_prices: Vec<BalanceOf<Runtime>>) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::AmmCdaHybrid);
        let liquidity = _10;
        assert_noop!(
            NeoSwaps::deploy_combinatorial_pool(
                RuntimeOrigin::signed(ALICE),
                2,
                vec![market_id],
                liquidity,
                spot_prices,
                CENT,
                Fuel::new(16, false),
            ),
            Error::<Runtime>::InvalidSpotPrices
        );
    });
}

#[test]
fn deploy_combinatorial_pool_fails_on_spot_price_below_min() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::AmmCdaHybrid);
        let liquidity = _10;
        let spot_price = MIN_SPOT_PRICE - 1;
        assert_noop!(
            NeoSwaps::deploy_combinatorial_pool(
                RuntimeOrigin::signed(ALICE),
                2,
                vec![market_id],
                liquidity,
                vec![spot_price, _1 - spot_price],
                CENT,
                Fuel::new(16, false),
            ),
            Error::<Runtime>::SpotPriceBelowMin
        );
    });
}

#[test]
fn deploy_combinatorial_pool_fails_on_spot_price_above_max() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::AmmCdaHybrid);
        let liquidity = _10;
        let spot_price = MAX_SPOT_PRICE + 1;
        assert_noop!(
            NeoSwaps::deploy_combinatorial_pool(
                RuntimeOrigin::signed(ALICE),
                2,
                vec![market_id],
                liquidity,
                vec![spot_price, _1 - spot_price],
                CENT,
                Fuel::new(16, false),
            ),
            Error::<Runtime>::SpotPriceAboveMax
        );
    });
}

#[test]
fn deploy_combinatorial_pool_fails_on_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(2), ScoringRule::AmmCdaHybrid);
        let liquidity = _10;

        #[cfg(feature = "parachain")]
        let expected_error = orml_tokens::Error::<Runtime>::BalanceTooLow;
        #[cfg(not(feature = "parachain"))]
        let expected_error = orml_currencies::Error::<Runtime>::BalanceTooLow;

        assert_noop!(
            NeoSwaps::deploy_combinatorial_pool(
                // BOB doesn't have enough funds
                RuntimeOrigin::signed(BOB),
                2,
                vec![market_id],
                liquidity,
                vec![_3_4, _1_4],
                CENT,
                Fuel::new(16, false),
            ),
            expected_error
        );
    });
}

#[test]
fn deploy_combinatorial_pool_fails_on_liquidity_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::AmmCdaHybrid);
        let amount = _1_2;
        assert_noop!(
            NeoSwaps::deploy_combinatorial_pool(
                RuntimeOrigin::signed(ALICE),
                2,
                vec![market_id],
                amount,
                vec![_1_2, _1_2],
                CENT,
                Fuel::new(16, false),
            ),
            Error::<Runtime>::LiquidityTooLow
        );
    });
}

#[test]
fn deploy_combinatorial_pool_fails_on_incorrect_asset_count() {
    ExtBuilder::default().build().execute_with(|| {
        let market_ids = vec![
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(3), ScoringRule::AmmCdaHybrid),
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(4), ScoringRule::AmmCdaHybrid),
            create_market(ALICE, BASE_ASSET, MarketType::Categorical(5), ScoringRule::AmmCdaHybrid),
        ];
        let amount = _1_2;
        assert_noop!(
            NeoSwaps::deploy_combinatorial_pool(
                RuntimeOrigin::signed(ALICE),
                61,
                market_ids,
                amount,
                vec![_1_2, _1_2], // Incorrect, but doesn't matter!
                CENT,
                Fuel::new(16, false),
            ),
            Error::<Runtime>::IncorrectAssetCount,
        );
    });
}
