// Copyright 2022 Zeitgeist PM LLC.
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

use core::marker::PhantomData;
use frame_support::{
    dispatch::RawOrigin,
    traits::{EnsureOrigin, EnsureOriginWithArg},
};
use orml_traits::asset_registry::{AssetMetadata, AssetProcessor};
use sp_runtime::DispatchError;

use super::*;

#[derive(
    Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Debug, Encode, Decode, TypeInfo, MaxEncodedLen,
)]
/// Implements orml_traits::asset_registry::AssetProcessor. Does not apply any post checks.
/// Only pre check is to ensure an asset id was passed.
pub struct CustomAssetProcessor;

impl AssetProcessor<CurrencyId, AssetMetadata<Balance, CustomMetadata>> for CustomAssetProcessor {
    fn pre_register(
        id: Option<CurrencyId>,
        metadata: AssetMetadata<Balance, CustomMetadata>,
    ) -> Result<(CurrencyId, AssetMetadata<Balance, CustomMetadata>), DispatchError> {
        match id {
            Some(id) => Ok((id, metadata)),
            None => Err(DispatchError::Other("asset-registry: AssetId is required")),
        }
    }

    fn post_register(
        _id: CurrencyId,
        _asset_metadata: AssetMetadata<Balance, CustomMetadata>,
    ) -> Result<(), DispatchError> {
        Ok(())
    }
}

