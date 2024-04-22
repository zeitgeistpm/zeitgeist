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

impl<T: Config> MultiLockableCurrency<T::AccountId> for Pallet<T> {
    type Moment = BlockNumberFor<T>;

    fn set_lock(
        lock_id: LockIdentifier,
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            if T::MarketAssets::asset_exists(asset) {
                return Err(Error::<T>::Unsupported.into());
            }
        }
        if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            return <T::Currencies as MultiLockableCurrency<T::AccountId>>::set_lock(
                lock_id, currency, who, amount,
            );
        }

        Err(Error::<T>::Unsupported.into())
    }

    fn extend_lock(
        lock_id: LockIdentifier,
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            if T::MarketAssets::asset_exists(asset) {
                return Err(Error::<T>::Unsupported.into());
            }
        }
        if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            return <T::Currencies as MultiLockableCurrency<T::AccountId>>::extend_lock(
                lock_id, currency, who, amount,
            );
        }

        Err(Error::<T>::Unsupported.into())
    }

    fn remove_lock(
        lock_id: LockIdentifier,
        currency_id: Self::CurrencyId,
        who: &T::AccountId,
    ) -> DispatchResult {
        if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
            if T::MarketAssets::asset_exists(asset) {
                return Err(Error::<T>::Unsupported.into());
            }
        }
        if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
            return <T::Currencies as MultiLockableCurrency<T::AccountId>>::remove_lock(
                lock_id, currency, who,
            );
        }

        Err(Error::<T>::Unsupported.into())
    }
}
