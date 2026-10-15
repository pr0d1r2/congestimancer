use std::path::Path;

use super::{LinkCounters, decode_iflist2, interface_counters, messages};
use crate::sys::raw::decode_hex;

/// `NET_RT_IFLIST2` for lo0 and en0 and one multicast message, captured
/// on an M4 macOS 26 host with link-layer addresses zeroed.
fn captured() -> Vec<u8> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir.join("tests/fixtures/sys/macos/net/NET_RT_IFLIST2");
    let text = std::fs::read_to_string(path).unwrap_or_default();
    decode_hex(&text).unwrap_or_default()
}

#[test]
fn messages_split_on_their_own_lengths() {
    let lengths: Vec<usize> =
        messages(&captured()).iter().map(|m| m.len()).collect();
    assert_eq!(lengths, [180, 180, 76]);
}

#[test]
fn a_truncated_or_zero_length_message_ends_the_walk() {
    let mut raw = captured();
    raw.truncate(200);
    assert_eq!(messages(&raw).len(), 1);
    assert!(messages(&[0, 0, 5, 0x12]).is_empty());
    assert!(messages(&[1]).is_empty());
}

#[test]
fn interfaces_decode_with_names_and_flags() {
    let all = decode_iflist2(&captured());
    let names: Vec<(&str, bool, bool)> = all
        .iter()
        .map(|i| (i.name.as_str(), i.up, i.loopback))
        .collect();
    assert_eq!(names, [("lo0", true, true), ("en0", true, false)]);
}

#[test]
fn counters_are_64_bit() {
    let all = decode_iflist2(&captured());
    let en0 = all.get(1);
    let rx = LinkCounters {
        bytes: 2_068_043_776,
        packets: 1_834_585,
        errors: 0,
    };
    assert_eq!(en0.map(|i| i.rx), Some(rx));
    assert_eq!(
        en0.map(|i| (i.tx.bytes, i.tx.packets)),
        Some((454_156_288, 936_696))
    );
}

#[test]
fn an_interface_without_its_link_address_is_skipped() {
    let mut raw = captured();
    raw.truncate(180);
    if let Some(addrs) = raw.get_mut(4) {
        *addrs = 0;
    }
    assert!(decode_iflist2(&raw).is_empty());
}

#[cfg(target_os = "macos")]
#[test]
fn header_offsets_match_the_sdk_struct() {
    use std::mem::{offset_of, size_of};

    use super::at;

    assert_eq!(offset_of!(libc::if_msghdr2, ifm_addrs), at::ADDRS);
    assert_eq!(offset_of!(libc::if_msghdr2, ifm_flags), at::FLAGS);
    assert_eq!(offset_of!(libc::if_msghdr2, ifm_data), at::DATA);
    assert_eq!(size_of::<libc::if_msghdr2>(), at::SDL);
}

#[cfg(target_os = "macos")]
#[test]
fn counter_offsets_match_the_sdk_struct() {
    use std::mem::offset_of;

    use super::at;

    let sdk = [
        offset_of!(libc::if_data64, ifi_ipackets),
        offset_of!(libc::if_data64, ifi_ierrors),
        offset_of!(libc::if_data64, ifi_opackets),
        offset_of!(libc::if_data64, ifi_oerrors),
        offset_of!(libc::if_data64, ifi_ibytes),
        offset_of!(libc::if_data64, ifi_obytes),
    ];
    let ours = [at::IPACKETS, at::IERRORS, at::OPACKETS, at::OERRORS]
        .into_iter()
        .chain([at::IBYTES, at::OBYTES])
        .map(|off| off.saturating_sub(at::DATA));
    assert!(ours.eq(sdk));
}

#[cfg(not(target_os = "macos"))]
#[test]
fn other_oses_are_unsupported() {
    let err = interface_counters().err().map(|e| e.kind());
    assert_eq!(err, Some(std::io::ErrorKind::Unsupported));
}

#[cfg(target_os = "macos")]
#[test]
fn host_loopback_is_listed() {
    let all = interface_counters().unwrap_or_default();
    assert!(all.iter().any(|i| i.name == "lo0" && i.loopback));
}
