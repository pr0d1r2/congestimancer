use super::{ClockTime, ClockWindow, parse_clock_window};
use crate::units::UnitError;

fn at(hour: u8, minute: u8) -> ClockTime {
    ClockTime::new(hour, minute)
        .unwrap_or_else(|| unreachable!("{hour}:{minute}"))
}

fn window(s: &str) -> ClockWindow {
    parse_clock_window(s).unwrap_or_else(|e| unreachable!("{e}"))
}

#[test]
fn a_window_reads_start_and_end() {
    let w = window("09:00-17:30");
    assert_eq!(
        (w.start(), w.end(), w.wraps()),
        (at(9, 0), at(17, 30), false)
    );
    let night = window("22:00-06:00");
    assert_eq!((night.start(), night.end()), (at(22, 0), at(6, 0)));
    assert!(night.wraps());
}

#[test]
fn a_day_window_holds_its_start_but_not_its_end() {
    let w = window("09:00-17:00");
    let inside = [at(9, 0), at(12, 0), at(16, 59)];
    let outside = [at(8, 59), at(17, 0), at(23, 0), at(0, 0)];
    assert!(inside.iter().all(|t| w.contains(*t)), "{w:?}");
    assert!(!outside.iter().any(|t| w.contains(*t)), "{w:?}");
}

#[test]
fn a_window_that_starts_after_it_ends_wraps_midnight() {
    let w = window("22:00-06:00");
    let inside = [at(22, 0), at(23, 59), at(0, 0), at(5, 59)];
    let outside = [at(6, 0), at(12, 0), at(21, 59)];
    assert!(inside.iter().all(|t| w.contains(*t)), "{w:?}");
    assert!(!outside.iter().any(|t| w.contains(*t)), "{w:?}");
}

#[test]
fn a_window_never_starts_where_it_ends() {
    for s in ["10:00-10:00", "00:00-00:00"] {
        let e = parse_clock_window(s);
        assert!(matches!(e, Err(UnitError::OutOfRange { .. })), "{s}");
    }
    assert_eq!(ClockWindow::new(at(1, 0), at(1, 0)), None);
}

#[test]
fn hours_past_23_and_minutes_past_59_are_out_of_range() {
    for s in ["24:00-01:00", "10:60-11:00", "01:00-99:99"] {
        let e = parse_clock_window(s);
        assert!(matches!(e, Err(UnitError::OutOfRange { .. })), "{s}");
    }
    assert_eq!(ClockTime::new(24, 0), None);
    assert_eq!(ClockTime::new(0, 60), None);
}

/// Not `HH:MM-HH:MM` with two digits each.
const BAD: [&str; 12] = [
    "",
    "9:00-17:00",
    "09:00",
    "09:00-",
    "0900-1700",
    "09:00 - 17:00",
    "09:00-17:00-18:00",
    "09.00-17.00",
    "+9:00-17:00",
    "09:00-17:00Z",
    "-09:00-17:00",
    "09:0a-17:00",
];

#[test]
fn anything_but_hh_mm_dash_hh_mm_is_malformed() {
    for s in BAD {
        let e = parse_clock_window(s);
        assert!(matches!(e, Err(UnitError::Malformed { .. })), "{s}");
    }
}

#[test]
fn the_error_names_the_input_and_the_form() {
    let Err(e) = parse_clock_window("9-17") else {
        unreachable!("9-17 parsed")
    };
    assert_eq!(e.input(), "9-17");
    assert!(e.expected().contains("HH:MM-HH:MM"), "{}", e.expected());
}

#[test]
fn times_read_back_and_from_str_parses() {
    let t = at(23, 59);
    assert_eq!((t.hour(), t.minute()), (23, 59));
    assert_eq!(
        "22:00-06:00".parse::<ClockWindow>(),
        Ok(window("22:00-06:00"))
    );
}
