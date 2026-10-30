//! Switch values read by the same parsers wherever they come from: argv,
//! the environment or a profile (src/cli:V10, src/profile:V5).

use std::fmt::Display;

use super::learn::{AUTO_DEFAULT, Need, parse_need};
use super::table::Value;
use crate::cgroup::parse_cgroup_mode;
use crate::cpu::parse_load_window;
use crate::io::parse_io_device;
use crate::net::parse_net_iface;
use crate::procs::parse_proc_limit;
use crate::remote::parse_host;
use crate::signal::{Level, Measure};
use crate::slot::parse_start_rate;
use crate::units::{
    Bytes, parse_bytes, parse_celsius, parse_clock_window, parse_count,
    parse_decimal, parse_duration, parse_name, parse_percent, parse_rate,
};

/// What `--mem-level` takes.
const LEVELS: &str = "--mem-level takes normal or warning";

/// `--mem-level`: `normal` or `warning` (src/mem:V5).
///
/// # Errors
///
/// The words the switch takes, for anything else.
pub(crate) fn parse_level(s: &str) -> Result<Level, String> {
    match s {
        "normal" => Ok(Level::Normal),
        "warning" => Ok(Level::Warning),
        _ => Err(format!("{s:?}: {LEVELS}")),
    }
}

/// What a `--needs-mem` value stands for where nothing has learned it:
/// the size, or an `auto` fallback (one GiB without one).
fn need_bytes(s: &str) -> Result<Bytes, String> {
    Ok(match parse_need(s)? {
        Need::Fixed(bytes) => bytes,
        Need::Auto(fallback) => fallback.unwrap_or(AUTO_DEFAULT),
    })
}

/// `Ok(())`, or the parser's error in words.
fn words<T, E: Display>(parsed: Result<T, E>) -> Result<(), String> {
    parsed.map(drop).map_err(|e| e.to_string())
}

/// `parsed` as a measure through `wrap`, or its error in words.
fn to<T, E: Display>(
    parsed: Result<T, E>,
    wrap: fn(T) -> Measure,
) -> Result<Measure, String> {
    parsed.map(wrap).map_err(|e| e.to_string())
}

/// Some text, or why none is no value.
fn text(s: &str) -> Result<(), String> {
    if s.trim().is_empty() {
        Err("needs a value".to_owned())
    } else {
        Ok(())
    }
}

impl Value {
    /// Whether `s` is a value of this kind; why not, in words.
    ///
    /// # Errors
    ///
    /// The parser's own message, naming the input and the form expected.
    pub(crate) fn check(self, s: &str) -> Result<(), String> {
        if let Some(measure) = self.measure(s) {
            return measure.map(drop);
        }
        match self {
            Self::Window => words(parse_clock_window(s)),
            Self::LoadWindow => words(parse_load_window(s)),
            Self::Device => words(parse_io_device(s)),
            Self::Iface => words(parse_net_iface(s)),
            Self::Host => words(parse_host(s)),
            Self::Cgroup => words(parse_cgroup_mode(s)),
            Self::ProcLimit => words(parse_proc_limit(s)),
            Self::StartRate => words(parse_start_rate(s)),
            Self::Name => words(parse_name(s)),
            _ => text(s),
        }
    }

    /// `s` as the measure a gate compares, for a value that is one;
    /// `None` for a value that is no measure (a path, a window).
    pub(crate) fn measure(self, s: &str) -> Option<Result<Measure, String>> {
        let m = match self {
            Self::Decimal => to(parse_decimal(s), Measure::Number),
            Self::Percent => to(parse_percent(s), Measure::Percent),
            Self::Bytes => to(parse_bytes(s), Measure::Bytes),
            Self::MemNeed => to(need_bytes(s), Measure::Bytes),
            Self::Rate => to(parse_rate(s), Measure::Rate),
            Self::Duration => to(parse_duration(s), Measure::Duration),
            Self::Celsius => to(parse_celsius(s), Measure::Celsius),
            Self::Count => to(parse_count(s), Measure::Count),
            Self::Level => to(parse_level(s), Measure::Level),
            _ => return None,
        };
        Some(m)
    }
}

#[cfg(test)]
mod tests;
