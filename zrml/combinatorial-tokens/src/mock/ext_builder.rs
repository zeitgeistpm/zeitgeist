// Copyright 2024-2025 Forecasting Technologies LTD.
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

use crate::mock::runtime::{Runtime, System};
use sp_io::TestExternalities;
use sp_runtime::BuildStorage;

#[cfg(feature = "parachain")]
use {
    crate::mock::{consts::FOREIGN_ASSET, runtime::AssetMetadata},
    parity_scale_codec::Encode,
    zeitgeist_primitives::types::CustomMetadata,
};

pub struct ExtBuilder;

impl ExtBuilder {
    pub fn build() -> TestExternalities {
        let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

        // See the logs in tests when using `RUST_LOG=debug cargo test -- --nocapture`
        let _ = env_logger::builder().is_test(true).try_init();

        pallet_balances::GenesisConfig::<Runtime> { balances: vec![] }
            .assimilate_storage(&mut t)
            .unwrap();

        #[cfg(feature = "parachain")]
        {
            orml_tokens::GenesisConfig::<Runtime> { balances: vec![] }
                .assimilate_storage(&mut t)
                .unwrap();

            let custom_metadata =
                CustomMetadata { allow_as_base_asset: true, ..Default::default() };

            orml_asset_registry::module::GenesisConfig::<Runtime> {
                assets: vec![(
                    FOREIGN_ASSET,
                    AssetMetadata {
                        decimals: 18,
                        name: "MKL".as_bytes().to_vec().try_into().unwrap(),
                        symbol: "MKL".as_bytes().to_vec().try_into().unwrap(),
                        existential_deposit: 0,
                        location: None,
                        additional: custom_metadata,
                    }
                    .encode(),
                )],
                last_asset_id: FOREIGN_ASSET,
            }
            .assimilate_storage(&mut t)
            .unwrap();
        }

        let mut test_ext: sp_io::TestExternalities = t.into();

        test_ext.execute_with(|| System::set_block_number(1));

        test_ext
    }
}
