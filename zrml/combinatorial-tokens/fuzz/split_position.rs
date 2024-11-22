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
    traits::CombinatorialIdManager,
    AccountIdOf, BalanceOf, CombinatorialIdOf, Config, MarketIdOf,
};

#[derive(Debug)]
struct SplitPositionFuzzParams {
    account_id: AccountIdOf<Runtime>,
    parent_collection_id: Option<CombinatorialIdOf<Runtime>>,
    market_id: MarketIdOf<Runtime>,
    partition: Vec<Vec<bool>>,
    amount: BalanceOf<Runtime>,
    force_max_work: bool,
}

impl<'a> Arbitrary<'a> for SplitPositionFuzzParams {
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

        let params = SplitPositionFuzzParams {
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

fuzz_target!(|params: SplitPositionFuzzParams| {
    let mut ext = ExtBuilder::build();

    ext.execute_with(|| {
        // We create a market and equip the user with the tokens they require to make the
        // `split_position` call meaningful.
        let collateral = Asset::Ztg;
        let asset_count = if let Some(member) = params.partition.first() {
            member.len().max(2) as u16
        } else {
            return;
        };
        let market = common::market::<Runtime>(
            params.market_id,
            collateral,
            MarketType::Categorical(asset_count),
        );
        <<Runtime as Config>::MarketCommons as MarketCommonsPalletApi>::push_market(market)
            .unwrap();

        let position = if let Some(pci) = params.parent_collection_id {
            let position_id =
                <Runtime as Config>::CombinatorialIdManager::get_position_id(collateral, pci);

            Asset::CombinatorialToken(position_id)
        } else {
            Asset::Ztg
        };
        <<Runtime as Config>::MultiCurrency>::deposit(position, &params.account_id, params.amount)
            .unwrap();

        let _ = CombinatorialTokens::split_position(
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
