use crate::{Config, Error, Pallet, types::Proposal, Proposals};
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

        let try_mutate_result = Proposals::<T>::try_mutate(to_be_scheduled_at, |proposals| {
            proposals.try_push(proposal).map_err(|_| Error::<T>::CacheFull)
        });

        Ok(try_mutate_result?)
    }
}
