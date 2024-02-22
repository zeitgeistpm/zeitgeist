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

impl<T: Config> MultiCurrency<T::AccountId> for Pallet<T> {
    type CurrencyId = T::AssetType;
    type Balance = T::Balance;

    fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
        let min_balance = route_call!(currency_id, minimum_balance, minimum_balance,);
        min_balance.unwrap_or_else(|_b| {
            Self::log_unsupported(currency_id, "minimum_balance");
            Self::Balance::zero()
        })
    }

    fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
        let total_issuance = route_call!(currency_id, total_issuance, total_issuance,);
        total_issuance.unwrap_or_else(|_b| {
            Self::log_unsupported(currency_id, "total_issuance");
            Self::Balance::zero()
        })
    }

    fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
        let total_balance = route_call!(currency_id, total_balance, balance, who);
        total_balance.unwrap_or_else(|_b| {
            Self::log_unsupported(currency_id, "total_balance");
            Self::Balance::zero()
        })
    }

    fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            // Route "pre new asset system" market assets to `CurrencyType`
            if T::MarketAssets::asset_exists(asset) {
                T::MarketAssets::reducible_balance(asset, who, false)
            } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                T::Currencies::free_balance(currency, who)
            } else {
                T::MarketAssets::reducible_balance(asset, who, false)
            }
        } else if let Ok(asset) = T::CampaignAssetType::try_from(currency_id) {
            T::CampaignAssets::reducible_balance(asset, who, false)
        } else if let Ok(asset) = T::CustomAssetType::try_from(currency_id) {
            T::CustomAssets::reducible_balance(asset, who, false)
        } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            T::Currencies::free_balance(currency, who)
        } else {
            Self::log_unsupported(currency_id, "free_balance");
            Self::Balance::zero()
        }
    }

    fn ensure_can_withdraw(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        let withdraw_consequence = if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            // Route "pre new asset system" market assets to `CurrencyType`
            if T::MarketAssets::asset_exists(asset) {
                T::MarketAssets::can_withdraw(asset, who, amount)
            } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                return T::Currencies::ensure_can_withdraw(currency, who, amount);
            } else {
                T::MarketAssets::can_withdraw(asset, who, amount)
            }
        } else if let Ok(asset) = T::CampaignAssetType::try_from(currency_id) {
            T::CampaignAssets::can_withdraw(asset, who, amount)
        } else if let Ok(asset) = T::CustomAssetType::try_from(currency_id) {
            T::CustomAssets::can_withdraw(asset, who, amount)
        } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            return T::Currencies::ensure_can_withdraw(currency, who, amount);
        } else {
            return Err(Error::<T>::UnknownAsset.into());
        };

        withdraw_consequence.into_result().map(|_| ())
    }

    fn transfer(
        currency_id: Self::CurrencyId,
        from: &T::AccountId,
        to: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            // Route "pre new asset system" market assets to `CurrencyType`
            if T::MarketAssets::asset_exists(asset) {
                T::MarketAssets::transfer(asset, from, to, amount, false).map(|_| ())
            } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                T::Currencies::transfer(currency, from, to, amount)
            } else {
                T::MarketAssets::transfer(asset, from, to, amount, false).map(|_| ())
            }
        } else if let Ok(asset) = T::CampaignAssetType::try_from(currency_id) {
            T::CampaignAssets::transfer(asset, from, to, amount, false).map(|_| ())
        } else if let Ok(asset) = T::CustomAssetType::try_from(currency_id) {
            T::CustomAssets::transfer(asset, from, to, amount, false).map(|_| ())
        } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            T::Currencies::transfer(currency, from, to, amount)
        } else {
            Err(Error::<T>::UnknownAsset.into())
        }
    }

    fn deposit(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        route_call!(currency_id, deposit, mint_into, who, amount)?
    }

    fn withdraw(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            // Route "pre new asset system" market assets to `CurrencyType`
            if T::MarketAssets::asset_exists(asset) {
                // Resulting balance can be ignored as `burn_from` ensures that the
                // requested amount can be burned.
                T::MarketAssets::burn_from(asset, who, amount).map(|_| ())
            } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                T::Currencies::withdraw(currency, who, amount)
            } else {
                T::MarketAssets::burn_from(asset, who, amount).map(|_| ())
            }
        } else if let Ok(asset) = T::CampaignAssetType::try_from(currency_id) {
            T::CampaignAssets::burn_from(asset, who, amount).map(|_| ())
        } else if let Ok(asset) = T::CustomAssetType::try_from(currency_id) {
            T::CustomAssets::burn_from(asset, who, amount).map(|_| ())
        } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            T::Currencies::withdraw(currency, who, amount)
        } else {
            Err(Error::<T>::UnknownAsset.into())
        }
    }

    fn can_slash(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> bool {
        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            // Route "pre new asset system" market assets to `CurrencyType`
            if T::MarketAssets::asset_exists(asset) {
                // Resulting balance can be ignored as `burn_from` ensures that the
                // requested amount can be burned.
                T::MarketAssets::reducible_balance(asset, who, false) >= value
            } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                T::Currencies::can_slash(currency, who, value)
            } else {
                T::MarketAssets::reducible_balance(asset, who, false) >= value
            }
        } else if let Ok(asset) = T::CampaignAssetType::try_from(currency_id) {
            T::CampaignAssets::reducible_balance(asset, who, false) >= value
        } else if let Ok(asset) = T::CustomAssetType::try_from(currency_id) {
            T::CustomAssets::reducible_balance(asset, who, false) >= value
        } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            T::Currencies::can_slash(currency, who, value)
        } else {
            Self::log_unsupported(currency_id, "can_slash");
            false
        }
    }

    fn slash(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> Self::Balance {
        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            // Route "pre new asset system" market assets to `CurrencyType`
            if T::MarketAssets::asset_exists(asset) {
                // Resulting balance can be ignored as `burn_from` ensures that the
                // requested amount can be burned.
                T::MarketAssets::slash(asset, who, amount)
                    .map(|b| amount.saturating_sub(b))
                    .unwrap_or_else(|_| amount)
            } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                T::Currencies::slash(currency, who, amount)
            } else {
                T::MarketAssets::slash(asset, who, amount)
                    .map(|b| amount.saturating_sub(b))
                    .unwrap_or_else(|_| amount)
            }
        } else if let Ok(asset) = T::CampaignAssetType::try_from(currency_id) {
            T::CampaignAssets::slash(asset, who, amount)
                .map(|b| amount.saturating_sub(b))
                .unwrap_or_else(|_| amount)
        } else if let Ok(asset) = T::CustomAssetType::try_from(currency_id) {
            T::CustomAssets::slash(asset, who, amount)
                .map(|b| amount.saturating_sub(b))
                .unwrap_or_else(|_| amount)
        } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            T::Currencies::slash(currency, who, amount)
        } else {
            Self::log_unsupported(currency_id, "slash");
            amount
        }
    }
}
