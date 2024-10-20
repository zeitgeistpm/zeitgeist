#![cfg(feature = "runtime-benchmarks")]

use crate::{types::DecisionMarketOracle, BalanceOf, Config, Pallet, MIN_SWAP_FEE};
use core::marker::PhantomData;
use frame_benchmarking::whitelisted_caller;
use orml_traits::MultiCurrency;
use sp_runtime::{Perbill, SaturatedConversion, Saturating};
use zeitgeist_primitives::{
    constants::{BASE, CENT},
    traits::{CompleteSetOperationsApi, FutarchyBenchmarkHelper, MarketBuilderTrait},
    types::{Asset, MarketCreation, MarketPeriod, MarketStatus, MarketType, ScoringRule},
};
use zrml_market_commons::{types::MarketBuilder, MarketCommonsPalletApi};

pub struct DecisionMarketBenchmarkHelper<T>(PhantomData<T>);

impl<T> FutarchyBenchmarkHelper<DecisionMarketOracle<T>> for DecisionMarketBenchmarkHelper<T>
where
    T: Config + zrml_market_commons::Config,
{
    fn create_oracle(value: bool) -> DecisionMarketOracle<T> {
        let collateral = Asset::Ztg;
        let alice: T::AccountId = whitelisted_caller();

        let mut market_builder: MarketBuilder<T> = MarketBuilder::new();
        market_builder
            .base_asset(collateral)
            .creation(MarketCreation::Permissionless)
            .creator(alice.clone())
            .creator_fee(Perbill::zero())
            .oracle(alice.clone())
            .metadata(vec![0; 50])
            .market_type(MarketType::Categorical(2))
            .period(MarketPeriod::Block(0u32.into()..1u32.into()))
            .deadlines(Default::default())
            .scoring_rule(ScoringRule::AmmCdaHybrid)
            .status(MarketStatus::Active)
            .report(None)
            .resolved_outcome(None)
            .dispute_mechanism(None)
            .bonds(Default::default())
            .early_close(None);
        let (market_id, _) = T::MarketCommons::build_market(market_builder).unwrap();

        let amount: BalanceOf<T> = (100 * BASE).saturated_into();
        let double_amount = amount.saturating_mul(2u8.into());
        T::MultiCurrency::deposit(collateral, &alice, amount).unwrap();
        T::CompleteSetOperations::buy_complete_set(alice, market_id, amount);

        Pallet::<T>::do_deploy_pool(
            alice,
            market_id,
            amount,
            vec![(51 * CENT).saturated_into(), (49 * CENT).saturated_into()],
            MIN_SWAP_FEE.saturated_into(),
        )
        .unwrap();

        let positive_outcome = Asset::CategoricalOutcome(market_id, (!value).into());
        let negative_outcome = Asset::CategoricalOutcome(market_id, value.into());

        DecisionMarketOracle { market_id, positive_outcome, negative_outcome }
    }
}
