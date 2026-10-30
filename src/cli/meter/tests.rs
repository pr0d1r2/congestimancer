use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::{PeakMeter, exclusive, gap, job_memory};
use crate::cli::env::Env;
use crate::cli::layers::{Layer, Resolved};
use crate::cli::learn::{Learning, apply};
use crate::launch::Meter;
use crate::signal::Root;
use crate::slot::learned_peak;

fn tree(name: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("cgmr-meter-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn put(root: &Path, file: &str, text: &str) {
    let path = root.join(file.trim_start_matches('/'));
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, text);
}

#[test]
fn a_cgroup_holding_only_the_job_is_the_jobs() {
    assert!(exclusive("10\n11\n", &[10, 11, 12]));
}

#[test]
fn a_cgroup_holding_anything_else_is_not() {
    assert!(!exclusive("10\n99\n", &[10, 11]));
}

#[test]
fn an_empty_cgroup_is_nobodys() {
    assert!(!exclusive("", &[10]));
}

/// A job cgroup of pids 10 and 11 with `descendants` child cgroups and the
/// given `memory.stat`, under `dir`.
fn job_cgroup(dir: &Path, descendants: u32, memory: &str) {
    let base = "/sys/fs/cgroup/work/job";
    put(dir, "/proc/10/cgroup", "0::/work/job\n");
    put(dir, &format!("{base}/cgroup.procs"), "10\n11\n");
    put(
        dir,
        &format!("{base}/cgroup.stat"),
        &format!("nr_descendants {descendants}\nnr_dying_descendants 0\n"),
    );
    put(dir, &format!("{base}/memory.stat"), memory);
}

#[test]
fn the_cgroup_of_the_job_gives_unreclaimable_memory_not_page_cache() {
    let dir = tree("cgroup");
    job_cgroup(&dir, 0, "anon 700\nfile 90000\nshmem 20\nkernel 80\n");
    let root = Root::new(&dir);
    assert_eq!(job_memory(&root, 10, &[10, 11], 5), 800);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_cgroup_with_descendants_falls_back_to_the_process_tree() {
    let dir = tree("parent");
    job_cgroup(&dir, 1, "anon 700\n");
    let root = Root::new(&dir);
    assert_eq!(job_memory(&root, 10, &[10, 11], 5), 5);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_shared_cgroup_falls_back_to_the_process_tree() {
    let dir = tree("shared");
    put(&dir, "/proc/10/cgroup", "0::/work/session\n");
    put(&dir, "/sys/fs/cgroup/work/session/cgroup.procs", "10\n77\n");
    put(
        &dir,
        "/sys/fs/cgroup/work/session/memory.stat",
        "anon 9999\n",
    );
    let root = Root::new(&dir);
    assert_eq!(job_memory(&root, 10, &[10], 5), 5);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn no_cgroup_at_all_falls_back_to_the_process_tree() {
    let dir = tree("none");
    let _ = fs::create_dir_all(&dir);
    assert_eq!(job_memory(&Root::new(&dir), 10, &[10], 5), 5);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn sampling_starts_at_a_second_and_backs_off_to_ten() {
    let secs = Duration::from_secs;
    assert_eq!(gap(secs(0)), secs(1));
    assert_eq!(gap(secs(6)), secs(3));
    assert_eq!(gap(secs(600)), secs(10));
}

/// A meter for the key `probe`, whose lock dir is under a fresh `tree`.
fn meter_in(name: &str) -> (PeakMeter, Learning, PathBuf) {
    let dir = tree(name);
    let _ = fs::create_dir_all(&dir);
    let env = Env::new([("TMPDIR".into(), dir.clone().into())]);
    let mut resolved =
        Resolved::default().with("needs-mem", &["auto"], Layer::Flag);
    let cmd = [std::ffi::OsString::from("probe")];
    let Some(learning) = apply(&env, &mut resolved, &cmd).ok().flatten() else {
        unreachable!("a key was made");
    };
    let meter = PeakMeter::new(learning.clone(), Root::system());
    (meter, learning, dir)
}

#[test]
fn a_run_keeps_its_peak_for_its_key_when_it_ends() {
    let (mut meter, learning, dir) = meter_in("run");
    let now = Instant::now();
    meter.sample(Some(std::process::id()), now);
    assert!(meter.peak().0 > 0);
    meter.sample(None, now + Duration::from_secs(60));
    meter.finish();
    let kept = learned_peak(learning.dir(), learning.name());
    assert_eq!(kept.ok().flatten(), Some(meter.peak()));
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn a_second_sample_is_not_due_until_the_gap_passes() {
    let (mut meter, _, dir) = meter_in("due");
    let (t0, me) = (Instant::now(), Some(std::process::id()));
    meter.sample(me, t0);
    let first = meter.last;
    meter.sample(me, t0 + Duration::from_millis(200));
    assert_eq!(meter.last, first);
    meter.sample(me, t0 + Duration::from_secs(2));
    assert_ne!(meter.last, first);
    let _ = fs::remove_dir_all(dir);
}
