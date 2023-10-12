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
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let liquidity = _10;
        let spot_prices = vec![_1_6, _5_6 + 1];
        let swap_fee = CENT;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        // Mock up some fees for Alice to withdraw.
        let mut pool = Pools::<Runtime>::get(market_id).unwrap();
        let fees = 123456789;
        assert_ok!(AssetManager::deposit(pool.collateral, &pool.account_id, fees));
        pool.liquidity_shares_manager.fees = fees;
        Pools::<Runtime>::insert(market_id, pool.clone());
        let alice_before = AssetManager::free_balance(pool.collateral, &ALICE);
        assert_ok!(NeoSwaps::withdraw_fees(RuntimeOrigin::signed(ALICE), market_id));
        let expected_pool_account_balance = AssetManager::minimum_balance(pool.collateral);
        assert_eq!(
            AssetManager::free_balance(pool.collateral, &pool.account_id),
            expected_pool_account_balance
        );
        assert_eq!(AssetManager::free_balance(pool.collateral, &ALICE), alice_before + fees);
        let pool_after = Pools::<Runtime>::get(market_id).unwrap();
        assert_eq!(pool_after.liquidity_shares_manager.fees, 0);
        System::assert_last_event(
            Event::FeesWithdrawn { who: ALICE, market_id, amount: fees }.into(),
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
