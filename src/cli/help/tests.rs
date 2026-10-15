use super::switch_help;
use crate::cli::args::command;
use crate::cli::table::spec;

/// Every verb `--help` lists.
const VERBS: [&str; 7] = [
    "run",
    "check",
    "status",
    "advise",
    "profiles",
    "completions",
    "man",
];

/// The `--help` text of `verb`, or of `cgmr` itself for `None`.
fn help(verb: Option<&str>) -> String {
    let mut cmd = command();
    let found = match verb {
        Some(name) => cmd.find_subcommand_mut(name).map(|c| c.render_help()),
        None => Some(cmd.render_help()),
    };
    found.map(|text| text.to_string()).unwrap_or_default()
}

#[test]
fn switch_help_names_the_environment_variable() {
    let load = spec("load").map(switch_help).unwrap_or_default();
    assert!(load.ends_with("[env: CGMR_LOAD]"), "{load}");
    let when = spec("when").map(switch_help).unwrap_or_default();
    assert!(!when.contains("env:"), "{when}");
}

#[test]
fn the_top_help_has_an_example_of_every_verb() {
    let top = help(None);
    let examples = top.split("Examples:").nth(1).unwrap_or_default();
    assert!(examples.contains("cgmr --load 2 -- make"), "{top}");
    for verb in VERBS.iter().skip(1) {
        assert!(examples.contains(&format!("cgmr {verb}")), "{verb}");
    }
}

#[test]
fn every_verb_help_has_examples() {
    for verb in VERBS {
        let text = help(Some(verb));
        assert!(text.contains("Examples:\n  cgmr"), "{verb}: {text}");
    }
}

#[test]
fn the_help_notes_the_exit_code_collision() {
    let top = help(None);
    assert!(top.contains("Exit code collision"), "{top}");
    for code in ["64", "69", "124"] {
        assert!(top.contains(&format!("  {code} ")), "{code}");
    }
    assert!(help(Some("run")).contains("exits 64, 69 or 124"));
}

#[test]
fn switch_help_lists_the_environment_names() {
    let run = help(Some("run"));
    assert!(run.contains("[env: CGMR_IO_BUSY]"), "{run}");
    assert!(run.contains("Waiting:"), "{run}");
}
