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
use frame_support::traits::tokens::fungibles::{Dust, Unbalanced};

impl<T: Config> Unbalanced<T::AccountId> for Pallet<T> {
    fn handle_raw_dust(asset: Self::AssetId, amount: Self::Balance) {
        let _ = route_call_with_trait!(asset, Unbalanced, handle_raw_dust, amount);
    }

    fn handle_dust(dust: Dust<T::AccountId, Self>) {
        let Dust(currency_id, amount) = dust;

        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            // Route "pre new asset system" market assets to `CurrencyType`
            if T::MarketAssets::asset_exists(asset) {
                T::MarketAssets::handle_dust(Dust(asset, amount));
            } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                T::Currencies::handle_dust(Dust(currency, amount));
            } else {
                T::MarketAssets::handle_dust(Dust(asset, amount));
            }
        } else if let Ok(asset) = T::CampaignAssetType::try_from(currency_id) {
            T::CampaignAssets::handle_dust(Dust(asset, amount));
        } else if let Ok(asset) = T::CustomAssetType::try_from(currency_id) {
            T::CustomAssets::handle_dust(Dust(asset, amount));
        } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            T::Currencies::handle_dust(Dust(currency, amount));
        }
    }

    fn write_balance(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> Result<Option<Self::Balance>, DispatchError> {
        route_call_with_trait!(asset, Unbalanced, write_balance, who, amount)?
    }

    fn set_total_issuance(asset: Self::AssetId, amount: Self::Balance) {
        let _ = route_call_with_trait!(asset, Unbalanced, set_total_issuance, amount);
    }

    fn decrease_balance(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
        precision: Precision,
        preservation: Preservation,
        force: Fortitude,
    ) -> Result<Self::Balance, DispatchError> {
        route_call_with_trait!(
            asset,
            Unbalanced,
            decrease_balance,
            who,
            amount,
            precision,
            preservation,
            force
        )?
    }

    fn increase_balance(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
        precision: Precision,
    ) -> Result<Self::Balance, DispatchError> {
        route_call_with_trait!(asset, Unbalanced, increase_balance, who, amount, precision)?
    }

    fn deactivate(asset: Self::AssetId, amount: Self::Balance) {
        let _ = route_call_with_trait!(asset, Unbalanced, deactivate, amount);
    }

    fn reactivate(asset: Self::AssetId, amount: Self::Balance) {
        let _ = route_call_with_trait!(asset, Unbalanced, reactivate, amount);
    }
}
