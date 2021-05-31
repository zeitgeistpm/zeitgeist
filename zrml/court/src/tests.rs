#![cfg(test)]

use crate::{mock::*, CourtPalletApi, MarketIdsPerReportBlock, Markets};
use frame_support::assert_ok;
use zeitgeist_primitives::types::{
    Market, MarketCreation, MarketEnd, MarketStatus, MarketType, OutcomeReport, Report,
};

#[test]
fn it_allows_to_dispute_the_outcome_of_a_market() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        create_reported_permissionless_categorical_market::<Runtime>();

        assert_ok!(Court::on_dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(0)
        ));

        let market = Court::markets(0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        let disputes = Court::disputes(0);
        assert_eq!(disputes.len(), 1);
        let dispute = &disputes[0];
        assert_eq!(dispute.at, 1);
        assert_eq!(dispute.by, CHARLIE);
        assert_eq!(dispute.outcome, OutcomeReport::Categorical(0));

        let market_ids = Court::market_ids_per_dispute_block(1);
        assert_eq!(market_ids.len(), 1);
        assert_eq!(market_ids[0], 0);
    });
}

#[test]
fn it_correctly_resolves_a_market_that_was_reported_on() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        create_reported_permissionless_categorical_market::<Runtime>();

        let reported_ids = Court::market_ids_per_report_block(1);
        assert_eq!(reported_ids.len(), 1);
        let id = reported_ids[0];
        assert_eq!(id, 0);

        System::set_block_number(11);
        Court::on_resolution(11).unwrap();
    });
}

#[test]
fn it_resolves_a_disputed_market() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        create_reported_permissionless_categorical_market::<Runtime>();

        assert_ok!(Court::on_dispute(
            Origin::signed(CHARLIE),
            0,
            OutcomeReport::Categorical(1)
        ));

        assert_ok!(Court::on_dispute(
            Origin::signed(DAVE),
            0,
            OutcomeReport::Categorical(0)
        ));

        assert_ok!(Court::on_dispute(
            Origin::signed(EVE),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market = Court::markets(0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        // check everyone's deposits
        let charlie_reserved = Balances::reserved_balance(&CHARLIE);
        assert_eq!(charlie_reserved, 100);

        let dave_reserved = Balances::reserved_balance(&DAVE);
        assert_eq!(dave_reserved, 125);

        let eve_reserved = Balances::reserved_balance(&EVE);
        assert_eq!(eve_reserved, 150);

        // check disputes length
        let disputes = Court::disputes(0);
        assert_eq!(disputes.len(), 3);

        // make sure the old mappings of market id per dispute block are erased
        let market_ids_1 = Court::market_ids_per_dispute_block(0);
        assert_eq!(market_ids_1.len(), 0);

        let market_ids_2 = Court::market_ids_per_dispute_block(1);
        assert_eq!(market_ids_2.len(), 1);

        System::set_block_number(11);

        assert_ok!(Court::on_resolution(11));

        let market_after = Court::markets(0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);
    });
}

fn create_reported_permissionless_categorical_market<T: crate::Config>() {
    Markets::<Runtime>::insert(
        0,
        Some(Market {
            creation: MarketCreation::Permissionless,
            creator_fee: 0,
            creator: ALICE,
            end: MarketEnd::Block(100),
            market_type: MarketType::Categorical(2),
            metadata: Default::default(),
            oracle: ALICE,
            report: Some(Report {
                at: 1,
                by: ALICE,
                outcome: OutcomeReport::Categorical(0),
            }),
            resolved_outcome: None,
            status: MarketStatus::Reported,
        }),
    );
    MarketIdsPerReportBlock::<Runtime>::mutate(System::block_number(), |v| {
        v.push(0);
    });
}
