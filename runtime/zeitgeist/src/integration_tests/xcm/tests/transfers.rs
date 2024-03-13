// Copyright 2022-2024 Forecasting Technologies LTD.
// Copyright 2021 Centrifuge Foundation (centrifuge.io).
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

use crate::{
    integration_tests::xcm::{
        setup::{
            adjusted_balance, btc, dot, eth, register_btc, register_eth, register_foreign_parent,
            register_foreign_ztg, sibling_parachain_account, zeitgeist_parachain_account, ztg,
            ALICE, BOB, BTC_ID, ETH_ID, FOREIGN_PARENT_ID, FOREIGN_ZTG_ID, PARA_ID_SIBLING,
        },
        test_net::{PolkadotNet, Sibling, TestNet, Zeitgeist},
    },
    xcm_config::{config::zeitgeist, fees::default_per_second},
    AssetManager, AssetRegistry, Balance, Balances, RuntimeOrigin, XTokens,
    ZeitgeistTreasuryAccount,
};

use frame_support::assert_ok;
use orml_traits::MultiCurrency;
use xcm::latest::{Junction, Junction::*, Junctions::*, MultiLocation, WeightLimit};
use xcm_emulator::TestExt;
use zeitgeist_primitives::{
    constants::{BalanceFractionalDecimals, BASE},
    types::{CustomMetadata, XcmAsset, XcmMetadata},
};

#[test]
fn transfer_ztg_to_sibling() {
    TestNet::reset();

    let alice_initial_balance = ztg(10);
    let transfer_amount = ztg(5);
    let mut treasury_initial_balance = 0;

    Sibling::execute_with(|| {
        treasury_initial_balance =
            AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &ZeitgeistTreasuryAccount::get());
        assert_eq!(AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &BOB), 0);
        register_foreign_ztg(None);
    });

    Zeitgeist::execute_with(|| {
        assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance);
        assert_eq!(Balances::free_balance(sibling_parachain_account()), 0);
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE),
            XcmAsset::Ztg,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 { network: None, id: BOB.into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));

        // Confirm that Alice's balance is initial_balance - amount_transferred
        assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance - transfer_amount);

        // Verify that the amount transferred is now part of the sibling account here
        assert_eq!(Balances::free_balance(sibling_parachain_account()), transfer_amount);
    });

    Sibling::execute_with(|| {
        let current_balance = AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &BOB);

        // Verify that BOB now has (amount transferred - fee)
        assert_eq!(current_balance, transfer_amount - ztg_fee());

        // Verify that fees (of foreign currency) have been put into treasury
        assert_eq!(
            AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &ZeitgeistTreasuryAccount::get()),
            treasury_initial_balance + ztg_fee()
        )
    });
}

#[test]
fn transfer_ztg_sibling_to_zeitgeist() {
    TestNet::reset();

    // In order to be able to transfer ZTG from Sibling to Zeitgeist, we need to first send
    // ZTG from Zeitgeist to Sibling, or else it fails since it'd be like Sibling had minted
    // ZTG on their side.
    transfer_ztg_to_sibling();

    let alice_initial_balance = ztg(5);
    let bob_initial_balance = ztg(5) - ztg_fee();
    let mut treasury_initial_balance = 0;
    let sibling_sovereign_initial_balance = ztg(5);
    let transfer_amount = ztg(1);
    // Note: This asset was registered in `transfer_ztg_to_sibling`

    Zeitgeist::execute_with(|| {
        treasury_initial_balance = Balances::free_balance(ZeitgeistTreasuryAccount::get());

        assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance);
        assert_eq!(
            Balances::free_balance(sibling_parachain_account()),
            sibling_sovereign_initial_balance
        );
    });

    Sibling::execute_with(|| {
        assert_eq!(Balances::free_balance(zeitgeist_parachain_account()), 0);
        assert_eq!(AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &BOB), bob_initial_balance);
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(BOB),
            FOREIGN_ZTG_ID,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(zeitgeist::ID),
                        Junction::AccountId32 { network: None, id: ALICE.into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));

        // Confirm that Bobs's balance is initial balance - amount transferred
        assert_eq!(
            AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &BOB),
            bob_initial_balance - transfer_amount
        );
    });

    Zeitgeist::execute_with(|| {
        // Verify that ALICE now has initial balance + amount transferred - fee
        assert_eq!(
            Balances::free_balance(&ALICE),
            alice_initial_balance + transfer_amount - ztg_fee(),
        );

        // Verify that the reserve has been adjusted properly
        assert_eq!(
            Balances::free_balance(sibling_parachain_account()),
            sibling_sovereign_initial_balance - transfer_amount
        );

        // Verify that fees (of native currency) have been put into treasury
        assert_eq!(
            Balances::free_balance(ZeitgeistTreasuryAccount::get()),
            treasury_initial_balance + ztg_fee()
        )
    });
}

