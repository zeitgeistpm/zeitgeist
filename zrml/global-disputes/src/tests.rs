#![cfg(test)]

use crate::{
    mock::{ExtBuilder, Runtime, GlobalDisputes},
    Error,
};
use frame_support::assert_noop;
use zeitgeist_primitives::{
    traits::DisputeApi,
    types::{
        Market, MarketCreation, MarketDispute, MarketDisputeMechanism, MarketPeriod, MarketStatus,
        MarketType, OutcomeReport, ScoringRule,
    },
};

const DEFAULT_MARKET: Market<u128, u64, u64> = Market {
    creation: MarketCreation::Permissionless,
    creator_fee: 0,
    creator: 0,
    market_type: MarketType::Scalar(0..=100),
    mdm: MarketDisputeMechanism::GlobalDisputes,
    metadata: vec![],
    oracle: 0,
    period: MarketPeriod::Block(0..100),
    report: None,
    resolved_outcome: None,
    scoring_rule: ScoringRule::CPMM,
    status: MarketStatus::Disputed,
};

#[test]
fn on_dispute_denies_non_global_disputes_markets() {
    ExtBuilder.build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.mdm = MarketDisputeMechanism::Court;
        assert_noop!(
            GlobalDisputes::on_dispute(&[], &0, &market),
            Error::<Runtime>::MarketDoesNotHaveGlobalDisputesMechanism
        );
    });
}

#[test]
fn on_resolution_denies_non_simple_disputes_markets() {
    ExtBuilder.build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.mdm = MarketDisputeMechanism::Court;
        assert_noop!(
            GlobalDisputes::on_resolution(&[], &0, &market),
            Error::<Runtime>::MarketDoesNotHaveGlobalDisputesMechanism
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
            &GlobalDisputes::on_resolution(&disputes, &0, &market).unwrap().unwrap(),
            &disputes.last().unwrap().outcome
        )
    });
}
