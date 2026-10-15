use std::io;
use std::path::Path;

use super::{
    SwapUsage, decode_int, decode_swap_usage, sized, swap_usage, sysctl_int,
};
use crate::sys::raw::decode_hex;

/// The raw bytes of sysctl `name` captured on an M4 macOS 26 host.
fn captured(name: &str) -> Vec<u8> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir.join("tests/fixtures/sys/macos/sysctl").join(name);
    let text = std::fs::read_to_string(path).unwrap_or_default();
    decode_hex(&text).unwrap_or_default()
}

#[test]
fn four_byte_values_decode() {
    assert_eq!(decode_int(&captured("kern.maxfiles")), Some(122_880));
    assert_eq!(decode_int(&captured("kern.num_files")), Some(4974));
    assert_eq!(decode_int(&captured("kern.memorystatus_level")), Some(45));
    assert_eq!(
        decode_int(&captured("kern.memorystatus_vm_pressure_level")),
        Some(1)
    );
}

#[test]
fn eight_byte_values_decode() {
    assert_eq!(decode_int(&captured("hw.memsize")), Some(17_179_869_184));
}

#[test]
fn other_widths_are_not_integers() {
    assert_eq!(decode_int(&[1, 2, 3]), None);
    assert_eq!(decode_int(&[]), None);
}

#[test]
fn swap_usage_decodes_xsw_usage() {
    let swap = decode_swap_usage(&captured("vm.swapusage"));
    assert_eq!(
        swap,
        Some(SwapUsage {
            total: 5_368_709_120,
            available: 1_257_308_160,
            used: 4_111_400_960,
        })
    );
}

#[test]
fn short_swap_usage_is_none() {
    assert_eq!(decode_swap_usage(&[0; 23]), None);
}

#[test]
fn an_unknown_sysctl_is_unsupported() {
    let err = sysctl_int("cgmr.no.such.sysctl").err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::Unsupported));
}

#[cfg(target_os = "macos")]
#[test]
fn a_name_with_a_nul_is_invalid_input() {
    let err = sysctl_int("kern.\0maxfiles").err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::InvalidInput));
}

#[cfg(target_os = "macos")]
#[test]
fn host_sysctls_read() {
    assert!(sysctl_int("kern.maxfiles").is_ok_and(|n| n > 0));
    assert!(swap_usage().is_ok());
}

/// A fake sysctl that reports `sizes` in turn (one per call) and fails
/// with `errno` on the calls `fail_on` names.
fn fake(
    sizes: &[usize],
    fail_on: &[usize],
    errno: i32,
) -> impl FnMut(*mut libc::c_void, &mut usize) -> io::Result<()> {
    let sizes = sizes.to_vec();
    let fail_on = fail_on.to_vec();
    let mut call = 0_usize;
    move |_, len| {
        let this = call;
        call = call.saturating_add(1);
        *len = sizes.get(this).copied().unwrap_or(0);
        if fail_on.contains(&this) {
            Err(io::Error::from_raw_os_error(errno))
        } else {
            Ok(())
        }
    }
}

#[test]
fn sized_asks_for_the_size_then_the_value() {
    assert_eq!(sized(fake(&[4, 4], &[], 0)).ok(), Some(vec![0; 4]));
}

#[test]
fn sized_keeps_only_the_bytes_written() {
    assert_eq!(sized(fake(&[4, 2], &[], 0)).ok(), Some(vec![0; 2]));
}

#[test]
fn sized_asks_again_when_the_value_grew() {
    let got = sized(fake(&[2, 0, 3, 3], &[1], libc::ENOMEM));
    assert_eq!(got.ok(), Some(vec![0; 3]));
}

#[test]
fn sized_gives_up_on_a_value_that_keeps_growing() {
    let got = sized(fake(&[1; 20], &[1, 3, 5, 7, 9, 11], libc::ENOMEM));
    assert_eq!(got.err().and_then(|e| e.raw_os_error()), Some(libc::ENOMEM));
}

#[test]
fn sized_passes_other_errors_through() {
    let got = sized(fake(&[1], &[0], libc::EPERM));
    assert_eq!(got.err().and_then(|e| e.raw_os_error()), Some(libc::EPERM));
}
