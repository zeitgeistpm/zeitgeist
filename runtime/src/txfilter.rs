#![cfg(feature = "txfilter")]

use super::{AccountId, Call};
use core::marker::PhantomData;
use frame_support::{
    dispatch::{Decode, Encode, Input},
    traits::Contains,
};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{DispatchInfoOf, Dispatchable, SignedExtension},
    transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError},
};

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
#[derive(TypeInfo)]
pub struct TransactionCallFilter<F: Contains<Call>, Call>(PhantomData<(F, Call)>);

impl<F: Contains<Call>, Call> Default for TransactionCallFilter<F, Call> {
    fn default() -> Self {
        Self::new()
    }
}
impl<F: Contains<Call>, Call> Encode for TransactionCallFilter<F, Call> {
    fn using_encoded<R, FO: FnOnce(&[u8]) -> R>(&self, f: FO) -> R {
        f(&b""[..])
    }
}
impl<F: Contains<Call>, Call> Decode for TransactionCallFilter<F, Call> {
    fn decode<I: Input>(_: &mut I) -> Result<Self, parity_scale_codec::Error> {
        Ok(Self::new())
    }
}
impl<F: Contains<Call>, Call> Clone for TransactionCallFilter<F, Call> {
    fn clone(&self) -> Self {
        Self::new()
    }
}
impl<F: Contains<Call>, Call> Eq for TransactionCallFilter<F, Call> {}
impl<F: Contains<Call>, Call> PartialEq for TransactionCallFilter<F, Call> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl<F: Contains<Call>, Call> core::fmt::Debug for TransactionCallFilter<F, Call> {
    fn fmt(&self, _: &mut core::fmt::Formatter) -> core::fmt::Result {
        Ok(())
    }
}

fn validate<F: Contains<Call>, Call>(call: &Call) -> TransactionValidity {
    if F::contains(call) {
        Ok(Default::default())
    } else {
        Err(InvalidTransaction::Custom(ValidityError::NoPermission.into()).into())
    }
}

impl<F, Call> SignedExtension for TransactionCallFilter<F, Call>
where
    F: 'static + Contains<Call> + Send + Sync + TypeInfo,
    Call: 'static + Dispatchable + Send + Sync + TypeInfo,
{
    const IDENTIFIER: &'static str = "TransactionCallFilter";
    type AccountId = AccountId;
    type Call = Call;
    type AdditionalSigned = ();
    type Pre = ();

    fn additional_signed(&self) -> Result<(), TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        _: &Self::AccountId,
        call: &Call,
        _: &DispatchInfoOf<Self::Call>,
        _: usize,
    ) -> TransactionValidity {
        validate::<F, _>(call)
    }

    fn validate_unsigned(
        call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        validate::<F, _>(call)
    }
}

impl<F: Contains<Call>, Call> TransactionCallFilter<F, Call> {
    /// Create a new instance.
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

#[derive(TypeInfo)]
pub struct IsCallable;
#[cfg(feature = "parachain")]
impl Contains<Call> for IsCallable {
    fn contains(call: &Call) -> bool {
        match call {
            // Allowed calls:
            Call::System(_)
            | Call::Sudo(_)
            | Call::Timestamp(_)
            | Call::AuthorInherent(_)
            | Call::AuthorMapping(_)
            | Call::DmpQueue(_)
            | Call::ParachainSystem(_)
            | Call::PolkadotXcm(_)
            | Call::XcmpQueue(_) => true,

            // Prohibited calls:
            Call::ParachainStaking(_)
            | Call::Crowdloan(_)
            | Call::Balances(_)
            | Call::Treasury(_)
            | Call::AdvisoryCommitteeCollective(_)
            | Call::AdvisoryCommitteeMembership(_)
            | Call::Identity(_)
            | Call::Utility(_)
            | Call::Currency(_)
            | Call::Authorized(_)
            | Call::Court(_)
            | Call::LiquidityMining(_)
            | Call::Swaps(_)
            | Call::PredictionMarkets(_)
            | Call::Vesting(_) => false,
        }
    }
}

#[cfg(not(feature = "parachain"))]
impl Contains<Call> for IsCallable {
    fn contains(call: &Call) -> bool {
        match call {
            // Allowed calls:
            Call::System(_) | Call::Grandpa(_) | Call::Sudo(_) | Call::Timestamp(_) => true,

            // Prohibited calls:
            Call::Balances(_)
            | Call::Treasury(_)
            | Call::AdvisoryCommitteeCollective(_)
            | Call::AdvisoryCommitteeMembership(_)
            | Call::Identity(_)
            | Call::Utility(_)
            | Call::Currency(_)
            | Call::Authorized(_)
            | Call::Court(_)
            | Call::LiquidityMining(_)
            | Call::Swaps(_)
            | Call::PredictionMarkets(_)
            | Call::Vesting(_) => false,
        }
    }
}
