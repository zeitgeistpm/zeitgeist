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
use test_case::test_case;

#[test_case(
    Asset::Ztg,
    CombinatorialToken([207, 168, 160, 93, 238, 221, 197, 1, 171, 102, 28, 24, 18, 107, 205, 231, 227, 98, 220, 105, 211, 29, 181, 30, 53, 7, 200, 154, 134, 246, 38, 139]),
    CombinatorialToken([101, 210, 61, 196, 5, 247, 150, 41, 186, 49, 11, 63, 139, 53, 25, 65, 161, 83, 24, 142, 225, 102, 57, 241, 199, 18, 226, 137, 68, 3, 219, 131])
)]
#[test_case(
    Asset::ForeignAsset(1),
    CombinatorialToken([97, 71, 129, 186, 219, 73, 163, 242, 183, 111, 224, 26, 45, 104, 11, 229, 241, 31, 154, 126, 118, 218, 142, 191, 3, 255, 156, 77, 32, 1, 66, 227]),
    CombinatorialToken([156, 42, 42, 43, 18, 242, 8, 247, 100, 196, 173, 111, 167, 225, 207, 149, 166, 194, 255, 1, 238, 128, 72, 199, 188, 57, 236, 168, 26, 58, 104, 156])
)]
fn merge_position_works_no_parent(
    collateral: Asset<MarketId>,
    ct_001: Asset<MarketId>,
    ct_110: Asset<MarketId>,
) {
    ExtBuilder::build().execute_with(|| {
        let amount = _100;
        let alice =
            Account::new(0).deposit(ct_001, amount).unwrap().deposit(ct_110, amount).unwrap();
        // Mock a deposit into the pallet's account.
        let pallet =
            Account::new(Pallet::<Runtime>::account_id()).deposit(collateral, amount).unwrap();

        let parent_collection_id = None;
        let market_id = create_market(collateral, MarketType::Categorical(3));
        let partition = vec![vec![B0, B0, B1], vec![B1, B1, B0]];
        assert_ok!(CombinatorialTokens::merge_position(
            alice.signed(),
            parent_collection_id,
            market_id,
            partition.clone(),
            amount,
            Fuel::new(16, false),
        ));

        assert_eq!(alice.free_balance(ct_001), 0);
        assert_eq!(alice.free_balance(ct_110), 0);
        assert_eq!(alice.free_balance(collateral), _100);
        assert_eq!(pallet.free_balance(collateral), 0);

        System::assert_last_event(
            Event::<Runtime>::TokenMerged {
                who: alice.id,
                parent_collection_id,
                market_id,
                partition,
                assets_in: vec![ct_001, ct_110],
                asset_out: collateral,
                amount,
            }
            .into(),
        );
    });
}

#[test]
fn merge_position_works_parent() {
    ExtBuilder::build().execute_with(|| {
        let ct_001 = CombinatorialToken([
            207, 168, 160, 93, 238, 221, 197, 1, 171, 102, 28, 24, 18, 107, 205, 231, 227, 98, 220,
            105, 211, 29, 181, 30, 53, 7, 200, 154, 134, 246, 38, 139,
        ]);
        let ct_001_0101 = CombinatorialToken([
            38, 14, 141, 152, 199, 40, 88, 165, 208, 236, 195, 198, 208, 75, 93, 85, 114, 4, 175,
            225, 211, 72, 142, 210, 98, 202, 168, 193, 245, 217, 239, 28,
        ]);
        let ct_001_1010 = CombinatorialToken([
            107, 142, 3, 38, 49, 137, 237, 239, 1, 131, 197, 221, 236, 46, 246, 93, 185, 197, 228,
            184, 75, 79, 107, 73, 89, 19, 22, 124, 15, 58, 110, 100,
        ]);

        let amount = _100;
        let alice = Account::new(0)
            .deposit(ct_001_0101, amount)
            .unwrap()
            .deposit(ct_001_1010, amount)
            .unwrap();

        let _ = create_market(Asset::Ztg, MarketType::Categorical(3));

        // Collection ID of [0, 0, 1].
        let parent_collection_id = Some([
            6, 44, 173, 50, 122, 106, 144, 185, 253, 19, 252, 218, 215, 241, 218, 37, 196, 112, 45,
            133, 165, 48, 231, 189, 87, 123, 131, 18, 190, 5, 110, 93,
        ]);
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(4));
        let partition = vec![vec![B0, B1, B0, B1], vec![B1, B0, B1, B0]];
        assert_ok!(CombinatorialTokens::merge_position(
            alice.signed(),
            parent_collection_id,
            market_id,
            partition.clone(),
            amount,
            Fuel::new(16, false),
        ));

        assert_eq!(alice.free_balance(ct_001), amount);
        assert_eq!(alice.free_balance(ct_001_0101), 0);
        assert_eq!(alice.free_balance(ct_001_1010), 0);

        System::assert_last_event(
            Event::<Runtime>::TokenMerged {
                who: alice.id,
                parent_collection_id,
                market_id,
                partition,
                assets_in: vec![ct_001_0101, ct_001_1010],
                asset_out: ct_001,
                amount,
            }
            .into(),
        );
    });
}

