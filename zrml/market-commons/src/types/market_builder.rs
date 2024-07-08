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
    AccountIdOf, AssetOf, BalanceOf, Config, DeadlinesOf, EarlyCloseOf, Error, MarketBondsOf,
    MarketIdOf, MarketOf, MarketPeriodOf, MomentOf, ReportOf,
};
use alloc::vec::Vec;
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{DispatchError, Perbill};
use zeitgeist_primitives::{
    traits::MarketBuilderTrait,
    types::{
        Market, MarketCreation, MarketDisputeMechanism, MarketStatus, MarketType, OutcomeReport,
        ScoringRule,
    },
};

/// Fully-fledged mutably referenced builder struct for `Market`.
#[derive(Clone)]
pub struct MarketBuilder<T>
where
    T: Config,
{
    market_id: Option<MarketIdOf<T>>,
    base_asset: Option<AssetOf<T>>,
    creator: Option<AccountIdOf<T>>,
    creation: Option<MarketCreation>,
    creator_fee: Option<Perbill>,
    oracle: Option<AccountIdOf<T>>,
    metadata: Option<Vec<u8>>,
    market_type: Option<MarketType>,
    period: Option<MarketPeriodOf<T>>,
    deadlines: Option<DeadlinesOf<T>>,
    scoring_rule: Option<ScoringRule>,
    status: Option<MarketStatus>,
    report: Option<Option<ReportOf<T>>>,
    resolved_outcome: Option<Option<OutcomeReport>>,
    dispute_mechanism: Option<Option<MarketDisputeMechanism>>,
    bonds: Option<MarketBondsOf<T>>,
    early_close: Option<Option<EarlyCloseOf<T>>>,
}

impl<T> MarketBuilder<T>
where
    T: Config,
{
    pub fn new() -> Self {
        MarketBuilder {
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

impl<T> Default for MarketBuilder<T>
where
    T: Config,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Implements setter methods for a mutably referenced builder struct. Fields are specified using
/// the pattern `{ field: type, ... }`.
macro_rules! impl_builder_methods {
    ($($field:ident: $type:ty),* $(,)?) => {
        $(
            fn $field(&mut self, $field: $type) -> &mut Self {
                self.$field = Some($field);
                self
            }
        )*
    }
}

/// Unwraps `opt` and throws `IncompleteMarketBuilder` in case of failure.
fn ok_or_incomplete<T, U>(opt: Option<U>) -> Result<U, DispatchError>
where
    T: Config,
{
    opt.ok_or(Error::<T>::IncompleteMarketBuilder.into())
}

impl<T>
    MarketBuilderTrait<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, MomentOf<T>, MarketIdOf<T>>
    for MarketBuilder<T>
where
    T: Config,
{
    fn build(self) -> Result<MarketOf<T>, DispatchError> {
        Ok(Market {
            market_id: ok_or_incomplete::<T, _>(self.market_id)?,
            base_asset: ok_or_incomplete::<T, _>(self.base_asset)?,
            creator: ok_or_incomplete::<T, _>(self.creator)?,
            creation: ok_or_incomplete::<T, _>(self.creation)?,
            creator_fee: ok_or_incomplete::<T, _>(self.creator_fee)?,
            oracle: ok_or_incomplete::<T, _>(self.oracle)?,
            metadata: ok_or_incomplete::<T, _>(self.metadata)?,
            market_type: ok_or_incomplete::<T, _>(self.market_type)?,
            period: ok_or_incomplete::<T, _>(self.period)?,
            deadlines: ok_or_incomplete::<T, _>(self.deadlines)?,
            scoring_rule: ok_or_incomplete::<T, _>(self.scoring_rule)?,
            status: ok_or_incomplete::<T, _>(self.status)?,
            report: ok_or_incomplete::<T, _>(self.report)?,
            resolved_outcome: ok_or_incomplete::<T, _>(self.resolved_outcome)?,
            dispute_mechanism: ok_or_incomplete::<T, _>(self.dispute_mechanism)?,
            bonds: ok_or_incomplete::<T, _>(self.bonds)?,
            early_close: ok_or_incomplete::<T, _>(self.early_close)?,
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
        early_close: Option<EarlyCloseOf<T>>,
    }
}
