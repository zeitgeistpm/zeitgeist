// Copyright 2022-2025 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//     Copyright (C) 2020-2022 Acala Foundation.
//     SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
//
//     This program is free software: you can redistribute it and/or modify
//     it under the terms of the GNU General Public License as published by
//     the Free Software Foundation, either version 3 of the License, or
//     (at your option) any later version.
//
//     This program is distributed in the hope that it will be useful,
//     but WITHOUT ANY WARRANTY; without even the implied warranty of
//     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//     GNU General Public License for more details.
//
//     You should have received a copy of the GNU General Public License
//     along with this program. If not, see <https://www.gnu.org/licenses/>.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//     Copyright (C) Parity Technologies (UK) Ltd.
//     SPDX-License-Identifier: Apache-2.0
//
//     Licensed under the Apache License, Version 2.0 (the "License");
//     you may not use this file except in compliance with the License.
//     You may obtain a copy of the License at
//
//     	http://www.apache.org/licenses/LICENSE-2.0
//
//     Unless required by applicable law or agreed to in writing, software
//     distributed under the License is distributed on an "AS IS" BASIS,
//     WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//     See the License for the specific language governing permissions and
//     limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "512"]
#![allow(clippy::crate_in_macro_def)]

pub mod fees;
pub mod weights;

#[macro_export]
macro_rules! decl_common_types {
    () => {
        use core::marker::PhantomData;
        use frame_support::{
            migration::storage_key_iter,
            migrations::RemovePallet,
            pallet_prelude::StorageVersion,
            parameter_types,
            storage::child,
            traits::{
                fungible::HoldConsideration,
                fungibles::Imbalance as FImbalance,
                tokens::{PayFromAccount, UnityAssetBalanceConversion},
                Currency, Get, Imbalance, LinearStoragePrice, NeverEnsureOrigin, OnRuntimeUpgrade,
                OnUnbalanced, TransformOrigin,
            },
            Blake2_256, BoundedVec, Twox64Concat,
        };
        use frame_system::EnsureSigned;
        use orml_traits::MultiCurrency;
        use pallet_balances::{CreditOf, NegativeImbalance};
        use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
        use scale_info::TypeInfo;
        use sp_consensus_slots::Slot;
        use sp_core::storage::ChildInfo;
        use sp_runtime::{
            generic, traits::IdentityLookup, DispatchError, DispatchResult, RuntimeDebug,
            SaturatedConversion,
        };
        use zeitgeist_primitives::traits::{DeployPoolApi, DistributeFees, MarketCommonsPalletApi};
        use zrml_combinatorial_tokens::types::{CryptographicIdManager, Fuel};
        use zrml_neo_swaps::types::DecisionMarketOracle;

        #[cfg(feature = "try-runtime")]
        use frame_try_runtime::{TryStateSelect, UpgradeCheckSelect};

        #[cfg(feature = "runtime-benchmarks")]
        use zrml_neo_swaps::types::DecisionMarketBenchmarkHelper;

        #[cfg(feature = "runtime-benchmarks")]
        use zrml_prediction_markets::types::PredictionMarketsCombinatorialTokensBenchmarkHelper;

        pub type Block = generic::Block<Header, UncheckedExtrinsic>;

        type Address = sp_runtime::MultiAddress<AccountId, ()>;

        #[cfg(feature = "parachain")]
        type Migrations = (
            pallet_parachain_staking::migrations::MigrateRoundWithFirstSlot<Runtime>,
            pallet_parachain_staking::migrations::MigrateParachainBondConfig<Runtime>,
            cumulus_pallet_xcmp_queue::migration::v4::MigrationToV4<Runtime>,
            cumulus_pallet_xcmp_queue::migration::v5::MigrateV4ToV5<Runtime>,
            // This `MigrateToLatestXcmVersion` migration can be permanently added to the runtime migrations. https://github.com/paritytech/polkadot-sdk/blob/87971b3e92721bdf10bf40b410eaae779d494ca0/polkadot/xcm/pallet-xcm/src/migration.rs#L83
            pallet_xcm::migration::MigrateToLatestXcmVersion<Runtime>,
            // u64::MAX from here: https://github.com/paritytech/polkadot-sdk/blob/304bbb8711f61503ae7afa3a5bfd4f78af5cbd62/polkadot/runtime/rococo/src/lib.rs#L1629-L1630
            pallet_identity::migration::versioned::V0ToV1<Runtime, { u64::MAX }>,
        );
        #[cfg(not(feature = "parachain"))]
        type Migrations = (
            pallet_grandpa::migrations::MigrateV4ToV5<Runtime>,
            // u64::MAX from here: https://github.com/paritytech/polkadot-sdk/blob/304bbb8711f61503ae7afa3a5bfd4f78af5cbd62/polkadot/runtime/rococo/src/lib.rs#L1629-L1630
            pallet_identity::migration::versioned::V0ToV1<Runtime, { u64::MAX }>,
        );

        pub type Executive = frame_executive::Executive<
            Runtime,
            Block,
            frame_system::ChainContext<Runtime>,
            Runtime,
            AllPalletsWithSystem,
            Migrations,
        >;

        pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
        pub(crate) type NodeBlock = generic::Block<Header, sp_runtime::OpaqueExtrinsic>;
        pub type SignedExtra = (
            CheckNonZeroSender<Runtime>,
            CheckSpecVersion<Runtime>,
            CheckTxVersion<Runtime>,
            CheckGenesis<Runtime>,
            CheckEra<Runtime>,
            CheckNonce<Runtime>,
            CheckWeight<Runtime>,
            // https://docs.rs/pallet-asset-tx-payment/latest/src/pallet_asset_tx_payment/lib.rs.html#32-34
            pallet_asset_tx_payment::ChargeAssetTxPayment<Runtime>,
            frame_metadata_hash_extension::CheckMetadataHash<Runtime>,
        );
        pub type EventRecord = frame_system::EventRecord<
            <Runtime as frame_system::Config>::RuntimeEvent,
            <Runtime as frame_system::Config>::Hash,
        >;
        pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
        pub type UncheckedExtrinsic =
            generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

        // Governance
        type AdvisoryCommitteeInstance = pallet_collective::Instance1;
        type AdvisoryCommitteeMembershipInstance = pallet_membership::Instance1;
        type CouncilInstance = pallet_collective::Instance2;
        type CouncilMembershipInstance = pallet_membership::Instance2;
        type TechnicalCommitteeInstance = pallet_collective::Instance3;
        type TechnicalCommitteeMembershipInstance = pallet_membership::Instance3;

        // Council vote proportions
        // At least 50%
        type EnsureRootOrHalfCouncil = EitherOfDiverse<
            EnsureRoot<AccountId>,
            EnsureProportionAtLeast<AccountId, CouncilInstance, 1, 2>,
        >;

        // At least 60%
        type EnsureRootOrThreeFifthsCouncil = EitherOfDiverse<
            EnsureRoot<AccountId>,
            EnsureProportionAtLeast<AccountId, CouncilInstance, 3, 5>,
        >;

        // At least 66%
        type EnsureRootOrTwoThirdsCouncil = EitherOfDiverse<
            EnsureRoot<AccountId>,
            EnsureProportionAtLeast<AccountId, CouncilInstance, 2, 3>,
        >;

        // At least 75%
        type EnsureRootOrThreeFourthsCouncil = EitherOfDiverse<
            EnsureRoot<AccountId>,
            EnsureProportionAtLeast<AccountId, CouncilInstance, 3, 4>,
        >;

        // At least 100%
        type EnsureRootOrAllCouncil = EitherOfDiverse<
            EnsureRoot<AccountId>,
            EnsureProportionAtLeast<AccountId, CouncilInstance, 1, 1>,
        >;

        // Technical committee vote proportions
        // At least 50%
        #[cfg(feature = "parachain")]
        type EnsureRootOrHalfTechnicalCommittee = EitherOfDiverse<
            EnsureRoot<AccountId>,
            EnsureProportionAtLeast<AccountId, TechnicalCommitteeInstance, 1, 2>,
        >;

        // At least 60%
        type EnsureRootOrThreeFifthsTechnicalCommittee = EitherOfDiverse<
            EnsureRoot<AccountId>,
            EnsureProportionAtLeast<AccountId, TechnicalCommitteeInstance, 3, 5>,
        >;

        // At least 66%
        type EnsureRootOrTwoThirdsTechnicalCommittee = EitherOfDiverse<
            EnsureRoot<AccountId>,
            EnsureProportionAtLeast<AccountId, TechnicalCommitteeInstance, 2, 3>,
        >;

        // At least 100%
        type EnsureRootOrAllTechnicalCommittee = EitherOfDiverse<
            EnsureRoot<AccountId>,
            EnsureProportionAtLeast<AccountId, TechnicalCommitteeInstance, 1, 1>,
        >;

        // Advisory Committee vote proportions
        // More than 33%
        type EnsureRootOrMoreThanOneThirdAdvisoryCommittee = EitherOfDiverse<
            EnsureRoot<AccountId>,
            EnsureProportionMoreThan<AccountId, AdvisoryCommitteeInstance, 1, 3>,
        >;

        // More than 50%
        type EnsureRootOrMoreThanHalfAdvisoryCommittee = EitherOfDiverse<
            EnsureRoot<AccountId>,
            EnsureProportionMoreThan<AccountId, AdvisoryCommitteeInstance, 1, 2>,
        >;

        // More than 66%
        type EnsureRootOrMoreThanTwoThirdsAdvisoryCommittee = EitherOfDiverse<
            EnsureRoot<AccountId>,
            EnsureProportionMoreThan<AccountId, AdvisoryCommitteeInstance, 2, 3>,
        >;

        // At least 66%
        type EnsureRootOrTwoThirdsAdvisoryCommittee = EitherOfDiverse<
            EnsureRoot<AccountId>,
            EnsureProportionAtLeast<AccountId, AdvisoryCommitteeInstance, 2, 3>,
        >;

        #[cfg(feature = "std")]
        /// The version information used to identify this runtime when compiled natively.
        pub fn native_version() -> NativeVersion {
            NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
        }

        // Accounts protected from being deleted due to a too low amount of funds.
        pub struct DustRemovalWhitelist;

        impl Contains<AccountId> for DustRemovalWhitelist
        where
            frame_support::PalletId: AccountIdConversion<AccountId>,
        {
            fn contains(ai: &AccountId) -> bool {
                let mut pallets = vec![
                    AuthorizedPalletId::get(),
                    CourtPalletId::get(),
                    GlobalDisputesPalletId::get(),
                    HybridRouterPalletId::get(),
                    OrderbookPalletId::get(),
                    ParimutuelPalletId::get(),
                    PmPalletId::get(),
                    SwapsPalletId::get(),
                    TreasuryPalletId::get(),
                ];

                if let Some(pallet_id) = frame_support::PalletId::try_from_sub_account::<u128>(ai) {
                    return pallets.contains(&pallet_id.0);
                }

                for pallet_id in pallets {
                    let pallet_acc: AccountId = pallet_id.into_account_truncating();

                    if pallet_acc == *ai {
                        return true;
                    }
                }

                false
            }
        }

        common_runtime::impl_fee_types!();

        pub mod opaque {
            //! Opaque types. These are used by the CLI to instantiate machinery that don't need to
            //! know the specifics of the runtime. They can then be made to be agnostic over
            //! specific formats of data like extrinsics, allowing for them to continue syncing the
            //! network through upgrades to even the core data structures.

            use super::Header;
            use alloc::vec::Vec;
            use sp_runtime::{generic, impl_opaque_keys};

            pub type Block = generic::Block<Header, sp_runtime::OpaqueExtrinsic>;

            #[cfg(feature = "parachain")]
            impl_opaque_keys! {
                pub struct SessionKeys {
                    pub nimbus: crate::AuthorInherent,
                    pub vrf: session_keys_primitives::VrfSessionKey,
                }
            }

            #[cfg(not(feature = "parachain"))]
            impl_opaque_keys! {
                pub struct SessionKeys {
                    pub aura: crate::Aura,
                    pub grandpa: crate::Grandpa,
                }
            }
        }
    };
}

