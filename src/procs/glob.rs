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
        let pattern: Vec<char> = self.0.chars().collect();
        let name: Vec<char> = name.chars().collect();
        let mut run = Run {
            pattern: &pattern,
            text: &name,
            star: None,
        };
        loop {
            if let Some(matched) = run.step() {
                return matched;
            }
        }
    }
}

/// Renders the pattern as given.
impl fmt::Display for Glob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Where the last `*` was: the pattern after it, and the text it stopped
/// at.
#[derive(Clone, Copy)]
struct Star<'a> {
    /// The pattern after the `*`.
    after: &'a [char],
    /// The text the `*` has not taken.
    at: &'a [char],
}

/// One match in progress. Backtracks to the last `*` only: linear space,
/// at most `pattern * text` steps.
struct Run<'a> {
    /// The pattern left.
    pattern: &'a [char],
    /// The text left.
    text: &'a [char],
    /// The last `*` seen.
    star: Option<Star<'a>>,
}

impl<'a> Run<'a> {
    /// One step; `Some` once the answer is known.
    fn step(&mut self) -> Option<bool> {
        match (self.pattern.split_first(), self.text.split_first()) {
            (Some(('*', after)), _) => self.take_star(after),
            (Some((&want, p)), Some((&got, t)))
                if want == '?' || want == got =>
            {
                self.pattern = p;
                self.text = t;
            }
            (None, None) => return Some(true),
            _ => return self.backtrack(),
        }
        None
    }

    /// Remembers a `*` that takes nothing yet; `after` follows it.
    const fn take_star(&mut self, after: &'a [char]) {
        self.star = Some(Star {
            after,
            at: self.text,
        });
        self.pattern = after;
    }

    /// Lets the last `*` take one more character; `Some(false)` when
    /// there is no `*` or no text left for it.
    fn backtrack(&mut self) -> Option<bool> {
        let Some(Star { after, at }) = self.star else {
            return Some(false);
        };
        let Some((_, at)) = at.split_first() else {
            return Some(false);
        };
        self.star = Some(Star { after, at });
        self.pattern = after;
        self.text = at;
        None
    }
}

#[cfg(test)]
mod tests;
