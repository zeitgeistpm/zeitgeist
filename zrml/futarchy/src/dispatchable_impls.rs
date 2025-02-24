// Copyright 2024-2025 Forecasting Technologies LTD.
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

use crate::{traits::ProposalStorage, types::Proposal, Config, Error, Event, Pallet};
use frame_support::{ensure, require_transactional, traits::Get};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{DispatchResult, Saturating};

impl<T: Config> Pallet<T> {
    #[require_transactional]
    pub(crate) fn do_submit_proposal(
        duration: BlockNumberFor<T>,
        proposal: Proposal<T>,
    ) -> DispatchResult {
        ensure!(duration >= T::MinDuration::get(), Error::<T>::DurationTooShort);

        let now = frame_system::Pallet::<T>::block_number();
        let to_be_scheduled_at = now.saturating_add(duration);

        <Pallet<T> as ProposalStorage<T>>::add(to_be_scheduled_at, proposal.clone())?;

        Self::deposit_event(Event::<T>::Submitted { duration, proposal });

        Ok(())
    }
}
