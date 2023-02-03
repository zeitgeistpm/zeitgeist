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

use cumulus_primitives_parachain_inherent::{
	ParachainInherentData, INHERENT_IDENTIFIER as PARACHAIN_INHERENT_IDENTIFIER,
};
use nimbus_primitives::{
	CompatibleDigestItem, DigestsProvider, NimbusApi, NimbusId, NIMBUS_ENGINE_ID,
};
use sc_consensus::BlockImportParams;
use sc_consensus_manual_seal::{ConsensusDataProvider, Error};
use sp_api::{BlockT, HeaderT, ProvideRuntimeApi, TransactionFor};
use sp_application_crypto::ByteArray;
use sp_inherents::InherentData;
use sp_keystore::SyncCryptoStorePtr;
use sp_runtime::{Digest, DigestItem};
use std::{marker::PhantomData, sync::Arc};

/// Provides nimbus-compatible pre-runtime digests for use with manual seal consensus
pub struct NimbusManualSealConsensusDataProvider<C, DP = (), P = ()> {
	/// Shared reference to keystore
	pub keystore: SyncCryptoStorePtr,

	/// Shared reference to the client
	pub client: Arc<C>,
	// Could have a skip_prediction field here if it becomes desireable
	/// Additional digests provider
	pub additional_digests_provider: DP,

	pub _phantom: PhantomData<P>,
}

impl<B, C, DP, P> ConsensusDataProvider<B> for NimbusManualSealConsensusDataProvider<C, DP, P>
where
	B: BlockT,
	C: ProvideRuntimeApi<B> + Send + Sync,
	C::Api: NimbusApi<B>,
	DP: DigestsProvider<NimbusId, <B as BlockT>::Hash> + Send + Sync,
	P: Send + Sync,
{
	type Transaction = TransactionFor<C, B>;
	type Proof = P;

	fn create_digest(&self, parent: &B::Header, inherents: &InherentData) -> Result<Digest, Error> {
		// Retrieve the relay chain block number to use as the slot number from the parachain inherent
		let slot_number = inherents
			.get_data::<ParachainInherentData>(&PARACHAIN_INHERENT_IDENTIFIER)
			.expect("Parachain inherent should decode correctly")
			.expect("Parachain inherent should be present because we are mocking it")
			.validation_data
			.relay_parent_number;

		// Fetch first eligible key from keystore
		let maybe_key = crate::first_eligible_key::<B, C>(
			self.client.clone(),
			&*self.keystore,
			parent,
			// For now we author all blocks in slot zero, which is consistent with  how we are
			// mocking the relay chain height which the runtime uses for slot beacon.
			// This should improve. See https://github.com/PureStake/nimbus/issues/3
			slot_number,
		);

		// If we aren't eligible, return an appropriate error
		match maybe_key {
			Some(key) => {
				let nimbus_id = NimbusId::from_slice(&key.1).map_err(|_| {
					Error::StringError(String::from("invalid nimbus id (wrong length)"))
				})?;
				let mut logs = vec![CompatibleDigestItem::nimbus_pre_digest(nimbus_id.clone())];
				logs.extend(
					self.additional_digests_provider
						.provide_digests(nimbus_id, parent.hash()),
				);
				Ok(Digest { logs })
			}
			None => Err(Error::StringError(String::from(
				"no nimbus keys available to manual seal",
			))),
		}
	}

	// This is where we actually sign with the nimbus key and attach the seal
	fn append_block_import(
		&self,
		_parent: &B::Header,
		params: &mut BlockImportParams<B, Self::Transaction>,
		_inherents: &InherentData,
		_proof: Self::Proof,
	) -> Result<(), Error> {
		// We have to reconstruct the type-public pair which is only communicated through the pre-runtime digest
		let claimed_author = params
			.header
			.digest()
			.logs
			.iter()
			.find_map(|digest| {
				match *digest {
					// We do not support the older author inherent in manual seal
					DigestItem::PreRuntime(id, ref author_id) if id == NIMBUS_ENGINE_ID => {
						Some(author_id.clone())
					}
					_ => None,
				}
			})
			.expect("Expected one pre-runtime digest that contains author id bytes");

		let nimbus_public = NimbusId::from_slice(&claimed_author)
			.map_err(|_| Error::StringError(String::from("invalid nimbus id (wrong length)")))?;

		let sig_digest =
			crate::seal_header::<B>(&params.header, &*self.keystore, &nimbus_public.into());

		params.post_digests.push(sig_digest);

		Ok(())
	}
}
