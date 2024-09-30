use super::*;
use rstest::rstest;

// Gnosis test cases using mocked keccak256 results, found here: https://docs.gnosis.io/conditionaltokens/docs/devguide05
#[rstest]
#[case(
    [
        0x52, 0xFF, 0x54, 0xF0, 0xF5, 0x61, 0x6E, 0x34, 0xA2, 0xD4, 0xF5, 0x6F, 0xB6, 0x8A, 0xB4,
        0xCC, 0x63, 0x6B, 0xF0, 0xD9, 0x21, 0x11, 0xDE, 0x74, 0xD1, 0xEC, 0x99, 0x04, 0x0A, 0x8D,
        0xA1, 0x18,
    ],
    None,
    Some([
        0x22, 0x9B, 0x06, 0x7E, 0x14, 0x2F, 0xCE, 0x0A, 0xEA, 0x84, 0xAF, 0xB9, 0x35, 0x09, 0x5C,
        0x6E, 0xCB, 0xEA, 0x86, 0x47, 0xB8, 0xA0, 0x13, 0xE7, 0x95, 0xCC, 0x0C, 0xED, 0x32, 0x10,
        0xA3, 0xD5,
    ])
)]
#[case(
    [
        0xD7, 0x9C, 0x1D, 0x3F, 0x71, 0xF6, 0xC9, 0xD9, 0x98, 0x35, 0x3B, 0xA2, 0xA8, 0x48, 0xE5,
        0x96, 0xF0, 0xC6, 0xC1, 0xA9, 0xF6, 0xFA, 0x63, 0x3F, 0x2C, 0x9E, 0xC6, 0x5A, 0xAA, 0x09,
        0x7C, 0xDC,
    ],
    None,
    Some([
        0x56, 0x0A, 0xE3, 0x73, 0xED, 0x30, 0x49, 0x32, 0xB6, 0xF4, 0x24, 0xC8, 0xA2, 0x43, 0x84,
        0x20, 0x92, 0xC1, 0x17, 0x64, 0x55, 0x33, 0x39, 0x0A, 0x3C, 0x1C, 0x95, 0xFF, 0x48, 0x15,
        0x87, 0xC2,
    ])
)]
#[case(
    [
        0xD7, 0x9C, 0x1D, 0x3F, 0x71, 0xF6, 0xC9, 0xD9, 0x98, 0x35, 0x3B, 0xA2, 0xA8, 0x48, 0xE5,
        0x96, 0xF0, 0xC6, 0xC1, 0xA9, 0xF6, 0xFA, 0x63, 0x3F, 0x2C, 0x9E, 0xC6, 0x5A, 0xAA, 0x09,
        0x7C, 0xDC,
    ],
    Some([
        0x22, 0x9B, 0x06, 0x7E, 0x14, 0x2F, 0xCE, 0x0A, 0xEA, 0x84, 0xAF, 0xB9, 0x35, 0x09, 0x5C,
        0x6E, 0xCB, 0xEA, 0x86, 0x47, 0xB8, 0xA0, 0x13, 0xE7, 0x95, 0xCC, 0x0C, 0xED, 0x32, 0x10,
        0xA3, 0xD5,
    ]),
    Some([
        0x6F, 0x72, 0x2A, 0xA2, 0x50, 0x22, 0x1A, 0xF2, 0xEB, 0xA9, 0x86, 0x8F, 0xC9, 0xD7, 0xD4,
        0x39, 0x94, 0x79, 0x41, 0x77, 0xDD, 0x6F, 0xA7, 0x76, 0x6E, 0x3E, 0x72, 0xBA, 0x3C, 0x11,
        0x19, 0x09,
    ])
)]
#[case(
    [
        0x52, 0xFF, 0x54, 0xF0, 0xF5, 0x61, 0x6E, 0x34, 0xA2, 0xD4, 0xF5, 0x6F, 0xB6, 0x8A, 0xB4,
        0xCC, 0x63, 0x6B, 0xF0, 0xD9, 0x21, 0x11, 0xDE, 0x74, 0xD1, 0xEC, 0x99, 0x04, 0x0A, 0x8D,
        0xA1, 0x18,
    ],
    Some([
        0x56, 0x0A, 0xE3, 0x73, 0xED, 0x30, 0x49, 0x32, 0xB6, 0xF4, 0x24, 0xC8, 0xA2, 0x43, 0x84,
        0x20, 0x92, 0xC1, 0x17, 0x64, 0x55, 0x33, 0x39, 0x0A, 0x3C, 0x1C, 0x95, 0xFF, 0x48, 0x15,
        0x87, 0xC2,
    ]),
    Some([
        0x6F, 0x72, 0x2A, 0xA2, 0x50, 0x22, 0x1A, 0xF2, 0xEB, 0xA9, 0x86, 0x8F, 0xC9, 0xD7, 0xD4,
        0x39, 0x94, 0x79, 0x41, 0x77, 0xDD, 0x6F, 0xA7, 0x76, 0x6E, 0x3E, 0x72, 0xBA, 0x3C, 0x11,
        0x19, 0x09,
    ])
)]
fn get_collection_id_works(
    #[case] hash: Hash,
    #[case] parent_collection_id: Option<Hash>,
    #[values(false, true)] force_max_work: bool,
    #[case] expected: Option<Hash>,
) {
    assert_eq!(get_collection_id(hash, parent_collection_id, force_max_work), expected);
}
