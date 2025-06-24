// Copyright 2023-2025 Forecasting Technologies LTD.
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

use crate::{
    math::fixed::{FixedDiv, FixedMulDiv},
    types::Asset,
};
use parity_scale_codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::AtLeast32BitUnsigned, DispatchError, RuntimeDebug};

pub type OrderId = u128;

#[derive(Clone, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Order<AccountId, Balance, MarketId: MaxEncodedLen + HasCompact> {
    pub market_id: MarketId,
    pub maker: AccountId,
    pub maker_asset: Asset<MarketId>,
    pub maker_amount: Balance,
    pub taker_asset: Asset<MarketId>,
    pub taker_amount: Balance,
}

impl<
        AccountId,
        Balance: AtLeast32BitUnsigned + Copy,
        MarketId: MaxEncodedLen + HasCompact + PartialEq,
    > Order<AccountId, Balance, MarketId>
{
    /// Return the price of the order.
    pub fn price(&self, base_asset: Asset<MarketId>) -> Result<Balance, DispatchError> {
        if self.maker_asset == base_asset {
            self.maker_amount.bdiv_floor(self.taker_amount)
        } else if self.taker_asset == base_asset {
            self.taker_amount.bdiv_floor(self.maker_amount)
        } else {
            Err(DispatchError::from("base asset not found"))
        }
    }

    /// Return the (partial) amounts the taker and maker need to provide
    /// to fill a `sub_maker_amount` (lower or equal to `maker_amount`) of an order.
    ///
    /// If the `sub_maker_amount` is higher than or equal to the `maker_amount`,
    /// the full fill amounts of the order are returned.
    /// If the `sub_maker_amount` is lower than the `maker_amount`,
    /// the corresponding amounts to partially fill the order are returned.
    ///
    /// The `taker_fill` is the amount in `maker_asset` the maker transfers to the taker.
    /// The `maker_fill` is the amount in `taker_asset` the taker transfers to the maker.
    ///
    /// Returns `(taker_fill, maker_fill)`.
    pub fn taker_and_maker_fill_from_maker_amount(
        &self,
        sub_maker_amount: Balance,
    ) -> Result<(Balance, Balance), DispatchError> {
        // concious switch to match maker and taker amounts to correct fill amounts
        // the taker gets the maker amount (`taker_fill`)
        // and the maker gets the taker amount (`maker_fill`)
        let (taker_fill, maker_fill) = if sub_maker_amount < self.maker_amount {
            let sub_taker_amount =
                sub_maker_amount.bmul_bdiv_floor(self.taker_amount, self.maker_amount)?;
            (sub_maker_amount, sub_taker_amount)
        } else {
            (self.maker_amount, self.taker_amount)
        };
        Ok((taker_fill, maker_fill))
    }

    /// Return the (partial) amounts the taker and maker need to provide
    /// to fill a `sub_taker_amount` (lower or equal to `taker_amount`) of an order.
    ///
    /// If the `sub_taker_amount` is higher than or equal to the `taker_amount`,
    /// the full fill amounts of the order are returned.
    /// If the `sub_taker_amount` is lower than the `taker_amount`,
    /// the corresponding amounts to partially fill the order are returned.
    ///
    /// The `taker_fill` is the amount in `maker_asset` the maker transfers to the taker.
    /// The `maker_fill` is the amount in `taker_asset` the taker transfers to the maker.
    ///
    /// Returns `(taker_fill, maker_fill)`.
    pub fn taker_and_maker_fill_from_taker_amount(
        &self,
        sub_taker_amount: Balance,
    ) -> Result<(Balance, Balance), DispatchError> {
        // concious switch to match maker and taker amounts to correct fill amounts
        // the taker gets the maker amount (`taker_fill`)
        // and the maker gets the taker amount (`maker_fill`)
        let (taker_fill, maker_fill) = if sub_taker_amount < self.taker_amount {
            let sub_maker_amount =
                sub_taker_amount.bmul_bdiv_floor(self.maker_amount, self.taker_amount)?;
            (sub_maker_amount, sub_taker_amount)
        } else {
            (self.maker_amount, self.taker_amount)
        };
        Ok((taker_fill, maker_fill))
    }
}

#[cfg(test)]
mod tests {
    use sp_runtime::ArithmeticError;
    use test_case::test_case;

    use super::*;
    use crate::{
        constants::{BASE, CENT},
        types::{AccountId, Asset, MarketId},
    };

    pub const BASE_ASSET: Asset<MarketId> = Asset::Ztg;

