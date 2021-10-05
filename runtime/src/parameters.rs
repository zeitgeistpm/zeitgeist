#![allow(
  // Constants parameters inside `parameter_types!` already check
  // arithmetic operations at compile time
  clippy::integer_arithmetic
)]

use crate::{AccountId, Balances, Origin, ParachainInfo, ParachainSystem, XcmpQueue, VERSION};
use frame_support::{
    match_type, parameter_types,
    traits::Everything,
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, WEIGHT_PER_SECOND},
        DispatchClass, Weight,
    },
};
use frame_system::limits::{BlockLength, BlockWeights};
use polkadot_parachain::primitives::Sibling;
use sp_runtime::{Perbill, Percent, SaturatedConversion};
use sp_version::RuntimeVersion;
use xcm::latest::{BodyId, Junction, Junctions, MultiLocation, NetworkId};
use xcm_builder::{
    AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, CurrencyAdapter,
    IsConcrete, ParentAsSuperuser, ParentIsDefault, RelayChainAsNative, SiblingParachainAsNative,
    SiblingParachainConvertsVia, SignedAccountId32AsNative, SovereignSignedViaLocation,
    TakeWeightCredit,
};
use zeitgeist_primitives::{
    constants::{MICRO, *},
    types::{Balance, *},
};

pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

pub type Barrier = (
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<Everything>,
    AllowUnpaidExecutionFrom<ParentOrParentsUnitPlurality>,
);
pub type LocalAssetTransactor =
    CurrencyAdapter<Balances, IsConcrete<RelayChainLocation>, LocationToAccountId, AccountId, ()>;
pub type LocalOriginToLocation = ();
pub type LocationToAccountId = (
    ParentIsDefault<AccountId>,
    SiblingParachainConvertsVia<Sibling, AccountId>,
    AccountId32Aliases<RelayChainNetwork, AccountId>,
);
pub type XcmOriginToTransactDispatchOrigin = (
    SovereignSignedViaLocation<LocationToAccountId, Origin>,
    RelayChainAsNative<RelayChainOrigin, Origin>,
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    ParentAsSuperuser<Origin>,
    SignedAccountId32AsNative<RelayChainNetwork, Origin>,
);
pub type XcmRouter = (cumulus_primitives_utility::ParentAsUmp<ParachainSystem, ()>, XcmpQueue);

match_type! {
  pub type ParentOrParentsUnitPlurality: impl Contains<MultiLocation> = {
    MultiLocation { parents: 1, interior: Junctions::Here } |
    MultiLocation { parents: 1, interior: Junctions::X1(Junction::Plurality { id: BodyId::Unit, .. }) }
  };
}

parameter_types! {
  // Collective
  pub const AdvisoryCommitteeMaxMembers: u32 = 100;
  pub const AdvisoryCommitteeMaxProposals: u32 = 64;
  pub const AdvisoryCommitteeMotionDuration: BlockNumber = 7 * BLOCKS_PER_DAY;

  // Crowdloan
  pub const InitializationPayment: Perbill = Perbill::from_percent(30);
  pub const Initialized: bool = false;
  pub const MaxInitContributorsBatchSizes: u32 = 500;
  pub const MinimumReward: Balance = 0;
  pub const RelaySignaturesThreshold: Perbill = Perbill::from_percent(100);

  // Cumulus and Polkadot
  pub Ancestry: MultiLocation = Junction::Parachain(ParachainInfo::parachain_id().into()).into();
  pub const RelayChainLocation: MultiLocation = MultiLocation::parent();
  pub const RelayChainNetwork: NetworkId = NetworkId::Kusama;
  pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
  pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
  pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
  pub UnitWeightCost: Weight = MICRO.saturated_into();

  // Identity
  pub const BasicDeposit: Balance = 8 * BASE;
  pub const FieldDeposit: Balance = 256 * CENT;
  pub const MaxAdditionalFields: u32 = 64;
  pub const MaxRegistrars: u32 = 8;
  pub const MaxSubAccounts: u32 = 64;
  pub const SubAccountDeposit: Balance = 2 * BASE;

  // Staking
  pub const CollatorDeposit: Balance = 2 * BASE;
  pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
  pub const DefaultParachainBondReservePercent: Percent = Percent::from_percent(30);
  pub const LeaveCandidatesDelay: u32 = 2;
  pub const LeaveNominatorsDelay: u32 = 2;
  pub const MaxCollatorsPerNominator: u32 = 16;
  pub const MaxNominatorsPerCollator: u32 = 32;
  pub const MinBlocksPerRound: u32 = (BLOCKS_PER_DAY / 6) as _;
  pub const MinCollatorStake: u128 = 64 * BASE;
  pub const MinNominatorStake: u128 = BASE / 2;
  pub const MinSelectedCandidates: u32 = 1;
  pub const RevokeNominationDelay: u32 = 2;
  pub const RewardPaymentDelay: u32 = 2;

  // System
  pub const Version: RuntimeVersion = VERSION;
  pub RuntimeBlockLength: BlockLength = BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
  pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
    .base_block(BlockExecutionWeight::get())
    .for_class(DispatchClass::all(), |weights| {
      weights.base_extrinsic = ExtrinsicBaseWeight::get();
    })
    .for_class(DispatchClass::Normal, |weights| {
      weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
    })
    .for_class(DispatchClass::Operational, |weights| {
      weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
      weights.reserved = Some(
        MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
      );
    })
    .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
    .build_or_panic();

  // Transaction payment
  pub const TransactionByteFee: Balance = 100 * MICRO;
}
