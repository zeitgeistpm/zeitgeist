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

/// A trait for keeping track of a certain amount of work to be done.
pub trait CombinatorialTokensFuel {
    /// Creates a `Fuel` object from a `total` value which indicates the total amount of work to be
    /// done. This is usually done for benchmarking purposes.
    fn from_total(total: u32) -> Self;

    /// Returns a `u32` which indicates the total amount of work to be done. Must be `O(1)` to avoid
    /// excessive calculation if this call is used when calculating extrinsic weight.
    fn total(&self) -> u32;
}
