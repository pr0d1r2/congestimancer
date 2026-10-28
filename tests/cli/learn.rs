//! `--needs-mem auto` from the binary's side (src/slot:V13): a run keeps
//! the memory peak of its key in the lock dir, and a fixed size or a run
//! too short to sample keeps none. Each test has a lock dir of its own
//! under `XDG_RUNTIME_DIR`.

use std::error::Error;
use std::fs;
use std::path::Path;

use super::{Outcome, cgmr};

/// A job that outlives the meter's second sample (one second after the
/// first) by a wide margin, so a loaded machine still sees it running, and
/// exits 7.
const SLOW: [&str; 4] = ["--", "sh", "-c", "sleep 3; exit 7"];

/// `cgmr` on the fixture tree with its lock dir under `runtime`.
fn cgmr_in(runtime: &Path) -> Result<assert_cmd::Command, Box<dyn Error>> {
    let mut cmd = cgmr()?;
    cmd.env("XDG_RUNTIME_DIR", runtime);
    Ok(cmd)
}

/// The peak kept for `key` under `runtime`, if any.
fn kept(runtime: &Path, key: &str) -> Option<u64> {
    let file = runtime.join("cgmr").join(format!("+peak.{key}"));
    fs::read_to_string(file).ok()?.trim().parse().ok()
}

#[test]
fn a_run_keeps_the_peak_of_its_key_and_passes_its_exit_code_on() -> Outcome {
    let runtime = tempfile::tempdir()?;
    let run = cgmr_in(runtime.path())?
        .args(["--needs-mem", "auto:1KB", "--mem-key", "learn"])
        .args(SLOW)
        .assert();
    run.code(7);
    assert!(kept(runtime.path(), "learn").is_some_and(|peak| peak > 0));
    Ok(())
}

#[test]
fn the_key_is_the_commands_name_without_a_mem_key() -> Outcome {
    let runtime = tempfile::tempdir()?;
    let run = cgmr_in(runtime.path())?
        .args(["--needs-mem", "auto:1KB"])
        .args(SLOW)
        .assert();
    run.code(7);
    assert!(kept(runtime.path(), "sh").is_some());
    Ok(())
}

#[test]
fn a_fixed_size_learns_nothing() -> Outcome {
    let runtime = tempfile::tempdir()?;
    let run = cgmr_in(runtime.path())?
        .args(["--needs-mem", "1KB", "--mem-key", "fixed"])
        .args(SLOW)
        .assert();
    run.code(7);
    assert_eq!(kept(runtime.path(), "fixed"), None);
    Ok(())
}

#[test]
fn a_run_too_short_to_sample_learns_nothing() -> Outcome {
    let runtime = tempfile::tempdir()?;
    let run = cgmr_in(runtime.path())?
        .args(["--needs-mem", "auto:1KB", "--mem-key", "short"])
        .args(["--", "true"])
        .assert();
    run.code(0);
    assert_eq!(kept(runtime.path(), "short"), None);
    Ok(())
}

#[test]
fn a_bad_fallback_is_a_usage_error() -> Outcome {
    let runtime = tempfile::tempdir()?;
    let run = cgmr_in(runtime.path())?
        .args(["--needs-mem", "auto:lots", "--", "true"])
        .assert();
    run.code(64);
    Ok(())
}

#[test]
fn a_bad_mem_key_is_a_usage_error() -> Outcome {
    let runtime = tempfile::tempdir()?;
    let run = cgmr_in(runtime.path())?
        .args(["--needs-mem", "auto", "--mem-key", "a/b", "--", "true"])
        .assert();
    run.code(64);
    Ok(())
}

#[test]
fn check_sizes_auto_by_its_fallback() -> Outcome {
    let runtime = tempfile::tempdir()?;
    let check = cgmr_in(runtime.path())?
        .args(["check", "--needs-mem", "auto:1KB"])
        .assert();
    check.code(0);
    Ok(())
}

#[test]
fn advise_sizes_auto_by_its_fallback() -> Outcome {
    let runtime = tempfile::tempdir()?;
    let advise = cgmr_in(runtime.path())?
        .args(["advise", "--needs-mem", "auto:1KB", "--jobs", "1"])
        .assert();
    advise.code(0);
    Ok(())
}

#[test]
fn status_lists_what_was_learned() -> Outcome {
    let runtime = tempfile::tempdir()?;
    let run = cgmr_in(runtime.path())?
        .args(["--needs-mem", "auto:1KB", "--mem-key", "listed"])
        .args(SLOW)
        .assert();
    run.code(7);
    let status = cgmr_in(runtime.path())?.args(["status", "--json"]).assert();
    let out = status.code(0).get_output().stdout.clone();
    let text = String::from_utf8(out)?;
    assert!(text.contains("\"learned\":[{\"key\":\"listed\",\"peak_bytes\":"));
    let again = cgmr_in(runtime.path())?.arg("status").assert();
    let out = again.code(0).get_output().stdout.clone();
    assert!(String::from_utf8(out)?.contains("learned memory peaks\nlisted  "));
    Ok(())
}
