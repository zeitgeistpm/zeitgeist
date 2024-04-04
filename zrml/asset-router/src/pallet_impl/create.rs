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

impl<T: Config> Create<T::AccountId> for Pallet<T> {
    fn create(
        id: Self::AssetId,
        admin: T::AccountId,
        is_sufficient: bool,
        min_balance: Self::Balance,
    ) -> DispatchResult {
        only_asset!(
            id,
            Err(Error::<T>::Unsupported.into()),
            Create,
            create,
            admin,
            is_sufficient,
            min_balance
        )
    }
}
