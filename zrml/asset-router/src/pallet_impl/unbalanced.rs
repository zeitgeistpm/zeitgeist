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
use frame_support::traits::tokens::fungibles::Unbalanced;

impl<T: Config> Unbalanced<T::AccountId> for Pallet<T> {
    fn set_balance(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> DispatchResult {
        route_call_with_trait!(asset, Unbalanced, set_balance, who, amount)?
    }

    fn set_total_issuance(asset: Self::AssetId, amount: Self::Balance) {
        let _ = route_call_with_trait!(asset, Unbalanced, set_total_issuance, amount);
    }

    fn decrease_balance(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        route_call_with_trait!(asset, Unbalanced, decrease_balance, who, amount)?
    }

    fn decrease_balance_at_most(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> Self::Balance {
        route_call_with_trait!(asset, Unbalanced, decrease_balance_at_most, who, amount)
            .unwrap_or(Zero::zero())
    }

    fn increase_balance(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        route_call_with_trait!(asset, Unbalanced, increase_balance, who, amount)?
    }

    fn increase_balance_at_most(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> Self::Balance {
        route_call_with_trait!(asset, Unbalanced, increase_balance_at_most, who, amount)
            .unwrap_or(Zero::zero())
    }
}
