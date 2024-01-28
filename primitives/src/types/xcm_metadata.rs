// Copyright 2022-2024 Forecasting Technologies LTD.
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

use crate::types::Balance;
pub use crate::{
    asset::*, market::*, max_runtime_usize::*, outcome_report::OutcomeReport, proxy_type::*,
    serde_wrapper::*,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(
    Clone,
    Copy,
    Debug,
    Decode,
    Default,
    Encode,
    Eq,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    TypeInfo,
)]
/// Custom XC asset metadata
pub struct CustomMetadata {
    /// XCM-related metadata.
    pub xcm: XcmMetadata,

    /// Whether an asset can be used as base_asset in pools.
    pub allow_as_base_asset: bool,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Decode,
    Default,
    Encode,
    Eq,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    TypeInfo,
)]
pub struct XcmMetadata {
    /// The factor used to determine the fee.
    /// It is multiplied by the fee that would have been paid in native currency, so it represents
    /// the ratio `native_price / other_asset_price`. It is a fixed point decimal number containing
    /// as many fractional decimals as the asset it is used for contains.
    /// Should be updated regularly.
    pub fee_factor: Option<Balance>,
}
