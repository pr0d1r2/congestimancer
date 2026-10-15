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
    /// The mask before [`block_forwarded`], put back on drop.
    old: libc::sigset_t,
    /// The blocked signals: what `sigtimedwait` takes.
    #[cfg(target_os = "linux")]
    set: libc::sigset_t,
    /// A kqueue that hears the blocked signals arrive (macOS lacks
    /// `sigtimedwait`).
    #[cfg(target_os = "macos")]
    queue: std::os::fd::OwnedFd,
    /// The mask belongs to this thread: not `Send`, not `Sync`.
    _thread: PhantomData<*const ()>,
}

/// What [`block_forwarded`] blocks, forwarded signals first.
const BLOCKED: [libc::c_int; 4] =
    [libc::SIGINT, libc::SIGTERM, libc::SIGHUP, libc::SIGCHLD];

/// What a taken signal number means; `None` for one not blocked here.
const fn wake_of(sig: libc::c_int) -> Option<Wake> {
    match sig {
        libc::SIGINT => Some(Wake::Forward(ForwardSignal::Int)),
        libc::SIGTERM => Some(Wake::Forward(ForwardSignal::Term)),
        libc::SIGHUP => Some(Wake::Forward(ForwardSignal::Hup)),
        libc::SIGCHLD => Some(Wake::Child),
        _ => None,
    }
}

/// Blocks `SIGINT`, `SIGTERM`, `SIGHUP` and `SIGCHLD` on this thread.
///
/// Call it on the thread that waits, with no other thread left to take
/// these signals. A child spawned after it starts with an empty mask
/// (`std` resets it), so the command still dies of what it is sent.
///
/// # Errors
///
/// The OS error when the mask cannot change.
pub fn block_forwarded() -> io::Result<Blocked> {
    let set = ffi::set_of(&BLOCKED)?;
    #[cfg(target_os = "macos")]
    let queue = ffi::watch(&BLOCKED)?;
    let old = ffi::block(&set)?;
    Ok(Blocked {
        old,
        #[cfg(target_os = "linux")]
        set,
        #[cfg(target_os = "macos")]
        queue,
        _thread: PhantomData,
    })
}

impl Blocked {
    /// Waits up to `d` for a blocked signal and takes it; `None` when
    /// none came in time, or the wait was interrupted.
    ///
    /// # Errors
    ///
    /// The OS error when the wait fails.
    #[cfg(target_os = "linux")]
    pub fn wait_timeout(&mut self, d: Duration) -> io::Result<Option<Wake>> {
        Ok(ffi::timed_wait(&self.set, d)?.and_then(wake_of))
    }

    /// Waits up to `d` for a blocked signal and takes it; `None` when
    /// none came in time, or the wait was interrupted.
    ///
    /// macOS has no `sigtimedwait`: a pending signal is taken with
    /// `sigwait`, which then returns at once; else the kqueue waits for
    /// one to arrive. `SIGCHLD`, ignored by default, may be dropped
    /// rather than left pending, so its arrival alone is a wake.
    ///
    /// # Errors
    ///
    /// The OS error when the wait fails.
    #[cfg(target_os = "macos")]
    pub fn wait_timeout(&mut self, d: Duration) -> io::Result<Option<Wake>> {
        let deadline = std::time::Instant::now().checked_add(d);
        loop {
            if let Some(sig) = ffi::take_pending(&BLOCKED)? {
                return Ok(wake_of(sig));
            }
            let left = time_left(deadline, d);
            if left.is_zero() {
                return Ok(None);
            }
            if ffi::next_arrival(&self.queue, left)? == Some(libc::SIGCHLD) {
                let pending = ffi::take_pending(&BLOCKED)?;
                return Ok(pending.map_or(Some(Wake::Child), wake_of));
            }
        }
    }
}

/// What is left until `deadline`; `whole` when it is past `Instant`'s
/// range.
#[cfg(target_os = "macos")]
fn time_left(
    deadline: Option<std::time::Instant>,
    whole: Duration,
) -> Duration {
    deadline.map_or(whole, |end| {
        end.saturating_duration_since(std::time::Instant::now())
    })
}

