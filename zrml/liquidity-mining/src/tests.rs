// Copyright 2021-2022 Zeitgeist PM LLC.
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

#![cfg(test)]

use crate::{
    mock::{Balances, ExtBuilder, LiquidityMining, Origin, Runtime, System, ALICE, BOB},
    track_incentives_based_on_bought_shares::TrackIncentivesBasedOnBoughtShares,
    track_incentives_based_on_sold_shares::TrackIncentivesBasedOnSoldShares,
    BlockBoughtShares, BlockSoldShares, LiquidityMiningPalletApi as _, OwnedValues,
};
use core::ops::Range;
use frame_support::{
    assert_err, assert_ok,
    dispatch::DispatchError,
    traits::{Currency, OnFinalize},
};
use frame_system::RawOrigin;
use zeitgeist_primitives::types::{
    Deadlines, Market, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus,
    MarketType, ScoringRule,
};
use zrml_market_commons::Markets;

#[test]
fn blocks_shares_are_updated_after_each_block() {
    ExtBuilder::default().build().execute_with(|| {
        LiquidityMining::add_shares(ALICE, 0, 1);
        LiquidityMining::add_shares(ALICE, 1, 1);
        LiquidityMining::remove_shares(&ALICE, &0, 1);
        assert_eq!(<BlockBoughtShares<Runtime>>::iter().count(), 2);
        assert_eq!(<BlockSoldShares<Runtime>>::iter().count(), 1);
        LiquidityMining::on_finalize(1);
        assert_eq!(<BlockBoughtShares<Runtime>>::iter().count(), 0);
        assert_eq!(<BlockSoldShares<Runtime>>::iter().count(), 0);
    });
}

#[test]
fn distribute_market_incentives_removes_market_and_distributes_all_incentives_to_lps() {
    ExtBuilder::default().build().execute_with(|| {
        let initial_alice_balance = Balances::free_balance(ALICE);
        create_default_market(0, 1..5);
        create_default_market(1, 1..5);

        LiquidityMining::add_shares(ALICE, 0, 10);
        LiquidityMining::add_shares(ALICE, 1, 10);
        LiquidityMining::on_finalize(2);

        assert_eq!(<OwnedValues<Runtime>>::iter().count(), 2);
        assert_ok!(LiquidityMining::distribute_market_incentives(&0));
        assert_ok!(LiquidityMining::distribute_market_incentives(&1));
        assert_eq!(<OwnedValues<Runtime>>::iter().count(), 0);

        // In this case, each market have the same amount of incentives
        let market_incentives = ExtBuilder::default().per_block_incentives / 2;
        // Perpetual balance for the entire campaign
        let entire_market_perpetual_balance = market_incentives / 1000;
        // Account only stayed 1 block out of 4 (25%), i.e, lost 75% of the perpetual balance
        let actual_market_perpetual_balance = entire_market_perpetual_balance / 4;
        // Ordinary balance
        let actual_market_incentives = market_incentives - entire_market_perpetual_balance;

        let new_balance = initial_alice_balance
            + 2 * actual_market_incentives
            + 2 * actual_market_perpetual_balance;

        assert_eq!(Balances::free_balance(ALICE), new_balance);
    });
}

#[test]
fn genesis_has_lm_account_and_initial_per_block_distribution() {
    ExtBuilder::default().build().execute_with(|| {
        let pallet_account_id = crate::Pallet::<Runtime>::pallet_account_id();
        assert_eq!(
            Balances::total_balance(&pallet_account_id),
            ExtBuilder::default().initial_balance
        );
        assert_eq!(
            <crate::PerBlockIncentive::<Runtime>>::get(),
            ExtBuilder::default().per_block_incentives
        );
    });
}

