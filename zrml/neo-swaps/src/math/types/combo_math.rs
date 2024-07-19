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
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//     Copyright (c) 2019 Alain Brenzikofer, modified by GalacticCouncil(2021)
//
//     Licensed under the Apache License, Version 2.0 (the "License");
//     you may not use this file except in compliance with the License.
//     You may obtain a copy of the License at
//
//          http://www.apache.org/licenses/LICENSE-2.0
//
//     Unless required by applicable law or agreed to in writing, software
//     distributed under the License is distributed on an "AS IS" BASIS,
//     WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//     See the License for the specific language governing permissions and
//     limitations under the License.
//
//     Original source: https://github.com/encointer/substrate-fixed
//
// The changes applied are: Re-used and extended tests for `exp` and other
// functions.

use crate::{
    math::{
        traits::ComboMathOps,
        transcendental::{exp, ln},
    },
    BalanceOf, Config, Error,
};
use alloc::vec::Vec;
use core::marker::PhantomData;
use fixed::FixedU128;
use sp_runtime::{
    traits::{One, Zero},
    DispatchError, SaturatedConversion,
};
use typenum::U80;

type Fractional = U80;
type FixedType = FixedU128<Fractional>;

// 32.44892769177272
const EXP_OVERFLOW_THRESHOLD: FixedType = FixedType::from_bits(0x20_72EC_ECDA_6EBE_EACC_40C7);

pub(crate) struct ComboMath<T>(PhantomData<T>);

impl<T> ComboMathOps<T> for ComboMath<T>
where
    T: Config,
{
    fn calculate_swap_amount_out_for_buy(
        buy: Vec<BalanceOf<T>>,
        sell: Vec<BalanceOf<T>>,
        amount_in: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        Ok(Zero::zero())
    }

    fn calculate_swap_amount_out_for_sell(
        buy: Vec<BalanceOf<T>>,
        sell: Vec<BalanceOf<T>>,
        amount_in: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        Ok(Zero::zero())
    }
}
