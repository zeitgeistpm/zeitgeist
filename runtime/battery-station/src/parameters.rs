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

#[cfg(feature = "parachain")]
use super::Runtime;
use super::{RuntimeHoldReason, VERSION};
use frame_support::{
    dispatch::DispatchClass,
    parameter_types,
    traits::{LockIdentifier, WithdrawReasons},
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, WEIGHT_REF_TIME_PER_SECOND},
        Weight,
    },
    PalletId,
};
use frame_system::limits::{BlockLength, BlockWeights};
use orml_traits::parameter_type_with_key;
use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use sp_runtime::{
    traits::{AccountIdConversion, Bounded},
    FixedPointNumber, Perbill, Percent, Permill, Perquintill,
};
use sp_version::RuntimeVersion;
use zeitgeist_primitives::{constants::*, types::*};

pub(crate) const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
pub(crate) const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
    // previously the block weight was 500ms, the updated execution time is 2000ms weight
    WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2),
    polkadot_primitives::MAX_POV_SIZE as u64,
);
pub(crate) const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

#[cfg(not(feature = "parachain"))]
parameter_types! {
    // Aura
    pub const AllowMultipleBlocksPerSlot: bool = false;
    pub const MaxAuthorities: u32 = 32;

    // Grandpa
    pub const MaxSetIdSessionEntries: u32 = 0;
    pub const MaxNominators: u32 = 0;
}

