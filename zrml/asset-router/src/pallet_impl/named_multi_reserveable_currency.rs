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

impl<T: Config> NamedMultiReservableCurrency<T::AccountId> for Pallet<T> {
    type ReserveIdentifier =
        <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::ReserveIdentifier;

    fn reserved_balance_named(
        id: &Self::ReserveIdentifier,
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
    ) -> Self::Balance {
        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            if T::MarketAssets::asset_exists(asset) {
                Self::log_unsupported(currency_id, "reserved_balance_named");
                return Zero::zero();
            }
        }
        if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            return <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::reserved_balance_named(
                id, currency, who,
            );
        }

        Self::log_unsupported(currency_id, "reserved_balance_named");
        Zero::zero()
    }

    fn reserve_named(
        id: &Self::ReserveIdentifier,
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        value: Self::Balance,
    ) -> DispatchResult {
        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            if T::MarketAssets::asset_exists(asset) {
                return Err(Error::<T>::Unsupported.into());
            }
        }
        if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            return <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::reserve_named(
                id, currency, who, value,
            );
        }

        Err(Error::<T>::Unsupported.into())
    }

    fn unreserve_named(
        id: &Self::ReserveIdentifier,
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        value: Self::Balance,
    ) -> Self::Balance {
        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            if T::MarketAssets::asset_exists(asset) {
                Self::log_unsupported(currency_id, "unreserve_named");
                return value;
            }
        }
        if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            return <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::unreserve_named(
                id, currency, who, value,
            );
        }

        Self::log_unsupported(currency_id, "unreserve_named");
        value
    }

    fn slash_reserved_named(
        id: &Self::ReserveIdentifier,
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        value: Self::Balance,
    ) -> Self::Balance {
        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            if T::MarketAssets::asset_exists(asset) {
                Self::log_unsupported(currency_id, "slash_reserved_named");
                return value;
            }
        }
        if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            return <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::slash_reserved_named(
                id, currency, who, value
            );
        }

        Self::log_unsupported(currency_id, "slash_reserved_named");
        value
    }

    fn repatriate_reserved_named(
        id: &Self::ReserveIdentifier,
        currency_id: Self::CurrencyId,
        slashed: &T::AccountId,
        beneficiary: &T::AccountId,
        value: Self::Balance,
        status: Status,
    ) -> Result<Self::Balance, DispatchError> {
        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            if T::MarketAssets::asset_exists(asset) {
                return Err(Error::<T>::Unsupported.into());
            }
        }
        if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            return <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::repatriate_reserved_named(
                id, currency, slashed, beneficiary, value, status
            );
        }

        Err(Error::<T>::Unsupported.into())
    }
}
