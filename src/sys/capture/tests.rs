use std::fs;
use std::path::{Path, PathBuf};

use super::{capture, copy_files};
use crate::signal::Root;

fn fixtures() -> Root {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    Root::new(dir.join("tests/fixtures/sys/root"))
}

/// An output directory of this test's own, removed by the guard.
struct Out(PathBuf);

impl Out {
    fn new(name: &str) -> Self {
        let pid = std::process::id();
        Self(std::env::temp_dir().join(format!("cgmr-capture-{name}-{pid}")))
    }
}

impl Drop for Out {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn relative(written: &[PathBuf], base: &Path) -> Vec<String> {
    let mut names: Vec<String> = written
        .iter()
        .filter_map(|p| p.strip_prefix(base).ok())
        .map(|p| p.display().to_string())
        .collect();
    names.sort();
    names
}

#[test]
fn listed_files_present_are_copied_at_their_paths() {
    let out = Out::new("files");
    let written = copy_files(&fixtures(), &out.0).unwrap_or_default();
    let want = [
        "proc/sys/fs/file-nr",
        "sys/block/sda/queue/rotational",
        "sys/block/sda/stat",
        "sys/class/thermal/thermal_zone0/temp",
        "sys/class/thermal/thermal_zone0/type",
    ];
    assert_eq!(relative(&written, &out.0), want);
}

#[test]
fn a_copy_is_verbatim() {
    let out = Out::new("verbatim");
    let _ = copy_files(&fixtures(), &out.0);
    let copied = fs::read_to_string(out.0.join("proc/sys/fs/file-nr")).ok();
    assert_eq!(copied.as_deref(), Some("2304\t0\t9223372036854775807\n"));
}

#[test]
fn files_off_the_list_are_left_behind() {
    let out = Out::new("unlisted");
    let _ = copy_files(&fixtures(), &out.0);
    let policy = out.0.join("sys/class/thermal/thermal_zone0/policy");
    assert!(!policy.exists());
}

#[test]
fn host_capture_writes_under_os_and_arch() {
    let out = Out::new("live");
    let written = capture(&Root::system(), &out.0).unwrap_or_default();
    let host = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    assert!(!written.is_empty());
    assert!(written.iter().all(|p| p.starts_with(out.0.join(&host))));
}
