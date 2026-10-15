//! The D-Bus wire codec, by hand: method calls out, replies in, only the
//! types logind's answers use (src/presence:R1, src:V4).

use super::error::BusError;

/// Bytes before the header fields: endianness, type, flags, version,
/// body length, serial, fields length.
pub(super) const HEAD: usize = 16;

/// A method call whose arguments are all strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Call<'a> {
    /// The object path.
    pub(super) path: &'a str,
    /// The interface.
    pub(super) interface: &'a str,
    /// The method.
    pub(super) member: &'a str,
    /// The bus name it goes to.
    pub(super) destination: &'a str,
    /// The arguments, each a D-Bus string.
    pub(super) args: &'a [&'a str],
}

/// A property value, of the types logind's session properties take.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Value {
    /// `s`.
    Str(String),
    /// `b`.
    Bool(bool),
    /// `t`.
    U64(u64),
}

/// A message read off the bus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Message {
    /// The message type: 1 call, 2 return, 3 error, 4 signal.
    pub(super) kind: u8,
    /// The call this answers, for a return or an error.
    pub(super) reply_serial: Option<u32>,
    /// The error name, for an error.
    pub(super) error_name: Option<String>,
    /// The body signature, empty for no body.
    pub(super) signature: String,
    /// The body bytes.
    pub(super) body: Vec<u8>,
}

/// `call` as bytes, with `serial`.
pub(super) fn encode(
    call: &Call<'_>,
    serial: u32,
) -> Result<Vec<u8>, BusError> {
    let _ = (call, serial);
    Err(BusError::Malformed("stub"))
}

/// The whole message length from its first [`HEAD`] bytes.
pub(super) fn frame_len(head: &[u8]) -> Result<usize, BusError> {
    let _ = head;
    Err(BusError::Malformed("stub"))
}

/// The message in `bytes`.
pub(super) fn decode(bytes: &[u8]) -> Result<Message, BusError> {
    let _ = bytes;
    Err(BusError::Malformed("stub"))
}

impl Message {
    /// This message, or the remote error it carries.
    pub(super) fn reply(self) -> Result<Self, BusError> {
        Err(BusError::Malformed("stub"))
    }

    /// A body of one string: `Hello`'s unique name.
    pub(super) fn string(&self) -> Result<String, BusError> {
        Err(BusError::Malformed("stub"))
    }

    /// The object paths of a `ListSessions` reply, `a(susso)`.
    pub(super) fn session_paths(&self) -> Result<Vec<String>, BusError> {
        Err(BusError::Malformed("stub"))
    }

    /// The value of a `Properties.Get` reply, `v`.
    pub(super) fn variant(&self) -> Result<Value, BusError> {
        Err(BusError::Malformed("stub"))
    }
}

#[cfg(test)]
mod tests;
