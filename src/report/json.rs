//! [`write_json`]: `status --json` and `check --json`, schema v1, written
//! by hand with no serializer (src/report:T1, V1, V6).

use std::io::{self, Write};

use super::{StatusReport, StatusRow, switch_name};
use crate::gate::{Checked, Custom, Subject, Verdict};
use crate::signal::{Measure, Reading};
use crate::units::{Bytes, Count, Rate};

/// The JSON schema version; a breaking change bumps it (src/report:V1).
pub const SCHEMA_VERSION: u32 = 1;

/// JSON's empty value.
const NULL: &str = "null";

/// Writes `report` as one line of JSON schema v1, then a newline.
///
/// # Errors
///
/// When `out` fails to take the bytes.
pub fn write_json<W: Write + ?Sized>(
    out: &mut W,
    report: &StatusReport,
) -> io::Result<()> {
    let host = json_string(&report.host);
    write!(out, "{{\"schema_version\":{SCHEMA_VERSION},\"host\":{host}")?;
    out.write_all(b",\"signals\":[")?;
    write_list(out, &report.signals, write_signal)?;
    out.write_all(b"],\"gates\":[")?;
    write_list(out, &report.gates, write_gate)?;
    out.write_all(b"]}\n")
}

/// Writes `items` one by one, separated by commas.
fn write_list<W: Write + ?Sized, T>(
    out: &mut W,
    items: &[T],
    write_one: fn(&mut W, &T) -> io::Result<()>,
) -> io::Result<()> {
    for (n, item) in items.iter().enumerate() {
        if n > 0 {
            out.write_all(b",")?;
        }
        write_one(out, item)?;
    }
    Ok(())
}

/// One entry of `signals`.
fn write_signal<W: Write + ?Sized>(
    out: &mut W,
    row: &StatusRow,
) -> io::Result<()> {
    let (value, unit) = row.reading.measure().map_or_else(
        || (NULL.to_owned(), NULL.to_owned()),
        |&m| (json_value(m), json_string(unit(m))),
    );
    let id = json_string(row.id.name());
    let trend = row
        .trend
        .map_or_else(|| NULL.to_owned(), |t| json_string(&t.to_string()));
    let supported = !matches!(row.reading, Reading::Unsupported { .. });
    let source = json_string(&row.source);
    write!(
        out,
        "{{\"id\":{id},\"value\":{value},\"unit\":{unit},\"trend\":{trend},\
         \"supported\":{supported},\"source\":{source}}}"
    )
}

/// One entry of `gates`.
fn write_gate<W: Write + ?Sized>(
    out: &mut W,
    checked: &Checked,
) -> io::Result<()> {
    let switch = json_string(&switch_name(&checked.subject));
    let threshold = json_string(&threshold_text(&checked.subject));
    let verdict = json_string(verdict_name(&checked.verdict));
    write!(
        out,
        "{{\"switch\":{switch},\"threshold\":{threshold},\
         \"verdict\":{verdict}}}"
    )
}

/// What a gate compares with, as its switch takes it: `2`, `1GiB`,
/// `22:00-06:00`, a `--when` command, a `--when-file` path.
fn threshold_text(subject: &Subject) -> String {
    match subject {
        Subject::Signal(gate) => gate.threshold.to_string(),
        Subject::Custom(Custom::When(cmd)) => cmd.clone(),
        Subject::Custom(Custom::WhenFile(path)) => path.display().to_string(),
    }
}

/// A verdict's one word.
const fn verdict_name(verdict: &Verdict) -> &'static str {
    match verdict {
        Verdict::Pass => "pass",
        Verdict::Fail { .. } => "fail",
        Verdict::Warming => "warming",
        Verdict::Unsupported { .. } => "unsupported",
    }
}

/// A measure's unit name.
const fn unit(measure: Measure) -> &'static str {
    match measure {
        Measure::Number(_) => "number",
        Measure::Count(_) => "count",
        Measure::Percent(_) => "percent",
        Measure::Bytes(_) => "bytes",
        Measure::Rate(_) => "bytes/s",
        Measure::Duration(_) => "seconds",
        Measure::Celsius(_) => "celsius",
        Measure::Flag(_) => "flag",
        Measure::Level(_) => "level",
        Measure::Clock(_) => "clock",
    }
}

/// A measure's value: whole quantities as integers, fractions with a `.`
/// (src/report:V6), flags as booleans, levels and times as strings.
fn json_value(measure: Measure) -> String {
    match measure {
        Measure::Count(Count(n))
        | Measure::Bytes(Bytes(n))
        | Measure::Rate(Rate(n)) => n.to_string(),
        Measure::Number(_)
        | Measure::Percent(_)
        | Measure::Duration(_)
        | Measure::Celsius(_) => json_number(measure.as_f64()),
        Measure::Flag(on) => on.to_string(),
        Measure::Level(_) | Measure::Clock(_) => {
            json_string(&measure.to_string())
        }
    }
}

/// A float as JSON: Rust renders it with a `.` and no exponent, whatever
/// the locale; a value JSON cannot hold is `null`.
fn json_number(value: f64) -> String {
    if value.is_finite() {
        value.to_string()
    } else {
        NULL.to_owned()
    }
}

/// `text` as a JSON string, quotes included: `"` and `\` escaped, control
/// characters as `\n`-style or `\u00XX` escapes, all else as UTF-8.
#[must_use]
pub fn json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len().saturating_add(2));
    out.push('"');
    for c in text.chars() {
        push_escaped(&mut out, c);
    }
    out.push('"');
    out
}

/// Pushes `c` as it stands inside a JSON string.
fn push_escaped(out: &mut String, c: char) {
    match c {
        '"' => out.push_str("\\\""),
        '\\' => out.push_str("\\\\"),
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\t' => out.push_str("\\t"),
        '\u{8}' => out.push_str("\\b"),
        '\u{c}' => out.push_str("\\f"),
        c if c < ' ' => out.push_str(&format!("\\u{:04x}", u32::from(c))),
        c => out.push(c),
    }
}

#[cfg(test)]
mod tests;
