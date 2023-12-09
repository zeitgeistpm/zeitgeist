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

use frame_support::Parameter;
use parity_scale_codec::MaxEncodedLen;
use scale_info::TypeInfo;
use sp_runtime::traits::{MaybeSerializeDeserialize, Member};

/// A trait implemented by asset ID types on Zeitgeist, based on trait bounds defined in
/// orml_tokens.
pub trait ZeitgeistAsset:
    Parameter + Member + Copy + MaxEncodedLen + MaybeSerializeDeserialize + Ord + TypeInfo
{
}

impl<T> ZeitgeistAsset for T where
    T: Parameter + Member + Copy + MaxEncodedLen + MaybeSerializeDeserialize + Ord + TypeInfo
{
}

/// A trait for asset ID providers on Zeitgeist which have an ID for Balancer pool shares.
///
/// # Generics
///
/// - `P`: The pool ID type.
pub trait PoolSharesId<P> {
    /// Returns the ID of the pool shares asset of the pool specified by `pool_id`.
    fn pool_shares_id(pool_id: P) -> Self;
}

/// Helper trait that lets developers to iterate over assets for testing and benchmarking.
///
/// # Generics
///
/// - `T`: The enumeration type.
pub trait ZeitgeistAssetEnumerator<T> {
    /// Maps `value` to an asset. The returned assets are pairwise distinct.
    fn create_asset_id(t: T) -> Self;
}
