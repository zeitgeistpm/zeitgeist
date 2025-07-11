// Copyright 2022-2025 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
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

use crate::traits::CombinatorialTokensBenchmarkHelper;
pub use crate::{
    asset::*, market::*, max_runtime_usize::*, outcome_report::OutcomeReport, proxy_type::*,
    serde_wrapper::*,
};
use alloc::vec::Vec;
use core::marker::PhantomData;
use frame_support::{dispatch::PostDispatchInfo, weights::Weight};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    DispatchResult, MultiSignature, OpaqueExtrinsic,
};

#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result, Unstructured};

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

/// The type used to identify combinatorial outcomes.
pub type CombinatorialId = [u8; 32];

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
pub type Nonce = u64;

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

pub mod well_known_relay_keys {
    use hex_literal::hex;

    pub const TIMESTAMP_NOW: &[u8] =
        &hex!["f0c365c3cf59d671eb72da0e7a4113c49f1f0515f462cdcf84e0f1d6045dfcbb"];
}

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

#[derive(
    Clone,
    Copy,
    Debug,
    Decode,
    Default,
    Encode,
    Eq,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    TypeInfo,
)]
/// Custom XC asset metadata
pub struct CustomMetadata {
    /// XCM-related metadata.
    pub xcm: XcmMetadata,

    /// Whether an asset can be used as base_asset in pools.
    pub allow_as_base_asset: bool,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Decode,
    Default,
    Encode,
    Eq,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    TypeInfo,
)]
pub struct XcmMetadata {
    /// The factor used to determine the fee of the foreign asset.
    ///
    /// In the fee calculations, the factor is multiplied by the fee that would have been paid in
    /// native currency, so it represents the ratio between the price of the native currency and the
    /// foreign asset, or, equivalently, the value of the trading pair (native currency)/(foreign
    /// asset). The factor is a fixed point decimal number with ten decimals.
    ///
    /// Should be updated regularly.
    pub fee_factor: Option<Balance>,
}

pub struct NoopCombinatorialTokensBenchmarkHelper<Balance, MarketId>(
    PhantomData<(Balance, MarketId)>,
);

impl<Balance, MarketId> CombinatorialTokensBenchmarkHelper
    for NoopCombinatorialTokensBenchmarkHelper<Balance, MarketId>
{
    type Balance = Balance;
    type MarketId = MarketId;

    fn setup_payout_vector(
        _market_id: Self::MarketId,
        _payout: Option<Vec<Self::Balance>>,
    ) -> DispatchResult {
        Ok(())
    }
}

pub struct SplitPositionDispatchInfo<CombinatorialId, MarketId> {
    pub collection_ids: Vec<CombinatorialId>,
    pub position_ids: Vec<Asset<MarketId>>,
    pub post_dispatch_info: PostDispatchInfo,
}
