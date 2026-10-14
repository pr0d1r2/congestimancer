//! [`ClockWindow`]: a daily window `HH:MM-HH:MM` in local time that wraps
//! midnight when it starts after it ends (src/units:V7).

use std::str::FromStr;

use super::UnitError;

/// The form a clock window takes, named in every error (src/units:V12).
const EXPECTED: &str = "a clock window HH:MM-HH:MM in local time, start \
                        not equal to end, like 09:00-17:00 or 22:00-06:00";

/// A time of day to the minute, in local time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClockTime {
    hour: u8,
    minute: u8,
}

impl ClockTime {
    /// The time `hour:minute`, if `hour` < 24 and `minute` < 60.
    #[must_use]
    pub const fn new(_hour: u8, _minute: u8) -> Option<Self> {
        None
    }

    /// The hour, 0 to 23.
    #[must_use]
    pub const fn hour(self) -> u8 {
        0
    }

    /// The minute, 0 to 59.
    #[must_use]
    pub const fn minute(self) -> u8 {
        0
    }
}

/// A daily window between two times of day, local time.
///
/// The start is inside the window and the end is not. A window that
/// starts after it ends wraps midnight: `22:00-06:00` is the night. A
/// window never starts where it ends (src/units:V7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockWindow {
    start: ClockTime,
    end: ClockTime,
}

impl ClockWindow {
    /// The window from `start` to `end`, unless they are equal.
    #[must_use]
    pub fn new(start: ClockTime, _end: ClockTime) -> Option<Self> {
        None.map(|end| Self { start, end })
    }

    /// Where the window starts, inside it.
    #[must_use]
    pub const fn start(self) -> ClockTime {
        self.start
    }

    /// Where the window ends, outside it.
    #[must_use]
    pub const fn end(self) -> ClockTime {
        self.end
    }

    /// Whether the window wraps midnight: it starts after it ends.
    #[must_use]
    pub fn wraps(self) -> bool {
        false
    }

    /// Whether `t` falls inside the window.
    #[must_use]
    pub fn contains(self, _t: ClockTime) -> bool {
        false
    }
}

/// Parses a clock window such as `09:00-17:00` or `22:00-06:00`.
///
/// # Errors
///
/// [`UnitError`] naming the input and the form expected: not
/// `HH:MM-HH:MM` with two digits each, an hour past 23 or a minute past
/// 59, or a window that starts where it ends.
pub fn parse_clock_window(s: &str) -> Result<ClockWindow, UnitError> {
    Err(UnitError::Malformed {
        input: s.to_owned(),
        expected: EXPECTED,
    })
}

impl FromStr for ClockWindow {
    type Err = UnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_clock_window(s)
    }
}

#[cfg(test)]
mod tests;
