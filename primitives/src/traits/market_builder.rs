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
    Asset, Deadlines, EarlyClose, Market, MarketBonds, MarketCreation, MarketDisputeMechanism,
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
pub trait MarketBuilderTrait<AccountId, Balance, BlockNumber, Moment, MarketId> {
    fn build(
        self,
    ) -> Result<Market<AccountId, Balance, BlockNumber, Moment, MarketId>, DispatchError>;

    builder_methods! {
        market_id: MarketId,
        base_asset: Asset<MarketId>,
        creator: AccountId,
        creation: MarketCreation,
        creator_fee: Perbill,
        oracle: AccountId,
        metadata: Vec<u8>,
        market_type: MarketType,
        period: MarketPeriod<BlockNumber, Moment>,
        deadlines: Deadlines<BlockNumber>,
        scoring_rule: ScoringRule,
        status: MarketStatus,
        report: Option<Report<AccountId, BlockNumber>>,
        resolved_outcome: Option<OutcomeReport>,
        dispute_mechanism: Option<MarketDisputeMechanism>,
        bonds: MarketBonds<AccountId, Balance>,
        early_close: Option<EarlyClose<BlockNumber, Moment>>,
    }
}