parameter_types! {
    // Authorized
    pub const AuthorizedPalletId: PalletId = AUTHORIZED_PALLET_ID;
    pub const CorrectionPeriod: BlockNumber = BLOCKS_PER_DAY;

    // Balance
    pub const ExistentialDeposit: u128 = BASE / 2;
    pub const MaxFreezes: u32 = 1;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;

    // Collective
    // Note: MaxMembers does not influence the pallet logic, but the worst-case weight estimation.
    pub const AdvisoryCommitteeMaxMembers: u32 = 100;
    // The maximum of proposals is currently u8::MAX otherwise the pallet_collective benchmark fails
    pub const AdvisoryCommitteeMaxProposals: u32 = 255;
    pub const AdvisoryCommitteeMotionDuration: BlockNumber = 3 * BLOCKS_PER_DAY;
    pub const CouncilMaxMembers: u32 = 100;
    pub const CouncilMaxProposals: u32 = 100;
    pub const CouncilMotionDuration: BlockNumber = 7 * BLOCKS_PER_DAY;
    pub MaxProposalWeight: Weight = Perbill::from_percent(50) * RuntimeBlockWeights::get().max_block;
    pub const TechnicalCommitteeMaxMembers: u32 = 100;
    pub const TechnicalCommitteeMaxProposals: u32 = 64;
    pub const TechnicalCommitteeMotionDuration: BlockNumber = 7 * BLOCKS_PER_DAY;

    // CombinatorialTokens
    pub const CombinatorialTokensPalletId: PalletId = COMBINATORIAL_TOKENS_PALLET_ID;

    // Contracts
    pub const ContractsMaxDelegateDependencies: u32 = 32;

    // Court
    /// (Slashable) Bond that is provided for overriding the last appeal.
    /// This bond increases exponentially with the number of appeals.
    /// Slashed in case the final outcome does match the appealed outcome for which the `AppealBond`
    /// was deposited.
    pub const AppealBond: Balance = 5 * BASE;
    /// The blocks per year required to calculate the yearly inflation for court incentivisation.
    pub const BlocksPerYear: BlockNumber = BLOCKS_PER_YEAR;
    /// Pallet identifier, mainly used for named balance reserves.
    pub const CourtPalletId: PalletId = COURT_PALLET_ID;
    /// The time in which the jurors can cast their secret vote.
    pub const CourtVotePeriod: BlockNumber = 3 * BLOCKS_PER_DAY;
    /// The time in which the jurors should reveal their secret vote.
    pub const CourtAggregationPeriod: BlockNumber = 3 * BLOCKS_PER_DAY;
    /// The time in which a court case can get appealed.
    pub const CourtAppealPeriod: BlockNumber = BLOCKS_PER_DAY;
    /// The court lock identifier.
    pub const CourtLockId: LockIdentifier = COURT_LOCK_ID;
    /// The time in which the inflation is periodically issued.
    pub const InflationPeriod: BlockNumber = 3 * BLOCKS_PER_DAY;
    /// The maximum number of appeals until the court fails.
    pub const MaxAppeals: u32 = 4;
    /// The maximum number of delegations per juror account.
    pub const MaxDelegations: u32 = 5;
    /// The maximum number of randomly selected `MinJurorStake` draws / atoms of jurors for a dispute.
    pub const MaxSelectedDraws: u32 = 510;
    /// The maximum number of jurors / delegators that can be registered.
    pub const MaxCourtParticipants: u32 = 1_000;
    /// The maximum yearly inflation for court incentivisation.
    pub const MaxYearlyInflation: Perbill = Perbill::from_percent(10);
    /// The minimum stake a user needs to reserve to become a juror.
    pub const MinJurorStake: Balance = 500 * BASE;
    /// The interval for requesting multiple court votes at once.
    pub const RequestInterval: BlockNumber = 7 * BLOCKS_PER_DAY;

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
    pub const DemocracyMaxProposals: u32 = 100;

    // Futarchy
    pub const FutarchyMaxProposals: u32 = 4;
    pub const MinDuration: BlockNumber = 7 * BLOCKS_PER_DAY;

    // Hybrid Router parameters
    pub const HybridRouterPalletId: PalletId = HYBRID_ROUTER_PALLET_ID;
    /// Maximum number of orders that can be placed in a single trade transaction.
    pub const MaxOrders: u32 = 100;

    // Identity
    /// The amount held on deposit for a registered identity
    pub const BasicDeposit: Balance = deposit(1, 258);
    /// The amount held on deposit per encoded byte for a registered identity.
    pub const IdentityByteDeposit: Balance = deposit(0, 1);
    /// Maximum number of additional fields that may be stored in an ID. Needed to bound the I/O
    /// required to access an identity, but can be pretty high.
    pub const MaxAdditionalFields: u32 = 64;
    /// Maxmimum number of registrars allowed in the system. Needed to bound the complexity
    /// of, e.g., updating judgements.
    pub const MaxRegistrars: u32 = 8;
    /// The maximum number of sub-accounts allowed per identified account.
    pub const MaxSubAccounts: u32 = 64;
    /// The maximum length of a suffix.
    pub const MaxSuffixLength: u32 = 7;
    /// The maximum length of a username, including its suffix and any system-added delimiters.
    pub const MaxUsernameLength: u32 = 32;
    /// The number of blocks within which a username grant must be accepted.
    pub const PendingUsernameExpiration: u64 = 7 * BLOCKS_PER_DAY;
    /// The amount held on deposit for a registered subaccount. This should account for the fact
    /// that one storage item's value will increase by the size of an account ID, and there will
    /// be another trie item whose value is the size of an account ID plus 32 bytes.
    pub const SubAccountDeposit: Balance = deposit(1, 53);

    // Multisig
    // One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
    pub const DepositBase: Balance = deposit(1, 88);
    // Additional storage item size of 32 bytes.
    pub const DepositFactor: Balance = deposit(0, 32);

    // NeoSwaps
    pub const NeoSwapsMaxSwapFee: Balance = 10 * CENT;
    pub const NeoSwapsPalletId: PalletId = NS_PALLET_ID;
    pub const MaxLiquidityTreeDepth: u32 = 9u32;
    pub const MaxSplits: u16 = 128u16;

    // ORML
    pub const GetNativeCurrencyId: CurrencyId = Asset::Ztg;

    // Prediction Market parameters
    /// (Slashable) Bond that is provided for creating an advised market that needs approval.
    /// Slashed in case the market is rejected.
    pub const AdvisoryBond: Balance = 25 * CENT;
    /// The percentage of the advisory bond that gets slashed when a market is rejected.
    pub const AdvisoryBondSlashPercentage: Percent = Percent::from_percent(0);
    /// (Slashable) Bond that is provided for disputing an early market close by the market creator.
    pub const CloseEarlyDisputeBond: Balance = BASE;
    // Fat-finger protection for the advisory committe to reject
    // the early market schedule.
    pub const CloseEarlyProtectionTimeFramePeriod: Moment = CloseEarlyProtectionBlockPeriod::get() * MILLISECS_PER_BLOCK as u64;
    // Fat-finger protection for the advisory committe to reject
    // the early market schedule.
    pub const CloseEarlyProtectionBlockPeriod: BlockNumber = BLOCKS_PER_HOUR;
    /// (Slashable) Bond that is provided for scheduling an early market close.
    pub const CloseEarlyRequestBond: Balance = BASE;
    /// (Slashable) Bond that is provided for disputing the outcome.
    /// Unreserved in case the dispute was justified otherwise slashed.
    /// This is when the resolved outcome is different to the default (reported) outcome.
    pub const DisputeBond: Balance = 25 * BASE;
    /// Maximum Categories a prediciton market can have (excluding base asset).
    pub const MaxCategories: u16 = MAX_CATEGORIES;
    /// Max creator fee, bounds the fraction per trade volume that is moved to the market creator.
    pub const MaxCreatorFee: Perbill = Perbill::from_percent(1);
    /// Maximum block period for a dispute.
    pub const MaxDisputeDuration: BlockNumber = MAX_DISPUTE_DURATION;
    /// Maximum number of disputes.
    pub const MaxDisputes: u16 = 6;
    /// Maximum string length for edit reason.
    pub const MaxEditReasonLen: u32 = 1024;
    /// Maximum block period for a grace_period.
    /// The grace_period is a delay between the point where the market closes and the point where the oracle may report.
    pub const MaxGracePeriod: BlockNumber = MAX_GRACE_PERIOD;
    /// The maximum allowed duration of a market from creation to market close in blocks.
    pub const MaxMarketLifetime: BlockNumber = MAX_MARKET_LIFETIME;
    /// Maximum block period for a oracle_duration.
    /// The oracle_duration is a duration where the oracle has to submit its report.
    pub const MaxOracleDuration: BlockNumber = MAX_ORACLE_DURATION;
    /// Maximum string length allowed for reject reason.
    pub const MaxRejectReasonLen: u32 = 1024;
    /// Minimum number of categories. The trivial minimum is 2, which represents a binary market.
    pub const MinCategories: u16 = 2;
    /// The dispute_duration is time where users can dispute the outcome.
    /// Minimum block period for a dispute.
    pub const MinDisputeDuration: BlockNumber = MIN_DISPUTE_DURATION;
    /// Minimum block period for a oracle_duration.
    pub const MinOracleDuration: BlockNumber = MIN_ORACLE_DURATION;
    /// (Slashable) The orcale bond. Slashed in case the final outcome does not match the
    /// outcome the oracle reported.
    pub const OracleBond: Balance = 50 * CENT;
    /// (Slashable) A bond for an outcome reporter, who is not the oracle.
    /// Slashed in case the final outcome does not match the outcome by the outsider.
    pub const OutsiderBond: Balance = 2 * OracleBond::get();
    /// Pallet identifier, mainly used for named balance reserves.
    pub const PmPalletId: PalletId = PM_PALLET_ID;
    // Waiting time for market creator to close
    // the market after an early close schedule.
    pub const CloseEarlyBlockPeriod: BlockNumber = BLOCKS_PER_HOUR;
    // Waiting time for market creator to close
    // the market after an early close schedule.
    pub const CloseEarlyTimeFramePeriod: Moment = CloseEarlyBlockPeriod::get() * MILLISECS_PER_BLOCK as u64;
    /// (Slashable) A bond for creation markets that do not require approval. Slashed in case
    /// the market is forcefully destroyed.
    pub const ValidityBond: Balance = 50 * CENT;

    // Preimage
    pub const PreimageMaxSize: u32 = 4096 * 1024;
    pub PreimageBaseDeposit: Balance = deposit(2, 64);
    pub PreimageByteDeposit: Balance = deposit(0, 1);
    pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);

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

    // Swaps parameters
    /// A precentage from the withdrawal amount a liquidity provider wants to withdraw
    /// from a pool before the pool is closed.
    pub const ExitFee: Balance = BASE / 10_000; // 0.01%
    /// Minimum number of assets.
    pub const MinAssets: u16 = 2;
    /// Maximum number of assets. `MaxCategories` plus one base asset.
    pub const MaxAssets: u16 = MAX_ASSETS;
    /// The maximum fee that is charged for swaps and single asset LP operations.
    pub const MaxSwapFee: Balance = BASE / 10; // 10%
    /// The sum of all weights of the assets within the pool is limited by `MaxTotalWeight`.
    pub const MaxTotalWeight: Balance = MaxWeight::get() * 2;
    /// The maximum weight a single asset can have.
    pub const MaxWeight: Balance = 64 * BASE;
    /// Minimum weight a single asset can have.
    pub const MinWeight: Balance = BASE;
    /// Pallet identifier, mainly used for named balance reserves.
    pub const SwapsPalletId: PalletId = SWAPS_PALLET_ID;

    // Orderbook parameters
    pub const OrderbookPalletId: PalletId = ORDERBOOK_PALLET_ID;

    // Parimutuel parameters
    pub const MinBetSize: Balance = 5 * BASE;
    pub const ParimutuelPalletId: PalletId = PARIMUTUEL_PALLET_ID;

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
    /// Maximum amount of the multiplier.
    pub MaximumMultiplier: Multiplier = Bounded::max_value();
    /// The percentage of the fees that are paid to the treasury.
    pub FeesTreasuryProportion: Perbill = Perbill::from_percent(100);

    // Treasury
    /// Percentage of spare funds (if any) that are burnt per spend period.
    pub const Burn: Permill = Permill::from_percent(10);
    /// The maximum number of approvals that can wait in the spending queue.
    pub const MaxApprovals: u32 = 100;
    /// Maximum amount a verified origin can spend
    pub const MaxTreasurySpend: Balance = Balance::MAX;
    /// The period during which an approved treasury spend has to be claimed.
    pub const PayoutPeriod: BlockNumber = 30 * BLOCKS_PER_DAY;
    /// Fraction of a proposal's value that should be bonded in order to place the proposal.
    /// An accepted proposal gets these back. A rejected proposal does not.
    pub const ProposalBond: Permill = Permill::from_percent(5);
    /// Period between successive spends.
    pub const SpendPeriod: BlockNumber = 24 * BLOCKS_PER_DAY;
    /// Pallet identifier, mainly used for named balance reserves.
    pub const TreasuryPalletId: PalletId = TREASURY_PALLET_ID;
    /// Treasury account.
    pub ZeitgeistTreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();

    // Timestamp
    /// MinimumPeriod for Timestamp
    pub const MinimumPeriodValue: u64 = MILLISECS_PER_BLOCK as u64 / 2;

    // Bounties
    /// The amount held on deposit for placing a bounty proposal.
    pub const BountyDepositBase: Balance = 100 * BASE;
    /// The delay period that a bounty beneficiary needs to wait before being able to claim the payout.
    pub const BountyDepositPayoutDelay: BlockNumber = 3 * BLOCKS_PER_DAY;

    /// Bounty duration in blocks.
    pub const BountyUpdatePeriod: BlockNumber = 35 * BLOCKS_PER_DAY;

    /// The curator deposit is calculated as a percentage of the curator fee.
    ///
    /// This deposit has optional upper and lower bounds with `CuratorDepositMax` and
    /// `CuratorDepositMin`.
    pub const CuratorDepositMultiplier: Permill = Permill::from_percent(50);

    /// Maximum amount of funds that should be placed in a deposit for making a proposal.
    pub const CuratorDepositMax: Balance = 500 * BASE;
    /// Minimum amount of funds that should be placed in a deposit for making a proposal.
    pub const CuratorDepositMin: Balance = 10 * BASE;
    /// Minimum value for a bounty.
    pub const BountyValueMinimum: Balance = 50 * BASE;

    /// The amount held on deposit per byte within the tip report reason or bounty description.
    pub DataDepositPerByte: Balance = BASE;
    /// Maximum acceptable reason length.
    ///
    /// Benchmarks depend on this value, be sure to update weights file when changing this value
    pub MaximumReasonLength: u32 = 8192;

    // Vesting
    pub const MinVestedTransfer: Balance = ExistentialDeposit::get();
    pub UnvestedFundsAllowedWithdrawReasons: WithdrawReasons =
         WithdrawReasons::except(WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE);
}