impl Blocked {
    /// Makes a child `command` spawns start with the mask from before
    /// [`block_forwarded`]: `std` hands the blocked mask on, and a
    /// command with these signals blocked would ignore what is forwarded
    /// to it.
    pub fn unblock_in(&self, command: &mut std::process::Command) {
        ffi::mask_in_child(command, self.old);
    }
}

impl Drop for Blocked {
    /// Puts the mask from before [`block_forwarded`] back; a signal
    /// still pending then meets its old disposition.
    fn drop(&mut self) {
        // Nothing to do on failure: the thread keeps the blocked mask.
        let _ = ffi::restore(&self.old);
    }
}

#[allow(unsafe_code, reason = "signal mask and wait FFI (src/sys:C)")]
mod ffi {
    use std::io;
    use std::mem::MaybeUninit;
    use std::ptr;
    use std::time::Duration;

    /// The set of `sigs`.
    pub(super) fn set_of(sigs: &[libc::c_int]) -> io::Result<libc::sigset_t> {
        let mut set = MaybeUninit::<libc::sigset_t>::uninit();
        // SAFETY: `sigemptyset` initialises the set it is given.
        if unsafe { libc::sigemptyset(set.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }
        for &sig in sigs {
            // SAFETY: the set was initialised by `sigemptyset` above.
            if unsafe { libc::sigaddset(set.as_mut_ptr(), sig) } != 0 {
                return Err(io::Error::last_os_error());
            }
        }
        // SAFETY: `sigemptyset` succeeded, so the set is initialised.
        Ok(unsafe { set.assume_init() })
    }

    /// Adds `set` to this thread's mask; returns the mask before.
    pub(super) fn block(set: &libc::sigset_t) -> io::Result<libc::sigset_t> {
        let mut old = MaybeUninit::<libc::sigset_t>::uninit();
        // SAFETY: both pointers are valid; on success the old mask is
        // written into `old`.
        let rc = unsafe {
            libc::pthread_sigmask(libc::SIG_BLOCK, set, old.as_mut_ptr())
        };
        if rc != 0 {
            return Err(io::Error::from_raw_os_error(rc));
        }
        // SAFETY: `pthread_sigmask` succeeded and filled `old`.
        Ok(unsafe { old.assume_init() })
    }

    /// Sets this thread's mask to `old`.
    pub(super) fn restore(old: &libc::sigset_t) -> io::Result<()> {
        // SAFETY: `old` is a valid set; a null out pointer is allowed.
        let rc = unsafe {
            libc::pthread_sigmask(libc::SIG_SETMASK, old, ptr::null_mut())
        };
        if rc == 0 {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(rc))
        }
    }

    /// Sets the mask of the child `command` spawns to `mask`, after fork
    /// and before exec.
    pub(super) fn mask_in_child(
        command: &mut std::process::Command,
        mask: libc::sigset_t,
    ) {
        use std::os::unix::process::CommandExt as _;
        // SAFETY: the hook runs in the forked child before exec, where
        // only async-signal-safe calls are sound; it makes one
        // `sigprocmask` call (async-signal-safe) on a set it owns by
        // copy, and allocates nothing.
        unsafe {
            command.pre_exec(move || {
                let rc = libc::sigprocmask(
                    libc::SIG_SETMASK,
                    &raw const mask,
                    ptr::null_mut(),
                );
                if rc == 0 {
                    Ok(())
                } else {
                    Err(io::Error::last_os_error())
                }
            })
        };
    }

    /// `d` as a `timespec`, saturating at the largest `time_t`.
    fn timespec(d: Duration) -> libc::timespec {
        libc::timespec {
            tv_sec: libc::time_t::try_from(d.as_secs())
                .unwrap_or(libc::time_t::MAX),
            tv_nsec: libc::c_long::from(d.subsec_nanos()),
        }
    }

    /// `None` for a wait that timed out or was interrupted, else `err`.
    fn quiet(err: io::Error) -> io::Result<Option<libc::c_int>> {
        match err.raw_os_error() {
            Some(libc::EAGAIN | libc::EINTR) => Ok(None),
            _ => Err(err),
        }
    }

