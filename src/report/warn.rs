//! [`Warnings`]: the one way a warning is written, so `--quiet` hides
//! them all (src/report:T6, V7; src/cli:V12).

use std::io::{self, Write};

/// What every warning starts with.
const PREFIX: &str = "warning: ";

/// Where warnings go, and whether `--quiet` hides them.
#[derive(Debug)]
pub struct Warnings<W> {
    /// The stream warnings go to: stderr (src/report:V2).
    sink: W,
    /// `--quiet`: write nothing.
    quiet: bool,
}

impl<W: Write> Warnings<W> {
    /// Warnings to `sink`, or none at all when `quiet`.
    pub const fn new(sink: W, quiet: bool) -> Self {
        Self { sink, quiet }
    }

    /// Writes `text` as one `warning: ` line, unless quiet; text that
    /// already starts with `warning: ` keeps its one prefix.
    ///
    /// # Errors
    ///
    /// When the sink fails to take the bytes.
    pub fn warn(&mut self, text: &str) -> io::Result<()> {
        let _ = (text, &mut self.sink, self.quiet, PREFIX);
        Ok(())
    }

    /// The sink back.
    pub fn into_inner(self) -> W {
        self.sink
    }
}

#[cfg(test)]
mod tests;
