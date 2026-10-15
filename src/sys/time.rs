//! Time of day: the local wall clock hour and minute, with the zone
//! (`TZ`) and any DST change applied by `localtime_r` (src/sys:T18,
//! src/presence:V4).

use std::io;

/// The hour and minute of now in local time.
///
/// # Errors
///
/// The OS error when the local time cannot be read;
/// [`io::ErrorKind::InvalidData`] for a broken-down time out of range.
pub fn local_time() -> io::Result<(u8, u8)> {
    let _ = split;
    Err(io::Error::from(io::ErrorKind::Unsupported))
}

/// The hour and minute of the broken-down time `tm`.
fn split(tm: &libc::tm) -> io::Result<(u8, u8)> {
    let _ = tm;
    Err(io::Error::from(io::ErrorKind::Unsupported))
}

#[cfg(test)]
mod tests;
