use proptest::prelude::*;

use super::{Host, parse_host, shell_quote, ssh_argv};

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|&w| (*w).to_owned()).collect()
}

fn host(name: &str) -> Host {
    parse_host(name).unwrap_or_else(|err| unreachable!("{err}"))
}

/// What a POSIX shell makes of a run of single-quoted words: the text
/// between quotes, and `\x` outside them as `x`.
fn unquote(quoted: &str) -> Option<String> {
    let mut out = String::new();
    let mut inside = false;
    let mut chars = quoted.chars();
    while let Some(c) = chars.next() {
        match (inside, c) {
            (_, '\'') => inside = !inside,
            (true, c) => out.push(c),
            (false, '\\') => out.push(chars.next()?),
            (false, _) => return None,
        }
    }
    (!inside).then_some(out)
}

#[test]
fn plain_words_are_single_quoted() {
    assert_eq!(shell_quote("--load"), "'--load'");
    assert_eq!(shell_quote("2"), "'2'");
}

#[test]
fn an_empty_word_stays_one_word() {
    assert_eq!(shell_quote(""), "''");
}

#[test]
fn a_quote_closes_escapes_and_reopens() {
    assert_eq!(shell_quote("it's"), "'it'\\''s'");
}

#[test]
fn shell_syntax_is_inert() {
    let hostile = "x; rm -rf ~ $(id) `id` \"$HOME\" | & > <";
    assert_eq!(shell_quote(hostile), format!("'{hostile}'"));
}

#[test]
fn argv_runs_check_json_in_batch_mode() {
    let argv =
        ssh_argv(&host("box"), &words(&["--load", "2", "--when", "a b"]));
    let tail = words(&[
        "box", "cgmr", "check", "--json", "'--load'", "'2'", "'--when'",
        "'a b'",
    ]);
    assert_eq!(argv.get(..2), Some(&words(&["-o", "BatchMode=yes"])[..]));
    assert!(argv.ends_with(&tail), "{argv:?}");
}

#[test]
fn every_gate_word_is_its_own_argv_word() {
    let gates = words(&["--when", "true; touch /tmp/pwned"]);
    let argv = ssh_argv(&host("box"), &gates);
    let last = argv.last().map(String::as_str);
    assert_eq!(last, Some("'true; touch /tmp/pwned'"));
}

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn quoting_round_trips(word in "\\PC*") {
        prop_assert_eq!(unquote(&shell_quote(&word)), Some(word));
    }
}
