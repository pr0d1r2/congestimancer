//! This process's disk IO priority: macOS `setiopolicy_np`, Linux
//! `ioprio_set` (src/sys:T10, V1, V2). Like the nice value it carries
//! across `exec`, so `src/launch` sets it before running the command.

use std::io;

/// How this process's disk IO competes with others'.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum IoPriority {
    /// The OS default.
    Default,
    /// Only when the disk is otherwise idle: macOS `IOPOL_THROTTLE`, Linux
    /// the idle class.
    Background,
}

/// Sets this process's disk IO priority.
///
/// # Errors
///
/// The OS error; Unsupported on an OS with neither call.
pub fn set_io_priority(priority: IoPriority) -> io::Result<()> {
    imp::set(priority)
}

/// This process's disk IO priority; any policy other than background reads
/// as [`IoPriority::Default`].
///
/// # Errors
///
/// As [`set_io_priority`].
pub fn io_priority() -> io::Result<IoPriority> {
    imp::get()
}

/// macOS `IOPOL_*` values (`sys/resource.h`).
#[cfg(any(test, target_os = "macos"))]
mod iopol {
    pub(super) const TYPE_DISK: i32 = 0;
    pub(super) const SCOPE_PROCESS: i32 = 0;
    pub(super) const DEFAULT: i32 = 0;
    pub(super) const THROTTLE: i32 = 3;
}

/// The macOS policy for `priority`.
#[cfg(any(test, target_os = "macos"))]
const fn policy_of(priority: IoPriority) -> i32 {
    match priority {
        IoPriority::Default => iopol::DEFAULT,
        IoPriority::Background => iopol::THROTTLE,
    }
}

/// The priority a macOS policy stands for.
#[cfg(any(test, target_os = "macos"))]
const fn priority_of_policy(policy: i32) -> IoPriority {
    if policy == iopol::THROTTLE {
        IoPriority::Background
    } else {
        IoPriority::Default
    }
}

/// Linux `IOPRIO_CLASS_*` and the class shift (`linux/ioprio.h`).
#[cfg(any(test, target_os = "linux"))]
mod ioprio {
    pub(super) const CLASS_SHIFT: u32 = 13;
    pub(super) const CLASS_BE: i32 = 2;
    pub(super) const CLASS_IDLE: i32 = 3;
    #[cfg(target_os = "linux")]
    pub(super) const WHO_PROCESS: i32 = 1;
}

/// The Linux ioprio value for `priority`: best effort level 4 (the
/// default) or the idle class.
#[cfg(any(test, target_os = "linux"))]
const fn ioprio_of(priority: IoPriority) -> i32 {
    match priority {
        IoPriority::Default => (ioprio::CLASS_BE << ioprio::CLASS_SHIFT) | 4,
        IoPriority::Background => ioprio::CLASS_IDLE << ioprio::CLASS_SHIFT,
    }
}

/// The priority a Linux ioprio value stands for.
#[cfg(any(test, target_os = "linux"))]
const fn priority_of_ioprio(value: i32) -> IoPriority {
    if value >> ioprio::CLASS_SHIFT == ioprio::CLASS_IDLE {
        IoPriority::Background
    } else {
        IoPriority::Default
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code, reason = "setiopolicy_np FFI (src/sys:C)")]
mod imp {
    use std::ffi::c_int;
    use std::io;

    use super::{IoPriority, iopol, policy_of, priority_of_policy};

    unsafe extern "C" {
        fn setiopolicy_np(iotype: c_int, scope: c_int, policy: c_int) -> c_int;
        fn getiopolicy_np(iotype: c_int, scope: c_int) -> c_int;
    }

    pub(super) fn set(priority: IoPriority) -> io::Result<()> {
        let policy = policy_of(priority);
        // SAFETY: plain integers in, a status out; no memory is shared.
        let rc = unsafe {
            setiopolicy_np(iopol::TYPE_DISK, iopol::SCOPE_PROCESS, policy)
        };
        if rc == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub(super) fn get() -> io::Result<IoPriority> {
        // SAFETY: plain integers in, a policy or -1 out.
        let policy =
            unsafe { getiopolicy_np(iopol::TYPE_DISK, iopol::SCOPE_PROCESS) };
        if policy < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(priority_of_policy(policy))
    }
}

#[cfg(target_os = "linux")]
#[allow(unsafe_code, reason = "ioprio syscalls (src/sys:C)")]
mod imp {
    use std::io;

    use super::{IoPriority, ioprio, ioprio_of, priority_of_ioprio};

    pub(super) fn set(priority: IoPriority) -> io::Result<()> {
        let value = libc::c_long::from(ioprio_of(priority));
        let who = libc::c_long::from(ioprio::WHO_PROCESS);
        // SAFETY: `ioprio_set(which, who, ioprio)` takes integers only;
        // who 0 is this process.
        let rc = unsafe { libc::syscall(libc::SYS_ioprio_set, who, 0, value) };
        if rc == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub(super) fn get() -> io::Result<IoPriority> {
        let who = libc::c_long::from(ioprio::WHO_PROCESS);
        // SAFETY: `ioprio_get(which, who)` takes integers only.
        let rc = unsafe { libc::syscall(libc::SYS_ioprio_get, who, 0) };
        let value =
            i32::try_from(rc).map_err(|_| io::Error::last_os_error())?;
        if value < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(priority_of_ioprio(value))
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
mod imp {
    use std::io;

    use super::IoPriority;

    pub(super) fn set(_: IoPriority) -> io::Result<()> {
        Err(crate::sys::error::unsupported(
            "IO priority: macOS and Linux only",
        ))
    }

    pub(super) fn get() -> io::Result<IoPriority> {
        Err(crate::sys::error::unsupported(
            "IO priority: macOS and Linux only",
        ))
    }
}

#[cfg(test)]
mod tests;
