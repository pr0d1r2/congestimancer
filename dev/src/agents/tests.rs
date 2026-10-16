use super::{run_verb, table};
use std::path::PathBuf;

#[test]
fn rows_become_the_table() {
    let rows = "a\tcommit\t**/*.rs\u{1F}Cargo.toml\tyes\nb\tpush\t\t-\n";
    let got = table(rows, 1, 2);
    let counts = "1 steps run on every commit and 2 on push";
    assert!(got.contains(counts), "{got}");
    let a = "| `a` | commit | `**/*.rs` `Cargo.toml` | yes |\n";
    assert!(got.contains(a), "{got}");
    let b = "| `b` | push | whole tree | - |\n<!-- END steps -->\n";
    assert!(got.ends_with(b), "{got}");
}

#[test]
fn the_verb_checks_the_real_agents_file() {
    // The repository itself: AGENTS.md must already match hk.pkl.
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    assert_eq!(run_verb(&repo, true), Ok(()));
}

#[test]
fn a_repository_without_hk_is_an_error() {
    let dir = std::env::temp_dir()
        .join(format!("cgmr-dev-agents-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    assert_eq!(run_verb(&dir, true).err().map(|e| e.0), Some(2));
}
