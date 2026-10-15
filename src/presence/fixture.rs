//! Test helpers: hand-built wire bytes from `tests/fixtures/presence/`
//! and an in-memory duplex standing in for the bus socket (src:V4).

use std::io::{self, Cursor, Read, Write};
use std::path::Path;

/// The bytes of fixture `name`: hex pairs, `#` starts a comment.
pub(super) fn hex(name: &str) -> Vec<u8> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir.join("tests/fixtures/presence").join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    text.lines()
        .flat_map(|line| {
            let data = line.split('#').next().unwrap_or_default();
            data.split_whitespace()
                .filter_map(|pair| u8::from_str_radix(pair, 16).ok())
                .collect::<Vec<u8>>()
        })
        .collect()
}

/// A socket stand-in: reads give the scripted server bytes, writes are
/// kept for the test to compare.
#[derive(Debug, Default)]
pub(super) struct Duplex {
    /// What the server says.
    input: Cursor<Vec<u8>>,
    /// What the client said.
    pub(super) output: Vec<u8>,
}

impl Duplex {
    /// A duplex whose server says fixture `name`.
    pub(super) fn serving(name: &str) -> Self {
        Self::from_bytes(hex(name))
    }

    /// A duplex whose server says `bytes`.
    pub(super) fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            input: Cursor::new(bytes),
            output: Vec::new(),
        }
    }
}

impl Read for Duplex {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.input.read(buf)
    }
}

impl Write for Duplex {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
