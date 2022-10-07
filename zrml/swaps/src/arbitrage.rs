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

use crate::{math::calc_spot_price, root::calc_preimage};
use parity_scale_codec::MaxEncodedLen;
use sp_runtime::traits::AtLeast32BitUnsigned;
use std::collections::HashMap;
use zeitgeist_primitives::{
    constants::BASE,
    types::{Asset, Pool, PoolId, PoolStatus, ScoringRule},
};

trait Arbitrage<Balance, MarketId>
where
    Balance: AtLeast32BitUnsigned,
    MarketId: MaxEncodedLen,
{
    fn calc_total_spot_price(
        self,
        balances: HashMap<Asset<MarketId>, Balance>,
    ) -> Result<Balance, &'static str>;

    fn calc_total_spot_price_after_buy_burn(
        self,
        balances: HashMap<Asset<MarketId>, Balance>,
        amount: Balance,
    ) -> Result<Balance, &'static str>;

    fn calc_arbitrage_amount(
        self,
        balances: HashMap<Asset<MarketId>, Balance>,
    ) -> Result<Balance, &'static str>;
}

impl<Balance, MarketId> Arbitrage<Balance, MarketId> for Pool<Balance, MarketId>
where
    Balance: AtLeast32BitUnsigned,
    MarketId: MaxEncodedLen,
{
    fn calc_total_spot_price(
        self,
        balances: HashMap<Asset<MarketId>, Balance>,
    ) -> Result<Balance, &'static str> {
        Ok(0u8.into())
        // // TODO Add a shift direction for other type of arbitrage.
        // self.assets
        //     .filter(|a, _| a != self.base_asset)
        //     .map(|a| {
        //         // We're deliberately _not_ using the pool's swap fee!
        //         calc_spot_price(
        //             balances[a],
        //             self.weights[a],
        //             balances[self.base_asset],
        //             weights[self.base_asset],
        //             0,
        //         )
        //     })
        //     .fold(|acc, val| acc + val)
    }

    fn calc_total_spot_price_after_buy_burn(
        self,
        balances: HashMap<Asset<MarketId>, Balance>,
        amount: Balance,
    ) -> Result<Balance, &'static str> {
        Ok(0u8.into())
        // TODO Check how to best do this: Copy the HashMap or edit it?
        // balances = balances.map(
    }

    // Calling with a non-CPMM pool results in undefined behavior.
    fn calc_arbitrage_amount(
        self,
        balances: HashMap<Asset<MarketId>, Balance>,
    ) -> Result<Balance, &'static str> {
        Ok(0u8.into())
        // let total_spot_price = self.calc_total_spot_price_after_shift(balances, 0);
        // if total_spot_price > BASE {
        // } else {
        // }
    }
}
