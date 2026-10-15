//! [`write_advice`] and [`write_advice_json`]: the advice as one line of
//! text or one line of JSON (src/advise:T4, V7).

use std::io::{self, Write};

use super::Advice;

/// Writes `advice` as one line: `jobs=3 limit=mem action=reduce`.
///
/// # Errors
///
/// When `out` fails to take the bytes.
pub fn write_advice<W: Write + ?Sized>(
    out: &mut W,
    advice: &Advice,
) -> io::Result<()> {
    let _ = (out, advice);
    Ok(())
}

/// Writes `advice` as one line of JSON sharing `status`'s
/// `schema_version`.
///
/// # Errors
///
/// When `out` fails to take the bytes.
pub fn write_advice_json<W: Write + ?Sized>(
    out: &mut W,
    advice: &Advice,
) -> io::Result<()> {
    let _ = (out, advice);
    Ok(())
}

#[cfg(test)]
mod tests;
