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

impl<T: Config> Destroy<T::AccountId> for Pallet<T> {
    fn start_destroy(id: Self::AssetId, maybe_check_owner: Option<T::AccountId>) -> DispatchResult {
        only_asset!(
            id,
            Err(Error::<T>::Unsupported.into()),
            Destroy,
            start_destroy,
            maybe_check_owner
        )
    }

    fn destroy_accounts(id: Self::AssetId, max_items: u32) -> Result<u32, DispatchError> {
        only_asset!(id, Err(Error::<T>::Unsupported.into()), Destroy, destroy_accounts, max_items)
    }

    fn destroy_approvals(id: Self::AssetId, max_items: u32) -> Result<u32, DispatchError> {
        only_asset!(id, Err(Error::<T>::Unsupported.into()), Destroy, destroy_approvals, max_items)
    }

    fn finish_destroy(id: Self::AssetId) -> DispatchResult {
        only_asset!(id, Err(Error::<T>::Unsupported.into()), Destroy, finish_destroy,)
    }
}
