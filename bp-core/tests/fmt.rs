use bp_core::utils::fmt::*;

#[test]
fn test_to_hex_display() {
    assert_eq!(format!("{}", ToHex(vec![1, 10, 255])), "01 0A FF");
    assert_eq!(
        format!("{}", ToHex(vec![0; MAX_DISPLAY_BYTES + 1])),
        "00 ".repeat(MAX_DISPLAY_BYTES) + "... 1 bytes omitted"
    );
}
