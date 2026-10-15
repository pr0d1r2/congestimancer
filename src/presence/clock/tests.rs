use std::io;

use super::{SystemWallClock, WallClock, clock_time};
use crate::units::ClockTime;

#[test]
fn the_system_wall_clock_is_past_2020() {
    // 2020-01-01T00:00:00Z in microseconds: a shape, not the machine's
    // value.
    assert!(SystemWallClock.unix_micros() > 1_577_836_800_000_000);
}

#[test]
fn the_system_local_time_is_a_time_of_day() {
    // A shape, not the machine's value: src/sys reads it with the zone and
    // DST applied (src/presence:V4).
    let now = SystemWallClock.local_time();
    assert!(now.is_ok(), "{now:?}");
}

#[test]
fn an_hour_and_minute_become_a_clock_time() {
    assert_eq!(clock_time(Ok((22, 5))).ok(), ClockTime::new(22, 5));
}

#[test]
fn an_unreadable_or_impossible_local_time_is_the_reason() {
    let failed = clock_time(Err(io::Error::other("no zone")));
    assert!(failed.is_err_and(|r| r.to_string().contains("no zone")));
    let impossible = clock_time(Ok((24, 0)));
    assert!(impossible.is_err_and(|r| r.to_string().contains("24:00")));
}
