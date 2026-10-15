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
        match self.pages_in.checked_add(self.pages_out) {
            Some(pages) => pages.checked_mul(page_size),
            None => None,
        }
    }
}

/// Parses `/proc/vmstat` for `pswpin` and `pswpout`.
///
/// # Errors
///
/// [`MemError::Malformed`] for a line that is not `key number`;
/// [`MemError::Missing`] when either counter is absent.
pub fn parse_vmstat(text: &str) -> Result<SwapPages, MemError> {
    let [pages_in, pages_out] =
        lookup(text, VMSTAT, ["pswpin", "pswpout"], count)?;
    Ok(SwapPages {
        pages_in,
        pages_out,
    })
}

/// The base page size from `/proc/self/smaps`: the smallest
/// `KernelPageSize`, in bytes, since huge page mappings report larger.
///
/// # Errors
///
/// [`MemError::Malformed`] for a `KernelPageSize` line that is not
/// `N kB`; [`MemError::Missing`] when there is none.
pub fn parse_page_size(text: &str) -> Result<u64, MemError> {
    let mut smallest: Option<u64> = None;
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("KernelPageSize:") else {
            continue;
        };
        let size = kib(rest).ok_or_else(|| MemError::malformed(SMAPS, line))?;
        smallest = Some(smallest.map_or(size, |s| s.min(size)));
    }
    smallest.ok_or(MemError::Missing {
        file: SMAPS,
        key: "KernelPageSize",
    })
}

/// `N kB` as bytes; `None` for any other form or past 64 bits.
fn kib(text: &str) -> Option<u64> {
    let mut words = text.split_whitespace();
    let n: u64 = words.next()?.parse().ok()?;
    (words.next()? == "kB" && words.next().is_none())
        .then(|| n.checked_mul(1024))
        .flatten()
}

/// The values of `keys` in `file`'s `key value` lines (a trailing `:` on
/// the key is dropped), in `keys` order, each read by `value`; other
/// lines are skipped.
fn lookup<const N: usize>(
    text: &str,
    file: &'static str,
    keys: [&'static str; N],
    value: fn(&str) -> Option<u64>,
) -> Result<[u64; N], MemError> {
    let mut found = [None; N];
    for line in text.lines() {
        let (key, rest) = split_key(line);
        let Some(slot) = keys.iter().position(|k| *k == key) else {
            continue;
        };
        let got = value(rest).ok_or_else(|| MemError::malformed(file, line))?;
        found.get_mut(slot).map(|s| s.replace(got));
    }
    let mut values = [0; N];
    for ((out, key), got) in values.iter_mut().zip(keys).zip(found) {
        *out = got.ok_or(MemError::Missing { file, key })?;
    }
    Ok(values)
}

/// A line's first word without a trailing `:`, and the rest.
fn split_key(line: &str) -> (&str, &str) {
    let line = line.trim();
    let (key, rest) =
        line.split_once(char::is_whitespace).unwrap_or((line, ""));
    (key.strip_suffix(':').unwrap_or(key), rest)
}

/// A bare count, alone on the rest of its line.
fn count(text: &str) -> Option<u64> {
    let mut words = text.split_whitespace();
    let n = words.next()?.parse().ok()?;
    words.next().is_none().then_some(n)
}

#[cfg(test)]
mod tests;
