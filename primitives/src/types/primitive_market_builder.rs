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

use crate::{
    traits::MarketBuilder,
    types::{
        Deadlines, EarlyClose, Market, MarketBonds, MarketCreation, MarketDisputeMechanism,
        MarketPeriod, MarketStatus, MarketType, OutcomeReport, Report, ScoringRule,
    },
};
use alloc::vec::Vec;
use sp_runtime::{DispatchError, Perbill};

/// A sample market builder struct used for testing. No verification is done when calling `build()`;
/// use at your own risk!
///
/// Fields are deliberately kept public to allow the straightforward construction of builder objects
/// in spots where correctness isn't the primary concern.
///
/// # Generics
///
/// * `AI`: The account ID type.
/// * `BA`: The balance type.
/// * `BN`: The block number type.
/// * `M`: The moment/time type.
/// * `A`: The asset type.
/// * `MI`: The market ID type.
#[derive(Clone)]
pub struct PrimitiveMarketBuilder<AI, BA, BN, M, A, MI> {
    pub market_id: Option<MI>,
    pub base_asset: A,
    pub creator: AI,
    pub creation: MarketCreation,
    pub creator_fee: Perbill,
    pub oracle: AI,
    pub metadata: Vec<u8>,
    pub market_type: MarketType,
    pub period: MarketPeriod<BN, M>,
    pub deadlines: Deadlines<BN>,
    pub scoring_rule: ScoringRule,
    pub status: MarketStatus,
    pub report: Option<Report<AI, BN>>,
    pub resolved_outcome: Option<OutcomeReport>,
    pub dispute_mechanism: Option<MarketDisputeMechanism>,
    pub bonds: MarketBonds<AI, BA>,
    pub early_close: Option<EarlyClose<BN, M>>,
}

macro_rules! impl_builder_methods {
    ($($field:ident: $type:ty),*) => {
        $(
            fn $field(&mut self, $field: $type) -> &mut Self {
                self.$field = $field;
                self
            }
        )*
    }
}

impl<AI, BA, BN, M, A, MI> MarketBuilder<AI, BA, BN, M, A, MI>
    for PrimitiveMarketBuilder<AI, BA, BN, M, A, MI>
{
    fn build(self) -> Result<Market<AI, BA, BN, M, A, MI>, DispatchError> {
        Ok(Market {
            market_id: self.market_id.unwrap(),
            base_asset: self.base_asset,
            creator: self.creator,
            creation: self.creation,
            creator_fee: self.creator_fee,
            oracle: self.oracle,
            metadata: self.metadata,
            market_type: self.market_type,
            period: self.period,
            deadlines: self.deadlines,
            scoring_rule: self.scoring_rule,
            status: self.status,
            report: self.report,
            resolved_outcome: self.resolved_outcome,
            dispute_mechanism: self.dispute_mechanism,
            bonds: self.bonds,
            early_close: self.early_close,
        })
    }

    fn market_id(&mut self, market_id: MI) -> &mut Self {
        self.market_id = Some(market_id);
        self
    }

    impl_builder_methods! {
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