    /// Takes one signal of `set` within `d`; `None` on timeout or EINTR.
    #[cfg(target_os = "linux")]
    pub(super) fn timed_wait(
        set: &libc::sigset_t,
        d: Duration,
    ) -> io::Result<Option<libc::c_int>> {
        let ts = timespec(d);
        // SAFETY: `set` and `ts` are valid for the call; a null info
        // pointer is allowed.
        let sig = unsafe { libc::sigtimedwait(set, ptr::null_mut(), &ts) };
        if sig >= 0 {
            return Ok(Some(sig));
        }
        quiet(io::Error::last_os_error())
    }

    /// The signals pending on this thread.
    #[cfg(target_os = "macos")]
    fn pending() -> io::Result<libc::sigset_t> {
        let mut pending = MaybeUninit::<libc::sigset_t>::uninit();
        // SAFETY: `sigpending` fills the set it is given.
        if unsafe { libc::sigpending(pending.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `sigpending` succeeded, so the set is initialised.
        Ok(unsafe { pending.assume_init() })
    }

    /// Takes the first of `order` (all blocked) pending on this thread.
    #[cfg(target_os = "macos")]
    pub(super) fn take_pending(
        order: &[libc::c_int],
    ) -> io::Result<Option<libc::c_int>> {
        let pending = pending()?;
        // SAFETY: `pending` is an initialised set.
        let is_pending = |sig| unsafe { libc::sigismember(&pending, sig) } == 1;
        let Some(&sig) = order.iter().find(|&&sig| is_pending(sig)) else {
            return Ok(None);
        };
        let one = set_of(&[sig])?;
        let mut taken: libc::c_int = 0;
        // SAFETY: `sig` is pending and blocked, so `sigwait` returns at
        // once; both pointers are valid.
        let rc = unsafe { libc::sigwait(&one, &raw mut taken) };
        if rc != 0 {
            return Err(io::Error::from_raw_os_error(rc));
        }
        Ok(Some(taken))
    }

    /// A kqueue that hears each of `sigs` arrive.
    #[cfg(target_os = "macos")]
    pub(super) fn watch(
        sigs: &[libc::c_int],
    ) -> io::Result<std::os::fd::OwnedFd> {
        use std::os::fd::{FromRawFd as _, OwnedFd};
        // SAFETY: `kqueue` takes nothing and returns a new fd or -1.
        let fd = unsafe { libc::kqueue() };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `fd` is a fresh kqueue owned by nobody else.
        let queue = unsafe { OwnedFd::from_raw_fd(fd) };
        for &sig in sigs {
            listen(&queue, sig)?;
        }
        Ok(queue)
    }

    /// Adds `sig` to what `queue` hears.
    #[cfg(target_os = "macos")]
    fn listen(
        queue: &std::os::fd::OwnedFd,
        sig: libc::c_int,
    ) -> io::Result<()> {
        use std::os::fd::AsRawFd as _;
        let change = libc::kevent {
            ident: usize::try_from(sig).map_err(io::Error::other)?,
            filter: libc::EVFILT_SIGNAL,
            flags: libc::EV_ADD,
            fflags: 0,
            data: 0,
            udata: ptr::null_mut(),
        };
        let (fd, none) = (queue.as_raw_fd(), ptr::null_mut());
        // SAFETY: one valid change in, no events out, no timeout.
        let rc = unsafe { libc::kevent(fd, &change, 1, none, 0, ptr::null()) };
        if rc < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// The next signal `queue` hears within `d`; `None` on timeout or
    /// EINTR.
    #[cfg(target_os = "macos")]
    pub(super) fn next_arrival(
        queue: &std::os::fd::OwnedFd,
        d: Duration,
    ) -> io::Result<Option<libc::c_int>> {
        use std::os::fd::AsRawFd as _;
        let (fd, ts) = (queue.as_raw_fd(), timespec(d));
        let mut event = MaybeUninit::<libc::kevent>::zeroed();
        let out = event.as_mut_ptr();
        // SAFETY: no changes in; room for one event; `ts` is valid.
        let n = unsafe { libc::kevent(fd, ptr::null(), 0, out, 1, &ts) };
        if n <= 0 {
            return if n == 0 {
                Ok(None)
            } else {
                quiet(io::Error::last_os_error())
            };
        }
        // SAFETY: `kevent` returned one event, so it filled `event`.
        let event = unsafe { event.assume_init() };
        Ok(libc::c_int::try_from(event.ident).ok())
    }
}

#[cfg(test)]
mod tests;
