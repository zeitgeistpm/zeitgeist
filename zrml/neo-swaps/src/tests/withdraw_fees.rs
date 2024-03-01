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
        let liquidity_parameter = 288_539_008_176;
        let pool_balances = [83_007_499_856, 400_000_000_000];

        let test_withdraw = |who: AccountIdOf<Runtime>,
                             fees_withdrawn: BalanceOf<Runtime>,
                             fees_remaining: BalanceOf<Runtime>| {
            // Make sure everybody's got at least the minimum deposit.
            assert_ok!(<Runtime as Config>::MultiCurrency::deposit(
                BASE_ASSET,
                &who,
                <Runtime as Config>::MultiCurrency::minimum_balance(BASE_ASSET)
            ));
            let old_balance = <Runtime as Config>::MultiCurrency::free_balance(BASE_ASSET, &who);
            assert_ok!(NeoSwaps::withdraw_fees(RuntimeOrigin::signed(who), market_id));
            assert_balance!(who, BASE_ASSET, old_balance + fees_withdrawn);
            assert_pool_state!(
                market_id,
                pool_balances,
                spot_prices,
                liquidity_parameter,
                create_b_tree_map!({ ALICE => _10, BOB => _10, CHARLIE => _20 }),
                fees_remaining,
            );
            System::assert_last_event(
                Event::FeesWithdrawn { who, market_id, amount: fees_withdrawn }.into(),
            );
        };
        test_withdraw(ALICE, _1_4, _3_4);
        test_withdraw(BOB, _1_4, _1_2);
        test_withdraw(CHARLIE, _1_2, 0);
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

#[test]
fn withdraw_fees_is_noop_if_there_are_no_fees() {
    ExtBuilder::default().build().execute_with(|| {
        let spot_prices = vec![_3_4, _1_4];
        let amount = _40;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(2),
            amount,
            spot_prices.clone(),
            CENT,
        );
        let pool_balances = [83_007_499_856, 400_000_000_000];
        let liquidity_parameter = 288_539_008_178;
        assert_pool_state!(
            market_id,
            pool_balances,
            spot_prices,
            liquidity_parameter,
            create_b_tree_map!({ ALICE => amount }),
            0,
        );
        assert_ok!(NeoSwaps::withdraw_fees(RuntimeOrigin::signed(ALICE), market_id));
        assert_pool_state!(
            market_id,
            pool_balances,
            spot_prices,
            liquidity_parameter,
            create_b_tree_map!({ ALICE => amount }),
            0,
        );
    });
}
