// Copyright 2024-2025 Forecasting Technologies LTD.
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

use super::*;

#[test]
fn split_position_works_vertical_no_parent() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();
        let pallet = Account::new(Pallet::<Runtime>::account_id());

        let parent_collection_id = None;
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![B0, B0, B1], vec![B1, B1, B0]];

        let amount = _1;
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            parent_collection_id,
            market_id,
            partition.clone(),
            amount,
            Fuel::new(16, false),
        ));

        let ct_001 = CombinatorialToken([
            207, 168, 160, 93, 238, 221, 197, 1, 171, 102, 28, 24, 18, 107, 205, 231, 227, 98, 220,
            105, 211, 29, 181, 30, 53, 7, 200, 154, 134, 246, 38, 139,
        ]);
        let ct_110 = CombinatorialToken([
            101, 210, 61, 196, 5, 247, 150, 41, 186, 49, 11, 63, 139, 53, 25, 65, 161, 83, 24, 142,
            225, 102, 57, 241, 199, 18, 226, 137, 68, 3, 219, 131,
        ]);

        assert_eq!(alice.free_balance(ct_001), amount);
        assert_eq!(alice.free_balance(ct_110), amount);
        assert_eq!(alice.free_balance(Asset::Ztg), _100 - amount);
        assert_eq!(pallet.free_balance(Asset::Ztg), amount);

        System::assert_last_event(
            Event::<Runtime>::TokenSplit {
                who: alice.id,
                parent_collection_id,
                market_id,
                partition,
                asset_in: Asset::Ztg,
                assets_out: vec![ct_001, ct_110],
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
fn split_position_works_vertical_with_parent() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();
        let pallet = Account::new(Pallet::<Runtime>::account_id());

        let parent_market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let parent_amount = _3;
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            parent_market_id,
            vec![vec![B0, B0, B1], vec![B1, B1, B0]],
            parent_amount,
            Fuel::new(16, false),
        ));

        let child_market_id = create_market(Asset::Ztg, MarketType::Categorical(4));
        let child_amount = _1;
        // Collection ID of [0, 0, 1].
        let parent_collection_id = [
            6, 44, 173, 50, 122, 106, 144, 185, 253, 19, 252, 218, 215, 241, 218, 37, 196, 112, 45,
            133, 165, 48, 231, 189, 87, 123, 131, 18, 190, 5, 110, 93,
        ];
        let partition = vec![vec![B0, B1, B0, B1], vec![B1, B0, B1, B0]];
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            Some(parent_collection_id),
            child_market_id,
            partition.clone(),
            child_amount,
            Fuel::new(16, false),
        ));

        // Alice is left with 2 units of [0, 0, 1], 3 units of [1, 1, 0] and one unit of each of the
        // two new tokens.
        let ct_001 = CombinatorialToken([
            207, 168, 160, 93, 238, 221, 197, 1, 171, 102, 28, 24, 18, 107, 205, 231, 227, 98, 220,
            105, 211, 29, 181, 30, 53, 7, 200, 154, 134, 246, 38, 139,
        ]);
        let ct_110 = CombinatorialToken([
            101, 210, 61, 196, 5, 247, 150, 41, 186, 49, 11, 63, 139, 53, 25, 65, 161, 83, 24, 142,
            225, 102, 57, 241, 199, 18, 226, 137, 68, 3, 219, 131,
        ]);
        let ct_001_0101 = CombinatorialToken([
            38, 14, 141, 152, 199, 40, 88, 165, 208, 236, 195, 198, 208, 75, 93, 85, 114, 4, 175,
            225, 211, 72, 142, 210, 98, 202, 168, 193, 245, 217, 239, 28,
        ]);
        let ct_001_1010 = CombinatorialToken([
            107, 142, 3, 38, 49, 137, 237, 239, 1, 131, 197, 221, 236, 46, 246, 93, 185, 197, 228,
            184, 75, 79, 107, 73, 89, 19, 22, 124, 15, 58, 110, 100,
        ]);

        assert_eq!(alice.free_balance(Asset::Ztg), _100 - parent_amount);
        assert_eq!(alice.free_balance(ct_001), parent_amount - child_amount);
        assert_eq!(alice.free_balance(ct_110), parent_amount);
        assert_eq!(alice.free_balance(ct_001_0101), child_amount);
        assert_eq!(alice.free_balance(ct_001_1010), child_amount);
        assert_eq!(pallet.free_balance(Asset::Ztg), parent_amount);
        assert_eq!(pallet.free_balance(ct_001), 0); // Combinatorial tokens are destroyed when split.

        System::assert_last_event(
            Event::<Runtime>::TokenSplit {
                who: alice.id,
                parent_collection_id: Some(parent_collection_id),
                market_id: child_market_id,
                partition,
                asset_in: ct_001,
                assets_out: vec![ct_001_0101, ct_001_1010],
                collection_ids: vec![
                    [
                        93, 24, 254, 39, 137, 146, 204, 128, 95, 226, 32, 110, 212, 68, 65, 13,
                        128, 86, 96, 119, 117, 240, 144, 57, 224, 160, 106, 176, 250, 172, 157, 47,
                    ],
                    [
                        98, 123, 162, 148, 54, 175, 126, 250, 173, 76, 229, 156, 108, 125, 245, 68,
                        132, 230, 48, 72, 247, 45, 233, 27, 100, 225, 243, 113, 21, 69, 45, 113,
                    ],
                ],
                amount: child_amount,
            }
            .into(),
        );
    });
}

