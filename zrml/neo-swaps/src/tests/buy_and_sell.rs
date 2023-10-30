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

use super::*;
use zeitgeist_primitives::constants::BASE;

macro_rules! assert_pool_status {
    ($market_id:expr, $reserves:expr, $spot_prices:expr, $fees:expr $(,)?) => {
        let pool = Pools::<Runtime>::get($market_id).unwrap();
        assert_eq!(pool.reserves.values().cloned().collect::<Vec<u128>>(), $reserves);
        assert_eq!(
            pool.assets()
                .iter()
                .map(|&a| pool.calculate_spot_price(a).unwrap())
                .collect::<Vec<u128>>(),
            $spot_prices,
        );
        let invariant = $spot_prices.iter().sum::<u128>();
        assert_approx!(invariant, _1, 1);
        assert_eq!(pool.liquidity_shares_manager.fees, $fees);
    };
}

#[test]
fn buy_and_sell() {
    ExtBuilder::default().build().execute_with(|| {
        let asset_count = 3;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            _100,
            vec![_1_2, _1_4, _1_4],
            CENT,
        );
        assert_ok!(AssetManager::deposit(BASE_ASSET, &ALICE, _1000));
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, _1000));
        assert_ok!(AssetManager::deposit(BASE_ASSET, &CHARLIE, _1000));

        assert_ok!(NeoSwaps::buy(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            Asset::CategoricalOutcome(market_id, 2),
            _10,
            0,
        ));
        assert_pool_status!(
            market_id,
            vec![598_000_000_000, 1_098_000_000_000, 767_092_556_931],
            [4_364_837_956, 2_182_418_978, 3_452_743_066],
            1_000_000_000,
        );

        assert_ok!(NeoSwaps::buy(
            RuntimeOrigin::signed(BOB),
            market_id,
            asset_count,
            Asset::CategoricalOutcome(market_id, 1),
            1_234_567_898_765,
            0,
        ));
        assert_pool_status!(
            market_id,
            vec![1_807_876_540_789, 113_931_597_104, 1_976_969_097_720],
            [815_736_444, 8_538_986_828, 645_276_728],
            13_345_678_988,
        );

        assert_ok!(NeoSwaps::buy(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            asset_count,
            Asset::CategoricalOutcome(market_id, 0),
            667 * BASE,
            0,
        ));
        assert_pool_status!(
            market_id,
            vec![76_875_275, 6_650_531_597_104, 8_513_569_097_720],
            [9_998_934_339, 990_789, 74_872],
            80_045_678_988,
        );

        // Selling asset 2 is illegal due to low spot price.
        assert_noop!(
            NeoSwaps::sell(
                RuntimeOrigin::signed(ALICE),
                market_id,
                asset_count,
                Asset::CategoricalOutcome(market_id, 2),
                123_456,
                0,
            ),
            Error::<Runtime>::NumericalLimits(NumericalLimitsError::SpotPriceTooLow),
        );

        assert_ok!(NeoSwaps::sell(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            asset_count,
            Asset::CategoricalOutcome(market_id, 0),
            _1,
            0,
        ));
        assert_pool_status!(
            market_id,
            vec![77_948_356, 6_640_532_670_185, 8_503_570_170_801],
            [9_998_919_465, 1_004_618, 75_917],
            80_145_668_257,
        );

        // Selling asset 1 is allowed, but selling too much will raise an error.
        assert_noop!(
            NeoSwaps::sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                asset_count,
                Asset::CategoricalOutcome(market_id, 1),
                _100,
                0,
            ),
            Error::<Runtime>::NumericalLimits(NumericalLimitsError::SpotPriceSlippedTooLow),
        );

        // Try to sell more than the maximum amount.
        assert_noop!(
            NeoSwaps::sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                asset_count,
                Asset::CategoricalOutcome(market_id, 1),
                _1000,
                0,
            ),
            Error::<Runtime>::NumericalLimits(NumericalLimitsError::MaxAmountExceeded),
        );

        // Buying a small amount from an asset with a low price fails...
        assert_noop!(
            NeoSwaps::buy(
                RuntimeOrigin::signed(CHARLIE),
                market_id,
                asset_count,
                Asset::CategoricalOutcome(market_id, 2),
                _1,
                0,
            ),
            Error::<Runtime>::NumericalLimits(NumericalLimitsError::MinAmountNotMet),
        );

        // ...but buying a large amount is fine.
        assert_ok!(NeoSwaps::buy(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            asset_count,
            Asset::CategoricalOutcome(market_id, 2),
            _100,
            0,
        ));
        assert_pool_status!(
            market_id,
            vec![980_077_948_356, 7_620_532_670_185, 214_308_675_476],
            [2_570_006_838, 258_215, 7_429_734_946],
            90_145_668_257,
        );
    });
}
