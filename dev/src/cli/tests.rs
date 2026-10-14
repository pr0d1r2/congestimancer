use super::run;
use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("cgmr-dev-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(&dir);
    let _ = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&dir)
        .output();
    dir
}

fn args(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| (*w).to_owned()).collect()
}

fn write(repo: &Path, path: &str, text: &str) {
    let _ = std::fs::write(repo.join(path), text);
}

#[test]
fn unknown_or_malformed_verbs_are_usage_errors() {
    let repo = scratch("usage");
    assert_eq!(run(&args(&[]), &repo), 2);
    assert_eq!(run(&args(&["nope"]), &repo), 2);
    assert_eq!(run(&args(&["commit-msg", "a", "b"]), &repo), 2);
}

#[test]
fn verbs_route_to_their_rules() {
    let repo = scratch("route");
    write(&repo, "good", "docs: x\n\nWhy.\n");
    write(&repo, "bad", "x\n");
    assert_eq!(run(&args(&["commit-msg", "good"]), &repo), 0);
    assert_eq!(run(&args(&["commit-msg", "bad"]), &repo), 1);
    assert_eq!(run(&args(&["changelog", "good"]), &repo), 0);
}
