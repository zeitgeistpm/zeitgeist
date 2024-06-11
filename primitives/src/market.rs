// Copyright 2022-2024 Forecasting Technologies LTD.
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

use crate::types::{MarketAssetClass, OutcomeReport, ScalarPosition};
use alloc::{vec, vec::Vec};
use core::ops::{Range, RangeInclusive};
use parity_scale_codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::per_things::Perbill;
use sp_runtime::RuntimeDebug;

/// Types
///
/// * `AI`: Account id
/// * `BA`: Balance type
/// * `BN`: Block number
/// * `M`: Moment (time moment)
/// * `A`: Asset
/// * `MI`: Market ID
#[derive(Clone, Decode, Encode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Market<AI, BA, BN, M, A, MI> {
    pub market_id: MI,
    /// Base asset of the market.
    pub base_asset: A,
    /// Creator of this market.
    pub creator: AI,
    /// Creation type.
    pub creation: MarketCreation,
    /// A fee that is charged each trade and given to the market creator.
    pub creator_fee: Perbill,
    /// Oracle that reports the outcome of this market.
    pub oracle: AI,
    /// Metadata for the market, usually a content address of IPFS
    /// hosted JSON. Currently limited to 66 bytes (see `MaxEncodedLen` implementation)
    pub metadata: Vec<u8>,
    /// The type of the market.
    pub market_type: MarketType,
    /// Market start and end
    pub period: MarketPeriod<BN, M>,
    /// Market deadlines.
    pub deadlines: Deadlines<BN>,
    /// The scoring rule used for the market.
    pub scoring_rule: ScoringRule,
    /// The current status of the market.
    pub status: MarketStatus,
    /// The report of the market. Only `Some` if it has been reported.
    pub report: Option<Report<AI, BN>>,
    /// The resolved outcome.
    pub resolved_outcome: Option<OutcomeReport>,
    /// See [`MarketDisputeMechanism`].
    pub dispute_mechanism: Option<MarketDisputeMechanism>,
    /// The bonds reserved for this market.
    pub bonds: MarketBonds<AI, BA>,
    /// The time at which the market was closed early.
    pub early_close: Option<EarlyClose<BN, M>>,
}

impl<AI, BA, BN, M, A, MI> Market<AI, BA, BN, M, A, MI>
where
    MI: Copy + HasCompact + MaxEncodedLen,
{
    /// Returns the `ResolutionMechanism` of market, currently either:
    /// - `RedeemTokens`, which implies that the module that handles the state transitions of
    ///    a market is also responsible to provide means for redeeming rewards
    /// - `Noop`, which implies that another module provides the means for redeeming rewards
    pub fn resolution_mechanism(&self) -> ResolutionMechanism {
        match self.scoring_rule {
            ScoringRule::AmmCdaHybrid => ResolutionMechanism::RedeemTokens,
            ScoringRule::Parimutuel => ResolutionMechanism::Noop,
        }
    }

    /// Returns whether the market is redeemable, i.e. reward payout is managed within
    /// the same module that controls the state transitions of the underlying market.
    pub fn is_redeemable(&self) -> bool {
        matches!(self.resolution_mechanism(), ResolutionMechanism::RedeemTokens)
    }

    /// Returns the number of outcomes for a market.
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

    /// Returns a `Vec` of all outcomes for `market_id`.
    pub fn outcome_assets(&self) -> Vec<MarketAssetClass<MI>> {
        match self.market_type {
            MarketType::Categorical(categories) => {
                let mut assets = Vec::new();

                for i in 0..categories {
                    match self.scoring_rule {
                        ScoringRule::AmmCdaHybrid => assets
                            .push(MarketAssetClass::<MI>::CategoricalOutcome(self.market_id, i)),
                        ScoringRule::Parimutuel => {
                            assets.push(MarketAssetClass::<MI>::ParimutuelShare(self.market_id, i))
                        }
                    };
                }

                assets
            }
            MarketType::Scalar(_) => {
                vec![
                    MarketAssetClass::<MI>::ScalarOutcome(self.market_id, ScalarPosition::Long),
                    MarketAssetClass::<MI>::ScalarOutcome(self.market_id, ScalarPosition::Short),
                ]
            }
        }
    }

    /// Tries to convert the reported outcome for `market_id` into an asset,
    /// returns `None` if not possible. Cases where `None` is returned are:
    /// - The reported outcome does not exist
    /// - The reported outcome does not have a corresponding asset type
    pub fn report_into_asset(&self) -> Option<MarketAssetClass<MI>> {
        let outcome = if let Some(ref report) = self.report {
            &report.outcome
        } else {
            return None;
        };

        self.outcome_report_into_asset(outcome)
    }

    /// Tries to convert the resolved outcome for `market_id` into an asset,
    /// returns `None` if not possible. Cases where `None` is returned are:
    /// - The resolved outcome does not exist
    /// - The resolved outcome does not have a corresponding asset type
    pub fn resolved_outcome_into_asset(&self) -> Option<MarketAssetClass<MI>> {
        let outcome = self.resolved_outcome.as_ref()?;
        self.outcome_report_into_asset(outcome)
    }

    /// Tries to convert a `outcome_report` for `market_id` into an asset,
    /// returns `None` if not possible.
    fn outcome_report_into_asset(
        &self,
        outcome_report: &OutcomeReport,
    ) -> Option<MarketAssetClass<MI>> {
        match outcome_report {
            OutcomeReport::Categorical(idx) => match self.scoring_rule {
                ScoringRule::AmmCdaHybrid => {
                    Some(MarketAssetClass::<MI>::CategoricalOutcome(self.market_id, *idx))
                }
                ScoringRule::Parimutuel => {
                    Some(MarketAssetClass::<MI>::ParimutuelShare(self.market_id, *idx))
                }
            },
            OutcomeReport::Scalar(_) => None,
        }
    }
}

