#![allow(
  // Constants parameters inside `parameter_types!` already check
  // arithmetic operations at compile time
  clippy::integer_arithmetic
)]

use crate::VERSION;
use frame_support::{
    parameter_types,
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, WEIGHT_PER_SECOND},
        DispatchClass, Weight,
    },
    PalletId,
};
use frame_system::limits::{BlockLength, BlockWeights};
use sp_runtime::{traits::AccountIdConversion, Perbill, Permill};
use sp_version::RuntimeVersion;
use zeitgeist_primitives::{constants::*, types::*};

pub(crate) const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
pub(crate) const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;
pub(crate) const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
  // Authority
  pub const MaxAuthorities: u32 = 32;

  // Collective
  // Note: MaxMembers does not influence the pallet logic, but the worst-case weight estimation.
  pub const AdvisoryCommitteeMaxMembers: u32 = 100;
  pub const AdvisoryCommitteeMaxProposals: u32 = 300;
  pub const AdvisoryCommitteeMotionDuration: BlockNumber = 3 * BLOCKS_PER_DAY;
  pub const CouncilMaxMembers: u32 = 100;
  pub const CouncilMaxProposals: u32 = 100;
  pub const CouncilMotionDuration: BlockNumber = 7 * BLOCKS_PER_DAY;
  pub const TechnicalCommitteeMaxMembers: u32 = 100;
  pub const TechnicalCommitteeMaxProposals: u32 = 64;
  pub const TechnicalCommitteeMotionDuration: BlockNumber = 7 * BLOCKS_PER_DAY;

  // Democracy
  pub const LaunchPeriod: BlockNumber = 5 * BLOCKS_PER_DAY;
    pub const VotingPeriod: BlockNumber = 5 * BLOCKS_PER_DAY;
    pub const FastTrackVotingPeriod: BlockNumber = 3 * BLOCKS_PER_HOUR;
    pub const MinimumDeposit: Balance = 100 * BASE;
    pub const EnactmentPeriod: BlockNumber = 2 * BLOCKS_PER_DAY;
    pub const VoteLockingPeriod: BlockNumber = 7 * BLOCKS_PER_DAY;
    pub const CooloffPeriod: BlockNumber = 7 * BLOCKS_PER_DAY;
    pub const InstantAllowed: bool = true;
    pub const MaxVotes: u32 = 100;
    pub const MaxProposals: u32 = 100;

  // Identity
  pub const BasicDeposit: Balance = 8 * BASE;
  pub const FieldDeposit: Balance = 256 * CENT;
  pub const MaxAdditionalFields: u32 = 64;
  pub const MaxRegistrars: u32 = 8;
  pub const MaxSubAccounts: u32 = 64;
  pub const SubAccountDeposit: Balance = 2 * BASE;

  // ORML
  pub DustAccount: AccountId = PalletId(*b"orml/dst").into_account();

  // Preimage
  pub const PreimageMaxSize: u32 = 4096 * 1024;
    pub PreimageBaseDeposit: Balance = deposit(2, 64);
    pub PreimageByteDeposit: Balance = deposit(0, 1);

  // Scheduler
  pub MaximumSchedulerWeight: Weight = Perbill::from_percent(10) * RuntimeBlockWeights::get().max_block;
  // No hard limit, used for worst-case weight estimation
    pub const MaxScheduledPerBlock: u32 = 50;
    pub const NoPreimagePostponement: Option<u64> = Some(5 * BLOCKS_PER_MINUTE);

  // System
  pub const SS58Prefix: u8 = 73;
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
  pub const OperationalFeeMultiplier: u8 = 5;
  pub const TransactionByteFee: Balance = 100 * MICRO;

  // Treasury
  pub const Burn: Permill = Permill::from_percent(50);
  pub const MaxApprovals: u32 = 100;
  pub const ProposalBond: Permill = Permill::from_percent(5);
  pub const ProposalBondMinimum: Balance = 10 * BASE;
  pub const ProposalBondMaximum: Balance = 500 * BASE;
  pub const SpendPeriod: BlockNumber = 24 * BLOCKS_PER_DAY;
  pub const TreasuryPalletId: PalletId = PalletId(*b"zge/tsry");

  // Vesting
  pub const MinVestedTransfer: Balance = CENT;
}
