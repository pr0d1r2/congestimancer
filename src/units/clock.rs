//! [`ClockWindow`]: a daily window `HH:MM-HH:MM` in local time that wraps
//! midnight when it starts after it ends (src/units:V7).

use std::fmt;
use std::str::FromStr;

use super::UnitError;
use super::number::LexError;

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
    pub const fn new(hour: u8, minute: u8) -> Option<Self> {
        if hour >= 24 || minute >= 60 {
            return None;
        }
        Some(Self { hour, minute })
    }

    /// The hour, 0 to 23.
    #[must_use]
    pub const fn hour(self) -> u8 {
        self.hour
    }

    /// The minute, 0 to 59.
    #[must_use]
    pub const fn minute(self) -> u8 {
        self.minute
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
    pub fn new(start: ClockTime, end: ClockTime) -> Option<Self> {
        (start != end).then_some(Self { start, end })
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
        self.start > self.end
    }

    /// Whether `t` falls inside the window.
    #[must_use]
    pub fn contains(self, t: ClockTime) -> bool {
        let (after_start, before_end) = (t >= self.start, t < self.end);
        if self.wraps() {
            after_start || before_end
        } else {
            after_start && before_end
        }
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
    let (start, end) = s
        .split_once('-')
        .ok_or_else(|| LexError::Malformed.error(s, EXPECTED))?;
    let window = ClockWindow::new(time(start, s)?, time(end, s)?);
    window.ok_or_else(|| out_of_range(s))
}

/// The time in `part` of `input`: `HH:MM`, two digits each.
fn time(part: &str, input: &str) -> Result<ClockTime, UnitError> {
    let (hour, minute) = part
        .split_once(':')
        .and_then(|(h, m)| Some((two_digits(h)?, two_digits(m)?)))
        .ok_or_else(|| LexError::Malformed.error(input, EXPECTED))?;
    ClockTime::new(hour, minute).ok_or_else(|| out_of_range(input))
}

/// Exactly two ASCII digits, as a number.
fn two_digits(s: &str) -> Option<u8> {
    let digits = s.len() == 2 && s.bytes().all(|b| b.is_ascii_digit());
    digits.then(|| s.parse().ok()).flatten()
}

/// The error for a well-formed window that is not one.
fn out_of_range(input: &str) -> UnitError {
    UnitError::OutOfRange {
        input: input.to_owned(),
        expected: EXPECTED,
    }
}

impl FromStr for ClockWindow {
    type Err = UnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_clock_window(s)
    }
}

/// Renders as `HH:MM`.
impl fmt::Display for ClockTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}

/// Renders as `HH:MM-HH:MM`.
impl fmt::Display for ClockWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

#[cfg(test)]
mod tests;
