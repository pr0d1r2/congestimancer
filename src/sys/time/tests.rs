use std::io;
use std::ptr;

use super::{local_time, split};

/// A fixed date, 2026-10-15 at midnight in a zone two hours east;
/// `tm_zone` is `*mut` on macOS and `*const` on glibc.
const DAY: libc::tm = libc::tm {
    tm_sec: 0,
    tm_min: 0,
    tm_hour: 0,
    tm_mday: 15,
    tm_mon: 9,
    tm_year: 126,
    tm_wday: 4,
    tm_yday: 287,
    tm_isdst: 1,
    tm_gmtoff: 7200,
    #[cfg(target_os = "macos")]
    tm_zone: ptr::null_mut(),
    #[cfg(not(target_os = "macos"))]
    tm_zone: ptr::null(),
};

/// [`DAY`] at `hour:minute:second`.
const fn tm_at(hour: i32, minute: i32, second: i32) -> libc::tm {
    libc::tm {
        tm_sec: second,
        tm_min: minute,
        tm_hour: hour,
        ..DAY
    }
}

#[test]
fn split_takes_the_hour_and_minute() {
    assert_eq!(split(&tm_at(9, 5, 30)).ok(), Some((9, 5)));
    assert_eq!(split(&tm_at(0, 0, 0)).ok(), Some((0, 0)));
    assert_eq!(split(&tm_at(23, 59, 60)).ok(), Some((23, 59)));
}

#[test]
fn split_refuses_a_time_out_of_range() {
    for (hour, minute) in [(24, 0), (-1, 0), (12, 60), (12, -1)] {
        let kind = split(&tm_at(hour, minute, 0)).err().map(|e| e.kind());
        assert_eq!(kind, Some(io::ErrorKind::InvalidData), "{hour}:{minute}");
    }
}

#[test]
fn local_time_is_a_time_of_day() {
    // A shape, not the machine's value (src/sys:V3).
    let read = local_time();
    assert!(
        read.as_ref().is_ok_and(|&(h, m)| h < 24 && m < 60),
        "{read:?}"
    );
}
