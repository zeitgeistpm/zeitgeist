use crate::{types::Proposal, Config, ProposalsOf};
use alloc::{collections::BTreeMap, vec::Vec};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::DispatchError;

pub(crate) trait ProposalStorage<T>
where
    T: Config,
{
    /// Returns the number of proposals currently in flight.
    #[allow(dead_code)]
    fn count() -> u32;

    /// Schedule `proposal` for evaluation at `block_number`.
    fn add(block_number: BlockNumberFor<T>, proposal: Proposal<T>) -> Result<(), DispatchError>;

    /// Take all proposals scheduled at `block_number`.
    fn take(block_number: BlockNumberFor<T>) -> Result<ProposalsOf<T>, DispatchError>;

    /// Returns all proposals scheduled at `block_number`.
    #[allow(dead_code)]
    fn get(block_number: BlockNumberFor<T>) -> ProposalsOf<T>;

    /// Mutates all scheduled proposals.
    fn mutate_all<R, F>(mutator: F) -> Result<BTreeMap<BlockNumberFor<T>, Vec<R>>, DispatchError>
    where
        F: FnMut(&mut Proposal<T>) -> R;
}
