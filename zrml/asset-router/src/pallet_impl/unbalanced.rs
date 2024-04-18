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
use frame_support::traits::tokens::{
    fungibles::{Dust, Unbalanced},
    Fortitude, Precision, Preservation,
};

impl<T: Config> Unbalanced<T::AccountId> for Pallet<T> {
    fn handle_raw_dust(asset: Self::AssetId, amount: Self::Balance) {
        route_call_with_trait!(asset, Unbalanced, handle_raw_dust, amount)?
    }

    fn handle_dust(dust: Dust<T::AccountId, Self>) {
        unimplemented!();
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
        route_call_with_trait!(asset, Unbalanced, deactivate, amount)?
    }

    fn reactivate(asset: Self::AssetId, amount: Self::Balance) {
        route_call_with_trait!(asset, Unbalanced, reactivate, amount)?
    }
}
