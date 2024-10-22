// Copyright 2024 Forecasting Technologies LTD.
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

#[test]
fn redeem_position_fails_on_no_payout_vector() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();
        MockPayout::set_return_value(None);
        assert_noop!(
            CombinatorialTokens::redeem_position(alice.signed(), None, 0, vec![]),
            Error::<Runtime>::PayoutVectorNotFound
        );
    });
}

#[test]
fn redeem_position_fails_on_market_not_found() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();
        MockPayout::set_return_value(Some(vec![_1_2, _1_2]));
        assert_noop!(
            CombinatorialTokens::redeem_position(alice.signed(), None, 0, vec![]),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test_case(vec![B0, B1, B0, B1]; "incorrect_len")]
#[test_case(vec![B0, B0, B0]; "all_zero")]
#[test_case(vec![B0, B0, B0]; "all_one")]
fn redeem_position_fails_on_incorrect_index_set(index_set: Vec<bool>) {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();
        MockPayout::set_return_value(Some(vec![_1_3, _1_3, _1_3]));
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        assert_noop!(
            CombinatorialTokens::redeem_position(alice.signed(), None, market_id, index_set),
            Error::<Runtime>::InvalidIndexSet
        );
    });
}

#[test]
fn redeem_position_fails_if_tokens_have_to_value() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();
        MockPayout::set_return_value(Some(vec![0, _1_2, _1_2, 0]));
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(4));
        let index_set = vec![B1, B0, B0, B1];
        assert_noop!(
            CombinatorialTokens::redeem_position(alice.signed(), None, market_id, index_set),
            Error::<Runtime>::TokenHasNoValue
        );
    });
}

#[test]
fn redeem_position_fails_if_user_holds_no_winning_tokens() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0).deposit(Asset::Ztg, _100).unwrap();
        MockPayout::set_return_value(Some(vec![0, _1_2, _1_2, 0]));
        let market_id = create_market(Asset::Ztg, MarketType::Categorical(4));
        let index_set = vec![B0, B1, B0, B1];
        assert_noop!(
            CombinatorialTokens::redeem_position(alice.signed(), None, market_id, index_set),
            Error::<Runtime>::NoTokensFound,
        );
    });
}

#[test]
fn redeem_position_works_sans_parent() {
    ExtBuilder::build().execute_with(|| {
        let ct_110 = CombinatorialToken([
            101, 210, 61, 196, 5, 247, 150, 41, 186, 49, 11, 63, 139, 53, 25, 65, 161, 83, 24, 142,
            225, 102, 57, 241, 199, 18, 226, 137, 68, 3, 219, 131,
        ]);
        let alice = Account::new(0).deposit(ct_110, _3).unwrap();
        let pallet = Account::new(Pallet::<Runtime>::account_id()).deposit(Asset::Ztg, _3).unwrap();

        MockPayout::set_return_value(Some(vec![_1_4, _1_2, _1_4]));

        let market_id = create_market(Asset::Ztg, MarketType::Categorical(3));
        let index_set = vec![B1, B1, B0];
        assert_ok!(CombinatorialTokens::redeem_position(
            alice.signed(),
            None,
            market_id,
            index_set
        ));

        assert_eq!(alice.free_balance(ct_110), 0);
        assert_eq!(alice.free_balance(Asset::Ztg), _2 + _1_4);
        assert_eq!(pallet.free_balance(Asset::Ztg), _3_4);
    });
}

#[test]
fn redeem_position_works_with_parent() {
    ExtBuilder::build().execute_with(|| {
        let ct_001 = CombinatorialToken([
            207, 168, 160, 93, 238, 221, 197, 1, 171, 102, 28, 24, 18, 107, 205, 231, 227, 98, 220,
            105, 211, 29, 181, 30, 53, 7, 200, 154, 134, 246, 38, 139,
        ]);
        let ct_001_0101 = CombinatorialToken([
            38, 14, 141, 152, 199, 40, 88, 165, 208, 236, 195, 198, 208, 75, 93, 85, 114, 4, 175,
            225, 211, 72, 142, 210, 98, 202, 168, 193, 245, 217, 239, 28,
        ]);

        let alice = Account::new(0).deposit(ct_001_0101, _7).unwrap();

        MockPayout::set_return_value(Some(vec![_1_4, 0, _1_2, _1_4]));

        let _ = create_market(Asset::Ztg, MarketType::Categorical(3));
        let child_market_id = create_market(Asset::Ztg, MarketType::Categorical(4));

        // Collection ID of [0, 0, 1].
        let parent_collection_id = [
            6, 44, 173, 50, 122, 106, 144, 185, 253, 19, 252, 218, 215, 241, 218, 37, 196, 112, 45,
            133, 165, 48, 231, 189, 87, 123, 131, 18, 190, 5, 110, 93,
        ];
        let index_set = vec![B0, B1, B0, B1];
        assert_ok!(CombinatorialTokens::redeem_position(
            alice.signed(),
            Some(parent_collection_id),
            child_market_id,
            index_set
        ));

        assert_eq!(alice.free_balance(ct_001_0101), 0);
        assert_eq!(alice.free_balance(ct_001), _1 + _3_4);
    });
}
