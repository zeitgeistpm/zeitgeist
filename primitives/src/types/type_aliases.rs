// Copyright 2022-2024 Forecasting Technologies LTD.
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

pub use crate::{
    asset::*, market::*, max_runtime_usize::*, outcome_report::OutcomeReport, proxy_type::*,
    serde_wrapper::*,
};
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

/// ORML adapter
pub type BasicCurrencyAdapter<R, B> = orml_currencies::BasicCurrencyAdapter<R, B, Amount, Balance>;

pub type CurrencyId = Asset<MarketId>;

/// The asset id specifically used for pallet_assets_tx_payment for
/// paying transaction fees in different assets.
/// Since the polkadot extension and wallets can't handle custom asset ids other than just u32,
/// we are using a u32 as on the asset-hubs here.
pub type TxPaymentAssetId = u32;

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

// Tests

pub type AccountIdTest = u128;

#[cfg(feature = "std")]
pub type BlockTest<R> = frame_system::mocking::MockBlock<R>;

#[cfg(feature = "std")]
pub type UncheckedExtrinsicTest<R> = frame_system::mocking::MockUncheckedExtrinsic<R>;
