use crate::{pool::ScoringRule, types::OutcomeReport};
use alloc::vec::Vec;
use core::ops::{Range, RangeInclusive};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

/// Types
///
/// * `AI`: Account Id
/// * `BN`: Block Number
/// * `M`: Moment (Time moment)
#[derive(Clone, Decode, Encode, PartialEq, RuntimeDebug, TypeInfo)]
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
    /// hosted JSON. Currently limited to 66 bytes (see `MaxEncodedLen` implementation)
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

    /// Check if `outcome_report` matches the type of this market.
    pub fn matches_outcome_report(&self, outcome_report: &OutcomeReport) -> bool {
        match outcome_report {
            OutcomeReport::Categorical(ref inner) => {
                if let MarketType::Categorical(ref categories) = &self.market_type {
                    inner < categories
                } else {
                    false
                }
            }
            OutcomeReport::Scalar(_) => {
                matches!(&self.market_type, MarketType::Scalar(_))
            }
        }
    }
}

impl<AI, BN, M> MaxEncodedLen for Market<AI, BN, M>
where
    AI: MaxEncodedLen,
    BN: MaxEncodedLen,
    M: MaxEncodedLen,
{
    fn max_encoded_len() -> usize {
        AI::max_encoded_len()
            .saturating_add(MarketCreation::max_encoded_len())
            .saturating_add(u8::max_encoded_len())
            .saturating_add(AI::max_encoded_len())
            // We assume that at max. a 512 bit hash function is used
            .saturating_add(u8::max_encoded_len().saturating_mul(68))
            .saturating_add(MarketType::max_encoded_len())
            .saturating_add(<MarketPeriod<BN, M>>::max_encoded_len())
            .saturating_add(ScoringRule::max_encoded_len())
            .saturating_add(MarketStatus::max_encoded_len())
            .saturating_add(<Option<Report<AI, BN>>>::max_encoded_len())
            .saturating_add(<Option<OutcomeReport>>::max_encoded_len())
            .saturating_add(<MarketDisputeMechanism<AI>>::max_encoded_len())
    }
}

/// Defines the type of market creation.
#[derive(Clone, Decode, Encode, MaxEncodedLen, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum MarketCreation {
    // A completely permissionless market that requires a higher
    // validity bond. May resolve as `Invalid`.
    Permissionless,
    // An advised market that must pass inspection by the advisory
    // committee. After being approved will never resolve as `Invalid`.
    Advised,
}

#[derive(Clone, Decode, Encode, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketDispute<AccountId, BlockNumber> {
    pub at: BlockNumber,
    pub by: AccountId,
    pub outcome: OutcomeReport,
}

/// How a market should resolve disputes
#[derive(Clone, Decode, Encode, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
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
#[derive(Clone, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum MarketPeriod<BN, M> {
    Block(Range<BN>),
    Timestamp(Range<M>),
}

impl<BN: MaxEncodedLen, M: MaxEncodedLen> MaxEncodedLen for MarketPeriod<BN, M> {
    fn max_encoded_len() -> usize {
        // Since it is an enum, the biggest element is the only one of interest here.
        BN::max_encoded_len().max(M::max_encoded_len()).saturating_mul(2)
    }
}

/// Defines the state of the market.
#[derive(Clone, Copy, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
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
#[derive(Clone, Decode, Encode, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum MarketType {
    /// A market with a number of categorical outcomes.
    Categorical(u16),
    /// A market with a range of potential outcomes.
    Scalar(RangeInclusive<u128>),
}

impl MaxEncodedLen for MarketType {
    fn max_encoded_len() -> usize {
        u128::max_encoded_len().saturating_mul(2)
    }
}

#[derive(Clone, Decode, Encode, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
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
#[derive(TypeInfo, Clone, Eq, PartialEq, Decode, Encode, MaxEncodedLen, RuntimeDebug)]
pub struct SubsidyUntil<BN, MO, MI> {
    /// Market id of associated market.
    pub market_id: MI,
    /// Market start and end.
    pub period: MarketPeriod<BN, MO>,
}

#[cfg(test)]
mod tests {
    use crate::market::*;
    use test_case::test_case;
    type Market = crate::market::Market<u32, u32, u32>;

    #[test_case(
        MarketType::Categorical(6),
        OutcomeReport::Categorical(3),
        true;
        "categorical market ok"
    )]
    #[test_case(
        MarketType::Categorical(6),
        OutcomeReport::Categorical(6),
        false;
        "categorical market report equals number of categories"
    )]
    #[test_case(
        MarketType::Categorical(6),
        OutcomeReport::Categorical(7),
        false;
        "categorical market report larger than number of categories"
    )]
    #[test_case(
        MarketType::Categorical(6),
        OutcomeReport::Scalar(3),
        false;
        "categorical market report is scalar"
    )]
    #[test_case(
        MarketType::Scalar(12..=34),
        OutcomeReport::Scalar(23),
        true;
        "scalar market ok"
    )]
    #[test_case(
        MarketType::Scalar(12..=34),
        OutcomeReport::Scalar(1),
        true;
        "scalar market short"
    )]
    #[test_case(
        MarketType::Scalar(12..=34),
        OutcomeReport::Scalar(45),
        true;
        "scalar market long"
    )]
    #[test_case(
        MarketType::Scalar(12..=34),
        OutcomeReport::Categorical(23),
        false;
        "scalar market report is categorical"
    )]
    fn market_matches_outcome_report(
        market_type: MarketType,
        outcome_report: OutcomeReport,
        expected: bool,
    ) {
        let market = Market {
            creator: 1,
            creation: MarketCreation::Permissionless,
            creator_fee: 2,
            oracle: 3,
            metadata: vec![4u8; 5],
            market_type, // : MarketType::Categorical(6),
            period: MarketPeriod::Block(7..8),
            scoring_rule: ScoringRule::CPMM,
            status: MarketStatus::Active,
            report: None,
            resolved_outcome: None,
            mdm: MarketDisputeMechanism::Authorized(9),
        };
        assert_eq!(market.matches_outcome_report(&outcome_report), expected);
    }
}
