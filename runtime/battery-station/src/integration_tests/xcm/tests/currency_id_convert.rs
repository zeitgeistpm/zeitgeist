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
            PARA_ID_SIBLING, FOREIGN_KSM_ID,
        },
        test_net::{KusamaNet, Sibling, TestNet, Zeitgeist},
    },
    xcm_config::{
        asset_registry::{CustomMetadata, XcmMetadata},
        config::{general_key, zeitgeist, AssetConvert},
        fees::default_per_second,
    },
    AssetRegistry, Balance, Balances, CurrencyId, Origin, Tokens, XTokens,
};

use frame_support::assert_ok;
use orml_traits::{asset_registry::AssetMetadata, FixedConversionRateProvider, MultiCurrency};
use parity_scale_codec::Encode;
use sp_runtime::traits::Convert as C2;
use xcm::{
	latest::{Junction, Junction::*, Junctions::*, MultiLocation, NetworkId},
	VersionedMultiLocation,
};
use xcm_emulator::TestExt;
use xcm_executor::traits::Convert as C1;


#[test]
fn convert_native() {
    assert_eq!(zeitgeist::KEY.to_vec(), vec![0, 1]);

    // The way Ztg is represented relative within the Zeitgeist runtime
    let ztg_location_inner: MultiLocation = MultiLocation::new(0, X1(general_key(zeitgeist::KEY)));

    assert_eq!(<AssetConvert as C1<_, _>>::convert(ztg_location_inner), Ok(CurrencyId::Ztg),);

    // The canonical way Ztg is represented out in the wild
    let air_location_canonical: MultiLocation =
        MultiLocation::new(1, X2(Parachain(zeitgeist::ID), general_key(zeitgeist::KEY)));

    Zeitgeist::execute_with(|| {
        assert_eq!(
            <AssetConvert as C2<_, _>>::convert(CurrencyId::Ztg),
            Some(air_location_canonical)
        )
    });
}

#[test]
fn convert_any_registered_multilocation() {
    let ksm_location: MultiLocation = MultiLocation::parent().into();

    Zeitgeist::execute_with(|| {
        // Register KSM as foreign asset in the sibling parachain
        let meta: AssetMetadata<Balance, CustomMetadata> = AssetMetadata {
            decimals: 12,
            name: "Kusama".into(),
            symbol: "KSM".into(),
            existential_deposit: 10_000_000_000,
            location: Some(VersionedMultiLocation::V1(ksm_location.clone())),
            additional: CustomMetadata::default(),
        };

        assert_ok!(AssetRegistry::register_asset(
            Origin::root(),
            meta,
            Some(FOREIGN_KSM_ID)
        ));

        assert_eq!(
            <AssetConvert as C1<_, _>>::convert(ksm_location.clone()),
            Ok(FOREIGN_KSM_ID),
        );

        assert_eq!(
            <AssetConvert as C2<_, _>>::convert(FOREIGN_KSM_ID),
            Some(ksm_location)
        )
    });
}

#[test]
fn convert_unkown_multilocation() {
    let unknown_location: MultiLocation =
        MultiLocation::new(1, X2(Parachain(zeitgeist::ID), general_key(&[42])));

    Zeitgeist::execute_with(|| {
        assert!(<AssetConvert as C1<_, _>>::convert(unknown_location.clone()).is_err());
    });
}

#[test]
fn convert_unsupported_currency() {
    Zeitgeist::execute_with(|| {
        assert_eq!(<AssetConvert as C2<_, _>>::convert(CurrencyId::CombinatorialOutcome), None)
    });
}
