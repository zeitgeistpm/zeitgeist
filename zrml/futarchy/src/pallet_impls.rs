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

use crate::{types::Proposal, weights::WeightInfoZeitgeist, Config, Event, Pallet};
use frame_support::{
    dispatch::RawOrigin,
    pallet_prelude::Weight,
    traits::schedule::{v3::Anon, DispatchTime, HARD_DEADLINE},
};
use zeitgeist_primitives::traits::FutarchyOracle;

impl<T: Config> Pallet<T> {
    /// Evaluates `proposal` using the specified oracle and schedules the contained call if the
    /// oracle approves.
    pub(crate) fn maybe_schedule_proposal(proposal: Proposal<T>) -> Weight {
        let (evaluate_weight, approved) = proposal.oracle.evaluate();

        if approved {
            let result = T::Scheduler::schedule(
                DispatchTime::At(proposal.when),
                None,
                HARD_DEADLINE,
                RawOrigin::Root.into(),
                proposal.call.clone(),
            );

            if result.is_ok() {
                Self::deposit_event(Event::<T>::Scheduled { proposal });
            } else {
                Self::deposit_event(Event::<T>::UnexpectedSchedulerError);
            }
        } else {
            Self::deposit_event(Event::<T>::Rejected { proposal });
        }

        T::WeightInfo::maybe_schedule_proposal().saturating_add(evaluate_weight)
    }
}
