#![no_main]
//! Fuzz test: Rikiddo pallet is called with calculated fee
//!   -> create, force fee by multiple update_volume, cost, price, all_prices, clear, destroy

use arbitrary::Arbitrary;
use frame_support::traits::{OnFinalize, OnInitialize};
use frame_system::RawOrigin;
use libfuzzer_sys::fuzz_target;
use substrate_fixed::{types::extra::U33, FixedI128, FixedU128};
use zrml_rikiddo::{Config, mock::*, traits::RikiddoSigmoidMVPallet, types::{EmaMarketVolume, FeeSigmoid, RikiddoSigmoidMV, Timespan}};

fn run_to_block(n: u64) {
    while System::block_number() < n {
        Timestamp::on_finalize(System::block_number());
        Balances::on_finalize(System::block_number());
        Rikiddo::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        Timestamp::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
        Rikiddo::on_initialize(System::block_number());
    }
}

fuzz_target!(|data: Data| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let mut rikiddo: RikiddoSigmoidMV<
            FixedU128<U33>,
            FixedI128<U33>,
            FeeSigmoid<FixedI128<U33>>,
            EmaMarketVolume<FixedU128<U33>>,
        > = Default::default();

        rikiddo.ma_short.config.ema_period = Timespan::Seconds(0);
        rikiddo.ma_long.config.ema_period = Timespan::Seconds(1);
        let pool_id = 0;
        let mut current_block = 0;
        let _ = Rikiddo::create(pool_id, rikiddo);
        let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 0).unwrap();

        // Initialize ma_short and ma_long ema
        for (idx, volume) in data.update_volumes.iter().enumerate() {
            let _ = Rikiddo::update_volume(pool_id, *volume);

            if idx % 2 == 1 {
                current_block += 1;
                run_to_block(current_block);
                let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), current_block).unwrap();
            }
        }

        let _ = Rikiddo::cost(pool_id, &data.asset_balances);
        let _ = Rikiddo::price(pool_id, data.price_for, &data.asset_balances);
        let _ = Rikiddo::all_prices(pool_id, &data.asset_balances);
        let _ = Rikiddo::fee(pool_id);
    });
    let _ = ext.commit_all();
});

#[derive(Debug, Arbitrary)]
struct Data {
    asset_balances: [<Runtime as Config>::Balance; 8],
    price_for: <Runtime as Config>::Balance,
    update_volumes: [<Runtime as Config>::Balance; 5],
}
