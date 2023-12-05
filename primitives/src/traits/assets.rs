// Copyright 2023 Forecasting Technologies LTD.
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

use frame_support::traits::tokens::fungibles::Destroy;
use sp_runtime::DispatchResult;

/// Manage the complete destruction of an asset.
pub trait ManagedDestroy<AccountId>: Destroy<AccountId> {
    /// Invoking this function will lead to a guaranteed complete destruction
    /// of an asset and the return of any deposits associated to it. The duration
    /// of the destrution process may vary.
    fn managed_destroy(
        asset: Self::AssetId,
        maybe_check_owner: Option<AccountId>,
    ) -> DispatchResult;

    /// Invoking this function will lead to a guaranteed complete destruction
    /// of an vector of assets and the return of any deposits associated to it. The duration
    /// of the destrution process may vary.
    fn managed_destroy_multi(
        asset: Vec<Self::AssetId>,
        maybe_check_owners: Option<Vec<Option<AccountId>>>,
    ) -> DispatchResult;
}