    #[test]
    fn price_calculation_works() {
        let maker = AccountId::from([1u8; 32]);
        let maker_asset: Asset<MarketId> = BASE_ASSET;
        let market_id = MarketId::default();
        let taker_asset: Asset<MarketId> = Asset::CategoricalOutcome(market_id, 0u16);
        let order = Order {
            market_id,
            maker,
            maker_asset,
            maker_amount: 100 * BASE,
            taker_asset,
            taker_amount: 50 * BASE,
        };

        let price = order.price(maker_asset).unwrap();
        assert_eq!(price, 2 * BASE);

        let price = order.price(taker_asset).unwrap();
        assert_eq!(price, 50 * CENT);
    }

    #[test]
    fn price_throws_error_on_division_by_zero() {
        let maker = AccountId::from([1u8; 32]);
        let maker_asset: Asset<MarketId> = BASE_ASSET;
        let market_id = MarketId::default();
        let taker_asset: Asset<MarketId> = Asset::CategoricalOutcome(market_id, 0u16);
        let order = Order {
            market_id,
            maker,
            maker_asset,
            maker_amount: 0u128,
            taker_asset,
            taker_amount: 0u128,
        };

        let price = order.price(maker_asset);
        assert_eq!(price, Err(DispatchError::Arithmetic(ArithmeticError::DivisionByZero)));

        let price = order.price(taker_asset);
        assert_eq!(price, Err(DispatchError::Arithmetic(ArithmeticError::DivisionByZero)));
    }

    #[test]
    fn price_throws_error_if_base_asset_not_found() {
        let maker = AccountId::from([1u8; 32]);
        let maker_asset: Asset<MarketId> = BASE_ASSET;
        let market_id = MarketId::default();
        let taker_asset: Asset<MarketId> = Asset::CategoricalOutcome(market_id, 0u16);
        let order = Order {
            market_id,
            maker,
            maker_asset,
            maker_amount: 100 * BASE,
            taker_asset,
            taker_amount: 50 * BASE,
        };

        let unknown_asset = Asset::CategoricalOutcome(market_id, 1u16);
        let price = order.price(unknown_asset);
        assert_eq!(price, Err(DispatchError::from("base asset not found")));
    }

    #[test_case(200 * BASE, 100 * BASE, 50 * BASE ; "sub_maker_amount is 200")]
    #[test_case(100 * BASE, 100 * BASE, 50 * BASE ; "sub_maker_amount is 100")]
    #[test_case(50 * BASE, 50 * BASE, 25 * BASE ; "sub_maker_amount is 50")]
    #[test_case(25 * BASE, 25 * BASE, 125000000000 ; "sub_maker_amount is 25")]
    fn taker_and_maker_fill_from_maker_amount_works(
        sub_maker_amount: u128,
        expected_taker_fill: u128,
        expected_maker_fill: u128,
    ) {
        let maker = AccountId::from([1u8; 32]);
        let maker_asset: Asset<MarketId> = BASE_ASSET;
        let market_id = MarketId::default();
        let taker_asset: Asset<MarketId> = Asset::CategoricalOutcome(market_id, 0u16);
        let maker_amount = 100 * BASE;
        let taker_amount = 50 * BASE;
        let order =
            Order { market_id, maker, maker_asset, maker_amount, taker_asset, taker_amount };

        let (taker_fill, maker_fill) =
            order.taker_and_maker_fill_from_maker_amount(sub_maker_amount).unwrap();
        assert_eq!(taker_fill, expected_taker_fill);
        assert_eq!(maker_fill, expected_maker_fill);
    }

    #[test_case(200 * BASE, 50 * BASE, 100 * BASE ; "sub_taker_amount is 200")]
    #[test_case(100 * BASE, 50 * BASE, 100 * BASE ; "sub_taker_amount is 100")]
    #[test_case(50 * BASE, 50 * BASE, 100 * BASE ; "sub_taker_amount is 50")]
    #[test_case(25 * BASE, 25 * BASE, 50 * BASE ; "sub_taker_amount is 25")]
    fn taker_and_maker_fill_from_taker_amount_works(
        sub_taker_amount: u128,
        expected_maker_fill: u128,
        expected_taker_fill: u128,
    ) {
        let maker = AccountId::from([1u8; 32]);
        let maker_asset: Asset<MarketId> = BASE_ASSET;
        let market_id = MarketId::default();
        let taker_asset: Asset<MarketId> = Asset::CategoricalOutcome(market_id, 0u16);
        let maker_amount = 100 * BASE;
        let taker_amount = 50 * BASE;
        let order =
            Order { market_id, maker, maker_asset, maker_amount, taker_asset, taker_amount };

        let (taker_fill, maker_fill) =
            order.taker_and_maker_fill_from_taker_amount(sub_taker_amount).unwrap();
        assert_eq!(taker_fill, expected_taker_fill);
        assert_eq!(maker_fill, expected_maker_fill);
    }
}
