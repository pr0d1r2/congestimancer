use super::{decode_hex, ne_i32, ne_i64, ne_u32, ne_u64};

#[test]
fn integers_read_native_endian_at_an_offset() {
    let mut bytes = vec![0xaa];
    bytes.extend(7_i32.to_ne_bytes());
    bytes.extend((-2_i32).to_ne_bytes());
    bytes.extend(u64::MAX.to_ne_bytes());
    bytes.extend((-3_i64).to_ne_bytes());
    assert_eq!(ne_i32(&bytes, 1), Some(7));
    assert_eq!(ne_i32(&bytes, 5), Some(-2));
    assert_eq!(ne_u64(&bytes, 9), Some(u64::MAX));
    assert_eq!(ne_i64(&bytes, 17), Some(-3));
    assert_eq!(ne_u32(&bytes, 5), Some(u32::MAX - 1));
}

#[test]
fn a_field_past_the_end_is_none() {
    let bytes = [0_u8; 6];
    assert_eq!(ne_i32(&bytes, 3), None);
    assert_eq!(ne_u64(&bytes, 0), None);
    assert_eq!(ne_i32(&bytes, usize::MAX), None);
}

#[test]
fn hex_spells_bytes() {
    let bytes = [0x00, 0x01, 0xab, 0xff];
    assert_eq!(decode_hex(" 0001ABff\n"), Some(bytes.to_vec()));
    assert_eq!(decode_hex(""), Some(Vec::new()));
}

#[test]
fn bad_hex_is_none() {
    assert_eq!(decode_hex("abc"), None);
    assert_eq!(decode_hex("zz"), None);
    assert_eq!(decode_hex("+1"), None);
}
