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
            foreign_parent_multilocation, foreign_sibling_multilocation, foreign_ztg_multilocation,
            register_foreign_parent, register_foreign_sibling, FOREIGN_PARENT_ID,
            FOREIGN_SIBLING_ID, PARA_ID_BATTERY_STATION,
        },
        test_net::BatteryStation,
    },
    xcm_config::config::{battery_station, general_key, AssetConvert},
    CurrencyId,
};
use core::fmt::Debug;
use sp_runtime::traits::{Convert, MaybeEquivalence};
use test_case::test_case;
use xcm::latest::{Junction::*, Junctions::*, MultiLocation};
use xcm_emulator::TestExt;
use zeitgeist_primitives::types::{Asset, CustomMetadata, ScalarPosition};

fn convert_common_native<T>(expected: T)
where
    T: Copy + Debug + PartialEq,
    AssetConvert: MaybeEquivalence<MultiLocation, T> + Convert<T, Option<MultiLocation>>,
{
    assert_eq!(battery_station::KEY.to_vec(), vec![0, 1]);

    // The way Ztg is represented relative within the Battery Station runtime
    let ztg_location_inner: MultiLocation =
        MultiLocation::new(0, X1(general_key(battery_station::KEY)));

    assert_eq!(
        <AssetConvert as MaybeEquivalence<_, _>>::convert(&ztg_location_inner),
        Some(expected)
    );

    // The canonical way Ztg is represented out in the wild
    BatteryStation::execute_with(|| {
        assert_eq!(
            <AssetConvert as Convert<_, _>>::convert(expected),
            Some(foreign_ztg_multilocation())
        )
    });
}

fn convert_common_non_native<T>(
    expected: T,
    multilocation: MultiLocation,
    register: fn(Option<CustomMetadata>),
) where
    T: Copy + Debug + PartialEq,
    AssetConvert: MaybeEquivalence<MultiLocation, T> + Convert<T, Option<MultiLocation>>,
{
    BatteryStation::execute_with(|| {
        assert_eq!(<AssetConvert as MaybeEquivalence<_, _>>::convert(&multilocation), None);
        assert_eq!(<AssetConvert as Convert<_, _>>::convert(expected), None);
        // Register parent as foreign asset in the Battery Station parachain
        register(None);
        assert_eq!(
            <AssetConvert as MaybeEquivalence<_, _>>::convert(&multilocation),
            Some(expected)
        );
        assert_eq!(<AssetConvert as Convert<_, _>>::convert(expected), Some(multilocation));
    });
}

#[test]
fn convert_native_assets() {
    convert_common_native(Asset::Ztg);
}

#[test]
fn convert_any_registered_parent_multilocation_assets() {
    convert_common_non_native(
        FOREIGN_PARENT_ID,
        foreign_parent_multilocation(),
        register_foreign_parent,
    );
}

#[test]
fn convert_any_registered_parent_multilocation_xcm_assets() {
    convert_common_non_native(
        FOREIGN_PARENT_ID,
        foreign_parent_multilocation(),
        register_foreign_parent,
    );
}

#[test]
fn convert_any_registered_sibling_multilocation_assets() {
    convert_common_non_native(
        FOREIGN_SIBLING_ID,
        foreign_sibling_multilocation(),
        register_foreign_sibling,
    );
}

#[test]
fn convert_any_registered_sibling_multilocation_xcm_assets() {
    convert_common_non_native(
        FOREIGN_SIBLING_ID,
        foreign_sibling_multilocation(),
        register_foreign_sibling,
    );
}

#[test]
fn convert_unkown_multilocation() {
    let unknown_location: MultiLocation =
        MultiLocation::new(1, X2(Parachain(PARA_ID_BATTERY_STATION), general_key(&[42])));

    BatteryStation::execute_with(|| {
        assert!(
            <AssetConvert as MaybeEquivalence<_, CurrencyId>>::convert(&unknown_location).is_none()
        );
    });
}

#[test_case(Asset::CategoricalOutcome(7, 8))]
#[test_case(Asset::ScalarOutcome(7, ScalarPosition::Long))]
#[test_case(Asset::PoolShare(7))]
#[test_case(Asset::ForeignAsset(7))]
#[test_case(Asset::ParimutuelShare(7, 8))]
fn convert_unsupported_asset<T>(asset: T)
where
    T: Copy + Debug + PartialEq,
    AssetConvert: Convert<T, Option<MultiLocation>>,
{
    BatteryStation::execute_with(|| {
        assert_eq!(<AssetConvert as Convert<_, _>>::convert(asset), None)
    });
}
