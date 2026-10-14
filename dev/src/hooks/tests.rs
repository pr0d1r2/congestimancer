use super::{changelog, commit_msg};
use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("cgmr-dev-hooks-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(dir.join("src"));
    git(&dir, &["init", "-q"]);
    write(&dir, "CHANGELOG.md", "# Changelog\n\n## [Unreleased]\n\n");
    git(&dir, &["add", "CHANGELOG.md"]);
    git(&dir, &["commit", "-qm", "chore: start"]);
    dir
}

fn git(repo: &Path, args: &[&str]) {
    let _ = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .current_dir(repo)
        .output();
}

fn write(repo: &Path, path: &str, text: &str) {
    let _ = std::fs::write(repo.join(path), text);
}

#[test]
fn commit_msg_judges_the_message_file() {
    let repo = scratch("msg");
    write(&repo, "good", "fix(dev): x\n\nWhy.\n");
    write(&repo, "bad", "fixed things\n");
    assert_eq!(commit_msg(&repo, Some("good")), Ok(()));
    assert_eq!(commit_msg(&repo, Some("bad")).err().map(|e| e.0), Some(1));
    assert_eq!(commit_msg(&repo, Some("none")).err().map(|e| e.0), Some(2));
}

#[test]
fn a_feat_on_src_needs_a_changelog_entry() {
    let repo = scratch("log");
    write(&repo, "src/a.rs", "pub fn a() {}\n");
    git(&repo, &["add", "src/a.rs"]);
    write(&repo, "msg", "feat(src): a\n\nWhy.\n");
    let refused = changelog(&repo, Some("msg")).err();
    assert_eq!(refused.map(|e| e.0), Some(1));
    write(
        &repo,
        "CHANGELOG.md",
        "# Changelog\n\n## [Unreleased]\n\n- a\n",
    );
    git(&repo, &["add", "CHANGELOG.md"]);
    assert_eq!(changelog(&repo, Some("msg")), Ok(()));
}

#[test]
fn a_docs_commit_owes_no_entry() {
    let repo = scratch("docs");
    write(&repo, "src/a.rs", "pub fn a() {}\n");
    git(&repo, &["add", "src/a.rs"]);
    write(&repo, "msg", "docs(src): a\n\nWhy.\n");
    assert_eq!(changelog(&repo, Some("msg")), Ok(()));
}
