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
    split(&ffi::now_local()?)
}

/// The hour and minute of the broken-down time `tm`.
fn split(tm: &libc::tm) -> io::Result<(u8, u8)> {
    let field = |value: libc::c_int, below: u8, name: &str| {
        u8::try_from(value)
            .ok()
            .filter(|&v| v < below)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("local time {name} {value} out of range"),
                )
            })
    };
    Ok((
        field(tm.tm_hour, 24, "hour")?,
        field(tm.tm_min, 60, "minute")?,
    ))
}

#[allow(unsafe_code, reason = "time and localtime_r FFI (src/sys:C)")]
mod ffi {
    use std::io;
    use std::mem::MaybeUninit;
    use std::ptr;

    /// Now, broken down in local time (`TZ` and DST applied).
    pub(super) fn now_local() -> io::Result<libc::tm> {
        // SAFETY: `time` with a null pointer only returns the time.
        let now = unsafe { libc::time(ptr::null_mut()) };
        if now == -1 {
            return Err(io::Error::last_os_error());
        }
        let mut tm = MaybeUninit::<libc::tm>::uninit();
        // SAFETY: both pointers are valid for the call; `localtime_r` is
        // the reentrant form and writes only into `tm`.
        let filled =
            unsafe { libc::localtime_r(&raw const now, tm.as_mut_ptr()) };
        if filled.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: a non-null answer means `localtime_r` filled in `tm`.
        Ok(unsafe { tm.assume_init() })
    }
}

#[cfg(test)]
mod tests;
