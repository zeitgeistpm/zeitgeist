pub use crate::{
    asset::*, market::*, max_runtime_usize::*, outcome_report::OutcomeReport, pool::*,
    pool_status::PoolStatus, proxy_type::*, serde_wrapper::*,
};
#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result, Unstructured};
use frame_support::dispatch::{Decode, Encode, Weight};
use scale_info::TypeInfo;
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    MultiSignature, OpaqueExtrinsic,
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

/// Block type.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;

/// An index to a block.
pub type BlockNumber = u64;

/// The index of the category for a `CategoricalOutcome` asset.
pub type CategoryIndex = u16;

/// Multihash for digest sizes up to 384 bit.
/// The multicodec encoding the hash algorithm uses only 1 byte,
/// effecitvely limiting the number of available hash types.
/// HashType (1B) + DigestSize (1B) + Hash (48B).
#[derive(TypeInfo, Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub enum MultiHash {
    Sha3_384([u8; 50]),
}

// Implementation for the fuzzer
#[cfg(feature = "arbitrary")]
impl<'a> Arbitrary<'a> for MultiHash {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let mut rand_bytes = <[u8; 50] as Arbitrary<'a>>::arbitrary(u)?;
        rand_bytes[0] = 0x15;
        rand_bytes[1] = 0x30;
        Ok(MultiHash::Sha3_384(rand_bytes))
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (50, Some(50))
    }
}

/// ORML adapter
pub type BasicCurrencyAdapter<R, B> = orml_currencies::BasicCurrencyAdapter<R, B, Amount, Balance>;

pub type CurrencyId = Asset<MarketId>;

/// Index of a transaction in the chain.
pub type Index = u64;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Digest item type.
pub type DigestItem = generic::DigestItem;

/// The market identifier type.
pub type MarketId = u128;

/// Time
pub type Moment = u64;

/// The identifier type for pools.
pub type PoolId = u128;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// The outcome index which is chosen of all pushed outcomes for the global dispute system.
pub type OutcomeIndex = u32;

// Tests

pub type AccountIdTest = u128;

#[cfg(feature = "std")]
pub type BlockTest<R> = frame_system::mocking::MockBlock<R>;

#[cfg(feature = "std")]
pub type UncheckedExtrinsicTest<R> = frame_system::mocking::MockUncheckedExtrinsic<R>;

#[derive(sp_runtime::RuntimeDebug, Clone, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub struct ResultWithWeightInfo<R> {
    pub result: R,
    pub weight: Weight,
}
