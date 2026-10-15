//! Users: names and ids from the password database (`getpwnam_r`,
//! `getpwuid_r`) and this process's uid, for counts by user
//! (src/sys:T18, src/procs).

use std::ffi::{CString, c_int};
use std::io;

/// The first buffer for the strings of one password entry.
const FIRST_BUFFER: usize = 1024;

/// The largest buffer tried before the lookup gives up.
const MAX_BUFFER: usize = 1 << 20;

/// The uid of the user called `name`; `None` when there is no such user.
///
/// # Errors
///
/// [`io::ErrorKind::InvalidInput`] for a name with a NUL; a lookup
/// failure other than "not found" as the OS error.
pub fn user_id(name: &str) -> io::Result<Option<u32>> {
    let c_name = CString::new(name).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidInput, format!("{name:?}: NUL"))
    })?;
    with_buffer(|buf| ffi::uid_of(&c_name, buf))
}

/// The name of the user with `uid`; `None` when no user has it.
///
/// # Errors
///
/// [`io::ErrorKind::InvalidData`] for a name not UTF-8; a lookup failure
/// other than "not found" as the OS error.
pub fn user_name(uid: u32) -> io::Result<Option<String>> {
    with_buffer(|buf| ffi::name_of(uid, buf))?
        .map(|bytes| {
            String::from_utf8(bytes).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("uid {uid}: name is not UTF-8"),
                )
            })
        })
        .transpose()
}

/// The real uid of this process.
#[must_use]
pub fn current_uid() -> u32 {
    rustix::process::getuid().as_raw()
}

/// Runs `call` on a scratch buffer, doubling it while the call answers
/// `ERANGE` (too small), up to [`MAX_BUFFER`]. `call` answers `Ok(None)`
/// for "no such entry" and `Err(errno)` for a failure.
fn with_buffer<T>(
    mut call: impl FnMut(&mut [u8]) -> Result<Option<T>, c_int>,
) -> io::Result<Option<T>> {
    let mut buf = vec![0_u8; FIRST_BUFFER];
    loop {
        match call(&mut buf) {
            Err(libc::ERANGE) if buf.len() < MAX_BUFFER => {
                buf.resize(buf.len().saturating_mul(2), 0);
            }
            Err(code) if is_not_found(code) => return Ok(None),
            Err(code) => return Err(io::Error::from_raw_os_error(code)),
            Ok(found) => return Ok(found),
        }
    }
}

/// Codes `getpw*_r` may answer for "no such entry" (`getpwnam_r(3)`).
const fn is_not_found(code: c_int) -> bool {
    matches!(code, libc::ENOENT | libc::ESRCH | libc::EBADF | libc::EPERM)
}

#[allow(unsafe_code, reason = "getpwnam_r and getpwuid_r FFI (src/sys:C)")]
mod ffi {
    use std::ffi::{CStr, c_char, c_int};
    use std::mem::MaybeUninit;
    use std::ptr;

    use libc::passwd;

    /// The uid of the entry named `name`, with `buf` for its strings.
    pub(super) fn uid_of(
        name: &CStr,
        buf: &mut [u8],
    ) -> Result<Option<u32>, c_int> {
        lookup(
            buf,
            // SAFETY: `name` is NUL-terminated and outlives the call;
            // `lookup` passes valid out pointers and a buffer of `len`.
            |entry, strings, len, found| unsafe {
                libc::getpwnam_r(name.as_ptr(), entry, strings, len, found)
            },
            |pw| pw.pw_uid,
        )
    }

    /// The name bytes of the entry with `uid`, with `buf` for its strings.
    pub(super) fn name_of(
        uid: u32,
        buf: &mut [u8],
    ) -> Result<Option<Vec<u8>>, c_int> {
        lookup(
            buf,
            // SAFETY: `lookup` passes valid out pointers and a buffer of
            // `len`.
            |entry, strings, len, found| unsafe {
                libc::getpwuid_r(uid, entry, strings, len, found)
            },
            // SAFETY: the call set `pw_name` to a NUL-terminated string
            // inside the buffer, alive and unchanged while `read` runs.
            |pw| unsafe { CStr::from_ptr(pw.pw_name) }.to_bytes().to_vec(),
        )
    }

    /// Runs a `getpw*_r` `call` on out pointers and `buf`, then `read`s
    /// the entry it found while `buf` still holds its strings.
    fn lookup<T>(
        buf: &mut [u8],
        call: impl FnOnce(
            *mut passwd,
            *mut c_char,
            usize,
            *mut *mut passwd,
        ) -> c_int,
        read: impl FnOnce(&passwd) -> T,
    ) -> Result<Option<T>, c_int> {
        let mut entry = MaybeUninit::<passwd>::uninit();
        let mut found: *mut passwd = ptr::null_mut();
        let code = call(
            entry.as_mut_ptr(),
            buf.as_mut_ptr().cast(),
            buf.len(),
            &raw mut found,
        );
        if code != 0 {
            return Err(code);
        }
        // SAFETY: a non-null `found` points at `entry`, which the call
        // filled in.
        Ok(unsafe { found.as_ref() }.map(read))
    }
}

#[cfg(test)]
mod tests;
