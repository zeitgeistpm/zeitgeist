use super::*;

#[test]
fn split_followed_by_merge_no_parent() {
    ExtBuilder::build().execute_with(|| {
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));

        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        let amount = _1;
        let partition = vec![vec![_B1, _B0, _B1], vec![_B0, _B1, _B0]];

        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            market_id,
            partition.clone(),
            amount,
        ));
        assert_eq!(alice.free_balance(Asset::Ztg), _99);

        assert_ok!(CombinatorialTokens::merge_position(
            alice.signed(),
            None,
            market_id,
            partition,
            amount,
        ));
        assert_eq!(alice.free_balance(Asset::Ztg), _100);
    });
}

#[test]
fn split_followed_by_merge_with_parent() {
    ExtBuilder::build().execute_with(|| {
        let parent_market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));

        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        let amount = _1;
        let partition = vec![vec![_B1, _B0, _B1], vec![_B0, _B1, _B0]];

        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            market_id,
            partition.clone(),
            amount,
        ));
        assert_eq!(alice.free_balance(Asset::Ztg), _99);

        assert_ok!(CombinatorialTokens::merge_position(
            alice.signed(),
            None,
            market_id,
            partition,
            amount,
        ));
        assert_eq!(alice.free_balance(Asset::Ztg), _100);
    });
}
