//! [`Output`] and [`Streams`]: JSON and `status` go to stdout, all else
//! to stderr (src/report:T3, V2).

use std::io::Write;

/// A kind of text cgmr writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Output {
    /// The `status` table.
    Status,
    /// `status --json`, `check --json`.
    Json,
    /// `--verbose` block reasons.
    Verbose,
    /// The `--heartbeat` line while waiting.
    Heartbeat,
    /// A warning (`--quiet` hides it).
    Warning,
    /// An error.
    Error,
}

/// One of the two standard output streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stream {
    /// Standard output: what scripts capture.
    Stdout,
    /// Standard error: everything about the run.
    Stderr,
}

impl Output {
    /// The stream this kind of text goes to (src/report:V2).
    #[must_use]
    pub const fn stream(self) -> Stream {
        match self {
            Self::Status | Self::Json => Stream::Stdout,
            Self::Verbose | Self::Heartbeat | Self::Warning | Self::Error => {
                Stream::Stderr
            }
        }
    }
}

/// The two streams, each picked by the kind of text written.
#[derive(Debug)]
pub struct Streams<O, E> {
    /// Where stdout text goes.
    stdout: O,
    /// Where stderr text goes.
    stderr: E,
}

impl<O: Write, E: Write> Streams<O, E> {
    /// Routes to `stdout` and `stderr`.
    pub const fn new(stdout: O, stderr: E) -> Self {
        Self { stdout, stderr }
    }

    /// The stream `output` goes to.
    pub fn to(&mut self, output: Output) -> &mut dyn Write {
        match output.stream() {
            Stream::Stdout => &mut self.stdout,
            Stream::Stderr => &mut self.stderr,
        }
    }

    /// The two streams back.
    pub fn into_inner(self) -> (O, E) {
        (self.stdout, self.stderr)
    }
}

#[cfg(test)]
mod tests;
