use super::{Bytes, parse_bytes};
use crate::units::UnitError;

const K: u64 = 1000;
const KI: u64 = 1024;

fn bytes(s: &str) -> Result<u64, UnitError> {
    parse_bytes(s).map(|b| b.0)
}

#[test]
fn decimal_units_are_powers_of_1000() {
    assert_eq!(bytes("1B"), Ok(1));
    assert_eq!(bytes("1KB"), Ok(K));
    assert_eq!(bytes("1MB"), Ok(K.pow(2)));
    assert_eq!(bytes("1GB"), Ok(K.pow(3)));
    assert_eq!(bytes("1TB"), Ok(K.pow(4)));
}

#[test]
fn binary_units_are_powers_of_1024() {
    assert_eq!(bytes("1KiB"), Ok(KI));
    assert_eq!(bytes("1MiB"), Ok(KI.pow(2)));
    assert_eq!(bytes("1GiB"), Ok(KI.pow(3)));
    assert_eq!(bytes("1TiB"), Ok(KI.pow(4)));
}

#[test]
fn units_are_case_insensitive_and_kb_is_not_kib() {
    assert_eq!(bytes("10kb"), Ok(10 * K));
    assert_eq!(bytes("10kib"), Ok(10 * KI));
    assert_eq!(bytes("2gB"), Ok(2 * K.pow(3)));
    assert_eq!(bytes("2GIB"), Ok(2 * KI.pow(3)));
    assert_eq!(bytes("3b"), Ok(3));
}

#[test]
fn fractions_round_down_to_whole_bytes() {
    assert_eq!(bytes("1.5GB"), Ok(1_500_000_000));
    assert_eq!(bytes("1.5KiB"), Ok(1536));
    assert_eq!(bytes("1.0009KB"), Ok(1000));
    assert_eq!(bytes("0.9B"), Ok(0));
}

#[test]
fn bit_units_and_unknown_units_are_malformed() {
    let bad = [
        "1Kbit", "1Mbps", "8bit", "10PB", "10EiB", "10K", "10k", "10GBs",
        "10iB", "10G", "10 GB", " 10GB", "10GB ",
    ];
    for s in bad {
        assert!(
            matches!(parse_bytes(s), Err(UnitError::Malformed { .. })),
            "{s}"
        );
    }
}

#[test]
fn a_size_needs_a_number_and_a_unit() {
    for s in ["", "GB", "10", "1.5", ".5GB", "1,5GB", "1e3B", "+1GB"] {
        assert!(
            matches!(parse_bytes(s), Err(UnitError::Malformed { .. })),
            "{s}"
        );
    }
}

#[test]
fn a_negative_size_is_negative() {
    let e = parse_bytes("-1GB");
    assert!(matches!(e, Err(UnitError::Negative { .. })), "{e:?}");
}

#[test]
fn past_u64_bytes_is_overflow() {
    assert_eq!(bytes("18446744073709551615B"), Ok(u64::MAX));
    for s in ["18446744073709551616B", "17179869184GiB", "18446745TB"] {
        assert!(
            matches!(parse_bytes(s), Err(UnitError::Overflow { .. })),
            "{s}"
        );
    }
}

#[test]
fn the_error_names_the_input_and_the_form() {
    let Err(e) = parse_bytes("10XB") else {
        unreachable!("10XB parsed")
    };
    assert_eq!(e.input(), "10XB");
    assert!(e.expected().contains("like 10GB"), "{}", e.expected());
}

#[test]
fn from_str_is_parse_bytes() {
    assert_eq!("4KiB".parse::<Bytes>(), Ok(Bytes(4096)));
}