#[test]
fn transfer_btc_sibling_to_zeitgeist() {
    TestNet::reset();

    let sibling_alice_initial_balance = ztg(10);
    let zeitgeist_alice_initial_balance = btc(0);
    let initial_sovereign_balance = btc(100);
    let transfer_amount = btc(100);
    let mut treasury_initial_balance = 0;

    Zeitgeist::execute_with(|| {
        register_btc(None);
        treasury_initial_balance =
            AssetManager::free_balance(BTC_ID.into(), &ZeitgeistTreasuryAccount::get());
        assert_eq!(
            AssetManager::free_balance(BTC_ID.into(), &ALICE),
            zeitgeist_alice_initial_balance,
        );
    });

    Sibling::execute_with(|| {
        assert_eq!(Balances::free_balance(&ALICE), sibling_alice_initial_balance);
        // Set the sovereign balance such that it is not subject to dust collection
        assert_ok!(Balances::set_balance(
            RuntimeOrigin::root(),
            zeitgeist_parachain_account().into(),
            initial_sovereign_balance,
            0
        ));
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE),
            // Target chain will interpret XcmAsset::Ztg as BTC in this context.
            XcmAsset::Ztg,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(zeitgeist::ID),
                        Junction::AccountId32 { network: None, id: ALICE.into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));

        // Confirm that Alice's balance is initial_balance - amount_transferred
        assert_eq!(Balances::free_balance(&ALICE), sibling_alice_initial_balance - transfer_amount);

        // Verify that the amount transferred is now part of the zeitgeist account here
        assert_eq!(
            Balances::free_balance(zeitgeist_parachain_account()),
            initial_sovereign_balance + transfer_amount
        );
    });

    Zeitgeist::execute_with(|| {
        let expected = transfer_amount - btc_fee();
        let expected_adjusted = adjusted_balance(btc(1), expected);

        // Verify that remote Alice now has initial balance + amount transferred - fee
        assert_eq!(
            AssetManager::free_balance(BTC_ID.into(), &ALICE),
            zeitgeist_alice_initial_balance + expected_adjusted,
        );

        // Verify that fees (of foreign currency) have been put into treasury
        assert_eq!(
            AssetManager::free_balance(BTC_ID.into(), &ZeitgeistTreasuryAccount::get()),
            // Align decimal fractional places
            treasury_initial_balance + adjusted_balance(btc(1), btc_fee())
        )
    });
}

#[test]
fn transfer_btc_zeitgeist_to_sibling() {
    TestNet::reset();

    let transfer_amount = btc(100) - btc_fee();
    let initial_sovereign_balance = 2 * btc(100);
    let sibling_bob_initial_balance = btc(0);

    transfer_btc_sibling_to_zeitgeist();

    Sibling::execute_with(|| {
        assert_eq!(AssetManager::free_balance(BTC_ID.into(), &BOB), sibling_bob_initial_balance,);
    });

    Zeitgeist::execute_with(|| {
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE),
            BTC_ID,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 { network: None, id: BOB.into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));

        // Confirm that Alice's balance is initial_balance - amount_transferred
        assert_eq!(AssetManager::free_balance(BTC_ID.into(), &ALICE), 0);
    });

    Sibling::execute_with(|| {
        let fee_adjusted = adjusted_balance(btc(1), btc_fee());
        let expected = transfer_amount - fee_adjusted;

        // Verify that Bob now has initial balance + amount transferred - fee
        assert_eq!(Balances::free_balance(&BOB), sibling_bob_initial_balance + expected,);

        // Verify that the amount transferred is now subtracted from the zeitgeist account at sibling
        assert_eq!(
            Balances::free_balance(zeitgeist_parachain_account()),
            initial_sovereign_balance - transfer_amount
        );
    });
}

