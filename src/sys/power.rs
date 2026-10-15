//! macOS power and presence through IOKit: the power source in use and
//! the internal battery (`IOPS`), and user idle time (`IOHIDSystem`
//! `HIDIdleTime`) (src/sys:T8, V1, V3). Every other OS answers
//! [`io::ErrorKind::Unsupported`].

use std::io;

use super::props::{PropValue, Props};

/// What powers the machine now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct PowerState {
    /// On wall power (`AC Power`); false on battery or UPS.
    pub external: bool,
    /// The internal battery's charge, 0 to 100; `None` with no battery.
    pub battery_percent: Option<u32>,
    /// Whether the internal battery is charging; `None` with no battery.
    pub charging: Option<bool>,
}

/// The power source in use and the internal battery.
///
/// # Errors
///
/// Unsupported when IOKit has no power source information, or off macOS.
pub fn power_state() -> io::Result<PowerState> {
    let (providing, sources) = imp::power_sources()?;
    Ok(decode_power(providing.as_deref(), &sources))
}

/// The providing power source type and every source's raw description,
/// for [`super::capture`].
#[cfg(target_os = "macos")]
pub(crate) fn sources() -> io::Result<Sources> {
    imp::power_sources()
}

/// Time since the last keyboard or mouse input, in nanoseconds.
///
/// # Errors
///
/// Unsupported when there is no `IOHIDSystem` (such as over ssh into a
/// headless host), or off macOS.
pub fn hid_idle_ns() -> io::Result<u64> {
    imp::hid_idle_ns()
}

/// The providing power source type and every source's description.
type Sources = (Option<String>, Vec<Props>);

/// `IOPSGetProvidingPowerSourceType` and every source's description.
fn decode_power(providing: Option<&str>, sources: &[Props]) -> PowerState {
    let battery = sources.iter().find(|source| {
        source.get("Type").and_then(PropValue::as_str)
            == Some("InternalBattery")
    });
    PowerState {
        external: providing == Some("AC Power"),
        battery_percent: battery.and_then(percent),
        charging: battery
            .and_then(|b| b.get("Is Charging"))
            .and_then(PropValue::as_bool),
    }
}

/// Current over max capacity, as a whole percent up to 100.
fn percent(battery: &Props) -> Option<u32> {
    let current = battery.get("Current Capacity")?.as_u64()?;
    let max = battery.get("Max Capacity")?.as_u64()?;
    let percent = current.checked_mul(100)?.checked_div(max)?;
    u32::try_from(percent.min(100)).ok()
}

/// `HIDIdleTime` of an `IOHIDSystem` table.
#[cfg(any(test, target_os = "macos"))]
fn decode_idle(props: &Props) -> Option<u64> {
    props.get("HIDIdleTime")?.as_u64()
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code, reason = "IOKit power source FFI (src/sys:C)")]
mod imp {
    use std::io;

    use objc2_core_foundation::{CFArray, CFRetained, CFType};
    use objc2_io_kit::{
        IOPSCopyPowerSourcesInfo, IOPSCopyPowerSourcesList,
        IOPSGetPowerSourceDescription, IOPSGetProvidingPowerSourceType,
    };

    use super::{Sources, decode_idle};
    use crate::sys::error::unsupported;
    use crate::sys::iokit::{props_of, services, value_of};
    use crate::sys::props::Props;

    pub(super) fn power_sources() -> io::Result<Sources> {
        let blob = IOPSCopyPowerSourcesInfo()
            .ok_or_else(|| unsupported("IOPS: no power source information"))?;
        // SAFETY: `blob` is the live snapshot just copied; both calls only
        // read it and return objects we own or retain.
        let (list, providing) = unsafe {
            (
                IOPSCopyPowerSourcesList(Some(&blob)),
                IOPSGetProvidingPowerSourceType(Some(&blob)),
            )
        };
        let providing = providing.map(|p| p.to_string());
        let list = list.map(|l| entries(&l)).unwrap_or_default();
        let sources =
            list.iter().filter_map(|ps| describe(&blob, ps)).collect();
        Ok((providing, sources))
    }

    /// The power source handles in an `IOPSCopyPowerSourcesList` array.
    fn entries(list: &CFArray) -> Vec<CFRetained<CFType>> {
        // SAFETY: a CoreFoundation array holds CoreFoundation objects, so
        // reading its elements as `CFType` is sound.
        unsafe { list.cast_unchecked::<CFType>() }.to_vec()
    }

    /// One power source's description as a flat table.
    fn describe(blob: &CFType, ps: &CFType) -> Option<Props> {
        // SAFETY: `blob` is the snapshot `ps` came from, both alive here.
        let dict =
            unsafe { IOPSGetPowerSourceDescription(Some(blob), Some(ps)) };
        dict.map(|d| props_of(&d))
    }

    pub(super) fn hid_idle_ns() -> io::Result<u64> {
        let hid = services(c"IOHIDSystem")?;
        let idle = hid.first().and_then(|s| s.property("HIDIdleTime"));
        let idle = idle.and_then(|v| value_of(&v));
        let props = idle.map(|v| Props::from([("HIDIdleTime".to_owned(), v)]));
        props
            .as_ref()
            .and_then(decode_idle)
            .ok_or_else(|| unsupported("IOHIDSystem: no HIDIdleTime"))
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    use std::io;

    use super::Sources;
    use crate::sys::error::unsupported;

    pub(super) fn power_sources() -> io::Result<Sources> {
        Err(unsupported("IOKit power sources: macOS only"))
    }

    pub(super) fn hid_idle_ns() -> io::Result<u64> {
        Err(unsupported("IOKit HIDIdleTime: macOS only"))
    }
}

#[cfg(test)]
mod tests;
