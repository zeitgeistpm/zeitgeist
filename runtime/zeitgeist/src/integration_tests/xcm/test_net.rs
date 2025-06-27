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
    genesis::zeitgeist,
    setup::{PARA_ID_SIBLING, PARA_ID_ZEITGEIST},
};
use crate::{
    xcm_config::config::LocationToAccountId, AssetManager, Balances, ParachainInfo, PolkadotXcm,
    XTokens, XcmpQueue,
};
use polkadot_emulated_chain::Polkadot;
use xcm_emulator::{decl_test_networks, decl_test_parachains};

decl_test_parachains! {
    pub struct Zeitgeist {
        genesis = zeitgeist::genesis(PARA_ID_ZEITGEIST),
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
        genesis = zeitgeist::genesis(PARA_ID_SIBLING),
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
        relay_chain = Polkadot,
        parachains = vec![
            Zeitgeist,
            Sibling,
        ],
        bridge = ()
    }
}
