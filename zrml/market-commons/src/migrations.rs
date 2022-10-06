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

use frame_support::{log, traits::OnRuntimeUpgrade};
use parity_scale_codec::{Decode, Encode};

const MARKET_COMMONS: &[u8] = b"MarketCommons";
const MARKETS: &[u8] = b"Markets";

#[derive(Clone, Decode, Encode)]
pub struct LegacyMarket<AI, BN, M> {
    pub creator: AI,
    pub creation: MarketCreation,
    pub creator_fee: u8,
    pub oracle: AI,
    pub metadata: Vec<u8>,
    pub market_type: MarketType,
    pub period: MarketPeriod<BN, M>,
    pub scoring_rule: ScoringRule,
    pub status: MarketStatus,
    pub report: Option<Report<AI, BN>>,
    pub resolved_outcome: Option<OutcomeReport>,
    pub dispute_mechanism: MarketDisputeMechanism<AI>,
}

type LegacyMarketOf<T> = LegacyMarket<
    <T as frame_system::Config>::AccountId,
    <T as frame_system::Config>::BlockNumber,
    MomentOf<T>,
>;
type MarketOf<T> = Market<
    <T as frame_system::Config>::AccountId,
    <T as frame_system::Config>::BlockNumber,
    MomentOf<T>,
>;

pub struct UpdateMarketsForAuthorizedMDM<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for UpdateMarketsForAuthorizedMDM<T> {
    fn on_runtime_upgrade() -> frame_support::weights::Weight
        where
            T: Config,
    {
        let mut total_weight = T::DbWeight::get().reads(1);
        let storage_version = utility::get_on_chain_storage_version_of_market_commons_pallet();
        if storage_version != MARKET_COMMONS_REQUIRED_STORAGE_VERSION {
            log::info!("Skipping updates of markets; prediction-markets already up to date");
            return total_weight;
        }
            let mut new_markets_data: Vec<(Vec<u8>, MarketOf<T>)> = Vec::new();
        for (key, legacy_market) in storage_iter::<LegacyMarketOf<T>>(MARKET_COMMONS, MARKETS) {
            total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
            let new_market = Market {
                creator: legacy_market.creator,
                creation: legacy_market.creation,
                creator_fee: legacy_market.creator_fee,
                oracle: legacy_market.oracle,
                metadata: legacy_market.metadata,
                market_type: legacy_market.market_type,
                period: legacy_market.period,
                scoring_rule: legacy_market.scoring_rule,
                status: legacy_market.status,
                report: legacy_market.report,
                resolved_outcome: legacy_market.resolved_outcome,
                dispute_mechanism: legacy_market.dispute_mechanism,
            };
            new_markets_data.push((key, new_market));   
    }
         for (key, new_market) in new_markets_data {
            put_storage_value::<MarketOf<T>>(MARKET_COMMONS, MARKETS, &key, new_market);
            total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        }
        log::info!("Completed updates of markets");
        utility::put_storage_version_of_market_commons_pallet(MARKET_COMMONS_NEXT_STORAGE_VERSION);
        total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
        total_weight
    }
       #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<(), &'static str> {
        Ok(())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
        Ok(())
    }

}
