//! Pure parsers for Linux memory files and the swap byte count
//! (src/mem:T5, V6).

use super::MemError;

/// `/proc/vmstat`: cumulative VM event counters, one `key value` a line.
pub(super) const VMSTAT: &str = "/proc/vmstat";

/// `/proc/self/smaps`: this process's mappings, each with its
/// `KernelPageSize`.
pub(super) const SMAPS: &str = "/proc/self/smaps";

/// Pages swapped in and out since boot (`pswpin`, `pswpout`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct SwapPages {
    /// Pages swapped in.
    pub pages_in: u64,
    /// Pages swapped out.
    pub pages_out: u64,
}

impl SwapPages {
    /// Swap in plus swap out, in bytes of `page_size` pages; `None` on
    /// overflow, never a wrapped count (src/mem:V6).
    #[must_use]
    pub const fn bytes(self, page_size: u64) -> Option<u64> {
        let _ = page_size;
        None
    }
}

/// Parses `/proc/vmstat` for `pswpin` and `pswpout`.
///
/// # Errors
///
/// [`MemError::Malformed`] for a line that is not `key number`;
/// [`MemError::Missing`] when either counter is absent.
pub fn parse_vmstat(text: &str) -> Result<SwapPages, MemError> {
    Err(MemError::malformed(VMSTAT, text))
}

/// The base page size from `/proc/self/smaps`: the smallest
/// `KernelPageSize`, in bytes, since huge page mappings report larger.
///
/// # Errors
///
/// [`MemError::Malformed`] for a `KernelPageSize` line that is not
/// `N kB`; [`MemError::Missing`] when there is none.
pub fn parse_page_size(text: &str) -> Result<u64, MemError> {
    let _ = text;
    Err(MemError::Missing {
        file: SMAPS,
        key: "KernelPageSize",
    })
}

#[cfg(test)]
mod tests;
