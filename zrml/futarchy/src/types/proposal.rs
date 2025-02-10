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

use crate::{BoundedCallOf, Config, OracleOf};
use frame_support::{CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(
    CloneNoBound, Decode, Encode, Eq, MaxEncodedLen, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[scale_info(skip_type_params(S, T))]
pub struct Proposal<T>
where
    T: Config,
{
    /// The time at which the proposal will be enacted.
    pub when: BlockNumberFor<T>,

    /// The proposed call.
    pub call: BoundedCallOf<T>,

    /// The oracle that evaluates if the proposal should be enacted.
    pub oracle: OracleOf<T>,
}
