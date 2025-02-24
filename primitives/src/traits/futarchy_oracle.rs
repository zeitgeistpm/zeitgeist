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

use frame_support::pallet_prelude::Weight;

pub trait FutarchyOracle {
    type BlockNumber;

    /// Evaluates the query at the current block and returns the weight consumed and a `bool`
    /// indicating whether the query evaluated positively.
    fn evaluate(&self) -> (Weight, bool);

    /// Updates the oracle's data and returns the weight consumed.
    fn update(&mut self, now: Self::BlockNumber) -> Weight;
}
