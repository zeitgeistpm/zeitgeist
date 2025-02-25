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

#![no_main]

mod common;

use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};
use libfuzzer_sys::fuzz_target;
use orml_traits::currency::MultiCurrency;
use zeitgeist_primitives::{
    traits::{CombinatorialTokensFuel, MarketCommonsPalletApi},
    types::{Asset, MarketType},
};
use zrml_combinatorial_tokens::{
    mock::{
        ext_builder::ExtBuilder,
        runtime::{CombinatorialTokens, Runtime, RuntimeOrigin},
    },
    traits::CombinatorialIdManager,
    AccountIdOf, BalanceOf, CombinatorialIdOf, Config, FuelOf, MarketIdOf,
};

#[derive(Debug)]
struct SplitPositionFuzzParams {
    account_id: AccountIdOf<Runtime>,
    parent_collection_id: Option<CombinatorialIdOf<Runtime>>,
    market_id: MarketIdOf<Runtime>,
    partition: Vec<Vec<bool>>,
    amount: BalanceOf<Runtime>,
    fuel: FuelOf<Runtime>,
}

impl<'a> Arbitrary<'a> for SplitPositionFuzzParams {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let account_id = u128::arbitrary(u)?;
        let parent_collection_id = Arbitrary::arbitrary(u)?;
        let market_id = 0u8.into();
        let amount = Arbitrary::arbitrary(u)?;
        let fuel = FuelOf::<Runtime>::from_total(u.int_in_range(1..=100)?);

        // Note: This might result in members of unequal length, but that's OK.
        let min_len = 0;
        let max_len = 10;
        let len = u.int_in_range(0..=max_len)?;
        let partition =
            (min_len..len).map(|_| Arbitrary::arbitrary(u)).collect::<ArbitraryResult<Vec<_>>>()?;

        let params = SplitPositionFuzzParams {
            account_id,
            parent_collection_id,
            market_id,
            partition,
            amount,
            fuel,
        };

        Ok(params)
    }
}

fuzz_target!(|params: SplitPositionFuzzParams| {
    let mut ext = ExtBuilder::build();

    ext.execute_with(|| {
        // We create a market and equip the user with the tokens they require to make the
        // `split_position` call meaningful.
        let collateral = Asset::Ztg;
        let asset_count = if let Some(member) = params.partition.first() {
            member.len().max(2) as u16
        } else {
            2u16 // In this case the index set doesn't fit the market.
        };
        let market = common::market::<Runtime>(
            params.market_id,
            collateral,
            MarketType::Categorical(asset_count),
        );
        <<Runtime as Config>::MarketCommons as MarketCommonsPalletApi>::push_market(market)
            .unwrap();

        let position = if let Some(pci) = params.parent_collection_id {
            let position_id =
                <Runtime as Config>::CombinatorialIdManager::get_position_id(collateral, pci);

            Asset::CombinatorialToken(position_id)
        } else {
            Asset::Ztg
        };
        <<Runtime as Config>::MultiCurrency>::deposit(position, &params.account_id, params.amount)
            .unwrap();

        let _ = CombinatorialTokens::split_position(
            RuntimeOrigin::signed(params.account_id),
            params.parent_collection_id,
            params.market_id,
            params.partition,
            params.amount,
            params.fuel,
        );
    });

    let _ = ext.commit_all();
});
