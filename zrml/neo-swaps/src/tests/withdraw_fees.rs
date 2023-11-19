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
fn withdraw_fees_works() {
    // Verify that fees are correctly distributed among LPs.
    ExtBuilder::default().build().execute_with(|| {
        let category_count = 2;
        let spot_prices = vec![_3_4, _1_4];
        let liquidity_parameter = 288539008176;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(category_count),
            _10,
            spot_prices.clone(),
            CENT,
        );
        let join = |who: AccountIdOf<Runtime>, amount: BalanceOf<Runtime>| {
            // Adding a little more to ensure that rounding doesn't cause issues.
            deposit_complete_set(market_id, who, amount + CENT);
            assert_ok!(NeoSwaps::join(
                RuntimeOrigin::signed(who),
                market_id,
                amount,
                vec![u128::MAX; category_count as usize],
            ));
        };
        join(BOB, _10);
        join(CHARLIE, _20);

        // Mock up some fees.
        let mut pool = Pools::<Runtime>::get(market_id).unwrap();
        let fee_amount = _1;
        assert_ok!(AssetManager::deposit(pool.collateral, &pool.account_id, fee_amount));
        assert_ok!(pool.liquidity_shares_manager.deposit_fees(fee_amount));
        Pools::<Runtime>::insert(market_id, pool.clone());
        let pool_balances = [83007499856, 400000000000];

        let test_withdraw = |who: AccountIdOf<Runtime>| -> BalanceOf<Runtime> {
            // Make sure everybody's got at least the minimum deposit.
            assert_ok!(<Runtime as Config>::MultiCurrency::deposit(
                BASE_ASSET,
                &who,
                <Runtime as Config>::MultiCurrency::minimum_balance(BASE_ASSET)
            ));
            let balance = <Runtime as Config>::MultiCurrency::free_balance(BASE_ASSET, &who);
            assert_ok!(NeoSwaps::withdraw_fees(RuntimeOrigin::signed(who), market_id));
            balance
        };

        let alice_balance = test_withdraw(ALICE);
        let alice_fees = _1 / 4;
        assert_balance!(ALICE, BASE_ASSET, alice_balance + alice_fees);
        assert_pool_status!(
            market_id,
            pool_balances,
            spot_prices,
            liquidity_parameter,
            create_b_tree_map!({ ALICE => _10, BOB => _10, CHARLIE => _20 })
        );
        System::assert_last_event(
            Event::FeesWithdrawn { who: ALICE, market_id, amount: alice_fees }.into(),
        );

        let bob_balance = test_withdraw(BOB);
        let bob_fees = _1 / 4;
        assert_balance!(BOB, BASE_ASSET, bob_balance + bob_fees);
        assert_pool_status!(
            market_id,
            pool_balances,
            spot_prices,
            liquidity_parameter,
            create_b_tree_map!({ ALICE => _10, BOB => _10, CHARLIE => _20 })
        );
        System::assert_last_event(
            Event::FeesWithdrawn { who: BOB, market_id, amount: bob_fees }.into(),
        );

        let charlie_balance = test_withdraw(CHARLIE);
        let charlie_fees = _1 / 2;
        assert_balance!(CHARLIE, BASE_ASSET, charlie_balance + charlie_fees);
        assert_pool_status!(
            market_id,
            pool_balances,
            spot_prices,
            liquidity_parameter,
            create_b_tree_map!({ ALICE => _10, BOB => _10, CHARLIE => _20 })
        );
        System::assert_last_event(
            Event::FeesWithdrawn { who: CHARLIE, market_id, amount: charlie_fees }.into(),
        );
    });
}

#[test]
fn withdraw_fees_fails_on_pool_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::Lmsr);
        assert_noop!(
            NeoSwaps::withdraw_fees(RuntimeOrigin::signed(ALICE), market_id),
            Error::<Runtime>::PoolNotFound
        );
    });
}

// TODO withdraw fees is noop if there are no fees
