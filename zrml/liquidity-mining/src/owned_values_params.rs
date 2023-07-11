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

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};

/// Parameters used by the `OwnedValues` storage.
///
/// # Types
///
/// * `BA`: BAlance
/// * `BN`: Block Number
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[derive(
    scale_info::TypeInfo,
    Clone,
    Debug,
    Decode,
    Default,
    Encode,
    Eq,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub struct OwnedValuesParams<BA, BN>
where
    BA: MaxEncodedLen,
    BN: MaxEncodedLen,
{
    /// The number of blocks an account participated in a market period.
    pub participated_blocks: BN,
    /// Owned amount of perpetual incentives. Won't go away when accounts exist early and is not
    /// attached to any share
    pub perpetual_incentives: BA,
    /// Owned incentives. Related to the total number of shares.
    pub total_incentives: BA,
    /// Owned quantity of shares. Related to the total amount of incentives.
    pub total_shares: BA,
}
