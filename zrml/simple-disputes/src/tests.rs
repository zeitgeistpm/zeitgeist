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

#![cfg(test)]

use crate::{
    mock::{ExtBuilder, Runtime, SimpleDisputes},
    Error, MarketOf,
};
use frame_support::assert_noop;
use zeitgeist_primitives::{
    traits::DisputeApi,
    types::{
        Asset, Deadlines, Market, MarketBonds, MarketCreation, MarketDispute,
        MarketDisputeMechanism, MarketPeriod, MarketStatus, MarketType, OutcomeReport, ScoringRule,
    },
};

const DEFAULT_MARKET: MarketOf<Runtime> = Market {
    base_asset: Asset::Ztg,
    creation: MarketCreation::Permissionless,
    creator_fee: 0,
    creator: 0,
    market_type: MarketType::Scalar(0..=100),
    dispute_mechanism: MarketDisputeMechanism::SimpleDisputes,
    metadata: vec![],
    oracle: 0,
    period: MarketPeriod::Block(0..100),
    deadlines: Deadlines { grace_period: 1_u64, oracle_duration: 1_u64, dispute_duration: 1_u64 },
    report: None,
    resolved_outcome: None,
    scoring_rule: ScoringRule::CPMM,
    status: MarketStatus::Disputed,
    bonds: MarketBonds { creation: None, oracle: None, outsider: None },
};

#[test]
fn on_dispute_denies_non_simple_disputes_markets() {
    ExtBuilder.build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.dispute_mechanism = MarketDisputeMechanism::Court;
        assert_noop!(
            SimpleDisputes::on_dispute(&[], &0, &market),
            Error::<Runtime>::MarketDoesNotHaveSimpleDisputesMechanism
        );
    });
}

#[test]
fn on_resolution_denies_non_simple_disputes_markets() {
    ExtBuilder.build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.dispute_mechanism = MarketDisputeMechanism::Court;
        assert_noop!(
            SimpleDisputes::on_resolution(&[], &0, &market),
            Error::<Runtime>::MarketDoesNotHaveSimpleDisputesMechanism
        );
    });
}

#[test]
fn on_resolution_sets_the_last_dispute_of_disputed_markets_as_the_canonical_outcome() {
    ExtBuilder.build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = MarketStatus::Disputed;
        let disputes = [
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(0) },
            MarketDispute { at: 0, by: 0, outcome: OutcomeReport::Scalar(20) },
        ];
        assert_eq!(
            &SimpleDisputes::on_resolution(&disputes, &0, &market).unwrap().unwrap(),
            &disputes.last().unwrap().outcome
        )
    });
}
