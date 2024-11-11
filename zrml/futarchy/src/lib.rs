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
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//     Copyright (C) Parity Technologies (UK) Ltd.
//     SPDX-License-Identifier: Apache-2.0
//
//     Licensed under the Apache License, Version 2.0 (the "License");
//     you may not use this file except in compliance with the License.
//     You may obtain a copy of the License at
//
//     	http://www.apache.org/licenses/LICENSE-2.0
//
//     Unless required by applicable law or agreed to in writing, software
//     distributed under the License is distributed on an "AS IS" BASIS,
//     WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//     See the License for the specific language governing permissions and
//     limitations under the License.

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod benchmarking;
mod dispatchable_impls;
pub mod mock;
mod pallet_impls;
mod tests;
pub mod traits;
pub mod types;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{types::Proposal, weights::WeightInfoZeitgeist};
    use alloc::fmt::Debug;
    use core::marker::PhantomData;
    use frame_support::{
        pallet_prelude::{EnsureOrigin, IsType, StorageMap, StorageVersion, ValueQuery, Weight},
        traits::{schedule::v3::Anon as ScheduleAnon, Bounded, Hooks, OriginTrait},
        transactional, Blake2_128Concat, BoundedVec,
    };
    use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
    use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;
    use sp_runtime::{
        traits::{ConstU32, Get},
        DispatchResult,
    };
    use zeitgeist_primitives::traits::FutarchyOracle;

    #[cfg(feature = "runtime-benchmarks")]
    use zeitgeist_primitives::traits::FutarchyBenchmarkHelper;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: FutarchyBenchmarkHelper<Self::Oracle>;

        /// The minimum allowed duration between the creation of a proposal and its evaluation.
        type MinDuration: Get<BlockNumberFor<Self>>;

        /// The type used to define the oracle a proposal.
        type Oracle: FutarchyOracle
            + Clone
            + Debug
            + Decode
            + Encode
            + Eq
            + MaxEncodedLen
            + PartialEq
            + TypeInfo;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Scheduler interface for executing proposals.
        type Scheduler: ScheduleAnon<BlockNumberFor<Self>, CallOf<Self>, PalletsOriginOf<Self>>;

        /// The origin that is allowed to submit proposals.
        type SubmitOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    pub(crate) type CacheSize = ConstU32<16>;
    pub(crate) type CallOf<T> = <T as frame_system::Config>::RuntimeCall;
    pub(crate) type BoundedCallOf<T> = Bounded<CallOf<T>>;
    pub(crate) type OracleOf<T> = <T as Config>::Oracle;
    pub(crate) type PalletsOriginOf<T> =
        <<T as frame_system::Config>::RuntimeOrigin as OriginTrait>::PalletsOrigin;

    pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::storage]
    pub type Proposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BoundedVec<Proposal<T>, CacheSize>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(crate) fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A proposal has been submitted.
        Submitted { duration: BlockNumberFor<T>, proposal: Proposal<T> },

        /// A proposal has been rejected by the oracle.
        Rejected { proposal: Proposal<T> },

        /// A proposal has been scheduled for execution.
        Scheduled { proposal: Proposal<T> },

        /// This is a logic error. You shouldn't see this.
        UnexpectedSchedulerError,
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The cache for this particular block is full. Try another block.
        CacheFull,

        /// The specified duration must be at least equal to `MinDuration`.
        DurationTooShort,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submits a `proposal` for evaluation in `duration` blocks.
        ///
        /// If, after `duration` blocks, the oracle `proposal.oracle` is evaluated positively, the
        /// proposal is scheduled for execution at `proposal.when`.
        #[pallet::call_index(0)]
        #[transactional]
        #[pallet::weight(T::WeightInfo::submit_proposal())]
        pub fn submit_proposal(
            origin: OriginFor<T>,
            duration: BlockNumberFor<T>,
            proposal: Proposal<T>,
        ) -> DispatchResult {
            T::SubmitOrigin::ensure_origin(origin)?;

            Self::do_submit_proposal(duration, proposal)
        }
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let mut total_weight = Weight::zero(); // Add buffer.

            let proposals = Proposals::<T>::take(now);
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));

            for proposal in proposals.into_iter() {
                let weight = Self::maybe_schedule_proposal(proposal);
                total_weight = total_weight.saturating_add(weight);
            }

            total_weight
        }
    }
}
