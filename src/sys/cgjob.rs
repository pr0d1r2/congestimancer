//! A delegated cgroup v2 for one `--govern-by cgroup` command: created
//! below the cgroup `cgmr` runs in, entered by the command before exec,
//! frozen and thawed through `cgroup.freeze`, killed through
//! `cgroup.kill` (src/sys:T27, src/launch:V12).

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command;

use rustix::process::{Pid, kill_process};

use super::GroupSignal;

/// Where cgroup v2 is mounted.
const MOUNT: &str = "/sys/fs/cgroup";

/// The kernel's account of this process's cgroups.
const PROC_CGROUP: &str = "/proc/self/cgroup";

/// This process's mounts, one per line.
const MOUNTINFO: &str = "/proc/self/mountinfo";

/// A cgroup the command lives in; one `create`d here is removed on drop
/// when it is empty.
#[derive(Debug)]
pub struct CgroupJob {
    /// Its directory.
    dir: PathBuf,
    /// `cgroup.procs`, open for writing: the child writes itself to it.
    procs: File,
    /// Whether this job made the directory, and so removes it.
    owned: bool,
}

/// The cgroup `cgmr` runs in, when it may create a child there: the
/// directory below the cgroup2 mount that `/proc/self/cgroup` names, in
/// which a trial child gets the files a [`CgroupJob`] writes.
///
/// # Errors
///
/// [`io::ErrorKind::Unsupported`] with the reason when there is none.
pub fn delegated_cgroup() -> io::Result<PathBuf> {
    let text = fs::read_to_string(PROC_CGROUP)
        .map_err(|err| unsupported(&format!("{PROC_CGROUP}: {err}")))?;
    // Unreadable, it is taken to mount the whole hierarchy.
    let mounts = fs::read_to_string(MOUNTINFO).unwrap_or_default();
    delegated(own_cgroup_dir(Path::new(MOUNT), &mounts, &text)?)
}

/// `dir`, when `cgmr` may make a cgroup below it that it could freeze
/// and kill: a trial `cgmr-<pid>` is made and removed again, so a parent
/// it cannot use is refused before the wait, not after (src/launch:V12).
fn delegated(dir: PathBuf) -> io::Result<PathBuf> {
    match CgroupJob::create(&dir, std::process::id()) {
        Ok(trial) => drop(trial),
        Err(err) if err.kind() == io::ErrorKind::Unsupported => {
            return Err(err);
        }
        Err(err) => {
            return Err(unsupported(&format!("{}: {err}", dir.display())));
        }
    }
    Ok(dir)
}

/// The cgroup directory `proc_cgroup` (the text of `/proc/self/cgroup`)
/// names below `mount`. `mountinfo` (the text of `/proc/self/mountinfo`)
/// gives the cgroup the mount shows at its top: `/`, the whole hierarchy,
/// when no cgroup2 line names `mount`.
///
/// # Errors
///
/// [`io::ErrorKind::Unsupported`] when no line names a cgroup v2, or it
/// is not below the mount's top.
pub fn own_cgroup_dir(
    mount: &Path,
    mountinfo: &str,
    proc_cgroup: &str,
) -> io::Result<PathBuf> {
    let unified = proc_cgroup
        .lines()
        .find_map(|line| line.strip_prefix("0::"))
        .ok_or_else(|| unsupported("this process is in no cgroup v2"))?
        .trim();
    let top = mount_top(mount, mountinfo).unwrap_or("/");
    let below = Path::new(unified).strip_prefix(top).map_err(|_| {
        unsupported(&format!(
            "cgroup {unified} is not below {top}, the top of {}",
            mount.display()
        ))
    })?;
    Ok(mount.join(below))
}

/// The root column of the last cgroup2 line of `mountinfo` mounted at
/// `mount`: the cgroup a bind mount of a subtree shows at its top.
fn mount_top<'a>(mount: &Path, mountinfo: &'a str) -> Option<&'a str> {
    mountinfo.lines().rev().find_map(|line| {
        let mut words = line.split_whitespace();
        let root = words.nth(3)?;
        let point = words.next()?;
        let mut tail = words.skip_while(|w| *w != "-").skip(1);
        let cgroup2 = tail.next() == Some("cgroup2");
        (cgroup2 && Path::new(point) == mount).then_some(root)
    })
}

/// `dir`'s `cgroup.procs`, open for writing.
fn open_procs(dir: &Path) -> io::Result<File> {
    let path = dir.join("cgroup.procs");
    OpenOptions::new()
        .write(true)
        .open(&path)
        .map_err(|err| unsupported(&format!("{}: {err}", path.display())))
}