// Intentionally left out as it is covered by
// `integration::vertical_split_followed_by_horizontal_split_no_parent`.
// #[test]
// fn split_position_works_horizontal_no_parent() {}

// Intentionally left out as it is covered by
// `integration::vertical_split_followed_by_horizontal_split_with_parent`.
// #[test]
// fn split_position_works_horizontal_with_parent() {}

#[test]
fn split_position_fails_if_market_not_found() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();
        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                0,
                vec![vec![B0, B0, B1], vec![B1, B1, B0]],
                1,
                Fuel::new(16, false),
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
        let partition = vec![vec![B1, B0, B1], vec![B0, B1]];

        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                market_id,
                partition,
                _1,
                Fuel::new(16, false),
            ),
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
        let partition = vec![vec![B1, B0, B1], vec![B0, B0, B0]];

        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                market_id,
                partition,
                _1,
                Fuel::new(16, false)
            ),
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
        let partition = vec![vec![B1, B0, B1], vec![B0, B0, B1]];

        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                market_id,
                partition,
                _1,
                Fuel::new(16, false),
            ),
            Error::<Runtime>::InvalidPartition
        );
    });
}

#[test]
fn split_position_fails_on_trivial_partition() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![B1, B1, B1]];

        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                market_id,
                partition,
                _1,
                Fuel::new(16, false)
            ),
            Error::<Runtime>::InvalidPartition
        );
    });
}

#[test]
fn split_position_fails_on_insufficient_funds_native_token_no_parent() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _99).unwrap();

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));

        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                market_id,
                vec![vec![B1, B0, B1], vec![B0, B1, B0]],
                _100,
                Fuel::new(16, false),
            ),
            orml_currencies::Error::<Runtime>::BalanceTooLow
        );
    });
}

#[test]
fn split_position_fails_on_insufficient_funds_foreign_token_no_parent() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::ForeignAsset(1), _99).unwrap();

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));

        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                market_id,
                vec![vec![B1, B0, B1], vec![B0, B1, B0]],
                _100,
                Fuel::new(16, false),
            ),
            orml_currencies::Error::<Runtime>::BalanceTooLow
        );
    });
}

#[test]
fn split_position_vertical_fails_on_insufficient_funds_combinatorial_token() {
    ExtBuilder::build().execute_with(|| {
        let ct_001 = CombinatorialToken([
            207, 168, 160, 93, 238, 221, 197, 1, 171, 102, 28, 24, 18, 107, 205, 231, 227, 98, 220,
            105, 211, 29, 181, 30, 53, 7, 200, 154, 134, 246, 38, 139,
        ]);

        let alice = Account::new(0).deposit(ct_001, _99).unwrap();

        // Collection ID of [0, 0, 1].
        let parent_collection_id = [
            6, 44, 173, 50, 122, 106, 144, 185, 253, 19, 252, 218, 215, 241, 218, 37, 196, 112, 45,
            133, 165, 48, 231, 189, 87, 123, 131, 18, 190, 5, 110, 93,
        ];

        let _ = create_market(Asset::Ztg, MarketType::Categorical(3));
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(4));

        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                Some(parent_collection_id),
                market_id,
                vec![vec![B1, B0, B1, B0], vec![B0, B1, B0, B1]],
                _100,
                Fuel::new(16, false),
            ),
            orml_tokens::Error::<Runtime>::BalanceTooLow
        );

        // Make sure that we're testing for the right balance. This call should work!
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            Some(parent_collection_id),
            market_id,
            vec![vec![B1, B0, B1, B0], vec![B0, B1, B0, B1]],
            _99,
            Fuel::new(16, false),
        ));
    });
}

#[test]
fn split_position_horizontal_fails_on_insufficient_funds_combinatorial_token() {
    ExtBuilder::build().execute_with(|| {
        let ct_110 = CombinatorialToken([
            101, 210, 61, 196, 5, 247, 150, 41, 186, 49, 11, 63, 139, 53, 25, 65, 161, 83, 24, 142,
            225, 102, 57, 241, 199, 18, 226, 137, 68, 3, 219, 131,
        ]);

        let alice = Account::new(0).deposit(ct_110, _99).unwrap();

        // Market has three outcomes, but there's an element in the partition of size two.
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));

        assert_noop!(
            CombinatorialTokens::split_position(
                alice.signed(),
                None,
                market_id,
                vec![vec![B1, B0, B0], vec![B0, B1, B0]],
                _100,
                Fuel::new(16, false),
            ),
            orml_tokens::Error::<Runtime>::BalanceTooLow
        );

        // Make sure that we're testing for the right balance. This call should work!
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            market_id,
            vec![vec![B1, B0, B0], vec![B0, B1, B0]],
            _99,
            Fuel::new(16, false),
        ));
    });
}
