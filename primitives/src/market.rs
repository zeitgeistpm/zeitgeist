use crate::{pool::ScoringRule, types::OutcomeReport};
use alloc::vec::Vec;
use core::ops::{Range, RangeInclusive};

/// Types
///
/// * `AI`: Account Id
/// * `BN`: Block Number
/// * `M`: Moment (Time moment)
#[derive(
    scale_info::TypeInfo,
    Clone,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub struct Market<AI, BN, M> {
    /// Creator of this market.
    pub creator: AI,
    /// Creation type.
    pub creation: MarketCreation,
    /// The fee the creator gets from each winning share.
    pub creator_fee: u8,
    /// Oracle that reports the outcome of this market.
    pub oracle: AI,
    /// Metadata for the market, usually a content address of IPFS
    /// hosted JSON.
    pub metadata: Vec<u8>,
    /// The type of the market.
    pub market_type: MarketType,
    /// Market start and end
    pub period: MarketPeriod<BN, M>,
    /// The scoring rule used for the market.
    pub scoring_rule: ScoringRule,
    /// The current status of the market.
    pub status: MarketStatus,
    /// The report of the market. Only `Some` if it has been reported.
    pub report: Option<Report<AI, BN>>,
    /// The resolved outcome.
    pub resolved_outcome: Option<OutcomeReport>,
    /// See [`MarketDisputeMechanism`].
    pub mdm: MarketDisputeMechanism<AI>,
}

impl<AI, BN, M> Market<AI, BN, M> {
    // Returns the number of outcomes for a market.
    pub fn outcomes(&self) -> u16 {
        match self.market_type {
            MarketType::Categorical(categories) => categories,
            MarketType::Scalar(_) => 2,
        }
    }
}

/// Defines the type of market creation.
#[derive(
    scale_info::TypeInfo,
    Clone,
    Eq,
    PartialEq,
    parity_scale_codec::Encode,
    parity_scale_codec::Decode,
    sp_runtime::RuntimeDebug,
)]
pub enum MarketCreation {
    // A completely permissionless market that requires a higher
    // validity bond. May resolve as `Invalid`.
    Permissionless,
    // An advised market that must pass inspection by the advisory
    // committee. After being approved will never resolve as `Invalid`.
    Advised,
}

#[derive(
    scale_info::TypeInfo,
    Clone,
    PartialEq,
    parity_scale_codec::Encode,
    parity_scale_codec::Decode,
    sp_runtime::RuntimeDebug,
)]
pub struct MarketDispute<AccountId, BlockNumber> {
    pub at: BlockNumber,
    pub by: AccountId,
    pub outcome: OutcomeReport,
}

/// How a market should resolve disputes
#[derive(
    scale_info::TypeInfo,
    Clone,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub enum MarketDisputeMechanism<AI> {
    Authorized(AI),
    Court,
    SimpleDisputes,
}

/// Defines whether the period is represented as a blocknumber or a timestamp.
///
/// ****** IMPORTANT *****
///
/// Must be an exclusive range because:
///
/// 1. `zrml_predition_markets::Pallet::admin_move_market_to_closed` uses the current block as the
/// end period.
/// 2. The liquidity mining pallet takes into consideration the different between the two blocks.
/// So 1..5 correctly outputs 4 (`5 - 1`) while 1..=5 would incorrectly output the same 4.
/// 3. With inclusive ranges it is not possible to express empty ranges and this feature
/// mostly conflicts with existent tests and corner cases.
#[derive(
    scale_info::TypeInfo,
    Clone,
    Eq,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub enum MarketPeriod<BN, M> {
    Block(Range<BN>),
    Timestamp(Range<M>),
}

/// Defines the state of the market.
#[derive(
    scale_info::TypeInfo,
    Clone,
    Copy,
    Eq,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub enum MarketStatus {
    /// The market has been proposed and is either waiting for approval
    /// from the governing committee, or hasn't reach its delay yet.
    Proposed,
    /// Trading on the market is active.
    Active,
    /// Trading on the market is temporarily paused.
    Suspended,
    /// Trading on the market has concluded.
    Closed,
    /// The market is collecting subsidy.
    CollectingSubsidy,
    /// The market was discarded due to insufficient subsidy.
    InsufficientSubsidy,
    /// The market has been reported.
    Reported,
    /// The market outcome is being disputed.
    Disputed,
    /// The market outcome has been resolved and can be cleaned up
    /// after the `MarketWipeDelay`.
    Resolved,
}

/// Defines the type of market.
/// All markets also have themin_assets_out `Invalid` resolution.
#[derive(
    scale_info::TypeInfo,
    Clone,
    Eq,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub enum MarketType {
    /// A market with a number of categorical outcomes.
    Categorical(u16),
    /// A market with a range of potential outcomes.
    Scalar(RangeInclusive<u128>),
}

#[derive(
    scale_info::TypeInfo,
    Clone,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub struct Report<AccountId, BlockNumber> {
    pub at: BlockNumber,
    pub by: AccountId,
    pub outcome: OutcomeReport,
}

/// Contains a market id and the market period.
///
/// * `BN`: Block Number
/// * `MO`: Moment (Time moment)
/// * `MI`: Market Id
#[derive(
    scale_info::TypeInfo,
    Clone,
    Eq,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub struct SubsidyUntil<BN, MO, MI> {
    /// Market id of associated market.
    pub market_id: MI,
    /// Market start and end.
    pub period: MarketPeriod<BN, MO>,
}
