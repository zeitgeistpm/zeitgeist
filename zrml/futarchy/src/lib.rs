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

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod mock;
mod tests;
mod traits;
pub mod types;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{traits::OracleQuery, types::Proposal};
    use core::marker::PhantomData;
    use frame_support::{
        ensure,
        pallet_prelude::{EnsureOrigin, IsType, StorageMap, StorageVersion, ValueQuery},
        require_transactional,
        traits::{
            schedule::{v3::Anon as ScheduleAnon, DispatchTime},
            Bounded, Hooks, QueryPreimage, StorePreimage,
        },
        transactional, Blake2_128Concat, BoundedVec,
    };
    use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
    use orml_traits::MultiCurrency;
    use sp_runtime::{
        traits::{ConstU32, Get},
        DispatchResult, Saturating,
    };
    use frame_support::pallet_prelude::Weight;use frame_support::dispatch::RawOrigin;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type MultiCurrency: MultiCurrency<Self::AccountId>;

        type MinDuration: Get<BlockNumberFor<Self>>;

        type OracleQuery: OracleQuery;

        /// Preimage interface for acquiring call data.
        type Preimages: QueryPreimage + StorePreimage;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Scheduler interface for executing proposals.
        type Scheduler: ScheduleAnon<BlockNumberFor<Self>, CallOf<Self>, OriginFor<Self>>;

        /// The origin that is allowed to submit proposals.
        type SubmitOrigin: EnsureOrigin<Self::RuntimeOrigin>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
    pub(crate) type CacheSize = ConstU32<16>;
    pub(crate) type CallOf<T> = <T as frame_system::Config>::RuntimeCall;
    pub(crate) type BoundedCallOf<T> = Bounded<CallOf<T>>;
    pub(crate) type OracleQueryOf<T> = <T as Config>::OracleQuery;
    pub(crate) type ProposalOf<T> = Proposal<BlockNumberFor<T>, BoundedCallOf<T>, OracleQueryOf<T>>;

    pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::storage]
    pub type Proposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BoundedVec<ProposalOf<T>, CacheSize>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// A proposal has been submitted.
        Submitted { duration: BlockNumberFor<T>, proposal: ProposalOf<T> },

        /// A proposal has been rejected by the oracle.
        Rejected,

        /// A proposal has been scheduled for execution.
        Scheduled,

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

    // TODO: Index for proposal?
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[transactional]
        #[pallet::weight({0})]
        pub fn submit_proposal(
            origin: OriginFor<T>,
            duration: BlockNumberFor<T>,
            proposal: ProposalOf<T>,
        ) -> DispatchResult {
            T::SubmitOrigin::ensure_origin(origin)?;
            Self::do_submit_proposal(duration, proposal)
        }
    }

    impl<T: Config> Pallet<T> {
        #[require_transactional]
        fn do_submit_proposal(
            duration: BlockNumberFor<T>,
            proposal: ProposalOf<T>,
        ) -> DispatchResult {
            ensure!(duration >= T::MinDuration::get(), Error::<T>::DurationTooShort);

            let now = frame_system::Pallet::<T>::block_number();
            let to_be_scheduled_at = now.saturating_add(duration);

            Ok(Proposals::<T>::try_mutate(to_be_scheduled_at, |proposals| {
                proposals.try_push(proposal).map_err(|_| Error::<T>::CacheFull)
            })?)
        }

        /// Evaluates `proposal` using the specified oracle and schedules the contained call if the
        /// oracle approves.
        fn maybe_schedule_proposal(proposal: ProposalOf<T>) -> Weight {
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

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(now: BlockNumberFor<T>) -> Weight {
            let mut total_weight = Weight::zero();

            let proposals = Proposals::<T>::take(now);
            for proposal in proposals.into_iter() {
                let weight = Self::maybe_schedule_proposal(proposal);
                total_weight = total_weight.saturating_add(weight);
            }

            total_weight
        }
    }
}
