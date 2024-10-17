use crate::{traits::OracleQuery, Config, Event, Pallet, ProposalOf};
use frame_support::{dispatch::RawOrigin, pallet_prelude::Weight, traits::schedule::DispatchTime};
use frame_support::traits::schedule::v3::Anon;

impl<T: Config> Pallet<T> {
    /// Evaluates `proposal` using the specified oracle and schedules the contained call if the
    /// oracle approves.
    pub(crate) fn maybe_schedule_proposal(proposal: ProposalOf<T>) -> Weight {
        let (evaluate_weight, approved) = proposal.query.evaluate();

        if approved {
            let result = T::Scheduler::schedule(
                DispatchTime::At(proposal.when),
                None,
                63,
                RawOrigin::Root.into(),
                proposal.call,
            );

            if result.is_ok() {
                Self::deposit_event(Event::<T>::Scheduled);
            } else {
                Self::deposit_event(Event::<T>::UnexpectedSchedulerError);
            }

            evaluate_weight // TODO Add benchmark!
        } else {
            Self::deposit_event(Event::<T>::Rejected);

            evaluate_weight
        }
    }
}
