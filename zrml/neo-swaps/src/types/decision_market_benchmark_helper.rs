#![cfg(feature = "runtime-benchmarks")]

use crate::{
    liquidity_tree::types::LiquidityTree,
    types::{DecisionMarketOracle, Pool},
    BalanceOf, Config, MarketIdOf, Pallet, Pools, MIN_SWAP_FEE,
};
use alloc::collections::BTreeMap;
use core::marker::PhantomData;
use frame_benchmarking::whitelisted_caller;
use orml_traits::MultiCurrency;
use sp_runtime::{traits::Zero, Perbill, SaturatedConversion, Saturating};
use zeitgeist_primitives::{
    constants::{BASE, CENT},
    math::fixed::{BaseProvider, ZeitgeistBase},
    traits::{CompleteSetOperationsApi, FutarchyBenchmarkHelper, MarketBuilderTrait},
    types::{Asset, MarketCreation, MarketPeriod, MarketStatus, MarketType, ScoringRule},
};
use zrml_market_commons::{types::MarketBuilder, MarketCommonsPalletApi};

pub struct DecisionMarketBenchmarkHelper<T>(PhantomData<T>);

impl<T> FutarchyBenchmarkHelper<DecisionMarketOracle<T>> for DecisionMarketBenchmarkHelper<T>
where
    T: Config + zrml_market_commons::Config,
    <zrml_market_commons::Pallet<T> as MarketCommonsPalletApi>::MarketId:
        Into<<T as zrml_market_commons::Config>::MarketId>,
    <T as zrml_market_commons::Config>::MarketId:
        Into<<zrml_market_commons::Pallet<T> as MarketCommonsPalletApi>::MarketId>,
{
    /// Creates a mocked up pool with prices so that the returned decision market oracle evaluates
    /// to `value`. The pool is technically in invalid state.
    fn create_oracle(value: bool) -> DecisionMarketOracle<T> {
        let market_id: MarketIdOf<T> = 0u8.into();
        let collateral = Asset::Ztg;

        // Create a `reserves` map so that `positive_outcome` has a higher price if and only if
        // `value` is `true`.
        let positive_outcome = Asset::CategoricalOutcome(market_id, 0u16);
        let negative_outcome = Asset::CategoricalOutcome(market_id, 1u16);
        let mut reserves = BTreeMap::new();
        let one: BalanceOf<T> = ZeitgeistBase::get().unwrap();
        let two: BalanceOf<T> = one.saturating_mul(2u8.into());
        if value {
            reserves.insert(positive_outcome, one);
            reserves.insert(negative_outcome, two);
        } else {
            reserves.insert(positive_outcome, two);
            reserves.insert(negative_outcome, one);
        }

        let account_id: T::AccountId = Pallet::<T>::pool_account_id(&market_id);
        let pool = Pool {
            account_id: account_id.clone(),
            reserves: reserves.try_into().unwrap(),
            collateral,
            liquidity_parameter: one.clone(),
            liquidity_shares_manager: LiquidityTree::new(account_id, one).unwrap(),
            swap_fee: Zero::zero(),
        };

        Pools::<T>::insert(market_id, pool);

        DecisionMarketOracle::new(market_id, positive_outcome, negative_outcome)
    }
}
