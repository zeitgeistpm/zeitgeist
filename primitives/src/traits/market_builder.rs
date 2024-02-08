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

use crate::types::{
    Deadlines, EarlyClose, Market, MarketBonds, MarketCreation, MarketDisputeMechanism,
    MarketPeriod, MarketStatus, MarketType, OutcomeReport, Report, ScoringRule,
};
use alloc::vec::Vec;
use sp_runtime::{DispatchError, Perbill};

macro_rules! builder_methods {
    ($($field:ident: $type:ty),* $(,)?) => {
        $(fn $field(&mut self, $field: $type) -> &mut Self;)*
    }
}

/// Mutably referenced builder struct for the `Market` object. The `build` call is pass-by-value, so
/// the usual calling pattern is:
///
/// ```ignore
/// let builder = MarketBuilderImpl::new();
/// builder.field1(value1).field2(value2);
/// builder.clone().build()
/// ```
pub trait MarketBuilderTrait<AI, BA, BN, M, A, MI> {
    fn build(self) -> Result<Market<AI, BA, BN, M, A, MI>, DispatchError>;

    builder_methods! {
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
        early_close: Option<EarlyClose<BN, M>>,
    }
}
