//! macOS `notify(3)` state: the C API that carries thermal pressure and
//! Low Power Mode, so no Objective-C is needed (src/sys:T8, R4;
//! src/power:R1, R2). Every other OS answers
//! [`io::ErrorKind::Unsupported`].

use std::io;

/// `kOSThermalNotificationPressureLevelName`: 0 is nominal; higher is
/// more thermal pressure (src/power:R1).
pub const THERMAL_PRESSURE: &str = "com.apple.system.thermalpressurelevel";

/// Low Power Mode: non-zero when on (src/power:R2).
pub const LOW_POWER_MODE: &str = "com.apple.system.lowpowermode";

/// The 64-bit state last posted under notification `name`; 0 if none was.
///
/// # Errors
///
/// [`io::ErrorKind::InvalidInput`] for a name with a NUL; the notify
/// status as an error; Unsupported off macOS.
pub fn notify_state(name: &str) -> io::Result<u64> {
    imp::state(name)
}

/// A notify status code as an error naming the call.
#[cfg(any(test, target_os = "macos"))]
fn status_error(call: &str, name: &str, status: u32) -> io::Error {
    io::Error::other(format!("{call} {name}: notify status {status}"))
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code, reason = "notify(3) FFI (src/sys:C, R4)")]
mod imp {
    use std::ffi::{CString, c_char, c_int};
    use std::io;

    use super::status_error;

    unsafe extern "C" {
        fn notify_register_check(name: *const c_char, token: *mut c_int)
        -> u32;
        fn notify_get_state(token: c_int, state: *mut u64) -> u32;
        fn notify_cancel(token: c_int) -> u32;
    }

    pub(super) fn state(name: &str) -> io::Result<u64> {
        let token = register(name)?;
        let mut value: u64 = 0;
        // SAFETY: `token` is registered; `value` is a valid out pointer.
        let status = unsafe { notify_get_state(token, &raw mut value) };
        // SAFETY: `token` is registered and used by nothing after this.
        let _ = unsafe { notify_cancel(token) };
        if status == 0 {
            Ok(value)
        } else {
            Err(status_error("notify_get_state", name, status))
        }
    }

    /// A check token for `name`, to cancel after use.
    fn register(name: &str) -> io::Result<c_int> {
        let c_name = CString::new(name)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
        let mut token: c_int = 0;
        // SAFETY: `c_name` is NUL-terminated and outlives the call; `token`
        // is a valid out pointer for one registration token.
        let status =
            unsafe { notify_register_check(c_name.as_ptr(), &raw mut token) };
        if status == 0 {
            Ok(token)
        } else {
            Err(status_error("notify_register_check", name, status))
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use std::io;

    pub(super) fn state(name: &str) -> io::Result<u64> {
        Err(crate::sys::error::unsupported(format_args!(
            "notify {name}: macOS only"
        )))
    }
}

#[cfg(test)]
mod tests;
