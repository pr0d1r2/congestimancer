//! Dogfood (tests:V5): `cgmr check` on the machine running the tests. Each
//! gate this OS supports must give a verdict, 0 or 1, never a usage error
//! (64) or "unsupported" (69). Thresholds are loose, so on a quiet machine
//! most pass, but only the exit code's class is asserted.

use std::error::Error;

type Outcome = Result<(), Box<dyn Error>>;

/// Gates every supported OS reads from the kernel it runs on.
const COMMON: &[&[&str]] = &[
    &["--load", "99"],
    &["--all-load", "9999"],
    &["--run-queue", "99"],
    &["--cpu-idle", "0"],
    &["--mem", "1KB"],
    &["--all-mem", "1KB"],
    &["--needs-mem", "1KB"],
    &["--swap-rate", "1TB/s"],
    &["--disk", "1KB"],
    &["--inodes", "1"],
    &["--fds", "99"],
    &["--pids", "99"],
    &["--net", "1TB/s"],
    &["--between", "00:00-23:59"],
    &[
        "--when-file",
        concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"),
    ],
];

/// Gates only this OS has (docs/CONGESTION.md).
#[cfg(target_os = "macos")]
const NATIVE: &[&[&str]] = &[
    &["--mem-level", "warning"],
    &["--io", "1TB/s"],
    &["--io-busy", "99"],
    &["--io-latency", "1s"],
    &["--no-low-power"],
];

/// Gates only this OS has (docs/CONGESTION.md).
#[cfg(target_os = "linux")]
const NATIVE: &[&[&str]] = &[&["--steal", "99"], &["--dirty", "1TB"]];

/// `cgmr check` on the live machine: no fixture root, no `CGMR_*` from the
/// caller, no wait beyond one short second sample.
fn check(gate: &[&str]) -> Result<Option<i32>, Box<dyn Error>> {
    let mut cmd = assert_cmd::Command::cargo_bin("cgmr")?;
    for (name, _) in std::env::vars_os() {
        if name.to_string_lossy().starts_with("CGMR_") {
            cmd.env_remove(name);
        }
    }
    cmd.args(["check", "--interval", "100ms"]).args(gate);
    Ok(cmd.output()?.status.code())
}

/// The gates of `gates` whose check exits outside 0 and 1.
fn broken(gates: &[&[&str]]) -> Result<Vec<String>, Box<dyn Error>> {
    let mut found = Vec::new();
    for gate in gates {
        let code = check(gate)?;
        if !matches!(code, Some(0 | 1)) {
            found.push(format!("{}: exit {code:?}", gate.join(" ")));
        }
    }
    Ok(found)
}

#[test]
fn check_without_gates_passes() -> Outcome {
    assert_eq!(check(&[])?, Some(0));
    Ok(())
}

#[test]
fn every_common_gate_gives_a_verdict_here() -> Outcome {
    let found = broken(COMMON)?;
    assert!(found.is_empty(), "{}", found.join("\n"));
    Ok(())
}

#[test]
fn every_native_gate_gives_a_verdict_here() -> Outcome {
    let found = broken(NATIVE)?;
    assert!(found.is_empty(), "{}", found.join("\n"));
    Ok(())
}
