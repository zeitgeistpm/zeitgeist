// Copyright 2024 Forecasting Technologies LTD.
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

#![allow(
    // Auto-generated code is a no man's land
    clippy::arithmetic_side_effects
)]
#![cfg(feature = "runtime-benchmarks")]

#[cfg(test)]
use crate::Pallet as HybridRouter;

use crate::*;
use frame_benchmarking::v2::*;
use frame_support::{
    assert_ok,
    storage::{with_transaction, TransactionOutcome::*},
};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::{Perbill, SaturatedConversion};
use types::Strategy;
use zeitgeist_primitives::{
    constants::{base_multiples::*, CENT},
    math::fixed::{BaseProvider, ZeitgeistBase},
    traits::{CompleteSetOperationsApi, DeployPoolApi},
    types::{Asset, Market, MarketCreation, MarketPeriod, MarketStatus, MarketType, ScoringRule},
};
use zrml_market_commons::MarketCommonsPalletApi;

pub const MIN_SPOT_PRICE: u128 = CENT / 2;

// Same behavior as `assert_ok!`, except that it wraps the call inside a transaction layer. Required
// when calling into functions marked `require_transactional` to avoid a `Transactional(NoLayer)`
// error.
macro_rules! assert_ok_with_transaction {
    ($expr:expr) => {{
        assert_ok!(with_transaction(|| match $expr {
            Ok(val) => Commit(Ok(val)),
            Err(err) => Rollback(Err(err)),
        }));
    }};
}

fn create_spot_prices<T: Config>(asset_count: u16) -> Vec<BalanceOf<T>> {
    let mut result = vec![MIN_SPOT_PRICE.saturated_into(); (asset_count - 1) as usize];
    // Price distribution has no bearing on the benchmarks.
    let remaining_u128 =
        ZeitgeistBase::<u128>::get().unwrap() - (asset_count - 1) as u128 * MIN_SPOT_PRICE;
    result.push(remaining_u128.saturated_into());
    result
}

fn create_market<T>(caller: T::AccountId, base_asset: AssetOf<T>, asset_count: u16) -> MarketIdOf<T>
where
    T: Config,
{
    let market = Market {
        base_asset,
        creation: MarketCreation::Permissionless,
        creator_fee: Perbill::zero(),
        creator: caller.clone(),
        oracle: caller,
        metadata: vec![0, 50],
        market_type: MarketType::Categorical(asset_count),
        period: MarketPeriod::Block(0u32.into()..1u32.into()),
        deadlines: Default::default(),
        scoring_rule: ScoringRule::AmmCdaHybrid,
        status: MarketStatus::Active,
        report: None,
        resolved_outcome: None,
        dispute_mechanism: None,
        bonds: Default::default(),
        early_close: None,
    };
    let maybe_market_id = T::MarketCommons::push_market(market);
    maybe_market_id.unwrap()
}

fn create_market_and_deploy_pool<T: Config>(
    caller: T::AccountId,
    base_asset: AssetOf<T>,
    asset_count: u16,
    amount: BalanceOf<T>,
) -> MarketIdOf<T>
where
    T: Config,
{
    let market_id = create_market::<T>(caller.clone(), base_asset, asset_count);
    let total_cost = amount + T::AssetManager::minimum_balance(base_asset);
    assert_ok!(T::AssetManager::deposit(base_asset, &caller, total_cost));
    assert_ok_with_transaction!(T::CompleteSetOperations::buy_complete_set(
        caller.clone(),
        market_id,
        amount
    ));
    assert_ok_with_transaction!(T::AmmPoolDeployer::deploy_pool(
        caller,
        market_id,
        amount,
        create_spot_prices::<T>(asset_count),
        CENT.saturated_into(),
    ));
    market_id
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn buy(n: Linear<2, 128>) {
        let buyer: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = n.try_into().unwrap();
        let market_id = create_market_and_deploy_pool::<T>(
            buyer.clone(),
            base_asset,
            asset_count,
            _10.saturated_into(),
        );

        let asset = Asset::CategoricalOutcome(market_id, 0u16);
        let amount_in = _1.saturated_into();
        assert_ok!(T::AssetManager::deposit(base_asset, &buyer, _10.saturated_into()));

        let max_price = _9_10.saturated_into();
        let orders = vec![];
        let strategy = Strategy::LimitOrder;

        #[extrinsic_call]
        buy(
            RawOrigin::Signed(buyer),
            market_id,
            asset_count,
            asset,
            amount_in,
            max_price,
            orders,
            strategy,
        );
    }

    impl_benchmark_test_suite!(
        HybridRouter,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime
    );
}
