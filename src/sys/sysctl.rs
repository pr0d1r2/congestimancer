//! macOS `sysctlbyname`: one thin FFI read of raw bytes, decoded by pure
//! functions into typed values (src/sys:T5, V1, V2, V3). Every other OS
//! answers [`io::ErrorKind::Unsupported`].

use std::io;

use super::raw::{ne_i32, ne_i64, ne_u64};

/// Swap space from `vm.swapusage`, in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct SwapUsage {
    /// Swap space.
    pub total: u64,
    /// Swap space free.
    pub available: u64,
    /// Swap space in use.
    pub used: u64,
}

/// The integer sysctl `name`, such as `kern.maxfiles` or
/// `kern.memorystatus_vm_pressure_level`.
///
/// # Errors
///
/// [`io::ErrorKind::Unsupported`] when this OS has no such sysctl;
/// [`io::ErrorKind::InvalidData`] when its value is not a 4 or 8 byte
/// integer.
pub fn sysctl_int(name: &str) -> io::Result<i64> {
    let bytes = read(name)?;
    decode_int(&bytes).ok_or_else(|| malformed(name, bytes.len()))
}

/// Swap space, from `vm.swapusage`.
///
/// # Errors
///
/// As [`sysctl_int`].
pub fn swap_usage() -> io::Result<SwapUsage> {
    let name = "vm.swapusage";
    let bytes = read(name)?;
    decode_swap_usage(&bytes).ok_or_else(|| malformed(name, bytes.len()))
}

fn malformed(name: &str, len: usize) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("sysctl {name}: unexpected {len} byte value"),
    )
}

/// A 4 or 8 byte native-endian integer.
fn decode_int(bytes: &[u8]) -> Option<i64> {
    match bytes.len() {
        4 => ne_i32(bytes, 0).map(i64::from),
        8 => ne_i64(bytes, 0),
        _ => None,
    }
}

/// `struct xsw_usage`: total, avail and used as `u64`, then page size and
/// the encrypted flag.
fn decode_swap_usage(bytes: &[u8]) -> Option<SwapUsage> {
    Some(SwapUsage {
        total: ne_u64(bytes, 0)?,
        available: ne_u64(bytes, 8)?,
        used: ne_u64(bytes, 16)?,
    })
}

/// The raw value of sysctl `name`.
#[cfg(target_os = "macos")]
fn read(name: &str) -> io::Result<Vec<u8>> {
    let c_name = std::ffi::CString::new(name)
        .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
    sized(|buf, len| by_name(&c_name, buf, len))
        .map_err(|err| not_present(name, err))
}

/// One `sysctlbyname` read into `buf`, null for a size query.
#[cfg(target_os = "macos")]
#[allow(unsafe_code, reason = "sysctlbyname FFI (src/sys:C)")]
fn by_name(
    name: &std::ffi::CStr,
    buf: *mut libc::c_void,
    len: &mut usize,
) -> io::Result<()> {
    // SAFETY: `name` is NUL-terminated; `sized`, the only caller's driver,
    // passes a null `buf` (a size query) or one of `*len` writable bytes,
    // and `len` is a valid `size_t` the kernel updates in place.
    let rc = unsafe {
        libc::sysctlbyname(name.as_ptr(), buf, len, std::ptr::null_mut(), 0)
    };
    if rc == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

/// `ENOENT` read as "no such sysctl here".
#[cfg(target_os = "macos")]
fn not_present(name: &str, err: io::Error) -> io::Error {
    if err.raw_os_error() == Some(libc::ENOENT) {
        super::error::unsupported(format_args!("sysctl {name}: not present"))
    } else {
        err
    }
}

/// Runs a sysctl-style `call` once with a null buffer for the size, then
/// with a buffer of that size, returning the bytes written. A value that
/// grew in between (`ENOMEM`) is asked for again, a few times.
#[cfg(any(test, target_os = "macos"))]
pub(crate) fn sized(
    mut call: impl FnMut(*mut libc::c_void, &mut usize) -> io::Result<()>,
) -> io::Result<Vec<u8>> {
    for _ in 0..4 {
        match sized_once(&mut call) {
            Err(err) if err.raw_os_error() == Some(libc::ENOMEM) => {}
            done => return done,
        }
    }
    Err(io::Error::from_raw_os_error(libc::ENOMEM))
}

/// One size query and one fill.
#[cfg(any(test, target_os = "macos"))]
fn sized_once(
    call: &mut impl FnMut(*mut libc::c_void, &mut usize) -> io::Result<()>,
) -> io::Result<Vec<u8>> {
    let mut len = 0;
    call(std::ptr::null_mut(), &mut len)?;
    let mut buf = vec![0_u8; len];
    call(buf.as_mut_ptr().cast(), &mut len)?;
    buf.truncate(len);
    Ok(buf)
}

#[cfg(not(target_os = "macos"))]
fn read(name: &str) -> io::Result<Vec<u8>> {
    Err(super::error::unsupported(format_args!(
        "sysctl {name}: macOS only"
    )))
}

#[cfg(test)]
mod tests;
