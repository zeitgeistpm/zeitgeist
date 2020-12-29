#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

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

/// Defines the type of market.
/// All markets also have the `Invalid` resolution.
#[derive(Eq, PartialEq, Encode, Decode, Clone, RuntimeDebug)]
pub enum MarketType {
    // Binary market that can resolve either as `Yes`, `No`, or `Invalid`.
    Binary,
    // A market with a number of categorical outcomes.
    Categorical,
    Scalar,
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

#[derive(Encode, Decode, RuntimeDebug)]
pub struct Market<AccountId> {
    // Creator of this market.
    pub creator: AccountId,
    // Creation type.
    pub creation: MarketCreation,
    // The fee the creator gets from each winning share.
    pub creator_fee: u8, //TODO: Make this into a percent.
    // Oracle that reports the outcome of this market.
    pub oracle: AccountId,
    // Ending block for this market.
    pub end: u64,
    // Metadata for the market, usually a content address of IPFS
    // hosted JSON.
    pub metadata: Vec<u8>,
    // The type of the market.
    pub market_type: MarketType,
    // The current status of the market.
    pub status: MarketStatus,
    // The reported outcome. Only `Some` if it has been reported.
    pub reported_outcome: Option<u16>,
    // The actual reporter of the market.
    pub reporter: Option<AccountId>,
    // Categories are only relevant to Categorical markets.
    pub categories: Option<u16>,
}

impl<AccountId> Market<AccountId> {
    pub fn outcomes(&self) -> u16 {
        match self.market_type {
            MarketType::Binary => 2,
            MarketType::Categorical => self.categories.unwrap(),
            MarketType::Scalar => 0, // TODO figure out scalar markets
        }
    } 
}

#[derive(Encode, Decode, RuntimeDebug, Clone)]
pub struct MarketDispute<AccountId, BlockNumber> {
    pub at: BlockNumber,
    pub by: AccountId,
    pub outcome: u16,
}
