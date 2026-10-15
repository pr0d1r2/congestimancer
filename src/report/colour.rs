//! [`Style`]: whether a stream gets ANSI colour, decided from facts the
//! caller passes in, never from the environment here (src/report:T4, V3).

use std::ffi::OsStr;

/// How text on one stream is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    /// No escape codes.
    Plain,
    /// ANSI colour.
    Ansi,
}

/// A look [`Style::paint`] gives text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Paint {
    /// Bold, for headers.
    Bold,
    /// Faint, for values not there yet.
    Dim,
    /// Red, for a blocking gate.
    Red,
    /// Yellow, for an unsupported signal.
    Yellow,
}

impl Style {
    /// Colour only when the stream is a TTY and `NO_COLOR` is unset
    /// (src/report:V3); `no_color` is the variable's value, if set.
    #[must_use]
    pub fn choose(is_tty: bool, no_color: Option<&OsStr>) -> Self {
        if is_tty && no_color.is_none() {
            Self::Ansi
        } else {
            Self::Plain
        }
    }

    /// `text` drawn with `paint`, or as it is when plain.
    #[must_use]
    pub fn paint(self, text: &str, paint: Paint) -> String {
        match self {
            Self::Plain => text.to_owned(),
            Self::Ansi => format!("\u{1b}[{}m{text}\u{1b}[0m", paint.code()),
        }
    }
}

impl Paint {
    /// The SGR code.
    const fn code(self) -> &'static str {
        match self {
            Self::Bold => "1",
            Self::Dim => "2",
            Self::Red => "31",
            Self::Yellow => "33",
        }
    }
}

#[cfg(test)]
mod tests;
