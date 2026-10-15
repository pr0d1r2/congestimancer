//! Test-only: the process table fixtures under `tests/fixtures/procs/`,
//! one `pid ppid uid name` per line, `#` comments, uid `-` for none.

use std::fs;
use std::path::Path;

use super::Proc;

/// The processes of fixture `case`; an unreadable or bad line is skipped,
/// so a broken fixture shows as a wrong count.
pub(crate) fn table(case: &str) -> Vec<Proc> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir.join("tests/fixtures/procs").join(case);
    let text = fs::read_to_string(path).unwrap_or_default();
    text.lines().filter_map(row).collect()
}

/// One fixture line as a process.
fn row(line: &str) -> Option<Proc> {
    if line.starts_with('#') {
        return None;
    }
    let mut fields = line.splitn(4, ' ');
    let pid = fields.next()?.parse().ok()?;
    let parent = fields.next()?.parse().ok()?;
    let uid = fields.next()?.parse().ok();
    let name = fields.next()?;
    Some(Proc::new(name, uid).with_lineage(pid, parent))
}

/// The user names the fixtures use.
pub(crate) fn user(uid: u32) -> Option<String> {
    let name = match uid {
        0 => "root",
        501 => "alice",
        30001 => "nixbld1",
        30002 => "nixbld2",
        _ => return None,
    };
    Some(name.to_owned())
}

#[cfg(test)]
mod tests;
