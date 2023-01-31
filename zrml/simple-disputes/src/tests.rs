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
    Disputes, Error, MarketOf,
};
use frame_support::{assert_noop, BoundedVec};
use zeitgeist_primitives::{
    constants::mock::{OutcomeBond, OutcomeFactor},
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
    bonds: MarketBonds { creation: None, oracle: None, outsider: None, dispute: None },
};

#[test]
fn on_dispute_denies_non_simple_disputes_markets() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.dispute_mechanism = MarketDisputeMechanism::Court;
        assert_noop!(
            SimpleDisputes::on_dispute(&0, &market),
            Error::<Runtime>::MarketDoesNotHaveSimpleDisputesMechanism
        );
    });
}

#[test]
fn get_resolution_outcome_denies_non_simple_disputes_markets() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.dispute_mechanism = MarketDisputeMechanism::Court;
        assert_noop!(
            SimpleDisputes::get_resolution_outcome(&0, &market),
            Error::<Runtime>::MarketDoesNotHaveSimpleDisputesMechanism
        );
    });
}

#[test]
fn get_resolution_outcome_sets_the_last_dispute_of_disputed_markets_as_the_canonical_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.status = MarketStatus::Disputed;
        let disputes = BoundedVec::try_from(
            [
                MarketDispute {
                    at: 0,
                    by: 0,
                    outcome: OutcomeReport::Scalar(0),
                    bond: OutcomeBond::get(),
                },
                MarketDispute {
                    at: 0,
                    by: 0,
                    outcome: OutcomeReport::Scalar(20),
                    bond: OutcomeFactor::get() * OutcomeBond::get(),
                },
            ]
            .to_vec(),
        )
        .unwrap();
        Disputes::<Runtime>::insert(0, &disputes);
        assert_eq!(
            &SimpleDisputes::get_resolution_outcome(&0, &market).unwrap().unwrap(),
            &disputes.last().unwrap().outcome
        )
    });
}

// TODO test `reserve_outcome` functionality and API functionality
