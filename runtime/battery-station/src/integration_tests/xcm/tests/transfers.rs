// Copyright 2022-2025 Forecasting Technologies LTD.
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
            adjusted_balance, btc, register_btc, register_foreign_parent, register_foreign_ztg,
            roc, sibling_parachain_account, zeitgeist_parachain_account, ztg, BTC_ID,
            FOREIGN_PARENT_ID, FOREIGN_ZTG_ID, PARA_ID_BATTERY_STATION, PARA_ID_SIBLING,
        },
        test_net::{BatteryStationPara, RococoRelay, SiblingPara},
    },
    xcm_config::fees::default_per_second,
    AssetManager, Balance, Balances, CurrencyId, RuntimeOrigin, Tokens, XTokens,
    ZeitgeistTreasuryAccount,
};

use frame_support::{assert_ok, traits::tokens::fungible::Mutate};
use orml_traits::MultiCurrency;
use rococo_emulated_chain::rococo_runtime;
use xcm::latest::{Junction, Junction::*, Junctions::*, Location, WeightLimit};
use xcm_emulator::{RelayChain, TestExt};
use zeitgeist_primitives::{
    constants::{BalanceFractionalDecimals, BASE},
    types::{Asset, CustomMetadata, XcmMetadata},
};

#[test]
fn transfer_ztg_to_sibling() {
    let mut alice_initial_balance = 0;
    let mut bob_initial_balance = 0;
    let transfer_amount = ztg(5);
    let mut treasury_initial_balance = 0;

    SiblingPara::execute_with(|| {
        treasury_initial_balance =
            AssetManager::free_balance(FOREIGN_ZTG_ID, &ZeitgeistTreasuryAccount::get());
        bob_initial_balance = AssetManager::free_balance(FOREIGN_ZTG_ID, &bob());
        register_foreign_ztg(None);
    });

    BatteryStationPara::execute_with(|| {
        alice_initial_balance = Balances::free_balance(alice());
        assert_eq!(Balances::free_balance(sibling_parachain_account()), 0);
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(alice()),
            CurrencyId::Ztg,
            transfer_amount,
            Box::new(
                Location::new(
                    1,
                    [
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 { network: None, id: bob().into() }
                    ]
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

    SiblingPara::execute_with(|| {
        let current_balance = AssetManager::free_balance(FOREIGN_ZTG_ID, &bob());
        let bob_expected = bob_initial_balance + transfer_amount - ztg_fee();
        let treasury_expected = treasury_initial_balance + ztg_fee();
        assert_eq!(current_balance, bob_expected);
        assert_eq!(
            AssetManager::free_balance(FOREIGN_ZTG_ID, &ZeitgeistTreasuryAccount::get()),
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

    SiblingPara::execute_with(|| {
        treasury_initial_balance =
            AssetManager::free_balance(FOREIGN_ZTG_ID, &ZeitgeistTreasuryAccount::get());
        bob_initial_balance = AssetManager::free_balance(FOREIGN_ZTG_ID, &bob());
        let custom_metadata = CustomMetadata {
            xcm: XcmMetadata { fee_factor: Some(fee_factor) },
            ..Default::default()
        };
        register_foreign_ztg(Some(custom_metadata));
    });

    BatteryStationPara::execute_with(|| {
        let alice_initial_balance = Balances::free_balance(alice());
        assert_eq!(Balances::free_balance(sibling_parachain_account()), 0);
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(alice()),
            Asset::Ztg,
            transfer_amount,
            Box::new(
                Location::new(
                    1,
                    [
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 { network: None, id: bob().into() }
                    ]
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

    SiblingPara::execute_with(|| {
        let current_balance = AssetManager::free_balance(FOREIGN_ZTG_ID, &bob());
        let custom_fee = ztg_fee() * fee_factor / BASE;
        let bob_expected = bob_initial_balance + transfer_amount - custom_fee;
        let treasury_expected = treasury_initial_balance + custom_fee;

        // Verify that bob() now has (amount transferred - fee)
        assert_eq!(current_balance, bob_expected);
        // Verify that fees (of foreign currency) have been put into treasury
        assert_eq!(
            AssetManager::free_balance(FOREIGN_ZTG_ID, &ZeitgeistTreasuryAccount::get()),
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

    BatteryStationPara::execute_with(|| {
        treasury_initial_balance = Balances::free_balance(ZeitgeistTreasuryAccount::get());
        alice_initial_balance = Balances::free_balance(alice());
        assert_eq!(
            Balances::set_balance(&sibling_parachain_account(), sibling_initial_balance),
            sibling_initial_balance
        );
    });

    SiblingPara::execute_with(|| {
        register_foreign_ztg(None);
        let bob_initial_balance = AssetManager::free_balance(FOREIGN_ZTG_ID, &bob());

        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(bob()),
            FOREIGN_ZTG_ID,
            transfer_amount,
            Box::new(
                Location::new(
                    1,
                    [
                        Parachain(PARA_ID_BATTERY_STATION),
                        Junction::AccountId32 { network: None, id: alice().into() }
                    ]
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));
        // Confirm that Bobs's balance is initial balance - amount transferred
        assert_eq!(
            AssetManager::free_balance(FOREIGN_ZTG_ID, &bob()),
            bob_initial_balance - transfer_amount
        );
    });

    BatteryStationPara::execute_with(|| {
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

    BatteryStationPara::execute_with(|| {
        register_btc(None);
        treasury_initial_balance =
            AssetManager::free_balance(BTC_ID, &ZeitgeistTreasuryAccount::get());
        zeitgeist_alice_initial_balance = AssetManager::free_balance(BTC_ID, &alice());
    });

    SiblingPara::execute_with(|| {
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
            CurrencyId::Ztg,
            transfer_amount,
            Box::new(
                Location::new(
                    1,
                    [
                        Parachain(PARA_ID_BATTERY_STATION),
                        Junction::AccountId32 { network: None, id: alice().into() }
                    ]
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

    BatteryStationPara::execute_with(|| {
        let expected = transfer_amount - btc_fee();
        let expected_adjusted = adjusted_balance(btc(1), expected);
        let expected_treasury = treasury_initial_balance + adjusted_balance(btc(1), btc_fee());

        // Verify that remote Alice now has initial balance + amount transferred - fee
        assert_eq!(
            AssetManager::free_balance(BTC_ID, &alice()),
            zeitgeist_alice_initial_balance + expected_adjusted,
        );
        // Verify that fees (of foreign currency) have been put into treasury
        assert_eq!(
            Tokens::free_balance(BTC_ID, &ZeitgeistTreasuryAccount::get()),
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

    SiblingPara::execute_with(|| {
        bob_initial_balance = Balances::free_balance(bob());
        // Set the sovereign balance such that it is not subject to dust collection
        assert_eq!(
            Balances::set_balance(&zeitgeist_parachain_account(), initial_sovereign_balance,),
            initial_sovereign_balance
        );
    });

    BatteryStationPara::execute_with(|| {
        register_btc(None);
        let alice_initial_balance = AssetManager::free_balance(BTC_ID, &alice());

        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(alice()),
            BTC_ID,
            transfer_amount,
            Box::new(
                Location::new(
                    1,
                    [
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 { network: None, id: bob().into() }
                    ]
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));

        let alice_balance = AssetManager::free_balance(BTC_ID, &alice());
        let alice_expected = alice_initial_balance - adjusted_balance(btc(1), transfer_amount);
        assert_eq!(alice_balance, alice_expected);
    });

    SiblingPara::execute_with(|| {
        let expected = bob_initial_balance + transfer_amount - adjusted_balance(btc(1), btc_fee());
        let expected_sovereign = initial_sovereign_balance - transfer_amount;

        // Verify that Bob now has initial balance + amount transferred - fee
        assert_eq!(Balances::free_balance(bob()), expected);
        // Verify that the amount transferred is now subtracted from the zeitgeist account at sibling
        assert_eq!(Balances::free_balance(zeitgeist_parachain_account()), expected_sovereign);
    });
}

#[test]
fn transfer_roc_from_relay_chain() {
    let transfer_amount: Balance = roc(1);
    let mut treasury_initial_balance = 0;
    let mut bob_initial_balance = 0;

    BatteryStationPara::execute_with(|| {
        register_foreign_parent(None);
        treasury_initial_balance =
            AssetManager::free_balance(FOREIGN_PARENT_ID, &ZeitgeistTreasuryAccount::get());
        bob_initial_balance = AssetManager::free_balance(FOREIGN_PARENT_ID, &bob());
    });

    RococoRelay::execute_with(|| {
        let initial_balance = rococo_runtime::Balances::free_balance(alice());
        assert!(initial_balance >= transfer_amount);

        assert_ok!(rococo_runtime::XcmPallet::limited_reserve_transfer_assets(
            rococo_runtime::RuntimeOrigin::signed(alice()),
            Box::new(Parachain(PARA_ID_BATTERY_STATION).into()),
            Box::new(Junction::AccountId32 { network: None, id: bob().into() }.into()),
            Box::new((Here, transfer_amount).into()),
            0,
            WeightLimit::Limited(4_000_000_000.into()),
        ));
    });

    BatteryStationPara::execute_with(|| {
        let expected = transfer_amount - roc_fee();
        let bob_expected = bob_initial_balance + adjusted_balance(roc(1), expected);
        let treasury_expected = treasury_initial_balance + adjusted_balance(roc(1), roc_fee());

        assert_eq!(AssetManager::free_balance(FOREIGN_PARENT_ID, &bob()), bob_expected);
        // Verify that fees (of foreign currency) have been put into treasury
        assert_eq!(
            AssetManager::free_balance(FOREIGN_PARENT_ID, &ZeitgeistTreasuryAccount::get()),
            treasury_expected
        )
    });
}

#[test]
fn transfer_roc_to_relay_chain() {
    let transfer_amount: Balance = roc(1);
    let transfer_amount_local: Balance = adjusted_balance(roc(1), transfer_amount);
    let mut initial_balance_bob = 0;

    RococoRelay::execute_with(|| {
        initial_balance_bob = rococo_runtime::Balances::free_balance(bob());
        let bs_acc =
            RococoRelay::sovereign_account_id_of_child_para(PARA_ID_BATTERY_STATION.into());
        assert_eq!(
            rococo_runtime::Balances::set_balance(&bs_acc, transfer_amount),
            transfer_amount
        );
    });

    BatteryStationPara::execute_with(|| {
        register_foreign_parent(None);
        let initial_balance = AssetManager::free_balance(FOREIGN_PARENT_ID, &alice());
        assert!(initial_balance >= transfer_amount_local);

        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(alice()),
            FOREIGN_PARENT_ID,
            transfer_amount,
            Box::new(
                Location::new(1, [Junction::AccountId32 { id: bob().into(), network: None }])
                    .into()
            ),
            WeightLimit::Limited(4_000_000_000.into())
        ));

        assert_eq!(
            AssetManager::free_balance(FOREIGN_PARENT_ID, &alice()),
            initial_balance - transfer_amount_local
        )
    });

    #[cfg(not(feature = "runtime-benchmarks"))]
    // rococo-runtime does not process messages when runtime-benchmarks is enabled:
    // https://github.com/paritytech/polkadot-sdk/blob/release-polkadot-v1.1.0/polkadot/runtime/rococo/src/lib.rs#L1078-L1080
    RococoRelay::execute_with(|| {
        let expected_fee = 10_651_797;
        let expected_balance_bob = initial_balance_bob + transfer_amount - expected_fee;
        assert_eq!(rococo_runtime::Balances::free_balance(&bob()), expected_balance_bob);
    });
}

#[test]
fn test_total_fee() {
    assert_eq!(ztg_fee(), 93_390_000);
    assert_eq!(btc_fee(), 933_900);
    assert_eq!(roc_fee(), 9_339_000_000);
}

#[inline]
fn ztg_fee() -> Balance {
    fee(BalanceFractionalDecimals::get().into(), 10)
}

#[inline]
fn fee(decimals: u32, multiplier: Balance) -> Balance {
    calc_fee(default_per_second(decimals), multiplier)
}

#[inline]
fn roc_fee() -> Balance {
    fee(12, 10)
}

#[inline]
fn btc_fee() -> Balance {
    fee(8, 10)
}

#[inline]
const fn calc_fee(fee_per_second: Balance, multiplier: Balance) -> Balance {
    // Adjust fee per second to actual test execution time
    fee_per_second / 10_000 * multiplier
}
