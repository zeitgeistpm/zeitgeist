use crate::{
    traits::ProposalStorage, types::Proposal, Config, Error, Pallet, ProposalCount, Proposals,
    ProposalsOf,
};
use alloc::vec::Vec;
use frame_support::{ensure, require_transactional, traits::Get};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{DispatchError, SaturatedConversion};
use zeitgeist_primitives::math::checked_ops_res::{CheckedIncRes, CheckedSubRes};

impl<T> ProposalStorage<T> for Pallet<T>
where
    T: Config,
{
    fn count() -> u32 {
        ProposalCount::<T>::get()
    }

    #[require_transactional]
    fn add(block_number: BlockNumberFor<T>, proposal: Proposal<T>) -> Result<(), DispatchError> {
        let proposal_count = ProposalCount::<T>::get();
        ensure!(proposal_count < T::MaxProposals::get(), Error::<T>::CacheFull);

        let new_proposal_count = proposal_count.checked_inc_res()?;
        ProposalCount::<T>::put(new_proposal_count);

        // Can't error unless state is invalid.
        let mutate_result = Proposals::<T>::try_mutate(block_number, |proposals| {
            proposals.try_push(proposal).map_err(|_| Error::<T>::CacheFull)
        });

        Ok(mutate_result?)
    }

    /// Take all proposals scheduled at `block_number`.
    fn take(block_number: BlockNumberFor<T>) -> Result<ProposalsOf<T>, DispatchError> {
        let proposals = Proposals::<T>::take(block_number);

        // Can't error unless state is invalid.
        let proposal_count = ProposalCount::<T>::get();
        let proposals_len: u32 = proposals.len().try_into().map_err(|_| Error::<T>::CacheFull)?;
        let new_proposal_count = proposal_count.checked_sub_res(&proposals_len)?;
        ProposalCount::<T>::put(new_proposal_count);

        Ok(proposals)
    }

    /// Returns all proposals scheduled at `block_number`.
    fn get(block_number: BlockNumberFor<T>) -> ProposalsOf<T> {
        Proposals::<T>::get(block_number)
    }

    fn try_mutate_all<F>(mut mutator: F) -> Result<(), DispatchError>
    where
        F: FnMut(&mut Proposal<T>),
    {
        // Collect keys to avoid iterating over the keys whilst modifying the map. Won't saturate
        // unless `usize` has fewer bits than `u32` for some reason.
        let keys: Vec<_> =
            Proposals::<T>::iter_keys().take(T::MaxProposals::get().saturated_into()).collect();

        for k in keys.into_iter() {
            let proposals = Self::get(k);

            // If mutation goes out of bounds, we've clearly failed.
            let proposals = proposals
                .try_mutate(|v| {
                    for p in v.iter_mut() {
                        mutator(p); // TODO Use weight.
                    }
                })
                .ok_or(Error::<T>::UnexpectedStorageFailure)?;

            Proposals::<T>::insert(k, proposals);
        }

        Ok(())
    }
}
