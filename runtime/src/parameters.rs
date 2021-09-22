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
use orml_traits::parameter_type_with_key;
use sp_runtime::{Perbill, Percent};
use sp_version::RuntimeVersion;
use zeitgeist_primitives::{constants::*, types::*};

pub(crate) const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
pub(crate) const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;
pub(crate) const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
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
  pub const SS58Prefix: u8 = 73;
  pub const TransactionByteFee: Balance = 100 * MICRO;
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
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
      Default::default()
    };
}
