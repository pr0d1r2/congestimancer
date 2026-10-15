//! The environment: `CGMR_<SWITCH>` defaults in the formats of
//! src/cli:V10, and a warning for a `CGMR_` name nobody reads
//! (src/cli:V11).

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};

use super::SWITCHES;
use super::args::TRUE;
use super::exit::Failure;
use super::table::{SwitchSpec, WHEN, env_name};
use crate::slot::HELD_SLOTS_ENV;

/// The prefix of every cgmr variable.
const PREFIX: &str = "CGMR_";

/// How an off flag is held.
pub(crate) const FALSE: &str = "false";

/// The hidden fixture root (src/cli:V8).
pub(crate) const FIXTURE_ROOT: &str = "CGMR_FIXTURE_ROOT";

/// The dogfood switch the dev-shell shim reads (.:V1).
pub(crate) const DOGFOOD: &str = "CGMR_DOGFOOD";

/// Variables cgmr reads that are no switch of the table (src/cli:V11).
const OTHERS: [&str; 3] = [DOGFOOD, FIXTURE_ROOT, HELD_SLOTS_ENV];

/// The process environment, as cgmr reads it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Env(BTreeMap<OsString, OsString>);

impl Env {
    /// The environment `vars` make.
    pub(crate) fn new<I>(vars: I) -> Self
    where
        I: IntoIterator<Item = (OsString, OsString)>,
    {
        Self(vars.into_iter().collect())
    }

    /// The variable `name`, if set.
    pub(crate) fn var_os(&self, name: &str) -> Option<&OsStr> {
        self.0.get(OsStr::new(name)).map(OsString::as_os_str)
    }

    /// The variable `name` as text, if set and not empty.
    ///
    /// # Errors
    ///
    /// A usage error for a value that is not UTF-8.
    pub(crate) fn text(&self, name: &str) -> Result<Option<&str>, Failure> {
        let Some(raw) = self.var_os(name).filter(|v| !v.is_empty()) else {
            return Ok(None);
        };
        let text = raw.to_str().ok_or_else(|| {
            Failure::usage(format!("{name}: the value is not UTF-8"))
        })?;
        Ok(Some(text))
    }

    /// The values `spec`'s variable gives, in the switch's own form
    /// (src/cli:V10); `None` when unset, and always for `--when`
    /// (src/cli:V3).
    ///
    /// # Errors
    ///
    /// A usage error for a boolean that is not one of the six words.
    pub(crate) fn switch(
        &self,
        spec: &SwitchSpec,
    ) -> Result<Option<Vec<String>>, Failure> {
        let Some(name) = spec.env().filter(|_| false) else {
            return Ok(None);
        };
        let Some(text) = self.text(&name)? else {
            return Ok(None);
        };
        if !spec.takes_value() {
            return boolean(&name, text).map(|b| Some(vec![b.to_owned()]));
        }
        if spec.repeats() {
            return Ok(Some(items(text)));
        }
        Ok(Some(vec![text.to_owned()]))
    }

    /// One warning per `CGMR_` variable cgmr does not read, naming the
    /// nearest one it does (src/cli:V11).
    pub(crate) fn unknown(&self) -> Vec<String> {
        let names = self.0.keys().filter_map(|k| k.to_str());
        let ours = names.filter(|k| k.starts_with(PREFIX));
        let _ = ours.filter_map(warning);
        Vec::new()
    }
}

/// A boolean switch's variable as `true` or `false` (src/cli:V10).
fn boolean(name: &str, text: &str) -> Result<&'static str, Failure> {
    match text.to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" => Ok(TRUE),
        "0" | "false" | "no" => Ok(FALSE),
        _ => Err(Failure::usage(format!(
            "{name}={text:?}: takes 1, true or yes (on), 0, false or no (off)"
        ))),
    }
}

/// A repeatable switch's items: comma-separated, blanks dropped.
fn items(text: &str) -> Vec<String> {
    let parts = text.split(',').map(str::trim).filter(|p| !p.is_empty());
    parts.map(str::to_owned).collect()
}

/// Every variable cgmr reads.
fn known() -> Vec<String> {
    let switches = SWITCHES.iter().filter_map(SwitchSpec::env);
    switches.chain(OTHERS.map(str::to_owned)).collect()
}

/// The warning for variable `name`, or `None` when cgmr reads it.
fn warning(name: &str) -> Option<String> {
    let known = known();
    if known.iter().any(|k| k == name) {
        return None;
    }
    if name == env_name(WHEN) {
        return Some(format!(
            "{name} is ignored: --when has no environment default, so the \
             environment cannot inject a command"
        ));
    }
    let hint = nearest(name, &known)
        .map(|k| format!("; did you mean {k}?"))
        .unwrap_or_default();
    Some(format!("unknown variable {name}{hint}"))
}

/// The known name closest to `name`, if any is close: at most a third of
/// its letters apart, or two for a short name.
fn nearest<'k>(name: &str, known: &'k [String]) -> Option<&'k str> {
    let near = |k: &&'k String| distance(name, k);
    let best = known.iter().min_by_key(near)?;
    let limit = name.len().saturating_sub(PREFIX.len()) / 3;
    (distance(name, best) <= limit.max(2)).then_some(best.as_str())
}

/// The edit distance between `a` and `b` (Levenshtein, by bytes).
fn distance(a: &str, b: &str) -> usize {
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.bytes().enumerate() {
        let mut cur = vec![i.saturating_add(1)];
        for (pair, cb) in prev.windows(2).zip(b.bytes()) {
            let [diagonal, up] = [pair.first(), pair.last()]
                .map(|v| v.copied().unwrap_or_default());
            let left = cur.last().copied().unwrap_or_default();
            let replace = diagonal.saturating_add(usize::from(ca != cb));
            let step = up.min(left).saturating_add(1);
            cur.push(replace.min(step));
        }
        prev = cur;
    }
    prev.last().copied().unwrap_or_default()
}

#[cfg(test)]
mod tests;
