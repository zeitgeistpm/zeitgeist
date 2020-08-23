#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

/// Defines the type of market.
/// All markets also have the `Invalid` resolution.
#[derive(Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum MarketType {
    // Binary market.
    YesNo,
    // A market with a number of categorical outcomes.
    Categorical,
    Scalar,
}

/// Defines the state of the market.
#[derive(Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
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

#[derive(Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Market<AccountId, BlockNumber> {
    // Creator of this market.
    pub creator: AccountId,
    // Oracle that reports the outcome of this market.
    pub oracle: AccountId,
    // Ending block for this market.
    pub end_block: BlockNumber,
    // Metadata for the market, usually and content address of IPFS
    // hosted JSON.
    pub metadata: [u8; 32],
    // The type of the market.
    pub market_type: MarketType,
    // Number of outcomes (always includes Invalid).
    pub outcomes: u16,
    // The current status of the market.
    pub status: MarketStatus,
    // The winning outcome. Only `Some` if it has been resolved.
    pub winning_outcome: Option<u16>,
}
