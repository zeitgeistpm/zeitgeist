// Copyright 2022-2023 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
// Copyright 2022 Sygma.
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

#![allow(
    // Constants parameters inside `parameter_types!` already check
    // arithmetic operations at compile time
    clippy::arithmetic_side_effects
)]
#![cfg(feature = "parachain")]

extern crate alloc;

use super::{parameters::MAXIMUM_BLOCK_WEIGHT, ParachainInfo, RuntimeOrigin};
use alloc::vec::Vec;
use frame_support::{parameter_types, weights::Weight, PalletId};
use orml_traits::parameter_type_with_key;
use sp_runtime::{traits::AccountIdConversion, Perbill, Percent};
use sygma_traits::{ChainID, VerifyingContractAddress};
use xcm::latest::{prelude::X1, Junction::Parachain, MultiLocation, NetworkId};
use zeitgeist_primitives::{
    constants::{BASE, BLOCKS_PER_MINUTE},
    types::{AccountId, Balance},
};

// This address is defined in the substrate E2E test of sygma-relayer
// This address is provided by Sygma bridge
pub(crate) const DEST_VERIFYING_CONTRACT_ADDRESS: &str = "6CdE2Cd82a4F8B74693Ff5e194c19CA08c2d1c68";

parameter_types! {
    // Author-Mapping
    /// The amount that should be taken as a security deposit when registering a NimbusId.
    pub const CollatorDeposit: Balance = 2 * BASE;

    // Cumulus and Polkadot
    pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
    pub const RelayLocation: MultiLocation = MultiLocation::parent();
    pub const RelayNetwork: NetworkId = NetworkId::Any;
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
    pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
    pub UnitWeightCost: u64 = 200_000_000;

    // Staking
    /// Rounds before the candidate bond increase/decrease can be executed
    pub const CandidateBondLessDelay: u32 = 2;
    /// Default fixed percent a collator takes off the top of due rewards
    pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
    /// Blocks per round
    pub const DefaultBlocksPerRound: u32 = 2 * BLOCKS_PER_MINUTE as u32;
    /// Default percent of inflation set aside for parachain bond every round
    pub const DefaultParachainBondReservePercent: Percent = Percent::from_percent(30);
    /// Rounds before the delegator bond increase/decrease can be executed
    pub const DelegationBondLessDelay: u32 = 2;
    /// Rounds before the collator leaving the candidates request can be executed
    pub const LeaveCandidatesDelay: u32 = 2;
    /// Rounds before the delegator exit can be executed
    pub const LeaveDelegatorsDelay: u32 = 2;
    /// Maximum bottom delegations per candidate
    pub const MaxBottomDelegationsPerCandidate: u32 = 50;
    /// Maximum delegations per delegator
    pub const MaxDelegationsPerDelegator: u32 = 100;
    /// Maximum top delegations per candidate
    pub const MaxTopDelegationsPerCandidate: u32 = 300;
    /// Minimum round length is 2 minutes
    pub const MinBlocksPerRound: u32 = 2 * BLOCKS_PER_MINUTE as u32;
    /// Minimum stake required to become a collator
    pub const MinCollatorStk: u128 = 64 * BASE;
    /// Minimum stake required to be reserved to be a delegator
    pub const MinDelegatorStk: u128 = BASE / 2;
    /// Minimum collators selected per round, default at genesis and minimum forever after
    pub const MinSelectedCandidates: u32 = 8;
    /// Rounds before the delegator revocation can be executed
    pub const RevokeDelegationDelay: u32 = 2;
    /// Rounds before the reward is paid
    pub const RewardPaymentDelay: u32 = 2;

    // XCM
    /// Base weight for XCM execution
    pub const BaseXcmWeight: u64 = 200_000_000;
    /// The maximum number of distinct assets allowed to be transferred in a
    /// single helper extrinsic.
    pub const MaxAssetsForTransfer: usize = 2;
    /// Max instructions per XCM
    pub const MaxInstructions: u32 = 100;
    // Relative self location
    pub SelfLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into())));

    // Sygma bridge
    pub NativeAssetLocation: MultiLocation = MultiLocation::here();
    // TODO this is the resource id of Phala currently
    pub NativeAssetSygmaResourceId: [u8; 32] = hex_literal::hex!("0000000000000000000000000000000000000000000000000000000000000001");
    // Make sure put same value with `construct_runtime`
    pub const SygmaAccessSegregatorPalletIndex: u8 = 140;
    pub const SygmaBasicFeeHandlerPalletIndex: u8 = 141;
    pub const SygmaBridgePalletIndex: u8 = 142;
    pub const SygmaFeeHandlerRouterPalletIndex: u8 = 143;
    // RegisteredExtrinsics here registers all valid (pallet index, extrinsic_name) paris
    // make sure to update this when adding new access control extrinsic
    pub RegisteredExtrinsics: Vec<(u8, Vec<u8>)> = [
        (SygmaAccessSegregatorPalletIndex::get(), b"grant_access".to_vec()),
        (SygmaBasicFeeHandlerPalletIndex::get(), b"set_fee".to_vec()),
        (SygmaBridgePalletIndex::get(), b"set_mpc_address".to_vec()),
        (SygmaBridgePalletIndex::get(), b"pause_bridge".to_vec()),
        (SygmaBridgePalletIndex::get(), b"unpause_bridge".to_vec()),
        (SygmaBridgePalletIndex::get(), b"register_domain".to_vec()),
        (SygmaBridgePalletIndex::get(), b"unregister_domain".to_vec()),
        (SygmaBridgePalletIndex::get(), b"retry".to_vec()),
        (SygmaFeeHandlerRouterPalletIndex::get(), b"set_fee_handler".to_vec()),
    ].to_vec();
    pub const SygmaBridgePalletId: PalletId = PalletId(*b"sygma/01");
    // SygmaBridgeAccount is an account for holding transferred asset collection
    // SygmaBridgeAccount address: 5EYCAe5jLbHcAAMKvLFSXgCTbPrLgBJusvPwfKcaKzuf5X5e
    pub SygmaBridgeAccount: AccountId = SygmaBridgePalletId::get().into_account_truncating();
    // TODO: How should the sygma bridge fee account public key look like? Is it an account controlled by Zeitgeist?
    // SygmaBridgeFeeAccountKey Address: 44NmbpHjqbz9FcXfVzFUbMFJh5q7qsKAcSTJvFAdYPqQ62Qv
    pub SygmaBridgeFeeAccountKey: [u8; 32] = hex_literal::hex!("a63f9ccf857e1ab9e806366e3c46ae650de853503d772a987197ab7e22c8f88c");
    pub SygmaBridgeFeeAccount: AccountId = SygmaBridgeFeeAccountKey::get().into();
    // TODO: How should the sygma bridge admin account public key look like? Is it an account controlled by Zeitgeist?
    // SygmaBridgeAdminAccountKey Address: 44NmbpHjqbz9FcXfVzFUbMFJh5q7qsKAcSTJvFAdYPqQ62Qv
    pub SygmaBridgeAdminAccountKey: [u8; 32] = hex_literal::hex!("a63f9ccf857e1ab9e806366e3c46ae650de853503d772a987197ab7e22c8f88c");
    pub SygmaBridgeAdminAccount: AccountId = SygmaBridgeAdminAccountKey::get().into();
    // TODO: How should the EIP712Chain id look like? Is it controlled by Zeitgeist?
    // EIP712ChainID is the chainID that pallet is assigned with, used in EIP712 typed data domain
    pub EIP712ChainID: ChainID = sp_core::U256::from(5233);
    // DestVerifyingContractAddress is a H160 address that is used in proposal signature verification, specifically EIP712 typed data
    // When relayers signing, this address will be included in the EIP712Domain
    // As long as the relayer and pallet configured with the same address, EIP712Domain should be recognized properly.
    pub DestVerifyingContractAddress: VerifyingContractAddress = sp_core::H160::from_slice(hex::decode(DEST_VERIFYING_CONTRACT_ADDRESS).ok().unwrap().as_slice());
}

parameter_type_with_key! {
    // XCM
    pub ParachainMinFee: |_location: MultiLocation| -> Option<u128> {
        None
    };
}
