//! Thin IOKit and CoreFoundation access for macOS: find services by
//! class or BSD name, walk the service plane, and copy a property table
//! out as [`Props`] (src/sys:I, R2, V1). Typed decoding lives with each
//! caller (`disk`, `power`, `gpu`).

use std::ffi::CStr;
use std::io;

use objc2_core_foundation::{
    CFBoolean, CFDictionary, CFNumber, CFRetained, CFString, CFType, Type,
};
use objc2_io_kit::{
    IOBSDNameMatching, IOIteratorNext, IOObjectConformsTo, IOObjectRelease,
    IORegistryEntryCreateCFProperty, IORegistryEntryGetChildEntry,
    IORegistryEntryGetParentEntry, IOServiceGetMatchingServices,
    IOServiceMatching, io_object_t,
};

use super::props::{PropValue, Props};

/// `kIOMainPortDefault`: port 0 asks IOKit for the default main port.
const MAIN_PORT: libc::mach_port_t = 0;

/// The registry plane parents and children are walked in.
const SERVICE_PLANE: &CStr = c"IOService";

/// One IOKit registry entry, released when dropped.
#[derive(Debug)]
pub(crate) struct Service(io_object_t);

impl Drop for Service {
    fn drop(&mut self) {
        let _ = IOObjectRelease(self.0);
    }
}

impl Service {
    /// Takes ownership of `object`; `None` for the null object.
    const fn new(object: io_object_t) -> Option<Self> {
        if object == 0 {
            None
        } else {
            Some(Self(object))
        }
    }

    /// Whether this entry is of class `class` or a subclass.
    pub(crate) fn conforms_to(&self, class: &CStr) -> bool {
        let mut name = plane_name(class);
        // SAFETY: `name` is a NUL-terminated `io_name_t` buffer that
        // outlives the call, which only reads it.
        unsafe { IOObjectConformsTo(self.0, &raw mut name) }
    }

    /// The property `key`, as IOKit stores it.
    pub(crate) fn property(&self, key: &str) -> Option<CFRetained<CFType>> {
        let key = CFString::from_str(key);
        // SAFETY: `self.0` is a live entry we hold a reference to; a null
        // allocator means the default one; options 0 asks for this entry.
        unsafe { IORegistryEntryCreateCFProperty(self.0, Some(&key), None, 0) }
    }

    /// The dictionary property `key` as a flat table; nested values are
    /// left out.
    pub(crate) fn props(&self, key: &str) -> Option<Props> {
        let value = self.property(key)?;
        value.downcast_ref::<CFDictionary>().map(props_of)
    }

    /// Every object left in this iterator.
    fn drain(&self) -> Vec<Self> {
        std::iter::from_fn(|| Self::new(IOIteratorNext(self.0))).collect()
    }

    /// This entry's parent in the service plane.
    pub(crate) fn parent(&self) -> Option<Self> {
        self.relative(IORegistryEntryGetParentEntry)
    }

    /// This entry's first child in the service plane.
    pub(crate) fn child(&self) -> Option<Self> {
        self.relative(IORegistryEntryGetChildEntry)
    }

    fn relative(
        &self,
        get: unsafe extern "C-unwind" fn(
            io_object_t,
            *mut [libc::c_char; 128],
            *mut io_object_t,
        ) -> libc::kern_return_t,
    ) -> Option<Self> {
        let mut plane = plane_name(SERVICE_PLANE);
        let mut out: io_object_t = 0;
        // SAFETY: `get` is IORegistryEntryGet{Parent,Child}Entry: it reads
        // the NUL-terminated `plane` and writes one object to `out`, which
        // we then own.
        let kr = unsafe { get(self.0, &raw mut plane, &raw mut out) };
        if kr == 0 { Self::new(out) } else { None }
    }
}

/// `text` in an `io_name_t` buffer, NUL-terminated, cut to fit.
fn plane_name(text: &CStr) -> [libc::c_char; 128] {
    let mut name = [0 as libc::c_char; 128];
    for (to, from) in name.iter_mut().zip(text.to_bytes().iter().take(127)) {
        *to = libc::c_char::from_ne_bytes([*from]);
    }
    name
}

/// Every service of class `class`, or a subclass.
pub(crate) fn services(class: &CStr) -> io::Result<Vec<Service>> {
    // SAFETY: `class` is NUL-terminated and outlives the call.
    let matching = unsafe { IOServiceMatching(class.as_ptr()) };
    let matching = matching.ok_or_else(|| no_matching(class))?;
    iterate(<CFDictionary as Type>::retain(&matching), class)
}

/// The service whose BSD name is `bsd_name`, such as `disk3s1`.
pub(crate) fn bsd_service(bsd_name: &CStr) -> io::Result<Service> {
    // SAFETY: `bsd_name` is NUL-terminated and outlives the call.
    let matching =
        unsafe { IOBSDNameMatching(MAIN_PORT, 0, bsd_name.as_ptr()) };
    let matching = matching.ok_or_else(|| no_matching(bsd_name))?;
    let found = iterate(<CFDictionary as Type>::retain(&matching), bsd_name)?;
    found.into_iter().next().ok_or_else(|| {
        super::error::unsupported(format_args!(
            "{}: no such BSD device",
            bsd_name.to_string_lossy()
        ))
    })
}

fn no_matching(what: &CStr) -> io::Error {
    let what = what.to_string_lossy();
    io::Error::other(format!("IOKit: cannot match {what}"))
}

/// Runs `matching` and collects every service it finds.
fn iterate(
    matching: CFRetained<CFDictionary>,
    what: &CStr,
) -> io::Result<Vec<Service>> {
    let mut iterator: io_object_t = 0;
    // SAFETY: the call consumes `matching`; `iterator` is a valid out
    // pointer whose object we own and release below.
    let kr = unsafe {
        IOServiceGetMatchingServices(
            MAIN_PORT,
            Some(matching),
            &raw mut iterator,
        )
    };
    if kr != 0 {
        return Err(no_matching(what));
    }
    Ok(Service::new(iterator)
        .map(|it| it.drain())
        .unwrap_or_default())
}

/// A CoreFoundation dictionary as a flat table: string keys, scalar
/// values; anything else is left out.
pub(crate) fn props_of(dict: &CFDictionary) -> Props {
    // SAFETY: a CoreFoundation dictionary holds CoreFoundation objects
    // as keys and values, so reading both as `CFType` is sound.
    let typed = unsafe { dict.cast_unchecked::<CFType, CFType>() };
    let (keys, values) = typed.to_vecs();
    keys.iter()
        .zip(values.iter())
        .filter_map(|(key, value)| {
            let key = key.downcast_ref::<CFString>()?.to_string();
            Some((key, value_of(value)?))
        })
        .collect()
}

/// One CoreFoundation scalar as a [`PropValue`].
pub(crate) fn value_of(value: &CFType) -> Option<PropValue> {
    let _ = (
        value.downcast_ref::<CFBoolean>(),
        value.downcast_ref::<CFNumber>(),
    );
    None
}

#[cfg(test)]
mod tests;
