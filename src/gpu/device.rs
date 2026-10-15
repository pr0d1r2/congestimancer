//! [`Gpu`]: one GPU's load in the units its gates take, whatever the
//! source (src/gpu:V1, V2).

use crate::units::{Bytes, Percent};

/// One GPU's load: what `--gpu` and `--gpu-mem` compare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct Gpu {
    /// Utilisation now; `None` if the device does not report it.
    pub busy: Option<Percent>,
    /// Free GPU memory; `None` on unified memory or if not reported
    /// (src/gpu:V6).
    pub vram_free: Option<Bytes>,
}

impl Gpu {
    /// A GPU `busy` and with `vram_free` free.
    #[must_use]
    pub const fn new(busy: Option<Percent>, vram_free: Option<Bytes>) -> Self {
        Self { busy, vram_free }
    }
}

/// `n` whole percent, if `n` is at most 100.
pub(crate) fn whole_percent(n: u64) -> Option<Percent> {
    u16::try_from(n)
        .ok()
        .and_then(|n| n.checked_mul(100))
        .and_then(Percent::from_hundredths)
}

#[cfg(test)]
mod tests;
