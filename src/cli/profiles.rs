//! Profiles from the CLI's side: the switch table lent to `src/profile`
//! so a profile line meets argv's rules (src/profile:V5), and the profile
//! layer `--profile` asks for (src/cli:V2).

use std::path::{Path, PathBuf};

use super::args::Given;
use super::env::Env;
use super::exit::Failure;
use super::layers::set_above;
use super::table::{Role, SwitchSpec, env_name, spec};
use crate::profile::{
    Arity, Catalog, Places, ProfileError, ReadText, Switch, SwitchFault,
    SwitchTable, discover,
};
use crate::units::parse_name;

/// The CLI's switch table, as profiles see it: every switch but
/// `--profile` and `--profile-file`, which pick the profile itself.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CliTable;

impl SwitchTable for CliTable {
    fn check(
        &self,
        flag: &str,
        value: Option<&str>,
    ) -> Result<Arity, SwitchFault> {
        let found = flag.strip_prefix("--x").and_then(spec);
        let spec = found.filter(|s| !picks_profile(s.name(), s.role));
        let spec = spec.ok_or(SwitchFault::Unknown)?;
        value_fits(spec, value)?;
        Ok(if spec.repeats() {
            Arity::Repeatable
        } else {
            Arity::Single
        })
    }
}

/// Whether `value` fits `spec`: none for a flag, one that parses
/// otherwise (src/profile:V5).
fn value_fits(
    spec: &SwitchSpec,
    value: Option<&str>,
) -> Result<(), SwitchFault> {
    let refuse = |why: &str| Err(SwitchFault::BadValue(why.to_owned()));
    match (spec.takes_value(), value) {
        (false, Some(_)) => refuse("takes no value"),
        (true, None) => refuse("needs a value"),
        (true, Some(v)) => spec.value.check(v).map_err(SwitchFault::BadValue),
        (false, None) => Ok(()),
    }
}

/// Whether switch `name` picks the profile rather than being part of one.
fn picks_profile(name: &str, role: Role) -> bool {
    role == Role::Meta && name.starts_with("profile")
}

/// Switch `name` from the command line, else its environment variable.
fn asked<'a>(
    given: &'a Given,
    env: &'a Env,
    name: &str,
) -> Result<Option<&'a str>, Failure> {
    if let Some(value) = given.get(name).and_then(<[String]>::first) {
        return Ok(Some(value));
    }
    env.text(&env_name(name))
}

/// Where profile files are looked for: `--profile-file` (or
/// `CGMR_PROFILE_FILE`) alone, else from `cwd` up, then the user file.
///
/// # Errors
///
/// A usage error for a `CGMR_PROFILE_FILE` that is not UTF-8.
pub(crate) fn places(
    given: &Given,
    env: &Env,
    cwd: &Path,
) -> Result<Places, Failure> {
    let explicit = asked(given, env, "profile-file")?.map(PathBuf::from);
    let var = |name| env.var_os(name).map(PathBuf::from);
    Ok(Places::new(cwd)
        .explicit(explicit)
        .config_home(var("XDG_CONFIG_HOME"))
        .home(var("HOME")))
}

/// The profile files in force.
///
/// # Errors
///
/// A profile error, exit 64.
pub(crate) fn catalog(
    places: &Places,
    reader: &dyn ReadText,
) -> Result<Catalog, Failure> {
    discover(places, reader).map_err(profile_failure)
}

/// The switches of the profile `--profile` (or `CGMR_PROFILE`) names,
/// less those the command line or environment set (src/profile:V6);
/// none without a name.
///
/// # Errors
///
/// A profile error, exit 64: a bad name, an unknown profile, a line the
/// CLI refuses (src/profile:V4).
pub(crate) fn profile_layer(
    given: &Given,
    env: &Env,
    cwd: &Path,
    reader: &dyn ReadText,
) -> Result<Vec<Switch>, Failure> {
    let Some(name) = asked(given, env, "profile")? else {
        return Ok(Vec::new());
    };
    let name = parse_name(name)
        .map_err(|e| Failure::usage(format!("--profile: {e}")))?;
    let catalog = catalog(&places(given, env, cwd)?, reader)?;
    let (_, profile) = catalog.lookup(&name).map_err(profile_failure)?;
    let above = |flag: &str| set_above(given, env, flag);
    let _ = profile.layer(&CliTable, above).map_err(profile_failure);
    Ok(Vec::new())
}

/// A profile error as a failure, exit 64.
fn profile_failure(err: ProfileError) -> Failure {
    Failure::new(ProfileError::EXIT, err)
}

#[cfg(test)]
mod tests;