// Construct runtime
#[macro_export]
macro_rules! create_runtime {
    ($($additional_pallets:tt)*) => {
        use alloc::{boxed::Box, vec::Vec};
        // Pallets are enumerated based on the dependency graph.
        //
        // For example, `PredictionMarkets` is pĺaced after `MarketCommons` because
        // `PredictionMarkets` depends on `MarketCommons`.

        construct_runtime!(
            pub enum Runtime {
                // System
                System: frame_system::{Call, Config<T>, Event<T>, Pallet, Storage} = 0,
                Timestamp: pallet_timestamp::{Call, Pallet, Storage, Inherent} = 1,
                RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip::{Pallet, Storage} = 2,
                Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 3,
                Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>, HoldReason} = 4,

                // Money
                Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage} = 10,
                TransactionPayment: pallet_transaction_payment::{Config<T>, Event<T>, Pallet, Storage} = 11,
                Treasury: pallet_treasury::{Call, Config<T>, Event<T>, Pallet, Storage} = 12,
                Vesting: pallet_vesting::{Call, Config<T>, Event<T>, Pallet, Storage} = 13,
                Multisig: pallet_multisig::{Call, Event<T>, Pallet, Storage} = 14,
                Bounties: pallet_bounties::{Call, Event<T>, Pallet, Storage} =  15,
                AssetTxPayment: pallet_asset_tx_payment::{Event<T>, Pallet} = 16,

                // Governance
                Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>} = 20,
                AdvisoryCommittee: pallet_collective::<Instance1>::{Call, Config<T>, Event<T>, Origin<T>, Pallet, Storage} = 21,
                AdvisoryCommitteeMembership: pallet_membership::<Instance1>::{Call, Config<T>, Event<T>, Pallet, Storage} = 22,
                Council: pallet_collective::<Instance2>::{Call, Config<T>, Event<T>, Origin<T>, Pallet, Storage} = 23,
                CouncilMembership: pallet_membership::<Instance2>::{Call, Config<T>, Event<T>, Pallet, Storage} = 24,
                TechnicalCommittee: pallet_collective::<Instance3>::{Call, Config<T>, Event<T>, Origin<T>, Pallet, Storage} = 25,
                TechnicalCommitteeMembership: pallet_membership::<Instance3>::{Call, Config<T>, Event<T>, Pallet, Storage} = 26,

                // Other Parity pallets
                Identity: pallet_identity::{Call, Event<T>, Pallet, Storage} = 30,
                Utility: pallet_utility::{Call, Event, Pallet, Storage} = 31,
                Proxy: pallet_proxy::{Call, Event<T>, Pallet, Storage} = 32,

                // Third-party
                AssetManager: orml_currencies::{Call, Pallet, Storage} = 40,
                Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage} = 41,

                // Zeitgeist
                MarketCommons: zrml_market_commons::{Pallet, Storage} = 50,
                Authorized: zrml_authorized::{Call, Event<T>, Pallet, Storage} = 51,
                Court: zrml_court::{Call, Event<T>, Pallet, Storage} = 52,
                Swaps: zrml_swaps::{Call, Event<T>, Pallet, Storage} = 56,
                PredictionMarkets: zrml_prediction_markets::{Call, Event<T>, Pallet, Storage} = 57,
                Styx: zrml_styx::{Call, Event<T>, Pallet, Storage} = 58,
                GlobalDisputes: zrml_global_disputes::{Call, Event<T>, Pallet, Storage} = 59,
                NeoSwaps: zrml_neo_swaps::{Call, Event<T>, Pallet, Storage} = 60,
                Orderbook: zrml_orderbook::{Call, Event<T>, Pallet, Storage} = 61,
                Parimutuel: zrml_parimutuel::{Call, Event<T>, Pallet, Storage} = 62,
                HybridRouter: zrml_hybrid_router::{Call, Event<T>, Pallet, Storage} = 64,
                CombinatorialTokens: zrml_combinatorial_tokens::{Call, Event<T>, Pallet, Storage} = 65,
                Futarchy: zrml_futarchy::{Call, Event<T>, Pallet, Storage} = 66,

                $($additional_pallets)*
            }
        );
    }
}

#[macro_export]
macro_rules! create_runtime_with_additional_pallets {
    ($($additional_pallets:tt)*) => {
        #[cfg(feature = "parachain")]
        create_runtime!(
            // System
            ParachainSystem: cumulus_pallet_parachain_system::{Call, Config<T>, Event<T>, Inherent, Pallet, Storage} = 100,
            ParachainInfo: parachain_info::{Config<T>, Pallet, Storage} = 101,

            // Consensus
            ParachainStaking: pallet_parachain_staking::{Call, Config<T>, Event<T>, Pallet, Storage} = 110,
            AuthorInherent: pallet_author_inherent::{Call, Inherent, Pallet, Storage} = 111,
            AuthorFilter: pallet_author_slot_filter::{Call, Config<T>, Event, Pallet, Storage} = 112,
            AuthorMapping: pallet_author_mapping::{Call, Config<T>, Event<T>, Pallet, Storage} = 113,

            // XCM
            CumulusXcm: cumulus_pallet_xcm::{Event<T>, Origin, Pallet} = 120,
            // TODO Remove this pallet once the lazy migration is complete. https://github.com/paritytech/polkadot-sdk/blob/87971b3e92721bdf10bf40b410eaae779d494ca0/cumulus/pallets/dmp-queue/src/lib.rs#L45
            DmpQueue: cumulus_pallet_dmp_queue::{Call, Event<T>, Pallet, Storage} = 121,
            PolkadotXcm: pallet_xcm::{Call, Config<T>, Event<T>, Origin, Pallet, Storage} = 122,
            XcmpQueue: cumulus_pallet_xcmp_queue::{Call, Event<T>, Pallet, Storage} = 123,
            AssetRegistry: orml_asset_registry::module::{Call, Config<T>, Event<T>, Pallet, Storage} = 124,
            UnknownTokens: orml_unknown_tokens::{Pallet, Storage, Event} = 125,
            XTokens: orml_xtokens::{Pallet, Storage, Call, Event<T>} = 126,
            MessageQueue: pallet_message_queue::{Pallet, Call, Storage, Event<T>} = 127,

            // Others
            $($additional_pallets)*
        );

        #[cfg(not(feature = "parachain"))]
        create_runtime!(
            // Consensus
            Aura: pallet_aura::{Config<T>, Pallet, Storage} = 100,
            Grandpa: pallet_grandpa::{Call, Config<T>, Event, Pallet, Storage} = 101,

            // Others
            $($additional_pallets)*
        );
    }
}

