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

use crate::{mock::*, types::*, utils::*, AccountIdOf, BalanceOf, MarketIdOf, *};
use frame_support::{assert_noop, assert_ok, traits::fungible::Mutate};
use orml_currencies::Error as CurrenciesError;
use orml_tokens::Error as TokensError;
use orml_traits::MultiCurrency;
use sp_runtime::{Perbill, SaturatedConversion};
use zeitgeist_primitives::{
    constants::{base_multiples::*, BASE, CENT},
    orderbook::Order,
    types::{
        AccountIdTest, Asset, Deadlines, MarketCreation, MarketId, MarketPeriod, MarketStatus,
        MarketType, MultiHash, ScoringRule,
    },
};
use zrml_market_commons::{Error as MError, MarketCommonsPalletApi, Markets};
use zrml_neo_swaps::Event as NeoSwapsEvent;
use zrml_orderbook::Orders;

mod buy;
mod sell;

#[cfg(not(feature = "parachain"))]
const BASE_ASSET: Asset<MarketId> = Asset::Ztg;
#[cfg(feature = "parachain")]
const BASE_ASSET: Asset<MarketId> = FOREIGN_ASSET;

fn create_market(
    creator: AccountIdTest,
    base_asset: AssetOf<Runtime>,
    market_type: MarketType,
    scoring_rule: ScoringRule,
) -> MarketIdOf<Runtime> {
    let mut metadata = [2u8; 50];
    metadata[0] = 0x15;
    metadata[1] = 0x30;
    assert_ok!(PredictionMarkets::create_market(
        RuntimeOrigin::signed(creator),
        base_asset,
        Perbill::zero(),
        EVE,
        MarketPeriod::Block(0..2),
        Deadlines {
            grace_period: 0_u32.into(),
            oracle_duration: <Runtime as zrml_prediction_markets::Config>::MinOracleDuration::get(),
            dispute_duration: 0_u32.into(),
        },
        MultiHash::Sha3_384(metadata),
        MarketCreation::Permissionless,
        market_type,
        None,
        scoring_rule,
    ));
    MarketCommons::latest_market_id().unwrap()
}

fn create_market_and_deploy_pool(
    creator: AccountIdOf<Runtime>,
    base_asset: AssetOf<Runtime>,
    market_type: MarketType,
    amount: BalanceOf<Runtime>,
    spot_prices: Vec<BalanceOf<Runtime>>,
    swap_fee: BalanceOf<Runtime>,
) -> MarketIdOf<Runtime> {
    let market_id = create_market(creator, base_asset, market_type, ScoringRule::AmmCdaHybrid);
    assert_ok!(PredictionMarkets::buy_complete_set(
        RuntimeOrigin::signed(ALICE),
        market_id,
        amount,
    ));
    assert_ok!(NeoSwaps::deploy_pool(
        RuntimeOrigin::signed(ALICE),
        market_id,
        amount,
        spot_prices.clone(),
        swap_fee,
    ));
    market_id
}