/// Tracks the status of a bond.
#[derive(Clone, Decode, Encode, MaxEncodedLen, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Bond<AI, BA> {
    /// The account that reserved the bond.
    pub who: AI,
    /// The amount reserved.
    pub value: BA,
    /// `true` if and only if the bond is unreserved and/or (partially) slashed.
    pub is_settled: bool,
}

impl<AI, BA> Bond<AI, BA> {
    pub fn new(who: AI, value: BA) -> Bond<AI, BA> {
        Bond { who, value, is_settled: false }
    }
}

/// Tracks bonds associated with a prediction market.
#[derive(Clone, Decode, Encode, MaxEncodedLen, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct MarketBonds<AI, BA> {
    pub creation: Option<Bond<AI, BA>>,
    pub oracle: Option<Bond<AI, BA>>,
    pub outsider: Option<Bond<AI, BA>>,
    pub dispute: Option<Bond<AI, BA>>,
    pub close_request: Option<Bond<AI, BA>>,
    pub close_dispute: Option<Bond<AI, BA>>,
}

impl<AI: Ord, BA: frame_support::traits::tokens::Balance> MarketBonds<AI, BA> {
    /// Return the combined value of the open bonds for `who`.
    pub fn total_amount_bonded(&self, who: &AI) -> BA {
        let value_or_default = |bond: &Option<Bond<AI, BA>>| match bond {
            Some(bond) if bond.who == *who => bond.value,
            _ => BA::zero(),
        };
        value_or_default(&self.creation)
            .saturating_add(value_or_default(&self.oracle))
            .saturating_add(value_or_default(&self.outsider))
            .saturating_add(value_or_default(&self.dispute))
            .saturating_add(value_or_default(&self.close_request))
            .saturating_add(value_or_default(&self.close_dispute))
    }
}

// Used primarily for testing purposes.
impl<AI, BA> Default for MarketBonds<AI, BA> {
    fn default() -> Self {
        MarketBonds {
            creation: None,
            oracle: None,
            outsider: None,
            dispute: None,
            close_request: None,
            close_dispute: None,
        }
    }
}