#[test]
fn owned_balances_are_updated_after_bought_shares() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(5);
        create_default_market(0, 1..11);
        create_default_market(1, 1..21);

        LiquidityMining::add_shares(ALICE, 0, 100);
        LiquidityMining::add_shares(ALICE, 1, 200);
        LiquidityMining::add_shares(BOB, 1, 300);
        TrackIncentivesBasedOnBoughtShares::<Runtime>::exec(5);

        let vec = <OwnedValues<Runtime>>::iter().collect::<Vec<_>>();

        assert_eq!(vec[2].2.total_shares, 100);
        assert_eq!(vec[1].2.total_shares, 200);
        assert_eq!(vec[0].2.total_shares, 300);

        // Market 0 has 42.8% of `per_block_incentives`
        let market_incentives_0 = 4280000000;
        // Market 1 has 57.1% of `per_block_incentives`
        let market_incentives_1 = 5710000000;

        // 0.1% of `market_incentives_0`.
        let perpetual_incentives_alice_0 = market_incentives_0 / 1000;

        // One share value for Market 1
        let market_1_one_share_value = one_share_value(market_incentives_1, 500);
        // Raw incentives for ALICE on Market 1
        let raw_incentives_alice_1 = market_1_one_share_value * 200;
        // Raw incentives for BOB on Market 1
        let raw_incentives_bob_1 = market_1_one_share_value * 300;
        // 0.1% of `raw_incentives_ALICE_1`
        let perpetual_incentives_alice_1 = raw_incentives_alice_1 / 1000;
        // 0.1% of `raw_incentives_BOB_1`
        let perpetual_incentives_bob_1 = raw_incentives_bob_1 / 1000;

        assert_eq!(vec[2].2.perpetual_incentives, perpetual_incentives_alice_0);
        assert_eq!(vec[1].2.perpetual_incentives, perpetual_incentives_alice_1);
        assert_eq!(vec[0].2.perpetual_incentives, perpetual_incentives_bob_1);

        assert_eq!(vec[2].2.total_incentives, market_incentives_0 - perpetual_incentives_alice_0);
        assert_eq!(
            vec[1].2.total_incentives,
            raw_incentives_alice_1 - perpetual_incentives_alice_1
        );
        assert_eq!(vec[0].2.total_incentives, raw_incentives_bob_1 - perpetual_incentives_bob_1);
    });
}

#[test]
fn owned_balances_are_updated_after_sold_shares() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(5);
        create_default_market(0, 1..11);
        create_default_market(1, 1..21);

        LiquidityMining::add_shares(ALICE, 0, 100);
        LiquidityMining::add_shares(ALICE, 1, 200);
        LiquidityMining::on_finalize(5);

        LiquidityMining::remove_shares(&ALICE, &0, 100);
        LiquidityMining::remove_shares(&ALICE, &1, 50);
        TrackIncentivesBasedOnSoldShares::<Runtime>::exec();

        let vec = <OwnedValues<Runtime>>::iter().collect::<Vec<_>>();

        // Market 0 has 42.8% of `per_block_incentives`
        let market_incentives_0 = 4280000000;
        // Market 1 has 57.1% of `per_block_incentives`
        let market_incentives_1 = 5710000000;

        // 0.1% of `market_incentives_0`.
        let perpetual_incentives_alice_0 = market_incentives_0 / 1000;
        // 0.1% of `market_incentives_1`.
        let perpetual_incentives_alice_1 = market_incentives_1 / 1000;
        // Alice sold 25% of her owned shares
        let incentives_alice_1 = (market_incentives_1 - perpetual_incentives_alice_1) / 4 * 3;

        assert_eq!(vec[1].2.total_shares, 0);
        assert_eq!(vec[1].2.total_incentives, 0);
        assert_eq!(vec[1].2.perpetual_incentives, perpetual_incentives_alice_0);

        assert_eq!(vec[0].2.total_shares, 150);
        assert_eq!(vec[0].2.total_incentives, incentives_alice_1);
        assert_eq!(vec[0].2.perpetual_incentives, perpetual_incentives_alice_1);
    });
}

#[test]
fn only_sudo_can_change_per_block_distribution() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(LiquidityMining::set_per_block_distribution(RawOrigin::Root.into(), 100));
        assert_err!(
            LiquidityMining::set_per_block_distribution(Origin::signed(ALICE), 100),
            DispatchError::BadOrigin
        );
    });
}

fn create_default_market(market_id: u128, period: Range<u64>) {
    Markets::<Runtime>::insert(
        market_id,
        Market {
            creation: MarketCreation::Permissionless,
            creator_fee: 0,
            creator: 0,
            market_type: MarketType::Categorical(0),
            dispute_mechanism: MarketDisputeMechanism::SimpleDisputes,
            metadata: vec![],
            oracle: 0,
            period: MarketPeriod::Block(period),
            deadlines: Deadlines {
                oracle_delay: 1_u32,
                oracle_duration: 1_u32,
                dispute_duration: 1_u32,
            },
            report: None,
            resolved_outcome: None,
            status: MarketStatus::Closed,
            scoring_rule: ScoringRule::CPMM,
        },
    );
}

// One bought share value of a particular market in a particular block
fn one_share_value(market_incentives: u128, total_shares: u128) -> u128 {
    market_incentives / total_shares
}
