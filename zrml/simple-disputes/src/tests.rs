#![cfg(test)]

use crate::{
    mock::{ExtBuilder, Runtime, SimpleDisputes},
    Error,
};
use frame_support::assert_noop;
use zeitgeist_primitives::{
    traits::DisputeApi,
    types::{
        Market, MarketCreation, MarketDispute, MarketDisputeMechanism, MarketPeriod, MarketStatus,
        MarketType, OutcomeReport, Report,
    },
};

const DEFAULT_MARKET: Market<u128, u64, u64> = Market {
    creation: MarketCreation::Permissionless,
    creator_fee: 0,
    creator: 0,
    market_type: MarketType::Scalar(0..=100),
    mdm: MarketDisputeMechanism::SimpleDisputes,
    metadata: vec![],
    oracle: 0,
    period: MarketPeriod::Block(0..100),
    report: None,
    resolved_outcome: None,
    status: MarketStatus::Disputed,
};

#[test]
fn on_dispute_denies_non_simple_disputes_markets() {
    ExtBuilder.build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.mdm = MarketDisputeMechanism::Court;
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
        market.mdm = MarketDisputeMechanism::Court;
        assert_noop!(
            SimpleDisputes::on_resolution(&[], &0, &market),
            Error::<Runtime>::MarketDoesNotHaveSimpleDisputesMechanism
        );
    });
}

#[test]
fn on_resolution_sets_reported_outcome_of_reported_markets_as_the_canonical_outcome() {
    ExtBuilder.build().execute_with(|| {
        let outcome = OutcomeReport::Scalar(3);
        let mut market = DEFAULT_MARKET;
        market.status = MarketStatus::Reported;
        market.report = Some(Report { at: 0, by: 0, outcome: outcome.clone() });
        assert_eq!(outcome, SimpleDisputes::on_resolution(&[], &0, &market).unwrap())
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
            &SimpleDisputes::on_resolution(&disputes, &0, &market).unwrap(),
            &disputes.last().unwrap().outcome
        )
    });
}
