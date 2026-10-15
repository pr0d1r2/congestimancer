//! NVIDIA GPUs through NVML, loaded at run time with `dlopen` so the
//! binary never links it (src/sys:T11, R3, V1, V4; src/gpu:C). Built
//! without the `nvml` feature, or with no `libnvidia-ml` on the machine,
//! every call is [`io::ErrorKind::Unsupported`].

use std::io;

/// One NVIDIA GPU's load.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct NvmlGpu {
    /// The NVML device index.
    pub index: u32,
    /// GPU busy percent over the last sample period; `None` if the device
    /// does not report it.
    pub utilization_percent: Option<u32>,
    /// Memory total, in bytes.
    pub memory_total: Option<u64>,
    /// Memory free, in bytes.
    pub memory_free: Option<u64>,
}

/// Every NVIDIA GPU NVML sees.
///
/// # Errors
///
/// Unsupported when built without `nvml`, when `libnvidia-ml` is absent or
/// its driver is not loaded; any other NVML failure as an error naming the
/// call.
pub fn nvml_gpus() -> io::Result<Vec<NvmlGpu>> {
    imp::gpus()
}

/// An NVML status as an error: driver or library absent and "not
/// supported" are Unsupported (src/sys:V4); the rest name the call.
#[cfg(any(test, feature = "nvml"))]
fn nvml_error(call: &str, status: i32) -> io::Error {
    /// `NVML_ERROR_NOT_SUPPORTED`, `_DRIVER_NOT_LOADED`,
    /// `_LIBRARY_NOT_FOUND`, `_FUNCTION_NOT_FOUND`.
    const ABSENT: [i32; 4] = [3, 9, 12, 13];
    let msg = format!("{call}: NVML status {status}");
    if ABSENT.contains(&status) {
        super::error::unsupported(msg)
    } else {
        io::Error::other(msg)
    }
}

#[cfg(feature = "nvml")]
#[allow(unsafe_code, reason = "NVML through dlopen (src/sys:C, R3)")]
mod imp {
    use std::ffi::{CStr, c_int, c_uint, c_void};
    use std::io;

    use super::{NvmlGpu, nvml_error};
    use crate::sys::error::unsupported;

    type Device = *mut c_void;
    type Init = unsafe extern "C" fn() -> c_int;
    type Count = unsafe extern "C" fn(*mut c_uint) -> c_int;
    type Handle = unsafe extern "C" fn(c_uint, *mut Device) -> c_int;
    type Utilization = unsafe extern "C" fn(Device, *mut [c_uint; 2]) -> c_int;
    type Memory = unsafe extern "C" fn(Device, *mut [u64; 3]) -> c_int;

    /// Where the library may be; the unversioned name is the dev symlink.
    const NAMES: [&CStr; 2] = [c"libnvidia-ml.so.1", c"libnvidia-ml.so"];

    /// An open `dlopen` handle, closed when dropped.
    struct Library(*mut c_void);

    impl Drop for Library {
        fn drop(&mut self) {
            // SAFETY: the handle came from a successful `dlopen` and no
            // symbol from it is used after the owning `Nvml` is gone.
            let _ = unsafe { libc::dlclose(self.0) };
        }
    }

    impl Library {
        fn open() -> io::Result<Self> {
            NAMES
                .iter()
                .find_map(|name| {
                    // SAFETY: `name` is NUL-terminated; dlopen only reads it.
                    let handle = unsafe {
                        libc::dlopen(
                            name.as_ptr(),
                            libc::RTLD_NOW | libc::RTLD_LOCAL,
                        )
                    };
                    (!handle.is_null()).then_some(Self(handle))
                })
                .ok_or_else(|| {
                    unsupported("NVML: no libnvidia-ml on this machine")
                })
        }

        /// `name` as a function pointer of type `F`.
        ///
        /// # Safety
        ///
        /// `F` must be the `extern "C"` function pointer type of `name`'s C
        /// signature.
        unsafe fn function<F: Copy>(&self, name: &CStr) -> io::Result<F> {
            const { assert!(size_of::<F>() == size_of::<*mut c_void>()) };
            let sym = self.symbol(name)?;
            // SAFETY: the sizes match (checked above) and the caller
            // vouches that `F` is this symbol's function pointer type.
            Ok(unsafe { std::mem::transmute_copy::<*mut c_void, F>(&sym) })
        }

