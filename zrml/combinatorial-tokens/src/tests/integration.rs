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
fn split_followed_by_merge_vertical_no_parent() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();
        let pallet = Account::new(Pallet::<Runtime>::account_id());

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![B0, B0, B1], vec![B1, B1, B0]];
        let amount = _1;

        let ct_001 = CombinatorialToken([
            207, 168, 160, 93, 238, 221, 197, 1, 171, 102, 28, 24, 18, 107, 205, 231, 227, 98, 220,
            105, 211, 29, 181, 30, 53, 7, 200, 154, 134, 246, 38, 139,
        ]);
        let ct_110 = CombinatorialToken([
            101, 210, 61, 196, 5, 247, 150, 41, 186, 49, 11, 63, 139, 53, 25, 65, 161, 83, 24, 142,
            225, 102, 57, 241, 199, 18, 226, 137, 68, 3, 219, 131,
        ]);

        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            market_id,
            partition.clone(),
            amount,
            Fuel::new(16, false),
        ));
        assert_eq!(alice.free_balance(Asset::Ztg), _99);
        assert_eq!(alice.free_balance(ct_001), _1);
        assert_eq!(alice.free_balance(ct_110), _1);
        assert_eq!(pallet.free_balance(Asset::Ztg), _1);

        assert_ok!(CombinatorialTokens::merge_position(
            alice.signed(),
            None,
            market_id,
            partition,
            amount,
            Fuel::new(16, false),
        ));
        assert_eq!(alice.free_balance(Asset::Ztg), _100);
        assert_eq!(alice.free_balance(ct_001), 0);
        assert_eq!(alice.free_balance(ct_110), 0);
        assert_eq!(pallet.free_balance(Asset::Ztg), 0);
    });
}

#[test]
fn split_followed_by_merge_vertical_with_parent() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();
        let pallet = Account::new(Pallet::<Runtime>::account_id());

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

        let parent_market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let parent_amount = _3;
        let parent_partition = vec![vec![B0, B0, B1], vec![B1, B1, B0]];
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            parent_market_id,
            parent_partition.clone(),
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
        let child_partition = vec![vec![B0, B1, B0, B1], vec![B1, B0, B1, B0]];
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            Some(parent_collection_id),
            child_market_id,
            child_partition.clone(),
            child_amount,
            Fuel::new(16, false),
        ));
        assert_eq!(alice.free_balance(ct_001), parent_amount - child_amount);
        assert_eq!(alice.free_balance(ct_110), parent_amount);
        assert_eq!(alice.free_balance(Asset::Ztg), _100 - parent_amount);
        assert_eq!(alice.free_balance(ct_001_0101), child_amount);
        assert_eq!(alice.free_balance(ct_001_1010), child_amount);
        assert_eq!(pallet.free_balance(Asset::Ztg), parent_amount);

        assert_ok!(CombinatorialTokens::merge_position(
            alice.signed(),
            Some(parent_collection_id),
            child_market_id,
            child_partition,
            child_amount,
            Fuel::new(16, false),
        ));
        assert_eq!(alice.free_balance(ct_001), parent_amount);
        assert_eq!(alice.free_balance(ct_110), parent_amount);
        assert_eq!(alice.free_balance(Asset::Ztg), _100 - parent_amount);
        assert_eq!(alice.free_balance(ct_001_0101), 0);
        assert_eq!(alice.free_balance(ct_001_1010), 0);
        assert_eq!(pallet.free_balance(Asset::Ztg), parent_amount);

        assert_ok!(CombinatorialTokens::merge_position(
            alice.signed(),
            None,
            parent_market_id,
            parent_partition,
            parent_amount,
            Fuel::new(16, false),
        ));
        assert_eq!(alice.free_balance(ct_001), 0);
        assert_eq!(alice.free_balance(ct_110), 0);
        assert_eq!(alice.free_balance(Asset::Ztg), _100);
        assert_eq!(alice.free_balance(ct_001_0101), 0);
        assert_eq!(alice.free_balance(ct_001_1010), 0);
        assert_eq!(pallet.free_balance(Asset::Ztg), 0);
    });
}

