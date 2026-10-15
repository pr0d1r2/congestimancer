//! [`GpuError`]: why a GPU file did not parse, or a GPU switch cannot be
//! gated on.

use std::fmt;

use crate::units::Count;

/// Why sysfs GPU text did not parse, or `--gpu-device` is unusable.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GpuError {
    /// The text is not in the form its file takes.
    Malformed {
        /// The file, such as `gpu_busy_percent`.
        file: &'static str,
        /// The text as read, trimmed.
        text: String,
    },
    /// `--gpu-device I` past the GPUs this machine has: a usage error,
    /// exit 64 (src/gpu:V3).
    UnknownDevice {
        /// The index as given.
        index: Count,
        /// How many GPUs there are.
        found: usize,
    },
}

impl GpuError {
    /// A [`GpuError::Malformed`] for `text` of `file`.
    pub(crate) fn malformed(file: &'static str, text: &str) -> Self {
        Self::Malformed {
            file,
            text: text.trim().to_owned(),
        }
    }
}

impl fmt::Display for GpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { file, text } => {
                write!(f, "malformed {file}: `{text}`")
            }
            Self::UnknownDevice { index, found } => write!(
                f,
                "--gpu-device {index}: no such GPU ({found} found, \
                 numbered from 0)"
            ),
        }
    }
}

impl std::error::Error for GpuError {}

#[cfg(test)]
mod tests;
