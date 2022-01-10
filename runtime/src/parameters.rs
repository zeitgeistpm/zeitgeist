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
};
use frame_system::limits::{BlockLength, BlockWeights};
use sp_runtime::{Perbill, Percent};
use sp_version::RuntimeVersion;
use zeitgeist_primitives::{constants::*, types::*};

pub(crate) const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
pub(crate) const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;
pub(crate) const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
  // Authority
  pub const MaxAuthorities: u32 = 32;

  // Collective
  pub const AdvisoryCommitteeMaxMembers: u32 = 100;
  pub const AdvisoryCommitteeMaxProposals: u32 = 64;
  pub const AdvisoryCommitteeMotionDuration: BlockNumber = 7 * BLOCKS_PER_DAY;

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

  // XCM
  pub const MaxInstructions: u32 = 100;
}
