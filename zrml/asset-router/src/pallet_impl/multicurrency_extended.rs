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
    fn update_balance_asset(
        currency_id: <Self as MultiCurrency<T::AccountId>>::CurrencyId,
        who: &T::AccountId,
        by_amount: <Self as MultiCurrencyExtended<T::AccountId>>::Amount,
    ) -> DispatchResult {
        if by_amount.is_zero() {
            return Ok(());
        }

        // Ensure that no overflows happen during abs().
        let by_amount_abs =
            if by_amount == <Self as MultiCurrencyExtended<T::AccountId>>::Amount::min_value() {
                return Err(Error::<T>::AmountIntoBalanceFailed.into());
            } else {
                by_amount.abs()
            };

        let by_balance =
            TryInto::<<Self as MultiCurrency<T::AccountId>>::Balance>::try_into(by_amount_abs)
                .map_err(|_| Error::<T>::AmountIntoBalanceFailed)?;
        if by_amount.is_positive() {
            Self::deposit(currency_id, who, by_balance)
        } else {
            Self::withdraw(currency_id, who, by_balance).map(|_| ())
        }
    }
}

impl<T: Config> MultiCurrencyExtended<T::AccountId> for Pallet<T> {
    type Amount = <T::Currencies as MultiCurrencyExtended<T::AccountId>>::Amount;

    fn update_balance(
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        by_amount: Self::Amount,
    ) -> DispatchResult {
        if by_amount.is_zero() {
            return Ok(());
        }

        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            // Route "pre new asset system" market assets to `CurrencyType`
            if T::MarketAssets::asset_exists(asset) {
                Self::update_balance_asset(currency_id, who, by_amount)
            } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                T::Currencies::update_balance(currency, who, by_amount)
            } else {
                Self::update_balance_asset(currency_id, who, by_amount)
            }
        } else if let Ok(_asset) = T::CampaignAssetType::try_from(currency_id) {
            Self::update_balance_asset(currency_id, who, by_amount)
        } else if let Ok(_asset) = T::CustomAssetType::try_from(currency_id) {
            Self::update_balance_asset(currency_id, who, by_amount)
        } else if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            T::Currencies::update_balance(currency, who, by_amount)
        } else {
            Err(Error::<T>::UnknownAsset.into())
        }
    }
}