impl CgroupJob {
    /// Creates `cgmr-<pid>` below `parent`.
    ///
    /// # Errors
    ///
    /// [`io::ErrorKind::Unsupported`] when `parent` is not a cgroup
    /// `cgmr` may create in; the OS error otherwise.
    pub fn create(parent: &Path, pid: u32) -> io::Result<Self> {
        open_procs(parent)?;
        let dir = parent.join(format!("cgmr-{pid}"));
        fs::create_dir(&dir)?;
        match Self::at(dir.clone()) {
            Ok(mut job) => {
                job.owned = true;
                Ok(job)
            }
            Err(err) => {
                let _ = fs::remove_dir(&dir);
                Err(err)
            }
        }
    }

    /// The cgroup in directory `dir`, as it is.
    ///
    /// # Errors
    ///
    /// [`io::ErrorKind::Unsupported`] when `dir` has no writable
    /// `cgroup.procs`, `cgroup.freeze` or `cgroup.kill`.
    pub fn at(dir: PathBuf) -> io::Result<Self> {
        let procs = open_procs(&dir)?;
        // Without these the job could neither pause nor end the command.
        for name in ["cgroup.freeze", "cgroup.kill"] {
            let path = dir.join(name);
            OpenOptions::new().write(true).open(&path).map_err(|err| {
                unsupported(&format!("{}: {err}", path.display()))
            })?;
        }
        Ok(Self {
            dir,
            procs,
            owned: false,
        })
    }

    /// Its directory.
    #[must_use]
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Makes the child `command` spawns enter this cgroup before exec,
    /// so nothing it forks is ever outside. The job must outlive the
    /// spawn.
    #[allow(unsafe_code, reason = "pre_exec write in the child (src/sys:C)")]
    pub fn enter_in(&self, command: &mut Command) {
        use std::os::fd::AsRawFd as _;
        use std::os::unix::process::CommandExt as _;
        let fd = self.procs.as_raw_fd();
        // SAFETY: the hook runs in the forked child before exec, where
        // only async-signal-safe calls are sound; it makes one `write`
        // (async-signal-safe) of a static byte to a descriptor number it
        // owns by copy, and allocates nothing. The job keeps `procs`
        // open until the spawn, so `fd` names the same file in the child.
        unsafe {
            command.pre_exec(move || {
                let wrote = libc::write(fd, b"0".as_ptr().cast(), 1);
                if wrote == 1 {
                    Ok(())
                } else {
                    Err(io::Error::last_os_error())
                }
            })
        };
    }

    /// Does what `sig` asks of every process in the cgroup: freezes it,
    /// thaws it, kills it, or signals each process.
    ///
    /// # Errors
    ///
    /// The OS error.
    pub fn signal(&self, sig: GroupSignal) -> io::Result<()> {
        match sig {
            GroupSignal::Stop => self.write("cgroup.freeze", "1"),
            GroupSignal::Cont => self.write("cgroup.freeze", "0"),
            GroupSignal::Kill => self.write("cgroup.kill", "1"),
            _ => self.each_process(sig),
        }
    }

    /// Writes `text` to the control file `name`.
    fn write(&self, name: &str, text: &str) -> io::Result<()> {
        let mut file =
            OpenOptions::new().write(true).open(self.dir.join(name))?;
        file.write_all(text.as_bytes())
    }

    /// Sends `sig` to each process `cgroup.procs` lists.
    fn each_process(&self, sig: GroupSignal) -> io::Result<()> {
        let text = fs::read_to_string(self.dir.join("cgroup.procs"))?;
        let mut result = Ok(());
        for pid in text.lines().filter_map(|l| l.trim().parse::<i32>().ok()) {
            let Some(pid) = Pid::from_raw(pid) else {
                continue;
            };
            // A process that left since the read is no failure.
            if let Err(err) = kill_process(pid, sig.raw())
                && err != rustix::io::Errno::SRCH
            {
                result = Err(err.into());
            }
        }
        result
    }
}

impl Drop for CgroupJob {
    fn drop(&mut self) {
        if self.owned {
            // Fails while a process is left in it; nothing to do then.
            let _ = fs::remove_dir(&self.dir);
        }
    }
}

/// The error for a cgroup this machine cannot give.
fn unsupported(why: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, why.to_owned())
}

#[cfg(test)]
mod tests;