#[macro_export]
macro_rules! impl_config_traits {
    () => {
        use common_runtime::weights;
        #[cfg(feature = "parachain")]
        use {
            cumulus_primitives_core::{AggregateMessageOrigin, ParaId},
            frame_support::traits::Nothing,
            parachains_common::message_queue::{NarrowOriginToSibling, ParaIdToSibling},
            xcm_config::config::*,
        };

        // TODO: Remove this pallet once the lazy migration is complete. https://github.com/paritytech/polkadot-sdk/blob/87971b3e92721bdf10bf40b410eaae779d494ca0/cumulus/pallets/dmp-queue/src/lib.rs#L45
        #[cfg(feature = "parachain")]
        impl cumulus_pallet_dmp_queue::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            type DmpSink = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
            type WeightInfo = weights::cumulus_pallet_dmp_queue::WeightInfo<Runtime>;
        }

        // Configure Pallets
        #[cfg(feature = "parachain")]
        impl cumulus_pallet_parachain_system::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            type OnSystemEvent = ();
            type OutboundXcmpMessageSource = XcmpQueue;
            type ReservedDmpWeight = crate::parachain_params::ReservedDmpWeight;
            type ReservedXcmpWeight = crate::parachain_params::ReservedXcmpWeight;
            type SelfParaId = parachain_info::Pallet<Runtime>;
            type XcmpMessageHandler = XcmpQueue;
            type CheckAssociatedRelayNumber =
                cumulus_pallet_parachain_system::RelayNumberStrictlyIncreases;
            type ConsensusHook = cumulus_pallet_parachain_system::ExpectParentIncluded;
            type DmpQueue = frame_support::traits::EnqueueWithOrigin<MessageQueue, RelayOrigin>;
            type WeightInfo = weights::cumulus_pallet_parachain_system::WeightInfo<Runtime>;
        }

        #[cfg(feature = "parachain")]
        impl cumulus_pallet_xcm::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
        }

        #[cfg(feature = "parachain")]
        impl cumulus_pallet_xcmp_queue::Config for Runtime {
            type ChannelInfo = ParachainSystem;
            type ControllerOrigin = EnsureRootOrThreeFifthsTechnicalCommittee;
            type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
            type PriceForSiblingDelivery =
                polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery<ParaId>;
            type MaxActiveOutboundChannels = MaxActiveOutboundChannels;
            // Most on-chain HRMP channels are configured to use 102400 bytes of max message size, so we
            // need to set the page size larger than that until we reduce the channel size on-chain.
            type MaxPageSize = MessageQueueHeapSize;
            type RuntimeEvent = RuntimeEvent;
            type VersionWrapper = PolkadotXcm;
            type XcmpQueue =
                TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
            type MaxInboundSuspended = MaxInboundSuspended;
            type WeightInfo = weights::cumulus_pallet_xcmp_queue::WeightInfo<Runtime>;
        }

        #[cfg(feature = "parachain")]
        impl cumulus_pallet_xcmp_queue::migration::v5::V5Config for Runtime {
            // This must be the same as the `ChannelInfo` from the `Config`:
            type ChannelList = ParachainSystem;
        }

        #[cfg(feature = "parachain")]
        impl pallet_message_queue::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            #[cfg(feature = "use-noop-message-processor")]
            type MessageProcessor =
                pallet_message_queue::mock_helpers::NoopMessageProcessor<AggregateMessageOrigin>;
            #[cfg(not(feature = "use-noop-message-processor"))]
            type MessageProcessor = xcm_builder::ProcessXcmMessage<
                AggregateMessageOrigin,
                xcm_executor::XcmExecutor<XcmConfig>,
                RuntimeCall,
            >;
            type Size = u32;
            type HeapSize = MessageQueueHeapSize;
            type MaxStale = MessageQueueMaxStale;
            type ServiceWeight = MessageQueueServiceWeight;
            // The XCMP queue pallet is only ever able to handle the `Sibling(ParaId)` origin:
            type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
            type QueuePausedQuery = NarrowOriginToSibling<XcmpQueue>;
            type WeightInfo = weights::pallet_message_queue::WeightInfo<Runtime>;
            type IdleMaxServiceWeight = MessageQueueServiceWeight;
        }

        impl frame_system::Config for Runtime {
            type AccountData = pallet_balances::AccountData<Balance>;
            type AccountId = AccountId;
            type BaseCallFilter = IsCallable;
            type Block = Block;
            type BlockHashCount = BlockHashCount;
            type BlockLength = RuntimeBlockLength;
            type BlockWeights = RuntimeBlockWeights;
            type RuntimeCall = RuntimeCall;
            type DbWeight = RocksDbWeight;
            type RuntimeEvent = RuntimeEvent;
            type Hash = Hash;
            type Hashing = BlakeTwo256;
            type Lookup = AccountIdLookup<AccountId, ()>;
            type Nonce = Nonce;
            type MaxConsumers = ConstU32<16>;
            type OnKilledAccount = ();
            type OnNewAccount = ();
            #[cfg(feature = "parachain")]
            type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
            #[cfg(not(feature = "parachain"))]
            type OnSetCode = ();
            type RuntimeOrigin = RuntimeOrigin;
            type RuntimeTask = RuntimeTask;
            type PalletInfo = PalletInfo;
            type SS58Prefix = SS58Prefix;
            type SystemWeightInfo = weights::frame_system::WeightInfo<Runtime>;
            type Version = Version;
            type SingleBlockMigrations = ();
            type MultiBlockMigrator = ();
            type PreInherents = ();
            type PostInherents = ();
            type PostTransactions = ();
        }

        #[cfg(not(feature = "parachain"))]
        impl pallet_aura::Config for Runtime {
            type AllowMultipleBlocksPerSlot = AllowMultipleBlocksPerSlot;
            type AuthorityId = sp_consensus_aura::sr25519::AuthorityId;
            type DisabledValidators = ();
            type MaxAuthorities = MaxAuthorities;
            type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
        }

        #[cfg(feature = "parachain")]
        impl pallet_author_inherent::Config for Runtime {
            type AccountLookup = AuthorMapping;
            type AuthorId = AccountId;
            type CanAuthor = AuthorFilter;
            type SlotBeacon = cumulus_pallet_parachain_system::RelaychainDataProvider<Self>;
            type WeightInfo = weights::pallet_author_inherent::WeightInfo<Runtime>;
        }

        #[cfg(feature = "parachain")]
        impl pallet_author_mapping::Config for Runtime {
            type DepositAmount = CollatorDeposit;
            type DepositCurrency = Balances;
            type RuntimeEvent = RuntimeEvent;
            type Keys = session_keys_primitives::VrfId;
            type WeightInfo = weights::pallet_author_mapping::WeightInfo<Runtime>;
        }

        #[cfg(feature = "parachain")]
        impl pallet_author_slot_filter::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            type RandomnessSource = RandomnessCollectiveFlip;
            type PotentialAuthors = ParachainStaking;
            type WeightInfo = weights::pallet_author_slot_filter::WeightInfo<Runtime>;
        }

        frame_support::parameter_types! {
            pub const MaxSetIdSessionEntries: u32 = 12;
        }

        #[cfg(not(feature = "parachain"))]
        impl pallet_grandpa::Config for Runtime {
            type EquivocationReportSystem = ();
            type KeyOwnerProof = sp_core::Void;
            type MaxAuthorities = MaxAuthorities;
            type MaxNominators = MaxNominators;
            type MaxSetIdSessionEntries = MaxSetIdSessionEntries;
            type RuntimeEvent = RuntimeEvent;
            // Currently the benchmark does yield an invalid weight implementation
            // type WeightInfo = weights::pallet_grandpa::WeightInfo<Runtime>;
            type WeightInfo = ();
        }

        #[cfg(feature = "parachain")]
        impl pallet_xcm::Config for Runtime {
            type AdminOrigin = EnsureRoot<AccountId>;
            type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
            type RuntimeCall = RuntimeCall;
            type RuntimeEvent = RuntimeEvent;
            type RuntimeOrigin = RuntimeOrigin;
            type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
            type UniversalLocation = UniversalLocation;
            type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
            type XcmExecuteFilter = Nothing;
            // ^ Disable dispatchable execute on the XCM pallet.
            // Needs to be `Everything` for local testing.
            type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
            type XcmTeleportFilter = Everything;
            type XcmReserveTransferFilter = Nothing;
            type XcmRouter = XcmRouter;

            type Currency = Balances;
            type CurrencyMatcher = ();
            type TrustedLockers = ();
            type SovereignAccountOf = LocationToAccountId;
            type MaxLockers = MaxLockers;
            type MaxRemoteLockConsumers = MaxRemoteLockConsumers;
            // TODO(#1425) use correct weight info after benchmarking
            type WeightInfo = pallet_xcm::TestWeightInfo;
            type RemoteLockConsumerIdentifier = ();

            const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
            // ^ Override for AdvertisedXcmVersion default
            type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
        }

        #[cfg(feature = "parachain")]
        pub struct OnInactiveCollator;
        #[cfg(feature = "parachain")]
        impl pallet_parachain_staking::OnInactiveCollator<Runtime> for OnInactiveCollator {
            fn on_inactive_collator(
                collator_id: AccountId,
                round: pallet_parachain_staking::RoundIndex,
            ) -> Result<
                Weight,
                sp_runtime::DispatchErrorWithPostInfo<frame_support::dispatch::PostDispatchInfo>,
            > {
                use pallet_parachain_staking::WeightInfo;

                ParachainStaking::go_offline_inner(collator_id)?;
                let extra_weight =
                    <Runtime as pallet_parachain_staking::Config>::WeightInfo::go_offline(
                        pallet_parachain_staking::MAX_CANDIDATES,
                    );

                Ok(<Runtime as frame_system::Config>::DbWeight::get()
                    .reads(1)
                    .saturating_add(extra_weight))
            }
        }
        #[cfg(feature = "parachain")]
        pub struct StakingRoundSlotProvider;

        #[cfg(feature = "parachain")]
        impl Get<Slot> for StakingRoundSlotProvider {
            fn get() -> Slot {
                let block_number: u64 =
                    frame_system::pallet::Pallet::<Runtime>::block_number().into();
                Slot::from(block_number)
            }
        }

        #[cfg(feature = "parachain")]
        impl pallet_parachain_staking::Config for Runtime {
            type BlockAuthor = AuthorInherent;
            type BlockTime = BlockTime;
            type CandidateBondLessDelay = CandidateBondLessDelay;
            type Currency = Balances;
            type DelegationBondLessDelay = DelegationBondLessDelay;
            type RuntimeEvent = RuntimeEvent;
            type LeaveCandidatesDelay = LeaveCandidatesDelay;
            type LeaveDelegatorsDelay = LeaveDelegatorsDelay;
            type MaxBottomDelegationsPerCandidate = MaxBottomDelegationsPerCandidate;
            type MaxCandidates = MaxCandidates;
            type MaxDelegationsPerDelegator = MaxDelegationsPerDelegator;
            type MaxTopDelegationsPerCandidate = MaxTopDelegationsPerCandidate;
            type MaxOfflineRounds = MaxOfflineRounds;
            type MinBlocksPerRound = MinBlocksPerRound;
            type MinCandidateStk = MinCandidateStk;
            type MinDelegation = MinDelegation;
            type MinSelectedCandidates = MinSelectedCandidates;
            type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
            type OnCollatorPayout = ();
            type OnInactiveCollator = OnInactiveCollator;
            type PayoutCollatorReward = ();
            type OnNewRound = ();
            type RevokeDelegationDelay = RevokeDelegationDelay;
            type RewardPaymentDelay = RewardPaymentDelay;
            type SlotDuration = SlotDuration;
            type SlotProvider = StakingRoundSlotProvider;
            type WeightInfo = weights::pallet_parachain_staking::WeightInfo<Runtime>;
        }

        #[cfg(feature = "parachain")]
        impl orml_asset_registry::module::Config for Runtime {
            type AssetId = CurrencyId;
            type AssetProcessor = CustomAssetProcessor;
            type AuthorityOrigin = AsEnsureOriginWithArg<EnsureRootOrThreeFifthsCouncil>;
            type Balance = Balance;
            type CustomMetadata = CustomMetadata;
            type RuntimeEvent = RuntimeEvent;
            type StringLimit = AssetRegistryStringLimit;
            type WeightInfo = ();
        }

        impl orml_currencies::Config for Runtime {
            type GetNativeCurrencyId = GetNativeCurrencyId;
            type MultiCurrency = Tokens;
            type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances>;
            type WeightInfo = weights::orml_currencies::WeightInfo<Runtime>;
        }

        pub struct CurrencyHooks<R>(sp_std::marker::PhantomData<R>);
        impl<C: orml_tokens::Config>
            orml_traits::currency::MutationHooks<AccountId, CurrencyId, Balance>
            for CurrencyHooks<C>
        {
            type OnDust = orml_tokens::TransferDust<Runtime, ZeitgeistTreasuryAccount>;
            type OnKilledTokenAccount = ();
            type OnNewTokenAccount = ();
            type OnSlash = ();
            type PostDeposit = ();
            type PostTransfer = ();
            type PreDeposit = ();
            type PreTransfer = ();
        }

        impl orml_tokens::Config for Runtime {
            type Amount = Amount;
            type Balance = Balance;
            type CurrencyHooks = CurrencyHooks<Runtime>;
            type CurrencyId = CurrencyId;
            type DustRemovalWhitelist = DustRemovalWhitelist;
            type RuntimeEvent = RuntimeEvent;
            type ExistentialDeposits = ExistentialDeposits;
            type MaxLocks = MaxLocks;
            type MaxReserves = MaxReserves;
            type ReserveIdentifier = [u8; 8];
            type WeightInfo = weights::orml_tokens::WeightInfo<Runtime>;
        }

        #[cfg(feature = "parachain")]
        impl orml_unknown_tokens::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
        }

        #[cfg(feature = "parachain")]
        impl orml_xtokens::Config for Runtime {
            type AccountIdToLocation = AccountIdToLocation;
            type Balance = Balance;
            type BaseXcmWeight = BaseXcmWeight;
            type CurrencyId = CurrencyId;
            type CurrencyIdConvert = AssetConvert;
            type RuntimeEvent = RuntimeEvent;
            type MaxAssetsForTransfer = MaxAssetsForTransfer;
            type MinXcmFee = ParachainMinFee;
            type LocationsFilter = Everything;
            type RateLimiter = ();
            type RateLimiterId = ();
            type ReserveProvider = orml_traits::location::AbsoluteReserveProvider;
            type SelfLocation = SelfLocation;
            type UniversalLocation = UniversalLocation;
            type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
            type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
        }

        pub struct DustIntoTreasury;
        type CreditOfBalances = CreditOf<Runtime, ()>;
        impl OnUnbalanced<CreditOfBalances> for DustIntoTreasury {
            fn on_nonzero_unbalanced(mut dust: CreditOfBalances) {
                let imbalance = NegativeImbalance::new(dust.peek());
                Treasury::on_nonzero_unbalanced(imbalance);
                // Ensure issuance is not reduced via OnDrop
                core::mem::forget(dust);
            }
        }

        impl pallet_balances::Config for Runtime {
            type AccountStore = System;
            type Balance = Balance;
            type DustRemoval = DustIntoTreasury;
            type ExistentialDeposit = ExistentialDeposit;
            type FreezeIdentifier = ();
            type MaxFreezes = MaxFreezes;
            type MaxLocks = MaxLocks;
            type MaxReserves = MaxReserves;
            type ReserveIdentifier = [u8; 8];
            type RuntimeEvent = RuntimeEvent;
            type RuntimeHoldReason = RuntimeHoldReason;
            type RuntimeFreezeReason = RuntimeFreezeReason;
            type WeightInfo = weights::pallet_balances::WeightInfo<Runtime>;
        }

        impl pallet_collective::Config<AdvisoryCommitteeInstance> for Runtime {
            type DefaultVote = PrimeDefaultVote;
            type RuntimeEvent = RuntimeEvent;
            type MaxMembers = AdvisoryCommitteeMaxMembers;
            type MaxProposals = AdvisoryCommitteeMaxProposals;
            type MaxProposalWeight = MaxProposalWeight;
            type MotionDuration = AdvisoryCommitteeMotionDuration;
            type RuntimeOrigin = RuntimeOrigin;
            type SetMembersOrigin = EnsureRoot<AccountId>;
            type Proposal = RuntimeCall;
            type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
        }

        impl pallet_collective::Config<CouncilInstance> for Runtime {
            type DefaultVote = PrimeDefaultVote;
            type RuntimeEvent = RuntimeEvent;
            type MaxMembers = CouncilMaxMembers;
            type MaxProposals = CouncilMaxProposals;
            type MaxProposalWeight = MaxProposalWeight;
            type MotionDuration = CouncilMotionDuration;
            type RuntimeOrigin = RuntimeOrigin;
            type SetMembersOrigin = EnsureRoot<AccountId>;
            type Proposal = RuntimeCall;
            type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
        }

        impl pallet_collective::Config<TechnicalCommitteeInstance> for Runtime {
            type DefaultVote = PrimeDefaultVote;
            type RuntimeEvent = RuntimeEvent;
            type MaxMembers = TechnicalCommitteeMaxMembers;
            type MaxProposals = TechnicalCommitteeMaxProposals;
            type MaxProposalWeight = MaxProposalWeight;
            type MotionDuration = TechnicalCommitteeMotionDuration;
            type RuntimeOrigin = RuntimeOrigin;
            type SetMembersOrigin = EnsureRoot<AccountId>;
            type Proposal = RuntimeCall;
            type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
        }

        impl pallet_democracy::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            type Currency = Balances;
            type EnactmentPeriod = EnactmentPeriod;
            type LaunchPeriod = LaunchPeriod;
            type VotingPeriod = VotingPeriod;
            type VoteLockingPeriod = VoteLockingPeriod;
            type MinimumDeposit = MinimumDeposit;
            /// Origin that can decide what their next motion is.
            type ExternalOrigin = EnsureRootOrHalfCouncil;
            /// Origin that can have the next scheduled referendum be a straight majority-carries vote.
            type ExternalMajorityOrigin = EnsureRootOrHalfCouncil;
            /// Origina that can have the next scheduled referendum be a straight default-carries
            /// (NTB) vote.
            type ExternalDefaultOrigin = EnsureRootOrAllCouncil;
            /// Origin that can have an ExternalMajority/ExternalDefault vote
            /// be tabled immediately and with a shorter voting/enactment period.
            type FastTrackOrigin = EnsureRootOrThreeFifthsTechnicalCommittee;
            /// Origin from which the next majority-carries (or more permissive) referendum may be tabled
            /// to vote immediately and asynchronously in a similar manner to the emergency origin.
            type InstantOrigin = EnsureRootOrAllTechnicalCommittee;
            type InstantAllowed = InstantAllowed;
            type FastTrackVotingPeriod = FastTrackVotingPeriod;
            /// Origin from which any referendum may be cancelled in an emergency.
            type CancellationOrigin = EnsureRootOrThreeFourthsCouncil;
            /// Origin from which proposals may be blacklisted.
            type BlacklistOrigin = EnsureRootOrAllCouncil;
            /// Origin from which a proposal may be cancelled and its backers slashed.
            type CancelProposalOrigin = EnsureRootOrAllTechnicalCommittee;
            /// Origin for anyone able to veto proposals.
            type VetoOrigin =
                pallet_collective::EnsureMember<AccountId, TechnicalCommitteeInstance>;
            type CooloffPeriod = CooloffPeriod;
            type Slash = Treasury;
            type Scheduler = Scheduler;
            type SubmitOrigin = EnsureSigned<AccountId>;
            type PalletsOrigin = OriginCaller;
            type MaxVotes = MaxVotes;
            type WeightInfo = weights::pallet_democracy::WeightInfo<Runtime>;
            type MaxProposals = DemocracyMaxProposals;
            type Preimages = Preimage;
            type MaxBlacklisted = ConstU32<100>;
            type MaxDeposits = ConstU32<100>;
        }

        impl pallet_identity::Config for Runtime {
            type BasicDeposit = BasicDeposit;
            type ByteDeposit = IdentityByteDeposit;
            type Currency = Balances;
            type IdentityInformation = pallet_identity::legacy::IdentityInfo<MaxAdditionalFields>;
            type RuntimeEvent = RuntimeEvent;
            type ForceOrigin = EnsureRootOrHalfCouncil;
            type MaxRegistrars = MaxRegistrars;
            type MaxSubAccounts = MaxSubAccounts;
            type MaxSuffixLength = MaxSuffixLength;
            type MaxUsernameLength = MaxUsernameLength;
            type OffchainSignature = Signature;
            type PendingUsernameExpiration = PendingUsernameExpiration;
            type RegistrarOrigin = EnsureRootOrHalfCouncil;
            type Slashed = Treasury;
            type SubAccountDeposit = SubAccountDeposit;
            type SigningPublicKey = <Signature as sp_runtime::traits::Verify>::Signer;
            type UsernameAuthorityOrigin = EnsureRoot<AccountId>;
            type WeightInfo = weights::pallet_identity::WeightInfo<Runtime>;
        }

        impl pallet_membership::Config<AdvisoryCommitteeMembershipInstance> for Runtime {
            type AddOrigin = EnsureRootOrThreeFifthsCouncil;
            type RuntimeEvent = RuntimeEvent;
            type MaxMembers = AdvisoryCommitteeMaxMembers;
            type MembershipChanged = AdvisoryCommittee;
            type MembershipInitialized = AdvisoryCommittee;
            type PrimeOrigin = EnsureRootOrThreeFifthsCouncil;
            type RemoveOrigin = EnsureRootOrThreeFifthsCouncil;
            type ResetOrigin = EnsureRootOrThreeFifthsCouncil;
            type SwapOrigin = EnsureRootOrThreeFifthsCouncil;
            type WeightInfo = weights::pallet_membership::WeightInfo<Runtime>;
        }

        impl pallet_membership::Config<CouncilMembershipInstance> for Runtime {
            type AddOrigin = EnsureRootOrThreeFifthsCouncil;
            type RuntimeEvent = RuntimeEvent;
            type MaxMembers = CouncilMaxMembers;
            type MembershipChanged = Council;
            type MembershipInitialized = Council;
            type PrimeOrigin = EnsureRootOrThreeFifthsCouncil;
            type RemoveOrigin = EnsureRootOrThreeFifthsCouncil;
            type ResetOrigin = EnsureRootOrThreeFifthsCouncil;
            type SwapOrigin = EnsureRootOrThreeFifthsCouncil;
            type WeightInfo = weights::pallet_membership::WeightInfo<Runtime>;
        }

        impl pallet_membership::Config<TechnicalCommitteeMembershipInstance> for Runtime {
            type AddOrigin = EnsureRootOrThreeFifthsCouncil;
            type RuntimeEvent = RuntimeEvent;
            type MaxMembers = TechnicalCommitteeMaxMembers;
            type MembershipChanged = TechnicalCommittee;
            type MembershipInitialized = TechnicalCommittee;
            type PrimeOrigin = EnsureRootOrThreeFifthsCouncil;
            type RemoveOrigin = EnsureRootOrThreeFifthsCouncil;
            type ResetOrigin = EnsureRootOrThreeFifthsCouncil;
            type SwapOrigin = EnsureRootOrThreeFifthsCouncil;
            type WeightInfo = weights::pallet_membership::WeightInfo<Runtime>;
        }

        impl pallet_multisig::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            type RuntimeCall = RuntimeCall;
            type Currency = Balances;
            type DepositBase = DepositBase;
            type DepositFactor = DepositFactor;
            type MaxSignatories = ConstU32<100>;
            type WeightInfo = weights::pallet_multisig::WeightInfo<Runtime>;
        }

        impl pallet_preimage::Config for Runtime {
            type WeightInfo = weights::pallet_preimage::WeightInfo<Runtime>;
            type RuntimeEvent = RuntimeEvent;
            type Currency = Balances;
            type ManagerOrigin = EnsureRoot<AccountId>;
            type Consideration = HoldConsideration<
                AccountId,
                Balances,
                PreimageHoldReason,
                LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
            >;
        }

        impl InstanceFilter<RuntimeCall> for ProxyType {
            fn filter(&self, c: &RuntimeCall) -> bool {
                match self {
                    ProxyType::Any => true,
                    ProxyType::CancelProxy => {
                        matches!(
                            c,
                            RuntimeCall::Proxy(pallet_proxy::Call::reject_announcement { .. })
                        )
                    }
                    ProxyType::Governance => matches!(
                        c,
                        RuntimeCall::Democracy(..)
                            | RuntimeCall::Council(..)
                            | RuntimeCall::TechnicalCommittee(..)
                            | RuntimeCall::AdvisoryCommittee(..)
                            | RuntimeCall::Treasury(..)
                    ),
                    #[cfg(feature = "parachain")]
                    ProxyType::Staking => matches!(c, RuntimeCall::ParachainStaking(..)),
                    #[cfg(not(feature = "parachain"))]
                    ProxyType::Staking => false,
                    ProxyType::CreateEditMarket => matches!(
                        c,
                        RuntimeCall::PredictionMarkets(
                            zrml_prediction_markets::Call::create_market { .. }
                        ) | RuntimeCall::PredictionMarkets(
                            zrml_prediction_markets::Call::edit_market { .. }
                        )
                    ),
                    ProxyType::ReportOutcome => matches!(
                        c,
                        RuntimeCall::PredictionMarkets(
                            zrml_prediction_markets::Call::report { .. }
                        )
                    ),
                    ProxyType::Dispute => matches!(
                        c,
                        RuntimeCall::PredictionMarkets(
                            zrml_prediction_markets::Call::dispute { .. }
                        )
                    ),
                    ProxyType::ProvideLiquidity => matches!(
                        c,
                        RuntimeCall::NeoSwaps(zrml_neo_swaps::Call::join { .. })
                            | RuntimeCall::NeoSwaps(zrml_neo_swaps::Call::exit { .. })
                            | RuntimeCall::NeoSwaps(zrml_neo_swaps::Call::deploy_pool { .. })
                            | RuntimeCall::NeoSwaps(zrml_neo_swaps::Call::withdraw_fees { .. })
                    ),
                    ProxyType::BuySellCompleteSets => matches!(
                        c,
                        RuntimeCall::PredictionMarkets(
                            zrml_prediction_markets::Call::buy_complete_set { .. }
                        ) | RuntimeCall::PredictionMarkets(
                            zrml_prediction_markets::Call::sell_complete_set { .. }
                        )
                    ),
                    ProxyType::Trading => matches!(
                        c,
                        RuntimeCall::HybridRouter(zrml_hybrid_router::Call::buy { .. })
                            | RuntimeCall::HybridRouter(zrml_hybrid_router::Call::sell { .. })
                            | RuntimeCall::NeoSwaps(zrml_neo_swaps::Call::buy { .. })
                            | RuntimeCall::NeoSwaps(zrml_neo_swaps::Call::sell { .. })
                            | RuntimeCall::Orderbook(zrml_orderbook::Call::place_order { .. })
                            | RuntimeCall::Orderbook(zrml_orderbook::Call::fill_order { .. })
                            | RuntimeCall::Orderbook(zrml_orderbook::Call::remove_order { .. })
                    ),
                    ProxyType::HandleAssets => matches!(
                        c,
                        RuntimeCall::HybridRouter(zrml_hybrid_router::Call::buy { .. })
                            | RuntimeCall::HybridRouter(zrml_hybrid_router::Call::sell { .. })
                            | RuntimeCall::NeoSwaps(zrml_neo_swaps::Call::join { .. })
                            | RuntimeCall::NeoSwaps(zrml_neo_swaps::Call::exit { .. })
                            | RuntimeCall::NeoSwaps(zrml_neo_swaps::Call::buy { .. })
                            | RuntimeCall::NeoSwaps(zrml_neo_swaps::Call::sell { .. })
                            | RuntimeCall::NeoSwaps(zrml_neo_swaps::Call::deploy_pool { .. })
                            | RuntimeCall::NeoSwaps(zrml_neo_swaps::Call::withdraw_fees { .. })
                            | RuntimeCall::PredictionMarkets(
                                zrml_prediction_markets::Call::buy_complete_set { .. }
                            )
                            | RuntimeCall::PredictionMarkets(
                                zrml_prediction_markets::Call::sell_complete_set { .. }
                            )
                            | RuntimeCall::Orderbook(zrml_orderbook::Call::place_order { .. })
                            | RuntimeCall::Orderbook(zrml_orderbook::Call::fill_order { .. })
                            | RuntimeCall::Orderbook(zrml_orderbook::Call::remove_order { .. })
                    ),
                }
            }

            fn is_superset(&self, o: &Self) -> bool {
                match (self, o) {
                    (x, y) if x == y => true,
                    (ProxyType::Any, _) => true,
                    (_, ProxyType::Any) => false,
                    _ => false,
                }
            }
        }

        impl pallet_proxy::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            type RuntimeCall = RuntimeCall;
            type Currency = Balances;
            type ProxyType = ProxyType;
            type ProxyDepositBase = ProxyDepositBase;
            type ProxyDepositFactor = ProxyDepositFactor;
            type MaxProxies = ConstU32<32>;
            type WeightInfo = weights::pallet_proxy::WeightInfo<Runtime>;
            type MaxPending = ConstU32<32>;
            type CallHasher = BlakeTwo256;
            type AnnouncementDepositBase = AnnouncementDepositBase;
            type AnnouncementDepositFactor = AnnouncementDepositFactor;
        }

        impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

        impl pallet_scheduler::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            type RuntimeOrigin = RuntimeOrigin;
            type PalletsOrigin = OriginCaller;
            type RuntimeCall = RuntimeCall;
            type MaximumWeight = MaximumSchedulerWeight;
            type ScheduleOrigin = EnsureRoot<AccountId>;
            #[cfg(feature = "runtime-benchmarks")]
            type MaxScheduledPerBlock = ConstU32<512>;
            #[cfg(not(feature = "runtime-benchmarks"))]
            type MaxScheduledPerBlock = MaxScheduledPerBlock;
            type WeightInfo = weights::pallet_scheduler::WeightInfo<Runtime>;
            type OriginPrivilegeCmp = EqualPrivilegeOnly;
            type Preimages = Preimage;
        }

        // Timestamp
        /// Custom getter for minimum timestamp delta.
        /// This ensures that consensus systems like Aura don't break assertions
        /// in a benchmark environment
        pub struct MinimumPeriod;
        impl MinimumPeriod {
            /// Returns the value of this parameter type.
            pub fn get() -> u64 {
                MinimumPeriodValue::get()
            }
        }
        impl<I: From<u64>> frame_support::traits::Get<I> for MinimumPeriod {
            fn get() -> I {
                I::from(Self::get())
            }
        }
        impl frame_support::traits::TypedGet for MinimumPeriod {
            type Type = u64;
            fn get() -> u64 {
                Self::get()
            }
        }

        impl pallet_timestamp::Config for Runtime {
            type MinimumPeriod = MinimumPeriod;
            type Moment = u64;
            type OnTimestampSet = ();
            type WeightInfo = weights::pallet_timestamp::WeightInfo<Runtime>;
        }

        common_runtime::impl_foreign_fees!();

        impl pallet_asset_tx_payment::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            type Fungibles = Tokens;
            type OnChargeAssetTransaction = TokensTxCharger;
        }

        impl pallet_transaction_payment::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Runtime>;
            type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
            type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<
                Balances,
                DealWithSubstrateFeesAndTip<Runtime, FeesTreasuryProportion>,
            >;
            type OperationalFeeMultiplier = OperationalFeeMultiplier;
            type WeightToFee = IdentityFee<Balance>;
        }

        #[cfg(feature = "runtime-benchmarks")]
        pub struct TreasuryBenchmarkHelper;

        #[cfg(feature = "runtime-benchmarks")]
        impl pallet_treasury::ArgumentsFactory<(), AccountId> for TreasuryBenchmarkHelper {
            fn create_asset_kind(_seed: u32) {
                // No-op
            }

            fn create_beneficiary(seed: [u8; 32]) -> AccountId {
                AccountId::from(seed)
            }
        }

        impl pallet_treasury::Config for Runtime {
            type AssetKind = ();
            type BalanceConverter = UnityAssetBalanceConversion;
            type Beneficiary = AccountId;
            type BeneficiaryLookup = IdentityLookup<AccountId>;
            type Burn = Burn;
            type BurnDestination = ();
            type Currency = Balances;
            type RuntimeEvent = RuntimeEvent;
            type MaxApprovals = MaxApprovals;
            type PalletId = TreasuryPalletId;
            type Paymaster = PayFromAccount<Balances, ZeitgeistTreasuryAccount>;
            type PayoutPeriod = PayoutPeriod;
            type RejectOrigin = EnsureRootOrTwoThirdsCouncil;
            type SpendFunds = Bounties;
            type SpendOrigin =
                EnsureWithSuccess<EnsureRoot<AccountId>, AccountId, MaxTreasurySpend>;
            type SpendPeriod = SpendPeriod;
            type WeightInfo = weights::pallet_treasury::WeightInfo<Runtime>;
            #[cfg(feature = "runtime-benchmarks")]
            type BenchmarkHelper = TreasuryBenchmarkHelper;
        }

        impl pallet_bounties::Config for Runtime {
            type BountyDepositBase = BountyDepositBase;
            type BountyDepositPayoutDelay = BountyDepositPayoutDelay;
            type BountyUpdatePeriod = BountyUpdatePeriod;
            type BountyValueMinimum = BountyValueMinimum;
            type ChildBountyManager = ();
            type CuratorDepositMax = CuratorDepositMax;
            type CuratorDepositMin = CuratorDepositMin;
            type CuratorDepositMultiplier = CuratorDepositMultiplier;
            type DataDepositPerByte = DataDepositPerByte;
            type RuntimeEvent = RuntimeEvent;
            type MaximumReasonLength = MaximumReasonLength;
            type OnSlash = Treasury;
            type WeightInfo = weights::pallet_bounties::WeightInfo<Runtime>;
        }

        impl pallet_utility::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            type RuntimeCall = RuntimeCall;
            type PalletsOrigin = OriginCaller;
            type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
        }

        impl pallet_vesting::Config for Runtime {
            type BlockNumberProvider = System;
            type RuntimeEvent = RuntimeEvent;
            type Currency = Balances;
            type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
            type MinVestedTransfer = MinVestedTransfer;
            type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
            type WeightInfo = weights::pallet_vesting::WeightInfo<Runtime>;

            // `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
            // highest number of schedules that encodes less than 2^10.
            const MAX_VESTING_SCHEDULES: u32 = 28;
        }

        #[cfg(feature = "parachain")]
        impl parachain_info::Config for Runtime {}

        impl zrml_authorized::Config for Runtime {
            type AuthorizedDisputeResolutionOrigin = EnsureRootOrMoreThanHalfAdvisoryCommittee;
            type Currency = Balances;
            type CorrectionPeriod = CorrectionPeriod;
            type DisputeResolution = zrml_prediction_markets::Pallet<Runtime>;
            type RuntimeEvent = RuntimeEvent;
            type MarketCommons = MarketCommons;
            type PalletId = AuthorizedPalletId;
            type WeightInfo = zrml_authorized::weights::WeightInfo<Runtime>;
        }

        impl zrml_combinatorial_tokens::Config for Runtime {
            #[cfg(feature = "runtime-benchmarks")]
            type BenchmarkHelper = PredictionMarketsCombinatorialTokensBenchmarkHelper<Runtime>;
            type CombinatorialIdManager = CryptographicIdManager<MarketId, Blake2_256>;
            type Fuel = Fuel;
            type MarketCommons = MarketCommons;
            type MultiCurrency = AssetManager;
            type Payout = PredictionMarkets;
            type RuntimeEvent = RuntimeEvent;
            type PalletId = CombinatorialTokensPalletId;
            type WeightInfo = zrml_combinatorial_tokens::weights::WeightInfo<Runtime>;
        }

        impl zrml_court::Config for Runtime {
            type AppealBond = AppealBond;
            type BlocksPerYear = BlocksPerYear;
            type VotePeriod = CourtVotePeriod;
            type AggregationPeriod = CourtAggregationPeriod;
            type AppealPeriod = CourtAppealPeriod;
            type LockId = CourtLockId;
            type PalletId = CourtPalletId;
            type Currency = Balances;
            type DisputeResolution = zrml_prediction_markets::Pallet<Runtime>;
            type RuntimeEvent = RuntimeEvent;
            type InflationPeriod = InflationPeriod;
            type MarketCommons = MarketCommons;
            type MaxAppeals = MaxAppeals;
            type MaxDelegations = MaxDelegations;
            type MaxSelectedDraws = MaxSelectedDraws;
            type MaxCourtParticipants = MaxCourtParticipants;
            type MaxYearlyInflation = MaxYearlyInflation;
            type MinJurorStake = MinJurorStake;
            type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
            type Random = RandomnessCollectiveFlip;
            type RequestInterval = RequestInterval;
            type Slash = Treasury;
            type TreasuryPalletId = TreasuryPalletId;
            type WeightInfo = zrml_court::weights::WeightInfo<Runtime>;
        }

        impl zrml_futarchy::Config for Runtime {
            #[cfg(feature = "runtime-benchmarks")]
            type BenchmarkHelper = DecisionMarketBenchmarkHelper<Runtime>;
            type MaxProposals = FutarchyMaxProposals;
            type MinDuration = MinDuration;
            type Oracle = DecisionMarketOracle<Runtime>;
            type RuntimeEvent = RuntimeEvent;
            type Scheduler = Scheduler;
            type WeightInfo = zrml_futarchy::weights::WeightInfo<Runtime>;
        }

        impl zrml_market_commons::Config for Runtime {
            type Balance = Balance;
            type MarketId = MarketId;
            type Timestamp = Timestamp;
        }

        impl zrml_prediction_markets::Config for Runtime {
            type AdvisoryBond = AdvisoryBond;
            type AdvisoryBondSlashPercentage = AdvisoryBondSlashPercentage;
            type ApproveOrigin = EnsureRootOrMoreThanOneThirdAdvisoryCommittee;
            type Authorized = Authorized;
            type Currency = Balances;
            type Court = Court;
            type CloseEarlyDisputeBond = CloseEarlyDisputeBond;
            type CloseMarketEarlyOrigin = EnsureRootOrMoreThanOneThirdAdvisoryCommittee;
            type CloseOrigin = EnsureRoot<AccountId>;
            type CloseEarlyProtectionTimeFramePeriod = CloseEarlyProtectionTimeFramePeriod;
            type CloseEarlyProtectionBlockPeriod = CloseEarlyProtectionBlockPeriod;
            type CloseEarlyRequestBond = CloseEarlyRequestBond;
            type DeployPool = NeoSwaps;
            type DisputeBond = DisputeBond;
            type RuntimeEvent = RuntimeEvent;
            type GlobalDisputes = GlobalDisputes;
            type MaxCategories = MaxCategories;
            type MaxCreatorFee = MaxCreatorFee;
            type MaxDisputes = MaxDisputes;
            type MaxMarketLifetime = MaxMarketLifetime;
            type MinDisputeDuration = MinDisputeDuration;
            type MaxDisputeDuration = MaxDisputeDuration;
            type MaxGracePeriod = MaxGracePeriod;
            type MaxOracleDuration = MaxOracleDuration;
            type MinOracleDuration = MinOracleDuration;
            type MinCategories = MinCategories;
            type MaxEditReasonLen = MaxEditReasonLen;
            type MaxRejectReasonLen = MaxRejectReasonLen;
            type OracleBond = OracleBond;
            type OutsiderBond = OutsiderBond;
            type PalletId = PmPalletId;
            type CloseEarlyBlockPeriod = CloseEarlyBlockPeriod;
            type CloseEarlyTimeFramePeriod = CloseEarlyTimeFramePeriod;
            type RejectOrigin = EnsureRootOrMoreThanTwoThirdsAdvisoryCommittee;
            type RequestEditOrigin = EnsureRootOrMoreThanOneThirdAdvisoryCommittee;
            type ResolveOrigin = EnsureRoot<AccountId>;
            type AssetManager = AssetManager;
            #[cfg(feature = "parachain")]
            type AssetRegistry = AssetRegistry;
            type Slash = Treasury;
            type ValidityBond = ValidityBond;
            type WeightInfo = zrml_prediction_markets::weights::WeightInfo<Runtime>;
        }

        impl zrml_global_disputes::Config for Runtime {
            type AddOutcomePeriod = AddOutcomePeriod;
            type Currency = Balances;
            type DisputeResolution = zrml_prediction_markets::Pallet<Runtime>;
            type RuntimeEvent = RuntimeEvent;
            type GlobalDisputeLockId = GlobalDisputeLockId;
            type GlobalDisputesPalletId = GlobalDisputesPalletId;
            type MarketCommons = MarketCommons;
            type MaxGlobalDisputeVotes = MaxGlobalDisputeVotes;
            type MaxOwners = MaxOwners;
            type MinOutcomeVoteAmount = MinOutcomeVoteAmount;
            type RemoveKeysLimit = RemoveKeysLimit;
            type GdVotingPeriod = GdVotingPeriod;
            type VotingOutcomeFee = VotingOutcomeFee;
            type WeightInfo = zrml_global_disputes::weights::WeightInfo<Runtime>;
        }

        impl zrml_swaps::Config for Runtime {
            type Asset = Asset<MarketId>;
            type RuntimeEvent = RuntimeEvent;
            type MultiCurrency = AssetManager;
            type ExitFee = ExitFee;
            type MinAssets = MinAssets;
            type MaxAssets = MaxAssets;
            type MaxSwapFee = MaxSwapFee;
            type MaxTotalWeight = MaxTotalWeight;
            type MaxWeight = MaxWeight;
            type MinWeight = MinWeight;
            type PalletId = SwapsPalletId;
            type WeightInfo = zrml_swaps::weights::WeightInfo<Runtime>;
        }

        impl zrml_styx::Config for Runtime {
            type RuntimeEvent = RuntimeEvent;
            type SetBurnAmountOrigin = EnsureRootOrHalfCouncil;
            type Currency = Balances;
            type WeightInfo = zrml_styx::weights::WeightInfo<Runtime>;
        }

        common_runtime::impl_market_creator_fees!();

        impl zrml_neo_swaps::Config for Runtime {
            type CombinatorialId = CombinatorialId;
            type CombinatorialTokens = CombinatorialTokens;
            type CombinatorialTokensUnsafe = CombinatorialTokens;
            type CompleteSetOperations = PredictionMarkets;
            type ExternalFees = MarketCreatorFee;
            type MarketCommons = MarketCommons;
            type MultiCurrency = AssetManager;
            type PoolId = MarketId;
            type RuntimeEvent = RuntimeEvent;
            type WeightInfo = zrml_neo_swaps::weights::WeightInfo<Runtime>;
            type MaxLiquidityTreeDepth = MaxLiquidityTreeDepth;
            type MaxSplits = MaxSplits;
            type MaxSwapFee = NeoSwapsMaxSwapFee;
            type PalletId = NeoSwapsPalletId;
        }

        impl zrml_orderbook::Config for Runtime {
            type AssetManager = AssetManager;
            type ExternalFees = MarketCreatorFee;
            type RuntimeEvent = RuntimeEvent;
            type MarketCommons = MarketCommons;
            type PalletId = OrderbookPalletId;
            type WeightInfo = zrml_orderbook::weights::WeightInfo<Runtime>;
        }

        impl zrml_parimutuel::Config for Runtime {
            type ExternalFees = MarketCreatorFee;
            type RuntimeEvent = RuntimeEvent;
            type MarketCommons = MarketCommons;
            type AssetManager = AssetManager;
            type MinBetSize = MinBetSize;
            type PalletId = ParimutuelPalletId;
            type WeightInfo = zrml_parimutuel::weights::WeightInfo<Runtime>;
        }

        impl zrml_hybrid_router::Config for Runtime {
            type AssetManager = AssetManager;
            #[cfg(feature = "runtime-benchmarks")]
            type AmmPoolDeployer = NeoSwaps;
            #[cfg(feature = "runtime-benchmarks")]
            type CompleteSetOperations = PredictionMarkets;
            type MarketCommons = MarketCommons;
            type Amm = NeoSwaps;
            type Orderbook = Orderbook;
            type MaxOrders = MaxOrders;
            type RuntimeEvent = RuntimeEvent;
            type PalletId = HybridRouterPalletId;
            type WeightInfo = zrml_hybrid_router::weights::WeightInfo<Runtime>;
        }
    };
}

