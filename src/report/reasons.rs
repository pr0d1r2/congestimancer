//! Why the gates block: `--verbose` lines and the `--heartbeat` line
//! (src/report:T5, V4).

use std::io::{self, Write};
use std::time::Duration;

use super::Style;
use crate::gate::Evaluation;

/// One line per gate that did not pass, in gate order: the switch, the
/// compared value, the threshold and the trend (src/report:V4), such as
/// `--load: 3 (needs < 2, rising)`.
#[must_use]
pub fn block_reasons(evaluation: &Evaluation) -> Vec<String> {
    let _ = evaluation;
    Vec::new()
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
    let _ = (out, evaluation, style);
    Ok(())
}

/// The `--heartbeat` line after waiting `waited`, no newline:
/// `cgmr: waiting 1m30s: --load: 3 (needs < 2, rising)`.
#[must_use]
pub fn heartbeat_line(evaluation: &Evaluation, waited: Duration) -> String {
    let _ = (evaluation, waited);
    String::new()
}

#[cfg(test)]
mod tests;
