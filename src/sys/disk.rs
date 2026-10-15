//! macOS block devices: which device holds a path ([`mount_source`]) and
//! the IOKit `IOBlockStorageDriver` counters of a physical disk
//! (src/sys:T7, V1, V3; src/io:R1, R2). Every other OS answers
//! [`io::ErrorKind::Unsupported`]; Linux reads `/proc/diskstats`.

use std::io;
use std::path::Path;

#[cfg(any(test, target_os = "macos"))]
use super::props::Props;

/// One direction's cumulative counters from an `IOBlockStorageDriver`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct BlockCounters {
    /// Bytes moved.
    pub bytes: u64,
    /// Operations done.
    pub operations: u64,
    /// Time spent in operations, in nanoseconds.
    pub total_time_ns: u64,
    /// Latency time, in nanoseconds; 0 where the driver does not count it.
    pub latency_time_ns: u64,
    /// Failed operations.
    pub errors: u64,
    /// Retried operations.
    pub retries: u64,
}

/// A physical disk's read and write counters since boot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct BlockStats {
    /// Reads.
    pub read: BlockCounters,
    /// Writes.
    pub write: BlockCounters,
}

/// One physical disk: its BSD name (`disk0`) and counters.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct BlockDevice {
    /// The whole-disk BSD name, such as `disk0`.
    pub name: String,
    /// Its counters.
    pub stats: BlockStats,
}

/// Where a path's filesystem is mounted from.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct MountSource {
    /// The mounted device, such as `/dev/disk3s1s1`.
    pub device: String,
    /// The filesystem type, such as `apfs`.
    pub fs_type: String,
}

/// The device and filesystem type holding `path` (`statfs`).
///
/// # Errors
///
/// The OS error for a path that cannot be read; Unsupported off macOS.
pub fn mount_source(path: &Path) -> io::Result<MountSource> {
    imp::mount_source(path)
}

/// The counters of the physical disk under BSD device `bsd_name` (`disk3s1`
/// or `/dev/disk3s1`): IOKit service-plane parents are walked until an
/// `IOBlockStorageDriver`, so an APFS volume reports its physical disk.
///
/// # Errors
///
/// Unsupported for an unknown device, one with no block storage driver
/// above it, or off macOS.
pub fn physical_disk_stats(bsd_name: &str) -> io::Result<BlockStats> {
    imp::physical_disk_stats(bsd_name)
}

/// Every physical disk with its counters.
///
/// # Errors
///
/// The IOKit error; Unsupported off macOS.
pub fn block_devices() -> io::Result<Vec<BlockDevice>> {
    imp::block_devices()
}

/// One direction of a `Statistics` table: `Bytes (Read)`, ... `Retries
/// (Read)`. Bytes, operations and total time are required.
#[cfg(any(test, target_os = "macos"))]
fn decode_counters(stats: &Props, direction: &str) -> Option<BlockCounters> {
    let _ = (stats, direction);
    None
}

/// An `IOBlockStorageDriver` `Statistics` table.
#[cfg(any(test, target_os = "macos"))]
fn decode_block_stats(stats: &Props) -> Option<BlockStats> {
    Some(BlockStats {
        read: decode_counters(stats, "Read")?,
        write: decode_counters(stats, "Write")?,
    })
}

/// The first of `start` and its ancestors that `is_target` accepts,
/// walking at most `limit` steps up through `parent`.
#[cfg(any(test, target_os = "macos"))]
fn walk_up<T>(
    start: T,
    limit: usize,
    parent: impl Fn(&T) -> Option<T>,
    is_target: impl Fn(&T) -> bool,
) -> Option<T> {
    let _ = (start, limit, parent, is_target);
    None
}

/// A device path or name as its bare BSD name: `/dev/disk3s1` -> `disk3s1`.
#[cfg(any(test, target_os = "macos"))]
fn bsd_name_of(device: &str) -> &str {
    device
}

#[cfg(target_os = "macos")]
mod imp {
    use std::ffi::{CString, c_char};
    use std::io;
    use std::path::Path;

    use super::{
        BlockDevice, BlockStats, MountSource, bsd_name_of, decode_block_stats,
        walk_up,
    };
    use crate::sys::error::unsupported;
    use crate::sys::iokit::{Service, bsd_service, services};

    const DRIVER: &std::ffi::CStr = c"IOBlockStorageDriver";

    pub(super) fn mount_source(path: &Path) -> io::Result<MountSource> {
        let raw = rustix::fs::statfs(path)?;
        Ok(MountSource {
            device: c_text(&raw.f_mntfromname),
            fs_type: c_text(&raw.f_fstypename),
        })
    }

    /// A NUL-terminated C string in a fixed buffer, lossily UTF-8.
    pub(super) fn c_text(raw: &[c_char]) -> String {
        let bytes: Vec<u8> = raw
            .iter()
            .map(|c| c.to_ne_bytes()[0])
            .take_while(|b| *b != 0)
            .collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    pub(super) fn physical_disk_stats(
        bsd_name: &str,
    ) -> io::Result<BlockStats> {
        let name = bsd_name_of(bsd_name);
        let c_name = CString::new(name)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
        let media = bsd_service(&c_name)?;
        let driver =
            walk_up(media, 32, Service::parent, |s| s.conforms_to(DRIVER))
                .ok_or_else(|| {
                    unsupported(format_args!("{name}: no block storage driver"))
                })?;
        stats_of(&driver)
            .ok_or_else(|| unsupported(format_args!("{name}: no Statistics")))
    }

    fn stats_of(driver: &Service) -> Option<BlockStats> {
        decode_block_stats(&driver.props("Statistics")?)
    }

    pub(super) fn block_devices() -> io::Result<Vec<BlockDevice>> {
        let drivers = services(DRIVER)?;
        Ok(drivers.iter().filter_map(device_of).collect())
    }

    /// A driver's whole-disk name (its child media's `BSD Name`) and
    /// counters; `None` for a driver with no media, such as an empty
    /// card reader.
    fn device_of(driver: &Service) -> Option<BlockDevice> {
        let media = driver.child()?;
        let name = media.property("BSD Name")?;
        let name = name.downcast_ref::<objc2_core_foundation::CFString>()?;
        Some(BlockDevice {
            name: name.to_string(),
            stats: stats_of(driver)?,
        })
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use std::io;
    use std::path::Path;

    use super::{BlockDevice, BlockStats, MountSource};
    use crate::sys::error::unsupported;

    pub(super) fn mount_source(_: &Path) -> io::Result<MountSource> {
        Err(unsupported("statfs mount source: macOS only"))
    }

    pub(super) fn physical_disk_stats(_: &str) -> io::Result<BlockStats> {
        Err(unsupported("IOKit block storage: macOS only"))
    }

    pub(super) fn block_devices() -> io::Result<Vec<BlockDevice>> {
        Err(unsupported("IOKit block storage: macOS only"))
    }
}

#[cfg(test)]
mod tests;
