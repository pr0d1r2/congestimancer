use super::run_verb;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A git repository holding every file the badges read.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("cgmr-dev-readme-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(dir.join(".github/workflows"));
    for (path, text) in FILES {
        let _ = std::fs::write(dir.join(path), text);
    }
    git(&dir, &["init", "-q"]);
    git(&dir, &["add", "."]);
    dir
}

fn git(repo: &Path, args: &[&str]) {
    let _ = Command::new("git").args(args).current_dir(repo).output();
}

const FILES: [(&str, &str); 7] = [
    (
        "Cargo.toml",
        "[package]\nedition = \"2024\"\nrust-version = \"1.95\"\n\
         license = \"MIT\"\nrepository = \"https://github.com/o/r\"\n\
         [dependencies]\nclap = \"4\"\n",
    ),
    (".coverage", "lines 97.3\n"),
    (
        "flake.lock",
        "\"nixpkgs\": {\n\"rev\": \"a033e66ab94a\",\n\"ref\": \"nixos-26.05\"\n",
    ),
    (".github/workflows/ci.yml", "os: [ubuntu-latest]\n"),
    ("SPEC.md", "V1: x\nV2: y\nB1|d|c|f\n"),
    (
        "hk.pkl",
        "hooks = new Mapping {\n\
         [\"pre-commit\"] = new Dynamic { steps = new Mapping { [\"a\"] = 1 } }\n\
         [\"pre-push\"] = new Dynamic { steps = new Mapping { [\"a\"] = 1 } }\n}\n",
    ),
    (
        "README.md",
        "# r\n\n<!-- BEGIN badges -->\n<!-- END badges -->\n\nbody\n",
    ),
];

fn readme(repo: &Path) -> String {
    std::fs::read_to_string(repo.join("README.md")).unwrap_or_default()
}

#[test]
fn the_badges_are_written_then_checked() {
    let repo = scratch("write");
    assert_eq!(run_verb(&repo, true).err().map(|e| e.0), Some(1));
    assert_eq!(run_verb(&repo, false), Ok(()));
    let got = readme(&repo);
    assert!(got.contains("invariants-2-6E4AFF"), "{got}");
    assert!(got.contains("gate_steps-1_commit_%2F_1_push"), "{got}");
    assert!(got.ends_with("<!-- END badges -->\n\nbody\n"), "{got}");
    assert_eq!(run_verb(&repo, true), Ok(()));
}

#[test]
fn a_readme_without_the_block_is_an_error() {
    let repo = scratch("no-block");
    let _ = std::fs::write(repo.join("README.md"), "# r\n");
    assert_eq!(run_verb(&repo, false).err().map(|e| e.0), Some(2));
}

#[test]
fn an_empty_source_is_refused() {
    let repo = scratch("empty");
    let _ = std::fs::write(repo.join(".coverage"), "# none\n");
    assert_eq!(run_verb(&repo, false).err().map(|e| e.0), Some(2));
    let _ = std::fs::write(repo.join(".coverage"), "lines 1\n");
    let _ = std::fs::write(repo.join("Cargo.toml"), "[package]\n");
    let err = run_verb(&repo, false).err().unwrap_or_default();
    assert!(err.1.contains("read empty or zero"), "{err:?}");
}
