// Copyright 2022 Forecasting Technologies Ltd.
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
// Copyright 2021 Centrifuge Foundation (centrifuge.io).
//
//
// Centrifuge is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version (see http://www.gnu.org/licenses).
// Centrifuge is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

#[macro_export]
macro_rules! impl_mock_registry {
    ($name:ident, $asset_id:ty, $balance:ty, $custom_metadata:ty) => {
        pub use orml_asset_registry_mock::$name;

        mod orml_asset_registry_mock {
            use frame_support::{
                dispatch::{
                    DispatchError,
                    DispatchResult,
                },
                traits::GenesisBuild,
            };
            use orml_traits::asset_registry::{
                AssetMetadata, Inspect,
                Mutate,
            };
            use xcm::{
                latest::prelude::MultiLocation,
                VersionedMultiLocation,
            };

            use super::*;

            pub struct $name;

            impl Inspect for $name {
                type AssetId = $asset_id;
                type Balance = $balance;
                type CustomMetadata = $custom_metadata;

                fn asset_id(location: &MultiLocation) -> Option<Self::AssetId> {
                    __private::STATE.with(|s| s.borrow().get_asset_from_location(location))
                }

                fn metadata(
                    asset_id: &Self::AssetId,
                ) -> Option<AssetMetadata<Self::Balance, Self::CustomMetadata>> {
                    __private::STATE.with(|s| s.borrow().get_meta(asset_id))
                }

                fn metadata_by_location(
                    location: &MultiLocation,
                ) -> Option<AssetMetadata<Self::Balance, Self::CustomMetadata>> {
                    __private::STATE.with(|s| s.borrow().get_meta_from_location(location))
                }

                fn location(
                    asset_id: &Self::AssetId,
                ) -> Result<Option<MultiLocation>, DispatchError> {
                    let maybe_location =
                        __private::STATE.with(|s| s.borrow().get_location(asset_id));

                    Ok(maybe_location)
                }
            }

            impl Mutate for $name {
                fn register_asset(
                    asset_id: Option<Self::AssetId>,
                    metadata: AssetMetadata<Self::Balance, Self::CustomMetadata>,
                ) -> DispatchResult {
                    if let Some(asset_id) = asset_id {
                        __private::STATE.with(|s| s.borrow_mut().insert_meta(&asset_id, metadata))
                    } else {
                        Err(DispatchError::Other(
                            "Mock can only register metadata with asset_id",
                        ))
                    }
                }

                fn update_asset(
                    asset_id: Self::AssetId,
                    decimals: Option<u32>,
                    name: Option<Vec<u8>>,
                    symbol: Option<Vec<u8>>,
                    existential_deposit: Option<Self::Balance>,
                    location: Option<Option<VersionedMultiLocation>>,
                    additional: Option<Self::CustomMetadata>,
                ) -> DispatchResult {
                    __private::STATE.with(|s| {
                        s.borrow_mut().update_asset(
                            asset_id,
                            decimals,
                            name,
                            symbol,
                            existential_deposit,
                            location,
                            additional,
                        )
                    })
                }
            }

            #[derive(Default)]
            pub struct GenesisConfig {
                pub metadata: Vec<($asset_id, AssetMetadata<$balance, $custom_metadata>)>,
            }

            use serde::{
                de::{
                    Deserialize, Deserializer,
                },
                ser::{
                    Serialize, SerializeStruct,
                    Serializer,
                },
            };

            impl GenesisBuild<()> for GenesisConfig {
                fn build(&self) {
                    for (asset, metadata) in &self.metadata {
                        __private::STATE
                            .with(|s| s.borrow_mut().insert_meta(asset, metadata.clone()))
                            .expect("Genesis must not fail")
                    }
                }
            }

            // NOTE: We need this dummy impl as `AssetMetadata` does NOT derive
            //       serialize in std
            impl Serialize for GenesisConfig {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    let state = serializer.serialize_struct("GenesisConfig", 1)?;
                    state.end()
                }
            }

            // NOTE: We need this dummy impl as `AssetMetadata` does NOT derive
            //       deserialize in std
            impl<'de> Deserialize<'de> for GenesisConfig {
                fn deserialize<D>(_deserializer: D) -> Result<GenesisConfig, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    Ok(GenesisConfig::default())
                }
            }

            mod __private {
                use std::{cell::RefCell, vec::Vec};

                use super::*;

                pub struct RegistryState {
                    pub location_to_asset: Vec<(MultiLocation, $asset_id)>,
                    pub metadata:
                        Vec<($asset_id, AssetMetadata<$balance, $custom_metadata>)>,
                }

                impl RegistryState {
                    pub fn get_meta(
                        &self,
                        asset_id: &$asset_id,
                    ) -> Option<AssetMetadata<$balance, $custom_metadata>> {
                        for (curr_id, meta) in &self.metadata {
                            if curr_id == asset_id {
                                return Some(meta.clone());
                            }
                        }

                        None
                    }

                    pub fn insert_meta(
                        &mut self,
                        asset_id: &$asset_id,
                        meta: AssetMetadata<$balance, $custom_metadata>,
                    ) -> DispatchResult {
                        for (curr_id, curr_meta) in &mut self.metadata {
                            if curr_id == asset_id {
                                *curr_meta = meta;
                                return Ok(());
                            }
                        }

                        self.metadata.push((asset_id.clone(), meta));
                        Ok(())
                    }

                    pub fn get_location(
                        &self,
                        asset_id: &$asset_id,
                    ) -> Option<MultiLocation> {
                        for (curr_id, meta) in &self.metadata {
                            if curr_id == asset_id {
                                return meta
                                    .location
                                    .as_ref()
                                    .map(|versioned| versioned.clone().try_into().ok())
                                    .flatten();
                            }
                        }

                        None
                    }

                    pub fn get_asset_from_location(
                        &self,
                        location: &MultiLocation,
                    ) -> Option<$asset_id> {
                        for (curr_location, asset_id) in &self.location_to_asset {
                            if curr_location == location {
                                return Some(asset_id.clone());
                            }
                        }

                        None
                    }

                    pub fn get_meta_from_location(
                        &self,
                        location: &MultiLocation,
                    ) -> Option<AssetMetadata<$balance, $custom_metadata>> {
                        let asset_id = self.get_asset_from_location(location)?;
                        self.get_meta(&asset_id)
                    }

                    pub fn update_asset(
                        &mut self,
                        asset_id: $asset_id,
                        _decimals: Option<u32>,
                        _name: Option<Vec<u8>>,
                        _symbol: Option<Vec<u8>>,
                        _existential_deposit: Option<$balance>,
                        _location: Option<Option<VersionedMultiLocation>>,
                        _additional: Option<$custom_metadata>,
                    ) -> DispatchResult {
                        if let Some(_meta) = self.get_meta(&asset_id) {
                            Ok(())
                        } else {
                            Err(DispatchError::Other("Asset not registered"))
                        }
                    }
                }

                impl RegistryState {
                    fn new() -> Self {
                        Self { location_to_asset: Vec::new(), metadata: Vec::new() }
                    }
                }

                thread_local! {
                    pub static STATE: RefCell<
                        RegistryState,
                    > = RefCell::new(RegistryState::new());
                }
            }
        }
    };
}

pub use impl_mock_registry;
