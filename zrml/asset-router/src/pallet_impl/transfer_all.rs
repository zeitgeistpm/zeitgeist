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

impl<T: Config> TransferAll<T::AccountId> for Pallet<T> {
    #[require_transactional]
    fn transfer_all(source: &T::AccountId, dest: &T::AccountId) -> DispatchResult {
        // Only transfers assets maintained in orml-tokens, not implementable for pallet-assets
        <T::Currencies as TransferAll<T::AccountId>>::transfer_all(source, dest)
    }
}
