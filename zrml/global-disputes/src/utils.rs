// Copyright 2023-2024 Forecasting Technologies LTD.
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

type MarketOf<T> = zeitgeist_primitives::types::Market<
    <T as frame_system::Config>::AccountId,
    BalanceOf<T>,
    <T as frame_system::Config>::BlockNumber,
    MomentOf<T>,
    zeitgeist_primitives::types::BaseAsset,
    MarketIdOf<T>,
>;

pub(crate) fn market_mock<T>() -> MarketOf<T>
where
    T: crate::Config,
{
    use frame_support::traits::Get;
    use sp_runtime::traits::AccountIdConversion;
    use zeitgeist_primitives::types::ScoringRule;

    zeitgeist_primitives::types::Market {
        market_id: Default::default(),
        base_asset: zeitgeist_primitives::types::BaseAsset::Ztg,
        creation: zeitgeist_primitives::types::MarketCreation::Permissionless,
        creator_fee: sp_runtime::Perbill::zero(),
        creator: T::GlobalDisputesPalletId::get().into_account_truncating(),
        market_type: zeitgeist_primitives::types::MarketType::Scalar(0..=u128::MAX),
        dispute_mechanism: Some(
            zeitgeist_primitives::types::MarketDisputeMechanism::SimpleDisputes,
        ),
        metadata: Default::default(),
        oracle: T::GlobalDisputesPalletId::get().into_account_truncating(),
        period: zeitgeist_primitives::types::MarketPeriod::Block(Default::default()),
        deadlines: zeitgeist_primitives::types::Deadlines {
            grace_period: 1_u32.into(),
            oracle_duration: 1_u32.into(),
            dispute_duration: 1_u32.into(),
        },
        report: None,
        resolved_outcome: None,
        scoring_rule: ScoringRule::AmmCdaHybrid,
        status: zeitgeist_primitives::types::MarketStatus::Disputed,
        bonds: Default::default(),
        early_close: None,
    }
}
