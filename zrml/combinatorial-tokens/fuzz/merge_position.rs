#![no_main]

mod common;

use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};
use libfuzzer_sys::fuzz_target;
use orml_traits::currency::MultiCurrency;
use zeitgeist_primitives::{
    traits::MarketCommonsPalletApi,
    types::{Asset, MarketType},
};
use zrml_combinatorial_tokens::{
    mock::{
        ext_builder::ExtBuilder,
        runtime::{CombinatorialTokens, Runtime, RuntimeOrigin},
    },
    AccountIdOf, BalanceOf, CombinatorialIdOf, Config, MarketIdOf,
};

#[derive(Debug)]
struct MergePositionFuzzParams {
    account_id: AccountIdOf<Runtime>,
    parent_collection_id: Option<CombinatorialIdOf<Runtime>>,
    market_id: MarketIdOf<Runtime>,
    partition: Vec<Vec<bool>>,
    amount: BalanceOf<Runtime>,
    force_max_work: bool,
}

impl<'a> Arbitrary<'a> for MergePositionFuzzParams {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let account_id = u128::arbitrary(u)?;
        let parent_collection_id = Arbitrary::arbitrary(u)?;
        let market_id = 0u8.into();
        let amount = Arbitrary::arbitrary(u)?;
        let force_max_work = Arbitrary::arbitrary(u)?;

        // Note: This might result in members of unequal length, but that's OK.
        let min_len = 0;
        let max_len = 10;
        let len = u.int_in_range(0..=max_len)?;
        let partition =
            (min_len..len).map(|_| Arbitrary::arbitrary(u)).collect::<ArbitraryResult<Vec<_>>>()?;

        let params = MergePositionFuzzParams {
            account_id,
            parent_collection_id,
            market_id,
            partition,
            amount,
            force_max_work,
        };

        Ok(params)
    }
}

fuzz_target!(|params: MergePositionFuzzParams| {
    let mut ext = ExtBuilder::build();

    ext.execute_with(|| {
        // We create a market and equip the user with the tokens they require to make the
        // `merge_position` call meaningful, and deposit collateral in the pallet account.
        let collateral = Asset::Ztg;
        let asset_count = if let Some(member) = params.partition.first() {
            member.len().max(2) as u16
        } else {
            2u16 // In this case the index set doesn't fit the market.
        };
        let market = common::market::<Runtime>(
            params.market_id,
            collateral,
            MarketType::Categorical(asset_count),
        );
        <<Runtime as Config>::MarketCommons as MarketCommonsPalletApi>::push_market(market)
            .unwrap();

        let positions = params
            .partition
            .iter()
            .cloned()
            .map(|index_set| {
                CombinatorialTokens::position_from_parent_collection(
                    params.parent_collection_id,
                    params.market_id,
                    index_set,
                    false,
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        for &position in positions.iter() {
            <<Runtime as Config>::MultiCurrency>::deposit(
                position,
                &params.account_id,
                params.amount,
            )
            .unwrap();
        }

        // Is not required if `parent_collection_id.is_some()`, but we're doing it anyways.
        <<Runtime as Config>::MultiCurrency>::deposit(
            collateral,
            &CombinatorialTokens::account_id(),
            params.amount,
        )
        .unwrap();

        let _ = CombinatorialTokens::merge_position(
            RuntimeOrigin::signed(params.account_id),
            params.parent_collection_id,
            params.market_id,
            params.partition,
            params.amount,
            params.force_max_work,
        );
    });

    let _ = ext.commit_all();
});