#[test]
fn split_followed_by_merge_vertical_with_parent_in_opposite_order() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        let market_0 = create_market(Asset::Ztg, MarketType::Categorical(3));
        let market_1 = create_market(Asset::Ztg, MarketType::Categorical(4));

        let partition_0 = vec![vec![B0, B0, B1], vec![B1, B1, B0]];
        let partition_1 = vec![vec![B0, B0, B1, B1], vec![B1, B1, B0, B0]];

        let amount = _1;

        let ct_001 = CombinatorialToken([
            207, 168, 160, 93, 238, 221, 197, 1, 171, 102, 28, 24, 18, 107, 205, 231, 227, 98, 220,
            105, 211, 29, 181, 30, 53, 7, 200, 154, 134, 246, 38, 139,
        ]);
        let ct_110 = CombinatorialToken([
            101, 210, 61, 196, 5, 247, 150, 41, 186, 49, 11, 63, 139, 53, 25, 65, 161, 83, 24, 142,
            225, 102, 57, 241, 199, 18, 226, 137, 68, 3, 219, 131,
        ]);
        let id_001 = [
            6, 44, 173, 50, 122, 106, 144, 185, 253, 19, 252, 218, 215, 241, 218, 37, 196, 112, 45,
            133, 165, 48, 231, 189, 87, 123, 131, 18, 190, 5, 110, 93,
        ];
        let id_110 = [
            1, 189, 94, 224, 153, 162, 145, 214, 33, 231, 230, 19, 122, 179, 122, 117, 193, 123,
            73, 220, 240, 131, 180, 180, 137, 14, 179, 148, 188, 13, 107, 65,
        ];

        let ct_0011 = CombinatorialToken([
            32, 70, 65, 46, 183, 161, 122, 58, 80, 224, 102, 106, 63, 89, 191, 19, 235, 137, 64,
            182, 25, 222, 198, 172, 230, 42, 120, 101, 100, 150, 172, 125,
        ]);
        let ct_1100 = CombinatorialToken([
            28, 158, 82, 180, 87, 230, 168, 233, 74, 123, 50, 76, 131, 203, 82, 194, 214, 165, 87,
            200, 58, 244, 23, 184, 79, 127, 201, 39, 82, 243, 186, 1,
        ]);
        let id_0011 = [
            77, 83, 228, 134, 221, 156, 53, 34, 133, 83, 120, 8, 232, 53, 54, 200, 181, 110, 13,
            145, 238, 130, 69, 147, 108, 167, 41, 217, 105, 22, 126, 136,
        ];
        let id_1100 = [
            10, 211, 115, 219, 24, 177, 205, 243, 234, 68, 234, 119, 21, 211, 103, 229, 185, 23,
            63, 75, 206, 10, 196, 75, 10, 110, 147, 40, 90, 61, 145, 90,
        ];

        let ct_001_0011 = CombinatorialToken([
            156, 47, 254, 154, 29, 5, 149, 94, 214, 135, 92, 36, 188, 120, 42, 144, 136, 151, 255,
            91, 232, 152, 91, 236, 177, 66, 36, 72, 134, 234, 212, 177,
        ]);
        let ct_001_1100 = CombinatorialToken([
            224, 47, 73, 22, 156, 226, 199, 74, 28, 251, 44, 108, 73, 125, 192, 151, 193, 60, 156,
            240, 215, 23, 138, 168, 181, 175, 241, 70, 71, 126, 48, 45,
        ]);
        let ct_110_0011 = CombinatorialToken([
            191, 106, 159, 227, 136, 131, 143, 101, 127, 7, 109, 82, 45, 169, 246, 45, 250, 217,
            33, 147, 166, 174, 232, 35, 58, 20, 111, 167, 6, 6, 73, 67,
        ]);
        let ct_110_1100 = CombinatorialToken([
            184, 155, 104, 90, 231, 10, 30, 1, 213, 7, 1, 58, 117, 172, 118, 72, 118, 89, 219, 216,
            140, 27, 228, 2, 87, 26, 169, 150, 172, 154, 49, 219,
        ]);

        // Split ZTG into A|B and C.
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            market_0,
            partition_0.clone(),
            amount,
            Fuel::new(16, false),
        ));

        // Split C into C&(U|V) and C&(W|X).
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            Some(id_001),
            market_1,
            partition_1.clone(),
            amount,
            Fuel::new(16, false),
        ));

        // Split A|B into into (A|B)&(U|V) and (A|B)&(W|X).
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            Some(id_110),
            market_1,
            partition_1.clone(),
            amount,
            Fuel::new(16, false),
        ));

        assert_eq!(alice.free_balance(ct_001), 0);
        assert_eq!(alice.free_balance(ct_110), 0);
        assert_eq!(alice.free_balance(ct_001_0011), _1);
        assert_eq!(alice.free_balance(ct_001_1100), _1);
        assert_eq!(alice.free_balance(ct_110_0011), _1);
        assert_eq!(alice.free_balance(ct_110_1100), _1);
        assert_eq!(alice.free_balance(ct_0011), 0);
        assert_eq!(alice.free_balance(ct_1100), 0);
        assert_eq!(alice.free_balance(Asset::Ztg), _99);

        // Merge C&(U|V) and (A|B)&(U|V) into U|V.
        assert_ok!(CombinatorialTokens::merge_position(
            alice.signed(),
            Some(id_1100),
            market_0,
            partition_0.clone(),
            amount,
            Fuel::new(16, false),
        ));

        assert_eq!(alice.free_balance(ct_001), 0);
        assert_eq!(alice.free_balance(ct_110), 0);
        assert_eq!(alice.free_balance(ct_001_0011), _1);
        assert_eq!(alice.free_balance(ct_001_1100), 0);
        assert_eq!(alice.free_balance(ct_110_0011), _1);
        assert_eq!(alice.free_balance(ct_110_1100), 0);
        assert_eq!(alice.free_balance(ct_0011), 0);
        assert_eq!(alice.free_balance(ct_1100), _1);
        assert_eq!(alice.free_balance(Asset::Ztg), _99);

        // Merge C&(W|X) and (A|B)&(W|X) into W|X.
        assert_ok!(CombinatorialTokens::merge_position(
            alice.signed(),
            Some(id_0011),
            market_0,
            partition_0,
            amount,
            Fuel::new(16, false),
        ));

        assert_eq!(alice.free_balance(ct_001), 0);
        assert_eq!(alice.free_balance(ct_110), 0);
        assert_eq!(alice.free_balance(ct_001_0011), 0);
        assert_eq!(alice.free_balance(ct_001_1100), 0);
        assert_eq!(alice.free_balance(ct_110_0011), 0);
        assert_eq!(alice.free_balance(ct_110_1100), 0);
        assert_eq!(alice.free_balance(ct_0011), _1);
        assert_eq!(alice.free_balance(ct_1100), _1);
        assert_eq!(alice.free_balance(Asset::Ztg), _99);

        // Merge U|V and W|X into ZTG.
        assert_ok!(CombinatorialTokens::merge_position(
            alice.signed(),
            None,
            market_1,
            partition_1,
            amount,
            Fuel::new(16, false),
        ));

        assert_eq!(alice.free_balance(ct_001), 0);
        assert_eq!(alice.free_balance(ct_110), 0);
        assert_eq!(alice.free_balance(ct_001_0011), 0);
        assert_eq!(alice.free_balance(ct_001_1100), 0);
        assert_eq!(alice.free_balance(ct_110_0011), 0);
        assert_eq!(alice.free_balance(ct_110_1100), 0);
        assert_eq!(alice.free_balance(ct_0011), 0);
        assert_eq!(alice.free_balance(ct_1100), 0);
        assert_eq!(alice.free_balance(Asset::Ztg), _100);
    });
}

