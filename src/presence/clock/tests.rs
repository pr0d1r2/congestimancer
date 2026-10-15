use super::{SystemWallClock, WallClock};

#[test]
fn the_system_wall_clock_is_past_2020() {
    // 2020-01-01T00:00:00Z in microseconds: a shape, not the machine's
    // value.
    assert!(SystemWallClock.unix_micros() > 1_577_836_800_000_000);
}

#[test]
fn the_system_local_time_is_not_read_yet() {
    // Needs a src/sys local time call (localtime_r); until then the
    // reason says so instead of guessing a zone.
    assert!(SystemWallClock.local_time().is_err());
}
