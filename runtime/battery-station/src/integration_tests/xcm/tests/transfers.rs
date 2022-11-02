// Copyright 2021 Centrifuge Foundation (centrifuge.io).
// Copyright 2022 Forecasting Technologies LTD.
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
            foreign, ksm, sibling, sibling_account, zeitgeist_account, ztg, ALICE, BOB,
            FOREIGN_PARENT_ID, FOREIGN_SIBLING_ID, FOREIGN_ZTG_ID, PARA_ID_SIBLING,
        },
        test_net::{KusamaNet, Sibling, TestNet, Zeitgeist},
    },
    xcm_config::{
        asset_registry::{CustomMetadata, XcmMetadata},
        config::{general_key, zeitgeist, AssetConvert},
        fees::default_per_second,
    },
    AssetRegistry, Balance, Balances, CurrencyId, ExistentialDeposit, Origin, Tokens, XTokens,
};

use frame_support::assert_ok;
use orml_traits::{asset_registry::AssetMetadata, FixedConversionRateProvider, MultiCurrency};
use parity_scale_codec::Encode;
use sp_runtime::traits::Convert as C2;
use xcm::{
    latest::{Error::BadOrigin, Junction, Junction::*, Junctions::*, MultiLocation, NetworkId},
    VersionedMultiLocation,
};
use xcm_emulator::TestExt;
use xcm_executor::traits::Convert as C1;
use zeitgeist_primitives::constants::BalanceFractionalDecimals;

/*

#[test]
fn transfer_ztg_to_sibling() {
    TestNet::reset();

    let alice_initial_balance = ztg(10);
    let bob_initial_balance = ztg(10);
    let transfer_amount = ztg(5);
    let ztg_in_sibling = FOREIGN_ZTG_ID;

    Zeitgeist::execute_with(|| {
        assert_eq!(Balances::free_balance(&ALICE.into()), alice_initial_balance);
        assert_eq!(Balances::free_balance(&sibling_account()), 0);
    });

    Sibling::execute_with(|| {
        assert_eq!(
            Tokens::free_balance(ztg_in_sibling.clone(), &BOB.into()),
            0
        );

        // Register ZTG as foreign asset in the sibling parachain
        let meta: AssetMetadata<Balance, CustomMetadata> = AssetMetadata {
            decimals: 10,
            name: "Zeitgeist".into(),
            symbol: "ZTG".into(),
            existential_deposit: ExistentialDeposit::get(),
            location: Some(VersionedMultiLocation::V1(MultiLocation::new(
                1,
                X2(
                    Parachain(zeitgeist::ID),
                    general_key(zeitgeist::KEY),
                ),
            ))),
            additional: CustomMetadata::default(),
        };
        assert_ok!(OrmlAssetRegistry::register_asset(
            Origin::root(),
            meta,
            Some(ztg_in_sibling.clone())
        ));
    });

    Zeitgeist::execute_with(|| {
        assert_ok!(XTokens::transfer(
            Origin::signed(ALICE.into()),
            CurrencyId::Native,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 {
                            network: NetworkId::Any,
                            id: BOB.into(),
                        }
                    )
                )
                .into()
            ),
            8_000_000_000_000,
        ));

        // Confirm that Alice's balance is initial balance - amount transferred
        assert_eq!(
            Balances::free_balance(&ALICE.into()),
            alice_initial_balance - transfer_amount
        );

        // Verify that the amount transferred is now part of the sibling account here
        assert_eq!(Balances::free_balance(&sibling_account()), transfer_amount);
    });

    Sibling::execute_with(|| {
        let current_balance = OrmlTokens::free_balance(ztg_in_sibling, &BOB.into());

        // Verify that BOB now has (amount transferred - fee)
        assert_eq!(current_balance, transfer_amount - fee(18));

        // Sanity check for the actual amount BOB ends up with
        assert_eq!(current_balance, 4990730400000000000);
    });
}

*/
/*

#[test]
fn test_total_fee() {
    assert_eq!(ztg_fee(), 926960000000);
    assert_eq!(fee(12), 9269600000);
}

fn ztg_fee() -> Balance {
    fee(BalanceFractionalDecimals::get().into())
}

fn fee(decimals: u32) -> Balance {
    calc_fee(default_per_second(decimals))
}

// The fee associated with transferring KSM tokens
fn ksm_fee() -> Balance {
    fee(12)
}

fn calc_fee(fee_per_second: Balance) -> Balance {
    // We divide the fee to align its unit and multiply by 4 as that seems to be the unit of
    // time the tests take.
    // NOTE: it is possible that in different machines this value may differ. We shall see.
    fee_per_second //.div_euclid(10_000) * 8
}
*/
