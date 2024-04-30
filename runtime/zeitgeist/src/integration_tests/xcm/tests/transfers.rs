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
            accounts::{alice, bob},
            adjusted_balance, btc, dot, eth, register_btc, register_eth, register_foreign_parent,
            register_foreign_ztg, sibling_parachain_account, zeitgeist_parachain_account, ztg,
            BTC_ID, ETH_ID, FOREIGN_PARENT_ID, FOREIGN_ZTG_ID, PARA_ID_SIBLING, PARA_ID_ZEITGEIST,
        },
        test_net::{Polkadot, Sibling, Zeitgeist},
    },
    xcm_config::fees::default_per_second,
    AssetManager, Balance, Balances, RuntimeOrigin, XTokens, ZeitgeistTreasuryAccount,
};

use frame_support::{assert_ok, traits::tokens::fungible::Mutate};
use orml_traits::MultiCurrency;
use xcm::latest::{Junction, Junction::*, Junctions::*, MultiLocation, WeightLimit};
use xcm_emulator::{RelayChain, TestExt};
use zeitgeist_primitives::{
    constants::{BalanceFractionalDecimals, BASE},
    types::{CustomMetadata, XcmAsset, XcmMetadata},
};

#[test]
fn transfer_ztg_to_sibling() {
    let mut alice_initial_balance = 0;
    let mut bob_initial_balance = 0;
    let transfer_amount = ztg(5);
    let mut treasury_initial_balance = 0;

    Sibling::execute_with(|| {
        treasury_initial_balance =
            AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &ZeitgeistTreasuryAccount::get());
        bob_initial_balance = AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &bob());
        register_foreign_ztg(None);
    });

    Zeitgeist::execute_with(|| {
        alice_initial_balance = Balances::free_balance(alice());
        assert_eq!(Balances::free_balance(sibling_parachain_account()), 0);
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(alice()),
            XcmAsset::Ztg,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 { network: None, id: bob().into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));

        // Confirm that Alice's balance is initial_balance - amount_transferred
        assert_eq!(Balances::free_balance(alice()), alice_initial_balance - transfer_amount);
        // Verify that the amount transferred is now part of the sibling account here
        assert_eq!(Balances::free_balance(sibling_parachain_account()), transfer_amount);
    });

    Sibling::execute_with(|| {
        let current_balance = AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &bob());
        let bob_expected = bob_initial_balance + transfer_amount - ztg_fee();
        let treasury_expected = treasury_initial_balance + ztg_fee();

        // Verify that bob() now has (amount transferred - fee)
        assert_eq!(current_balance, bob_expected);
        // Verify that fees (of foreign currency) have been put into treasury
        assert_eq!(
            AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &ZeitgeistTreasuryAccount::get()),
            treasury_expected
        )
    });
}

#[test]
fn transfer_ztg_to_sibling_with_custom_fee() {
    // 10x fee factor, so ZTG has 10x the worth of sibling currency.
    let fee_factor = 100_000_000_000;
    let transfer_amount = ztg(5);
    let mut treasury_initial_balance = 0;
    let mut bob_initial_balance = 0;

    Sibling::execute_with(|| {
        treasury_initial_balance =
            AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &ZeitgeistTreasuryAccount::get());
        bob_initial_balance = AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &bob());
        let custom_metadata = CustomMetadata {
            xcm: XcmMetadata { fee_factor: Some(fee_factor) },
            ..Default::default()
        };
        register_foreign_ztg(Some(custom_metadata));
    });

    Zeitgeist::execute_with(|| {
        let alice_initial_balance = Balances::free_balance(alice());
        assert_eq!(Balances::free_balance(sibling_parachain_account()), 0);
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(alice()),
            XcmAsset::Ztg,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 { network: None, id: bob().into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));
        // Confirm that Alice's balance is initial_balance - amount_transferred
        assert_eq!(Balances::free_balance(alice()), alice_initial_balance - transfer_amount);
        // Verify that the amount transferred is now part of the sibling account here
        assert_eq!(Balances::free_balance(sibling_parachain_account()), transfer_amount);
    });

    Sibling::execute_with(|| {
        let current_balance = AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &bob());
        let custom_fee = ztg_fee() * fee_factor / BASE;
        let bob_expected = bob_initial_balance + transfer_amount - custom_fee;
        let treasury_expected = treasury_initial_balance + custom_fee;

        // Verify that bob() now has (amount transferred - fee)
        assert_eq!(current_balance, bob_expected);
        // Verify that fees (of foreign currency) have been put into treasury
        assert_eq!(
            AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &ZeitgeistTreasuryAccount::get()),
            treasury_expected
        )
    });
}

