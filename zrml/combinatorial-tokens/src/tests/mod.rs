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

#![cfg(all(feature = "mock", test))]

mod integration;
mod merge_position;
mod redeem_position;
mod split_position;

use crate::{
    mock::{
        ext_builder::ExtBuilder,
        runtime::{CombinatorialTokens, Currencies, MarketCommons, Runtime, RuntimeOrigin, System},
        types::MockPayout,
    },
    types::cryptographic_id_manager::Fuel,
    Error, Event, Pallet,
};
use frame_support::{assert_noop, assert_ok};
use orml_traits::MultiCurrency;
use sp_runtime::{DispatchError, Perbill};
use zeitgeist_primitives::{
    constants::base_multiples::*,
    types::{
        AccountIdTest, Asset, Asset::CombinatorialToken, Balance, Market, MarketBonds,
        MarketCreation, MarketId, MarketPeriod, MarketStatus, MarketType, ScoringRule,
    },
};
use zrml_market_commons::MarketCommonsPalletApi;

// For better readability of index sets.
pub(crate) const B0: bool = false;
pub(crate) const B1: bool = true;

fn create_market(base_asset: Asset<MarketId>, market_type: MarketType) -> MarketId {
    let market = Market {
        base_asset,
        market_id: Default::default(),
        creation: MarketCreation::Permissionless,
        creator_fee: Perbill::zero(),
        creator: Default::default(),
        market_type,
        dispute_mechanism: None,
        metadata: Default::default(),
        oracle: Default::default(),
        period: MarketPeriod::Block(Default::default()),
        deadlines: Default::default(),
        report: None,
        resolved_outcome: None,
        scoring_rule: ScoringRule::AmmCdaHybrid,
        status: MarketStatus::Disputed,
        bonds: MarketBonds::default(),
        early_close: None,
    };
    MarketCommons::push_market(market).unwrap();
    MarketCommons::latest_market_id().unwrap()
}

/// Utility struct for managing test accounts.
pub(crate) struct Account {
    id: AccountIdTest,
}

impl Account {
    // TODO Not a pressing issue, but double booking accounts should be illegal.
    pub(crate) fn new(id: AccountIdTest) -> Account {
        Account { id }
    }

    /// Deposits `amount` of `asset` and returns the account to allow call chains.
    pub(crate) fn deposit(
        self,
        asset: Asset<MarketId>,
        amount: Balance,
    ) -> Result<Account, DispatchError> {
        Currencies::deposit(asset, &self.id, amount).map(|_| self)
    }

    pub(crate) fn signed(&self) -> RuntimeOrigin {
        RuntimeOrigin::signed(self.id)
    }

    pub(crate) fn free_balance(&self, asset: Asset<MarketId>) -> Balance {
        Currencies::free_balance(asset, &self.id)
    }
}
