// Copyright 2024-2025 Forecasting Technologies LTD.
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
use zeitgeist_primitives::traits::PayoutApi;

#[test]
fn payout_vector_works_categorical() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );

        let market_id = 0;

        let market = MarketCommons::market(&market_id).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        run_blocks(market.deadlines.dispute_duration);

        assert_eq!(PredictionMarkets::payout_vector(market_id), Some(vec![0, BASE]));
    });
}

#[test_case(50, vec![0, BASE])]
#[test_case(100, vec![0, BASE])]
#[test_case(130, vec![30 * CENT, 70 * CENT])]
#[test_case(200, vec![BASE, 0])]
#[test_case(250, vec![BASE, 0])]
fn payout_vector_works_scalar(value: u128, expected: Vec<BalanceOf<Runtime>>) {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        simple_create_scalar_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );

        let market_id = 0;

        let market = MarketCommons::market(&market_id).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Scalar(value)
        ));

        run_blocks(market.deadlines.dispute_duration);

        assert_eq!(PredictionMarkets::payout_vector(market_id), Some(expected));
    });
}

#[test]
fn payout_vector_fails_on_market_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(PredictionMarkets::payout_vector(1), None);
    });
}

#[test]
fn payout_vector_fails_if_market_is_not_redeemable() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::Parimutuel,
        );

        assert_ok!(MarketCommons::mutate_market(&0, |market_inner| {
            market_inner.status = MarketStatus::Resolved;
            Ok(())
        }));

        assert_eq!(PredictionMarkets::payout_vector(0), None);
    });
}

#[test_case(MarketStatus::Proposed)]
#[test_case(MarketStatus::Active)]
#[test_case(MarketStatus::Closed)]
#[test_case(MarketStatus::Reported)]
#[test_case(MarketStatus::Disputed)]
fn payout_vector_fails_on_invalid_market_status(status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::AmmCdaHybrid,
        );

        assert_ok!(MarketCommons::mutate_market(&0, |market_inner| {
            market_inner.status = status;
            Ok(())
        }));

        assert_eq!(PredictionMarkets::payout_vector(0), None);
    });
}
