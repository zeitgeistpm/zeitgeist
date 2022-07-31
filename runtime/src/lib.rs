#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

extern crate alloc;

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[cfg(feature = "testnet")]
pub mod battery_station;
mod common;
#[cfg(not(feature = "testnet"))]
pub mod zeitgeist;

// Expose runtime
#[cfg(feature = "testnet")]
pub use battery_station::{api, parameters::SS58Prefix, Call, Runtime, RuntimeApi, VERSION};
#[cfg(all(feature = "std", feature = "testnet"))]
pub use battery_station::{
    AdvisoryCommitteeMembershipConfig, BalancesConfig, CouncilMembershipConfig, GenesisConfig,
    LiquidityMiningConfig, SudoConfig, SystemConfig, TechnicalCommitteeMembershipConfig,
};
#[cfg(all(feature = "std", feature = "testnet", not(feature = "parachain")))]
pub use battery_station::{AuraConfig, GrandpaConfig};

#[cfg(not(feature = "testnet"))]
pub use zeitgeist::{api, parameters::SS58Prefix, Call, Runtime, RuntimeApi, VERSION};
#[cfg(all(feature = "std", not(feature = "testnet")))]
pub use zeitgeist::{
    AdvisoryCommitteeMembershipConfig, BalancesConfig, CouncilMembershipConfig, GenesisConfig,
    LiquidityMiningConfig, SystemConfig, TechnicalCommitteeMembershipConfig,
};
#[cfg(all(feature = "std", not(feature = "testnet"), not(feature = "parachain")))]
pub use zeitgeist::{AuraConfig, GrandpaConfig};

// Expose functions and types required to construct node CLI
#[cfg(feature = "std")]
pub use common::native_version;
pub use common::{
    opaque::Block, ChargeTransactionPayment, CheckEra, CheckGenesis, CheckNonZeroSender,
    CheckNonce, CheckSpecVersion, CheckTxVersion, CheckWeight, SignedExtra, SignedPayload,
    SystemCall, UncheckedExtrinsic,
};
