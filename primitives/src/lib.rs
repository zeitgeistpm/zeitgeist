#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod asset;
mod serde_wrapper;
mod swaps;
mod zeitgeist_currencies_extension;
mod zeitgeist_multi_reservable_currency;

pub use asset::*;
use frame_support::{parameter_types, PalletId};
pub use serde_wrapper::SerdeWrapper;
use sp_runtime::{
    generic,
    traits::{IdentifyAccount, Verify},
    MultiSignature,
};
pub use swaps::Swaps;
pub use zeitgeist_currencies_extension::ZeitgeistCurrenciesExtension;
pub use zeitgeist_multi_reservable_currency::ZeitgeistMultiReservableCurrency;

pub const BASE: u128 = 10_000_000_000;

// Swaps parameters
parameter_types! {
    pub const ExitFee: Balance = 0;
    pub const MaxAssets: usize = 9;
    pub const MaxInRatio: Balance = BASE / 2;
    pub const MaxOutRatio: Balance = (BASE / 3) + 1;
    pub const MaxTotalWeight: Balance = 50 * BASE;
    pub const MaxWeight: Balance = 50 * BASE;
    pub const MinLiquidity: Balance = 100 * BASE;
    pub const MinWeight: Balance = BASE;
    pub const SwapsPalletId: PalletId = PalletId(*b"zge/swap");
}

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

pub type CurrencyId = Asset<MarketId>;

/// Index of a transaction in the chain.
pub type Index = u64;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// The market identifier type.
pub type MarketId = u128;

/// TODO
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
