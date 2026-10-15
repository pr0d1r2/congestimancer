//! [`write_advice`] and [`write_advice_json`]: the advice as one line of
//! text or one line of JSON (src/advise:T4, V7).

use std::io::{self, Write};

use super::Advice;
use crate::gate::{Checked, Subject, Verdict};
use crate::report::{SCHEMA_VERSION, json_string, switch_name};

/// JSON's empty value.
const NULL: &str = "null";

/// Writes `advice` as one line: `jobs=3 limit=mem action=reduce`.
///
/// # Errors
///
/// When `out` fails to take the bytes.
pub fn write_advice<W: Write + ?Sized>(
    out: &mut W,
    advice: &Advice,
) -> io::Result<()> {
    writeln!(
        out,
        "jobs={} limit={} action={}",
        advice.jobs, advice.limit, advice.action
    )
}

/// Writes `advice` as one line of JSON sharing `status`'s
/// `schema_version`: `jobs`, `limit`, `action`, and `blocking`, each
/// gate that did not pass with its `switch`, its `value` (`null` while
/// warming) and `threshold` as their switches take them, and its
/// `verdict`.
///
/// # Errors
///
/// When `out` fails to take the bytes.
pub fn write_advice_json<W: Write + ?Sized>(
    out: &mut W,
    advice: &Advice,
) -> io::Result<()> {
    let limit = json_string(&advice.limit.to_string());
    let action = json_string(&advice.action.to_string());
    write!(
        out,
        "{{\"schema_version\":{SCHEMA_VERSION},\"jobs\":{},\
         \"limit\":{limit},\"action\":{action},\"blocking\":[",
        advice.jobs
    )?;
    for (n, checked) in advice.blocking.iter().enumerate() {
        if n > 0 {
            out.write_all(b",")?;
        }
        write_blocking(out, checked)?;
    }
    out.write_all(b"]}\n")
}

/// One entry of `blocking`.
fn write_blocking<W: Write + ?Sized>(
    out: &mut W,
    checked: &Checked,
) -> io::Result<()> {
    let switch = json_string(&switch_name(&checked.subject));
    let (value, verdict) = value_and_verdict(&checked.verdict);
    let threshold = match &checked.subject {
        Subject::Signal(gate) => json_string(&gate.threshold.to_string()),
        Subject::Custom(_) => NULL.to_owned(),
    };
    write!(
        out,
        "{{\"switch\":{switch},\"value\":{value},\
         \"threshold\":{threshold},\"verdict\":{verdict}}}"
    )
}

/// A verdict's value (`null` unless it failed on one) and its one word,
/// both as JSON.
fn value_and_verdict(verdict: &Verdict) -> (String, String) {
    let (value, word) = match verdict {
        Verdict::Fail { value, .. } => {
            (json_string(&value.to_string()), "fail")
        }
        Verdict::Pass => (NULL.to_owned(), "pass"),
        Verdict::Warming => (NULL.to_owned(), "warming"),
        Verdict::Unsupported { .. } => (NULL.to_owned(), "unsupported"),
    };
    (value, json_string(word))
}

#[cfg(test)]
mod tests;
