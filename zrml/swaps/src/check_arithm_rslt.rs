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
//
// This file incorporates work covered by the license above but
// published without copyright notice by Balancer Labs
// (<https://balancer.finance>, contact@balancer.finance) in the
// balancer-core repository
// <https://github.com/balancer-labs/balancer-core>.

use crate::consts::ARITHM_OF;
use frame_support::dispatch::DispatchError;
use sp_runtime::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};

/// Check Arithmetic - Result
///
/// Checked arithmetic operations returning `Result<_, DispatchError>`.
pub trait CheckArithmRslt: CheckedAdd + CheckedDiv + CheckedMul + CheckedSub {
    /// Check Addition - Result
    ///
    /// Same as `sp_runtime::traits::CheckedAdd::checked_add` but returns a
    /// `Result` instead of `Option`.
    fn check_add_rslt(&self, n: &Self) -> Result<Self, DispatchError>;

    /// Check Division - Result
    ///
    /// Same as `sp_runtime::traits::CheckedDiv::checked_div` but returns a
    /// `Result` instead of `Option`.
    fn check_div_rslt(&self, n: &Self) -> Result<Self, DispatchError>;

    /// Check Multiplication - Result
    ///
    /// Same as `sp_runtime::traits::CheckedMul::checked_mul` but returns a
    /// `Result` instead of `Option`.
    fn check_mul_rslt(&self, n: &Self) -> Result<Self, DispatchError>;

    /// Check Subtraction - Result
    ///
    /// Same as `sp_runtime::traits::CheckedSub::checked_sub` but returns a
    /// `Result` instead of `Option`.
    fn check_sub_rslt(&self, n: &Self) -> Result<Self, DispatchError>;
}

impl<T> CheckArithmRslt for T
where
    T: CheckedAdd + CheckedDiv + CheckedMul + CheckedSub,
{
    #[inline]
    fn check_add_rslt(&self, n: &Self) -> Result<Self, DispatchError> {
        self.checked_add(n).ok_or(ARITHM_OF)
    }

    #[inline]
    fn check_div_rslt(&self, n: &Self) -> Result<Self, DispatchError> {
        self.checked_div(n).ok_or(ARITHM_OF)
    }

    #[inline]
    fn check_mul_rslt(&self, n: &Self) -> Result<Self, DispatchError> {
        self.checked_mul(n).ok_or(ARITHM_OF)
    }

    #[inline]
    fn check_sub_rslt(&self, n: &Self) -> Result<Self, DispatchError> {
        self.checked_sub(n).ok_or(ARITHM_OF)
    }
}
