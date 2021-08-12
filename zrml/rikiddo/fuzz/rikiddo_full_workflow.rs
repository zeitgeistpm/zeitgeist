#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use zrml_rikiddo::mock::ExtBuilder;

fuzz_target!(|data: Data| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
    });
    let _ = ext.commit_all();
});

#[derive(Debug, Arbitrary)]
struct Data {
}