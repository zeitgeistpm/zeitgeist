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

#![allow(
    // Constants parameters inside `parameter_types!` already check
    // arithmetic operations at compile time
    clippy::integer_arithmetic
)]
#![cfg(feature = "parachain")]

use super::{parameters::MAXIMUM_BLOCK_WEIGHT, Origin, ParachainInfo};
use frame_support::{parameter_types, weights::Weight};
use orml_traits::parameter_type_with_key;
use sp_runtime::{Perbill, Percent};
use xcm::latest::{prelude::X1, Junction::Parachain, MultiLocation, NetworkId};
use zeitgeist_primitives::{
    constants::{BASE, BLOCKS_PER_MINUTE},
    types::Balance,
};

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
    pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
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
}

parameter_type_with_key! {
    // XCM
    pub ParachainMinFee: |_location: MultiLocation| -> Option<u128> {
        None
    };
}