parameter_types! {
    // Global Disputes
    /// The time period in which the addition of new outcomes are allowed.
    pub const AddOutcomePeriod: BlockNumber = BLOCKS_PER_DAY;
    /// Vote lock identifier, mainly used for the LockableCurrency on the native token.
    pub const GlobalDisputeLockId: LockIdentifier = GLOBAL_DISPUTES_LOCK_ID;
    /// Pallet identifier
    pub const GlobalDisputesPalletId: PalletId = GLOBAL_DISPUTES_PALLET_ID;
    /// The maximum number of owners for a voting outcome for private API calls of `push_vote_outcome`.
    pub const MaxOwners: u32 = 10;
    /// The maximum number of market ids (participate in multiple different global disputes at the same time) for one account to vote on outcomes.
    pub const MaxGlobalDisputeVotes: u32 = 50;
    /// The minimum required amount to vote on an outcome.
    pub const MinOutcomeVoteAmount: Balance = 10 * BASE;
    /// The time period in which votes are allowed.
    pub const GdVotingPeriod: BlockNumber = 3 * BLOCKS_PER_DAY;
    /// The fee required to add a voting outcome.
    pub const VotingOutcomeFee: Balance = 200 * BASE;
    /// The remove limit for the Outcomes storage double map.
    pub const RemoveKeysLimit: u32 = 250;
}

