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

use crate::{BoundedVec, Config, ConstU32, Weight, MAX_ASSETS_IN_DESTRUCTION};
use core::cmp::Ordering;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub(crate) type DestroyAssetsT<T> = BoundedVec<
    AssetInDestruction<<T as Config>::AssetType>,
    ConstU32<{ MAX_ASSETS_IN_DESTRUCTION }>,
>;

pub(crate) enum DestructionOk {
    Complete(Weight),
    Incomplete(Weight),
}

pub(crate) enum DestructionError {
    Indestructible(Weight),
    WrongState(Weight),
}

pub(crate) type DestructionResult = Result<DestructionOk, DestructionError>;

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Decode, Encode, MaxEncodedLen, TypeInfo,
)]
pub(crate) enum DestructionState {
    Accounts,
    Approvals,
    Finalization,
    Destroyed,
    Indestructible,
}
pub(crate) const DESTRUCTION_STATES: u8 = 5;

#[derive(Clone, Copy, Encode, Eq, Debug, Decode, MaxEncodedLen, PartialEq, TypeInfo)]
pub(crate) struct AssetInDestruction<A> {
    asset: A,
    state: DestructionState,
}

impl<A> PartialOrd for AssetInDestruction<A>
where
    A: Eq + Ord + PartialEq + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Ordering for binary search of assets in destruction.
// Prioritize asset state first, then asset.
impl<A> Ord for AssetInDestruction<A>
where
    A: Eq + Ord + PartialEq + PartialOrd,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self.state.cmp(&other.state) {
            Ordering::Equal => {
                // Since asset destruction will always pop from the vector, sorting has to be reverse.
                match self.asset.cmp(&other.asset) {
                    Ordering::Equal => Ordering::Equal,
                    Ordering::Less => Ordering::Greater,
                    Ordering::Greater => Ordering::Less,
                }
            }
            Ordering::Less => Ordering::Less,
            Ordering::Greater => Ordering::Greater,
        }
    }
}

impl<A> AssetInDestruction<A> {
    pub(crate) fn new(asset: A) -> Self {
        AssetInDestruction { asset, state: DestructionState::Accounts }
    }

    pub(crate) fn asset(&self) -> &A {
        &self.asset
    }

    pub(crate) fn state(&self) -> &DestructionState {
        &self.state
    }

    pub(crate) fn transit_indestructible(&mut self) {
        self.state = DestructionState::Indestructible;
    }

    // Returns the new state on change, None otherwise
    pub(crate) fn transit_state(&mut self) -> Option<&DestructionState> {
        let state_before = self.state;

        self.state = match self.state {
            DestructionState::Accounts => DestructionState::Approvals,
            DestructionState::Approvals => DestructionState::Finalization,
            DestructionState::Destroyed => DestructionState::Destroyed,
            DestructionState::Finalization => DestructionState::Destroyed,
            DestructionState::Indestructible => DestructionState::Indestructible,
        };

        if state_before != self.state { Some(&self.state) } else { None }
    }
}
