use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;

/// Defines the type of market creation.
#[derive(Eq, PartialEq, Encode, Decode, Clone, RuntimeDebug)]
pub enum MarketCreation {
    // A completely permissionless market that requires a higher
    // validity bond. May resolve as `Invalid`.
    Permissionless,
    // An advised market that must pass inspection by the advisory
    // committee. After being approved will never resolve as `Invalid`.
    Advised,
}

/// Defines whether the end is represented as a blocknumber or a timestamp.
#[derive(Eq, PartialEq, Encode, Decode, Clone, RuntimeDebug, Copy)]
pub enum MarketEnd<BlockNumber> {
    Block(BlockNumber),
    Timestamp(u64),
}

/// An inclusive range between the left side (lower) and right (upper).
pub type RangeInclusive<T> = (T, T);

/// Defines the type of market.
/// All markets also have the `Invalid` resolution.
#[derive(Eq, PartialEq, Encode, Decode, Clone, RuntimeDebug)]
pub enum MarketType {
    // A market with a number of categorical outcomes.
    Categorical(u16),
    // A market with a range of potential outcomes.
    Scalar(RangeInclusive<u128>),
}

/// Defines the state of the market.
#[derive(Eq, PartialEq, Encode, Decode, RuntimeDebug, Clone, Copy)]
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

#[derive(Eq, PartialEq, Encode, Decode, RuntimeDebug, Clone)]
pub enum Outcome {
    Categorical(u16),
    Scalar(u128),
}

#[derive(Encode, Decode, RuntimeDebug)]
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
    pub resolved_outcome: Option<Outcome>,
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

#[derive(Encode, Decode, RuntimeDebug, Clone)]
pub struct Report<AccountId, BlockNumber> {
    pub at: BlockNumber,
    pub by: AccountId,
    pub outcome: Outcome,
}

#[derive(Encode, Decode, RuntimeDebug, Clone)]
pub struct MarketDispute<AccountId, BlockNumber> {
    pub at: BlockNumber,
    pub by: AccountId,
    pub outcome: Outcome,
}
