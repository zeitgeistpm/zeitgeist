// Copyright 2022-2023 Forecasting Technologies LTD.
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

use crate::{mock::*, *};
use frame_support::assert_ok;
use zeitgeist_primitives::types::{Asset, MarketStatus, MarketType, Outcome};
use zrml_market_commons::Markets;

#[test]
fn buy_works() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.market_type = MarketType::Categorical(10u16);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(Outcome::CategoricalOutcome(market_id, 0u16));
        let amount = <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount,));
    });
}

#[test]
fn claim_rewards_works() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        Markets::<Runtime>::insert(market_id, market_mock::<Runtime>());
    });
}

#[test]
fn refund_pot_works() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        Markets::<Runtime>::insert(market_id, market_mock::<Runtime>());
    });
}
