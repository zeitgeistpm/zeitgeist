// Copyright 2024 Forecasting Technologies LTD.
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

/*
pub(super) struct ExtBuilder {
    balances: Vec<(AccountId, Assets, Balance)>,
    parachain_id: u32,
    safe_xcm_version: Option<u32>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self { balances: vec![], parachain_id: battery_station::ID, safe_xcm_version: None }
    }
}

impl ExtBuilder {
    pub fn set_balances(mut self, balances: Vec<(AccountId, Assets, Balance)>) -> Self {
        self.balances = balances;
        self
    }

    pub fn set_parachain_id(mut self, parachain_id: u32) -> Self {
        self.parachain_id = parachain_id;
        self
    }

    pub fn with_safe_xcm_version(mut self, safe_xcm_version: u32) -> Self {
        self.safe_xcm_version = Some(safe_xcm_version);
        self
    }

    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
        let native_currency_id = Assets::Ztg;

        pallet_balances::GenesisConfig::<Runtime> {
            balances: self
                .balances
                .clone()
                .into_iter()
                .filter(|(_, currency_id, _)| *currency_id == native_currency_id)
                .map(|(account_id, _, initial_balance)| (account_id, initial_balance))
                .collect::<Vec<_>>(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        orml_tokens::GenesisConfig::<Runtime> {
            balances: self
                .balances
                .into_iter()
                .filter(|(_, currency_id, _)| *currency_id != native_currency_id)
                .map(|(account_id, currency_id, initial_balance)| {
                    (account_id, currency_id.try_into().unwrap(), initial_balance)
                })
                .collect::<Vec<_>>(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        parachain_info::GenesisConfig::<Runtime> {
            _config: Default::default(),
            parachain_id: self.parachain_id.into(),
        }
        .assimilate_storage(&mut t)
        .unwrap();

        pallet_xcm::GenesisConfig::<Runtime> {
            _config: Default::default(),
            safe_xcm_version: self.safe_xcm_version,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}
*/