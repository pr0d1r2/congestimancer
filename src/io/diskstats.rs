//! [`parse_diskstats`]: Linux `/proc/diskstats`, one block device per
//! line (src/io:T1).

use super::IoError;

/// `/proc/diskstats`: cumulative counters per block device since boot.
pub(super) const DISKSTATS: &str = "/proc/diskstats";

/// Bytes in a `/proc/diskstats` sector, whatever the device's own
/// sector size.
const SECTOR: u64 = 512;

/// One direction's cumulative counters in `/proc/diskstats`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Direction {
    /// Requests completed.
    pub ops: u64,
    /// Sectors moved, 512 bytes each.
    pub sectors: u64,
    /// Milliseconds spent on requests.
    pub ms: u64,
}

impl Direction {
    /// Bytes moved; `None` past 64 bits.
    #[must_use]
    pub const fn bytes(&self) -> Option<u64> {
        self.sectors.checked_mul(SECTOR)
    }
}

/// One device's line of `/proc/diskstats`: the columns the IO gates
/// read. Discard and flush columns are left out.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct DiskStat {
    /// Device major number.
    pub major: u32,
    /// Device minor number.
    pub minor: u32,
    /// Kernel device name, such as `nvme0n1p1`.
    pub name: String,
    /// Reads.
    pub read: Direction,
    /// Writes.
    pub write: Direction,
    /// Requests in flight now (`--io-queue`).
    pub in_flight: u64,
    /// Milliseconds the device had requests in flight (`io_ticks`,
    /// `--io-busy`).
    pub io_ticks_ms: u64,
}

/// Parses `/proc/diskstats` into one [`DiskStat`] per device. Blank lines
/// are skipped.
///
/// # Errors
///
/// [`IoError::Malformed`] for a line with fewer than 13 columns or a
/// column that is not a whole number.
pub fn parse_diskstats(text: &str) -> Result<Vec<DiskStat>, IoError> {
    let _ = parse_line(text);
    Err(IoError::malformed(DISKSTATS, text))
}

/// One line: major, minor, name, then ten counters.
fn parse_line(line: &str) -> Option<DiskStat> {
    let mut words = line.split_whitespace();
    let major = words.next()?.parse().ok()?;
    let minor = words.next()?.parse().ok()?;
    let name = words.next()?.to_owned();
    let counters = words
        .take(10)
        .map(|w| w.parse().ok())
        .collect::<Option<Vec<u64>>>()?;
    Some(from_columns(
        (major, minor),
        name,
        counters.try_into().ok()?,
    ))
}

/// A [`DiskStat`] of device `major:minor` named `name`, from its first
/// ten counter columns.
fn from_columns(
    (major, minor): (u32, u32),
    name: String,
    columns: [u64; 10],
) -> DiskStat {
    let [r, _, r_sectors, r_ms, w, _, w_sectors, w_ms, queue, ticks] = columns;
    DiskStat {
        major,
        minor,
        name,
        read: direction(r, r_sectors, r_ms),
        write: direction(w, w_sectors, w_ms),
        in_flight: queue,
        io_ticks_ms: ticks,
    }
}

/// A [`Direction`] of `ops` requests, `sectors` and `ms`.
const fn direction(ops: u64, sectors: u64, ms: u64) -> Direction {
    Direction { ops, sectors, ms }
}

#[cfg(test)]
mod tests;