#[macro_export]
macro_rules! create_genesis_config_preset {
    ($($additional_genesis_config:tt)*) => {
        use sp_core::{sr25519, Pair, Public};
        use sp_genesis_builder::PresetId;
        use sp_runtime::traits::{IdentifyAccount, Verify};
        use zeitgeist_primitives::types::{AccountId, Balance, Signature};
        #[cfg(feature = "parachain")]
        use {
            crate::{
                DefaultBlocksPerRound, DefaultCollatorCommission, DefaultParachainBondReservePercent,
            },
            nimbus_primitives::NimbusId,
            pallet_parachain_staking::InflationInfo,
            sp_runtime::Percent,
            zeitgeist_primitives::constants::{
                ztg::{STAKING_PTD, TOTAL_INITIAL_ZTG},
                BASE,
            },
        };

        const BATTERY_STATION_PARACHAIN_ID: u32 = 2101;
        #[cfg(feature = "parachain")]
        const DEFAULT_STAKING_AMOUNT_BATTERY_STATION: u128 = 2_000 * BASE;

        #[cfg(feature = "parachain")]
        pub fn inflation_config(
            annual_inflation_min: Perbill,
            annual_inflation_ideal: Perbill,
            annual_inflation_max: Perbill,
            total_supply: zeitgeist_primitives::types::Balance,
        ) -> pallet_parachain_staking::inflation::InflationInfo<zeitgeist_primitives::types::Balance> {
            fn to_round_inflation(
                annual: pallet_parachain_staking::inflation::Range<Perbill>,
            ) -> pallet_parachain_staking::inflation::Range<Perbill> {
                use crate::parachain_params::DefaultBlocksPerRound;
                use pallet_parachain_staking::inflation::perbill_annual_to_perbill_round;

                perbill_annual_to_perbill_round(
                    annual,
                    // rounds per year
                    u32::try_from(zeitgeist_primitives::constants::BLOCKS_PER_YEAR).unwrap()
                        / DefaultBlocksPerRound::get(),
                )
            }
            let annual = pallet_parachain_staking::inflation::Range {
                min: annual_inflation_min,
                ideal: annual_inflation_ideal,
                max: annual_inflation_max,
            };
            pallet_parachain_staking::inflation::InflationInfo {
                // staking expectations
                expect: pallet_parachain_staking::inflation::Range {
                    min: Perbill::from_percent(5).mul_floor(total_supply),
                    ideal: Perbill::from_percent(10).mul_floor(total_supply),
                    max: Perbill::from_percent(15).mul_floor(total_supply),
                },
                // annual inflation
                annual,
                round: to_round_inflation(annual),
            }
        }

        #[cfg(feature = "parachain")]
        pub struct AdditionalChainSpec {
            pub blocks_per_round: u32,
            pub candidates: Vec<(AccountId, NimbusId, Balance)>,
            pub collator_commission: Perbill,
            pub inflation_info: InflationInfo<Balance>,
            pub nominations: Vec<(AccountId, AccountId, Balance, Percent)>,
            pub parachain_bond_reserve_percent: Percent,
            pub parachain_id: ParaId,
            pub num_selected_candidates: u32,
        }

        #[cfg(not(feature = "parachain"))]
        pub struct AdditionalChainSpec {
            pub initial_authorities:
                Vec<(sp_consensus_aura::sr25519::AuthorityId, sp_consensus_grandpa::AuthorityId)>,
        }

        type AccountPublic = <Signature as Verify>::Signer;
        #[inline]
        fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
        where
            AccountPublic: From<<TPublic::Pair as Pair>::Public>,
        {
            AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
        }

        fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
            TPublic::Pair::from_string(&alloc::format!("//{}", seed), None)
                .expect("static values are valid; qed")
                .public()
        }

        #[derive(Clone)]
        pub struct EndowedAccountWithBalance(AccountId, Balance);

        pub fn generic_genesis(
            acs: AdditionalChainSpec,
            endowed_accounts: Vec<EndowedAccountWithBalance>,
        ) -> crate::RuntimeGenesisConfig {
            crate::RuntimeGenesisConfig {
                // Common genesis
                advisory_committee: Default::default(),
                advisory_committee_membership: crate::AdvisoryCommitteeMembershipConfig {
                    members: vec![].try_into().unwrap(),
                    phantom: Default::default(),
                },
                #[cfg(feature = "parachain")]
                asset_registry: Default::default(),
                #[cfg(not(feature = "parachain"))]
                aura: crate::AuraConfig {
                    authorities: acs.initial_authorities.iter().map(|x| (x.0.clone())).collect(),
                },
                #[cfg(feature = "parachain")]
                author_filter: crate::AuthorFilterConfig {
                    eligible_count: EligibilityValue::new_unchecked(1),
                    ..Default::default()
                },
                #[cfg(feature = "parachain")]
                author_mapping: crate::AuthorMappingConfig {
                    mappings: acs
                        .candidates
                        .iter()
                        .cloned()
                        .map(|(account_id, author_id, _)| (author_id, account_id))
                        .collect(),
                },
                balances: crate::BalancesConfig {
                    balances: endowed_accounts.iter().cloned().map(|k| (k.0, k.1)).collect(),
                },
                council: Default::default(),
                council_membership: crate::CouncilMembershipConfig {
                    members: vec![].try_into().unwrap(),
                    phantom: Default::default(),
                },
                democracy: Default::default(),
                #[cfg(not(feature = "parachain"))]
                grandpa: crate::GrandpaConfig {
                    authorities: acs.initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
                    ..Default::default()
                },
                #[cfg(feature = "parachain")]
                parachain_info: crate::ParachainInfoConfig {
                    parachain_id: acs.parachain_id,
                    ..Default::default()
                },
                #[cfg(feature = "parachain")]
                parachain_staking: crate::ParachainStakingConfig {
                    blocks_per_round: acs.blocks_per_round,
                    candidates: acs
                        .candidates
                        .iter()
                        .cloned()
                        .map(|(account, _, bond)| (account, bond))
                        .collect(),
                    collator_commission: acs.collator_commission,
                    inflation_config: acs.inflation_info,
                    delegations: acs.nominations,
                    parachain_bond_reserve_percent: acs.parachain_bond_reserve_percent,
                    num_selected_candidates: acs.num_selected_candidates,
                },
                #[cfg(feature = "parachain")]
                parachain_system: Default::default(),
                #[cfg(feature = "parachain")]
                // Default should use the pallet configuration
                polkadot_xcm: PolkadotXcmConfig::default(),
                system: crate::SystemConfig::default(),
                technical_committee: Default::default(),
                technical_committee_membership: crate::TechnicalCommitteeMembershipConfig {
                    members: vec![].try_into().unwrap(),
                    phantom: Default::default(),
                },
                treasury: Default::default(),
                transaction_payment: Default::default(),
                tokens: Default::default(),
                vesting: Default::default(),

                // Additional genesis
                $($additional_genesis_config)*
            }
        }

        const INITIAL_BALANCE: Balance = Balance::MAX >> 4;

        #[cfg(not(feature = "parachain"))]
        fn authority_keys_from_seed(
            s: &str,
        ) -> (sp_consensus_aura::sr25519::AuthorityId, sp_consensus_grandpa::AuthorityId) {
            (
                get_from_seed::<sp_consensus_aura::sr25519::AuthorityId>(s),
                get_from_seed::<sp_consensus_grandpa::AuthorityId>(s),
            )
        }

        fn get_genesis_config() -> serde_json::Value {
            serde_json::to_value(&generic_genesis(
                #[cfg(feature = "parachain")]
                AdditionalChainSpec {
                    blocks_per_round: DefaultBlocksPerRound::get(),
                    candidates: vec![(
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_from_seed::<nimbus_primitives::NimbusId>("Alice"),
                        DEFAULT_STAKING_AMOUNT_BATTERY_STATION,
                    )],
                    collator_commission: DefaultCollatorCommission::get(),
                    inflation_info: inflation_config(
                        STAKING_PTD * Perbill::from_percent(40),
                        STAKING_PTD * Perbill::from_percent(70),
                        STAKING_PTD,
                        TOTAL_INITIAL_ZTG * BASE,
                    ),
                    nominations: vec![],
                    parachain_bond_reserve_percent: DefaultParachainBondReservePercent::get(),
                    parachain_id: BATTERY_STATION_PARACHAIN_ID.into(),
                    num_selected_candidates: 8,
                },
                #[cfg(not(feature = "parachain"))]
                AdditionalChainSpec { initial_authorities: vec![authority_keys_from_seed("Alice")] },
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ]
                .into_iter()
                .map(|acc| EndowedAccountWithBalance(acc, INITIAL_BALANCE))
                .collect(),
            ))
            .expect("Could not generate JSON for battery station staging genesis.")
        }

        /// Provides the JSON representation of predefined genesis config for given `id`.
        pub fn get_genesis_config_preset(id: &PresetId) -> Option<Vec<u8>> {
            let patch = match id.try_into() {
                Ok(sp_genesis_builder::DEV_RUNTIME_PRESET) => get_genesis_config(),
                _ => return None,
            };
            Some(
                serde_json::to_string(&patch)
                    .expect("serialization to json is expected to work. qed.")
                    .into_bytes(),
            )
        }

        /// List of supported presets.
        pub fn get_genesis_config_preset_names() -> Vec<PresetId> {
            vec![PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET)]
        }
    };
}

