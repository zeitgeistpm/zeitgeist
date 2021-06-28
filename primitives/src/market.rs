use crate::types::OutcomeReport;
use alloc::vec::Vec;

#[derive(
    PartialEq, parity_scale_codec::Decode, parity_scale_codec::Encode, sp_runtime::RuntimeDebug,
)]
pub struct Market<AccountId, BlockNumber> {
    // Creator of this market.
    pub creator: AccountId,
    // Creation type.
    pub creation: MarketCreation,
    // The fee the creator gets from each winning share.
    pub creator_fee: u8,
    // Oracle that reports the outcome of this market.
    pub oracle: AccountId,
    // Ending block for this market.
    pub end: MarketEnd<BlockNumber>,
    // Metadata for the market, usually a content address of IPFS
    // hosted JSON.
    pub metadata: Vec<u8>,
    // The type of the market.
    pub market_type: MarketType,
    // The current status of the market.
    pub status: MarketStatus,
    // The report of the market. Only `Some` if it has been reported.
    pub report: Option<Report<AccountId, BlockNumber>>,
    // The resolved outcome.
    pub resolved_outcome: Option<OutcomeReport>,
}

impl<AccountId, B> Market<AccountId, B> {
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

/// Defines whether the end is represented as a blocknumber or a timestamp.
#[derive(
    Clone,
    Copy,
    Eq,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub enum MarketEnd<BlockNumber> {
    Block(BlockNumber),
    Timestamp(u64),
}

/// Defines the state of the market.
#[derive(
    Clone,
    Copy,
    Eq,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub enum MarketStatus {
    // The market has been proposed and is either waiting for approval
    // from the governing committee, or hasn't reach its delay yet.
    Proposed,
    // Trading on the market is active.
    Active,
    // Trading on the market is temporarily paused.
    Suspended,
    // Trading on the market has concluded.
    Closed,
    // The market has been reported.
    Reported,
    // The market outcome is being disputed.
    Disputed,
    // The market outcome has been resolved and can be cleaned up
    // after the `MarketWipeDelay`.
    Resolved,
}

/// Defines the type of market.
/// All markets also have the `Invalid` resolution.
#[derive(
    Clone,
    Eq,
    PartialEq,
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    sp_runtime::RuntimeDebug,
)]
pub enum MarketType {
    // A market with a number of categorical outcomes.
    Categorical(u16),
    // A market with a range of potential outcomes.
    Scalar(RangeInclusive<u128>),
}

#[derive(
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

// An inclusive range between the left side (lower) and right (upper).
type RangeInclusive<T> = (T, T);
