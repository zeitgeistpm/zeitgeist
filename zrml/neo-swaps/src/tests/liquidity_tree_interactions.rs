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

use super::*;

#[test]
fn withdraw_fees_interacts_correctly_with_join() {
    ExtBuilder::default().build().execute_with(|| {
        let category_count = 2;
        let spot_prices = vec![_3_4, _1_4];
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(category_count),
            _10,
            spot_prices.clone(),
            CENT,
        );

        // Mock up some fees.
        let mut pool = Pools::<Runtime>::get(market_id).unwrap();
        let fee_amount = _1;
        assert_ok!(AssetManager::deposit(pool.collateral, &pool.account_id, fee_amount));
        assert_ok!(pool.liquidity_shares_manager.deposit_fees(fee_amount));
        Pools::<Runtime>::insert(market_id, pool.clone());

        // Bob joins the pool after fees are distributed.
        let join_amount = _10;
        deposit_complete_set(market_id, BOB, join_amount + CENT);
        assert_ok!(NeoSwaps::join(
            RuntimeOrigin::signed(BOB),
            market_id,
            join_amount,
            vec![u128::MAX; category_count as usize],
        ));

        // Alice withdraws and should receive all fees.
        let old_balance = <Runtime as Config>::MultiCurrency::free_balance(BASE_ASSET, &ALICE);
        assert_ok!(NeoSwaps::withdraw_fees(RuntimeOrigin::signed(ALICE), market_id));
        assert_balance!(ALICE, BASE_ASSET, old_balance + fee_amount);
        assert_ok!(NeoSwaps::withdraw_fees(RuntimeOrigin::signed(BOB), market_id));
        assert_balance!(BOB, BASE_ASSET, 0);
    });
}
