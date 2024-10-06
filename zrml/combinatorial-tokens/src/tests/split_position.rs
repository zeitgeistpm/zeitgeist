use super::*;

#[test]
fn split_position_fails_if_market_not_found() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();
        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                0,
                vec![vec![_0, _0, _1], vec![_1, _1, _0]],
                1,
            ),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist,
        );
    });
}
