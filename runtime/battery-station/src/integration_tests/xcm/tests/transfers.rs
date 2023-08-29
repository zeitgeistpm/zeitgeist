// Copyright 2022-2023 Forecasting Technologies LTD.
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
            adjusted_balance, btc, register_btc, register_foreign_parent, register_foreign_ztg,
            roc, sibling_parachain_account, zeitgeist_parachain_account, ztg, ALICE, BOB, BTC_ID,
            FOREIGN_PARENT_ID, FOREIGN_ZTG_ID, PARA_ID_SIBLING,
        },
        test_net::{RococoNet, Sibling, TestNet, Zeitgeist},
    },
    xcm_config::{config::battery_station, fees::default_per_second},
    AssetRegistry, Balance, Balances, CurrencyId, RuntimeOrigin, Tokens, XTokens,
    ZeitgeistTreasuryAccount,
};

use frame_support::assert_ok;
use orml_traits::MultiCurrency;
use xcm::latest::{Junction, Junction::*, Junctions::*, MultiLocation, WeightLimit};
use xcm_emulator::TestExt;
use zeitgeist_primitives::{
    constants::{BalanceFractionalDecimals, BASE},
    types::{CustomMetadata, XcmMetadata},
};

#[test]
fn transfer_ztg_to_sibling() {
    TestNet::reset();

    let alice_initial_balance = ztg(10);
    let transfer_amount = ztg(5);
    let mut treasury_initial_balance = 0;

    Sibling::execute_with(|| {
        treasury_initial_balance =
            Tokens::free_balance(FOREIGN_ZTG_ID, &ZeitgeistTreasuryAccount::get());
        assert_eq!(Tokens::free_balance(FOREIGN_ZTG_ID, &BOB), 0);
        register_foreign_ztg(None);
    });

    Zeitgeist::execute_with(|| {
        assert_eq!(Balances::free_balance(&ALICE), alice_initial_balance);
        assert_eq!(Balances::free_balance(sibling_parachain_account()), 0);
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE),
            CurrencyId::Ztg,
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
        let current_balance = Tokens::free_balance(FOREIGN_ZTG_ID, &BOB);

        // Verify that BOB now has (amount transferred - fee)
        assert_eq!(current_balance, transfer_amount - ztg_fee());

        // Verify that fees (of foreign currency) have been put into treasury
        assert_eq!(
            Tokens::free_balance(FOREIGN_ZTG_ID, &ZeitgeistTreasuryAccount::get()),
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
        assert_eq!(Tokens::free_balance(FOREIGN_ZTG_ID, &BOB), bob_initial_balance);
        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(BOB),
            FOREIGN_ZTG_ID,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(battery_station::ID),
                        Junction::AccountId32 { network: None, id: ALICE.into() }
                    )
                )
                .into()
            ),
            WeightLimit::Limited(4_000_000_000.into()),
        ));

        // Confirm that Bobs's balance is initial balance - amount transferred
        assert_eq!(
            Tokens::free_balance(FOREIGN_ZTG_ID, &BOB),
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

    Zeitgeist::execute_with(|| {
        register_btc(None);

        assert_eq!(Tokens::free_balance(BTC_ID, &ALICE), zeitgeist_alice_initial_balance,);
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
            // Target chain will interpret CurrencyId::Ztg as BTC in this context.
            CurrencyId::Ztg,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(battery_station::ID),
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
            Tokens::free_balance(BTC_ID, &ALICE),
            zeitgeist_alice_initial_balance + expected_adjusted,
        );
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
        assert_eq!(Tokens::free_balance(BTC_ID, &BOB), sibling_bob_initial_balance,);
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
        assert_eq!(Tokens::free_balance(BTC_ID, &ALICE), 0);
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
fn transfer_roc_from_relay_chain() {
    TestNet::reset();

    let transfer_amount: Balance = roc(1);

    Zeitgeist::execute_with(|| {
        register_foreign_parent(None);
    });

    RococoNet::execute_with(|| {
        let initial_balance = rococo_runtime::Balances::free_balance(&ALICE);
        assert!(initial_balance >= transfer_amount);

        assert_ok!(rococo_runtime::XcmPallet::reserve_transfer_assets(
            rococo_runtime::RuntimeOrigin::signed(ALICE),
            Box::new(Parachain(battery_station::ID).into()),
            Box::new(Junction::AccountId32 { network: None, id: BOB.into() }.into()),
            Box::new((Here, transfer_amount).into()),
            0
        ));
    });

    Zeitgeist::execute_with(|| {
        let expected = transfer_amount - roc_fee();
        let expected_adjusted = adjusted_balance(roc(1), expected);
        assert_eq!(Tokens::free_balance(FOREIGN_PARENT_ID, &BOB), expected_adjusted);
    });
}

#[test]
fn transfer_roc_to_relay_chain() {
    TestNet::reset();

    let transfer_amount: Balance = roc(1);
    let transfer_amount_local: Balance = adjusted_balance(roc(1), transfer_amount);
    transfer_roc_from_relay_chain();

    Zeitgeist::execute_with(|| {
        let initial_balance = Tokens::free_balance(FOREIGN_PARENT_ID, &ALICE);
        assert!(initial_balance >= transfer_amount);

        assert_ok!(XTokens::transfer(
            RuntimeOrigin::signed(ALICE),
            FOREIGN_PARENT_ID,
            transfer_amount,
            Box::new(
                MultiLocation::new(1, X1(Junction::AccountId32 { id: BOB.into(), network: None }))
                    .into()
            ),
            WeightLimit::Limited(4_000_000_000.into())
        ));

        assert_eq!(
            Tokens::free_balance(FOREIGN_PARENT_ID, &ALICE),
            initial_balance - transfer_amount_local
        )
    });

    RococoNet::execute_with(|| {
        assert_eq!(rococo_runtime::Balances::free_balance(&BOB), 999_990_415_728);
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
            Tokens::free_balance(FOREIGN_ZTG_ID, &ZeitgeistTreasuryAccount::get());
        assert_eq!(Tokens::free_balance(FOREIGN_ZTG_ID, &BOB), 0);

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
            CurrencyId::Ztg,
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
        let current_balance = Tokens::free_balance(FOREIGN_ZTG_ID, &BOB);
        let custom_fee = calc_fee(default_per_second(10) * 10);

        // Verify that BOB now has (amount transferred - fee)
        assert_eq!(current_balance, transfer_amount - ztg_fee() * fee_factor / BASE);

        // Verify that fees (of foreign currency) have been put into treasury
        assert_eq!(
            Tokens::free_balance(FOREIGN_ZTG_ID, &ZeitgeistTreasuryAccount::get()),
            treasury_initial_balance + custom_fee
        )
    });
}

#[test]
fn test_total_fee() {
    assert_eq!(ztg_fee(), 80_128_000);
    assert_eq!(roc_fee(), 8_012_800_000);
}

#[inline]
fn ztg_fee() -> Balance {
    fee(BalanceFractionalDecimals::get().into())
}

#[inline]
fn fee(decimals: u32) -> Balance {
    calc_fee(default_per_second(decimals))
}

// The fee associated with transferring roc tokens
#[inline]
fn roc_fee() -> Balance {
    fee(12)
}

#[inline]
fn btc_fee() -> Balance {
    fee(8)
}

#[inline]
const fn calc_fee(fee_per_second: Balance) -> Balance {
    // We divide the fee to align its unit and multiply by 8 as that seems to be the unit of
    // time the tests take.
    // NOTE: it is possible that in different machines this value may differ. We shall see.
    fee_per_second / 10_000 * 8
}
