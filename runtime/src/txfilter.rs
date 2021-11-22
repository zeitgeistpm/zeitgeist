// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Common runtime code for Polkadot and Kusama.

use core::marker::PhantomData;
use sp_runtime::{traits::{SignedExtension, DispatchInfoOf},
	transaction_validity::{TransactionValidityError, TransactionValidity, InvalidTransaction}
};
use super::{AccountId, Call};
use frame_support::{
    dispatch::{Decode, Encode, Input},
    traits::Contains
};
use sp_runtime::traits::Dispatchable;

/// Custom validity errors used in Polkadot while validating transactions.
#[repr(u8)]
pub enum ValidityError {
	/// The Ethereum signature is invalid.
	InvalidEthereumSignature = 0,
	/// The signer has no claim.
	SignerHasNoClaim = 1,
	/// No permission to execute the call.
	NoPermission = 2,
	/// An invalid statement was made for a claim.
	InvalidStatement = 3,
}

impl From<ValidityError> for u8 {
	fn from(err: ValidityError) -> Self {
		err as u8
	}
}


/// Apply a given filter to transactions.
pub struct TransactionCallFilter<T: Contains<Call>, Call>(PhantomData<(T, Call)>);

impl<F: Contains<Call>, Call> Default for TransactionCallFilter<F, Call> {
	fn default() -> Self { Self::new() }
}
impl<F: Contains<Call>, Call> Encode for TransactionCallFilter<F, Call> {
	fn using_encoded<R, FO: FnOnce(&[u8]) -> R>(&self, f: FO) -> R { f(&b""[..]) }
}
impl<F: Contains<Call>, Call> Decode for TransactionCallFilter<F, Call> {
	fn decode<I: Input>(_: &mut I) -> Result<Self, parity_scale_codec::Error> { Ok(Self::new()) }
}
impl<F: Contains<Call>, Call> Clone for TransactionCallFilter<F, Call> {
	fn clone(&self) -> Self { Self::new() }
}
impl<F: Contains<Call>, Call> Eq for TransactionCallFilter<F, Call> {}
impl<F: Contains<Call>, Call> PartialEq for TransactionCallFilter<F, Call> {
	fn eq(&self, _: &Self) -> bool { true }
}
impl<F: Contains<Call>, Call> core::fmt::Debug for TransactionCallFilter<F, Call> {
	fn fmt(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result { Ok(()) }
}

fn validate<F: Contains<Call>, Call>(call: &Call) -> TransactionValidity {
	if F::contains(call) {
		Ok(Default::default())
	} else {
		Err(InvalidTransaction::Custom(ValidityError::NoPermission.into()).into())
	}
}

impl<F: Contains<Call> + Send + Sync, Call: Dispatchable + Send + Sync>
	SignedExtension for TransactionCallFilter<F, Call>
{
	const IDENTIFIER: &'static str = "TransactionCallFilter";
	type AccountId = AccountId;
	type Call = Call;
	type AdditionalSigned = ();
	type Pre = ();

	fn additional_signed(&self) -> Result<(), TransactionValidityError> { Ok(()) }

	fn validate(&self,
		_: &Self::AccountId,
		call: &Call,
		_: &DispatchInfoOf<Self::Call>,
		_: usize,
	) -> TransactionValidity { validate::<F, _>(call) }

	fn validate_unsigned(
		call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity { validate::<F, _>(call) }
}

impl<F: Contains<Call>, Call> TransactionCallFilter<F, Call> {
	/// Create a new instance.
	pub fn new() -> Self {
		Self(PhantomData)
	}
}


pub struct IsCallable;
impl Contains<Call> for IsCallable {
	fn contains(call: &Call) -> bool {
        match call {
            // Allowed calls:
            Call::System(_) | Call::Sudo(_) => true,
            
            // Prohibited calls:
            Call::Timestamp(_) | Call::ParachainSystem(_) | Call::ParachainStaking(_) |
            Call::AuthorInherent(_) | Call::AuthorMapping(_) | Call::DmpQueue(_) |
            Call::PolkadotXcm(_) | Call::XcmpQueue(_) | Call::Crowdloan(_) |
            Call::Balances(_) | Call::Treasury(_) | Call::AdvisoryCommitteeCollective(_) |
            Call::AdvisoryCommitteeMembership(_) | Call::Identity(_) | Call::Utility(_) |
            Call::Currency(_) | Call::Authorized(_) | Call::Court(_) | Call::LiquidityMining(_) |
            Call::Swaps(_) | Call::PredictionMarkets(_) => false
        }
	}
}