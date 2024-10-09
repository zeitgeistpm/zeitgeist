use super::*;

#[test]
fn split_followed_by_merge_vertical_no_parent() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![B1, B0, B1], vec![B0, B1, B0]];
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
        // let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        // let parent_market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        // let amount = _1;
        // let partition = vec![vec![B1, B0, B1], vec![B0, B1, B0]];

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

// This test shows that splitting a token horizontally can be accomplished by splitting the parent
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
            vec![vec![B0, B0, B1], vec![B1, B1, B0]],
            amount,
        ));
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            market_id,
            vec![vec![B1, B0, B0], vec![B0, B1, B0]],
            amount,
        ));

        let ct_001 = CombinatorialToken([
            207, 168, 160, 93, 238, 221, 197, 1, 171, 102, 28, 24, 18, 107, 205, 231, 227, 98, 220,
            105, 211, 29, 181, 30, 53, 7, 200, 154, 134, 246, 38, 139,
        ]);
        let ct_010 = CombinatorialToken([
            23, 108, 101, 109, 145, 51, 201, 192, 240, 28, 43, 57, 53, 4, 75, 101, 116, 20, 184,
            25, 227, 71, 149, 136, 59, 82, 81, 105, 41, 160, 39, 142,
        ]);
        let ct_100 = CombinatorialToken([
            63, 95, 93, 48, 199, 160, 113, 178, 33, 24, 52, 193, 247, 121, 229, 30, 231, 100, 209,
            14, 57, 98, 193, 214, 34, 251, 53, 51, 136, 146, 93, 26,
        ]);

        assert_eq!(alice.free_balance(ct_001), amount);
        assert_eq!(alice.free_balance(ct_010), amount);
        assert_eq!(alice.free_balance(ct_100), amount);

        // Split vertically. This should yield the same amount as the two splits above.
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            market_id,
            vec![vec![B1, B0, B0], vec![B0, B1, B0], vec![B0, B0, B1]],
            amount,
        ));

        assert_eq!(alice.free_balance(ct_001), 2 * amount);
        assert_eq!(alice.free_balance(ct_010), 2 * amount);
        assert_eq!(alice.free_balance(ct_100), 2 * amount);
    });
}

// This test shows that splitting a token horizontally can be accomplished by splitting a the parent
// token vertically with a finer partition.
#[test]
fn vertical_split_followed_by_horizontal_split_with_parent() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();
        let pallet = Account::new(Pallet::<Runtime>::account_id());

        // Prepare level 1 token.
        let parent_market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let parent_amount = _6;
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            parent_market_id,
            vec![vec![B0, B0, B1], vec![B1, B1, B0]],
            parent_amount,
        ));

        let child_market_id = create_market(Asset::Ztg, MarketType::Categorical(4));
        let child_amount_first_pass = _3;
        // Collection ID of [0, 0, 1].
        let parent_collection_id = [
            6, 44, 173, 50, 122, 106, 144, 185, 253, 19, 252, 218, 215, 241, 218, 37, 196, 112, 45,
            133, 165, 48, 231, 189, 87, 123, 131, 18, 190, 5, 110, 93,
        ];

        let ct_001 = CombinatorialToken([
            207, 168, 160, 93, 238, 221, 197, 1, 171, 102, 28, 24, 18, 107, 205, 231, 227, 98, 220,
            105, 211, 29, 181, 30, 53, 7, 200, 154, 134, 246, 38, 139,
        ]);
        let ct_110 = CombinatorialToken([
            101, 210, 61, 196, 5, 247, 150, 41, 186, 49, 11, 63, 139, 53, 25, 65, 161, 83, 24, 142,
            225, 102, 57, 241, 199, 18, 226, 137, 68, 3, 219, 131,
        ]);
        let ct_001_0011 = CombinatorialToken([
            156, 47, 254, 154, 29, 5, 149, 94, 214, 135, 92, 36, 188, 120, 42, 144, 136, 151, 255,
            91, 232, 152, 91, 236, 177, 66, 36, 72, 134, 234, 212, 177,
        ]);
        let ct_001_1100 = CombinatorialToken([
            224, 47, 73, 22, 156, 226, 199, 74, 28, 251, 44, 108, 73, 125, 192, 151, 193, 60, 156,
            240, 215, 23, 138, 168, 181, 175, 241, 70, 71, 126, 48, 45,
        ]);
        let ct_001_1000 = CombinatorialToken([
            9, 208, 130, 141, 130, 87, 234, 29, 150, 109, 181, 68, 138, 137, 66, 8, 251, 157, 224,
            152, 176, 104, 231, 193, 178, 99, 184, 123, 78, 213, 63, 150,
        ]);
        let ct_001_0100 = CombinatorialToken([
            220, 137, 106, 212, 207, 90, 155, 125, 22, 15, 184, 90, 227, 159, 173, 59, 33, 73, 50,
            245, 183, 245, 46, 56, 66, 199, 94, 129, 154, 18, 48, 73,
        ]);

        // Split vertically and then horizontally.
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            Some(parent_collection_id),
            child_market_id,
            vec![vec![B0, B0, B1, B1], vec![B1, B1, B0, B0]],
            child_amount_first_pass,
        ));
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            Some(parent_collection_id),
            child_market_id,
            vec![vec![B1, B0, B0, B0], vec![B0, B1, B0, B0]],
            child_amount_first_pass,
        ));

        assert_eq!(alice.free_balance(ct_001), parent_amount - child_amount_first_pass);
        assert_eq!(alice.free_balance(ct_110), parent_amount);
        assert_eq!(alice.free_balance(ct_001_0011), child_amount_first_pass);
        assert_eq!(alice.free_balance(ct_001_1100), 0);
        assert_eq!(alice.free_balance(ct_001_1000), child_amount_first_pass);
        assert_eq!(alice.free_balance(ct_001_0100), child_amount_first_pass);
        assert_eq!(pallet.free_balance(Asset::Ztg), parent_amount);
        assert_eq!(pallet.free_balance(ct_001_1100), 0);

        // Split vertically. This should yield the same amount as the two splits above.
        let child_amount_second_pass = _2;
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            Some(parent_collection_id),
            child_market_id,
            vec![vec![B1, B0, B0, B0], vec![B0, B1, B0, B0], vec![B0, B0, B1, B1]],
            child_amount_second_pass,
        ));

        let total_child_amount = child_amount_first_pass + child_amount_second_pass;
        assert_eq!(alice.free_balance(ct_001), parent_amount - total_child_amount);
        assert_eq!(alice.free_balance(ct_110), parent_amount);
        assert_eq!(alice.free_balance(ct_001_0011), total_child_amount);
        assert_eq!(alice.free_balance(ct_001_1100), 0);
        assert_eq!(alice.free_balance(ct_001_1000), total_child_amount);
        assert_eq!(alice.free_balance(ct_001_0100), total_child_amount);
        assert_eq!(pallet.free_balance(Asset::Ztg), parent_amount);
        assert_eq!(pallet.free_balance(ct_001_1100), 0);
    });
}
