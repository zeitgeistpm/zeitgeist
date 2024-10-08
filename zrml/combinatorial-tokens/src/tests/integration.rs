use super::*;

#[test]
fn split_followed_by_merge_vertical_no_parent() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![_B1, _B0, _B1], vec![_B0, _B1, _B0]];
        let amount = _1;

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
fn split_followed_by_merge_vertical_with_parent() {
    ExtBuilder::build().execute_with(|| {
        // TODO
        // let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        // let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        // let amount = _1;
        // let partition = vec![vec![_B1, _B0, _B1], vec![_B0, _B1, _B0]];

        // assert_ok!(CombinatorialTokens::split_position(
        //     alice.signed(),
        //     None,
        //     market_id,
        //     partition.clone(),
        //     amount,
        // ));
        // assert_eq!(alice.free_balance(Asset::Ztg), _99);

        // assert_ok!(CombinatorialTokens::merge_position(
        //     alice.signed(),
        //     None,
        //     market_id,
        //     partition,
        //     amount,
        // ));
        // assert_eq!(alice.free_balance(Asset::Ztg), _100);
    });
}

// This test shows that splitting a token horizontally can be accomplished by splitting a the parent
// token vertically with a finer partition.
#[test]
fn vertical_split_followed_by_horizontal_split_no_parent() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let amount = _1;

        // Split vertically and then horizontally.
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            market_id,
            vec![vec![_B0, _B0, _B1], vec![_B1, _B1, _B0]],
            amount,
        ));
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            market_id,
            vec![vec![_B1, _B0, _B0], vec![_B0, _B1, _B0]],
            amount,
        ));

        let ct0 = CombinatorialToken([
            207, 168, 160, 93, 238, 221, 197, 1, 171, 102, 28, 24, 18, 107, 205, 231, 227, 98, 220,
            105, 211, 29, 181, 30, 53, 7, 200, 154, 134, 246, 38, 139,
        ]);
        let ct1 = CombinatorialToken([
            23, 108, 101, 109, 145, 51, 201, 192, 240, 28, 43, 57, 53, 4, 75, 101, 116, 20, 184,
            25, 227, 71, 149, 136, 59, 82, 81, 105, 41, 160, 39, 142,
        ]);
        let ct2 = CombinatorialToken([
            63, 95, 93, 48, 199, 160, 113, 178, 33, 24, 52, 193, 247, 121, 229, 30, 231, 100, 209,
            14, 57, 98, 193, 214, 34, 251, 53, 51, 136, 146, 93, 26,
        ]);

        assert_eq!(alice.free_balance(ct0), amount);
        assert_eq!(alice.free_balance(ct1), amount);
        assert_eq!(alice.free_balance(ct2), amount);

        // Split vertically. This should yield the same amount as the two splits above.
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            market_id,
            vec![vec![_B1, _B0, _B0], vec![_B0, _B1, _B0], vec![_B0, _B0, _B1]],
            amount,
        ));

        assert_eq!(alice.free_balance(ct0), 2 * amount);
        assert_eq!(alice.free_balance(ct1), 2 * amount);
        assert_eq!(alice.free_balance(ct2), 2 * amount);
    });
}
