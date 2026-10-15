//! The file moves every slot step is built from: open a lock file, wait
//! a moment for a short-held lock, publish a file already locked, and
//! tell a dead holder's file from a live one.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::sys::try_lock_exclusive;

/// How long a short-held lock (a counter, a start log) is waited for.
const LOCK_WAIT: Duration = Duration::from_secs(5);

/// How long to sleep between tries for a short-held lock.
const LOCK_POLL: Duration = Duration::from_millis(1);

/// Tells apart the temporary names this process publishes through.
static NEXT_TMP: AtomicU64 = AtomicU64::new(0);

/// Opens `path` to read and write, made mode 0600 when absent, never
/// truncated.
pub(crate) fn open_lock(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path)
}

/// Takes the `flock` on `file`, polling while someone else briefly holds
/// it; [`io::ErrorKind::TimedOut`] after [`LOCK_WAIT`].
pub(crate) fn lock_wait(file: &File) -> io::Result<()> {
    lock_within(file, LOCK_WAIT)
}

/// [`lock_wait`], giving up after `limit`.
fn lock_within(file: &File, limit: Duration) -> io::Result<()> {
    let start = Instant::now();
    while !try_lock_exclusive(file)? {
        if start.elapsed() > limit {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "a lock file stayed held",
            ));
        }
        std::thread::sleep(LOCK_POLL);
    }
    Ok(())
}

/// Writes `content` to a new file, locks it, and only then moves it to
/// `dir/name`, so no one ever sees it unlocked and takes it for dead.
pub(crate) fn publish(
    dir: &Path,
    name: &str,
    content: &str,
) -> io::Result<File> {
    let n = NEXT_TMP.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!("+tmp.{}.{n}", std::process::id()));
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp)?;
    file.write_all(content.as_bytes())?;
    if !try_lock_exclusive(&file)? {
        return Err(io::Error::other("a fresh file was already locked"));
    }
    fs::rename(&tmp, dir.join(name))?;
    Ok(file)
}

/// Whether the holder of `path` is gone: its lock is free (or the file
/// already is). A dead file is removed.
pub(crate) fn reap_if_dead(path: &Path) -> io::Result<bool> {
    let file = match File::open(path) {
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(true),
        opened => opened?,
    };
    if !try_lock_exclusive(&file)? {
        return Ok(false);
    }
    match fs::remove_file(path) {
        Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
        _ => Ok(true),
    }
}

/// The whole of `file`, from the start.
pub(crate) fn read_all(file: &mut File) -> io::Result<String> {
    let mut text = String::new();
    file.rewind()?;
    file.read_to_string(&mut text)?;
    Ok(text)
}

/// Replaces the whole of `file` with `text`.
pub(crate) fn rewrite(file: &mut File, text: &str) -> io::Result<()> {
    file.set_len(0)?;
    file.rewind()?;
    file.write_all(text.as_bytes())
}

/// The entry names of `dir` that start with `prefix`, the rest of each.
pub(crate) fn suffixes(dir: &Path, prefix: &str) -> io::Result<Vec<String>> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir)? {
        let name = entry?.file_name();
        let rest = name.to_str().and_then(|n| n.strip_prefix(prefix));
        found.extend(rest.map(str::to_owned));
    }
    Ok(found)
}

#[cfg(test)]
mod tests;
