//! `--export-jobs` from the binary's side (src/launch:V10): the command
//! sees each named variable set to its share of the CPUs.

use std::error::Error;
use std::path::Path;

use super::{Outcome, cgmr};

/// What the command sees in the two exported variables, run with `extra`
/// switches and its lock dir under `runtime`.
fn seen(runtime: &Path, extra: &[&str]) -> Result<String, Box<dyn Error>> {
    let mut cmd = cgmr()?;
    cmd.env("XDG_RUNTIME_DIR", runtime);
    let names = ["--export-jobs", "CGMR_TEST_JOBS", "--export-jobs", "OTHER"];
    let show = ["--", "sh", "-c", "echo $CGMR_TEST_JOBS $OTHER"];
    let out = cmd.args(names).args(extra).args(show).output()?;
    Ok(String::from_utf8(out.stdout)?.trim().to_owned())
}

#[test]
fn export_jobs_hands_the_command_its_share_of_the_cpus() -> Outcome {
    let runtime = tempfile::tempdir()?;
    let alone = seen(runtime.path(), &[])?;
    let (first, second) = alone.split_once(' ').ok_or("no two values")?;
    assert_eq!(first, second);
    assert!(first.parse::<u64>()? >= 1);
    let slotted = ["--slots", "1", "--slot-key", "share"];
    assert_eq!(seen(runtime.path(), &slotted)?, alone);
    Ok(())
}