#[test]
fn transfer_ztg_sibling_to_zeitgeist() {
    let mut alice_initial_balance = 0;
    let mut treasury_initial_balance = 0;
    let transfer_amount = ztg(1);
    let sibling_initial_balance = transfer_amount;

    Zeitgeist::execute_with(|| {
        treasury_initial_balance = Balances::free_balance(ZeitgeistTreasuryAccount::get());
        alice_initial_balance = Balances::free_balance(alice());
        assert_eq!(
            Balances::set_balance(&sibling_parachain_account(), sibling_initial_balance),
            sibling_initial_balance
        );
    });

    Sibling::execute_with(|| {
        register_foreign_ztg(None);
        let bob_initial_balance = AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &bob());

        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(bob()),
            FOREIGN_ZTG_ID,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_ZEITGEIST),
                        Junction::AccountId32 { network: None, id: alice().into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));
        // Confirm that Bobs's balance is initial balance - amount transferred
        assert_eq!(
            AssetManager::free_balance(FOREIGN_ZTG_ID.into(), &bob()),
            bob_initial_balance - transfer_amount
        );
    });

    Zeitgeist::execute_with(|| {
        // Verify that alice() now has initial balance + amount transferred - fee
        assert_eq!(
            Balances::free_balance(alice()),
            alice_initial_balance + transfer_amount - ztg_fee(),
        );
        // Verify that the reserve has been adjusted properly
        assert_eq!(
            Balances::free_balance(sibling_parachain_account()),
            sibling_initial_balance - transfer_amount
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
    let mut zeitgeist_alice_initial_balance = 0;
    let transfer_amount = btc(100);
    let mut treasury_initial_balance = 0;

    Zeitgeist::execute_with(|| {
        register_btc(None);
        treasury_initial_balance =
            AssetManager::free_balance(BTC_ID.into(), &ZeitgeistTreasuryAccount::get());
        zeitgeist_alice_initial_balance = AssetManager::free_balance(BTC_ID.into(), &alice());
    });

    Sibling::execute_with(|| {
        let alice_initial_balance = Balances::free_balance(alice());
        let initial_sovereign_balance = transfer_amount;

        // Set the sovereign balance such that it is not subject to dust collection
        assert_eq!(
            Balances::set_balance(&zeitgeist_parachain_account(), initial_sovereign_balance,),
            initial_sovereign_balance
        );
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(alice()),
            // Target chain will interpret XcmAsset::Ztg as BTC in this context.
            XcmAsset::Ztg,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_ZEITGEIST),
                        Junction::AccountId32 { network: None, id: alice().into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));
        // Confirm that Alice's balance is initial_balance - amount_transferred
        assert_eq!(Balances::free_balance(alice()), alice_initial_balance - transfer_amount);
        // Verify that the amount transferred is now part of the zeitgeist account here
        assert_eq!(
            Balances::free_balance(zeitgeist_parachain_account()),
            initial_sovereign_balance + transfer_amount
        );
    });

    Zeitgeist::execute_with(|| {
        let expected = transfer_amount - btc_fee();
        let expected_adjusted = adjusted_balance(btc(1), expected);
        let expected_treasury = treasury_initial_balance + adjusted_balance(btc(1), btc_fee());

        // Verify that remote Alice now has initial balance + amount transferred - fee
        assert_eq!(
            AssetManager::free_balance(BTC_ID.into(), &alice()),
            zeitgeist_alice_initial_balance + expected_adjusted,
        );
        // Verify that fees (of foreign currency) have been put into treasury
        assert_eq!(
            AssetManager::free_balance(BTC_ID.into(), &ZeitgeistTreasuryAccount::get()),
            // Align decimal fractional places
            expected_treasury
        )
    });
}

#[test]
fn transfer_btc_zeitgeist_to_sibling() {
    let transfer_amount = btc(100);
    let initial_sovereign_balance = transfer_amount;
    let mut bob_initial_balance = 0;

    Sibling::execute_with(|| {
        bob_initial_balance = Balances::free_balance(bob());
        // Set the sovereign balance such that it is not subject to dust collection
        assert_eq!(
            Balances::set_balance(&zeitgeist_parachain_account(), initial_sovereign_balance,),
            initial_sovereign_balance
        );
    });

    Zeitgeist::execute_with(|| {
        register_btc(None);
        let alice_initial_balance = AssetManager::free_balance(BTC_ID.into(), &alice());

        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(alice()),
            BTC_ID,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 { network: None, id: bob().into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));

        // Confirm that Alice's balance is initial_balance - amount_transferred
        let alice_balance = AssetManager::free_balance(BTC_ID.into(), &alice());
        let alice_expected = alice_initial_balance - adjusted_balance(btc(1), transfer_amount);
        assert_eq!(alice_balance, alice_expected);
    });

    Sibling::execute_with(|| {
        let expected = bob_initial_balance + transfer_amount - adjusted_balance(btc(1), btc_fee());
        let expected_sovereign = initial_sovereign_balance - transfer_amount;

        // Verify that Bob now has initial balance + amount transferred - fee
        assert_eq!(Balances::free_balance(bob()), expected);
        // Verify that the amount transferred is now subtracted from the zeitgeist account at sibling
        assert_eq!(Balances::free_balance(zeitgeist_parachain_account()), expected_sovereign);
    });
}

