// Copyright 2019-2022 PureStake Inc.
// This file is part of Nimbus.

// Nimbus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Nimbus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Nimbus.  If not, see <http://www.gnu.org/licenses/>.

//! Small pallet responsible determining which accounts are eligible to author at the current
//! slot.
//!
//! Using a randomness beacon supplied by the `Randomness` trait, this pallet takes the set of
//! currently active accounts from an upstream source, and filters them down to a pseudorandom subset.
//! The current technique gives no preference to any particular author. In the future, we could
//! disfavor authors who are authoring a disproportionate amount of the time in an attempt to
//! "even the playing field".

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(any(test, feature = "runtime-benchmarks"))]
mod benchmarks;

pub mod migration;
pub mod num;
pub mod weights;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {

	use crate::num::NonZeroU32;
	use crate::weights::WeightInfo;
	use frame_support::{pallet_prelude::*, traits::Randomness};
	use frame_system::pallet_prelude::*;
	use log::debug;
	use nimbus_primitives::CanAuthor;
	use sp_core::H256;
	use sp_runtime::Percent;
	use sp_std::vec::Vec;

	/// The Author Filter pallet
	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(PhantomData<T>);

	/// Configuration trait of this pallet.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type
		type Event: From<Event> + IsType<<Self as frame_system::Config>::Event>;
		/// Deterministic on-chain pseudo-randomness used to do the filtering
		type RandomnessSource: Randomness<H256, Self::BlockNumber>;
		//TODO introduce a new trait for exhaustive sets and use it here.
		// Oh actually, we can use the same trait. First we call the inner one
		// to determine whether this particular author is eligible there. then we
		// use the author as part of the subject when querying eligibility. I like this better.
		/// A source for the complete set of potential authors.
		/// The starting point of the filtering.
		type PotentialAuthors: Get<Vec<Self::AccountId>>;
		type WeightInfo: WeightInfo;
	}

	/// Compute a pseudo-random subset of the input accounts by using Pallet's
	/// source of randomness, `Config::RandomnessSource`.
	/// Returns (Eligible, Ineligible), each is a set of accounts
	pub fn compute_pseudo_random_subset<T: Config>(
		mut active: Vec<T::AccountId>,
		seed: &u32,
	) -> (Vec<T::AccountId>, Vec<T::AccountId>) {
		let mut num_eligible = EligibleCount::<T>::get().get() as usize;
		if num_eligible > active.len() {
			num_eligible = active.len();
		}

		let mut eligible = Vec::with_capacity(num_eligible);

		for i in 0..num_eligible {
			// A context identifier for grabbing the randomness. Consists of three parts
			// 1. Constant string *b"filter" - to identify this pallet
			// 2. First 2 bytes of index.to_le_bytes when selecting the ith eligible author
			// 3. First 4 bytes of seed.to_be_bytes
			let mut first_two_bytes_of_index = &i.to_le_bytes()[..2];
			let mut first_four_bytes_of_seed = &seed.to_be_bytes()[..4];
			let mut constant_string: [u8; 6] = [b'f', b'i', b'l', b't', b'e', b'r'];
			let mut subject: [u8; 12] = [0u8; 12];
			subject[..6].copy_from_slice(&mut constant_string);
			subject[6..8].copy_from_slice(&mut first_two_bytes_of_index);
			subject[8..].copy_from_slice(&mut first_four_bytes_of_seed);
			let (randomness, _) = T::RandomnessSource::random(&subject);
			debug!(target: "author-filter", "🎲Randomness sample {}: {:?}", i, &randomness);

			// Cast to u32 first so we get consistent results on 32- and 64-bit platforms.
			let bytes: [u8; 4] = randomness.to_fixed_bytes()[0..4]
				.try_into()
				.expect("H256 has at least 4 bytes; qed");
			let randomness = u32::from_le_bytes(bytes) as usize;

			// Move the selected author from the original vector into the eligible vector
			// TODO we could short-circuit this check by returning early when the claimed
			// author is selected. For now I'll leave it like this because:
			// 1. it is easier to understand what our core filtering logic is
			// 2. we currently show the entire filtered set in the debug event
			eligible.push(active.remove(randomness % active.len()));
		}
		(eligible, active)
	}

	// This code will be called by the author-inherent pallet to check whether the reported author
	// of this block is eligible in this slot. We calculate that result on demand and do not
	// record it in storage (although we do emit a debugging event for now).
	impl<T: Config> CanAuthor<T::AccountId> for Pallet<T> {
		#[cfg(not(feature = "try-runtime"))]
		fn can_author(author: &T::AccountId, slot: &u32) -> bool {
			// Compute pseudo-random subset of potential authors
			let (eligible, ineligible) =
				compute_pseudo_random_subset::<T>(T::PotentialAuthors::get(), slot);

			// Print some logs for debugging purposes.
			debug!(target: "author-filter", "Eligible Authors: {:?}", eligible);
			debug!(target: "author-filter", "Ineligible Authors: {:?}", &ineligible);
			debug!(target: "author-filter",
				"Current author, {:?}, is eligible: {}",
				author,
				eligible.contains(author)
			);

			eligible.contains(author)
		}
		#[cfg(feature = "runtime-benchmarks")]
		fn get_authors(slot: &u32) -> Vec<T::AccountId> {
			// Compute pseudo-random subset of potential authors
			let (eligible, _) = compute_pseudo_random_subset::<T>(T::PotentialAuthors::get(), slot);
			eligible
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Update the eligible count. Intended to be called by governance.
		#[pallet::weight(T::WeightInfo::set_eligible())]
		pub fn set_eligible(
			origin: OriginFor<T>,
			new: EligibilityValue,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			EligibleCount::<T>::put(&new);
			<Pallet<T>>::deposit_event(Event::EligibleUpdated(new));

			Ok(Default::default())
		}
	}

	/// The type of eligibility to use
	pub type EligibilityValue = NonZeroU32;

	impl EligibilityValue {
		/// Default total number of eligible authors, must NOT be 0.
		pub fn default() -> Self {
			NonZeroU32::new_unchecked(50)
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn eligible_ratio)]
	#[deprecated(note = "use `pallet::EligibleCount` instead")]
	pub type EligibleRatio<T: Config> = StorageValue<_, Percent, ValueQuery, Half<T>>;

	// Default value for the `EligibleRatio` is one half.
	#[pallet::type_value]
	pub fn Half<T: Config>() -> Percent {
		Percent::from_percent(50)
	}

	/// The number of active authors that will be eligible at each height.
	#[pallet::storage]
	#[pallet::getter(fn eligible_count)]
	pub type EligibleCount<T: Config> =
		StorageValue<_, EligibilityValue, ValueQuery, DefaultEligibilityValue<T>>;

	// Default value for the `EligibleCount`.
	#[pallet::type_value]
	pub fn DefaultEligibilityValue<T: Config>() -> EligibilityValue {
		EligibilityValue::default()
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig {
		pub eligible_count: EligibilityValue,
	}

	#[cfg(feature = "std")]
	impl Default for GenesisConfig {
		fn default() -> Self {
			Self {
				eligible_count: EligibilityValue::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			EligibleCount::<T>::put(self.eligible_count.clone());
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event {
		/// The amount of eligible authors for the filter to select has been changed.
		EligibleUpdated(EligibilityValue),
	}
}
