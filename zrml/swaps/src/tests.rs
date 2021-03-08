use crate::{
    GenericPoolEvent,
    mock::*
};
use frame_support::{assert_noop, assert_ok};
use sp_core::H256;
use zrml_traits::shares::Shares as SharesTrait;

pub const ASSET_A: H256 = H256::repeat_byte(65);
pub const ASSET_B: H256 = H256::repeat_byte(66);
pub const ASSET_C: H256 = H256::repeat_byte(67);
pub const ASSET_D: H256 = H256::repeat_byte(68);
pub const ASSET_E: H256 = H256::repeat_byte(69);

pub const ASSETS: [H256; 4] = [ASSET_A, ASSET_B, ASSET_C, ASSET_D];

#[test]
fn ensure_emitted_events() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Module::<Test>::set_block_number(1);
        create_initial_pool_with_initial_asset_funds();

        let alice = Origin::signed(ALICE);
        let pool_id = 0;

        let _ = Swaps::join_pool(alice.clone(), pool_id, BASE, vec!(BASE, BASE, BASE, BASE));
        let joined_pool_event = GenericPoolEvent { pool_id, who: 0 };
        let event = TestEvent::zrml_swaps(crate::RawEvent::JoinedPool(joined_pool_event));
        frame_system::Module::<Test>::events().iter().any(|e| e.event == event);

        let _ = Swaps::exit_pool(alice, pool_id, BASE, vec!(BASE, BASE, BASE, BASE));
        let joined_pool_event = GenericPoolEvent { pool_id, who: 0 };
        let event = TestEvent::zrml_swaps(crate::RawEvent::ExitedPool(joined_pool_event));
        frame_system::Module::<Test>::events().iter().any(|e| e.event == event);
    });
}

#[test]
fn it_creates_a_new_pool_external() {
    ExtBuilder::default().build().execute_with(|| {
        let next_pool_before = Swaps::next_pool_id();
        assert_eq!(next_pool_before, 0);

        create_initial_pool();

        let next_pool_after = Swaps::next_pool_id();
        assert_eq!(next_pool_after, 1);

        let pool = Swaps::pools(0).unwrap();

        assert_eq!(pool.assets, ASSETS.iter().cloned().collect::<Vec<_>>());
        assert_eq!(pool.swap_fee, 0);
        assert_eq!(pool.total_weight, 8 * BASE);

        assert_eq!(*pool.weights.get(&ASSET_A).unwrap(), 2 * BASE);
        assert_eq!(*pool.weights.get(&ASSET_B).unwrap(), 2 * BASE);
        assert_eq!(*pool.weights.get(&ASSET_C).unwrap(), 2 * BASE);
        assert_eq!(*pool.weights.get(&ASSET_D).unwrap(), 2 * BASE);
    });
}

