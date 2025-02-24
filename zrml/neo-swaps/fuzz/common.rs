// Copyright 2024-2025 Forecasting Technologies LTD.
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

use zeitgeist_primitives::{
    traits::MarketOf,
    types::{Market, MarketCreation, MarketPeriod, MarketStatus, MarketType, ScoringRule},
};
use zrml_neo_swaps::{AssetOf, Config, MarketIdOf};

pub(crate) fn market<T>(
    market_id: MarketIdOf<T>,
    base_asset: AssetOf<T>,
    market_type: MarketType,
) -> MarketOf<<T as Config>::MarketCommons>
where
    T: Config,
    <T as frame_system::Config>::AccountId: Default,
{
    Market {
        market_id,
        base_asset,
        creator: Default::default(),
        creation: MarketCreation::Permissionless,
        creator_fee: Default::default(),
        oracle: Default::default(),
        metadata: Default::default(),
        market_type,
        period: MarketPeriod::Block(0u8.into()..10u8.into()),
        deadlines: Default::default(),
        scoring_rule: ScoringRule::AmmCdaHybrid,
        status: MarketStatus::Active,
        report: None,
        resolved_outcome: None,
        dispute_mechanism: None,
        bonds: Default::default(),
        early_close: None,
    }
}