parameter_type_with_key! {
    // Existential deposits used by orml-tokens.
    // Only native ZTG and foreign assets should have an existential deposit.
    // Winning outcome tokens are redeemed completely by the user, losing outcome tokens
    // are cleaned up automatically. In case of scalar outcomes, the market account can have dust.
    // Unless LPs use `pool_exit_with_exact_asset_amount`, there can be some dust pool shares remaining.
    // Explicit match arms are used to ensure new asset types are respected.
    pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
        match currency_id {
            Asset::CategoricalOutcome(_,_) => ExistentialDeposit::get(),
            Asset::CombinatorialToken(_) => ExistentialDeposit::get(),
            Asset::CombinatorialOutcomeLegacy => ExistentialDeposit::get(),
            Asset::PoolShare(_)  => ExistentialDeposit::get(),
            Asset::ScalarOutcome(_,_)  => ExistentialDeposit::get(),
            #[cfg(feature = "parachain")]
            Asset::ForeignAsset(id) => {
                let maybe_metadata = <
                    orml_asset_registry::module::Pallet<Runtime> as orml_traits::asset_registry::Inspect
                >::metadata(&Asset::ForeignAsset(*id));

                if let Some(metadata) = maybe_metadata {
                    return metadata.existential_deposit;
                }

                1
            }
            #[cfg(not(feature = "parachain"))]
            Asset::ForeignAsset(_) => ExistentialDeposit::get(),
            Asset::Ztg => ExistentialDeposit::get(),
            Asset::ParimutuelShare(_,_) => ExistentialDeposit::get(),
        }
    };
}

// Parameterized slow adjusting fee updated based on
// https://research.web3.foundation/en/latest/polkadot/overview/2-token-economics.html#-2.-slow-adjusting-mechanism
pub type SlowAdjustingFeeUpdate<R> = TargetedFeeAdjustment<
    R,
    TargetBlockFullness,
    AdjustmentVariable,
    MinimumMultiplier,
    MaximumMultiplier,
>;
