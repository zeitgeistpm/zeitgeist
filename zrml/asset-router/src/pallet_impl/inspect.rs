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

// Supertrait of Create and Destroy
impl<T: Config> Inspect<T::AccountId> for Pallet<T> {
    type AssetId = T::AssetType;
    type Balance = T::Balance;

    fn total_issuance(asset: Self::AssetId) -> Self::Balance {
        route_call_with_trait!(asset, Inspect, total_issuance,).unwrap_or(Zero::zero())
    }

    fn active_issuance(asset: Self::AssetId) -> Self::Balance {
        route_call_with_trait!(asset, Inspect, active_issuance,).unwrap_or(Zero::zero())
    }

    fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
        route_call_with_trait!(asset, Inspect, minimum_balance,).unwrap_or(Zero::zero())
    }

    fn balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        route_call_with_trait!(asset, Inspect, balance, who).unwrap_or(Zero::zero())
    }

    fn total_balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        route_call_with_trait!(asset, Inspect, total_balance, who).unwrap_or(Zero::zero())
    }

    fn reducible_balance(
        asset: Self::AssetId,
        who: &T::AccountId,
        preservation: Preservation,
        force: Fortitude,
    ) -> Self::Balance {
        route_call_with_trait!(asset, Inspect, reducible_balance, who, preservation, force)
            .unwrap_or(Zero::zero())
    }

    fn can_deposit(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
        provenance: Provenance,
    ) -> DepositConsequence {
        route_call_with_trait!(asset, Inspect, can_deposit, who, amount, provenance)
            .unwrap_or(DepositConsequence::UnknownAsset)
    }

    fn can_withdraw(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        route_call_with_trait!(asset, Inspect, can_withdraw, who, amount)
            .unwrap_or(WithdrawConsequence::UnknownAsset)
    }

    fn asset_exists(asset: Self::AssetId) -> bool {
        route_call_with_trait!(asset, Inspect, asset_exists,).unwrap_or(false)
    }
}
