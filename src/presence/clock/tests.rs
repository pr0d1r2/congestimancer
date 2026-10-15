use super::{SystemWallClock, WallClock};

#[test]
fn the_system_wall_clock_is_past_2020() {
    // 2020-01-01T00:00:00Z in microseconds: a shape, not the machine's
    // value.
    assert!(SystemWallClock.unix_micros() > 1_577_836_800_000_000);
}
