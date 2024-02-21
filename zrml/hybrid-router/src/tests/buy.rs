// Copyright 2023-2024 Forecasting Technologies LTD.
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

use crate::{mock::*, utils::*, *};
use core::ops::RangeInclusive;
use frame_support::{assert_noop, assert_ok};
use orml_traits::MultiCurrency;
use test_case::test_case;
use zeitgeist_primitives::types::{Asset, MarketStatus, MarketType, ScoringRule};
use zrml_market_commons::{Error as MError, Markets};

#[test]
fn buy_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);
    });
}