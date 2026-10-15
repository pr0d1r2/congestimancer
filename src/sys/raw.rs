//! Raw bytes from the OS: native-endian integers at an offset, and hex
//! text, the form captured fixtures keep raw bytes in (src/sys:V3, V7).

/// The `N` bytes at `at`, if `bytes` holds them.
fn field<const N: usize>(bytes: &[u8], at: usize) -> Option<[u8; N]> {
    let _ = (bytes, at);
    None
}

/// The native-endian `i32` at byte `at`.
pub(crate) fn ne_i32(bytes: &[u8], at: usize) -> Option<i32> {
    field(bytes, at).map(i32::from_ne_bytes)
}

/// The native-endian `u64` at byte `at`.
pub(crate) fn ne_u64(bytes: &[u8], at: usize) -> Option<u64> {
    field(bytes, at).map(u64::from_ne_bytes)
}

/// The native-endian `i64` at byte `at`.
pub(crate) fn ne_i64(bytes: &[u8], at: usize) -> Option<i64> {
    field(bytes, at).map(i64::from_ne_bytes)
}

/// The bytes a hex string spells; surrounding whitespace is ignored. `None`
/// for an odd length or a non-hex digit. Tests read fixtures with it.
#[cfg(test)]
pub(crate) fn decode_hex(text: &str) -> Option<Vec<u8>> {
    let _ = text;
    None
}

#[cfg(test)]
mod tests;
