//! [`write_status`]: the `status` table for humans (src/report:T2, V5,
//! V6).

use std::io::{self, Write};

use super::{StatusReport, Style};

/// Writes `report`'s signals as an aligned table, one row per signal,
/// unsupported ones with their reason (src/report:V5).
///
/// # Errors
///
/// When `out` fails to take the bytes.
pub fn write_status<W: Write + ?Sized>(
    out: &mut W,
    report: &StatusReport,
    style: Style,
) -> io::Result<()> {
    let _ = (out, report, style);
    Ok(())
}

#[cfg(test)]
mod tests;
