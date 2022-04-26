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
use pallet_transaction_payment::Multiplier;
use sp_runtime::{traits::AccountIdConversion, FixedPointNumber, Perbill, Permill, Perquintill};
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
    /// How often (in blocks) new public referenda are launched.
    pub const LaunchPeriod: BlockNumber = 5 * BLOCKS_PER_DAY;
    /// How often (in blocks) to check for new votes.
    pub const VotingPeriod: BlockNumber = 5 * BLOCKS_PER_DAY;
    /// Minimum voting period allowed for a fast-track referendum.
    pub const FastTrackVotingPeriod: BlockNumber = 3 * BLOCKS_PER_HOUR;
    /// The minimum amount to be used as a deposit for a public referendum proposal.
    pub const MinimumDeposit: Balance = 100 * BASE;
    /// The period between a proposal being approved and enacted.
    /// It should generally be a little more than the unstake period to ensure that voting stakers
    /// have an opportunity to remove themselves from the system in the case where they are on the
    /// losing side of a vote.
    pub const EnactmentPeriod: BlockNumber = 2 * BLOCKS_PER_DAY;
    /// The minimum period of vote locking.
    /// It should be no shorter than enactment period to ensure that in the case of an approval,
    /// those successful voters are locked into the consequences that their votes entail.
    pub const VoteLockingPeriod: BlockNumber = VotingPeriod::get() + EnactmentPeriod::get();
    /// Period in blocks where an external proposal may not be re-submitted after being vetoed.
    pub const CooloffPeriod: BlockNumber = 7 * BLOCKS_PER_DAY;
    /// Indicator for whether an emergency origin is even allowed to happen.
    pub const InstantAllowed: bool = true;
    /// The maximum number of votes for an account. Also used to compute weight, an overly big value
    /// can lead to extrinsic with very big weight: see delegate for instance.
    pub const MaxVotes: u32 = 100;
    /// The maximum number of public proposals that can exist at any time.
    pub const MaxProposals: u32 = 100;

    // Identity
    pub const BasicDeposit: Balance = 8 * BASE;
    pub const FieldDeposit: Balance = 256 * CENT;
    pub const MaxAdditionalFields: u32 = 64;
    pub const MaxRegistrars: u32 = 8;
    pub const MaxSubAccounts: u32 = 64;
    pub const SubAccountDeposit: Balance = 2 * BASE;

    // Multisig
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);

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
    // The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
    // than this will decrease the weight and more will increase.
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
    // * https://paritytech.github.io/substrate/master/pallet_transaction_payment/struct.TargetedFeeAdjustment.html
    // * With a target block time of 12 seconds (7200 blocks per day) 
    // * where p is the amount of change over 7200 blocks.
    // * p >= AdjustmentVariable * BlocksPerDay * (TargetBlockFullness * NORMAL_DISPATCH_RATIO)
    // * p >= 0.00003 * 7200 * (1 - 0.25 * 0.75)
    // * p >= 0.1755
    // * Meaning that fees can change by around ~17.55% per day, given extreme congestion.
    // The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
    // change the fees more rapidly.
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
    // Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
    // that combined with `AdjustmentVariable`, we can recover from the minimum.
    // See `multiplier_can_grow_from_zero`.
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000u128);

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
