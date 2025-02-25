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
    constants::base_multiples::*,
    traits::{CombinatorialTokensFuel, MarketCommonsPalletApi},
    types::{Asset, MarketType},
};
use zrml_combinatorial_tokens::{
    mock::{
        ext_builder::ExtBuilder,
        runtime::{CombinatorialTokens, Runtime, RuntimeOrigin},
        types::MockPayout,
    },
    traits::CombinatorialIdManager,
    AccountIdOf, BalanceOf, CombinatorialIdOf, Config, FuelOf, MarketIdOf,
};

#[derive(Debug)]
struct RedeemPositionFuzzParams {
    account_id: AccountIdOf<Runtime>,
    parent_collection_id: Option<CombinatorialIdOf<Runtime>>,
    market_id: MarketIdOf<Runtime>,
    index_set: Vec<bool>,
    fuel: FuelOf<Runtime>,
    payout_vector: Option<Vec<BalanceOf<Runtime>>>,
    amount: BalanceOf<Runtime>,
}

impl<'a> Arbitrary<'a> for RedeemPositionFuzzParams {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let account_id = u128::arbitrary(u)?;
        let parent_collection_id = Arbitrary::arbitrary(u)?;
        let market_id = 0u8.into();
        let amount = Arbitrary::arbitrary(u)?;
        let fuel = FuelOf::<Runtime>::from_total(u.int_in_range(1..=100)?);

        let min_len = 2;
        let max_len = 1000;
        let len = u.int_in_range(0..=max_len)?;
        let index_set =
            (min_len..len).map(|_| bool::arbitrary(u)).collect::<ArbitraryResult<Vec<_>>>()?;

        // Clamp every value of the payout vector to [0..1]. That doesn't ensure that the payout
        // vector is valid, but it's valid enough to avoid most overflows.
        let payout_vector = Some(
            (min_len..len)
                .map(|_| Ok(u128::arbitrary(u)? % _1))
                .collect::<ArbitraryResult<Vec<_>>>()?,
        );

        let params = RedeemPositionFuzzParams {
            account_id,
            parent_collection_id,
            market_id,
            index_set,
            fuel,
            payout_vector,
            amount,
        };

        Ok(params)
    }
}

fuzz_target!(|params: RedeemPositionFuzzParams| {
    let mut ext = ExtBuilder::build();

    ext.execute_with(|| {
        // We create a market and equip the user with the tokens they require to make the
        // `redeem_position` call meaningful. We also provide the pallet account with collateral in
        // case it's required.
        let collateral = Asset::Ztg;
        let asset_count = params.index_set.len() as u16;
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

        // Is not required if `parent_collection_id.is_some()`, but we're doing it anyways.
        <<Runtime as Config>::MultiCurrency>::deposit(
            collateral,
            &CombinatorialTokens::account_id(),
            params.amount * asset_count as u128,
        )
        .unwrap();

        // Mock up the payout vector.
        MockPayout::set_return_value(params.payout_vector);

        let _ = CombinatorialTokens::redeem_position(
            RuntimeOrigin::signed(params.account_id),
            params.parent_collection_id,
            params.market_id,
            params.index_set,
            params.fuel,
        );
    });

    let _ = ext.commit_all();
});
