use crate::{
    mock::{Balances, ExtBuilder, Origin, Runtime, ZeitgeistLiquidityMining, ALICE},
    BlockBoughtShares, OwnedBalances,
};
use frame_support::{
    assert_err, assert_ok,
    dispatch::DispatchError,
    traits::{Currency, OnFinalize},
};
use frame_system::RawOrigin;

#[test]
fn blocks_shares_are_erased_after_each_block() {
    ExtBuilder::default().build().execute_with(|| {
        <BlockBoughtShares<Runtime>>::insert(ALICE, 0, 1);
        <BlockBoughtShares<Runtime>>::insert(ALICE, 1, 1);
        assert_eq!(<BlockBoughtShares<Runtime>>::iter().count(), 2);
        ZeitgeistLiquidityMining::on_finalize(1);
        assert_eq!(<BlockBoughtShares<Runtime>>::iter().count(), 0);
    });
}

#[test]
fn owned_balances_is_updated_after_each_block() {
    ExtBuilder::default().build().execute_with(|| {
        let first_market_shares = 15;
        let second_market_shares = 25;
        <BlockBoughtShares<Runtime>>::insert(ALICE, 0, first_market_shares);
        <BlockBoughtShares<Runtime>>::insert(ALICE, 1, second_market_shares);
        ZeitgeistLiquidityMining::on_finalize(1);
        let total_shares = first_market_shares + second_market_shares;
        let one_share_value = ExtBuilder::default().per_block_distribution / total_shares;
        let mut vec = <OwnedBalances<Runtime>>::iter().collect::<Vec<_>>();
        vec.sort_unstable_by(|(.., a), (.., b)| a.cmp(b));
        assert_eq!(vec[0].2, one_share_value * first_market_shares);
        assert_eq!(vec[1].2, one_share_value * second_market_shares);
    });
}

#[test]
fn genesis_has_lm_account_and_initial_per_block_distribution() {
    ExtBuilder::default().build().execute_with(|| {
        let pallet_account_id = crate::Pallet::<Runtime>::pallet_account_id();
        assert_eq!(
            Balances::total_balance(&pallet_account_id),
            ExtBuilder::default().initial_balance
        );
        assert_eq!(
            <crate::PerBlockDistribution::<Runtime>>::get(),
            ExtBuilder::default().per_block_distribution
        );
    });
}

#[test]
fn only_sudo_can_change_per_block_distribution() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(ZeitgeistLiquidityMining::set_per_block_distribution(
            RawOrigin::Root.into(),
            100
        ));
        assert_err!(
            ZeitgeistLiquidityMining::set_per_block_distribution(Origin::signed(ALICE), 100),
            DispatchError::BadOrigin
        );
    });
}
