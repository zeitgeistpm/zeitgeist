// Copyright 2022-2024 Forecasting Technologies LTD.
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

#![cfg(all(feature = "mock", test))]

mod buy_complete_set;

use crate::{mock::*, Config, Error, Event};
use core::ops::Range;
use frame_support::{assert_noop, assert_ok};
use orml_traits::MultiCurrency;
use sp_arithmetic::Perbill;
use zeitgeist_primitives::{
    constants::mock::{BASE, CENT},
    types::{
        Asset, Deadlines, MarketCreation, MarketDisputeMechanism, MarketId, MarketPeriod,
        MarketStatus, MarketType, MultiHash, ScoringRule,
    },
};
use zrml_market_commons::MarketCommonsPalletApi;

fn get_deadlines() -> Deadlines<<Runtime as frame_system::Config>::BlockNumber> {
    Deadlines {
        grace_period: 1_u32.into(),
        oracle_duration: <Runtime as Config>::MinOracleDuration::get(),
        dispute_duration: <Runtime as Config>::MinDisputeDuration::get(),
    }
}

fn gen_metadata(byte: u8) -> MultiHash {
    let mut metadata = [byte; 50];
    metadata[0] = 0x15;
    metadata[1] = 0x30;
    MultiHash::Sha3_384(metadata)
}

fn simple_create_categorical_market(
    base_asset: Asset<MarketId>,
    creation: MarketCreation,
    period: Range<u64>,
    scoring_rule: ScoringRule,
) {
    assert_ok!(PredictionMarkets::create_market(
        RuntimeOrigin::signed(ALICE),
        base_asset,
        Perbill::zero(),
        BOB,
        MarketPeriod::Block(period),
        get_deadlines(),
        gen_metadata(2),
        creation,
        MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
        Some(MarketDisputeMechanism::SimpleDisputes),
        scoring_rule
    ));
}