// Implement runtime apis
#[macro_export]
macro_rules! create_runtime_api {
    ($($additional_apis:tt)*) => {
        impl_runtime_apis! {
            #[cfg(feature = "parachain")]
            impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
                fn collect_collation_info(
                    header: &<Block as BlockT>::Header
                ) -> cumulus_primitives_core::CollationInfo {
                    ParachainSystem::collect_collation_info(header)
                }
            }

            #[cfg(feature = "parachain")]
            impl nimbus_primitives::NimbusApi<Block> for Runtime {
                fn can_author(
                    author: nimbus_primitives::NimbusId,
                    slot: u32,
                    parent_header: &<Block as BlockT>::Header
                ) -> bool {

                    // Ensure that an update is enforced when we are close to maximum block number
                    let block_number = if let Some(bn) = parent_header.number.checked_add(1) {
                        bn
                    } else {
                        log::error!("ERROR: No block numbers left");
                        return false;
                    };

                    use frame_support::traits::OnInitialize;
                    System::initialize(
                        &block_number,
                        &parent_header.hash(),
                        &parent_header.digest,
                    );
                    RandomnessCollectiveFlip::on_initialize(block_number);

                    // Because the staking solution calculates the next staking set at the beginning
                    // of the first block in the new round, the only way to accurately predict the
                    // authors is to compute the selection during prediction.
                    if pallet_parachain_staking::Pallet::<Self>::round().should_update(block_number) {
                        // get author account id
                        use nimbus_primitives::AccountLookup;
                        let author_account_id = if let Some(account) =
                            pallet_author_mapping::Pallet::<Self>::lookup_account(&author) {
                            account
                        } else {
                            // return false if author mapping not registered like in can_author impl
                            return false
                        };

                        let candidates = pallet_parachain_staking::Pallet::<Self>::compute_top_candidates();
                        if candidates.is_empty() {
                            // If there are zero selected candidates, we use the same eligibility
                            // as the previous round
                            return AuthorInherent::can_author(&author, &slot);
                        }

                        // predict eligibility post-selection by computing selection results now
                        let (eligible, _) =
                            pallet_author_slot_filter::compute_pseudo_random_subset::<Self>(
                                candidates,
                                &slot
                            );

                        eligible.contains(&author_account_id)
                    } else {
                        AuthorInherent::can_author(&author, &slot)
                    }
                }
            }

            #[cfg(feature = "runtime-benchmarks")]
            impl frame_benchmarking::Benchmark<Block> for Runtime {
                fn benchmark_metadata(extra: bool) -> (
                    Vec<frame_benchmarking::BenchmarkList>,
                    Vec<frame_support::traits::StorageInfo>,
                ) {
                    use alloc::vec::Vec;
                    use frame_benchmarking::{list_benchmark, baseline::Pallet as BaselineBench, Benchmarking, BenchmarkList};
                    use frame_support::traits::StorageInfoTrait;
                    use frame_system_benchmarking::Pallet as SystemBench;
                    use orml_benchmarking::list_benchmark as orml_list_benchmark;

                    let mut list = Vec::<BenchmarkList>::new();

                    list_benchmark!(list, extra, frame_benchmarking, BaselineBench::<Runtime>);
                    list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
                    orml_list_benchmark!(list, extra, orml_currencies, crate::benchmarks::currencies);
                    orml_list_benchmark!(list, extra, orml_tokens, crate::benchmarks::tokens);
                    list_benchmark!(list, extra, pallet_balances, Balances);
                    list_benchmark!(list, extra, pallet_bounties, Bounties);
                    list_benchmark!(list, extra, pallet_collective, AdvisoryCommittee);
                    list_benchmark!(list, extra, pallet_democracy, Democracy);
                    list_benchmark!(list, extra, pallet_identity, Identity);
                    list_benchmark!(list, extra, pallet_membership, AdvisoryCommitteeMembership);
                    list_benchmark!(list, extra, pallet_multisig, Multisig);
                    list_benchmark!(list, extra, pallet_preimage, Preimage);
                    list_benchmark!(list, extra, pallet_proxy, Proxy);
                    list_benchmark!(list, extra, pallet_scheduler, Scheduler);
                    list_benchmark!(list, extra, pallet_timestamp, Timestamp);
                    list_benchmark!(list, extra, pallet_treasury, Treasury);
                    list_benchmark!(list, extra, pallet_utility, Utility);
                    list_benchmark!(list, extra, pallet_vesting, Vesting);
                    list_benchmark!(list, extra, zrml_swaps, Swaps);
                    list_benchmark!(list, extra, zrml_authorized, Authorized);
                    list_benchmark!(list, extra, zrml_combinatorial_tokens, CombinatorialTokens);
                    list_benchmark!(list, extra, zrml_court, Court);
                    list_benchmark!(list, extra, zrml_futarchy, Futarchy);
                    list_benchmark!(list, extra, zrml_global_disputes, GlobalDisputes);
                    list_benchmark!(list, extra, zrml_orderbook, Orderbook);
                    list_benchmark!(list, extra, zrml_parimutuel, Parimutuel);
                    list_benchmark!(list, extra, zrml_hybrid_router, HybridRouter);
                    #[cfg(not(feature = "parachain"))]
                    list_benchmark!(list, extra, zrml_prediction_markets, PredictionMarkets);
                    list_benchmark!(list, extra, zrml_styx, Styx);
                    list_benchmark!(list, extra, zrml_neo_swaps, NeoSwaps);

                    cfg_if::cfg_if! {
                        if #[cfg(feature = "parachain")] {
                            list_benchmark!(list, extra, cumulus_pallet_parachain_system, ParachainSystem);
                            list_benchmark!(list, extra, cumulus_pallet_xcmp_queue, XcmpQueue);
                            list_benchmark!(list, extra, cumulus_pallet_dmp_queue, DmpQueue);
                            list_benchmark!(list, extra, pallet_author_inherent, AuthorInherent);
                            list_benchmark!(list, extra, pallet_author_mapping, AuthorMapping);
                            list_benchmark!(list, extra, pallet_author_slot_filter, AuthorFilter);
                            list_benchmark!(list, extra, pallet_message_queue, MessageQueue);
                            list_benchmark!(list, extra, pallet_parachain_staking, ParachainStaking);
                        } else {
                            list_benchmark!(list, extra, pallet_grandpa, Grandpa);
                        }
                    }

                    (list, AllPalletsWithSystem::storage_info())
                }

                fn dispatch_benchmark(
                    config: frame_benchmarking::BenchmarkConfig,
                ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
                    use frame_benchmarking::{
                        add_benchmark,
                        baseline::{
                            Pallet as BaselineBench, Config as BaselineConfig
                        },
                        BenchmarkBatch, Benchmarking
                    };
                    use alloc::{vec, vec::Vec};
                    use frame_support::traits::{TrackedStorageKey, WhitelistedStorageKeys};
                    use frame_system_benchmarking::Pallet as SystemBench;
                    use orml_benchmarking::{add_benchmark as orml_add_benchmark};

                    #[allow(non_local_definitions)]
                    impl frame_system_benchmarking::Config for Runtime {}
                    #[allow(non_local_definitions)]
                    impl BaselineConfig for Runtime {}

                    let mut whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();
                    let additional_whitelist: Vec<TrackedStorageKey> = vec![
                        // ParachainStaking Round
                        hex_literal::hex!(  "a686a3043d0adcf2fa655e57bc595a78"
                                            "13792e785168f725b60e2969c7fc2552")
                            .to_vec().into(),
                        // Treasury Account (zge/tsry)
                        hex_literal::hex!(  "26aa394eea5630e07c48ae0c9558cef7"
                                            "b99d880ec681799c0cf30e8886371da9"
                                            "7be2919ac397ba499ea5e57132180ec6"
                                            "6d6f646c7a67652f7473727900000000"
                                            "00000000000000000000000000000000"
                        ).to_vec().into(),
                        // ParachainInfo ParachainId
                        hex_literal::hex!(  "0d715f2646c8f85767b5d2764bb27826"
                                            "04a74d81251e398fd8a0a4d55023bb3f")
                            .to_vec().into(),
                    ];
                    whitelist.extend(additional_whitelist.into_iter());

                    let mut batches = Vec::<BenchmarkBatch>::new();
                    let params = (&config, &whitelist);

                    add_benchmark!(params, batches, frame_benchmarking, BaselineBench::<Runtime>);
                    add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
                    orml_add_benchmark!(params, batches, orml_currencies, crate::benchmarks::currencies);
                    orml_add_benchmark!(params, batches, orml_tokens, crate::benchmarks::tokens);
                    add_benchmark!(params, batches, pallet_balances, Balances);
                    add_benchmark!(params, batches, pallet_bounties, Bounties);
                    add_benchmark!(params, batches, pallet_collective, AdvisoryCommittee);
                    add_benchmark!(params, batches, pallet_democracy, Democracy);
                    add_benchmark!(params, batches, pallet_identity, Identity);
                    add_benchmark!(params, batches, pallet_membership, AdvisoryCommitteeMembership);
                    add_benchmark!(params, batches, pallet_multisig, Multisig);
                    add_benchmark!(params, batches, pallet_preimage, Preimage);
                    add_benchmark!(params, batches, pallet_proxy, Proxy);
                    add_benchmark!(params, batches, pallet_scheduler, Scheduler);
                    add_benchmark!(params, batches, pallet_timestamp, Timestamp);
                    add_benchmark!(params, batches, pallet_treasury, Treasury);
                    add_benchmark!(params, batches, pallet_utility, Utility);
                    add_benchmark!(params, batches, pallet_vesting, Vesting);
                    add_benchmark!(params, batches, zrml_swaps, Swaps);
                    add_benchmark!(params, batches, zrml_authorized, Authorized);
                    add_benchmark!(params, batches, zrml_combinatorial_tokens, CombinatorialTokens);
                    add_benchmark!(params, batches, zrml_court, Court);
                    add_benchmark!(params, batches, zrml_futarchy, Futarchy);
                    add_benchmark!(params, batches, zrml_global_disputes, GlobalDisputes);
                    add_benchmark!(params, batches, zrml_orderbook, Orderbook);
                    add_benchmark!(params, batches, zrml_parimutuel, Parimutuel);
                    add_benchmark!(params, batches, zrml_hybrid_router, HybridRouter);
                    #[cfg(not(feature = "parachain"))]
                    add_benchmark!(params, batches, zrml_prediction_markets, PredictionMarkets);
                    add_benchmark!(params, batches, zrml_styx, Styx);
                    add_benchmark!(params, batches, zrml_neo_swaps, NeoSwaps);


                    cfg_if::cfg_if! {
                        if #[cfg(feature = "parachain")] {
                            add_benchmark!(params, batches, cumulus_pallet_parachain_system, ParachainSystem);
                            add_benchmark!(params, batches, cumulus_pallet_xcmp_queue, XcmpQueue);
                            add_benchmark!(params, batches, cumulus_pallet_dmp_queue, DmpQueue);
                            add_benchmark!(params, batches, pallet_author_inherent, AuthorInherent);
                            add_benchmark!(params, batches, pallet_author_mapping, AuthorMapping);
                            add_benchmark!(params, batches, pallet_author_slot_filter, AuthorFilter);
                            add_benchmark!(params, batches, pallet_message_queue, MessageQueue);
                            add_benchmark!(params, batches, pallet_parachain_staking, ParachainStaking);

                        } else {
                            add_benchmark!(params, batches, pallet_grandpa, Grandpa);
                        }
                    }

                    if batches.is_empty() {
                        return Err("Benchmark not found for this module.".into());
                    }
                    Ok(batches)
                }
            }

            impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
                fn account_nonce(account: AccountId) -> Nonce {
                    System::account_nonce(account)
                }
            }

            impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
                fn query_info(
                    uxt: <Block as BlockT>::Extrinsic,
                    len: u32,
                ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
                    TransactionPayment::query_info(uxt, len)
                }
                fn query_fee_details(
                    uxt: <Block as BlockT>::Extrinsic,
                    len: u32,
                ) -> pallet_transaction_payment::FeeDetails<Balance> {
                    TransactionPayment::query_fee_details(uxt, len)
                }
                fn query_weight_to_fee(weight: Weight) -> Balance {
                    TransactionPayment::weight_to_fee(weight)
                }
                fn query_length_to_fee(length: u32) -> Balance {
                    TransactionPayment::length_to_fee(length)
                }
            }

            impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
            for Runtime
            {
                fn query_call_info(
                    call: RuntimeCall,
                    len: u32,
                ) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
                    TransactionPayment::query_call_info(call, len)
                }

                fn query_call_fee_details(
                    call: RuntimeCall,
                    len: u32,
                ) -> pallet_transaction_payment::FeeDetails<Balance> {
                    TransactionPayment::query_call_fee_details(call, len)
                }

                fn query_weight_to_fee(weight: Weight) -> Balance {
                    TransactionPayment::weight_to_fee(weight)
                }

                fn query_length_to_fee(length: u32) -> Balance {
                    TransactionPayment::length_to_fee(length)
                }
            }

            #[cfg(feature = "parachain")]
            impl session_keys_primitives::VrfApi<Block> for Runtime {
                fn get_last_vrf_output() -> Option<<Block as BlockT>::Hash> {
                    None
                }
                fn vrf_key_lookup(
                    nimbus_id: nimbus_primitives::NimbusId
                ) -> Option<session_keys_primitives::VrfId> {
                    use session_keys_primitives::KeysLookup;
                    AuthorMapping::lookup_keys(&nimbus_id)
                }
            }

            impl sp_api::Core<Block> for Runtime {
                fn version() -> RuntimeVersion {
                    VERSION
                }

                fn execute_block(block: Block) {
                    Executive::execute_block(block);
                }

                fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
                    Executive::initialize_block(header)
                }
            }

            impl sp_api::Metadata<Block> for Runtime {
                fn metadata() -> OpaqueMetadata {
                    OpaqueMetadata::new(Runtime::metadata().into())
                }

                fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
                    Runtime::metadata_at_version(version)
                }

                fn metadata_versions() -> Vec<u32> {
                    Runtime::metadata_versions()
                }
            }

            impl sp_block_builder::BlockBuilder<Block> for Runtime {
                fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
                    Executive::apply_extrinsic(extrinsic)
                }

                fn finalize_block() -> <Block as BlockT>::Header {
                    Executive::finalize_block()
                }

                fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
                    data.create_extrinsics()
                }

                fn check_inherents(
                    block: Block,
                    data: sp_inherents::InherentData,
                ) -> sp_inherents::CheckInherentsResult {
                    data.check_extrinsics(&block)
                }
            }

            #[cfg(not(feature = "parachain"))]
            impl sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId> for Runtime {
                fn slot_duration() -> sp_consensus_aura::SlotDuration {
                    sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
                }

                fn authorities() -> Vec<sp_consensus_aura::sr25519::AuthorityId> {
                    pallet_aura::Authorities::<Runtime>::get().into_inner()
                }
            }

            #[cfg(not(feature = "parachain"))]
            impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
                fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
                    Grandpa::grandpa_authorities()
                }

                fn current_set_id() -> sp_consensus_grandpa::SetId {
                    Grandpa::current_set_id()
                }

                fn submit_report_equivocation_unsigned_extrinsic(
                    _equivocation_proof: sp_consensus_grandpa::EquivocationProof<
                        <Block as BlockT>::Hash,
                        sp_runtime::traits::NumberFor<Block>,
                    >,
                    _key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
                ) -> Option<()> {
                    None
                }

                fn generate_key_ownership_proof(
                    _set_id: sp_consensus_grandpa::SetId,
                    _authority_id: sp_consensus_grandpa::AuthorityId,
                ) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
                    // NOTE: this is the only implementation possible since we've
                    // defined our key owner proof type as a bottom type (i.e. a type
                    // with no values).
                    None
                }
            }

            impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
                fn offchain_worker(header: &<Block as BlockT>::Header) {
                    Executive::offchain_worker(header)
                }
            }

            impl sp_session::SessionKeys<Block> for Runtime {
                fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
                    opaque::SessionKeys::generate(seed)
                }

                fn decode_session_keys(
                    encoded: Vec<u8>,
                ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
                    opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
                }
            }

            #[cfg(not(feature = "disable-genesis-builder"))]
            impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
				fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
					frame_support::genesis_builder_helper::build_state::<RuntimeGenesisConfig>(config)
				}

				fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
					frame_support::genesis_builder_helper::get_preset::<RuntimeGenesisConfig>(id, get_genesis_config_preset)
				}

				fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
					get_genesis_config_preset_names()
				}
			}

            impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
                fn validate_transaction(
                    source: TransactionSource,
                    tx: <Block as BlockT>::Extrinsic,
                    block_hash: <Block as BlockT>::Hash,
                ) -> TransactionValidity {
                    // Filtered calls should not enter the tx pool as they'll fail if inserted.
                    // If this call is not allowed, we return early.
                    if !<Runtime as frame_system::Config>::BaseCallFilter::contains(&tx.function) {
                        return frame_support::pallet_prelude::InvalidTransaction::Call.into();
                    }

                    Executive::validate_transaction(source, tx, block_hash)
                }
            }

            impl zrml_swaps_runtime_api::SwapsApi<Block, PoolId, AccountId, Balance, MarketId>
            for Runtime
            {
                fn get_spot_price(
                    pool_id: &PoolId,
                    asset_in: &Asset<MarketId>,
                    asset_out: &Asset<MarketId>,
                    with_fees: bool,
                ) -> SerdeWrapper<Balance> {
                    SerdeWrapper(Swaps::get_spot_price(pool_id, asset_in, asset_out, with_fees).ok().unwrap_or(0))
                }

                fn pool_account_id(pool_id: &PoolId) -> AccountId {
                    Swaps::pool_account_id(pool_id)
                }

                fn pool_shares_id(pool_id: PoolId) -> Asset<SerdeWrapper<MarketId>> {
                    Asset::PoolShare(pool_id)
                }
            }

            #[cfg(feature = "try-runtime")]
            impl frame_try_runtime::TryRuntime<Block> for Runtime {
                fn on_runtime_upgrade(checks: UpgradeCheckSelect) -> (Weight, Weight) {
                    // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
                    // have a backtrace here. If any of the pre/post migration checks fail, we shall stop
                    // right here and right now.
                    let weight = Executive::try_runtime_upgrade(checks).unwrap();
                    (weight, RuntimeBlockWeights::get().max_block)
                }

                fn execute_block(
                    block: Block,
                    state_root_check: bool,
                    signature_check: bool,
                    select: TryStateSelect,
                ) -> Weight {
                    // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
                    // have a backtrace here.
                    Executive::try_execute_block(block, state_root_check, signature_check, select).unwrap()
                }
            }

            #[cfg(feature = "parachain")]
            impl async_backing_primitives::UnincludedSegmentApi<Block> for Runtime {
                fn can_build_upon(
                    _included_hash: <Block as BlockT>::Hash,
                    _slot: async_backing_primitives::Slot,
                ) -> bool {
                    // This runtime API can be called only when asynchronous backing is enabled client-side
                    // We return false here to force the client to not use async backing.
                    false
                }
            }

            $($additional_apis)*
        }

        // Check the timestamp and parachain inherents
        #[cfg(feature = "parachain")]
        struct CheckInherents;


        // Parity has decided to depreciate this trait, but does not offer a satisfactory replacement,
        // see issue: https://github.com/paritytech/polkadot-sdk/issues/2841
        #[allow(deprecated)]
        #[cfg(feature = "parachain")]
        impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
            fn check_inherents(
                block: &Block,
                relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
            ) -> sp_inherents::CheckInherentsResult {
                let relay_chain_slot = relay_state_proof
                    .read_slot()
                    .expect("Could not read the relay chain slot from the proof");

                let inherent_data =
                    cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
                        relay_chain_slot,
                        core::time::Duration::from_secs(6),
                    )
                    .create_inherent_data()
                    .expect("Could not create the timestamp inherent data");

                inherent_data.check_extrinsics(block)
            }
        }

        // Nimbus's Executive wrapper allows relay validators to verify the seal digest
        #[cfg(feature = "parachain")]
        cumulus_pallet_parachain_system::register_validate_block! {
            Runtime = Runtime,
            BlockExecutor = pallet_author_inherent::BlockExecutor::<Runtime, Executive>,
            CheckInherents = CheckInherents,
        }
    }
}