        /// The address of `name`.
        fn symbol(&self, name: &CStr) -> io::Result<*mut c_void> {
            // SAFETY: `self.0` is open and `name` NUL-terminated.
            let sym = unsafe { libc::dlsym(self.0, name.as_ptr()) };
            if sym.is_null() {
                let name = name.to_string_lossy();
                return Err(unsupported(format_args!("NVML: no {name}")));
            }
            Ok(sym)
        }
    }

    /// The NVML entry points this node calls, initialised; shut down and
    /// closed when dropped.
    struct Nvml {
        shutdown: Init,
        count: Count,
        handle: Handle,
        utilization: Utilization,
        memory: Memory,
        _lib: Library,
    }

    impl Drop for Nvml {
        fn drop(&mut self) {
            // SAFETY: NVML was initialised by `load`; shutdown pairs it.
            let _ = unsafe { (self.shutdown)() };
        }
    }

    /// An NVML entry point's status as a result.
    fn call(name: &str, status: c_int) -> io::Result<()> {
        if status == 0 {
            Ok(())
        } else {
            Err(nvml_error(name, status))
        }
    }

    fn load() -> io::Result<Nvml> {
        let lib = Library::open()?;
        // SAFETY: `Init` spells `nvmlReturn_t nvmlInit_v2(void)`.
        let init: Init = unsafe { lib.function(c"nvmlInit_v2")? };
        let nvml = entry_points(lib)?;
        // SAFETY: the library stays open inside `nvml`; init takes nothing.
        call("nvmlInit_v2", unsafe { init() })?;
        Ok(nvml)
    }

    fn entry_points(lib: Library) -> io::Result<Nvml> {
        // SAFETY: each type alias above spells the C signature of the NVML
        // function it is loaded from.
        unsafe {
            Ok(Nvml {
                shutdown: lib.function(c"nvmlShutdown")?,
                count: lib.function(c"nvmlDeviceGetCount_v2")?,
                handle: lib.function(c"nvmlDeviceGetHandleByIndex_v2")?,
                utilization: lib.function(c"nvmlDeviceGetUtilizationRates")?,
                memory: lib.function(c"nvmlDeviceGetMemoryInfo")?,
                _lib: lib,
            })
        }
    }

    pub(super) fn gpus() -> io::Result<Vec<NvmlGpu>> {
        let nvml = load()?;
        let mut count: c_uint = 0;
        // SAFETY: `count` is a valid out pointer.
        let status = unsafe { (nvml.count)(&raw mut count) };
        call("nvmlDeviceGetCount_v2", status)?;
        Ok((0..count).filter_map(|index| gpu(&nvml, index)).collect())
    }

    /// One device's readings; `None` if NVML has no handle for `index`.
    fn gpu(nvml: &Nvml, index: c_uint) -> Option<NvmlGpu> {
        let mut device: Device = std::ptr::null_mut();
        // SAFETY: `device` is a valid out pointer for one handle.
        let status = unsafe { (nvml.handle)(index, &raw mut device) };
        (status == 0).then(|| readings(nvml, index, device))
    }

    fn readings(nvml: &Nvml, index: c_uint, device: Device) -> NvmlGpu {
        let (mut util, mut mem) = ([0 as c_uint; 2], [0_u64; 3]);
        // SAFETY: `device` is a live handle; `util` is `nvmlUtilization_t`
        // (gpu, memory) and `mem` is `nvmlMemory_t` (total, free, used).
        let (u, m) = unsafe {
            let u = (nvml.utilization)(device, &raw mut util);
            (u, (nvml.memory)(device, &raw mut mem))
        };
        let ([busy, _], [total, free, _]) = (util, mem);
        NvmlGpu {
            index,
            utilization_percent: (u == 0).then_some(busy),
            memory_total: (m == 0).then_some(total),
            memory_free: (m == 0).then_some(free),
        }
    }
}

#[cfg(not(feature = "nvml"))]
mod imp {
    use std::io;

    use super::NvmlGpu;

    pub(super) fn gpus() -> io::Result<Vec<NvmlGpu>> {
        Err(crate::sys::error::unsupported("NVML: built without nvml"))
    }
}

#[cfg(test)]
mod tests;