// This test shows that splitting a token horizontally can be accomplished by splitting the parent
// token vertically with a finer partition.
#[test]
fn split_vertical_followed_by_horizontal_split_no_parent() {
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
            Fuel::new(16, false),
        ));
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            market_id,
            vec![vec![B1, B0, B0], vec![B0, B1, B0]],
            amount,
            Fuel::new(16, false),
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
            Fuel::new(16, false),
        ));

        assert_eq!(alice.free_balance(ct_001), 2 * amount);
        assert_eq!(alice.free_balance(ct_010), 2 * amount);
        assert_eq!(alice.free_balance(ct_100), 2 * amount);
    });
}

// This test shows that splitting a token horizontally can be accomplished by splitting a the parent
// token vertically with a finer partition.
#[test]
fn split_vertical_followed_by_horizontal_split_with_parent() {
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
            Fuel::new(16, false),
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
            Fuel::new(16, false),
        ));
        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            Some(parent_collection_id),
            child_market_id,
            vec![vec![B1, B0, B0, B0], vec![B0, B1, B0, B0]],
            child_amount_first_pass,
            Fuel::new(16, false),
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
            Fuel::new(16, false),
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

#[test]
fn split_horizontal_followed_by_merge_horizontal() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let amount = _1;

        let ct_001 = CombinatorialToken([
            207, 168, 160, 93, 238, 221, 197, 1, 171, 102, 28, 24, 18, 107, 205, 231, 227, 98, 220,
            105, 211, 29, 181, 30, 53, 7, 200, 154, 134, 246, 38, 139,
        ]);
        let ct_110 = CombinatorialToken([
            101, 210, 61, 196, 5, 247, 150, 41, 186, 49, 11, 63, 139, 53, 25, 65, 161, 83, 24, 142,
            225, 102, 57, 241, 199, 18, 226, 137, 68, 3, 219, 131,
        ]);

        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            market_id,
            vec![vec![B0, B0, B1], vec![B1, B1, B0]],
            amount,
            Fuel::new(16, false),
        ));

        assert_ok!(CombinatorialTokens::split_position(
            alice.signed(),
            None,
            market_id,
            vec![vec![B1, B0, B0], vec![B0, B1, B0]],
            amount,
            Fuel::new(16, false),
        ));

        assert_ok!(CombinatorialTokens::merge_position(
            alice.signed(),
            None,
            market_id,
            vec![vec![B1, B0, B0], vec![B0, B1, B0]],
            amount,
            Fuel::new(16, false),
        ));

        assert_eq!(alice.free_balance(ct_001), _1);
        assert_eq!(alice.free_balance(ct_110), _1);
    });
}
