#![no_main]

use arbitrary::Arbitrary;
use frame_support::traits::Hooks;
use libfuzzer_sys::fuzz_target;
use zeitgeist_primitives::types::{MarketCreation, MarketEnd, MultiHash, OutcomeReport};
use zrml_prediction_markets::mock::{ExtBuilder, Origin, PredictionMarkets, System};

fuzz_target!(|data: Data| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let _ = PredictionMarkets::on_initialize(1);
        System::set_block_number(1);

        let _ = PredictionMarkets::create_scalar_market(
            Origin::signed(data.create_scalar_market_origin.into()),
            data.create_scalar_market_oracle.into(),
            MarketEnd::Timestamp(data.create_scalar_market_timestamp),
            data.create_scalar_market_metadata,
            market_creation(data.create_scalar_market_creation),
            data.create_scalar_market_outcome_range,
        );

        let _ = PredictionMarkets::on_initialize(2);
        System::set_block_number(2);

        let _ = PredictionMarkets::buy_complete_set(
            Origin::signed(data.buy_complete_set_origin.into()),
            data.buy_complete_set_market_id.into(),
            data.buy_complete_set_amount,
        );
        let _ = PredictionMarkets::on_initialize(3);
        System::set_block_number(3);

        let _ = PredictionMarkets::report(
            Origin::signed(data.report_origin.into()),
            data.report_market_id.into(),
            outcome(data.report_outcome),
        );

        let _ = PredictionMarkets::on_initialize(4);
        System::set_block_number(4);

        let _ = PredictionMarkets::dispute(
            Origin::signed(data.dispute_origin.into()),
            data.dispute_market_id.into(),
            outcome(data.report_outcome),
        );

        let _ = PredictionMarkets::on_initialize(5);
        System::set_block_number(5);

        let _ = PredictionMarkets::redeem_shares(
            Origin::signed(data.redeem_origin.into()),
            data.redeem_market_id.into(),
        );

        let _ = PredictionMarkets::on_initialize(6);
    });
    let _ = ext.commit_all();
});

#[derive(Debug, Arbitrary)]
struct Data {
    create_scalar_market_origin: u8,
    create_scalar_market_oracle: u8,
    create_scalar_market_timestamp: u64,
    create_scalar_market_metadata: MultiHash,
    create_scalar_market_creation: u8,
    create_scalar_market_outcome_range: (u128, u128),

    buy_complete_set_origin: u8,
    buy_complete_set_market_id: u8,
    buy_complete_set_amount: u128,

    report_origin: u8,
    report_market_id: u8,
    report_outcome: u128,

    dispute_origin: u8,
    dispute_market_id: u8,
    dispute_outcome: u128,

    redeem_origin: u8,
    redeem_market_id: u8,
}

#[inline]
fn market_creation(seed: u8) -> MarketCreation {
    if seed % 2 == 0 { MarketCreation::Advised } else { MarketCreation::Permissionless }
}

#[inline]
fn outcome(seed: u128) -> OutcomeReport {
    if seed % 2 == 0 {
        OutcomeReport::Categorical(seed as _)
    } else {
        OutcomeReport::Scalar(seed as _)
    }
}
