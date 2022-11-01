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
	Balances, CurrencyId, Origin, AssetRegistry, Tokens, XTokens,
    xcm_config::{
        asset_registry::{CustomMetadata, XcmMetadata},
        config::{general_key, zeitgeist, AssetConvert},
        fees::default_per_second,
    },
    integration_tests::xcm::{
        setup::{
            ztg, zeitgeist_account, sibling, foreign, ksm, sibling_account, ALICE, BOB,
            PARA_ID_SIBLING,
        },
        test_net::{Zeitgeist, KusamaNet, Sibling, TestNet},
    },
};

use parity_scale_codec::Encode;
use frame_support::assert_ok;
use orml_traits::{asset_registry::AssetMetadata, FixedConversionRateProvider, MultiCurrency};
use sp_runtime::{
	traits::{Convert as C2},
};
use xcm::{
	latest::{Error::BadOrigin, Junction, Junction::*, Junctions::*, MultiLocation, NetworkId},
	VersionedMultiLocation,
};
use xcm_emulator::TestExt;
use xcm_executor::traits::Convert as C1;

/*

#[test]
fn convert_native() {
	assert_eq!(parachains::kusama::altair::AIR_KEY.to_vec(), vec![0, 1]);

	// The way AIR is represented relative within the Altair runtime
	let air_location_inner: MultiLocation =
		MultiLocation::new(0, X1(general_key(parachains::kusama::altair::AIR_KEY)));

	assert_eq!(
		<AssetConvert as C1<_, _>>::convert(air_location_inner),
		Ok(CurrencyId::Native),
	);

	// The canonical way AIR is represented out in the wild
	let air_location_canonical: MultiLocation = MultiLocation::new(
		1,
		X2(
			Parachain(parachains::kusama::altair::ID),
			general_key(parachains::kusama::altair::AIR_KEY),
		),
	);

	Altair::execute_with(|| {
		assert_eq!(
			<AssetConvert as C2<_, _>>::convert(CurrencyId::Native),
			Some(air_location_canonical)
		)
	});
}

#[test]
fn convert_sibling() {
	assert_eq!(parachains::kusama::karura::AUSD_KEY, &[0, 129]);

	let ausd_location: MultiLocation = MultiLocation::new(
		1,
		X2(
			Parachain(parachains::kusama::karura::ID),
			general_key(parachains::kusama::karura::AUSD_KEY),
		),
	);

	Altair::execute_with(|| {
		assert_eq!(
			<AssetConvert as C1<_, _>>::convert(ausd_location.clone()),
			Ok(CurrencyId::AUSD),
		);

		assert_eq!(
			<AssetConvert as C2<_, _>>::convert(CurrencyId::AUSD),
			Some(ausd_location)
		)
	});
}

#[test]
fn convert_parent() {
	let ksm_location: MultiLocation = MultiLocation::parent().into();

	Altair::execute_with(|| {
		assert_eq!(
			<AssetConvert as C1<_, _>>::convert(ksm_location.clone()),
			Ok(CurrencyId::KSM),
		);

		assert_eq!(
			<AssetConvert as C2<_, _>>::convert(CurrencyId::KSM),
			Some(ksm_location)
		)
	});
}

*/

#[test]
fn convert_unkown_multilocation() {
	let unknown_location: MultiLocation = MultiLocation::new(
		1,
		X2(
			Parachain(zeitgeist::ID),
			general_key(&[42]),
		),
	);

	Zeitgeist::execute_with(|| {
		assert!(<AssetConvert as C1<_, _>>::convert(unknown_location.clone()).is_err());
	});
}

#[test]
fn convert_unsupported_currency() {
	Zeitgeist::execute_with(|| {
		assert_eq!(
			<AssetConvert as C2<_, _>>::convert(CurrencyId::CombinatorialOutcome),
			None
		)
	});
}