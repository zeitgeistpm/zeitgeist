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

#![cfg(all(feature = "mock", test))]

use crate::{AccountIdOf, BalanceOf, MarketIdOf};
use frame_system::pallet_prelude::BlockNumberFor;
use zeitgeist_primitives::{
    traits::MarketCommonsPalletApi,
    types::{BaseAsset, Market},
};

pub(crate) type MomentOf<T> =
    <<T as crate::Config>::MarketCommons as MarketCommonsPalletApi>::Moment;
pub(crate) type MarketOf<T> =
    Market<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, MomentOf<T>, BaseAsset, MarketIdOf<T>>;

pub(crate) fn market_mock<T>(creator: T::AccountId) -> MarketOf<T>
where
    T: crate::Config,
{
    use sp_runtime::{traits::AccountIdConversion, Perbill};
    use zeitgeist_primitives::{
        constants::mock::PmPalletId,
        types::{
            BaseAssetClass, Deadlines, MarketBonds, MarketCreation, MarketDisputeMechanism,
            MarketPeriod, MarketStatus, MarketType, ScoringRule,
        },
    };

    zeitgeist_primitives::types::Market {
        market_id: 0u8.into(),
        base_asset: BaseAssetClass::Ztg,
        creation: MarketCreation::Permissionless,
        creator_fee: Perbill::zero(),
        creator,
        market_type: MarketType::Categorical(10u16),
        dispute_mechanism: Some(MarketDisputeMechanism::Court),
        metadata: Default::default(),
        oracle: PmPalletId::get().into_account_truncating(),
        period: MarketPeriod::Block(Default::default()),
        deadlines: Deadlines {
            grace_period: 1_u32.into(),
            oracle_duration: 1_u32.into(),
            dispute_duration: 1_u32.into(),
        },
        report: None,
        resolved_outcome: None,
        scoring_rule: ScoringRule::AmmCdaHybrid,
        status: MarketStatus::Active,
        bonds: MarketBonds::default(),
        early_close: None,
    }
}
