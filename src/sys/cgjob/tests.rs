use std::fs;
use std::io;
use std::os::unix::process::CommandExt as _;
use std::path::PathBuf;
use std::process::Command;

use super::{CgroupJob, GroupSignal, own_cgroup_dir};

/// A directory that looks like a cgroup: the files the job writes.
fn fake(test: &str) -> io::Result<PathBuf> {
    let dir = std::env::temp_dir()
        .join(format!("cgmr-cgjob-{test}-{}", std::process::id()));
    fs::create_dir_all(&dir)?;
    for file in ["cgroup.procs", "cgroup.freeze", "cgroup.kill"] {
        fs::write(dir.join(file), "")?;
    }
    Ok(dir)
}

fn read(dir: &std::path::Path, file: &str) -> io::Result<String> {
    fs::read_to_string(dir.join(file))
}

#[test]
fn own_dir_is_the_unified_line_below_the_mount() -> io::Result<()> {
    let mount = PathBuf::from("/sys/fs/cgroup");
    let dir = own_cgroup_dir(&mount, "0::/user.slice/job.service\n")?;
    assert_eq!(dir, mount.join("user.slice/job.service"));
    let root = own_cgroup_dir(&mount, "0::/\n")?;
    assert_eq!(root, mount);
    Ok(())
}

#[test]
fn own_dir_is_unsupported_without_a_unified_line() {
    let mount = PathBuf::from("/sys/fs/cgroup");
    let err = own_cgroup_dir(&mount, "1:name=systemd:/x\n").err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::Unsupported));
}

#[test]
fn at_wants_a_cgroup_procs_file() -> io::Result<()> {
    let bare = std::env::temp_dir()
        .join(format!("cgmr-cgjob-bare-{}", std::process::id()));
    fs::create_dir_all(&bare)?;
    let err = CgroupJob::at(bare.clone()).err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::Unsupported));
    fs::remove_dir_all(&bare)
}

#[test]
fn create_is_unsupported_where_the_parent_is_no_cgroup() -> io::Result<()> {
    let parent = std::env::temp_dir()
        .join(format!("cgmr-cgjob-plain-{}", std::process::id()));
    fs::create_dir_all(&parent)?;
    let err = CgroupJob::create(&parent, 7).err();
    assert_eq!(err.map(|e| e.kind()), Some(io::ErrorKind::Unsupported));
    // The failed attempt leaves no `cgmr-7` behind.
    assert!(!parent.join("cgmr-7").exists());
    fs::remove_dir_all(&parent)
}

#[test]
fn stop_writes_freeze_one_and_cont_writes_zero() -> io::Result<()> {
    let dir = fake("freeze")?;
    let job = CgroupJob::at(dir.clone())?;
    job.signal(GroupSignal::Stop)?;
    assert_eq!(read(&dir, "cgroup.freeze")?, "1");
    job.signal(GroupSignal::Cont)?;
    assert_eq!(read(&dir, "cgroup.freeze")?, "0");
    drop(job);
    fs::remove_dir_all(&dir)
}

#[test]
fn kill_writes_cgroup_kill() -> io::Result<()> {
    let dir = fake("kill")?;
    let job = CgroupJob::at(dir.clone())?;
    job.signal(GroupSignal::Kill)?;
    assert_eq!(read(&dir, "cgroup.kill")?, "1");
    drop(job);
    fs::remove_dir_all(&dir)
}

#[test]
fn term_reaches_every_process_listed_in_cgroup_procs() -> io::Result<()> {
    let dir = fake("term")?;
    let mut sleeper =
        Command::new("sleep").arg("30").process_group(0).spawn()?;
    fs::write(dir.join("cgroup.procs"), format!("{}\n", sleeper.id()))?;
    let job = CgroupJob::at(dir.clone())?;
    job.signal(GroupSignal::Term)?;
    let status = sleeper.wait()?;
    assert!(!status.success());
    drop(job);
    fs::remove_dir_all(&dir)
}

#[test]
fn enter_in_makes_the_child_write_itself_to_cgroup_procs() -> io::Result<()> {
    let dir = fake("enter")?;
    let job = CgroupJob::at(dir.clone())?;
    let mut command = Command::new("true");
    job.enter_in(&mut command)?;
    let child = command.spawn()?.wait()?;
    assert!(child.success());
    // The fake `cgroup.procs` is a plain file: the child wrote "0".
    assert_eq!(read(&dir, "cgroup.procs")?, "0");
    drop(job);
    fs::remove_dir_all(&dir)
}
