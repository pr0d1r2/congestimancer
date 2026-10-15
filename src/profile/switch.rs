//! [`Switch`]: one line of a profile, a switch exactly as on the command
//! line, and the [`SwitchTable`] that checks it (src/profile:V4, V5).

use super::{At, ProfileError};

/// How often a switch may be given.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arity {
    /// Once: a second time in one profile is an error, and the command
    /// line or environment overrides it (src/profile:V6).
    Single,
    /// Any number of times, such as a gate on several paths.
    Repeatable,
}

/// Why the CLI refuses a switch.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwitchFault {
    /// The CLI has no such switch.
    Unknown,
    /// The value does not parse; the text says why, as on the command line.
    BadValue(String),
}

/// The CLI's own switch parser, lent to profiles so a switch in a file is
/// held to exactly the rules it meets on the command line (src/profile:V5).
///
/// `src/profile` cannot see the CLI (it is built first); `src/cli`
/// implements this with the same `clap` definitions and `src/units`
/// parsers it uses for argv.
pub trait SwitchTable {
    /// Checks `flag` (as written, such as `--load`) with its value.
    ///
    /// # Errors
    ///
    /// [`SwitchFault::Unknown`] for a switch the CLI lacks,
    /// [`SwitchFault::BadValue`] for a value it refuses.
    fn check(
        &self,
        flag: &str,
        value: Option<&str>,
    ) -> Result<Arity, SwitchFault>;
}

/// One switch from a profile, with the line it came from.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Switch {
    flag: String,
    value: Option<String>,
    at: At,
}

impl Switch {
    /// The switch as written, such as `--load`.
    #[must_use]
    pub fn flag(&self) -> &str {
        &self.flag
    }

    /// Its value, if it has one: `2` for `--load 2`.
    #[must_use]
    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }

    /// The line it came from.
    #[must_use]
    pub const fn at(&self) -> &At {
        &self.at
    }

    /// The switch as argv words: `["--load", "2"]`, or `["--quiet"]`.
    #[must_use]
    pub fn args(&self) -> Vec<&str> {
        let mut args = vec![self.flag.as_str()];
        args.extend(self.value());
        args
    }

    /// Checks the switch against the CLI's own parser (src/profile:V5).
    ///
    /// # Errors
    ///
    /// [`ProfileError::UnknownSwitch`] or [`ProfileError::BadValue`], both
    /// naming `file:line` (src/profile:V4).
    pub fn check(
        &self,
        table: &impl SwitchTable,
    ) -> Result<Arity, ProfileError> {
        table.check(&self.flag, self.value()).map_err(|fault| {
            let (at, flag) = (self.at.clone(), self.flag.clone());
            match fault {
                SwitchFault::Unknown => {
                    ProfileError::UnknownSwitch { at, flag }
                }
                SwitchFault::BadValue(reason) => {
                    ProfileError::BadValue { at, flag, reason }
                }
            }
        })
    }
}

/// Why a line is not a switch.
pub(super) const NOT_A_SWITCH: &str = "not a switch: a line is a [name] \
                                       header or one switch as on the \
                                       command line, like --load 2";

/// Parses a trimmed line as a switch, or says why it is not one.
///
/// The switch ends at the first space or `=`; the rest, trimmed, is its
/// value, with one pair of matching quotes dropped.
pub(super) fn parse_switch(line: &str, at: At) -> Result<Switch, &'static str> {
    let (flag, rest) = line
        .split_once(|c: char| c == '=' || c.is_whitespace())
        .unwrap_or((line, ""));
    if !flag.starts_with('-') || flag.trim_start_matches('-').is_empty() {
        return Err(NOT_A_SWITCH);
    }
    let rest = rest.trim();
    Ok(Switch {
        flag: flag.to_owned(),
        value: (!rest.is_empty()).then(|| unquote(rest).to_owned()),
        at,
    })
}

/// `value` without one pair of matching surrounding quotes, if it has one.
fn unquote(value: &str) -> &str {
    let strip = |q: char| value.strip_prefix(q)?.strip_suffix(q);
    strip('"').or_else(|| strip('\'')).unwrap_or(value)
}

/// The switch table the profile tests use: a small CLI with the real
/// `src/units` parsers behind it (src/profile:V5).
///
/// `--load` (decimal), `--mem` (size) and `--when` (a command) take one
/// value, `--path` repeats, and `--quiet` takes none.
#[cfg(test)]
pub(crate) struct TestTable;

#[cfg(test)]
impl SwitchTable for TestTable {
    fn check(
        &self,
        flag: &str,
        value: Option<&str>,
    ) -> Result<Arity, SwitchFault> {
        let refuse = |why: &str| Err(SwitchFault::BadValue(why.to_owned()));
        match (flag, value) {
            ("--quiet", None) => Ok(Arity::Single),
            ("--quiet", Some(_)) => refuse("takes no value"),
            ("--load" | "--mem" | "--when" | "--path", None) => {
                refuse("a value is required")
            }
            (_, Some(v)) => valued(flag, v),
            (_, None) => Err(SwitchFault::Unknown),
        }
    }
}

/// [`TestTable`] on a switch given a value.
#[cfg(test)]
fn valued(flag: &str, value: &str) -> Result<Arity, SwitchFault> {
    use crate::units::{UnitError, parse_bytes, parse_decimal};
    let unit = |r: Result<(), UnitError>| {
        r.map(|()| Arity::Single)
            .map_err(|e| SwitchFault::BadValue(e.to_string()))
    };
    match flag {
        "--load" => unit(parse_decimal(value).map(drop)),
        "--mem" => unit(parse_bytes(value).map(drop)),
        "--when" => Ok(Arity::Single),
        "--path" => Ok(Arity::Repeatable),
        _ => Err(SwitchFault::Unknown),
    }
}

#[cfg(test)]
mod tests;
