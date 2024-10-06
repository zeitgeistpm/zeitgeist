use crate::mock::runtime::{Runtime, System};
use sp_io::TestExternalities;
use sp_runtime::BuildStorage;

#[cfg(feature = "parachain")]
use {crate::mock::consts::FOREIGN_ASSET, zeitgeist_primitives::types::CustomMetadata};

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

            orml_asset_registry::GenesisConfig::<Runtime> {
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
