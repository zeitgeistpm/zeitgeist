#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

extern crate alloc;

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod common;
#[cfg(feature = "testnet")]
pub mod battery_station;
#[cfg(not(feature = "testnet"))]
pub mod zeitgeist;

// Expose runtime
#[cfg(feature = "testnet")]
pub use battery_station::{api, parameters::SS58Prefix, Call, Runtime, RuntimeApi, VERSION};
#[cfg(all(feature = "std", feature = "testnet"))]
pub use battery_station::{GenesisConfig, AdvisoryCommitteeMembershipConfig, BalancesConfig, CouncilMembershipConfig, LiquidityMiningConfig, SudoConfig, SystemConfig, TechnicalCommitteeMembershipConfig};
#[cfg(all(feature = "std", feature = "testnet", not(feature = "parachain")))]
pub use battery_station::{AuraConfig, GrandpaConfig};

#[cfg(not(feature = "testnet"))]
pub use zeitgeist::{api, parameters::SS58Prefix, Runtime, Call, RuntimeApi, VERSION};
#[cfg(all(feature = "std", not(feature = "testnet")))]
pub use zeitgeist::{GenesisConfig, AdvisoryCommitteeMembershipConfig, BalancesConfig, CouncilMembershipConfig, LiquidityMiningConfig, SystemConfig, TechnicalCommitteeMembershipConfig};
#[cfg(all(feature = "std", not(feature = "testnet"), not(feature = "parachain")))]
pub use zeitgeist::{AuraConfig, GrandpaConfig};

// Expose functions and types required to construct node CLI
pub use common::{opaque::Block, SignedPayload, UncheckedExtrinsic, SystemCall, SignedExtra, CheckNonZeroSender, CheckSpecVersion, CheckTxVersion, CheckGenesis, CheckEra, CheckNonce, CheckWeight, ChargeTransactionPayment};
#[cfg(feature = "std")]
pub use common::native_version;