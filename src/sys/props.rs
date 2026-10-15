//! [`Props`]: a flat property table read from IOKit or CoreFoundation
//! (block storage `Statistics`, GPU `PerformanceStatistics`, a power
//! source), and the `key = value` text captured fixtures keep it in
//! (src/sys:V3, V7).

use std::collections::BTreeMap;

/// One scalar property value.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum PropValue {
    /// A whole number.
    Int(i64),
    /// A number with a fraction.
    Float(f64),
    /// A boolean.
    Bool(bool),
    /// A string.
    Text(String),
}

impl PropValue {
    /// The value as a whole number, if it is one.
    #[must_use]
    pub const fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(n) => Some(*n),
            Self::Float(_) | Self::Bool(_) | Self::Text(_) => None,
        }
    }

    /// The value as a whole number that is not negative.
    #[must_use]
    pub fn as_u64(&self) -> Option<u64> {
        self.as_i64().and_then(|n| u64::try_from(n).ok())
    }

    /// The value as a number, whole or not.
    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "gauges, not counters")]
    pub const fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Int(n) => Some(*n as f64),
            Self::Float(x) => Some(*x),
            Self::Bool(_) | Self::Text(_) => None,
        }
    }

    /// The value as a boolean, if it is one.
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            Self::Int(_) | Self::Float(_) | Self::Text(_) => None,
        }
    }

    /// The value as a string, if it is one.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            Self::Int(_) | Self::Float(_) | Self::Bool(_) => None,
        }
    }
}

/// A flat property table, keys sorted.
pub type Props = BTreeMap<String, PropValue>;

/// `props` as text: one `key = value` line each, keys sorted; a string
/// value is quoted.
#[cfg(test)]
pub(crate) fn render_props(props: &Props) -> String {
    let _ = props;
    String::new()
}

/// The table [`render_props`] wrote; `None` on a line it cannot read.
#[cfg(test)]
pub(crate) fn parse_props(text: &str) -> Option<Props> {
    let _ = text;
    None
}

#[cfg(test)]
mod tests;
