//! macOS mach host calls: per-CPU ticks (`host_processor_info`), VM
//! counters (`host_statistics64`) and the run queue
//! (`processor_set_statistics`). The FFI copies raw words out; pure
//! functions decode them (src/sys:T6, V1, V3). Every other OS answers
//! [`io::ErrorKind::Unsupported`].

use std::io;

use super::raw::{ne_i32, ne_u32, ne_u64};

/// One CPU's cumulative ticks per state since boot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct CpuTicks {
    /// Ticks in user mode.
    pub user: u64,
    /// Ticks in the kernel.
    pub system: u64,
    /// Ticks idle.
    pub idle: u64,
    /// Ticks in user mode at raised nice.
    pub nice: u64,
}

/// VM page counts (gauges) and cumulative page events (counters).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct VmStats {
    /// Free pages.
    pub free: u64,
    /// Active pages.
    pub active: u64,
    /// Inactive pages.
    pub inactive: u64,
    /// Wired pages.
    pub wired: u64,
    /// Pages held by the compressor.
    pub compressor: u64,
    /// Pages read in, since boot.
    pub pageins: u64,
    /// Pages written out, since boot.
    pub pageouts: u64,
    /// Pages swapped in, since boot.
    pub swapins: u64,
    /// Pages swapped out, since boot.
    pub swapouts: u64,
}

/// `PROCESSOR_SET_LOAD_INFO` of the default processor set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct PsetLoad {
    /// Tasks in the set.
    pub task_count: i32,
    /// Threads in the set.
    pub thread_count: i32,
    /// Runnable threads times 1000, a ~5 s average (src/cpu:R1).
    pub load_average: i32,
    /// The mach factor times 1000.
    pub mach_factor: i32,
}

/// Every CPU's ticks, in CPU order.
///
/// # Errors
///
/// [`io::ErrorKind::Unsupported`] off macOS; the mach error otherwise.
pub fn cpu_ticks() -> io::Result<Vec<CpuTicks>> {
    ffi::cpu_load().map(|raw| decode_cpu_ticks(&raw))
}

/// The host's VM statistics.
///
/// # Errors
///
/// As [`cpu_ticks`], or [`io::ErrorKind::InvalidData`] for a short reply.
pub fn vm_stats() -> io::Result<VmStats> {
    let raw = ffi::vm_info()?;
    decode_vm_stats(&raw).ok_or_else(|| short("host_statistics64", raw.len()))
}

/// The default processor set's load, the macOS run queue (src/cpu:R1).
///
/// # Errors
///
/// As [`vm_stats`].
pub fn pset_load() -> io::Result<PsetLoad> {
    let raw = ffi::pset_load()?;
    decode_pset_load(&raw)
        .ok_or_else(|| short("processor_set_statistics", raw.len()))
}

fn short(call: &str, len: usize) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{call}: short reply of {len} bytes"),
    )
}

/// `CPU_STATE_MAX` (4) `u32` ticks a CPU: user, system, idle, nice. A
/// trailing partial CPU is dropped.
fn decode_cpu_ticks(raw: &[u8]) -> Vec<CpuTicks> {
    let state = |cpu: &[u8], at| ne_u32(cpu, at).map_or(0, u64::from);
    raw.chunks_exact(16)
        .map(|cpu| CpuTicks {
            user: state(cpu, 0),
            system: state(cpu, 4),
            idle: state(cpu, 8),
            nice: state(cpu, 12),
        })
        .collect()
}

/// `struct vm_statistics64`, by its stable leading field offsets.
fn decode_vm_stats(raw: &[u8]) -> Option<VmStats> {
    let page = |at| ne_u32(raw, at).map(u64::from);
    let count = |at| ne_u64(raw, at);
    Some(VmStats {
        free: page(0)?,
        active: page(4)?,
        inactive: page(8)?,
        wired: page(12)?,
        compressor: page(128)?,
        pageins: count(32)?,
        pageouts: count(40)?,
        swapins: count(112)?,
        swapouts: count(120)?,
    })
}

/// `struct processor_set_load_info`: four `i32`.
fn decode_pset_load(raw: &[u8]) -> Option<PsetLoad> {
    Some(PsetLoad {
        task_count: ne_i32(raw, 0)?,
        thread_count: ne_i32(raw, 4)?,
        load_average: ne_i32(raw, 8)?,
        mach_factor: ne_i32(raw, 12)?,
    })
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code, reason = "mach host FFI (src/sys:C)")]
mod ffi {
    use std::io;
    use std::sync::OnceLock;

    use libc::{c_int, integer_t, kern_return_t, mach_msg_type_number_t};
    use libc::{mach_port_t, natural_t};

    unsafe extern "C" {
        fn processor_set_default(
            host: mach_port_t,
            default_set: *mut mach_port_t,
        ) -> kern_return_t;
        fn processor_set_statistics(
            pset: mach_port_t,
            flavor: c_int,
            info_out: *mut integer_t,
            info_out_count: *mut mach_msg_type_number_t,
        ) -> kern_return_t;
    }

    /// A failed mach call, named.
    fn mach_error(call: &str, kr: kern_return_t) -> io::Error {
        io::Error::other(format!("{call}: kern_return_t {kr}"))
    }

