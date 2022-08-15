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
use orml_traits::parameter_type_with_key;
use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use sp_runtime::{traits::AccountIdConversion, FixedPointNumber, Perbill, Permill, Perquintill};
use sp_version::RuntimeVersion;
use zeitgeist_primitives::{constants::*, types::*};

pub(crate) const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
pub(crate) const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;
pub(crate) const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
    // Authorized
    pub const AuthorizedPalletId: PalletId = AUTHORIZED_PALLET_ID;

    // Authority
    pub const MaxAuthorities: u32 = 32;

    // Balance
    pub const ExistentialDeposit: u128 = 5 * CENT;
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
    /// Duration of a single court case.
    pub const CourtCaseDuration: u64 = BLOCKS_PER_DAY;
    /// Pallet identifier, mainly used for named balance reserves.
    pub const CourtPalletId: PalletId = COURT_PALLET_ID;
    /// This value is multiplied by the current number of jurors to determine the stake
    /// the juror has to pay.
    pub const StakeWeight: u128 = 2 * BASE;

    // Global Disputes
    /// Pallet identifier
    pub const GlobalDisputesPalletId: PalletId = GLOBAL_DISPUTES_PALLET_ID;
    /// Vote lock identifier, mainly used for the LockableCurrency on the native token.
    pub const VoteLockIdentifier: LockIdentifier = *b"zge/vote";
    /// The minimum required amount to vote on an outcome.
    pub const MinOutcomeVoteAmount: Balance = 10 * CENT;
    /// The maximum number of voting outcomes.
    pub const MaxOutcomeLimit: u32 = u32::MAX;
    /// The minimum number of voting outcomes.
    pub const MinOutcomes: u32 = 2;
    /// The fee required to add a voting outcome.
    pub const VotingOutcomeFee: Balance = 100 * CENT;
    /// The remove limit for the Outcomes storage double map.
    pub const RemoveKeysLimit: u32 = 250;

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
    /// The amount held on deposit for a registered identity
    pub const BasicDeposit: Balance = 8 * BASE;
    /// The amount held on deposit per additional field for a registered identity.
    pub const FieldDeposit: Balance = 256 * CENT;
    /// Maximum number of additional fields that may be stored in an ID. Needed to bound the I/O
    /// required to access an identity, but can be pretty high.
    pub const MaxAdditionalFields: u32 = 64;
    /// Maxmimum number of registrars allowed in the system. Needed to bound the complexity
    /// of, e.g., updating judgements.
    pub const MaxRegistrars: u32 = 8;
    /// The maximum number of sub-accounts allowed per identified account.
    pub const MaxSubAccounts: u32 = 64;
    /// The amount held on deposit for a registered subaccount. This should account for the fact
    /// that one storage item's value will increase by the size of an account ID, and there will
    /// be another trie item whose value is the size of an account ID plus 32 bytes.
    pub const SubAccountDeposit: Balance = 2 * BASE;

    // Liquidity Mining parameters
    /// Pallet identifier, mainly used for named balance reserves.
    pub const LiquidityMiningPalletId: PalletId = LM_PALLET_ID;

    // Multisig
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);

    // ORML
    pub const GetNativeCurrencyId: CurrencyId = Asset::Ztg;
    pub DustAccount: AccountId = PalletId(*b"orml/dst").into_account();

    // Prediction Market parameters
    /// (Slashable) Bond that is provided for creating an advised market that needs approval.
    /// Slashed in case the market is rejected.
    pub const AdvisoryBond: Balance = 25 * CENT;
    /// (Slashable) Bond that is provided for disputing the outcome.
    /// Slashed in case the final outcome does not match the dispute for which the `DisputeBond`
    /// was deposited.
    pub const DisputeBond: Balance = 5 * BASE;
    /// `DisputeBond` is increased by this factor after every dispute.
    pub const DisputeFactor: Balance = 2 * BASE;
    /// After reporting the outcome and after every dispute, the dispute period is extended
    /// by `DisputePeriod`.
    pub const DisputePeriod: BlockNumber = BLOCKS_PER_DAY;
    /// The period for a global dispute to end.
    pub const GlobalDisputesPeriod: BlockNumber = 7 * BLOCKS_PER_DAY;
    /// Maximum Categories a prediciton market can have (excluding base asset).
    pub const MaxCategories: u16 = MAX_CATEGORIES;
    /// Maximum number of disputes.
    pub const MaxDisputes: u16 = 6;
    /// Minimum number of categories. The trivial minimum is 2, which represents a binary market.
    pub const MinCategories: u16 = 2;
    // 60_000 = 1 minute. Should be raised to something more reasonable in the future.
    /// Minimum number of milliseconds a Rikiddo market must be in subsidy gathering phase.
    pub const MinSubsidyPeriod: Moment = 60_000;
    // 2_678_400_000 = 31 days.
    /// Maximum number of milliseconds a Rikiddo market can be in subsidy gathering phase.
    pub const MaxSubsidyPeriod: Moment = 2_678_400_000;
    // Requirements: MaxPeriod + ReportingPeriod + MaxDisputes * DisputePeriod < u64::MAX.
    /// The maximum market period.
    pub const MaxMarketPeriod: Moment = u64::MAX / 2;
    /// (Slashable) The orcale bond. Slashed in case the final outcome does not match the
    /// outcome the oracle reported.
    pub const OracleBond: Balance = 50 * CENT;
    /// Pallet identifier, mainly used for named balance reserves.
    pub const PmPalletId: PalletId = PM_PALLET_ID;
    /// Timeframe during which the oracle can report the final outcome after the market closed.
    pub const ReportingPeriod: u32 = BLOCKS_PER_DAY as u32;
    /// (Slashable) A bond for creation markets that do not require approval. Slashed in case
    /// the market is forcefully destroyed.
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
    /// Pallet identifier, mainly used for named balance reserves.
    pub const SimpleDisputesPalletId: PalletId = SD_PALLET_ID;

    // Swaps parameters
    /// A precentage from the withdrawal amount a liquidity provider wants to withdraw
    /// from a pool before the pool is closed.
    pub const ExitFee: Balance = 3 * BASE / 1000; // 0.3%
    /// Minimum number of assets.
    pub const MinAssets: u16 = 2;
    /// Maximum number of assets. `MaxCategories` plus one base asset.
    pub const MaxAssets: u16 = MAX_ASSETS;
    /// Mathematical constraint set by the Balancer algorithm. DO NOT CHANGE.
    pub const MaxInRatio: Balance = (BASE / 3) + 1;
    /// Mathematical constraint set by the Balancer algorithm. DO NOT CHANGE.
    pub const MaxOutRatio: Balance = (BASE / 3) + 1;
    /// The maximum fee that is charged for swaps and single asset LP operations.
    pub const MaxSwapFee: Balance = BASE / 10; // 10%
    /// The sum of all weights of the assets within the pool is limited by `MaxTotalWeight`.
    pub const MaxTotalWeight: Balance = 128 * BASE;
    /// The maximum weight a single asset can have.
    pub const MaxWeight: Balance = 64 * BASE;
    /// Minimum amount of liquidity required to launch a CPMM pool.
    pub const MinLiquidity: Balance = 100 * BASE;
    /// Minimum subsidy required to launch a Rikiddo pool.
    pub const MinSubsidy: Balance = MinLiquidity::get();
    /// Minimum subsidy a single account can provide.
    pub const MinSubsidyPerAccount: Balance = MinSubsidy::get();
    /// Minimum weight a single asset can have.
    pub const MinWeight: Balance = BASE;
    /// Pallet identifier, mainly used for named balance reserves.
    pub const SwapsPalletId: PalletId = SWAPS_PALLET_ID;

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
    /// A fee mulitplier for Operational extrinsics to compute “virtual tip”
    /// to boost their priority.
    pub const OperationalFeeMultiplier: u8 = 5;
    /// The fee that's paid for every byte in the transaction.
    pub const TransactionByteFee: Balance = 100 * MICRO;
    /// Once the dispatchables in a block consume that percentage of the total weight
    /// that's available for dispatchables in a block, the fee adjustment will start.
    pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(10);
    // With a target block time of 12 seconds (7200 blocks per day)
    // the weight fees can increase by at most ~21.46% per day, given extreme congestion.
    /// See https://paritytech.github.io/substrate/master/pallet_transaction_payment/struct.TargetedFeeAdjustment.html for details.
    pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
    /// Minimum amount of the multiplier. The test `multiplier_can_grow_from_zero` ensures
    /// that this value is not too low.
    pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000u128);

    // Treasury
    /// Percentage of spare funds (if any) that are burnt per spend period.
    pub const Burn: Permill = Permill::from_percent(50);
    /// The maximum number of approvals that can wait in the spending queue.
    pub const MaxApprovals: u32 = 100;
    /// Fraction of a proposal's value that should be bonded in order to place the proposal.
    /// An accepted proposal gets these back. A rejected proposal does not.
    pub const ProposalBond: Permill = Permill::from_percent(5);
    /// Minimum amount of funds that should be placed in a deposit for making a proposal.
    pub const ProposalBondMinimum: Balance = 10 * BASE;
    /// Maximum amount of funds that should be placed in a deposit for making a proposal.
    pub const ProposalBondMaximum: Balance = 500 * BASE;
    /// Period between successive spends.
    pub const SpendPeriod: BlockNumber = 24 * BLOCKS_PER_DAY;
    /// Pallet identifier, mainly used for named balance reserves.
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
