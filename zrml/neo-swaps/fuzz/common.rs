use zeitgeist_primitives::{
    traits::MarketOf,
    types::{Market, MarketCreation, MarketPeriod, MarketStatus, MarketType, ScoringRule},
};
use zrml_neo_swaps::{AssetOf, Config, MarketIdOf};

pub(crate) fn market<T>(
    market_id: MarketIdOf<T>,
    base_asset: AssetOf<T>,
    market_type: MarketType,
) -> MarketOf<<T as Config>::MarketCommons>
where
    T: Config,
    <T as frame_system::Config>::AccountId: Default,
{
    Market {
        market_id,
        base_asset,
        creator: Default::default(),
        creation: MarketCreation::Permissionless,
        creator_fee: Default::default(),
        oracle: Default::default(),
        metadata: Default::default(),
        market_type,
        period: MarketPeriod::Block(0u8.into()..10u8.into()),
        deadlines: Default::default(),
        scoring_rule: ScoringRule::AmmCdaHybrid,
        status: MarketStatus::Active,
        report: None,
        resolved_outcome: None,
        dispute_mechanism: None,
        bonds: Default::default(),
        early_close: None,
    }
}
