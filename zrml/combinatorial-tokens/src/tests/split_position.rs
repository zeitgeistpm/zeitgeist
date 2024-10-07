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
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                market_id,
                partition,
                _1,
            ),
            Error::<Runtime>::InvalidPartition
        );
    });
}

#[test]
fn split_position_fails_on_trivial_partition_member() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        // Market has three outcomes, but there's an element in the partition of size two.
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![_B1, _B0, _B1], vec![_B0, _B0, _B0]];

        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                market_id,
                partition,
                _1,
            ),
            Error::<Runtime>::InvalidPartition
        );
    });
}

#[test]
fn split_position_fails_on_overlapping_partition_members() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        // Market has three outcomes, but there's an element in the partition of size two.
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![_B1, _B0, _B1], vec![_B0, _B0, _B1]];

        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                market_id,
                partition,
                _1,
            ),
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
