//! `--needs-mem auto[:SIZE]`: the memory a kind of job needs, learned
//! from the peaks of its earlier runs (src/slot:V13). The switch is
//! settled before anything reads it: a learned key becomes its peak and
//! a tenth over, an unknown one the fallback, so the gates and the
//! reservation see an ordinary size.

use std::ffi::OsString;

use super::env::Env;
use super::exit::{Failure, slot_failure};
use super::layers::Resolved;
use super::slots::lock_dir;
use crate::slot::{LockDir, learned_peak, reserve_for, slot_key};
use crate::units::{Bytes, Name, parse_bytes};

/// The word that asks for a learned size.
const AUTO: &str = "auto";

/// What an unknown key reserves without a fallback: one GiB, as `advise`
/// sizes a job.
pub(crate) const AUTO_DEFAULT: Bytes = Bytes(1 << 30);

/// What `--needs-mem` was given.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Need {
    /// A size.
    Fixed(Bytes),
    /// `auto`, with the fallback for a key never measured.
    Auto(Option<Bytes>),
}

/// Reads a `--needs-mem` value: a size, `auto` or `auto:SIZE`.
///
/// # Errors
///
/// The form expected, in words.
pub(crate) fn parse_need(s: &str) -> Result<Need, String> {
    let Some(rest) = s.strip_prefix(AUTO) else {
        return parse_bytes(s).map(Need::Fixed).map_err(|e| e.to_string());
    };
    match rest.strip_prefix(':') {
        None if rest.is_empty() => Ok(Need::Auto(None)),
        Some(size) => parse_bytes(size)
            .map(|b| Need::Auto(Some(b)))
            .map_err(|e| format!("{s:?}: the fallback is {e}")),
        None => Err(format!("{s:?}: {FORM}")),
    }
}

/// The forms `--needs-mem` takes, for an error.
const FORM: &str = "a size like 2GiB, or auto, or auto:SIZE for a fallback";

/// A run whose memory peak is to be kept for its key.
#[derive(Debug, Clone)]
pub(crate) struct Learning {
    /// The lock dir the ledger is in.
    dir: LockDir,
    /// The key the peak is kept under.
    key: Name,
}

impl Learning {
    /// The lock dir the peak goes to.
    pub(crate) const fn dir(&self) -> &LockDir {
        &self.dir
    }

    /// The key.
    pub(crate) const fn name(&self) -> &Name {
        &self.key
    }
}

/// Settles `--needs-mem auto` in `resolved` for `cmd`: the size to reserve
/// is written back as bytes. Returns what to learn from the run, or
/// `None` for a fixed size or when no key can be made.
///
/// # Errors
///
/// 64 for a bad `--mem-key`; 69 for a lock dir that cannot be used.
pub(crate) fn apply(
    env: &Env,
    resolved: &mut Resolved,
    cmd: &[OsString],
) -> Result<Option<Learning>, Failure> {
    let Some(Need::Auto(fallback)) = resolved.parse("needs-mem", parse_need)?
    else {
        return Ok(None);
    };
    let fallback = fallback.unwrap_or(AUTO_DEFAULT);
    let Some(key) = mem_key(resolved, cmd)? else {
        resolved.set("needs-mem", &bytes(fallback));
        return Ok(None);
    };
    let dir = lock_dir(env)?;
    let peak = learned_peak(&dir, &key).map_err(slot_failure)?;
    resolved.set("needs-mem", &bytes(peak.map_or(fallback, reserve_for)));
    Ok(Some(Learning { dir, key }))
}

/// The key a run's peak is kept under: `--mem-key`, else `--slot-key`,
/// else the command's name; `None` when none can be made and none was
/// asked for.
fn mem_key(
    resolved: &Resolved,
    cmd: &[OsString],
) -> Result<Option<Name>, Failure> {
    let program = cmd.first().and_then(|c| c.to_str());
    let explicit = resolved.text("mem-key").or(resolved.text("slot-key"));
    match slot_key(explicit, program) {
        Ok(key) => Ok(Some(key)),
        Err(_) if explicit.is_none() => Ok(None),
        Err(err) => Err(slot_failure(err)),
    }
}

/// `b` as the size text every `--needs-mem` reader takes.
fn bytes(b: Bytes) -> String {
    format!("{}B", b.0)
}

#[cfg(test)]
mod tests;