#[test]
fn merge_position_horizontal_works() {
    ExtBuilder::build().execute_with(|| {
        let ct_100 = CombinatorialToken([
            63, 95, 93, 48, 199, 160, 113, 178, 33, 24, 52, 193, 247, 121, 229, 30, 231, 100, 209,
            14, 57, 98, 193, 214, 34, 251, 53, 51, 136, 146, 93, 26,
        ]);
        let ct_010 = CombinatorialToken([
            23, 108, 101, 109, 145, 51, 201, 192, 240, 28, 43, 57, 53, 4, 75, 101, 116, 20, 184,
            25, 227, 71, 149, 136, 59, 82, 81, 105, 41, 160, 39, 142,
        ]);
        let ct_110 = CombinatorialToken([
            101, 210, 61, 196, 5, 247, 150, 41, 186, 49, 11, 63, 139, 53, 25, 65, 161, 83, 24, 142,
            225, 102, 57, 241, 199, 18, 226, 137, 68, 3, 219, 131,
        ]);

        let amount = _100;
        let alice = Account::new(0).deposit(ct_100, _100).unwrap().deposit(ct_010, _100).unwrap();

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));

        assert_ok!(CombinatorialTokens::merge_position(
            alice.signed(),
            None,
            market_id,
            vec![vec![B0, B1, B0], vec![B1, B0, B0]],
            amount,
            Fuel::new(16, false),
        ));

        assert_eq!(alice.free_balance(ct_110), amount);
        assert_eq!(alice.free_balance(ct_100), 0);
        assert_eq!(alice.free_balance(ct_010), 0);
    });
}

#[test]
fn merge_position_fails_if_market_not_found() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        assert_noop!(
            CombinatorialTokens::merge_position(
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
fn merge_position_fails_on_invalid_partition_length() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        // Market has three outcomes, but there's an element in the partition of size two.
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![B1, B0, B1], vec![B0, B1]];

        assert_noop!(
            CombinatorialTokens::merge_position(
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
fn merge_position_fails_on_trivial_partition_member() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![B1, B0, B1], vec![B0, B0, B0]];

        assert_noop!(
            CombinatorialTokens::merge_position(
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
fn merge_position_fails_on_overlapping_partition_members() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let partition = vec![vec![B1, B0, B1], vec![B0, B0, B1]];

        assert_noop!(
            CombinatorialTokens::merge_position(
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
fn merge_position_fails_on_insufficient_funds() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _99).unwrap();

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));

        assert_noop!(
            CombinatorialTokens::merge_position(
                alice.signed(),
                None,
                market_id,
                vec![vec![B1, B0, B1], vec![B0, B1, B0]],
                _100,
                Fuel::new(16, false),
            ),
            orml_tokens::Error::<Runtime>::BalanceTooLow
        );
    });
}

#[test]
fn merge_position_fails_on_insufficient_funds_foreign_token() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::ForeignAsset(1), _99).unwrap();

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));

        assert_noop!(
            CombinatorialTokens::merge_position(
                alice.signed(),
                None,
                market_id,
                vec![vec![B1, B0, B1], vec![B0, B1, B0]],
                _100,
                Fuel::new(16, false),
            ),
            orml_tokens::Error::<Runtime>::BalanceTooLow
        );
    });
}
