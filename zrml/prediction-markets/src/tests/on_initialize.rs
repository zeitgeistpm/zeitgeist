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

use crate::LastTimeFrame;
use frame_support::traits::Hooks;
use zeitgeist_primitives::constants::MILLISECS_PER_BLOCK;

#[test]
fn on_initialize_skips_the_genesis_block() {
    // We ensure that a timestamp of zero will not be stored at genesis into LastTimeFrame storage.
    let blocks = 5;
    let end = (blocks * MILLISECS_PER_BLOCK) as u64;
    ExtBuilder::default().build().execute_with(|| {
        let category_count = 3;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            ALICE,
            MarketPeriod::Timestamp(0..end),
            get_deadlines(),
            gen_metadata(50),
            MarketCreation::Permissionless,
            MarketType::Categorical(category_count),
            Some(MarketDisputeMechanism::SimpleDisputes),
            ScoringRule::Lmsr,
        ));

        // Blocknumber = 0
        assert_eq!(Timestamp::get(), 0);
        PredictionMarkets::on_initialize(0);
        assert_eq!(LastTimeFrame::<Runtime>::get(), None);

        // Blocknumber = 1
        assert_eq!(Timestamp::get(), 0);
        PredictionMarkets::on_initialize(1);
        assert_eq!(LastTimeFrame::<Runtime>::get(), None);

        // Blocknumer != 0, 1
        set_timestamp_for_on_initialize(end);
        PredictionMarkets::on_initialize(2);
        assert_eq!(LastTimeFrame::<Runtime>::get(), Some(blocks.into()));
    });
}
