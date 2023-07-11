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

use frame_support::dispatch::DispatchError;
use zeitgeist_primitives::constants::BASE;

pub const ARITHM_OF: DispatchError = DispatchError::Other("Arithmetic overflow");

/// The amount of precision to use in exponentiation.
pub const BPOW_PRECISION: u128 = 10;
/// The minimum value of the base parameter in bpow_approx.
pub const BPOW_APPROX_BASE_MIN: u128 = BASE / 4;
/// The maximum value of the base parameter in bpow_approx.
pub const BPOW_APPROX_BASE_MAX: u128 = 7 * BASE / 4;
/// The maximum number of terms from the binomial series used to calculate bpow_approx.
pub const BPOW_APPROX_MAX_ITERATIONS: u128 = 100;
