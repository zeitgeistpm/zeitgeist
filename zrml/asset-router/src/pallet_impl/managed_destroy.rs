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

use crate::pallet::*;

impl<T: Config> Pallet<T> {
    fn add_asset_to_managed_destruction(
        destroy_assets: &mut DestroyAssetsT<T>,
        asset: T::AssetType,
        maybe_check_owner: Option<T::AccountId>,
    ) -> DispatchResult {
        ensure!(Self::asset_exists(asset), Error::<T>::UnknownAsset);
        frame_support::ensure!(!destroy_assets.is_full(), Error::<T>::TooManyManagedDestroys);
        let asset_to_insert = AssetInDestruction::new(asset);

        let idx = match destroy_assets.binary_search(&asset_to_insert) {
            Ok(_) => return Err(Error::<T>::DestructionInProgress.into()),
            Err(idx) => {
                if IndestructibleAssets::<T>::get().binary_search(&asset).is_ok() {
                    return Err(Error::<T>::AssetIndestructible.into());
                }

                idx
            }
        };

        destroy_assets
            .try_insert(idx, asset_to_insert)
            .map_err(|_| Error::<T>::TooManyManagedDestroys)?;

        Self::start_destroy(asset, maybe_check_owner)?;
        Ok(())
    }
}

impl<T: Config> ManagedDestroy<T::AccountId> for Pallet<T> {
    fn managed_destroy(
        asset: Self::AssetId,
        maybe_check_owner: Option<T::AccountId>,
    ) -> DispatchResult {
        let mut destroy_assets = DestroyAssets::<T>::get();
        Self::add_asset_to_managed_destruction(&mut destroy_assets, asset, maybe_check_owner)?;
        DestroyAssets::<T>::put(destroy_assets);
        Ok(())
    }

    fn managed_destroy_multi(
        assets: BTreeMap<Self::AssetId, Option<T::AccountId>>,
    ) -> DispatchResult {
        let mut destroy_assets = DestroyAssets::<T>::get();

        for (asset, maybe_check_owner) in assets {
            Self::add_asset_to_managed_destruction(&mut destroy_assets, asset, maybe_check_owner)?;
        }

        DestroyAssets::<T>::put(destroy_assets);
        Ok(())
    }
}
