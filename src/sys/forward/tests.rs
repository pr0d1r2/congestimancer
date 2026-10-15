use std::io;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use super::{ForwardSignal, Wake, block_forwarded};
use crate::sys::GroupSignal;

/// One signal test at a time: kqueue sees signals process-wide.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Long enough for a signal already pending, short enough to keep the
/// suite fast.
const WAIT: Duration = Duration::from_secs(1);

#[allow(unsafe_code, reason = "test-only signal FFI (src/sys:C)")]
mod ffi {
    use std::io;

    /// Sends `sig` to this thread only, so no other test thread sees it.
    pub(super) fn raise(sig: libc::c_int) -> io::Result<()> {
        // SAFETY: `raise` takes a plain signal number; the caller has it
        // blocked, so no handler or default action runs.
        if unsafe { libc::raise(sig) } == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    /// This thread, as a number another thread can hold.
    pub(super) fn this_thread() -> usize {
        // SAFETY: `pthread_self` has no preconditions.
        let me = unsafe { libc::pthread_self() };
        me as usize
    }

    /// Sends `sig` to thread `thread` (from [`this_thread`]).
    pub(super) fn kill_thread(
        thread: usize,
        sig: libc::c_int,
    ) -> io::Result<()> {
        // SAFETY: `thread` came from `pthread_self` of a thread that is
        // still running (it waits on the joining test).
        let rc = unsafe { libc::pthread_kill(thread as libc::pthread_t, sig) };
        if rc == 0 {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(rc))
        }
    }

    /// Whether `sig` is blocked on this thread now.
    pub(super) fn is_blocked(sig: libc::c_int) -> io::Result<bool> {
        let mut mask = std::mem::MaybeUninit::<libc::sigset_t>::zeroed();
        // SAFETY: a null new set only reads the mask into `mask`.
        let rc = unsafe {
            libc::pthread_sigmask(
                libc::SIG_BLOCK,
                std::ptr::null(),
                mask.as_mut_ptr(),
            )
        };
        if rc != 0 {
            return Err(io::Error::from_raw_os_error(rc));
        }
        // SAFETY: `pthread_sigmask` filled `mask` on success.
        Ok(unsafe { libc::sigismember(mask.as_ptr(), sig) } == 1)
    }
}

#[test]
fn a_pending_signal_is_taken_as_what_to_forward() -> io::Result<()> {
    let _serial = serial();
    let mut blocked = block_forwarded()?;
    for (sig, want) in [
        (libc::SIGINT, ForwardSignal::Int),
        (libc::SIGTERM, ForwardSignal::Term),
        (libc::SIGHUP, ForwardSignal::Hup),
    ] {
        ffi::raise(sig)?;
        assert_eq!(blocked.wait_timeout(WAIT)?, Some(Wake::Forward(want)));
    }
    Ok(())
}

#[test]
fn a_child_signal_wakes_the_wait() -> io::Result<()> {
    let _serial = serial();
    let mut blocked = block_forwarded()?;
    ffi::raise(libc::SIGCHLD)?;
    assert_eq!(blocked.wait_timeout(WAIT)?, Some(Wake::Child));
    Ok(())
}

#[test]
fn a_signal_sent_during_the_wait_ends_it() -> io::Result<()> {
    let _serial = serial();
    let mut blocked = block_forwarded()?;
    let me = ffi::this_thread();
    let sender = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(30));
        ffi::kill_thread(me, libc::SIGTERM)
    });
    let woke = blocked.wait_timeout(WAIT);
    sender
        .join()
        .map_err(|_| io::Error::other("sender panicked"))??;
    assert_eq!(woke?, Some(Wake::Forward(ForwardSignal::Term)));
    Ok(())
}

#[test]
fn nothing_to_forward_waits_out_the_timeout() -> io::Result<()> {
    let _serial = serial();
    let mut blocked = block_forwarded()?;
    let start = Instant::now();
    let woke = blocked.wait_timeout(Duration::from_millis(40))?;
    // A SIGCHLD from another test's child may wake it; nothing else.
    assert!(!matches!(woke, Some(Wake::Forward(_))));
    if woke.is_none() {
        assert!(start.elapsed() >= Duration::from_millis(30));
    }
    Ok(())
}

#[test]
fn dropping_puts_the_old_mask_back() -> io::Result<()> {
    let _serial = serial();
    assert!(!ffi::is_blocked(libc::SIGHUP)?);
    let blocked = block_forwarded()?;
    for sig in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP, libc::SIGCHLD] {
        assert!(ffi::is_blocked(sig)?);
    }
    drop(blocked);
    for sig in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP, libc::SIGCHLD] {
        assert!(!ffi::is_blocked(sig)?);
    }
    Ok(())
}

#[test]
fn a_forwarded_signal_maps_to_the_same_group_signal() {
    assert_eq!(ForwardSignal::Int.group(), GroupSignal::Int);
    assert_eq!(ForwardSignal::Term.group(), GroupSignal::Term);
    assert_eq!(ForwardSignal::Hup.group(), GroupSignal::Hup);
}
