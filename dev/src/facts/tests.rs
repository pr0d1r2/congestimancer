use super::{claim, dependencies, nixpkgs, platforms, quoted, rows};

const CARGO: &str = r#"[package]
name = "congestimancer"
rust-version = "1.95"

[dependencies]
# a comment
clap = { version = "4", features = [
  "std",
] }
libc = "0.2"

[target.'cfg(target_os = "macos")'.dependencies]
objc2-io-kit = "0.3"

[dev-dependencies]
proptest = "1"
"#;

#[test]
fn quoted_reads_a_string_value() {
    assert_eq!(quoted(CARGO, "name").as_deref(), Some("congestimancer"));
    assert_eq!(quoted(CARGO, "rust-version").as_deref(), Some("1.95"));
    assert_eq!(quoted(CARGO, "license"), None);
}

#[test]
fn dependencies_counts_direct_tables_only() {
    assert_eq!(dependencies(CARGO), 3);
    assert_eq!(dependencies("[package]\nname = \"x\"\n"), 0);
}

#[test]
fn claim_is_the_coverage_line_as_written() {
    let coverage = "# comment\nlines 97.3\n";
    assert_eq!(claim(coverage).as_deref(), Some("97.3"));
    assert_eq!(claim("lines \n"), None);
    assert_eq!(claim(""), None);
}

#[test]
fn rows_counts_numbered_rows_across_specs() {
    let a = "V1: x\nV2: y\nB1|2026|c|f\nVx: no\nV: no\n".to_owned();
    let b = "V1: z\nT1|.|t|-\n".to_owned();
    let specs = [a, b];
    assert_eq!(rows(&specs, 'V', ':'), 3);
    assert_eq!(rows(&specs, 'B', '|'), 1);
}

const LOCK: &str = r#"{
  "nodes": {
    "other": {
      "inputs": {
        "nixpkgs": [
          "nixpkgs-lock"
        ]
      }
    },
    "nixpkgs": {
      "locked": {
        "rev": "a033e66ab94a6bedf2c3d58d93a36319d3cbbbaf"
      },
      "original": {
        "ref": "nixos-26.05"
      }
    }
  }
}"#;

#[test]
fn nixpkgs_is_the_series_and_short_rev() {
    let got = nixpkgs(LOCK);
    assert_eq!(got, Some(("26.05".to_owned(), "a033e66".to_owned())));
    assert_eq!(nixpkgs("{}"), None);
}

#[test]
fn platforms_follow_the_ci_matrix_in_badge_order() {
    let ci = "    os: [macos-latest, ubuntu-24.04-arm, ubuntu-latest]\n";
    let want = vec![("intel", "linux"), ("arm", "linux"), ("arm", "macos")];
    assert_eq!(platforms(ci), Ok(want));
    assert_eq!(platforms("os: [macos-13]"), Ok(vec![("intel", "macos")]));
}

#[test]
fn an_unknown_runner_or_no_matrix_is_an_error() {
    assert!(platforms("os: [windows-latest]").is_err());
    assert!(platforms("jobs:\n").is_err());
}