#[test]
fn transfer_eth_sibling_to_zeitgeist() {
    TestNet::reset();

    let sibling_alice_initial_balance = ztg(10) + eth(1);
    let zeitgeist_alice_initial_balance = eth(0);
    let initial_sovereign_balance = eth(1);
    let transfer_amount = eth(1);
    let mut treasury_initial_balance = 0;

    Zeitgeist::execute_with(|| {
        register_eth(None);
        treasury_initial_balance =
            AssetManager::free_balance(ETH_ID.into(), &ZeitgeistTreasuryAccount::get());
        assert_eq!(
            AssetManager::free_balance(ETH_ID.into(), &ALICE),
            zeitgeist_alice_initial_balance,
        );
    });

    Sibling::execute_with(|| {
        // Set the sovereign balance such that it is not subject to dust collection
        assert_ok!(Balances::set_balance(
            RuntimeOrigin::root(),
            zeitgeist_parachain_account().into(),
            initial_sovereign_balance,
            0
        ));
        // Add 1 "fake" ETH to Alice's balance
        assert_ok!(Balances::set_balance(
            RuntimeOrigin::root(),
            ALICE.into(),
            sibling_alice_initial_balance,
            0
        ));
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE),
            // Target chain will interpret XcmAsset::Ztg as ETH in this context.
            XcmAsset::Ztg,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(zeitgeist::ID),
                        Junction::AccountId32 { network: None, id: ALICE.into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));

        // Confirm that Alice's balance is initial_balance - amount_transferred
        assert_eq!(Balances::free_balance(&ALICE), sibling_alice_initial_balance - transfer_amount);

        // Verify that the amount transferred is now part of the zeitgeist account here
        assert_eq!(
            Balances::free_balance(zeitgeist_parachain_account()),
            initial_sovereign_balance + transfer_amount
        );
    });

    Zeitgeist::execute_with(|| {
        let expected = transfer_amount - eth_fee();
        let expected_adjusted = adjusted_balance(eth(1), expected);

        // Verify that remote Alice now has initial balance + amount transferred - fee
        assert_eq!(
            AssetManager::free_balance(ETH_ID.into(), &ALICE),
            zeitgeist_alice_initial_balance + expected_adjusted,
        );

        // Verify that fees (of foreign currency) have been put into treasury
        assert_eq!(
            AssetManager::free_balance(ETH_ID.into(), &ZeitgeistTreasuryAccount::get()),
            // Align decimal fractional places
            treasury_initial_balance + adjusted_balance(eth(1), eth_fee())
        )
    });
}

#[test]
fn transfer_eth_zeitgeist_to_sibling() {
    TestNet::reset();

    let transfer_amount = eth(1) - eth_fee();
    let initial_sovereign_balance = 2 * eth(1);
    let sibling_bob_initial_balance = eth(0);

    transfer_eth_sibling_to_zeitgeist();

    Sibling::execute_with(|| {
        assert_eq!(AssetManager::free_balance(ETH_ID.into(), &BOB), sibling_bob_initial_balance,);
    });

    Zeitgeist::execute_with(|| {
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE),
            ETH_ID,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 { network: None, id: BOB.into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));

        // Confirm that Alice's balance is initial_balance - amount_transferred
        assert_eq!(AssetManager::free_balance(ETH_ID.into(), &ALICE), 0);
    });

    Sibling::execute_with(|| {
        let fee_adjusted = adjusted_balance(eth(1), eth_fee());
        let expected = transfer_amount - fee_adjusted;

        // Verify that Bob now has initial balance + amount transferred - fee
        assert_eq!(Balances::free_balance(&BOB), sibling_bob_initial_balance + expected,);

        // Verify that the amount transferred is now subtracted from the zeitgeist account at sibling
        assert_eq!(
            Balances::free_balance(zeitgeist_parachain_account()),
            initial_sovereign_balance - transfer_amount
        );
    });
}

#[test]
fn transfer_dot_from_relay_chain() {
    TestNet::reset();

    let transfer_amount: Balance = dot(2);

    Zeitgeist::execute_with(|| {
        register_foreign_parent(None);
    });

    PolkadotNet::execute_with(|| {
        let initial_balance = polkadot_runtime::Balances::free_balance(&ALICE);
        assert!(initial_balance >= transfer_amount);

        assert_ok!(polkadot_runtime::XcmPallet::reserve_transfer_assets(
            polkadot_runtime::RuntimeOrigin::signed(ALICE),
            Box::new(Parachain(zeitgeist::ID).into()),
            Box::new(Junction::AccountId32 { network: None, id: BOB.into() }.into()),
            Box::new((Here, transfer_amount).into()),
            0
        ));
    });

    Zeitgeist::execute_with(|| {
        assert_eq!(
            AssetManager::free_balance(FOREIGN_PARENT_ID.into(), &BOB),
            transfer_amount - dot_fee()
        );
    });
}

