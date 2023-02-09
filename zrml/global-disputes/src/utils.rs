#![cfg(any(feature = "runtime-benchmarks", test))]

use crate::*;
use frame_support::traits::Currency;

type CurrencyOf<T> =
    <<T as Config>::MarketCommons as zrml_market_commons::MarketCommonsPalletApi>::Currency;
type BalanceOf<T> =
    <CurrencyOf<T> as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type MarketOf<T> = zeitgeist_primitives::types::Market<
    <T as frame_system::Config>::AccountId,
    BalanceOf<T>,
    <T as frame_system::Config>::BlockNumber,
    MomentOf<T>,
    zeitgeist_primitives::types::Asset<MarketIdOf<T>>,
>;

pub(crate) fn market_mock<T>() -> MarketOf<T>
where
    T: crate::Config,
{
    use frame_support::traits::Get;
    use sp_runtime::traits::AccountIdConversion;
    use zeitgeist_primitives::types::ScoringRule;

    zeitgeist_primitives::types::Market {
        base_asset: zeitgeist_primitives::types::Asset::Ztg,
        creation: zeitgeist_primitives::types::MarketCreation::Permissionless,
        creator_fee: 0,
        creator: T::GlobalDisputesPalletId::get().into_account_truncating(),
        market_type: zeitgeist_primitives::types::MarketType::Scalar(0..=u128::MAX),
        dispute_mechanism: zeitgeist_primitives::types::MarketDisputeMechanism::SimpleDisputes,
        metadata: Default::default(),
        oracle: T::GlobalDisputesPalletId::get().into_account_truncating(),
        period: zeitgeist_primitives::types::MarketPeriod::Block(Default::default()),
        deadlines: zeitgeist_primitives::types::Deadlines {
            grace_period: 1_u32.into(),
            oracle_duration: 1_u32.into(),
            dispute_duration: 1_u32.into(),
        },
        report: None,
        resolved_outcome: None,
        scoring_rule: ScoringRule::CPMM,
        status: zeitgeist_primitives::types::MarketStatus::Disputed,
        bonds: Default::default(),
    }
}
