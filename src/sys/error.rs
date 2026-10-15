//! The one error shape for "this machine has no such source"
//! (src/sys:V4): [`io::ErrorKind::Unsupported`] with a reason for the user.

use std::fmt::Display;
use std::io;

/// An [`io::ErrorKind::Unsupported`] error whose message is `reason`.
pub(crate) fn unsupported(reason: impl Display) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, reason.to_string())
}

#[cfg(test)]
mod tests;