#[macro_export]
macro_rules! create_common_benchmark_logic {
    () => {
        #[cfg(feature = "runtime-benchmarks")]
        pub(crate) mod benchmarks {
            pub(crate) mod currencies {
                use super::utils::{lookup_of_account, set_balance};
                use crate::{
                    AccountId, Amount, AssetManager, Balance, CurrencyId, ExistentialDeposit,
                    GetNativeCurrencyId, Runtime,
                };
                use alloc::vec;
                use frame_benchmarking::{account, whitelisted_caller};
                use frame_system::RawOrigin;
                use orml_benchmarking::runtime_benchmarks;
                use orml_traits::MultiCurrency;
                use sp_runtime::traits::UniqueSaturatedInto;
                use zeitgeist_primitives::{constants::BASE, types::Asset};

                const SEED: u32 = 0;
                const NATIVE: CurrencyId = GetNativeCurrencyId::get();
                const ASSET: CurrencyId = Asset::CategoricalOutcome(0, 0);

                runtime_benchmarks! {
                    { Runtime, orml_currencies }

                    // `transfer` non-native currency
                    transfer_non_native_currency {
                        let amount: Balance = 1_000 * BASE;
                        let from: AccountId = whitelisted_caller();
                        set_balance(ASSET, &from, amount);

                        let to: AccountId = account("to", 0, SEED);
                        let to_lookup = lookup_of_account(to.clone());
                    }: transfer(RawOrigin::Signed(from), to_lookup, ASSET, amount)
                    verify {
                        assert_eq!(
                            <AssetManager as MultiCurrency<_>>::total_balance(ASSET, &to),
                            amount,
                        );
                    }

                    // `transfer` native currency and in worst case
                    #[extra]
                    transfer_native_currency_worst_case {
                        let existential_deposit = ExistentialDeposit::get();
                        let amount: Balance = existential_deposit.saturating_mul(1000);
                        let from: AccountId = whitelisted_caller();
                        set_balance(NATIVE, &from, amount);

                        let to: AccountId = account("to", 0, SEED);
                        let to_lookup = lookup_of_account(to.clone());
                    }: transfer(RawOrigin::Signed(from), to_lookup, NATIVE, amount)
                    verify {
                        assert_eq!(
                            <AssetManager as MultiCurrency<_>>::total_balance(NATIVE, &to),
                            amount,
                        );
                    }

                    // `transfer_native_currency` in worst case
                    // * will create the `to` account.
                    // * will kill the `from` account.
                    transfer_native_currency {
                        let existential_deposit = ExistentialDeposit::get();
                        let amount: Balance = existential_deposit.saturating_mul(1000);
                        let from: AccountId = whitelisted_caller();
                        set_balance(NATIVE, &from, amount);

                        let to: AccountId = account("to", 0, SEED);
                        let to_lookup = lookup_of_account(to.clone());
                    }: _(RawOrigin::Signed(from), to_lookup, amount)
                    verify {
                        assert_eq!(
                            <AssetManager as MultiCurrency<_>>::total_balance(NATIVE, &to),
                            amount,
                        );
                    }

                    // `update_balance` for non-native currency
                    update_balance_non_native_currency {
                        let balance: Balance = 2 * BASE;
                        let amount: Amount = balance.unique_saturated_into();
                        let who: AccountId = account("who", 0, SEED);
                        let who_lookup = lookup_of_account(who.clone());
                    }: update_balance(RawOrigin::Root, who_lookup, ASSET, amount)
                    verify {
                        assert_eq!(
                            <AssetManager as MultiCurrency<_>>::total_balance(ASSET, &who),
                            balance,
                        );
                    }

                    // `update_balance` for native currency
                    // * will create the `who` account.
                    update_balance_native_currency_creating {
                        let existential_deposit = ExistentialDeposit::get();
                        let balance: Balance = existential_deposit.saturating_mul(1000);
                        let amount: Amount = balance.unique_saturated_into();
                        let who: AccountId = account("who", 0, SEED);
                        let who_lookup = lookup_of_account(who.clone());
                    }: update_balance(RawOrigin::Root, who_lookup, NATIVE, amount)
                    verify {
                        assert_eq!(
                            <AssetManager as MultiCurrency<_>>::total_balance(NATIVE, &who),
                            balance,
                        );
                    }

                    // `update_balance` for native currency
                    // * will kill the `who` account.
                    update_balance_native_currency_killing {
                        let existential_deposit = ExistentialDeposit::get();
                        let balance: Balance = existential_deposit.saturating_mul(1000);
                        let amount: Amount = balance.unique_saturated_into();
                        let who: AccountId = account("who", 0, SEED);
                        let who_lookup = lookup_of_account(who.clone());
                        set_balance(NATIVE, &who, balance);
                    }: update_balance(RawOrigin::Root, who_lookup, NATIVE, -amount)
                    verify {
                        assert_eq!(
                            <AssetManager as MultiCurrency<_>>::free_balance(NATIVE, &who),
                            0,
                        );
                    }
                }

                #[cfg(test)]
                mod tests {
                    use super::*;
                    use crate::benchmarks::utils::tests::new_test_ext;
                    use orml_benchmarking::impl_benchmark_test_suite;

                    impl_benchmark_test_suite!(new_test_ext(),);
                }
            }

            pub(crate) mod tokens {
                use super::utils::{lookup_of_account, set_balance as update_balance};
                use crate::{AccountId, Balance, CurrencyId, Runtime, Tokens};
                use alloc::vec;
                use frame_benchmarking::{account, whitelisted_caller};
                use frame_system::RawOrigin;
                use orml_benchmarking::runtime_benchmarks;
                use orml_traits::MultiCurrency;
                use zeitgeist_primitives::{constants::BASE, types::Asset};

                const SEED: u32 = 0;
                const ASSET: CurrencyId = Asset::CategoricalOutcome(0, 0);

                runtime_benchmarks! {
                    { Runtime, orml_tokens }

                    transfer {
                        let amount: Balance = BASE;

                        let from: AccountId = whitelisted_caller();
                        update_balance(ASSET, &from, amount);

                        let to: AccountId = account("to", 0, SEED);
                        let to_lookup = lookup_of_account(to.clone());
                    }: _(RawOrigin::Signed(from), to_lookup, ASSET, amount)
                    verify {
                        assert_eq!(<Tokens as MultiCurrency<_>>::total_balance(ASSET, &to), amount);
                    }

                    transfer_all {
                        let amount: Balance = BASE;

                        let from: AccountId = whitelisted_caller();
                        update_balance(ASSET, &from, amount);

                        let to: AccountId = account("to", 0, SEED);
                        let to_lookup = lookup_of_account(to);
                    }: _(RawOrigin::Signed(from.clone()), to_lookup, ASSET, false)
                    verify {
                        assert_eq!(<Tokens as MultiCurrency<_>>::total_balance(ASSET, &from), 0);
                    }

                    transfer_keep_alive {
                        let from: AccountId = whitelisted_caller();
                        update_balance(ASSET, &from, 2 * BASE);

                        let to: AccountId = account("to", 0, SEED);
                        let to_lookup = lookup_of_account(to.clone());
                    }: _(RawOrigin::Signed(from), to_lookup, ASSET, BASE)
                    verify {
                        assert_eq!(<Tokens as MultiCurrency<_>>::total_balance(ASSET, &to), BASE);
                    }

                    force_transfer {
                        let from: AccountId = account("from", 0, SEED);
                        let from_lookup = lookup_of_account(from.clone());
                        update_balance(ASSET, &from, 2 * BASE);

                        let to: AccountId = account("to", 0, SEED);
                        let to_lookup = lookup_of_account(to.clone());
                    }: _(RawOrigin::Root, from_lookup, to_lookup, ASSET, BASE)
                    verify {
                        assert_eq!(<Tokens as MultiCurrency<_>>::total_balance(ASSET, &to), BASE);
                    }

                    set_balance {
                        let who: AccountId = account("who", 0, SEED);
                        let who_lookup = lookup_of_account(who.clone());

                    }: _(RawOrigin::Root, who_lookup, ASSET, BASE, BASE)
                    verify {
                        assert_eq!(
                            <Tokens as MultiCurrency<_>>::total_balance(ASSET, &who),
                            2 * BASE,
                        );
                    }
                }

                #[cfg(test)]
                mod tests {
                    use super::*;
                    use crate::benchmarks::utils::tests::new_test_ext;
                    use orml_benchmarking::impl_benchmark_test_suite;

                    impl_benchmark_test_suite!(new_test_ext(),);
                }
            }

            pub(crate) mod utils {
                use crate::{AccountId, AssetManager, Balance, CurrencyId, Runtime};
                use frame_support::assert_ok;
                use orml_traits::MultiCurrencyExtended;
                use sp_runtime::traits::{SaturatedConversion, StaticLookup};

                pub fn lookup_of_account(
                    who: AccountId,
                ) -> <<Runtime as frame_system::Config>::Lookup as StaticLookup>::Source {
                    <Runtime as frame_system::Config>::Lookup::unlookup(who)
                }

                pub fn set_balance(currency_id: CurrencyId, who: &AccountId, balance: Balance) {
                    assert_ok!(<AssetManager as MultiCurrencyExtended<_>>::update_balance(
                        currency_id,
                        who,
                        balance.saturated_into()
                    ));
                }

                #[cfg(test)]
                pub mod tests {
                    use crate::Runtime;
                    use sp_runtime::BuildStorage;

                    pub fn new_test_ext() -> sp_io::TestExternalities {
                        frame_system::GenesisConfig::<Runtime>::default()
                            .build_storage()
                            .unwrap()
                            .into()
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! create_common_tests {
    () => {
        #[cfg(test)]
        mod common_tests {
            common_runtime::fee_tests!();

            mod utility {
                use crate::{Balances, BlockNumber, Futarchy, Preimage, Scheduler, System};
                use frame_support::traits::Hooks;

                // Beware! This only advances certain pallets.
                pub(crate) fn run_to_block(to: BlockNumber) {
                    while System::block_number() < to {
                        let now = System::block_number();

                        Futarchy::on_finalize(now);
                        Balances::on_finalize(now);
                        Preimage::on_finalize(now);
                        Scheduler::on_finalize(now);
                        System::on_finalize(now);

                        let next = now + 1;
                        System::set_block_number(next);

                        System::on_initialize(next);
                        Scheduler::on_initialize(next);
                        Preimage::on_initialize(next);
                        Balances::on_initialize(next);
                        Futarchy::on_initialize(next);
                    }
                }
            }

            mod dust_removal {
                use crate::*;
                use frame_support::PalletId;
                use sp_runtime::BuildStorage;
                use test_case::test_case;

                #[test_case(AuthorizedPalletId::get(); "authorized")]
                #[test_case(CourtPalletId::get(); "court")]
                #[test_case(PmPalletId::get(); "prediction_markets")]
                #[test_case(SwapsPalletId::get(); "swaps")]
                #[test_case(TreasuryPalletId::get(); "treasury")]
                fn whitelisted_pallet_accounts_dont_get_reaped(pallet_id: PalletId) {
                    let mut t: sp_io::TestExternalities =
                        frame_system::GenesisConfig::<Runtime>::default()
                            .build_storage()
                            .unwrap()
                            .into();
                    t.execute_with(|| {
                        let pallet_main_account: AccountId = pallet_id.into_account_truncating();
                        let pallet_sub_account: AccountId =
                            pallet_id.into_sub_account_truncating(42);
                        assert!(DustRemovalWhitelist::contains(&pallet_main_account));
                        assert!(DustRemovalWhitelist::contains(&pallet_sub_account));
                    });
                }

                #[test]
                fn non_whitelisted_accounts_get_reaped() {
                    let mut t: sp_io::TestExternalities =
                        frame_system::GenesisConfig::<Runtime>::default()
                            .build_storage()
                            .unwrap()
                            .into();
                    t.execute_with(|| {
                        let not_whitelisted = AccountId::from([0u8; 32]);
                        assert!(!DustRemovalWhitelist::contains(&not_whitelisted))
                    });
                }
            }

            mod futarchy {
                use crate::{
                    common_tests::utility, AccountId, Asset, AssetManager, Balance, Balances,
                    Futarchy, MarketId, NeoSwaps, PredictionMarkets, Preimage, Runtime,
                    RuntimeCall, RuntimeOrigin, Scheduler, System,
                };
                use frame_support::{assert_ok, dispatch::RawOrigin, traits::StorePreimage};
                use orml_traits::MultiCurrency;
                use sp_runtime::{
                    traits::{Hash, Zero},
                    BuildStorage, Perbill,
                };
                use zeitgeist_primitives::{
                    math::fixed::{BaseProvider, ZeitgeistBase},
                    traits::MarketBuilderTrait,
                    types::{
                        Deadlines, MarketCreation, MarketPeriod, MarketType, MultiHash, ScoringRule,
                    },
                };
                use zrml_futarchy::types::Proposal;
                use zrml_market_commons::types::MarketBuilder;
                use zrml_neo_swaps::types::{DecisionMarketOracle, DecisionMarketOracleScoreboard};

                #[test]
                fn futarchy_schedules_and_executes_call() {
                    let mut t: sp_io::TestExternalities =
                        frame_system::GenesisConfig::<Runtime>::default()
                            .build_storage()
                            .unwrap()
                            .into();
                    t.execute_with(|| {
                        let alice = AccountId::from([0u8; 32]);

                        let collateral: Asset<MarketId> = Asset::Ztg;
                        let one: Balance = ZeitgeistBase::get().unwrap();
                        let total_cost: Balance = one.saturating_mul(100_000u128);
                        assert_ok!(AssetManager::deposit(collateral, &alice, total_cost));

                        let mut metadata = [0x01; 50];
                        metadata[0] = 0x15;
                        metadata[1] = 0x30;
                        let multihash = MultiHash::Sha3_384(metadata);

                        let oracle_duration =
                            <Runtime as zrml_prediction_markets::Config>::MinOracleDuration::get();
                        let deadlines = Deadlines {
                            grace_period: Default::default(),
                            oracle_duration,
                            dispute_duration: Zero::zero(),
                        };
                        assert_ok!(PredictionMarkets::create_market(
                            RuntimeOrigin::signed(alice.clone()),
                            collateral,
                            Perbill::zero(),
                            alice.clone(),
                            MarketPeriod::Block(0..999),
                            deadlines,
                            multihash,
                            MarketCreation::Permissionless,
                            MarketType::Categorical(2),
                            None,
                            ScoringRule::AmmCdaHybrid,
                        ));

                        let market_id = 0;
                        let amount = one * 100u128;
                        assert_ok!(PredictionMarkets::buy_complete_set(
                            RuntimeOrigin::signed(alice.clone()),
                            market_id,
                            amount,
                        ));

                        assert_ok!(NeoSwaps::deploy_pool(
                            RuntimeOrigin::signed(alice.clone()),
                            market_id,
                            amount,
                            vec![one / 10u128 * 9u128, one / 10u128],
                            one / 100,
                        ));

                        let duration = <Runtime as zrml_futarchy::Config>::MinDuration::get();

                        // Wrap `remark_with_event` call in `dispatch_as` so that it doesn't error
                        // with `BadOrigin`.
                        let bob = AccountId::from([0x01; 32]);
                        let remark = b"hullo".to_vec();
                        let remark_dispatched_as = pallet_utility::Call::<Runtime>::dispatch_as {
                            as_origin: Box::new(RawOrigin::Signed(bob.clone()).into()),
                            call: Box::new(
                                frame_system::Call::remark_with_event { remark: remark.clone() }
                                    .into(),
                            ),
                        };
                        let call =
                            Preimage::bound(RuntimeCall::from(remark_dispatched_as)).unwrap();
                        let scoreboard =
                            DecisionMarketOracleScoreboard::new(40_000, 10_000, one / 7, one);
                        let oracle = DecisionMarketOracle::new(
                            market_id,
                            Asset::CategoricalOutcome(market_id, 0),
                            Asset::CategoricalOutcome(market_id, 1),
                            scoreboard,
                        );
                        let when = duration + 10;
                        let proposal = Proposal { when, call, oracle };

                        assert_ok!(Futarchy::submit_proposal(
                            RawOrigin::Root.into(),
                            duration,
                            proposal.clone()
                        ));

                        utility::run_to_block(when);

                        let hash = <Runtime as frame_system::Config>::Hashing::hash(&remark);
                        System::assert_has_event(
                            frame_system::Event::<Runtime>::Remarked { sender: bob, hash }.into(),
                        );
                        System::assert_has_event(
                            pallet_scheduler::Event::<Runtime>::Dispatched {
                                task: (when, 0),
                                id: None,
                                result: Ok(()),
                            }
                            .into(),
                        );
                    });
                }
            }
        }
    };
}