impl<AI, BA, BN, M, A, MI> MaxEncodedLen for Market<AI, BA, BN, M, A, MI>
where
    AI: MaxEncodedLen,
    BA: MaxEncodedLen,
    BN: MaxEncodedLen,
    M: MaxEncodedLen,
    A: MaxEncodedLen,
    MI: MaxEncodedLen,
{
    fn max_encoded_len() -> usize {
        AI::max_encoded_len()
            .saturating_add(MI::max_encoded_len())
            .saturating_add(A::max_encoded_len())
            .saturating_add(MarketCreation::max_encoded_len())
            .saturating_add(Perbill::max_encoded_len())
            .saturating_add(AI::max_encoded_len())
            // We assume that at max. a 512 bit hash function is used
            .saturating_add(u8::max_encoded_len().saturating_mul(68))
            .saturating_add(MarketType::max_encoded_len())
            .saturating_add(<MarketPeriod<BN, M>>::max_encoded_len())
            .saturating_add(Deadlines::<BN>::max_encoded_len())
            .saturating_add(ScoringRule::max_encoded_len())
            .saturating_add(MarketStatus::max_encoded_len())
            .saturating_add(<Option<Report<AI, BN>>>::max_encoded_len())
            .saturating_add(<Option<OutcomeReport>>::max_encoded_len())
            .saturating_add(<Option<MarketDisputeMechanism>>::max_encoded_len())
            .saturating_add(<MarketBonds<AI, BA>>::max_encoded_len())
            .saturating_add(<Option<EarlyClose<BN, M>>>::max_encoded_len())
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

/// Defines a global dispute item for the initialisation of a global dispute.
#[derive(Clone, Decode, Encode, MaxEncodedLen, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct GlobalDisputeItem<AccountId, Balance> {
    /// The account that already paid somehow for the outcome.
    pub owner: AccountId,
    /// The outcome that was already paid for
    /// and should be added as vote outcome inside global disputes.
    pub outcome: OutcomeReport,
    /// The initial amount added in the global dispute vote system initially for the outcome.
    pub initial_vote_amount: Balance,
}

#[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketDispute<AccountId, BlockNumber, Balance> {
    pub at: BlockNumber,
    pub by: AccountId,
    pub outcome: OutcomeReport,
    pub bond: Balance,
}

/// How a market should resolve disputes
#[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub enum MarketDisputeMechanism {
    Authorized,
    Court,
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
        BN::max_encoded_len().max(M::max_encoded_len()).saturating_mul(2).saturating_add(1)
    }
}

#[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct EarlyClose<BN, M> {
    pub old: MarketPeriod<BN, M>,
    pub new: MarketPeriod<BN, M>,
    pub state: EarlyCloseState,
}

#[derive(Clone, Decode, Encode, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
pub enum EarlyCloseState {
    ScheduledAsMarketCreator,
    ScheduledAsOther,
    Disputed,
    Rejected,
}

/// Defines deadlines for market.
#[derive(
    Clone, Copy, Decode, Default, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo,
)]
pub struct Deadlines<BN> {
    pub grace_period: BN,
    pub oracle_duration: BN,
    pub dispute_duration: BN,
}

#[derive(TypeInfo, Clone, Copy, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug)]
pub enum ScoringRule {
    AmmCdaHybrid,
    Parimutuel,
}

/// Defines the state of the market.
#[derive(Clone, Copy, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub enum MarketStatus {
    /// The market has been proposed and is either waiting for approval
    /// from the governing committee, or hasn't reach its delay yet.
    Proposed,
    /// Trading on the market is active.
    Active,
    /// Trading on the market has concluded.
    Closed,
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
        u128::max_encoded_len().saturating_mul(2).saturating_add(1)
    }
}

#[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Report<AccountId, BlockNumber> {
    pub at: BlockNumber,
    pub by: AccountId,
    pub outcome: OutcomeReport,
}

#[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AuthorityReport<BlockNumber> {
    pub resolve_at: BlockNumber,
    pub outcome: OutcomeReport,
}

pub enum ResolutionMechanism {
    RedeemTokens,
    Noop,
}

