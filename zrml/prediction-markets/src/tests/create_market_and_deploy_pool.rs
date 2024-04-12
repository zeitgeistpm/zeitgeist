// Copyright 2022-2024 Forecasting Technologies LTD.
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

use super::*;
use zeitgeist_primitives::types::{Bond, MarketBonds};

// TODO(#1239) Issue: Separate integration tests and other; use mocks for unit testing
// TODO(#1239) do_buy_complete_set failure
// TODO(#1239) deploy_pool failure

#[test]
fn create_market_and_deploy_pool_works() {
    ExtBuilder::default().build().execute_with(|| {
        let creator = ALICE;
        let creator_fee = Perbill::from_parts(1);
        let oracle = BOB;
        let period = MarketPeriod::Block(1..2);
        let deadlines = Deadlines {
            grace_period: 1,
            oracle_duration: <Runtime as Config>::MinOracleDuration::get() + 2,
            dispute_duration: <Runtime as Config>::MinDisputeDuration::get() + 3,
        };
        let metadata = gen_metadata(0x99);
        let MultiHash::Sha3_384(multihash) = metadata;
        let market_type = MarketType::Categorical(7);
        let dispute_mechanism = Some(MarketDisputeMechanism::Authorized);
        let amount = 1234567890;
        let swap_prices = vec![50 * CENT, 50 * CENT];
        let swap_fee = CENT;
        let market_id = 0;
        assert_ok!(PredictionMarkets::create_market_and_deploy_pool(
            RuntimeOrigin::signed(creator),
            BaseAsset::Ztg,
            creator_fee,
            oracle,
            period.clone(),
            deadlines,
            metadata,
            market_type.clone(),
            dispute_mechanism.clone(),
            amount,
            swap_prices.clone(),
            swap_fee,
        ));
        let market = MarketCommons::market(&0).unwrap();
        let bonds = MarketBonds {
            creation: Some(Bond::new(ALICE, <Runtime as Config>::ValidityBond::get())),
            oracle: Some(Bond::new(ALICE, <Runtime as Config>::OracleBond::get())),
            outsider: None,
            dispute: None,
            close_dispute: None,
            close_request: None,
        };
        assert_eq!(market.creator, creator);
        assert_eq!(market.creation, MarketCreation::Permissionless);
        assert_eq!(market.creator_fee, creator_fee);
        assert_eq!(market.oracle, oracle);
        assert_eq!(market.metadata, multihash);
        assert_eq!(market.market_type, market_type);
        assert_eq!(market.period, period);
        assert_eq!(market.deadlines, deadlines);
        assert_eq!(market.scoring_rule, ScoringRule::AmmCdaHybrid);
        assert_eq!(market.status, MarketStatus::Active);
        assert_eq!(market.report, None);
        assert_eq!(market.resolved_outcome, None);
        assert_eq!(market.dispute_mechanism, dispute_mechanism);
        assert_eq!(market.bonds, bonds);
        // Check that the correct amount of full sets were bought.
        assert_eq!(
            AssetManager::free_balance(Asset::CategoricalOutcome(market_id, 0), &ALICE),
            amount
        );
        assert!(DeployPoolMock::called_once_with(
            creator,
            market_id,
            amount,
            swap_prices,
            swap_fee
        ));
    });
}
