#![no_main]

mod common;

use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};
use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
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
        let partition = Arbitrary::arbitrary(u)?;
        let amount = Arbitrary::arbitrary(u)?;
        let force_max_work = Arbitrary::arbitrary(u)?;

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
        let market =
            common::market::<Runtime>(params.market_id, collateral, MarketType::Categorical(7));
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