#[cfg(test)]
mod tests {
    use crate::{
        market::*,
        types::{Asset, MarketAsset},
    };
    use test_case::test_case;
    type MarketId = u128;
    type Market = crate::market::Market<u32, u32, u32, u32, Asset<MarketId>, MarketId>;

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
            market_id: 9,
            base_asset: Asset::Ztg,
            creator: 1,
            creation: MarketCreation::Permissionless,
            creator_fee: Default::default(),
            oracle: 3,
            metadata: vec![4u8; 5],
            market_type,
            period: MarketPeriod::Block(7..8),
            deadlines: Deadlines {
                grace_period: 1_u32,
                oracle_duration: 1_u32,
                dispute_duration: 1_u32,
            },
            scoring_rule: ScoringRule::AmmCdaHybrid,
            status: MarketStatus::Active,
            report: None,
            resolved_outcome: None,
            dispute_mechanism: Some(MarketDisputeMechanism::Authorized),
            bonds: MarketBonds::default(),
            early_close: None,
        };
        assert_eq!(market.matches_outcome_report(&outcome_report), expected);
    }

    #[test_case(
        MarketType::Categorical(2),
        ScoringRule::AmmCdaHybrid,
        vec![MarketAsset::CategoricalOutcome(0, 0), MarketAsset::CategoricalOutcome(0, 1)];
        "categorical_market_amm_cda_hybrid"
    )]
    #[test_case(
        MarketType::Categorical(2),
        ScoringRule::Parimutuel,
        vec![MarketAsset::ParimutuelShare(0, 0), MarketAsset::ParimutuelShare(0, 1)];
        "categorical_market_parimutuel"
    )]
    #[test_case(
        MarketType::Scalar(12..=34),
        ScoringRule::AmmCdaHybrid,
        vec![
            MarketAsset::ScalarOutcome(0, ScalarPosition::Long),
            MarketAsset::ScalarOutcome(0, ScalarPosition::Short),
        ];
        "scalar_market"
    )]
    fn provides_correct_list_of_assets(
        market_type: MarketType,
        scoring_rule: ScoringRule,
        expected: Vec<MarketAsset>,
    ) {
        let market = Market {
            market_id: 0,
            base_asset: Asset::Ztg,
            creator: 1,
            creation: MarketCreation::Permissionless,
            creator_fee: Default::default(),
            oracle: 3,
            metadata: vec![4u8; 5],
            market_type,
            period: MarketPeriod::Block(7..8),
            deadlines: Deadlines {
                grace_period: 1_u32,
                oracle_duration: 1_u32,
                dispute_duration: 1_u32,
            },
            scoring_rule,
            status: MarketStatus::Active,
            report: None,
            resolved_outcome: None,
            dispute_mechanism: Some(MarketDisputeMechanism::Authorized),
            bonds: MarketBonds::default(),
            early_close: None,
        };
        assert_eq!(market.outcome_assets(), expected);
    }

    #[test_case(
        MarketType::Categorical(2),
        ScoringRule::AmmCdaHybrid,
        OutcomeReport::Categorical(2),
        Some(MarketAsset::CategoricalOutcome(0, 2));
        "categorical_market_amm_cda_hybrid"
    )]
    #[test_case(
        MarketType::Categorical(2),
        ScoringRule::Parimutuel,
        OutcomeReport::Categorical(2),
        Some(MarketAsset::ParimutuelShare(0, 2));
        "categorical_market_parimutuel"
    )]
    #[test_case(
        MarketType::Scalar(12..=34),
        ScoringRule::AmmCdaHybrid,
        OutcomeReport::Scalar(2),
        None;
        "scalar_market"
    )]
    fn converts_outcome_correctly(
        market_type: MarketType,
        scoring_rule: ScoringRule,
        outcome: OutcomeReport,
        expected: Option<MarketAsset>,
    ) {
        let report = Some(Report {
            at: Default::default(),
            by: Default::default(),
            outcome: outcome.clone(),
        });

        let market = Market {
            market_id: 0,
            base_asset: Asset::Ztg,
            creator: 1,
            creation: MarketCreation::Permissionless,
            creator_fee: Default::default(),
            oracle: 3,
            metadata: vec![4u8; 5],
            market_type,
            period: MarketPeriod::Block(7..8),
            deadlines: Deadlines {
                grace_period: 1_u32,
                oracle_duration: 1_u32,
                dispute_duration: 1_u32,
            },
            scoring_rule,
            status: MarketStatus::Active,
            report,
            resolved_outcome: Some(outcome),
            dispute_mechanism: Some(MarketDisputeMechanism::Authorized),
            bonds: MarketBonds::default(),
            early_close: None,
        };
        assert_eq!(market.resolved_outcome_into_asset(), expected);
        assert_eq!(market.report_into_asset(), expected);
    }

    #[test]
    fn max_encoded_len_market_type() {
        // `MarketType::Scalar` is the largest enum variant.
        let market_type = MarketType::Scalar(1u128..=2);
        let len = parity_scale_codec::Encode::encode(&market_type).len();
        assert_eq!(MarketType::max_encoded_len(), len);
    }

    #[test]
    fn max_encoded_len_market_period() {
        let market_period: MarketPeriod<u32, u32> = MarketPeriod::Block(Default::default());
        let len = parity_scale_codec::Encode::encode(&market_period).len();
        assert_eq!(MarketPeriod::<u32, u32>::max_encoded_len(), len);
    }
}
