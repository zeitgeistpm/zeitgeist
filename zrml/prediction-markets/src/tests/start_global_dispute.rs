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

use zeitgeist_primitives::types::OutcomeReport;

// TODO(#1239) MarketDoesNotExist
// TODO(#1239) NoDisputeMechanism
// TODO(#1239) InvalidMarketStatus
// TODO(#1239) GlobalDisputeExistsAlready
// TODO(#1239) MarketIsNotReported
// TODO(#1239) MarketDisputeMechanismNotFailed

#[test]
fn start_global_dispute_fails_on_wrong_mdm() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..2),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MaxDisputes::get() + 1),
            Some(MarketDisputeMechanism::Authorized),
            ScoringRule::Lmsr,
        ));
        let market_id = MarketCommons::latest_market_id().unwrap();

        let market = MarketCommons::market(&market_id).unwrap();
        let grace_period = market.deadlines.grace_period;
        run_to_block(end + grace_period + 1);
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            market_id,
            OutcomeReport::Categorical(0)
        ));
        let dispute_at_0 = end + grace_period + 2;
        run_to_block(dispute_at_0);

        // only one dispute allowed for authorized mdm
        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), market_id,));
        run_blocks(1);
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        assert_noop!(
            PredictionMarkets::start_global_dispute(RuntimeOrigin::signed(CHARLIE), market_id),
            Error::<Runtime>::InvalidDisputeMechanism
        );
    });
}
