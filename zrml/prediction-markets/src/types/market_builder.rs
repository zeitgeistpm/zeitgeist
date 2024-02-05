// Copyright 2024 Forecasting Technologies LTD.
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

use alloc::vec::Vec;
use sp_runtime::Perbill;
use zeitgeist_primitives::{
    traits::MarketBuilder,
    types::{
        Deadlines, EarlyClose, Market, MarketBonds, MarketCreation, MarketDisputeMechanism,
        MarketPeriod, MarketStatus, MarketType, OutcomeReport, Report, ScoringRule,
    },
};

/// # Generics
///
/// * `AI`: The account ID type.
/// * `BA`: The balance type.
/// * `BN`: The block number type.
/// * `M`: The moment/time type.
/// * `A`: The asset type.
/// * `MI`: The market ID type.
#[derive(Clone)]
pub struct PredictionMarketBuilder<AI, BA, BN, M, A, MI> {
    pub market_id: Option<MI>,
    pub base_asset: Option<A>,
    pub creator: Option<AI>,
    pub creation: Option<MarketCreation>,
    pub creator_fee: Option<Perbill>,
    pub oracle: Option<AI>,
    pub metadata: Option<Vec<u8>>,
    pub market_type: Option<MarketType>,
    pub period: Option<MarketPeriod<BN, M>>,
    pub deadlines: Option<Deadlines<BN>>,
    pub scoring_rule: Option<ScoringRule>,
    pub status: Option<MarketStatus>,
    pub report: Option<Option<Report<AI, BN>>>,
    pub resolved_outcome: Option<Option<OutcomeReport>>,
    pub dispute_mechanism: Option<Option<MarketDisputeMechanism>>,
    pub bonds: Option<MarketBonds<AI, BA>>,
    pub early_close: Option<Option<EarlyClose<BN, M>>>,
}

impl<AI, BA, BN, M, A, MI> PredictionMarketBuilder<AI, BA, BN, M, A, MI> {
    pub(crate) fn new() -> Self {
        PredictionMarketBuilder {
            market_id: None,
            base_asset: None,
            creator: None,
            creation: None,
            creator_fee: None,
            oracle: None,
            metadata: None,
            market_type: None,
            period: None,
            deadlines: None,
            scoring_rule: None,
            status: None,
            report: None,
            resolved_outcome: None,
            dispute_mechanism: None,
            bonds: None,
            early_close: None,
        }
    }
}

macro_rules! impl_builder_methods {
    ($($field:ident: $type:ty),*) => {
        $(
            fn $field(&mut self, $field: $type) -> &mut Self {
                self.$field = Some($field);
                self
            }
        )*
    }
}

impl<AI, BA, BN, M, A, MI> MarketBuilder<AI, BA, BN, M, A, MI>
    for PredictionMarketBuilder<AI, BA, BN, M, A, MI>
{
    fn build(self) -> Market<AI, BA, BN, M, A, MI> {
        // TODO Remove unwraps, make build return result
        Market {
            market_id: self.market_id.unwrap(),
            base_asset: self.base_asset.unwrap(),
            creator: self.creator.unwrap(),
            creation: self.creation.unwrap(),
            creator_fee: self.creator_fee.unwrap(),
            oracle: self.oracle.unwrap(),
            metadata: self.metadata.unwrap(),
            market_type: self.market_type.unwrap(),
            period: self.period.unwrap(),
            deadlines: self.deadlines.unwrap(),
            scoring_rule: self.scoring_rule.unwrap(),
            status: self.status.unwrap(),
            report: self.report.unwrap(),
            resolved_outcome: self.resolved_outcome.unwrap(),
            dispute_mechanism: self.dispute_mechanism.unwrap(),
            bonds: self.bonds.unwrap(),
            early_close: self.early_close.unwrap(),
        }
    }

    impl_builder_methods! {
        market_id: MI,
        base_asset: A,
        creator: AI,
        creation: MarketCreation,
        creator_fee: Perbill,
        oracle: AI,
        metadata: Vec<u8>,
        market_type: MarketType,
        period: MarketPeriod<BN, M>,
        deadlines: Deadlines<BN>,
        scoring_rule: ScoringRule,
        status: MarketStatus,
        report: Option<Report<AI, BN>>,
        resolved_outcome: Option<OutcomeReport>,
        dispute_mechanism: Option<MarketDisputeMechanism>,
        bonds: MarketBonds<AI, BA>,
        early_close: Option<EarlyClose<BN, M>>
    }
}
