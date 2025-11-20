// Copyright 2022-2025 Forecasting Technologies LTD.
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

use super::{
    genesis::battery_station,
    setup::{PARA_ID_BATTERY_STATION, PARA_ID_SIBLING},
};
use crate::{
    xcm_config::config::LocationToAccountId, AssetManager, Balances, ParachainInfo, PolkadotXcm,
    XTokens, XcmpQueue,
};
use rococo_emulated_chain::Rococo;
use xcm_emulator::{decl_test_networks, decl_test_parachains};

decl_test_parachains! {
    pub struct BatteryStation {
        genesis = battery_station::genesis(PARA_ID_BATTERY_STATION),
        on_init = (),
        runtime = crate,
        core = {
            XcmpMessageHandler: XcmpQueue,
            LocationToAccountId: LocationToAccountId,
            ParachainInfo: ParachainInfo,
            MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
        },
        pallets = {
            PolkadotXcm: PolkadotXcm,
            AssetManager: AssetManager,
            Balances: Balances,
            XTokens: XTokens,
        }
    },
    pub struct Sibling {
        genesis = battery_station::genesis(PARA_ID_SIBLING),
        on_init = (),
        runtime = crate,
        core = {
            XcmpMessageHandler: XcmpQueue,
            LocationToAccountId: LocationToAccountId,
            ParachainInfo: ParachainInfo,
            MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
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
