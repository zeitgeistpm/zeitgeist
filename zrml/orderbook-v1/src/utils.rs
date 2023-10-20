// Copyright 2023 Forecasting Technologies LTD.
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

#![cfg(any(feature = "runtime-benchmarks", test))]

use crate::*;
use sp_runtime::traits::AccountIdConversion;
use zeitgeist_primitives::types::{
    Asset, Deadlines, Market, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus,
    MarketType, ScoringRule,
};

type MarketOf<T> = Market<
    <T as frame_system::Config>::AccountId,
    BalanceOf<T>,
    <T as frame_system::Config>::BlockNumber,
    MomentOf<T>,
    Asset<MarketIdOf<T>>,
>;

pub(crate) fn market_mock<T>() -> MarketOf<T>
where
    T: crate::Config,
{
    Market {
        base_asset: Asset::Ztg,
        creation: MarketCreation::Permissionless,
        creator_fee: sp_runtime::Perbill::zero(),
        creator: T::PalletId::get().into_account_truncating(),
        market_type: MarketType::Categorical(64u16),
        dispute_mechanism: Some(MarketDisputeMechanism::Authorized),
        metadata: Default::default(),
        oracle: T::PalletId::get().into_account_truncating(),
        period: MarketPeriod::Block(Default::default()),
        deadlines: Deadlines {
            grace_period: 1_u32.into(),
            oracle_duration: 1_u32.into(),
            dispute_duration: 1_u32.into(),
        },
        report: None,
        resolved_outcome: None,
        scoring_rule: ScoringRule::Orderbook,
        status: MarketStatus::Active,
        bonds: Default::default(),
    }
}
