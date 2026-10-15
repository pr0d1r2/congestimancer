//! [`Glob`]: a process or user name pattern, `*` for any run of
//! characters and `?` for one; every other character stands for itself
//! (src/procs:T1, V1, V2). Zero-dep (src/procs:C).

use std::fmt;

/// A name pattern: `*` matches any run of characters (none included),
/// `?` exactly one, anything else itself. Matching is case sensitive and
/// covers the whole name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Glob(String);

impl Glob {
    /// The pattern `pattern`; every string is a pattern.
    #[must_use]
    pub fn new(pattern: &str) -> Self {
        Self(pattern.to_owned())
    }

    /// The pattern as given.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether `name` matches the whole pattern.
    #[must_use]
    pub fn matches(&self, name: &str) -> bool {
        let _ = name;
        false
    }
}

/// Renders the pattern as given.
impl fmt::Display for Glob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests;
