//! `--export-jobs` from the binary's side (src/launch:V11): the command
//! sees its share of the CPUs in each named variable.

use std::error::Error;
use std::thread::available_parallelism;

use super::{Outcome, cgmr};

/// A shell that prints the two variables the tests export.
const SHOW: [&str; 4] = ["--", "sh", "-c", "echo $A $B"];

/// The CPUs this machine has, as `cgmr` counts them.
fn cpus() -> Result<usize, Box<dyn Error>> {
    Ok(available_parallelism()?.get())
}

#[test]
fn a_lone_job_gets_every_cpu_in_each_variable() -> Outcome {
    let runtime = tempfile::tempdir()?;
    let out = cgmr()?
        .env("XDG_RUNTIME_DIR", runtime.path())
        .args(["--slots", "2", "--slot-key", "exp"])
        .args(["--export-jobs", "A", "--export-jobs", "B"])
        .args(SHOW)
        .output()?;
    let want = format!("{0} {0}\n", cpus()?);
    assert_eq!(String::from_utf8(out.stdout)?, want);
    Ok(())
}

#[test]
fn without_slots_the_share_is_every_cpu() -> Outcome {
    let out = cgmr()?.args(["--export-jobs", "A"]).args(SHOW).output()?;
    assert_eq!(String::from_utf8(out.stdout)?, format!("{}\n", cpus()?));
    Ok(())
}

#[test]
fn a_bad_variable_name_is_a_usage_error() -> Outcome {
    cgmr()?.args(["--export-jobs", "A=B"]).assert().code(64);
    Ok(())
}
