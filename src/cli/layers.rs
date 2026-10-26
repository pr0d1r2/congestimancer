//! [`Resolved`]: each switch's value from the highest layer that sets it:
//! flag, then environment, then profile, then the built-in default
//! (src/cli:V2, src/profile:V1).

use std::collections::BTreeMap;
use std::fmt::{self, Display};

use super::SWITCHES;
use super::args::{Given, TRUE};
use super::env::Env;
use super::exit::Failure;
use super::table::SwitchSpec;
use crate::profile::{At, Switch};

/// Where a switch's value came from, for error messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Layer {
    /// The command line.
    Flag,
    /// An environment variable, by name.
    Env(String),
    /// A profile line.
    Profile(At),
}

/// One switch's values and their layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The values as text; `true` or `false` for a flag.
    pub(crate) values: Vec<String>,
    /// Where they came from.
    pub(crate) layer: Layer,
}

/// Every switch set by some layer, by name.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Resolved(BTreeMap<&'static str, Entry>);

impl Resolved {
    /// The first value of switch `name`, if set.
    pub(crate) fn text(&self, name: &str) -> Option<&str> {
        self.0.get(name)?.values.first().map(String::as_str)
    }

    /// Every value of switch `name`; none when unset.
    pub(crate) fn all(&self, name: &str) -> &[String] {
        self.0.get(name).map_or(&[], |e| e.values.as_slice())
    }

    /// Whether flag `name` is on.
    pub(crate) fn on(&self, name: &str) -> bool {
        self.text(name) == Some(TRUE)
    }

    /// Whether switch `name` is set to something other than off.
    pub(crate) fn has(&self, name: &str) -> bool {
        self.text(name).is_some_and(|v| v != super::env::FALSE)
    }

    /// Switch `name`'s first value read by `parse`.
    ///
    /// # Errors
    ///
    /// A usage error naming the layer and the parser's reason.
    pub(crate) fn parse<T, E: Display>(
        &self,
        name: &str,
        parse: impl Fn(&str) -> Result<T, E>,
    ) -> Result<Option<T>, Failure> {
        let Some(entry) = self.0.get(name) else {
            return Ok(None);
        };
        let Some(text) = entry.values.first() else {
            return Ok(None);
        };
        parse(text).map(Some).map_err(|e| entry.fault(name, &e))
    }

    /// Replaces the value of switch `name` with `text`, keeping its layer;
    /// nothing when the switch is not set.
    pub(crate) fn set(&mut self, name: &str, text: &str) {
        if let Some(entry) = self.0.get_mut(name) {
            entry.values = vec![text.to_owned()];
        }
    }

    /// Checks every value against its switch's parser (src/cli:V10).
    ///
    /// # Errors
    ///
    /// A usage error for the first value that does not parse.
    pub(crate) fn check(&self) -> Result<(), Failure> {
        for (name, entry) in &self.0 {
            let Some(spec) = super::table::spec(name) else {
                continue;
            };
            let refused = entry.values.iter().find_map(|text| {
                spec.takes_value().then(|| spec.value.check(text).err())?
            });
            if let Some(why) = refused {
                return Err(entry.fault(name, &why));
            }
        }
        Ok(())
    }

    /// Sets `name` to `values` from `layer`, for tests.
    #[cfg(test)]
    pub(crate) fn with(
        mut self,
        name: &'static str,
        values: &[&str],
        layer: Layer,
    ) -> Self {
        let values = values.iter().map(|v| (*v).to_owned()).collect();
        self.0.insert(name, Entry { values, layer });
        self
    }
}

impl Entry {
    /// A usage error for a value of switch `name` refused because `why`.
    fn fault(&self, name: &str, why: &dyn Display) -> Failure {
        Failure::usage(format!("{}: {why}", Shown(name, &self.layer)))
    }
}

/// A switch as its layer names it: `--load`, `CGMR_LOAD`, `file:3: --load`.
struct Shown<'a>(&'a str, &'a Layer);

impl Display for Shown<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.1 {
            Layer::Flag => write!(f, "--{}", self.0),
            Layer::Env(var) => f.write_str(var),
            Layer::Profile(at) => write!(f, "{at}: --{}", self.0),
        }
    }
}

/// Each switch from the highest layer that sets it: `given` on the
/// command line, then `env`, then `profile` (src/cli:V2).
///
/// # Errors
///
/// A usage error for an environment value not in its form (src/cli:V10).
pub(crate) fn resolve(
    given: &Given,
    env: &Env,
    profile: &[Switch],
) -> Result<Resolved, Failure> {
    let mut resolved = Resolved::default();
    for spec in SWITCHES {
        if let Some(entry) = layered(spec, given, env, profile)? {
            resolved.0.insert(spec.name(), entry);
        }
    }
    Ok(resolved)
}

/// `spec`'s entry from the highest layer that sets it.
fn layered(
    spec: &SwitchSpec,
    given: &Given,
    env: &Env,
    profile: &[Switch],
) -> Result<Option<Entry>, Failure> {
    if let Some(values) = given.get(spec.name()) {
        let values = values.to_vec();
        return Ok(Some(Entry {
            values,
            layer: Layer::Flag,
        }));
    }
    if let (Some(values), Some(var)) = (env.switch(spec)?, spec.env()) {
        return Ok(Some(Entry {
            values,
            layer: Layer::Env(var),
        }));
    }
    Ok(from_profile(spec, profile))
}

/// `spec`'s values in `profile`, if it sets them.
fn from_profile(spec: &SwitchSpec, profile: &[Switch]) -> Option<Entry> {
    let flag = format!("--{}", spec.name());
    let lines: Vec<&Switch> =
        profile.iter().filter(|s| s.flag() == flag).collect();
    let first = lines.first()?;
    let value = |s: &&Switch| s.value().unwrap_or(TRUE).to_owned();
    Some(Entry {
        values: lines.iter().map(value).collect(),
        layer: Layer::Profile(first.at().clone()),
    })
}

/// Whether the command line or the environment sets `flag` (as written,
/// `--load`), so a profile's value of it is left out (src/profile:V6).
pub(crate) fn set_above(given: &Given, env: &Env, flag: &str) -> bool {
    let Some(spec) = flag.strip_prefix("--").and_then(super::table::spec)
    else {
        return false;
    };
    given.get(spec.name()).is_some()
        || matches!(env.switch(spec), Ok(Some(_)) | Err(_))
}

#[cfg(test)]
mod tests;