    /// `KERN_SUCCESS`, or the named error.
    fn check(call: &str, kr: kern_return_t) -> io::Result<()> {
        if kr == 0 {
            Ok(())
        } else {
            Err(mach_error(call, kr))
        }
    }

    /// This task's host port, asked once: each `mach_host_self` adds a
    /// send right, so asking every sample would leak them.
    fn host() -> mach_port_t {
        static HOST: OnceLock<mach_port_t> = OnceLock::new();
        // SAFETY: `mach_host_self` takes nothing and only returns a port.
        #[allow(deprecated, reason = "libc points at mach2; src/sys:R1")]
        *HOST.get_or_init(|| unsafe { libc::mach_host_self() })
    }

    /// The default processor set's name port, asked once for the same
    /// reason as [`host`]; a refusal is kept too.
    fn pset() -> io::Result<mach_port_t> {
        static PSET: OnceLock<Result<mach_port_t, kern_return_t>> =
            OnceLock::new();
        let got = PSET.get_or_init(|| {
            let mut port: mach_port_t = 0;
            // SAFETY: `port` is a valid out pointer for one port name.
            let kr = unsafe { processor_set_default(host(), &raw mut port) };
            if kr == 0 { Ok(port) } else { Err(kr) }
        });
        got.map_err(|kr| mach_error("processor_set_default", kr))
    }

    pub(super) fn pset_load() -> io::Result<Vec<u8>> {
        let mut info = [0 as integer_t; 4];
        let mut count = 4 as mach_msg_type_number_t;
        let pset = pset()?;
        // SAFETY: `info` holds `count` (4) integers, the size of
        // `PROCESSOR_SET_LOAD_INFO`; the kernel writes at most `count`.
        let kr = unsafe {
            processor_set_statistics(
                pset,
                libc::PROCESSOR_SET_LOAD_INFO,
                info.as_mut_ptr(),
                &raw mut count,
            )
        };
        words(&info, count, kr, "processor_set_statistics")
    }

    pub(super) fn vm_info() -> io::Result<Vec<u8>> {
        let mut info = [0 as integer_t; 64];
        let mut count = 64 as mach_msg_type_number_t;
        // SAFETY: `info` holds `count` (64) integers, more than any
        // `vm_statistics64` so far; the kernel writes at most `count`.
        let kr = unsafe {
            libc::host_statistics64(
                host(),
                libc::HOST_VM_INFO64,
                info.as_mut_ptr(),
                &raw mut count,
            )
        };
        words(&info, count, kr, "host_statistics64")
    }

    /// The first `count` words of `info` as bytes, or the mach error.
    fn words(
        info: &[integer_t],
        count: mach_msg_type_number_t,
        kr: kern_return_t,
        call: &str,
    ) -> io::Result<Vec<u8>> {
        check(call, kr)?;
        let count = usize::try_from(count).unwrap_or(usize::MAX);
        Ok(info
            .iter()
            .take(count)
            .flat_map(|w| w.to_ne_bytes())
            .collect())
    }

    pub(super) fn cpu_load() -> io::Result<Vec<u8>> {
        let (array, len) = processor_info()?;
        // SAFETY: `array` points at `len` initialised integers the kernel
        // mapped for us; they stay mapped until `release` below.
        let raw = unsafe { std::slice::from_raw_parts(array, len) };
        let bytes = raw.iter().flat_map(|w| w.to_ne_bytes()).collect();
        release(array, len);
        Ok(bytes)
    }

    /// `PROCESSOR_CPU_LOAD_INFO`: the mapped array and its length.
    fn processor_info() -> io::Result<(*mut integer_t, usize)> {
        let mut cpus: natural_t = 0;
        let mut array: *mut integer_t = std::ptr::null_mut();
        let mut count: mach_msg_type_number_t = 0;
        // SAFETY: all three are valid out pointers; on success the kernel
        // maps a fresh array of `count` integers into this task at `array`.
        let kr = unsafe {
            libc::host_processor_info(
                host(),
                libc::PROCESSOR_CPU_LOAD_INFO,
                &raw mut cpus,
                &raw mut array,
                &raw mut count,
            )
        };
        check("host_processor_info", kr)?;
        Ok((array, usize::try_from(count).unwrap_or(0)))
    }

    /// Unmaps the array `host_processor_info` mapped into this task.
    fn release(array: *mut integer_t, len: usize) {
        let size = len.saturating_mul(size_of::<integer_t>());
        // SAFETY: `array` and `size` are exactly the region the kernel
        // mapped for us, and nothing reads it after this call.
        #[allow(deprecated, reason = "libc points at mach2; src/sys:R1")]
        let _ = unsafe {
            libc::vm_deallocate(libc::mach_task_self(), array as usize, size)
        };
    }
}

#[cfg(not(target_os = "macos"))]
mod ffi {
    use std::io;

    fn macos_only(call: &str) -> io::Error {
        super::super::error::unsupported(format_args!("{call}: macOS only"))
    }

    pub(super) fn cpu_load() -> io::Result<Vec<u8>> {
        Err(macos_only("host_processor_info"))
    }

    pub(super) fn vm_info() -> io::Result<Vec<u8>> {
        Err(macos_only("host_statistics64"))
    }

    pub(super) fn pset_load() -> io::Result<Vec<u8>> {
        Err(macos_only("processor_set_statistics"))
    }
}

#[cfg(test)]
mod tests;
