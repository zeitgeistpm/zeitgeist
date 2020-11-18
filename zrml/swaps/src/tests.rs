use crate::{mock::*, Error};
use frame_support::{
    assert_noop, assert_ok,
    dispatch::DispatchError,
};
use sp_core::H256;
use zrml_traits::shares::Shares as SharesTrait;

pub const ASSET_A: H256 = H256::repeat_byte(65);
pub const ASSET_B: H256 = H256::repeat_byte(66);
pub const ASSET_C: H256 = H256::repeat_byte(67);
pub const ASSET_D: H256 = H256::repeat_byte(68);

#[test]
fn it_creates_a_new_pool_internal() {
    ExtBuilder::default().build().execute_with(|| {
        
        let next_pool_before = Swaps::next_pool_id();
        assert_eq!(next_pool_before, 0);

        let assets = vec!(ASSET_A, ASSET_B, ASSET_C, ASSET_D);

        assert_ok!(
            Swaps::create_pool(
                assets.clone(),
                0,
                vec!(2 * BASE, 2 * BASE, 2 * BASE, 2 * BASE),
            )
        );

        let next_pool_after = Swaps::next_pool_id();
        assert_eq!(next_pool_after, 1);

        let pool = Swaps::pools(0).unwrap();

        assert_eq!(pool.assets, assets);
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
        let assets = vec!(ASSET_A, ASSET_B, ASSET_C, ASSET_D);

        assert_ok!(
            Swaps::create_pool(
                assets.clone(),
                0,
                vec!(2 * BASE, 2 * BASE, 2 * BASE, 2 * BASE),
            )
        );

        Shares::generate(ASSET_A, &ALICE, 25 * BASE).ok();
        Shares::generate(ASSET_B, &ALICE, 25 * BASE).ok();
        Shares::generate(ASSET_C, &ALICE, 25 * BASE).ok();
        Shares::generate(ASSET_D, &ALICE, 25 * BASE).ok();


        // joining the pool
        assert_ok!(
            Swaps::join_pool(
                Origin::signed(ALICE),
                0,
                5 * BASE,
                vec!(25 * BASE, 25 * BASE, 25 * BASE, 25 * BASE),
            )
        );


    });
}
