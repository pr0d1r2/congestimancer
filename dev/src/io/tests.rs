use super::{hooks, message, read, run, show, update};
use std::path::{Path, PathBuf};
use std::process::Command;

/// A fresh git repository in the system temp directory.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("cgmr-dev-io-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(&dir);
    git(&dir, &["init", "-q"]);
    dir
}

fn git(repo: &Path, args: &[&str]) {
    let _ = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(args)
        .current_dir(repo)
        .output();
}

#[test]
fn run_returns_stdout_in_the_repository() {
    let repo = scratch("run-ok");
    let out = run(&repo, "git", &["rev-parse", "--is-inside-work-tree"]);
    assert_eq!(out, Ok("true\n".to_owned()));
}

#[test]
fn run_reports_a_failing_or_missing_program() {
    let repo = scratch("run-bad");
    let failed = run(&repo, "git", &["no-such-verb"]).err();
    let (code, why) = failed.unwrap_or_default();
    assert_eq!(code, 2);
    assert!(why.starts_with("git no-such-verb failed:"), "{why}");
    let missing = run(&repo, "cgmr-no-such-program", &[]).err();
    let (code, why) = missing.unwrap_or_default();
    assert_eq!(code, 2);
    assert!(why.starts_with("cannot run cgmr-no-such-program"), "{why}");
}

#[test]
fn show_is_empty_for_a_missing_object() {
    let repo = scratch("show");
    assert_eq!(show(&repo, "HEAD~1:CHANGELOG.md"), Ok(String::new()));
    let _ = std::fs::write(repo.join("f"), "x\n");
    git(&repo, &["add", "f"]);
    git(&repo, &["commit", "-qm", "c"]);
    assert_eq!(show(&repo, "HEAD:f"), Ok("x\n".to_owned()));
}

#[test]
fn message_reads_a_file_or_commit_editmsg() {
    let repo = scratch("message");
    let _ = std::fs::write(repo.join("m"), "feat: a\n");
    assert_eq!(message(&repo, Some("m")), Ok("feat: a\n".to_owned()));
    let _ = std::fs::write(repo.join(".git/COMMIT_EDITMSG"), "fix: b\n");
    assert_eq!(message(&repo, None), Ok("fix: b\n".to_owned()));
    let (code, why) = message(&repo, Some("gone")).err().unwrap_or_default();
    assert_eq!(code, 2);
    assert!(why.starts_with("no message file at"), "{why}");
}

#[test]
fn read_names_a_file_it_cannot_read() {
    let repo = scratch("read");
    let _ = std::fs::write(repo.join("f"), "x\n");
    assert_eq!(read(&repo, "f"), Ok("x\n".to_owned()));
    let (code, why) = read(&repo, "gone").err().unwrap_or_default();
    assert_eq!(code, 2);
    assert!(why.starts_with("cannot read gone"), "{why}");
}

#[test]
fn update_writes_or_reports_drift() {
    let repo = scratch("update");
    let _ = std::fs::write(repo.join("f"), "old\n");
    let drift = update(&repo, "f", "new\n", Some("readme")).err();
    let (code, why) = drift.unwrap_or_default();
    assert_eq!(code, 1);
    assert!(why.contains("-- readme`"), "{why}");
    assert_eq!(update(&repo, "f", "new\n", None), Ok(()));
    assert_eq!(read(&repo, "f"), Ok("new\n".to_owned()));
    assert_eq!(update(&repo, "f", "new\n", Some("readme")), Ok(()));
}

/// A hk.pkl with one step on commit and two on push.
const HK: &str = "hooks = new Mapping {
  [\"pre-commit\"] = new Dynamic { steps = new Mapping { [\"a\"] = 1 } }
  [\"pre-push\"] = new Dynamic {
    steps = new Mapping {
      [\"a\"] = 1
      [\"b\"] = 2
    }
  }
}
";

#[test]
fn hooks_counts_steps_with_pkl() {
    let repo = scratch("hooks");
    let _ = std::fs::write(repo.join("hk.pkl"), HK);
    assert_eq!(hooks(&repo), Ok((1, 2)));
    let _ = std::fs::write(repo.join("hk.pkl"), "hooks = 1\n");
    assert_eq!(hooks(&repo).err().map(|e| e.0), Some(2));
}
