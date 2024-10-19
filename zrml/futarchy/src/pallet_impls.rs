use crate::{Config, Event, Pallet, types::Proposal};
use zeitgeist_primitives::traits::FutarchyOracle;
use frame_support::{dispatch::RawOrigin, pallet_prelude::Weight, traits::schedule::DispatchTime};
use frame_support::traits::schedule::v3::Anon;

impl<T: Config> Pallet<T> {
    /// Evaluates `proposal` using the specified oracle and schedules the contained call if the
    /// oracle approves.
    pub(crate) fn maybe_schedule_proposal(proposal: Proposal<T>) -> Weight {
        let (evaluate_weight, approved) = proposal.oracle.evaluate();

        if approved {
            let result = T::Scheduler::schedule(
                DispatchTime::At(proposal.when),
                None,
                63,
                RawOrigin::Root.into(),
                proposal.call.clone(),
            );

            if result.is_ok() {
                Self::deposit_event(Event::<T>::Scheduled { proposal });
            } else {
                Self::deposit_event(Event::<T>::UnexpectedSchedulerError);
            }

            evaluate_weight // TODO Add benchmark!
        } else {
            Self::deposit_event(Event::<T>::Rejected { proposal });

            evaluate_weight
        }
    }
}