#[test]
fn it_allows_the_full_user_lifecycle() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool_with_initial_asset_funds();

        assert_ok!(Swaps::join_pool(
            Origin::signed(ALICE),
            0,
            5 * BASE,
            vec!(25 * BASE, 25 * BASE, 25 * BASE, 25 * BASE),
        ));

        let pool_shares_id = Swaps::pool_shares_id(0);
        let balance = Shares::free_balance(pool_shares_id, &ALICE);
        assert_eq!(balance, 5 * BASE);

        let asset_a_bal = Shares::free_balance(ASSET_A, &ALICE);
        let asset_b_bal = Shares::free_balance(ASSET_B, &ALICE);
        let asset_c_bal = Shares::free_balance(ASSET_C, &ALICE);
        let asset_d_bal = Shares::free_balance(ASSET_D, &ALICE);
        assert_eq!(asset_a_bal, asset_b_bal);
        assert_eq!(asset_b_bal, asset_c_bal);
        assert_eq!(asset_c_bal, asset_d_bal);
        assert_eq!(asset_a_bal, 20 * BASE);

        // swap_exact_amount_in
        let spot_price = Swaps::get_spot_price(0, ASSET_A, ASSET_B);
        assert_eq!(spot_price, BASE);

        let pool_account = Swaps::pool_account_id(0);

        let in_balance = Shares::free_balance(ASSET_A, &pool_account);
        assert_eq!(in_balance, 105 * BASE);

        let expected = crate::math::calc_out_given_in(
            in_balance,
            2 * BASE,
            Shares::free_balance(ASSET_B, &pool_account),
            2 * BASE,
            BASE,
            0,
        );

        assert_ok!(Swaps::swap_exact_amount_in(
            Origin::signed(ALICE),
            0,
            ASSET_A,
            BASE,
            ASSET_B,
            BASE / 2,
            2 * BASE,
        ));

        let asset_a_bal_after = Shares::free_balance(ASSET_A, &ALICE);
        assert_eq!(asset_a_bal_after, asset_a_bal - BASE);

        let asset_b_bal_after = Shares::free_balance(ASSET_B, &ALICE);
        assert_eq!(asset_b_bal_after - asset_b_bal, expected);

        assert_eq!(expected, 9_905_660_415);

        //swap_exact_amount_out
        let expected_in = crate::math::calc_in_given_out(
            Shares::free_balance(ASSET_A, &pool_account),
            2 * BASE,
            Shares::free_balance(ASSET_B, &pool_account),
            2 * BASE,
            BASE,
            0,
        );

        assert_eq!(expected_in, 10_290_319_622);

        assert_ok!(Swaps::swap_exact_amount_out(
            Origin::signed(ALICE),
            0,
            ASSET_A,
            2 * BASE,
            ASSET_B,
            BASE,
            3 * BASE,
        ));

        let asset_a_bal_after_2 = Shares::free_balance(ASSET_A, &ALICE);
        assert_eq!(asset_a_bal_after_2, asset_a_bal_after - expected_in);

        let asset_b_bal_after_2 = Shares::free_balance(ASSET_B, &ALICE);
        assert_eq!(asset_b_bal_after_2 - asset_b_bal_after, BASE);
    });
}

#[test]
fn provided_values_len_must_equal_assets_len() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool();
        let signed = Origin::signed(ALICE);
        assert_noop!(
            Swaps::join_pool(signed.clone(), 0, 5 * BASE, vec![]),
            crate::Error::<Test>::ProvidedValuesLenMustEqualAssetsLen
        );
        assert_noop!(
            Swaps::exit_pool(signed, 0, 5 * BASE, vec![]),
            crate::Error::<Test>::ProvidedValuesLenMustEqualAssetsLen
        );
    });
}

#[test]
fn assets_must_be_bounded() {
    ExtBuilder::default().build().execute_with(|| {
        create_initial_pool();
        let signed = Origin::signed(ALICE);
        assert_noop!(
            Swaps::swap_exact_amount_in(signed.clone(), 0, ASSET_A, 1, ASSET_E, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );
        assert_noop!(
            Swaps::swap_exact_amount_in(signed.clone(), 0, ASSET_E, 1, ASSET_A, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );

        assert_noop!(
            Swaps::swap_exact_amount_out(signed.clone(), 0, ASSET_A, 1, ASSET_E, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );
        assert_noop!(
            Swaps::swap_exact_amount_out(signed.clone(), 0, ASSET_E, 1, ASSET_A, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );

        assert_noop!(
            Swaps::joinswap_extern_amount_in(signed.clone(), 0, ASSET_E, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );

        assert_noop!(
            Swaps::exitswap_pool_amount_in(signed.clone(), 0, ASSET_E, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );
        
        assert_noop!(
            Swaps::exitswap_extern_amount_out(signed, 0, ASSET_E, 1, 1),
            crate::Error::<Test>::AssetNotBound
        );
    });
}

fn create_initial_pool() {
    ASSETS.iter().cloned().for_each(|asset| {
        let _ = Shares::generate(asset, &BOB, 100 * BASE);
    });
    assert_ok!(Swaps::create_pool(
        Origin::signed(BOB),
        ASSETS.iter().cloned().collect(),
        vec!(2 * BASE, 2 * BASE, 2 * BASE, 2 * BASE),
    ));
}

fn create_initial_pool_with_initial_asset_funds() {
    create_initial_pool();
    let _ = Shares::generate(ASSET_A, &ALICE, 25 * BASE);
    let _ = Shares::generate(ASSET_B, &ALICE, 25 * BASE);
    let _ = Shares::generate(ASSET_C, &ALICE, 25 * BASE);
    let _ = Shares::generate(ASSET_D, &ALICE, 25 * BASE);
}