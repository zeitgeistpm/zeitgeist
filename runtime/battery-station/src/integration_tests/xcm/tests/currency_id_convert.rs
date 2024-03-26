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
            FOREIGN_SIBLING_ID,
        },
        test_net::Zeitgeist,
    },
    xcm_config::config::{battery_station, general_key, AssetConvert},
    Assets, CustomMetadata, ScalarPosition, XcmAsset,
};
use core::fmt::Debug;
use frame_support::assert_err;
use sp_runtime::traits::Convert as C2;
use test_case::test_case;
use xcm::latest::{Junction::*, Junctions::*, MultiLocation};
use xcm_emulator::TestExt;
use xcm_executor::traits::Convert as C1;

fn convert_common_native<T>(expected: T)
where
    T: Copy + Debug + PartialEq,
    AssetConvert: C1<MultiLocation, T> + C2<T, Option<MultiLocation>>,
{
    assert_eq!(battery_station::KEY.to_vec(), vec![0, 1]);

    // The way Ztg is represented relative within the Zeitgeist runtime
    let ztg_location_inner: MultiLocation =
        MultiLocation::new(0, X1(general_key(battery_station::KEY)));

    assert_eq!(<AssetConvert as C1<_, _>>::convert(ztg_location_inner), Ok(expected));

    // The canonical way Ztg is represented out in the wild
    Zeitgeist::execute_with(|| {
        assert_eq!(<AssetConvert as C2<_, _>>::convert(expected), Some(foreign_ztg_multilocation()))
    });
}

fn convert_common_non_native<T>(
    expected: T,
    multilocation: MultiLocation,
    register: fn(Option<CustomMetadata>),
) where
    T: Copy + Debug + PartialEq,
    AssetConvert: C1<MultiLocation, T> + C2<T, Option<MultiLocation>>,
{
    Zeitgeist::execute_with(|| {
        assert_err!(<AssetConvert as C1<_, _>>::convert(multilocation), multilocation);
        assert_eq!(<AssetConvert as C2<_, _>>::convert(expected), None);
        // Register parent as foreign asset in the Zeitgeist parachain
        register(None);
        assert_eq!(<AssetConvert as C1<_, _>>::convert(multilocation), Ok(expected));
        assert_eq!(<AssetConvert as C2<_, _>>::convert(expected), Some(multilocation));
    });
}

#[test]
fn convert_native_assets() {
    convert_common_native(Assets::Ztg);
}

#[test]
fn convert_native_xcm_assets() {
    convert_common_native(XcmAsset::Ztg);
}

#[test]
fn convert_any_registered_parent_multilocation_assets() {
    convert_common_non_native(
        Assets::from(FOREIGN_PARENT_ID),
        foreign_parent_multilocation(),
        register_foreign_parent,
    );
}

#[test]
fn convert_any_registered_parent_multilocation_xcm_assets() {
    convert_common_non_native(
        XcmAsset::try_from(Assets::from(FOREIGN_PARENT_ID)).unwrap(),
        foreign_parent_multilocation(),
        register_foreign_parent,
    );
}

#[test]
fn convert_any_registered_sibling_multilocation_assets() {
    convert_common_non_native(
        Assets::from(FOREIGN_SIBLING_ID),
        foreign_sibling_multilocation(),
        register_foreign_sibling,
    );
}

#[test]
fn convert_any_registered_sibling_multilocation_xcm_assets() {
    convert_common_non_native(
        XcmAsset::try_from(Assets::from(FOREIGN_SIBLING_ID)).unwrap(),
        foreign_sibling_multilocation(),
        register_foreign_sibling,
    );
}

#[test]
fn convert_unkown_multilocation() {
    let unknown_location: MultiLocation =
        MultiLocation::new(1, X2(Parachain(battery_station::ID), general_key(&[42])));

    Zeitgeist::execute_with(|| {
        assert!(<AssetConvert as C1<_, Assets>>::convert(unknown_location).is_err());
    });
}

#[test_case(
    Assets::CategoricalOutcome(7, 8);
    "assets_categorical"
)]
#[test_case(
    Assets::ScalarOutcome(7, ScalarPosition::Long);
    "assets_scalar"
)]
#[test_case(
    Assets::PoolShare(7);
    "assets_pool_share"
)]
#[test_case(
    Assets::ForeignAsset(7);
    "assets_foreign"
)]
#[test_case(
    Assets::ParimutuelShare(7, 8);
    "assets_parimutuel_share"
)]
#[test_case(
    Assets::CampaignAsset(7);
    "assets_campaign_asset"
)]
#[test_case(
    Assets::CustomAsset(7);
    "assets_custom_asset"
)]
#[test_case(
    XcmAsset::ForeignAsset(7);
    "xcm_assets_foreign"
)]
fn convert_unsupported_asset<T>(asset: T)
where
    T: Copy + Debug + PartialEq,
    AssetConvert: C2<T, Option<MultiLocation>>,
{
    Zeitgeist::execute_with(|| assert_eq!(<AssetConvert as C2<_, _>>::convert(asset), None));
}
