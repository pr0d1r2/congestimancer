//! The D-Bus wire codec, by hand: method calls out, replies in, only the
//! types logind's answers use (src/presence:R1, src:V4).
//!
//! Little endian only: every platform this crate builds for is, and a
//! peer answers in the byte order of its own machine. Alignment counts
//! from the message start; the fixed header is 16 bytes and the body
//! starts 8-aligned, so fields and body are each read from offset 0.

use super::error::BusError;

/// Bytes before the header fields: endianness, type, flags, version,
/// body length, serial, fields length.
pub(super) const HEAD: usize = 16;

/// The largest message read: logind's replies are a few KiB.
const MAX_MESSAGE: usize = 1 << 20;

/// Message type `METHOD_CALL`.
const METHOD_CALL: u8 = 1;
/// Message type `ERROR`.
const ERROR: u8 = 3;

/// Header field codes (D-Bus spec, "Header Fields").
const PATH: u8 = 1;
/// `INTERFACE`.
const INTERFACE: u8 = 2;
/// `MEMBER`.
const MEMBER: u8 = 3;
/// `ERROR_NAME`.
const ERROR_NAME: u8 = 4;
/// `REPLY_SERIAL`.
const REPLY_SERIAL: u8 = 5;
/// `DESTINATION`.
const DESTINATION: u8 = 6;
/// `SIGNATURE`.
const SIGNATURE: u8 = 8;

/// A message that ends before its own lengths say.
const TRUNCATED: BusError = BusError::Malformed("message cut short");

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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
    let mut body = Writer::default();
    for arg in call.args {
        body.string(arg)?;
    }
    let fields = header_fields(call)?;
    let mut out = Writer::default();
    out.buf.extend([b'l', METHOD_CALL, 0, 1]);
    out.u32(length(body.buf.len())?);
    out.u32(serial);
    out.u32(length(fields.len())?);
    out.buf.extend(fields);
    out.pad(8);
    out.buf.extend(body.buf);
    Ok(out.buf)
}

/// The header fields array of `call`, without its length.
fn header_fields(call: &Call<'_>) -> Result<Vec<u8>, BusError> {
    let mut fields = Writer::default();
    fields.field(PATH, "o", call.path)?;
    fields.field(INTERFACE, "s", call.interface)?;
    fields.field(MEMBER, "s", call.member)?;
    fields.field(DESTINATION, "s", call.destination)?;
    if !call.args.is_empty() {
        fields.field(SIGNATURE, "g", &"s".repeat(call.args.len()))?;
    }
    Ok(fields.buf)
}

/// A length as the `u32` the wire carries.
fn length(len: usize) -> Result<u32, BusError> {
    u32::try_from(len).map_err(|_| BusError::Malformed("message too long"))
}

/// The whole message length from its first [`HEAD`] bytes.
pub(super) fn frame_len(head: &[u8]) -> Result<usize, BusError> {
    let mut r = Reader::open(head)?;
    r.take(4)?;
    let body = r.size()?;
    r.u32()?;
    let fields = r.size()?.checked_next_multiple_of(8).ok_or(TRUNCATED)?;
    HEAD.checked_add(fields)
        .and_then(|len| len.checked_add(body))
        .filter(|&len| len <= MAX_MESSAGE)
        .ok_or(BusError::Malformed("message too long"))
}

/// The message in `bytes`.
pub(super) fn decode(bytes: &[u8]) -> Result<Message, BusError> {
    let mut r = Reader::open(bytes)?;
    let mut message = Message {
        kind: kind(&mut r)?,
        ..Message::default()
    };
    let (body_len, _serial) = (r.size()?, r.u32()?);
    let end = r.size()?.checked_add(HEAD).ok_or(TRUNCATED)?;
    while r.pos < end {
        message.field(&mut r)?;
    }
    r.align(8)?;
    message.body = r.take(body_len)?.to_vec();
    Ok(message)
}

/// The message type from the first four bytes, if version 1.
fn kind(r: &mut Reader<'_>) -> Result<u8, BusError> {
    let [_, kind, _, version] = r.array::<4>()?;
    if version != 1 {
        return Err(BusError::Malformed("D-Bus protocol version is not 1"));
    }
    Ok(kind)
}

impl Message {
    /// Reads one header field into this message.
    fn field(&mut self, r: &mut Reader<'_>) -> Result<(), BusError> {
        r.align(8)?;
        let code = r.u8()?;
        match (code, r.signature()?.as_str()) {
            (REPLY_SERIAL, "u") => self.reply_serial = Some(r.u32()?),
            (ERROR_NAME, "s") => self.error_name = Some(r.string()?),
            (SIGNATURE, "g") => self.signature = r.signature()?,
            (_, "s" | "o") => drop(r.string()?),
            (_, "g") => drop(r.signature()?),
            (_, "u") => drop(r.u32()?),
            _ => return Err(BusError::Malformed("header field of odd type")),
        }
        Ok(())
    }

    /// This message, or the remote error it carries.
    pub(super) fn reply(self) -> Result<Self, BusError> {
        if self.kind != ERROR {
            return Ok(self);
        }
        Err(BusError::Remote {
            message: self.string().unwrap_or_default(),
            name: self.error_name.unwrap_or_default(),
        })
    }

