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
