// Copyright 2024 Forecasting Technologies LTD.
// Copyright 2023 Parity Technologies (UK) Ltd.

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

/// Provides `checked_div_per_component` implementation to determine the
/// smallest division result between two `ref_time` and `proof_size`.
/// To be removed once sp-weights is upgraded to polkadot-v0.9.39
use frame_support::pallet_prelude::Weight;

pub trait CheckedDivPerComponent {
    /// Calculates how many `other` fit into `self`.
    ///
    /// Divides each component of `self` against the same component of `other`. Returns the minimum
    /// of all those divisions. Returns `None` in case **all** components of `other` are zero.
    ///
    /// This returns `Some` even if some components of `other` are zero as long as there is at least
    /// one non-zero component in `other`. The division for this particular component will then
    /// yield the maximum value (e.g u64::MAX). This is because we assume not every operation and
    /// hence each `Weight` will necessarily use each resource.
    fn checked_div_per_component(self, other: &Self) -> Option<u64>;
}

impl CheckedDivPerComponent for Weight {
    fn checked_div_per_component(self, other: &Self) -> Option<u64> {
        let mut all_zero = true;
        let ref_time = match self.ref_time().checked_div(other.ref_time()) {
            Some(ref_time) => {
                all_zero = false;
                ref_time
            }
            None => u64::MAX,
        };
        let proof_size = match self.proof_size().checked_div(other.proof_size()) {
            Some(proof_size) => {
                all_zero = false;
                proof_size
            }
            None => u64::MAX,
        };
        if all_zero {
            None
        } else {
            Some(if ref_time < proof_size { ref_time } else { proof_size })
        }
    }
}
