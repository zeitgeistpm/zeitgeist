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

//! Nimbus Consensus Primitives
//!
//! Primitive types and traits for working with the Nimbus consensus framework.
//! This code can be built to no_std for use in the runtime

#![cfg_attr(not(feature = "std"), no_std)]

use sp_application_crypto::KeyTypeId;
use sp_runtime::generic::DigestItem;
use sp_runtime::traits::BlockNumberProvider;
use sp_runtime::ConsensusEngineId;
#[cfg(feature = "runtime-benchmarks")]
use sp_std::vec;
use sp_std::vec::Vec;

pub mod digests;
mod inherents;

pub use digests::CompatibleDigestItem;

pub use inherents::{InherentDataProvider, INHERENT_IDENTIFIER};

pub trait DigestsProvider<Id, BlockHash> {
	type Digests: IntoIterator<Item = DigestItem>;
	fn provide_digests(&self, id: Id, parent: BlockHash) -> Self::Digests;
}

impl<Id, BlockHash> DigestsProvider<Id, BlockHash> for () {
	type Digests = [DigestItem; 0];
	fn provide_digests(&self, _id: Id, _parent: BlockHash) -> Self::Digests {
		[]
	}
}

impl<F, Id, BlockHash, D> DigestsProvider<Id, BlockHash> for F
where
	F: Fn(Id, BlockHash) -> D,
	D: IntoIterator<Item = DigestItem>,
{
	type Digests = D;

	fn provide_digests(&self, id: Id, parent: BlockHash) -> Self::Digests {
		(*self)(id, parent)
	}
}

/// The given account ID is the author of the current block.
pub trait EventHandler<Author> {
	//TODO should we be tking ownership here?
	fn note_author(author: Author);
}

impl<T> EventHandler<T> for () {
	fn note_author(_author: T) {}
}

/// A mechanism for determining the current slot.
/// For now we use u32 as the slot type everywhere. Let's see how long we can get away with that.
pub trait SlotBeacon {
	fn slot() -> u32;
	#[cfg(feature = "runtime-benchmarks")]
	fn set_slot(_slot: u32) {}
}

/// Anything that can provide a block height can be used as a slot beacon. This could be
/// used in at least two realistic ways.
/// 1. Use your own chain's height as the slot number
/// 2. If you're a parachain, use the relay chain's height as the slot number.
impl<T: BlockNumberProvider<BlockNumber = u32>> SlotBeacon for T {
	fn slot() -> u32 {
		Self::current_block_number()
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn set_slot(slot: u32) {
		Self::set_block_number(slot);
	}
}

/// PLANNED: A SlotBeacon that starts a new slot based on the timestamp. Behaviorally, this is
/// similar to what aura, babe and company do. Implementation-wise it is different because it
/// depends on the timestamp pallet for its notion of time.
pub struct IntervalBeacon;

impl SlotBeacon for IntervalBeacon {
	fn slot() -> u32 {
		todo!()
	}
}

/// Trait to determine whether this author is eligible to author in this slot.
/// This is the primary trait your nimbus filter needs to implement.
///
/// This is the proposition-logic variant.
/// That is to say the caller specifies an author an author and the implementation
/// replies whether that author is eligible. This is useful in many cases and is
/// particularly useful when the active set is unbounded.
/// There may be another variant where the caller only supplies a slot and the
/// implementation replies with a complete set of eligible authors.
pub trait CanAuthor<AuthorId> {
	#[cfg(feature = "try-runtime")]
	// With `try-runtime` the local author should always be able to author a block.
	fn can_author(author: &AuthorId, slot: &u32) -> bool {
		true
	}
	#[cfg(not(feature = "try-runtime"))]
	fn can_author(author: &AuthorId, slot: &u32) -> bool;
	#[cfg(feature = "runtime-benchmarks")]
	fn get_authors(_slot: &u32) -> Vec<AuthorId> {
		vec![]
	}
	#[cfg(feature = "runtime-benchmarks")]
	fn set_eligible_author(_slot: &u32) {}
}
/// Default implementation where anyone can author.
///
/// This is identical to Cumulus's RelayChainConsensus
impl<T> CanAuthor<T> for () {
	fn can_author(_: &T, _: &u32) -> bool {
		true
	}
}

/// A Trait to lookup runtime AccountIds from AuthorIds (probably NimbusIds)
/// The trait is generic over the AccountId, becuase different runtimes use
/// different notions of AccoutId. It is also generic over the AuthorId to
/// support the usecase where the author inherent is used for beneficiary info
/// and contains an AccountId directly.
pub trait AccountLookup<AccountId> {
	fn lookup_account(author: &NimbusId) -> Option<AccountId>;
}

// A dummy impl used in simple tests
impl<AccountId> AccountLookup<AccountId> for () {
	fn lookup_account(_: &NimbusId) -> Option<AccountId> {
		None
	}
}

/// The ConsensusEngineId for nimbus consensus
/// this same identifier will be used regardless of the filters installed
pub const NIMBUS_ENGINE_ID: ConsensusEngineId = *b"nmbs";

/// The KeyTypeId used in the Nimbus consensus framework regardles of wat filters are in place.
/// If this gets well adopted, we could move this definition to sp_core to avoid conflicts.
pub const NIMBUS_KEY_ID: KeyTypeId = KeyTypeId(*b"nmbs");

// The strongly-typed crypto wrappers to be used by Nimbus in the keystore
mod nimbus_crypto {
	use sp_application_crypto::{app_crypto, sr25519};
	app_crypto!(sr25519, crate::NIMBUS_KEY_ID);
}

/// A nimbus author identifier (A public key).
pub type NimbusId = nimbus_crypto::Public;

/// A nimbus signature.
pub type NimbusSignature = nimbus_crypto::Signature;

sp_application_crypto::with_pair! {
	/// A nimbus keypair
	pub type NimbusPair = nimbus_crypto::Pair;
}

sp_api::decl_runtime_apis! {
	/// The runtime api used to predict whether a Nimbus author will be eligible in the given slot
	pub trait NimbusApi {
		fn can_author(author: NimbusId, relay_parent: u32, parent_header: &Block::Header) -> bool;
	}
}
