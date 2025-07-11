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

#![allow(
    // Constants parameters inside `parameter_types!` already check
    // arithmetic operations at compile time
    clippy::arithmetic_side_effects
)]

use super::{parameters::MAXIMUM_BLOCK_WEIGHT, ParachainInfo, RuntimeBlockWeights, RuntimeOrigin};
use cumulus_primitives_core::AggregateMessageOrigin;
use frame_support::{parameter_types, weights::Weight};
use orml_traits::parameter_type_with_key;
use sp_runtime::{Perbill, Percent};
use xcm::latest::{
    prelude::{AllOf, AssetFilter, GlobalConsensus, InteriorLocation, Wild, WildFungible},
    Junction::Parachain,
    Location, NetworkId,
};
use zeitgeist_primitives::{
    constants::{BASE, BLOCKS_PER_MINUTE},
    types::Balance,
};

parameter_types! {
    // Asset registry
    pub const AssetRegistryStringLimit: u32 = 1024;

    // Author-Mapping
    /// The amount that should be taken as a security deposit when registering a NimbusId.
    pub const CollatorDeposit: Balance = 2 * BASE;

    // Cumulus and Polkadot
    pub const RelayNetwork: NetworkId = NetworkId::Rococo;
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
    pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
    pub UnitWeightCost: Weight = Weight::from_parts(200_000_000u64, 0);
    pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;

    // Staking
    /// Average time beetween 2 blocks in milliseconds
    pub const BlockTime: u64 = 12_000;
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
    /// Maximum number of collator candidates
    pub const MaxCandidates: u32 = 200;
    /// Maximum delegations per delegator
    pub const MaxDelegationsPerDelegator: u32 = 100;
    /// Maximum top delegations per candidate
    pub const MaxTopDelegationsPerCandidate: u32 = 300;
    /// Number of rounds until a collator not producing blocks is marked as inactive.
    pub const MaxOfflineRounds: u32 = 1;
    /// Minimum round length is 2 minutes
    pub const MinBlocksPerRound: u32 = 2 * BLOCKS_PER_MINUTE as u32;
    /// Minimum stake required to become a collator
    pub const MinCandidateStk: u128 = 64 * BASE;
    /// Minimum stake required to be reserved to be a delegator
    pub const MinDelegation: u128 = BASE / 2;
    /// Minimum collators selected per round, default at genesis and minimum forever after
    pub const MinSelectedCandidates: u32 = 8;
    /// Slot duration in milliseconds
    pub const SlotDuration: u64 = 12_000;
    /// Rounds before the delegator revocation can be executed
    pub const RevokeDelegationDelay: u32 = 2;
    /// Rounds before the reward is paid
    pub const RewardPaymentDelay: u32 = 2;

    // XCM
    /// Base weight for XCM execution
    pub const BaseXcmWeight: Weight = Weight::from_parts(200_000_000u64, 0);
    /// The maximum number of distinct assets allowed to be transferred in a
    /// single helper extrinsic
    pub const MaxAssetsForTransfer: usize = 2;
    /// Maximum amount of tokens the holding register can store
    pub const MaxAssetsIntoHolding: u32 = 64;
    /// Max instructions per XCM
    pub const MaxInstructions: u32 = 100;
    /// The maximum number of local XCM locks that a single account may have.
    pub const MaxLockers: u32 = 8;
    /// The maximum number of consumers a single remote lock may have.
    pub const MaxRemoteLockConsumers: u32 = 0;
    /// Maximal number of outbound XCMP channels that can have messages queued at the same time.
    pub const MaxActiveOutboundChannels: u32 = 128;
    /// The maximum number of inbound XCMP channels that can be suspended simultaneously.
    pub const MaxInboundSuspended: u32 = 1_000;
    /// Maximum number of relay block to skip before trigering the Paused mode.
    pub const PausedThreshold: u32 = 300;
    /// The amount of weight (if any) which should be provided to the message queue for
    /// servicing enqueued items.
    ///
    /// This may be legitimately `None` in the case that you will call
    /// `ServiceQueues::service_queues` manually.
    pub MessageQueueServiceWeight: Weight =
        Perbill::from_percent(25) * RuntimeBlockWeights::get().max_block;
    /// The maximum number of stale pages (i.e. of overweight messages) allowed before culling
    /// can happen. Once there are more stale pages than this, then historical pages may be
    /// dropped, even if they contain unprocessed overweight messages.
    pub const MessageQueueMaxStale: u32 = 8;
    /// The size of the page; this implies the maximum message size which can be sent.
    ///
    /// A good value depends on the expected message sizes, their weights, the weight that is
    /// available for processing them and the maximal needed message size. The maximal message
    /// size is slightly lower than this as defined by [`MaxMessageLenOf`].
    pub const MessageQueueHeapSize: u32 = 103 * 1024;

    /// Relative self location
    pub SelfLocation: Location = Location::new(1, [Parachain(ParachainInfo::parachain_id().into())]);
    /// This chain's Universal Location
    pub UniversalLocation: InteriorLocation = [GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into())].into();
    pub const RelayLocation: Location = Location::parent();
    pub RelayLocationFilter: AssetFilter = Wild(AllOf {
        fun: WildFungible,
        id: xcm::prelude::AssetId(RelayLocation::get()),
    });
    pub AssetHubLocation: Location = Location::new(1, [Parachain(1000)]);
    pub RelayChainNativeAssetFromAssetHub: (AssetFilter, Location) = (
        RelayLocationFilter::get(),
        AssetHubLocation::get()
    );
}

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
    // XCM
    /// A `Location` that can be reached via `XcmRouter`. Used only in benchmarks.
    pub ReachableDest: Option<Location> = Some(xcm::latest::prelude::Parent.into());
}

parameter_type_with_key! {
    // XCM
    pub ParachainMinFee: |_location: Location| -> Option<u128> {
        None
    };
}
