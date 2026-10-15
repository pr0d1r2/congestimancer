//! A small JSON reader for the remote reply: the crate takes no JSON
//! dependency (src/report:C), and the reply is one short line.

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
        let _ = key;
        None
    }

    /// A string's text.
    pub(super) const fn as_text(&self) -> Option<&str> {
        None
    }

    /// An array's items.
    pub(super) const fn as_list(&self) -> Option<&[Self]> {
        None
    }
}

/// Reads `text` as exactly one JSON value, whitespace around it allowed.
pub(super) fn parse(text: &str) -> Result<Value, String> {
    let _ = text;
    Err(String::new())
}

#[cfg(test)]
mod tests;