#[test]
fn transfer_dot_to_relay_chain() {
    TestNet::reset();

    let transfer_amount: Balance = dot(2);
    transfer_dot_from_relay_chain();

    Zeitgeist::execute_with(|| {
        let initial_balance = AssetManager::free_balance(FOREIGN_PARENT_ID.into(), &ALICE);
        assert!(initial_balance >= transfer_amount);

        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE),
            FOREIGN_PARENT_ID,
            transfer_amount,
            Box::new(
                MultiLocation::new(1, X1(Junction::AccountId32 { id: BOB.into(), network: None }))
                    .into()
            ),
            WeightLimit::Unlimited,
        ));

        assert_eq!(
            AssetManager::free_balance(FOREIGN_PARENT_ID.into(), &ALICE),
            initial_balance - transfer_amount
        )
    });

    PolkadotNet::execute_with(|| {
        assert_eq!(polkadot_runtime::Balances::free_balance(&BOB), 19_637_471_000);
    });
}

#[test]
fn transfer_ztg_to_sibling_with_custom_fee() {
    TestNet::reset();

    let alice_initial_balance = ztg(10);
    // 10x fee factor, so ZTG has 10x the worth of sibling currency.
    let fee_factor = 100_000_000_000;
    let transfer_amount = ztg(5);
    let mut treasury_initial_balance = 0;

    Sibling::execute_with(|| {
        treasury_initial_balance =
            AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &ZeitgeistTreasuryAccount::get());
        assert_eq!(AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &BOB), 0);

        register_foreign_ztg(None);
        let custom_metadata = CustomMetadata {
            xcm: XcmMetadata { fee_factor: Some(fee_factor) },
            ..Default::default()
        };
        assert_ok!(AssetRegistry::do_update_asset(
            FOREIGN_ZTG_ID,
            None,
            None,
            None,
            None,
            None,
            Some(custom_metadata)
        ));
    });

    Zeitgeist::execute_with(|| {
        assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance);
        assert_eq!(Balances::free_balance(sibling_parachain_account()), 0);
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE),
            XcmAsset::Ztg,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 { network: None, id: BOB.into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));

        // Confirm that Alice's balance is initial_balance - amount_transferred
        assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance - transfer_amount);

        // Verify that the amount transferred is now part of the sibling account here
        assert_eq!(Balances::free_balance(sibling_parachain_account()), transfer_amount);
    });

    Sibling::execute_with(|| {
        let current_balance = AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &BOB);
        let custom_fee = calc_fee(default_per_second(10) * 10);

        // Verify that BOB now has (amount transferred - fee)
        assert_eq!(current_balance, transfer_amount - custom_fee);

        // Sanity check for the actual amount BOB ends up with
        assert_eq!(current_balance, transfer_amount - ztg_fee() * fee_factor / BASE);

        // Verify that fees (of foreign currency) have been put into treasury
        assert_eq!(
            AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &ZeitgeistTreasuryAccount::get()),
            treasury_initial_balance + custom_fee
        )
    });
}

#[test]
fn test_total_fee() {
    assert_eq!(ztg_fee(), 80_128_000);
    assert_eq!(dot_fee(), ztg_fee());
}

#[inline]
fn ztg_fee() -> Balance {
    fee(BalanceFractionalDecimals::get().into())
}

#[inline]
fn fee(decimals: u32) -> Balance {
    calc_fee(default_per_second(decimals))
}

// The fee associated with transferring dot tokens
#[inline]
fn dot_fee() -> Balance {
    fee(10)
}

#[inline]
fn btc_fee() -> Balance {
    fee(8)
}

#[inline]
fn eth_fee() -> Balance {
    fee(18)
}

#[inline]
const fn calc_fee(fee_per_second: Balance) -> Balance {
    // We divide the fee to align its unit and multiply by 8 as that seems to be the unit of
    // time the tests take.
    // NOTE: it is possible that in different machines this value may differ. We shall see.
    fee_per_second / 10_000 * 8
}
