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

/// This macro converts the invoked asset type into the respective
/// implementation that handles it and finally calls the $method on it.
macro_rules! route_call {
    ($currency_id:expr, $currency_method:ident, $asset_method:ident, $($args:expr),*) => {
        if let Ok(asset) = T::MarketAssetType::try_from($currency_id) {
            // Route "pre new asset system" market assets to `CurrencyType`
            if T::MarketAssets::asset_exists(asset) {
                Ok(T::MarketAssets::$asset_method(asset, $($args),*))
            } else {
                if let Ok(currency) = T::CurrencyType::try_from($currency_id) {
                    Ok(<T::Currencies as MultiCurrency<T::AccountId>>::$currency_method(currency, $($args),*))
                } else {
                    Ok(T::MarketAssets::$asset_method(asset, $($args),*))
                }
            }
        } else if let Ok(asset) = T::CampaignAssetType::try_from($currency_id) {
            Ok(T::CampaignAssets::$asset_method(asset, $($args),*))
        } else if let Ok(asset) = T::CustomAssetType::try_from($currency_id)  {
            Ok(T::CustomAssets::$asset_method(asset, $($args),*))
        } else if let Ok(currency) = T::CurrencyType::try_from($currency_id) {
            Ok(<T::Currencies as MultiCurrency<T::AccountId>>::$currency_method(currency, $($args),*))
        } else {
            Err(Error::<T>::UnknownAsset)
        }
    };
}

macro_rules! route_call_with_trait {
    ($currency_id:expr, $trait:ident, $method:ident, $($args:expr),*) => {
        if let Ok(asset) = T::MarketAssetType::try_from($currency_id) {
            // Route "pre new asset system" market assets to `CurrencyType`
            if T::MarketAssets::asset_exists(asset) {
                Ok(<T::MarketAssets as $trait<T::AccountId>>::$method(asset, $($args),*))
            } else {
                if let Ok(currency) = T::CurrencyType::try_from($currency_id) {
                    Ok(<T::Currencies as $trait<T::AccountId>>::$method(currency, $($args),*))
                } else {
                    Ok(<T::MarketAssets as $trait<T::AccountId>>::$method(asset, $($args),*))
                }
            }
        } else if let Ok(asset) = T::CampaignAssetType::try_from($currency_id) {
            Ok(<T::CampaignAssets as $trait<T::AccountId>>::$method(asset, $($args),*))
        } else if let Ok(asset) = T::CustomAssetType::try_from($currency_id)  {
            Ok(<T::CustomAssets as $trait<T::AccountId>>::$method(asset, $($args),*))
        } else if let Ok(currency) = T::CurrencyType::try_from($currency_id) {
            Ok(<T::Currencies as $trait<T::AccountId>>::$method(currency, $($args),*))
        } else {
            Err(Error::<T>::UnknownAsset)
        }
    };
}

/// This macro delegates a call to Currencies if the asset represents a currency, otherwise
/// it returns an error.
macro_rules! only_currency {
    ($currency_id:expr, $error:expr, $currency_trait:ident, $currency_method:ident, $($args:expr),+) => {
        if let Ok(asset) = T::MarketAssetType::try_from($currency_id) {
            // Route "pre new asset system" market assets to `CurrencyType`
            if T::MarketAssets::asset_exists(asset) {
                Self::log_unsupported($currency_id, stringify!($currency_method));
                $error
            } else {
                if let Ok(currency) = T::CurrencyType::try_from($currency_id) {
                    <T::Currencies as $currency_trait<T::AccountId>>::$currency_method(currency, $($args),+)
                } else {
                    Self::log_unsupported($currency_id, stringify!($currency_method));
                    $error
                }
            }
        }
        else if let Ok(currency) = T::CurrencyType::try_from($currency_id) {
            <T::Currencies as $currency_trait<T::AccountId>>::$currency_method(currency, $($args),+)
        } else {
            Self::log_unsupported($currency_id, stringify!($currency_method));
            $error
        }
    };
}

/// This macro delegates a call to one *Asset instance if the asset does not represent a currency,
/// otherwise it returns an error.
macro_rules! only_asset {
    ($asset_id:expr, $error:expr, $asset_trait:ident, $asset_method:ident, $($args:expr),*) => {
        if let Ok(asset) = T::MarketAssetType::try_from($asset_id) {
            <T::MarketAssets as $asset_trait<T::AccountId>>::$asset_method(asset, $($args),*)
        } else if let Ok(asset) = T::CampaignAssetType::try_from($asset_id) {
            T::CampaignAssets::$asset_method(asset, $($args),*)
        } else if let Ok(asset) = T::CustomAssetType::try_from($asset_id)  {
            T::CustomAssets::$asset_method(asset, $($args),*)
        } else {
            Self::log_unsupported($asset_id, stringify!($asset_method));
            $error
        }
    };
}

/// This macro handles the single stages of the asset destruction.
macro_rules! handle_asset_destruction {
    ($asset:expr, $remaining_weight:expr, $asset_storage:expr, $asset_method:ident, $outer_loop:tt) => {
        let state_before = *($asset.state());
        let call_result = Self::$asset_method($asset, $remaining_weight);
        match call_result {
            Ok(DestructionOk::Incomplete(weight)) => {
                // Should be infallible since the asset was just popped and force inserting
                // is not possible.
                if let Err(e) = $asset_storage.try_insert($asset_storage.len(), *($asset)) {
                    log::error!(
                        target: LOG_TARGET,
                        "Cannot reintroduce asset {:?} into DestroyAssets storage: {:?}",
                        $asset,
                        e
                    );
                }

                $remaining_weight = weight;
                break $outer_loop;
            },
            Ok(DestructionOk::Complete(weight)) | Err(DestructionError::WrongState(weight))  => {
                $remaining_weight = weight;
            },
            Err(DestructionError::Indestructible(weight)) => {
                $remaining_weight = weight;

                if state_before != DestructionState::Finalization {
                    break $outer_loop;
                } else {
                    // In case destruction failed during finalization, there is most likely still
                    // some weight available.
                    break;
                }
            }
        }
    };
}
