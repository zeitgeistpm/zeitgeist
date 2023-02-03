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

use crate::mock::*;
use crate::pallet::Author;
use frame_support::traits::{OnFinalize, OnInitialize};
use nimbus_primitives::{NimbusId, NIMBUS_ENGINE_ID};
use parity_scale_codec::Encode;
use sp_core::{ByteArray, H256};
use sp_runtime::{Digest, DigestItem};

#[test]
fn kick_off_authorship_validation_is_mandatory() {
	use frame_support::dispatch::{DispatchClass, GetDispatchInfo};

	let info = crate::Call::<Test>::kick_off_authorship_validation {}.get_dispatch_info();
	assert_eq!(info.class, DispatchClass::Mandatory);
}

#[test]
fn test_author_is_available_after_on_initialize() {
	new_test_ext().execute_with(|| {
		let block_number = 1;
		System::initialize(
			&block_number,
			&H256::default(),
			&Digest {
				logs: vec![DigestItem::PreRuntime(
					NIMBUS_ENGINE_ID,
					NimbusId::from_slice(&ALICE_NIMBUS).unwrap().encode(),
				)],
			},
		);

		AuthorInherent::on_initialize(block_number);
		assert_eq!(Some(ALICE), <Author<Test>>::get());
	});
}

#[test]
fn test_author_is_still_available_after_on_finalize() {
	new_test_ext().execute_with(|| {
		let block_number = 1;
		System::initialize(
			&block_number,
			&H256::default(),
			&Digest {
				logs: vec![DigestItem::PreRuntime(
					NIMBUS_ENGINE_ID,
					NimbusId::from_slice(&ALICE_NIMBUS).unwrap().encode(),
				)],
			},
		);

		AuthorInherent::on_initialize(block_number);
		AuthorInherent::on_finalize(block_number);
		assert_eq!(Some(ALICE), <Author<Test>>::get());
	});
}
