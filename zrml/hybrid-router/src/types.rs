// Copyright 2024 Forecasting Technologies LTD.
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

use crate::{AmmTradeOf, BalanceOf, Config, OrderTradeOf};
use alloc::vec::Vec;
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, DispatchError};
use zeitgeist_primitives::math::checked_ops_res::{CheckedAddRes, CheckedSubRes};

/// Represents the strategy used when placing an order in a trading environment.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Encode,
    Decode,
    RuntimeDebug,
    MaxEncodedLen,
    TypeInfo,
)]
pub enum Strategy {
    /// The trade is rolled back if it cannot be executed fully.
    ImmediateOrCancel,
    /// Partially fulfills the order if possible, placing the remainder in the order book. Favors
    /// achieving a specific price rather than immediate execution.
    LimitOrder,
}

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, TypeInfo)]
pub enum TxType {
    Buy,
    Sell,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq, TypeInfo)]
pub enum Trade<'a, T: Config> {
    Orderbook(&'a OrderTradeOf<T>),
    Amm(AmmTradeOf<T>),
}

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, TypeInfo)]
pub struct TradeEventInfo<T: Config> {
    pub amount_out: BalanceOf<T>,
    pub external_fee_amount: BalanceOf<T>,
    pub swap_fee_amount: BalanceOf<T>,
}

impl<T: Config> TradeEventInfo<T> {
    pub fn new() -> Self {
        Self {
            amount_out: BalanceOf::<T>::zero(),
            external_fee_amount: BalanceOf::<T>::zero(),
            swap_fee_amount: BalanceOf::<T>::zero(),
        }
    }

    pub fn add_amount_out_minus_fees(&mut self, additional: Self) -> Result<(), DispatchError> {
        self.external_fee_amount =
            self.external_fee_amount.checked_add_res(&additional.external_fee_amount)?;
        self.swap_fee_amount = self.swap_fee_amount.checked_add_res(&additional.swap_fee_amount)?;
        let fees = additional.external_fee_amount.checked_add_res(&additional.swap_fee_amount)?;
        let amount_minus_fees = additional.amount_out.checked_sub_res(&fees)?;
        self.amount_out = self.amount_out.checked_add_res(&amount_minus_fees)?;

        Ok(())
    }

    pub fn add_amount_out_and_fees(&mut self, additional: Self) -> Result<(), DispatchError> {
        self.external_fee_amount =
            self.external_fee_amount.checked_add_res(&additional.external_fee_amount)?;
        self.swap_fee_amount = self.swap_fee_amount.checked_add_res(&additional.swap_fee_amount)?;
        self.amount_out = self.amount_out.checked_add_res(&additional.amount_out)?;

        Ok(())
    }
}

pub struct OrderAmmTradesInfo<T: Config> {
    pub remaining: BalanceOf<T>,
    pub order_trades: Vec<OrderTradeOf<T>>,
    pub amm_trades: Vec<AmmTradeOf<T>>,
}
