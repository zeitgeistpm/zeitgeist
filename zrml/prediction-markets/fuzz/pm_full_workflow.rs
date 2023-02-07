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

#![no_main]

use arbitrary::Arbitrary;
use core::ops::{Range, RangeInclusive};
use frame_support::traits::Hooks;
use libfuzzer_sys::fuzz_target;
use zeitgeist_primitives::types::{
    Asset, Deadlines, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketType, MultiHash,
    OutcomeReport, ScoringRule,
};
use zrml_prediction_markets::mock::{ExtBuilder, PredictionMarkets, RuntimeOrigin, System};

fuzz_target!(|data: Data| {
    let mut ext = ExtBuilder::default().build();
    ext.execute_with(|| {
        let _ = PredictionMarkets::on_initialize(1);
        System::set_block_number(1);

        let deadlines = Deadlines {
            grace_period: 1_u32.into(),
            oracle_duration: 1_u32.into(),
            dispute_duration: 3_u32.into(),
        };
        let _ = PredictionMarkets::create_market(
            RuntimeOrigin::signed(data.create_scalar_market_origin.into()),
            Asset::Ztg,
            data.create_scalar_market_oracle.into(),
            MarketPeriod::Block(data.create_scalar_market_period),
            deadlines,
            data.create_scalar_market_metadata,
            market_creation(data.create_scalar_market_creation),
            MarketType::Scalar(data.create_scalar_market_outcome_range),
            market_dispute_mechanism(data.create_scalar_market_dispute_mechanism),
            ScoringRule::CPMM,
        );

        let _ = PredictionMarkets::on_initialize(2);
        System::set_block_number(2);

        let _ = PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(data.buy_complete_set_origin.into()),
            data.buy_complete_set_market_id.into(),
            data.buy_complete_set_amount,
        );
        let _ = PredictionMarkets::on_initialize(3);
        System::set_block_number(3);

        let _ = PredictionMarkets::report(
            RuntimeOrigin::signed(data.report_origin.into()),
            data.report_market_id.into(),
            outcome(data.report_outcome),
        );

        let _ = PredictionMarkets::on_initialize(4);
        System::set_block_number(4);

        let dispute_market_id = data.dispute_market_id.into();
        let _ = PredictionMarkets::dispute(
            RuntimeOrigin::signed(data.report_origin.into()),
            dispute_market_id,
            outcome(data.report_outcome),
        );

        let _ = PredictionMarkets::on_initialize(5);
        System::set_block_number(5);

        let _ = PredictionMarkets::redeem_shares(
            RuntimeOrigin::signed(data.redeem_origin.into()),
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
    create_scalar_market_period: Range<u64>,
    create_scalar_market_metadata: MultiHash,
    create_scalar_market_creation: u8,
    create_scalar_market_outcome_range: RangeInclusive<u128>,
    create_scalar_market_dispute_mechanism: u8,

    buy_complete_set_origin: u8,
    buy_complete_set_market_id: u8,
    buy_complete_set_amount: u128,

    report_origin: u8,
    report_market_id: u8,
    report_outcome: u128,

    dispute_market_id: u8,

    redeem_origin: u8,
    redeem_market_id: u8,
}

#[inline]
fn market_creation(seed: u8) -> MarketCreation {
    if seed % 2 == 0 { MarketCreation::Advised } else { MarketCreation::Permissionless }
}

#[inline]
fn market_dispute_mechanism(seed: u8) -> MarketDisputeMechanism {
    match seed % 3 {
        0 => MarketDisputeMechanism::Authorized,
        1 => MarketDisputeMechanism::Court,
        _ => MarketDisputeMechanism::SimpleDisputes,
    }
}

#[inline]
fn outcome(seed: u128) -> OutcomeReport {
    if seed % 2 == 0 {
        OutcomeReport::Categorical(seed as _)
    } else {
        OutcomeReport::Scalar(seed as _)
    }
}
