use crate::MomentOf;
use zeitgeist_primitives::types::{
    Market, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus, MarketType,
};

pub(crate) fn market_mock<T>(ai: T::AccountId) -> Market<T::AccountId, T::BlockNumber, MomentOf<T>>
where
    T: crate::Config,
{
    Market {
        creation: MarketCreation::Permissionless,
        creator_fee: 0,
        creator: T::AccountId::default(),
        market_type: MarketType::Scalar(0..=100),
        mdm: MarketDisputeMechanism::Authorized(ai),
        metadata: Default::default(),
        oracle: T::AccountId::default(),
        period: MarketPeriod::Block(Default::default()),
        report: None,
        resolved_outcome: None,
        status: MarketStatus::Closed,
    }
}
