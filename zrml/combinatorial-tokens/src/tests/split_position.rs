use super::*;

#[test]
fn split_position_works_vertical_no_parent() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();
        let pallet = Account::new(Pallet::<Runtime>::account_id());

        let parent_collection_id = None;
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![_B0, _B0, _B1], vec![_B1, _B1, _B0]];

        let amount = _1;
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            parent_collection_id,
            market_id,
            partition.clone(),
            amount,
        ));

        let ct1 = CombinatorialToken([
            207, 168, 160, 93, 238, 221, 197, 1, 171, 102, 28, 24, 18, 107, 205, 231, 227, 98, 220,
            105, 211, 29, 181, 30, 53, 7, 200, 154, 134, 246, 38, 139,
        ]);
        let ct2 = CombinatorialToken([
            101, 210, 61, 196, 5, 247, 150, 41, 186, 49, 11, 63, 139, 53, 25, 65, 161, 83, 24, 142,
            225, 102, 57, 241, 199, 18, 226, 137, 68, 3, 219, 131,
        ]);

        assert_eq!(alice.free_balance(ct1), amount);
        assert_eq!(alice.free_balance(ct2), amount);
        assert_eq!(alice.free_balance(Asset::Ztg), _100 - amount);
        assert_eq!(pallet.free_balance(Asset::Ztg), amount);

        System::assert_last_event(
            Event::<Runtime>::TokenSplit {
                who: alice.id,
                parent_collection_id,
                market_id,
                partition,
                asset_in: Asset::Ztg,
                assets_out: vec![ct1, ct2],
                collection_ids: vec![
                    [
                        6, 44, 173, 50, 122, 106, 144, 185, 253, 19, 252, 218, 215, 241, 218, 37,
                        196, 112, 45, 133, 165, 48, 231, 189, 87, 123, 131, 18, 190, 5, 110, 93,
                    ],
                    [
                        1, 189, 94, 224, 153, 162, 145, 214, 33, 231, 230, 19, 122, 179, 122, 117,
                        193, 123, 73, 220, 240, 131, 180, 180, 137, 14, 179, 148, 188, 13, 107, 65,
                    ],
                ],
                amount,
            }
            .into(),
        );
    });
}

#[test]
fn split_position_fails_if_market_not_found() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();
        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                0,
                vec![vec![_B0, _B0, _B1], vec![_B1, _B1, _B0]],
                1,
            ),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist,
        );
    });
}

#[test]
fn split_position_fails_on_invalid_partition_length() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        // Market has three outcomes, but there's an element in the partition of size two.
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![_B1, _B0, _B1], vec![_B0, _B1]];

        assert_noop!(
            CombinatorialTokens::split_position(alice.signed(), None, market_id, partition, _1),
            Error::<Runtime>::InvalidPartition
        );
    });
}

#[test]
fn split_position_fails_on_empty_partition_member() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        // Second element is empty.
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![_B1, _B0, _B1], vec![_B0, _B0, _B0]];

        assert_noop!(
            CombinatorialTokens::split_position(alice.signed(), None, market_id, partition, _1,),
            Error::<Runtime>::InvalidPartition
        );
    });
}

#[test]
fn split_position_fails_on_overlapping_partition_members() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        // Last elements overlap.
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![_B1, _B0, _B1], vec![_B0, _B0, _B1]];

        assert_noop!(
            CombinatorialTokens::split_position(alice.signed(), None, market_id, partition, _1),
            Error::<Runtime>::InvalidPartition
        );
    });
}

#[test]
fn split_position_fails_on_trivial_partition() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![_B1, _B1, _B1]];

        assert_noop!(
            CombinatorialTokens::split_position(alice.signed(), None, market_id, partition, _1),
            Error::<Runtime>::InvalidPartition
        );
    });
}

#[test]
fn split_position_fails_on_insufficient_funds_native_token() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _99).unwrap();

        // Market has three outcomes, but there's an element in the partition of size two.
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));

        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                market_id,
                vec![vec![_B1, _B0, _B1], vec![_B0, _B1, _B0]],
                _100,
            ),
            orml_currencies::Error::<Runtime>::BalanceTooLow
        );
    });
}

#[test]
fn split_position_fails_on_insufficient_funds_foreign_token() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::ForeignAsset(1), _99).unwrap();

        // Market has three outcomes, but there's an element in the partition of size two.
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));

        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                market_id,
                vec![vec![_B1, _B0, _B1], vec![_B0, _B1, _B0]],
                _100,
            ),
            orml_currencies::Error::<Runtime>::BalanceTooLow
        );
    });
}
