//! Why the gates block: `--verbose` lines and the `--heartbeat` line
//! (src/report:T5, V4).

use std::io::{self, Write};
use std::time::Duration;

use super::{Paint, Style, switch_name};
use crate::gate::{Checked, Evaluation, Op, Subject, Threshold, Verdict};
use crate::signal::{Measure, Trend};
use crate::units::render_duration;

/// One line per gate that did not pass, in gate order: the switch, the
/// compared value, the threshold and the trend (src/report:V4), such as
/// `--load: 3 (needs < 2, rising)`.
#[must_use]
pub fn block_reasons(evaluation: &Evaluation) -> Vec<String> {
    evaluation.verdicts.iter().filter_map(reason).collect()
}

/// Writes each of [`block_reasons`] as a `blocked:` line.
///
/// # Errors
///
/// When `out` fails to take the bytes.
pub fn write_verbose<W: Write + ?Sized>(
    out: &mut W,
    evaluation: &Evaluation,
    style: Style,
) -> io::Result<()> {
    let label = style.paint("blocked:", Paint::Red);
    for reason in block_reasons(evaluation) {
        writeln!(out, "{label} {reason}")?;
    }
    Ok(())
}

/// The `--heartbeat` line after waiting `waited`, no newline:
/// `cgmr: waiting 1m30s: --load: 3 (needs < 2, rising)`.
#[must_use]
pub fn heartbeat_line(evaluation: &Evaluation, waited: Duration) -> String {
    let reasons = block_reasons(evaluation);
    let why = if reasons.is_empty() {
        "all gates pass".to_owned()
    } else {
        reasons.join("; ")
    };
    format!("cgmr: waiting {}: {why}", render_duration(waited))
}

/// Why `checked` blocks, or `None` when it passed.
fn reason(checked: &Checked) -> Option<String> {
    let what = match &checked.verdict {
        Verdict::Pass => return None,
        Verdict::Fail {
            value,
            threshold,
            trend,
        } => failure(&checked.subject, value, threshold, *trend),
        Verdict::Warming => "warming up".to_owned(),
        Verdict::Unsupported { reason } => format!("unsupported: {reason}"),
    };
    Some(format!("{}: {what}", shown_switch(&checked.subject)))
}

/// `3 (needs < 2, rising)`.
fn failure(
    subject: &Subject,
    value: &Measure,
    threshold: &Threshold,
    trend: Option<Trend>,
) -> String {
    let op = needs(subject, threshold);
    let trend = trend.map(|t| format!(", {t}")).unwrap_or_default();
    format!("{value} (needs {op}{threshold}{trend})")
}

/// The switch as typed: `--all-load`, `--when CMD`.
fn shown_switch(subject: &Subject) -> String {
    match subject {
        Subject::Signal(_) => format!("--{}", switch_name(subject)),
        Subject::Custom(custom) => custom.to_string(),
    }
}

/// How the value must stand against the threshold: `< `, `> `, `<= `
/// for a level, nothing for a flag, a window or a custom gate.
fn needs(subject: &Subject, threshold: &Threshold) -> &'static str {
    let Subject::Signal(gate) = subject else {
        return "";
    };
    match (gate.op, threshold) {
        (Op::Below, _) => "< ",
        (Op::Above, _) => "> ",
        (Op::Is, Threshold::Value(Measure::Level(_))) => "<= ",
        (Op::Is, _) => "",
    }
}

#[cfg(test)]
mod tests;
