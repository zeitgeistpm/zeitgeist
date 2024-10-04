use super::*;

fn field_modulus_works() {
    let expected = U256::from_str_prefixed(
        "0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47",
    )
    .unwrap();
    assert_eq!(field_modulus(), expected);
}

