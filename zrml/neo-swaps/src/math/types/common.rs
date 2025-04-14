// Copyright 2024-2025 Forecasting Technologies LTD.
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

use crate::math::transcendental::exp;
use fixed::FixedU128;
use typenum::U80;
use zeitgeist_primitives::{
    constants::DECIMALS,
    math::fixed::{IntoFixedDecimal, IntoFixedFromDecimal},
};

type Fractional = U80;
pub(crate) type FixedType = FixedU128<Fractional>;

// 32.44892769177272
pub(crate) const EXP_NUMERICAL_THRESHOLD: FixedType =
    FixedType::from_bits(0x20_72EC_ECDA_6EBE_EACC_40C7);

pub(crate) fn to_fixed<B>(value: B) -> Option<FixedType>
where
    B: Into<u128> + From<u128>,
{
    value.to_fixed_from_fixed_decimal(DECIMALS).ok()
}

pub(crate) fn from_fixed<B>(value: FixedType) -> Option<B>
where
    B: Into<u128> + From<u128>,
{
    value.to_fixed_decimal(DECIMALS).ok()
}

/// Calculates `exp(value)` but returns `None` if `value` lies outside of the numerical
/// boundaries.
pub(crate) fn protected_exp(value: FixedType, neg: bool) -> Option<FixedType> {
    if value < EXP_NUMERICAL_THRESHOLD {
        exp(value, neg).ok()
    } else {
        None
    }
}
