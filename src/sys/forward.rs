//! Signals `--govern` forwards, without a handler: `SIGINT`, `SIGTERM`,
//! `SIGHUP` and `SIGCHLD` are blocked on this thread and taken one at a
//! time with a timed wait (src/sys:T22, src/launch:V4, `.:V22`).
//!
//! While `cgmr` waits for the gates, nothing is blocked: a signal has its
//! default disposition and ends `cgmr` (130, 143). Only once `--govern`
//! has started the command does the caller block them here, so they reach
//! the command's group instead; dropping [`Blocked`] puts the old mask
//! back.

use std::io;
use std::marker::PhantomData;
use std::time::Duration;

use super::GroupSignal;

/// A signal `--govern` passes on to the command's group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ForwardSignal {
    /// `SIGINT`.
    Int,
    /// `SIGTERM`.
    Term,
    /// `SIGHUP`.
    Hup,
}

impl ForwardSignal {
    /// The same signal, as sent to a group.
    #[must_use]
    pub const fn group(self) -> GroupSignal {
        match self {
            Self::Int => GroupSignal::Int,
            Self::Term => GroupSignal::Term,
            Self::Hup => GroupSignal::Hup,
        }
    }
}

/// What ended a [`Blocked::wait_timeout`] before its time ran out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Wake {
    /// A signal to pass on.
    Forward(ForwardSignal),
    /// `SIGCHLD`: a child changed state; may be spurious, so the caller
    /// checks its child.
    Child,
}

/// The forwarded signals and `SIGCHLD`, blocked on this thread until
/// drop.
#[derive(Debug)]
pub struct Blocked {
    /// The thread is the one whose mask changed: not `Send`.
    _thread: PhantomData<*const ()>,
}

/// Blocks `SIGINT`, `SIGTERM`, `SIGHUP` and `SIGCHLD` on this thread.
///
/// # Errors
///
/// The OS error when the mask cannot change.
pub fn block_forwarded() -> io::Result<Blocked> {
    Err(io::Error::from(io::ErrorKind::Unsupported))
}

impl Blocked {
    /// Waits up to `d` for a blocked signal and takes it; `None` when
    /// none came in time.
    ///
    /// # Errors
    ///
    /// The OS error when the wait fails.
    pub fn wait_timeout(&mut self, d: Duration) -> io::Result<Option<Wake>> {
        let _ = d;
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }
}

impl Drop for Blocked {
    /// Puts the mask from before [`block_forwarded`] back.
    fn drop(&mut self) {}
}

#[cfg(test)]
mod tests;
