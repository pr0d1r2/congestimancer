use std::ffi::OsStr;

use super::{Paint, Style};

#[test]
fn a_tty_without_no_color_gets_colour() {
    assert_eq!(Style::choose(true, None), Style::Ansi);
}

#[test]
fn a_pipe_stays_plain() {
    assert_eq!(Style::choose(false, None), Style::Plain);
}

#[test]
fn no_color_turns_colour_off_even_on_a_tty() {
    assert_eq!(Style::choose(true, Some(OsStr::new("1"))), Style::Plain);
}

#[test]
fn an_empty_no_color_still_counts_as_set() {
    assert_eq!(Style::choose(true, Some(OsStr::new(""))), Style::Plain);
}

#[test]
fn plain_text_has_no_escape_codes() {
    assert_eq!(Style::Plain.paint("x", Paint::Red), "x");
}

#[test]
fn each_paint_wraps_text_in_its_code_and_a_reset() {
    let codes = [
        (Paint::Bold, "1"),
        (Paint::Dim, "2"),
        (Paint::Red, "31"),
        (Paint::Yellow, "33"),
    ];
    for (paint, code) in codes {
        let want = format!("\u{1b}[{code}mx\u{1b}[0m");
        assert_eq!(Style::Ansi.paint("x", paint), want);
    }
}
