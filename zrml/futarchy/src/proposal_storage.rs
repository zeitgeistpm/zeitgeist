use crate::{
    traits::ProposalStorage, types::Proposal, Config, Error, Pallet, ProposalCount, Proposals,
    ProposalsOf,
};
use frame_support::{ensure, require_transactional, traits::Get};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::DispatchError;
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
}
