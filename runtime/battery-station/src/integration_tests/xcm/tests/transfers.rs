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
            foreign, ksm, register_foreign_ztg, sibling, sibling_account, zeitgeist_account, ztg,
            ALICE, BOB, FOREIGN_PARENT_ID, FOREIGN_SIBLING_ID, FOREIGN_ZTG_ID, PARA_ID_SIBLING, register_foreign_parent, foreign_ztg_multilocation,
            foreign_parent_multilocation,
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

use frame_support::{assert_ok, traits::tokens::fungible::Mutate};
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



#[test]
fn transfer_ztg_to_sibling() {
    TestNet::reset();

    let alice_initial_balance = ztg(10);
    let transfer_amount = ztg(5);

    Sibling::execute_with(|| {
        assert_eq!(Tokens::free_balance(FOREIGN_ZTG_ID, &BOB.into()), 0);

        // We don't have to register ZTG because ZTG is the native currency
        // of the sibling chain (it is a Zeitgeist clone).
        register_foreign_ztg(None);
    });

    Zeitgeist::execute_with(|| {
        assert_eq!(Balances::free_balance(&ALICE.into()), alice_initial_balance);
        assert_eq!(Balances::free_balance(&sibling_account()), 0);

        assert_ok!(XTokens::transfer(
            Origin::signed(ALICE.into()),
            CurrencyId::Ztg,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(PARA_ID_SIBLING),
                        Junction::AccountId32 { network: NetworkId::Any, id: BOB.into() }
                    )
                )
                .into()
            ),
			4_000_000_000,
        ));

        // Confirm that Alice's balance is initial_balance - amount_transferred
        assert_eq!(Balances::free_balance(&ALICE.into()), alice_initial_balance - transfer_amount);

        // Verify that the amount transferred is now part of the sibling account here
        assert_eq!(Balances::free_balance(&sibling_account()), transfer_amount);
    });

    Sibling::execute_with(|| {
        let current_balance = Tokens::free_balance(FOREIGN_ZTG_ID, &BOB.into());

        // Verify that BOB now has (amount transferred - fee)
        assert_eq!(current_balance, transfer_amount - ztg_fee());

        // Sanity check for the actual amount BOB ends up with
        assert_eq!(current_balance, 49_936_000_000);
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
    let sibling_sovereign_initial_balance = ztg(5);
    let transfer_amount = ztg(1);
    // Note: This asset was registered in `transfer_ztg_to_sibling`

    Zeitgeist::execute_with(|| {
        assert_eq!(Balances::free_balance(&ALICE.into()), alice_initial_balance);
        assert_eq!(Balances::free_balance(&sibling_account()), sibling_sovereign_initial_balance);
    });

    Sibling::execute_with(|| {
        assert_eq!(Balances::free_balance(&zeitgeist_account()), 0);
        assert_eq!(Tokens::free_balance(FOREIGN_ZTG_ID, &BOB.into()), bob_initial_balance);
        assert_ok!(XTokens::transfer(
            Origin::signed(BOB.into()),
            FOREIGN_ZTG_ID,
            transfer_amount,
            Box::new(
                MultiLocation::new(
                    1,
                    X2(
                        Parachain(zeitgeist::ID),
                        Junction::AccountId32 { network: NetworkId::Any, id: ALICE.into() }
                    )
                )
                .into()
            ),
			4_000_000_000,
        ));

        // Confirm that Bobs's balance is initial balance - amount transferred
        assert_eq!(Tokens::free_balance(FOREIGN_ZTG_ID, &BOB.into()), bob_initial_balance - transfer_amount);
    });

    Zeitgeist::execute_with(|| {
        // Verify that ALICE now has initial balance + amount transferred - fee
        assert_eq!(
            Balances::free_balance(&ALICE.into()),
            alice_initial_balance + transfer_amount - ztg_fee(),
        );

        // Verify that the reserve has been adjusted properly
        assert_eq!(
            Balances::free_balance(&sibling_account()),
            sibling_sovereign_initial_balance - transfer_amount
        );
    });
}

#[test]
fn transfer_ksm_from_relay_chain() {
	let transfer_amount: Balance = ksm(1);

    Zeitgeist::execute_with(|| {
        register_foreign_parent(None);
    });

	KusamaNet::execute_with(|| {
		assert_ok!(kusama_runtime::XcmPallet::reserve_transfer_assets(
			kusama_runtime::Origin::signed(ALICE.into()),
			Box::new(Parachain(zeitgeist::ID).into().into()),
			Box::new(
				Junction::AccountId32 {
					network: NetworkId::Any,
					id: BOB,
				}
				.into()
				.into()
			),
			Box::new((Here, transfer_amount).into()),
			0
		));
	});

	Zeitgeist::execute_with(|| {
		assert_eq!(
			Tokens::free_balance(FOREIGN_PARENT_ID, &BOB.into()),
			transfer_amount - ksm_fee()
		);
	});
}

#[test]
fn transfer_ksm_to_relay_chain() {
    let transfer_amount: Balance = ksm(1);
    transfer_ksm_from_relay_chain();

	Zeitgeist::execute_with(|| {
        let initial_balance = Tokens::free_balance(FOREIGN_PARENT_ID, &ALICE.into());
        assert!(initial_balance >= transfer_amount);

		assert_ok!(XTokens::transfer(
			Origin::signed(ALICE.into()),
			FOREIGN_PARENT_ID,
			transfer_amount,
			Box::new(
				MultiLocation::new(
					1,
					X1(Junction::AccountId32 {
						id: BOB,
						network: NetworkId::Any,
					})
				)
				.into()
			),
			4_000_000_000
		));

        assert_eq!(
            Tokens::free_balance(FOREIGN_PARENT_ID, &ALICE.into()),
            initial_balance - transfer_amount
        )
	});

	KusamaNet::execute_with(|| {
		assert_eq!(
			kusama_runtime::Balances::free_balance(&BOB.into()),
			999834059328
		);
	});
}



// Test handling of unknown tokens
// Test treasury assignments


#[test]
fn test_total_fee() {
    assert_eq!(ztg_fee(), 64000000);
    assert_eq!(ksm_fee(), 6400000000);
}


#[inline]
fn ztg_fee() -> Balance {
    fee(BalanceFractionalDecimals::get().into())
}

#[inline]
fn fee(decimals: u32) -> Balance {
    calc_fee(default_per_second(decimals))
}

// The fee associated with transferring KSM tokens
#[inline]
fn ksm_fee() -> Balance {
    fee(12)
}

#[inline]
fn calc_fee(fee_per_second: Balance) -> Balance {
    // We divide the fee to align its unit and multiply by 8 as that seems to be the unit of
    // time the tests take.
    // NOTE: it is possible that in different machines this value may differ. We shall see.
    fee_per_second / 10_000 * 8
}
