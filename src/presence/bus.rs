//! [`Bus`]: a D-Bus connection over any byte stream: SASL `EXTERNAL`,
//! `Hello`, then calls matched to their replies (src/presence:R1).
//!
//! Generic over `Read + Write`, so tests drive it with an in-memory
//! duplex and only `crate::sys` touches the real socket.

use std::io::{Read, Write};

use super::error::BusError;
use super::wire::{Call, Message};

/// A connection that has authenticated and said `Hello`.
#[derive(Debug)]
pub(super) struct Bus<S> {
    /// The byte stream to the bus.
    stream: S,
    /// The serial of the last call sent.
    serial: u32,
}

impl<S: Read + Write> Bus<S> {
    /// Authenticates as `uid` over `stream` and says `Hello`.
    pub(super) fn open(stream: S, uid: u32) -> Result<Self, BusError> {
        let _ = (stream, uid);
        Err(BusError::Malformed("stub"))
    }

    /// Sends `call` and reads until its reply; other messages are skipped.
    pub(super) fn call(
        &mut self,
        call: &Call<'_>,
    ) -> Result<Message, BusError> {
        let _ = (call, &mut self.stream, self.serial);
        Err(BusError::Malformed("stub"))
    }
}

#[cfg(test)]
mod tests;