    /// A reader over the body, if its signature is `signature`.
    fn body(&self, signature: &str) -> Result<Reader<'_>, BusError> {
        if self.signature != signature {
            return Err(BusError::Malformed(
                "reply of an unexpected signature",
            ));
        }
        Ok(Reader::new(&self.body))
    }

    /// A body of one string: `Hello`'s unique name, an error's message.
    pub(super) fn string(&self) -> Result<String, BusError> {
        self.body("s")?.string()
    }

    /// The object paths of a `ListSessions` reply, `a(susso)`.
    pub(super) fn session_paths(&self) -> Result<Vec<String>, BusError> {
        let mut r = self.body("a(susso)")?;
        let len = r.size()?;
        r.align(8)?;
        let end = r.pos.checked_add(len).ok_or(TRUNCATED)?;
        let mut paths = Vec::new();
        while r.pos < end {
            r.align(8)?;
            let _ = (r.string()?, r.u32()?, r.string()?, r.string()?);
            paths.push(r.string()?);
        }
        Ok(paths)
    }

    /// The value of a `Properties.Get` reply, `v`.
    pub(super) fn variant(&self) -> Result<Value, BusError> {
        let mut r = self.body("v")?;
        match r.signature()?.as_str() {
            "s" => r.string().map(Value::Str),
            "b" => r.boolean().map(Value::Bool),
            "t" => r.u64().map(Value::U64),
            _ => Err(BusError::Malformed("property of an unexpected type")),
        }
    }
}

/// Reads aligned little-endian values off a byte slice.
#[derive(Debug)]
struct Reader<'a> {
    /// The bytes.
    buf: &'a [u8],
    /// The next byte to read.
    pos: usize,
}

impl<'a> Reader<'a> {
    /// A reader at the start of `buf`.
    const fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// A reader over a whole message, if it is little endian.
    fn open(buf: &'a [u8]) -> Result<Self, BusError> {
        match buf.first() {
            Some(b'l') => Ok(Self::new(buf)),
            Some(_) => Err(BusError::Malformed("message not little endian")),
            None => Err(TRUNCATED),
        }
    }

    /// The next `n` bytes.
    fn take(&mut self, n: usize) -> Result<&'a [u8], BusError> {
        let end = self.pos.checked_add(n).ok_or(TRUNCATED)?;
        let bytes = self.buf.get(self.pos..end).ok_or(TRUNCATED)?;
        self.pos = end;
        Ok(bytes)
    }

    /// The next `N` bytes as an array.
    fn array<const N: usize>(&mut self) -> Result<[u8; N], BusError> {
        self.take(N)?.try_into().map_err(|_| TRUNCATED)
    }

    /// Skips padding up to a multiple of `n`.
    fn align(&mut self, n: usize) -> Result<(), BusError> {
        let to = self.pos.checked_next_multiple_of(n).ok_or(TRUNCATED)?;
        let gap = to.saturating_sub(self.pos);
        self.take(gap).map(drop)
    }

    /// A `y`.
    fn u8(&mut self) -> Result<u8, BusError> {
        let [byte] = self.array()?;
        Ok(byte)
    }

    /// A `u`.
    fn u32(&mut self) -> Result<u32, BusError> {
        self.align(4)?;
        self.array().map(u32::from_le_bytes)
    }

    /// A `t`.
    fn u64(&mut self) -> Result<u64, BusError> {
        self.align(8)?;
        self.array().map(u64::from_le_bytes)
    }

    /// A `u` used as a length.
    fn size(&mut self) -> Result<usize, BusError> {
        usize::try_from(self.u32()?).map_err(|_| TRUNCATED)
    }

    /// A `b`: a `u` that is 0 or 1.
    fn boolean(&mut self) -> Result<bool, BusError> {
        match self.u32()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(BusError::Malformed("boolean neither 0 nor 1")),
        }
    }

    /// An `s` or `o`: a `u` length, the UTF-8 bytes, a nul.
    fn string(&mut self) -> Result<String, BusError> {
        let len = self.size()?;
        self.text(len)
    }

    /// A `g`: a `y` length, the bytes, a nul.
    fn signature(&mut self) -> Result<String, BusError> {
        let len = usize::from(self.u8()?);
        self.text(len)
    }

    /// `len` UTF-8 bytes and their nul.
    fn text(&mut self, len: usize) -> Result<String, BusError> {
        let bytes = self.take(len)?;
        if self.u8()? != 0 {
            return Err(BusError::Malformed("string without its nul"));
        }
        let text = std::str::from_utf8(bytes);
        text.map(str::to_owned)
            .map_err(|_| BusError::Malformed("string not UTF-8"))
    }
}

/// Writes aligned little-endian values.
#[derive(Debug, Default)]
struct Writer {
    /// The bytes so far.
    buf: Vec<u8>,
}

impl Writer {
    /// Zero padding up to a multiple of `n`.
    fn pad(&mut self, n: usize) {
        while !self.buf.len().is_multiple_of(n) {
            self.buf.push(0);
        }
    }

    /// A `u`.
    fn u32(&mut self, value: u32) {
        self.pad(4);
        self.buf.extend(value.to_le_bytes());
    }

    /// An `s` or `o`.
    fn string(&mut self, text: &str) -> Result<(), BusError> {
        self.u32(length(text.len())?);
        self.text(text);
        Ok(())
    }

    /// A `g`.
    fn signature(&mut self, text: &str) -> Result<(), BusError> {
        let len = u8::try_from(text.len())
            .map_err(|_| BusError::Malformed("signature too long"))?;
        self.buf.push(len);
        self.text(text);
        Ok(())
    }

    /// The bytes of `text` and a nul.
    fn text(&mut self, text: &str) {
        self.buf.extend(text.as_bytes());
        self.buf.push(0);
    }

    /// A header field `(yv)` whose value is an `s`, `o` or `g`.
    fn field(
        &mut self,
        code: u8,
        signature: &str,
        value: &str,
    ) -> Result<(), BusError> {
        self.pad(8);
        self.buf.push(code);
        self.signature(signature)?;
        if signature == "g" {
            self.signature(value)
        } else {
            self.string(value)
        }
    }
}

#[cfg(test)]
mod tests;
