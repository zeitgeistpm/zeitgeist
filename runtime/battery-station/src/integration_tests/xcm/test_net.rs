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
    parameters::ZeitgeistTreasuryAccount, xcm_config::config::battery_station, Assets, DmpQueue,
    XcmpQueue,
};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use sp_runtime::BuildStorage;
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};
use xcm_simulator::TestExt;

use super::setup::{roc, ztg, ExtBuilder, ALICE, FOREIGN_PARENT_ID, PARA_ID_SIBLING};

decl_test_relay_chain! {
    pub struct RococoNet {
        Runtime = rococo_runtime::Runtime,
		RuntimeCall = rococo_runtime::RuntimeCall,
		RuntimeEvent = rococo_runtime::RuntimeEvent,
		XcmConfig = rococo_runtime::XcmConfig,
		MessageQueue = rococo_runtime::MessageQueue,
		System = rococo_runtime::System,
        new_ext = relay_ext(),
    }
}

decl_test_parachain! {
    pub struct Zeitgeist {
        Runtime = Runtime,
        XcmpMessageHandler = XcmpQueue,
        DmpMessageHandler = DmpQueue,
        new_ext = para_ext(battery_station::ID),
    }
}

decl_test_parachain! {
    pub struct Sibling {
        Runtime = Runtime,
        XcmpMessageHandler = XcmpQueue,
        DmpMessageHandler = DmpQueue,
        new_ext = para_ext(PARA_ID_SIBLING),
    }
}

decl_test_network! {
    pub struct TestNet {
        relay_chain = RococoNet,
        parachains = vec![
            // N.B: Ideally, we could use the defined para id constants but doing so
            // fails with: "error: arbitrary expressions aren't allowed in patterns"

            // Be sure to use `xcm_config::config::battery_station::ID`
            (2101, Zeitgeist),
            // Be sure to use `PARA_ID_SIBLING`
            (3000, Sibling),
        ],
    }
}

pub(super) fn relay_ext() -> sp_io::TestExternalities {
    use rococo_runtime::{Runtime, System};

    let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

    pallet_balances::GenesisConfig::<Runtime> { balances: vec![(ALICE, roc(2002))] }
        .assimilate_storage(&mut t)
        .unwrap();

    polkadot_runtime_parachains::configuration::GenesisConfig::<Runtime> {
        config: mock_relay_config(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_xcm::GenesisConfig::<Runtime> { _config: Default::default(), safe_xcm_version: Some(2) }
        .assimilate_storage(&mut t)
        .unwrap();

    let mut ext = sp_io::TestExternalities::new(t);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub(super) fn para_ext(parachain_id: u32) -> sp_io::TestExternalities {
    let _ = env_logger::builder().is_test(true).try_init();
    
    ExtBuilder::default()
        .set_balances(vec![
            (ALICE, Assets::Ztg, ztg(10)),
            (ALICE, FOREIGN_PARENT_ID.into(), roc(10)),
            (ZeitgeistTreasuryAccount::get(), FOREIGN_PARENT_ID.into(), roc(1)),
        ])
        .set_parachain_id(parachain_id)
        .with_safe_xcm_version(3)
        .build()
}

pub fn mock_relay_config() -> HostConfiguration<polkadot_primitives::BlockNumber> {
	HostConfiguration::<polkadot_primitives::BlockNumber> {
		hrmp_channel_max_capacity: u32::MAX,
		hrmp_channel_max_total_size: u32::MAX,
		hrmp_max_parachain_inbound_channels: 10,
		hrmp_max_parachain_outbound_channels: 10,
		hrmp_channel_max_message_size: u32::MAX,
		// Changed to avoid aritmetic errors within hrmp_close
		max_downward_message_size: 100_000u32,
		..Default::default()
	}
}
