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

use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Debug, TypeInfo)]
pub struct AmmTrade<Balance> {
    pub amount_in: Balance,
    pub amount_out: Balance,
    pub swap_fee_amount: Balance,
    pub external_fee_amount: Balance,
}

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Debug, TypeInfo)]
pub struct ExternalFee<AccountId, Balance> {
    pub account: AccountId,
    pub amount: Balance,
}

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Debug, TypeInfo)]
pub struct OrderbookTrade<AccountId, Balance> {
    pub filled_maker_amount: Balance,
    pub filled_taker_amount: Balance,
    pub external_fee: ExternalFee<AccountId, Balance>,
}

pub trait FailSoft {}

#[derive(Debug)]
pub enum AmmSoftFail {
    Numerical,
}

impl FailSoft for AmmSoftFail {}

#[derive(Debug)]
pub enum OrderbookSoftFail {
    BelowMinimumBalance,
    PartialFillNearFullFillNotAllowed,
}

impl FailSoft for OrderbookSoftFail {}

#[derive(Debug)]
pub enum ApiError<S: FailSoft> {
    SoftFailure(S),
    HardFailure(DispatchError),
}
