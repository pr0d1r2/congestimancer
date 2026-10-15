//! [`write_json`]: `status --json` and `check --json`, schema v1, written
//! by hand with no serializer (src/report:T1, V1, V6).

use std::io::{self, Write};

use super::StatusReport;

/// The JSON schema version; a breaking change bumps it (src/report:V1).
pub const SCHEMA_VERSION: u32 = 1;

/// Writes `report` as one line of JSON schema v1, then a newline.
///
/// # Errors
///
/// When `out` fails to take the bytes.
pub fn write_json<W: Write + ?Sized>(
    out: &mut W,
    report: &StatusReport,
) -> io::Result<()> {
    let _ = (out, report);
    Ok(())
}

/// `text` as a JSON string, quotes included.
#[must_use]
pub fn json_string(text: &str) -> String {
    let _ = text;
    String::new()
}

#[cfg(test)]
mod tests;