#[test]
fn transfer_eth_sibling_to_zeitgeist() {
    let mut zeitgeist_alice_initial_balance = 0;
    let transfer_amount = eth(100);
    let mut treasury_initial_balance = 0;

    Zeitgeist::execute_with(|| {
        register_eth(None);
        treasury_initial_balance =
            AssetManager::free_balance(ETH_ID.into(), &ZeitgeistTreasuryAccount::get());
        zeitgeist_alice_initial_balance = AssetManager::free_balance(ETH_ID.into(), &alice());
    });

    Sibling::execute_with(|| {
        let alice_initial_balance = Balances::free_balance(alice());
        let initial_sovereign_balance = transfer_amount;

        // Set the sovereign balance such that it is not subject to dust collection
        assert_eq!(
            Balances::set_balance(&zeitgeist_parachain_account(), initial_sovereign_balance,),
            initial_sovereign_balance
        );
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(alice()),
            // Target chain will interpret XcmAsset::Ztg as ETH in this context.
            XcmAsset::Ztg,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_ZEITGEIST),
                        Junction::AccountId32 { network: None, id: alice().into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));
        // Confirm that Alice's balance is initial_balance - amount_transferred
        assert_eq!(Balances::free_balance(alice()), alice_initial_balance - transfer_amount);
        // Verify that the amount transferred is now part of the zeitgeist account here
        assert_eq!(
            Balances::free_balance(zeitgeist_parachain_account()),
            initial_sovereign_balance + transfer_amount
        );
    });

    Zeitgeist::execute_with(|| {
        let expected = transfer_amount - eth_fee();
        let expected_adjusted = adjusted_balance(eth(1), expected);
        let expected_treasury = treasury_initial_balance + adjusted_balance(eth(1), eth_fee());

        // Verify that remote Alice now has initial balance + amount transferred - fee
        assert_eq!(
            AssetManager::free_balance(ETH_ID.into(), &alice()),
            zeitgeist_alice_initial_balance + expected_adjusted,
        );
        // Verify that fees (of foreign currency) have been put into treasury
        assert_eq!(
            AssetManager::free_balance(ETH_ID.into(), &ZeitgeistTreasuryAccount::get()),
            // Align decimal fractional places
            expected_treasury
        )
    });
}

#[test]
fn transfer_eth_zeitgeist_to_sibling() {
    let transfer_amount = eth(100);
    let initial_sovereign_balance = transfer_amount;
    let mut bob_initial_balance = 0;

    Sibling::execute_with(|| {
        bob_initial_balance = Balances::free_balance(bob());
        // Set the sovereign balance such that it is not subject to dust collection
        assert_eq!(
            Balances::set_balance(&zeitgeist_parachain_account(), initial_sovereign_balance,),
            initial_sovereign_balance
        );
    });

    Zeitgeist::execute_with(|| {
        register_eth(None);
        let alice_initial_balance = AssetManager::free_balance(ETH_ID.into(), &alice());

        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(alice()),
            ETH_ID,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 { network: None, id: bob().into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));

        // Confirm that Alice's balance is initial_balance - amount_transferred
        let alice_balance = AssetManager::free_balance(ETH_ID.into(), &alice());
        let alice_expected = alice_initial_balance - adjusted_balance(eth(1), transfer_amount);
        assert_eq!(alice_balance, alice_expected);
    });

    Sibling::execute_with(|| {
        let expected = bob_initial_balance + transfer_amount - adjusted_balance(eth(1), eth_fee());
        let expected_sovereign = initial_sovereign_balance - transfer_amount;

        // Verify that Bob now has initial balance + amount transferred - fee
        assert_eq!(Balances::free_balance(bob()), expected);
        // Verify that the amount transferred is now subtracted from the zeitgeist account at sibling
        assert_eq!(Balances::free_balance(zeitgeist_parachain_account()), expected_sovereign);
    });
}

