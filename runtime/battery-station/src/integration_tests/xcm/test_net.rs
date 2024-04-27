// Copyright 2022-2024 Forecasting Technologies LTD.
// Copyright 2021-2022 Centrifuge GmbH (centrifuge.io).
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
    parameters::ZeitgeistTreasuryAccount, xcm_config::config::{LocationToAccountId}, battery_station, Assets, DmpQueue,
    XcmpQueue, ParachainInfo, PolkadotXcm, AssetManager, Balances, XTokens
};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use sp_runtime::BuildStorage;
use xcm_emulator::{decl_test_networks, decl_test_parachains, decl_test_relay_chains, DefaultMessageProcessor};

use super::setup::{roc, ztg, FOREIGN_PARENT_ID, PARA_ID_SIBLING, PARA_ID_BATTERY_STATION};
use super::genesis::{battery_station as battery_station_genesis, rococo};

decl_test_relay_chains! {
	#[api_version(5)]
	pub struct Rococo {
		genesis = rococo::genesis(),
		on_init = (),
		runtime = rococo_runtime,
		core = {
            MessageProcessor: DefaultMessageProcessor<Rococo>,
			SovereignAccountOf: rococo_runtime::xcm_config::LocationConverter,
		},
		pallets = {
			XcmPallet: rococo_runtime::XcmPallet,
			Sudo: rococo_runtime::Sudo,
			Balances: rococo_runtime::Balances,
		}
	},
}

decl_test_parachains! {
	pub struct BatteryStation {
		genesis = battery_station_genesis::genesis(PARA_ID_BATTERY_STATION),
		on_init = (),
		runtime = crate,
		core = {
			XcmpMessageHandler: XcmpQueue,
			DmpMessageHandler: DmpQueue,
			LocationToAccountId: LocationToAccountId,
			ParachainInfo: ParachainInfo,
		},
		pallets = {
			PolkadotXcm: PolkadotXcm,
			AssetManager: AssetManager,
			Balances: Balances,
            XTokens: XTokens,
		}
	},
	pub struct Sibling {
		genesis = battery_station_genesis::genesis(PARA_ID_SIBLING),
		on_init = (),
		runtime = crate,
		core = {
			XcmpMessageHandler: XcmpQueue,
			DmpMessageHandler: DmpQueue,
			LocationToAccountId: LocationToAccountId,
			ParachainInfo: ParachainInfo,
		},
		pallets = {
			PolkadotXcm: PolkadotXcm,
			AssetManager: AssetManager,
			Balances: Balances,
            XTokens: XTokens,
		}
	},
}

decl_test_networks! {
    pub struct TestNet {
        relay_chain = Rococo,
        parachains = vec![
            BatteryStation,
			Sibling,
        ],
        bridge = ()
    }
}