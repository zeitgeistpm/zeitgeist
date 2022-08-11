#![allow(
    // Constants parameters inside `parameter_types!` already check
    // arithmetic operations at compile time
    clippy::integer_arithmetic
)]

use super::VERSION;
use frame_support::{
    parameter_types,
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, WEIGHT_PER_SECOND},
        DispatchClass, Weight,
    },
    PalletId,
};
use frame_system::limits::{BlockLength, BlockWeights};
use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use sp_runtime::{traits::AccountIdConversion, FixedPointNumber, Perbill, Permill, Perquintill};
use sp_version::RuntimeVersion;
use zeitgeist_primitives::{constants::*, types::*};

pub(crate) const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
pub(crate) const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;
pub(crate) const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);


parameter_types! {
    // Authorized
    pub const AuthorizedPalletId: PalletId = PalletId(*b"zge/atzd");

    // Authority
    pub const MaxAuthorities: u32 = 32;

    // Balance
    pub const ExistentialDeposit: u128 = CENT;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;

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

    // Court
    pub const CourtCaseDuration: u64 = BLOCKS_PER_DAY;
    pub const CourtPalletId: PalletId = PalletId(*b"zge/cout");
    pub const StakeWeight: u128 = 2 * BASE;

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

    // Liquidity Mining parameters
    pub const LiquidityMiningPalletId: PalletId = PalletId(*b"zge/lymg");

    // Multisig
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);

    // ORML
    pub const GetNativeCurrencyId: CurrencyId = Asset::Ztg;
    pub DustAccount: AccountId = PalletId(*b"orml/dst").into_account();

    // Prediction Market parameters
    pub const AdvisoryBond: Balance = 25 * CENT;
    pub const DisputeBond: Balance = 5 * BASE;
    pub const DisputeFactor: Balance = 2 * BASE;
    pub const DisputePeriod: BlockNumber = BLOCKS_PER_DAY;
    pub const MaxCategories: u16 = 10;
    pub const MaxDisputes: u16 = 6;
    pub const MinCategories: u16 = 2;
    // 60_000 = 1 minute. Should be raised to something more reasonable in the future.
    pub const MinSubsidyPeriod: Moment = 60_000;
    // 2_678_400_000 = 31 days.
    pub const MaxSubsidyPeriod: Moment = 2_678_400_000;
    // Requirements: MaxPeriod + ReportingPeriod + MaxDisputes * DisputePeriod < u64::MAX.
    pub const MaxMarketPeriod: Moment = u64::MAX / 2;
    pub const OracleBond: Balance = 50 * CENT;
    pub const PmPalletId: PalletId = PalletId(*b"zge/pred");
    pub const ReportingPeriod: u32 = BLOCKS_PER_DAY as _;
    pub const ValidityBond: Balance = 50 * CENT;

    // Preimage
    pub const PreimageMaxSize: u32 = 4096 * 1024;
    pub PreimageBaseDeposit: Balance = deposit(2, 64);
    pub PreimageByteDeposit: Balance = deposit(0, 1);

    // Proxy
    // One storage item; key size 32, value size 8; .
    pub const ProxyDepositBase: Balance = deposit(1, 8);
    // Additional storage item size of 33 bytes.
    pub const ProxyDepositFactor: Balance = deposit(0, 33);
    pub const AnnouncementDepositBase: Balance = deposit(1, 8);
    pub const AnnouncementDepositFactor: Balance = deposit(0, 66);

    // Scheduler
    pub MaximumSchedulerWeight: Weight =
        Perbill::from_percent(10) * RuntimeBlockWeights::get().max_block;
    // No hard limit, used for worst-case weight estimation
    pub const MaxScheduledPerBlock: u32 = 50;
    pub const NoPreimagePostponement: Option<u64> = Some(5 * BLOCKS_PER_MINUTE);

    // Simple disputes parameters
    pub const SimpleDisputesPalletId: PalletId = PalletId(*b"zge/sedp");

    // Swaps parameters
    pub const ExitFee: Balance = 3 * BASE / 1000; // 0.3%
    pub const MinAssets: u16 = 2;
    pub const MaxAssets: u16 = MaxCategories::get() + 1;
    pub const MaxInRatio: Balance = (BASE / 3) + 1;
    pub const MaxOutRatio: Balance = (BASE / 3) + 1;
    pub const MaxSwapFee: Balance = BASE / 10; // 10%
    pub const MaxTotalWeight: Balance = 50 * BASE;
    pub const MaxWeight: Balance = 50 * BASE;
    pub const MinLiquidity: Balance = 100 * BASE;
    pub const MinSubsidy: Balance = MinLiquidity::get();
    pub const MinSubsidyPerAccount: Balance = MinSubsidy::get();
    pub const MinWeight: Balance = BASE;
    pub const SwapsPalletId: PalletId = PalletId(*b"zge/swap");

    // System
    pub const BlockHashCount: u64 = 250;
    pub const SS58Prefix: u8 = 73;
    pub const Version: RuntimeVersion = VERSION;
    pub RuntimeBlockLength: BlockLength = BlockLength::max_with_normal_ratio(
        5 * 1024 * 1024,
        NORMAL_DISPATCH_RATIO,
    );
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

    // Timestamp
    pub const MinimumPeriod: u64 = MILLISECS_PER_BLOCK as u64 / 2;

    // Transaction payment
    pub const OperationalFeeMultiplier: u8 = 5;
    pub const TransactionByteFee: Balance = 100 * MICRO;
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(10);
    // With a target block time of 12 seconds (7200 blocks per day)
    // the weight fees can increase by at most ~21.46% per day, given extreme congestion.
    // See https://paritytech.github.io/substrate/master/pallet_transaction_payment/struct.TargetedFeeAdjustment.html for details.
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
    // Minimum amount of the multiplier. The test `multiplier_can_grow_from_zero` ensures
    // that this value is not too low.
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

parameter_type_with_key! {
    // Well, not every asset is a currency ¯\_(ツ)_/¯
    pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
        match currency_id {
            Asset::Ztg => ExistentialDeposit::get(),
            _ => 0
        }
    };
}

// Parameterized slow adjusting fee updated based on
// https://research.web3.foundation/en/latest/polkadot/overview/2-token-economics.html#-2.-slow-adjusting-mechanism
pub type SlowAdjustingFeeUpdate<R> =
    TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