#[test]
fn transfer_dot_from_relay_chain() {
    let transfer_amount: Balance = dot(1);
    let mut treasury_initial_balance = 0;
    let mut bob_initial_balance = 0;

    Zeitgeist::execute_with(|| {
        register_foreign_parent(None);
        treasury_initial_balance =
            AssetManager::free_balance(FOREIGN_PARENT_ID.into(), &ZeitgeistTreasuryAccount::get());
        bob_initial_balance = AssetManager::free_balance(FOREIGN_PARENT_ID.into(), &bob());
    });

    Polkadot::execute_with(|| {
        let initial_balance = polkadot_runtime::Balances::free_balance(alice());
        assert!(initial_balance >= transfer_amount);

        assert_ok!(polkadot_runtime::XcmPallet::reserve_transfer_assets(
            polkadot_runtime::RuntimeOrigin::signed(alice()),
            Box::new(Parachain(PARA_ID_ZEITGEIST).into()),
            Box::new(Junction::AccountId32 { network: None, id: bob().into() }.into()),
            Box::new((Here, transfer_amount).into()),
            0
        ));
    });

    Zeitgeist::execute_with(|| {
        let expected = transfer_amount - dot_fee();
        let bob_expected = bob_initial_balance + adjusted_balance(dot(1), expected);
        let treasury_expected = treasury_initial_balance + adjusted_balance(dot(1), dot_fee());

        assert_eq!(AssetManager::free_balance(FOREIGN_PARENT_ID.into(), &bob()), bob_expected);
        // Verify that fees (of foreign currency) have been put into treasury
        assert_eq!(
            AssetManager::free_balance(FOREIGN_PARENT_ID.into(), &ZeitgeistTreasuryAccount::get()),
            treasury_expected
        )
    });
}

#[test]
fn transfer_dot_to_relay_chain() {
    let transfer_amount: Balance = dot(1);
    let transfer_amount_local: Balance = adjusted_balance(dot(1), transfer_amount);
    let mut initial_balance_bob = 0;

    Polkadot::execute_with(|| {
        initial_balance_bob = polkadot_runtime::Balances::free_balance(bob());
        let bs_acc = Polkadot::sovereign_account_id_of_child_para(PARA_ID_ZEITGEIST.into());
        assert_eq!(
            polkadot_runtime::Balances::set_balance(&bs_acc, transfer_amount),
            transfer_amount
        );
    });

    Zeitgeist::execute_with(|| {
        register_foreign_parent(None);
        let initial_balance = AssetManager::free_balance(FOREIGN_PARENT_ID.into(), &alice());
        assert!(initial_balance >= transfer_amount_local);

        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(alice()),
            FOREIGN_PARENT_ID,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X1(Junction::AccountId32 { id: bob().into(), network: None })
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into())
        ));

        assert_eq!(
            AssetManager::free_balance(FOREIGN_PARENT_ID.into(), &alice()),
            initial_balance - transfer_amount_local
        )
    });

    #[cfg(not(feature = "runtime-benchmarks"))]
    // polkadot-runtime does not process messages when runtime-benchmarks is enabled:
    // https://github.com/paritytech/polkadot-sdk/blob/release-polkadot-v1.1.0/polkadot/runtime/polkadot/src/lib.rs#L1138-L1140
    Polkadot::execute_with(|| {
        let expected_fee = 21_062_795;
        let expected_balance_bob = initial_balance_bob + transfer_amount - expected_fee;
        assert_eq!(polkadot_runtime::Balances::free_balance(&bob()), expected_balance_bob);
    });
}

#[test]
fn test_total_fee() {
    assert_eq!(btc_fee(), 642_960);
    assert_eq!(dot_fee(), 80_370_000);
    assert_eq!(ztg_fee(), 64_296_000);
    assert_eq!(eth_fee(), 6_429_600_000_000_000);
}

#[inline]
fn ztg_fee() -> Balance {
    fee(BalanceFractionalDecimals::get().into(), 8)
}

#[inline]
fn fee(decimals: u32, multiplier: Balance) -> Balance {
    calc_fee(default_per_second(decimals), multiplier)
}

#[inline]
fn dot_fee() -> Balance {
    fee(10, 10)
}

#[inline]
fn btc_fee() -> Balance {
    fee(8, 8)
}

#[inline]
fn eth_fee() -> Balance {
    fee(18, 8)
}

#[inline]
const fn calc_fee(fee_per_second: Balance, multiplier: Balance) -> Balance {
    // Adjust fee per second to actual test execution time
    fee_per_second / 10_000 * multiplier
}
