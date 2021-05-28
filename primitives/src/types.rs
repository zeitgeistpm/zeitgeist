pub use crate::asset::*;
pub use crate::market::*;
pub use crate::outcome_report::OutcomeReport;
pub use crate::pool::Pool;
pub use crate::pool_status::PoolStatus;
pub use crate::serde_wrapper::*;

#[cfg(feature = "std")]
use arbitrary::{Arbitrary, Result, Unstructured};

use frame_support::dispatch::{Decode, Encode};
use sp_runtime::{
    generic,
    traits::{IdentifyAccount, Verify},
    MultiSignature,
};

/// Signed counter-part of Balance
pub type Amount = i128;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
/// never know...
pub type AccountIndex = u64;

/// Balance of an account.
pub type Balance = u128;

/// An index to a block.
pub type BlockNumber = u64;

/// The index of the category for a `CategoricalOutcome` asset.
pub type CategoryIndex = u16;

/// Multihash for digest sizes up to 384 bit.
/// The multicodec encoding the hash algorithm uses only 1 byte,
/// effecitvely limiting the number of available hash types.
/// HashType (1B) + DigestSize (1B) + Hash (48B).
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub enum MultiHash {
    Sha3_384([u8; 50]),
}

// Implementation for the fuzzer
#[cfg(feature = "std")]
impl<'a> Arbitrary<'a> for MultiHash {
    fn arbitrary(_: &mut Unstructured<'a>) -> Result<Self> {
        Ok(MultiHash::Sha3_384([0u8; 50]))
    }

    fn arbitrary_take_rest(mut u: Unstructured<'a>) -> Result<Self> {
        Self::arbitrary(&mut u)
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        let _ = depth;
        (0, None)
    }
}

pub type CurrencyId = Asset<MarketId>;

/// Index of a transaction in the chain.
pub type Index = u64;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// The market identifier type.
pub type MarketId = u128;

/// Time
pub type Moment = u64;

/// The identifier type for pools.
pub type PoolId = u128;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

// Tests

pub type AccountIdTest = u128;

#[cfg(feature = "std")]
pub type BlockTest<R> = frame_system::mocking::MockBlock<R>;

#[cfg(feature = "std")]
pub type UncheckedExtrinsicTest<R> = frame_system::mocking::MockUncheckedExtrinsic<R>;
