// Copyright 2023 Forecasting Technologies LTD.
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

use frame_support::dispatch::DispatchError;
use sp_arithmetic::{
    traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub},
    ArithmeticError,
};

pub trait CheckedAddRes
where
    Self: Sized,
{
    fn checked_add_res(&self, other: &Self) -> Result<Self, DispatchError>;
}

pub trait CheckedSubRes
where
    Self: Sized,
{
    fn checked_sub_res(&self, other: &Self) -> Result<Self, DispatchError>;
}

pub trait CheckedMulRes
where
    Self: Sized,
{
    fn checked_mul_res(&self, other: &Self) -> Result<Self, DispatchError>;
}

pub trait CheckedDivRes
where
    Self: Sized,
{
    fn checked_div_res(&self, other: &Self) -> Result<Self, DispatchError>;
}

impl<T> CheckedAddRes for T
where
    T: CheckedAdd,
{
    #[inline]
    fn checked_add_res(&self, other: &Self) -> Result<Self, DispatchError> {
        self.checked_add(other).ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))
    }
}

impl<T> CheckedSubRes for T
where
    T: CheckedSub,
{
    #[inline]
    fn checked_sub_res(&self, other: &Self) -> Result<Self, DispatchError> {
        self.checked_sub(other).ok_or(DispatchError::Arithmetic(ArithmeticError::Underflow))
    }
}

impl<T> CheckedMulRes for T
where
    T: CheckedMul,
{
    #[inline]
    fn checked_mul_res(&self, other: &Self) -> Result<Self, DispatchError> {
        self.checked_mul(other).ok_or(DispatchError::Arithmetic(ArithmeticError::Overflow))
    }
}

impl<T> CheckedDivRes for T
where
    T: CheckedDiv,
{
    #[inline]
    fn checked_div_res(&self, other: &Self) -> Result<Self, DispatchError> {
        self.checked_div(other).ok_or(DispatchError::Arithmetic(ArithmeticError::DivisionByZero))
    }
}
