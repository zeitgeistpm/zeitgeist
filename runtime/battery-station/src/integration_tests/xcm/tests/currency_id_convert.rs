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
            foreign_parent_multilocation, foreign_sibling_multilocation, foreign_ztg_multilocation,
            register_foreign_parent, register_foreign_sibling, FOREIGN_PARENT_ID,
            FOREIGN_SIBLING_ID,
        },
        test_net::Zeitgeist,
    },
    xcm_config::config::{battery_station, general_key, AssetConvert},
    CurrencyId,
};

use frame_support::assert_err;
use sp_runtime::traits::Convert as C2;
use xcm::latest::{Junction::*, Junctions::*, MultiLocation};
use xcm_emulator::TestExt;
use xcm_executor::traits::Convert as C1;

#[test]
fn convert_native() {
    assert_eq!(battery_station::KEY.to_vec(), vec![0, 1]);

    // The way Ztg is represented relative within the Zeitgeist runtime
    let ztg_location_inner: MultiLocation =
        MultiLocation::new(0, X1(general_key(battery_station::KEY)));

    assert_eq!(<AssetConvert as C1<_, _>>::convert(ztg_location_inner), Ok(CurrencyId::Ztg));

    // The canonical way Ztg is represented out in the wild
    Zeitgeist::execute_with(|| {
        assert_eq!(
            <AssetConvert as C2<_, _>>::convert(CurrencyId::Ztg),
            Some(foreign_ztg_multilocation())
        )
    });
}

#[test]
fn convert_any_registered_parent_multilocation() {
    Zeitgeist::execute_with(|| {
        assert_err!(
            <AssetConvert as C1<_, _>>::convert(foreign_parent_multilocation()),
            foreign_parent_multilocation()
        );

        assert_eq!(<AssetConvert as C2<_, _>>::convert(FOREIGN_PARENT_ID), None,);

        // Register parent as foreign asset in the Zeitgeist parachain
        register_foreign_parent(None);

        assert_eq!(
            <AssetConvert as C1<_, _>>::convert(foreign_parent_multilocation()),
            Ok(FOREIGN_PARENT_ID),
        );

        assert_eq!(
            <AssetConvert as C2<_, _>>::convert(FOREIGN_PARENT_ID),
            Some(foreign_parent_multilocation())
        );
    });
}

#[test]
fn convert_any_registered_sibling_multilocation() {
    Zeitgeist::execute_with(|| {
        assert_err!(
            <AssetConvert as C1<_, _>>::convert(foreign_sibling_multilocation()),
            foreign_sibling_multilocation()
        );

        assert_eq!(<AssetConvert as C2<_, _>>::convert(FOREIGN_SIBLING_ID), None,);

        // Register parent as foreign asset in the Zeitgeist parachain
        register_foreign_sibling(None);

        assert_eq!(
            <AssetConvert as C1<_, _>>::convert(foreign_sibling_multilocation()),
            Ok(FOREIGN_SIBLING_ID),
        );

        assert_eq!(
            <AssetConvert as C2<_, _>>::convert(FOREIGN_SIBLING_ID),
            Some(foreign_sibling_multilocation())
        );
    });
}

#[test]
fn convert_unkown_multilocation() {
    let unknown_location: MultiLocation =
        MultiLocation::new(1, X2(Parachain(battery_station::ID), general_key(&[42])));

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
