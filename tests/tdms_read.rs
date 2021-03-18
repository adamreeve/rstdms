extern crate hex_literal;

use hex_literal::hex;

#[test]
fn it_does_stuff() {
    let bytes = hex!(
        "FF 0F
        10 00"
    );
    assert_eq!(bytes[0], 255u8);
    assert_eq!(bytes[1], 15u8);
    assert_eq!(bytes[2], 16u8);
    assert_eq!(bytes[3], 0u8);
}
