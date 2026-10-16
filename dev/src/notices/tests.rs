use super::{claim, crates, run_verb, stale_prose, table};
use std::path::PathBuf;

const TREE: &str = "congestimancer v0.0.0 (/x/congestimancer)|MIT\n\
    memchr v2.8.3|Unlicense OR MIT\n\
    clap v4.6.7|MIT OR Apache-2.0\n\
    memchr v2.8.3|Unlicense OR MIT (*)\n\
    broken line\n";

#[test]
fn crates_come_from_cargo_tree_once_each_without_this_crate() {
    let got = crates(TREE);
    let names: Vec<&str> = got.iter().map(|(n, _, _)| n.as_str()).collect();
    assert_eq!(names, ["clap", "memchr"]);
}

#[test]
fn the_table_lists_each_crate_between_markers() {
    let got = table(&crates(TREE));
    assert!(got.starts_with("<!-- BEGIN crates: generated"), "{got}");
    assert!(
        got.contains("| memchr | 2.8.3 | Unlicense OR MIT |\n"),
        "{got}"
    );
    assert!(got.ends_with("<!-- END crates -->\n"), "{got}");
}

#[test]
fn the_prose_is_held_to_the_counts() {
    let set = crates(TREE);
    let doc = "cgmr has **1 direct dependencies**, which pull in\n2 crates.";
    assert_eq!(stale_prose(doc, 1, &set), None);
    let stale = stale_prose(doc, 3, &set).unwrap_or_default();
    assert!(stale.contains(&claim(3, 2)), "{stale}");
}

#[test]
fn the_verb_checks_the_real_page() {
    // The repository itself: its page must already match Cargo.lock.
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    assert_eq!(run_verb(&repo, true), Ok(()));
}

#[test]
fn a_repository_without_the_page_is_an_error() {
    let dir = std::env::temp_dir()
        .join(format!("cgmr-dev-notices-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    assert_eq!(run_verb(&dir, true).err().map(|e| e.0), Some(2));
}
