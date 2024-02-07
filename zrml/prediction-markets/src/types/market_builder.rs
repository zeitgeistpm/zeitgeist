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
    AccountIdOf, AssetOf, BalanceOf, BlockNumberOf, Config, DeadlinesOf, EarlyCloseOf,
    MarketBondsOf, MarketIdOf, MarketOf, MarketPeriodOf, MomentOf, ReportOf,
};
use alloc::vec::Vec;
use sp_runtime::{DispatchError, Perbill};
use zeitgeist_primitives::{
    traits::MarketBuilder,
    types::{
        Deadlines, EarlyClose, Market, MarketBonds, MarketCreation, MarketDisputeMechanism,
        MarketPeriod, MarketStatus, MarketType, OutcomeReport, Report, ScoringRule,
    },
};

/// Fully-fledged mutably referenced market builder struct.
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
pub struct PredictionMarketBuilder<T>
where
    T: Config,
{
    pub market_id: Option<MarketIdOf<T>>,
    pub base_asset: Option<AssetOf<T>>,
    pub creator: Option<AccountIdOf<T>>,
    pub creation: Option<MarketCreation>,
    pub creator_fee: Option<Perbill>,
    pub oracle: Option<AccountIdOf<T>>,
    pub metadata: Option<Vec<u8>>,
    pub market_type: Option<MarketType>,
    pub period: Option<MarketPeriodOf<T>>,
    pub deadlines: Option<DeadlinesOf<T>>,
    pub scoring_rule: Option<ScoringRule>,
    pub status: Option<MarketStatus>,
    pub report: Option<Option<ReportOf<T>>>,
    pub resolved_outcome: Option<Option<OutcomeReport>>,
    pub dispute_mechanism: Option<Option<MarketDisputeMechanism>>,
    pub bonds: Option<MarketBondsOf<T>>,
    pub early_close: Option<Option<EarlyCloseOf<T>>>,
}

impl<T> PredictionMarketBuilder<T>
where
    T: Config,
{
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

impl<T>
    MarketBuilder<
        AccountIdOf<T>,
        BalanceOf<T>,
        BlockNumberOf<T>,
        MomentOf<T>,
        AssetOf<T>,
        MarketIdOf<T>,
    > for PredictionMarketBuilder<T>
where
    T: Config,
{
    fn build(self) -> Result<MarketOf<T>, DispatchError> {
        // TODO Remove unwraps, make build return result
        Ok(Market {
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
        })
    }

    impl_builder_methods! {
        market_id: MarketIdOf<T>,
        base_asset: AssetOf<T>,
        creator: AccountIdOf<T>,
        creation: MarketCreation,
        creator_fee: Perbill,
        oracle: AccountIdOf<T>,
        metadata: Vec<u8>,
        market_type: MarketType,
        period: MarketPeriodOf<T>,
        deadlines: DeadlinesOf<T>,
        scoring_rule: ScoringRule,
        status: MarketStatus,
        report: Option<ReportOf<T>>,
        resolved_outcome: Option<OutcomeReport>,
        dispute_mechanism: Option<MarketDisputeMechanism>,
        bonds: MarketBondsOf<T>,
        early_close: Option<EarlyCloseOf<T>>
    }
}
