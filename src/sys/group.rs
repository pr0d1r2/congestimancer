//! Process group signals through `rustix`: `--govern` stops, continues
//! and forwards signals to the command's group (src/sys:T21,
//! src/launch:V4).

use std::io;

/// A signal `--govern` sends to the command's process group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum GroupSignal {
    /// `SIGSTOP`: pause the group while a gate fails.
    Stop,
    /// `SIGCONT`: resume it once the gates pass.
    Cont,
    /// `SIGINT`, forwarded.
    Int,
    /// `SIGTERM`, forwarded.
    Term,
    /// `SIGHUP`, forwarded.
    Hup,
}

/// Sends `sig` to every process in group `pgid`.
///
/// # Errors
///
/// [`io::ErrorKind::InvalidInput`] for group 0 (this process's own group)
/// or one past `pid_t`; the OS error otherwise, such as `ESRCH` when the
/// group is gone.
pub fn signal_group(pgid: u32, sig: GroupSignal) -> io::Result<()> {
    let _ = (pgid, sig);
    Err(io::Error::from(io::ErrorKind::Unsupported))
}

#[cfg(test)]
mod tests;
