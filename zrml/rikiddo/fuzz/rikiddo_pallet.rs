#![no_main]
//! Fuzz test: Rikiddo pallet is called with calculated fee
//!   -> create, force fee by multiple update_volume, cost, price, all_prices, clear, destroy

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use zrml_rikiddo::mock::ExtBuilder;

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
        rikiddo.config.ma_short(Timespan::Seconds(0));
        rikiddo.config.ma_long(Timespan::Seconds(1));
        let pool_id = 0;
        let mut current_timestamp = 0;
        Rikiddo::create(pool_id, rikiddo);
        run_to_block(1);
        let mut current_block = 1;
        let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), 0).unwrap();

        // Initialize ma_short and ma_long ema
        for (idx, volume) in data.update_volumes.iter().enumerate() {
            let timestamped_volume = TimestampedVolume {
                timestamp: (idx / 2) as UnixTimestamp,
                volume: fixed_from_u128(*volume),
            };
            let _ = Rikiddo::update_volume(&timestamped_volume);

            if idx % 2 == 1 && idx != 5 {
                current_block += 1;
                current_timestamp += 1;
                run_to_block(current_block);
                let _ = <Runtime as Config>::Timestamp::set(RawOrigin::None.into(), current_timestamp).unwrap()
            }
        }

    });
    let _ = ext.commit_all();
});

#[derive(Debug, Arbitrary)]
struct Data {
    asset_balances: [Balance; 8],
    price_for: Balance,
    update_volumes: [Balance; 5],
}
