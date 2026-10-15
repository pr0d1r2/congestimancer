//! [`Bus`]: a D-Bus connection over any byte stream: SASL `EXTERNAL`,
//! `Hello`, then calls matched to their replies (src/presence:R1).
//!
//! Generic over `Read + Write`, so tests drive it with an in-memory
//! duplex and only `crate::sys` touches the real socket.

use std::io::{Read, Write};

use super::error::BusError;
use super::wire::{Call, HEAD, Message, decode, encode, frame_len};

/// The longest auth line the bus may send: `OK <32 hex>` and change.
const MAX_LINE: usize = 512;

/// The call every connection makes before any other.
const HELLO: Call<'static> = Call {
    path: "/org/freedesktop/DBus",
    interface: "org.freedesktop.DBus",
    member: "Hello",
    destination: "org.freedesktop.DBus",
    args: &[],
};

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
    pub(super) fn open(mut stream: S, uid: u32) -> Result<Self, BusError> {
        stream.write_all(&auth_request(uid))?;
        let line = read_line(&mut stream)?;
        if !line.starts_with("OK ") {
            return Err(BusError::Auth(line));
        }
        stream.write_all(b"BEGIN\r\n")?;
        let mut bus = Self { stream, serial: 0 };
        bus.call(&HELLO)?;
        Ok(bus)
    }

    /// Sends `call` and reads until its reply; other messages are skipped.
    pub(super) fn call(
        &mut self,
        call: &Call<'_>,
    ) -> Result<Message, BusError> {
        self.serial = self.serial.wrapping_add(1).max(1);
        self.stream.write_all(&encode(call, self.serial)?)?;
        loop {
            let message = self.receive()?;
            if message.reply_serial == Some(self.serial) {
                return message.reply();
            }
        }
    }

    /// The next message off the stream.
    fn receive(&mut self) -> Result<Message, BusError> {
        let mut bytes = vec![0; HEAD];
        self.stream.read_exact(&mut bytes)?;
        bytes.resize(frame_len(&bytes)?, 0);
        let rest = bytes.get_mut(HEAD..).unwrap_or_default();
        self.stream.read_exact(rest)?;
        decode(&bytes)
    }
}

/// The client's opening: a nul, then `AUTH EXTERNAL` with `uid` in
/// decimal, hex encoded.
fn auth_request(uid: u32) -> Vec<u8> {
    let hex: String = uid
        .to_string()
        .bytes()
        .map(|digit| format!("{digit:02x}"))
        .collect();
    format!("\0AUTH EXTERNAL {hex}\r\n").into_bytes()
}

/// One `\r\n`-ended auth line from the bus, without its ending.
fn read_line(stream: &mut impl Read) -> Result<String, BusError> {
    let mut line = Vec::new();
    while !line.ends_with(b"\r\n") {
        if line.len() >= MAX_LINE {
            return Err(BusError::Malformed("auth line too long"));
        }
        let mut byte = [0];
        stream.read_exact(&mut byte)?;
        line.extend(byte);
    }
    line.truncate(line.len().saturating_sub(2));
    String::from_utf8(line).map_err(|_| BusError::Malformed("auth not UTF-8"))
}

#[cfg(test)]
mod tests;
