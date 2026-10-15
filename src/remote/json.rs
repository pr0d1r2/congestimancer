//! A small JSON reader for the remote reply: the crate takes no JSON
//! dependency (src/report:C), and the reply is one short line.

use std::iter::{Peekable, from_fn};
use std::str::Chars;

/// How deep arrays and objects may nest; the reply nests three deep, so
/// anything far past it is not a reply, and the reader's stack stays
/// small.
const MAX_DEPTH: u8 = 32;

/// One JSON value. Numbers keep their text: the reply's integers may pass
/// 2^53 and its fractions are shown, never computed with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Value {
    /// `null`.
    Null,
    /// `true` or `false`.
    Bool(bool),
    /// A number, as written.
    Number(String),
    /// A string, unescaped.
    Text(String),
    /// An array.
    List(Vec<Value>),
    /// An object, its members in order.
    Object(Vec<(String, Value)>),
}

impl Value {
    /// The member `key` of an object.
    pub(super) fn get(&self, key: &str) -> Option<&Self> {
        let Self::Object(members) = self else {
            return None;
        };
        members.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// A string's text.
    pub(super) fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    /// An array's items.
    pub(super) fn as_list(&self) -> Option<&[Self]> {
        match self {
            Self::List(items) => Some(items),
            _ => None,
        }
    }
}

/// Reads `text` as exactly one JSON value, whitespace around it allowed.
pub(super) fn parse(text: &str) -> Result<Value, String> {
    let mut reader = Reader {
        chars: text.chars().peekable(),
        depth: 0,
    };
    let value = reader.value()?;
    reader.skip_space();
    match reader.chars.next() {
        None => Ok(value),
        Some(c) => Err(format!("{c:?} after the value")),
    }
}

/// What a reader reads one part with.
type Read<'a, T> = fn(&mut Reader<'a>) -> Result<T, String>;

/// The text left to read, and how deep it is nested.
struct Reader<'a> {
    chars: Peekable<Chars<'a>>,
    depth: u8,
}

impl<'a> Reader<'a> {
    /// Skips JSON whitespace.
    fn skip_space(&mut self) {
        while self.chars.next_if(|c| " \t\n\r".contains(*c)).is_some() {}
    }

    /// One value of any kind.
    fn value(&mut self) -> Result<Value, String> {
        self.skip_space();
        match self.chars.peek() {
            Some('{') => self.nested(Self::object),
            Some('[') => self.nested(Self::list),
            Some('"') => self.text().map(Value::Text),
            Some('-' | '0'..='9') => self.number(),
            Some(_) => self.word(),
            None => Err("the reply ends early".to_owned()),
        }
    }

    /// An array or object, one level deeper.
    fn nested(&mut self, read: Read<'a, Value>) -> Result<Value, String> {
        let deeper = self.depth.checked_add(1).filter(|&d| d <= MAX_DEPTH);
        self.depth = deeper.ok_or("nested too deep")?;
        let value = read(self);
        self.depth = self.depth.saturating_sub(1);
        value
    }

    /// `null`, `true` or `false`.
    fn word(&mut self) -> Result<Value, String> {
        let next = || self.chars.next_if(char::is_ascii_alphabetic);
        let word: String = from_fn(next).collect();
        match word.as_str() {
            "null" => Ok(Value::Null),
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(format!("unexpected {word:?}")),
        }
    }

    /// A number, kept as written once it reads as one.
    fn number(&mut self) -> Result<Value, String> {
        let part = |c: &char| c.is_ascii_digit() || "+-.eE".contains(*c);
        let text: String = from_fn(|| self.chars.next_if(part)).collect();
        match text.parse::<f64>() {
            Ok(_) => Ok(Value::Number(text)),
            Err(_) => Err(format!("{text:?} is not a number")),
        }
    }

    /// Takes `want`, after any whitespace.
    fn expect(&mut self, want: char) -> Result<(), String> {
        self.skip_space();
        match self.chars.next() {
            Some(c) if c == want => Ok(()),
            Some(c) => Err(format!("expected {want:?}, found {c:?}")),
            None => Err(format!("expected {want:?}, found the end")),
        }
    }

    /// `[ value, ... ]`.
    fn list(&mut self) -> Result<Value, String> {
        self.expect('[')?;
        self.items(']', Self::value).map(Value::List)
    }

    /// `{ "key": value, ... }`.
    fn object(&mut self) -> Result<Value, String> {
        self.expect('{')?;
        self.items('}', Self::member).map(Value::Object)
    }

    /// `"key": value`.
    fn member(&mut self) -> Result<(String, Value), String> {
        self.skip_space();
        let key = self.text()?;
        self.expect(':')?;
        Ok((key, self.value()?))
    }

    /// Comma-separated items up to `close`, which may come at once.
    fn items<T>(
        &mut self,
        close: char,
        item: Read<'a, T>,
    ) -> Result<Vec<T>, String> {
        let mut out = Vec::new();
        self.skip_space();
        if self.chars.next_if_eq(&close).is_some() {
            return Ok(out);
        }
        loop {
            out.push(item(self)?);
            if !self.more(close)? {
                return Ok(out);
            }
        }
    }

    /// After an item: `,` means more, `close` means done.
    fn more(&mut self, close: char) -> Result<bool, String> {
        self.skip_space();
        match self.chars.next() {
            Some(',') => Ok(true),
            Some(c) if c == close => Ok(false),
            Some(c) => Err(format!("expected , or {close:?}, found {c:?}")),
            None => Err(format!("expected {close:?}, found the end")),
        }
    }

    /// A string, quotes and all.
    fn text(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            match self.chars.next() {
                Some('"') => return Ok(out),
                Some('\\') => out.push(self.escape()?),
                Some(c) if c < ' ' => {
                    return Err(format!("{c:?} inside a string"));
                }
                Some(c) => out.push(c),
                None => return Err("a string never ends".to_owned()),
            }
        }
    }

    /// The character after a `\`.
    fn escape(&mut self) -> Result<char, String> {
        match self.chars.next() {
            Some(c @ ('"' | '\\' | '/')) => Ok(c),
            Some('b') => Ok('\u{8}'),
            Some('f') => Ok('\u{c}'),
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some('u') => self.unicode(),
            other => Err(format!("bad escape \\{other:?}")),
        }
    }

    /// The character after `\u`: four hex digits, or a surrogate pair of
    /// two `\uXXXX` escapes.
    fn unicode(&mut self) -> Result<char, String> {
        let first = self.hex4()?;
        let mut units = vec![first];
        if (0xD800..0xDC00).contains(&first) {
            self.expect_raw('\\')?;
            self.expect_raw('u')?;
            units.push(self.hex4()?);
        }
        match char::decode_utf16(units).next() {
            Some(Ok(c)) => Ok(c),
            _ => Err("a lone surrogate in \\u".to_owned()),
        }
    }

    /// Takes `want` right here, no whitespace skipped.
    fn expect_raw(&mut self, want: char) -> Result<(), String> {
        match self.chars.next_if_eq(&want) {
            Some(_) => Ok(()),
            None => Err(format!("expected {want:?} in a string")),
        }
    }

    /// Four hex digits.
    fn hex4(&mut self) -> Result<u16, String> {
        let digits: String = self.chars.by_ref().take(4).collect();
        let hex =
            digits.len() == 4 && digits.chars().all(|c| c.is_ascii_hexdigit());
        u16::from_str_radix(&digits, 16)
            .ok()
            .filter(|_| hex)
            .ok_or_else(|| format!("bad \\u{digits}"))
    }
}

#[cfg(test)]
mod tests;
