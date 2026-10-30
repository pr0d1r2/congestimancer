//! argv through `clap`: the verbs, `run` when none is named (src/cli:V1),
//! and every switch of the table on each gate verb, given at most once
//! unless it repeats (src/cli:V9).

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Args, Command, FromArgMatches};
use clap::{Parser, Subcommand};
use clap_complete::Shell;

use super::SWITCHES;
use super::help;
use super::table::{Role, SwitchSpec, Value};

/// `cgmr`: the verbs.
#[derive(Debug, Parser)]
#[command(
    name = "cgmr",
    version,
    about = help::ABOUT,
    after_help = help::TOP,
    propagate_version = true,
    subcommand_required = true,
    disable_help_subcommand = true
)]
pub(crate) struct Cli {
    /// What to do.
    #[command(subcommand)]
    pub(crate) verb: Verb,
}

/// The verbs (src/cli:I).
#[derive(Debug, Subcommand)]
pub(crate) enum Verb {
    /// Wait until every gate passes, then exec the command after `--`
    /// (the default verb).
    #[command(after_help = help::RUN)]
    Run(RunArgs),
    /// Evaluate every gate once: exit 0 if all pass, 1 if not.
    #[command(after_help = help::CHECK)]
    Check(Report),
    /// Print every signal with its current value.
    #[command(after_help = help::STATUS)]
    Status(Report),
    /// Say how many parallel jobs fit now and what limits them.
    #[command(after_help = help::ADVISE)]
    Advise(AdviseArgs),
    /// List the profiles in force: names, files and switches.
    #[command(after_help = help::PROFILES)]
    Profiles(Given),
    /// Print a shell completion script.
    #[command(after_help = help::COMPLETIONS)]
    Completions {
        /// The shell to complete for.
        shell: Shell,
    },
    /// Print the man page (roff).
    #[command(after_help = help::MAN)]
    Man,
    /// Developer tools.
    #[command(hide = true, subcommand)]
    Debug(DebugVerb),
}

/// Hidden developer verbs (src/cli:I).
#[derive(Debug, Subcommand)]
pub(crate) enum DebugVerb {
    /// Copy this machine's raw sources into DIR as a fixture tree
    /// (src/sys:V7).
    Capture {
        /// Where the fixture tree goes.
        dir: PathBuf,
    },
}

/// `run`: the switches and the command.
#[derive(Debug, Args)]
pub(crate) struct RunArgs {
    /// The switches.
    #[command(flatten)]
    pub(crate) given: Given,
    /// The command, only after `--`.
    #[arg(last = true, value_name = "CMD")]
    pub(crate) cmd: Vec<OsString>,
}

/// `check` and `status`: the switches and `--json`.
#[derive(Debug, Args)]
pub(crate) struct Report {
    /// The switches.
    #[command(flatten)]
    pub(crate) given: Given,
    /// Print JSON for agents and scripts.
    #[arg(long)]
    pub(crate) json: bool,
}

/// `advise`: the switches, `--jobs` and `--json`.
#[derive(Debug, Args)]
pub(crate) struct AdviseArgs {
    /// The switches and `--json`.
    #[command(flatten)]
    pub(crate) report: Report,
    /// The parallelism you want; the advice says how much of it fits.
    #[arg(long, value_name = "J")]
    pub(crate) jobs: Option<u64>,
}

/// The table's switches as given on the command line: name to values,
/// `true` for a flag.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Given(BTreeMap<&'static str, Vec<String>>);

impl Given {
    /// The values of switch `name`, if it was given.
    pub(crate) fn get(&self, name: &str) -> Option<&[String]> {
        self.0.get(name).map(Vec::as_slice)
    }

    /// The switches in `matches`.
    fn from_matches(matches: &ArgMatches) -> Self {
        let found = SWITCHES.iter().filter_map(|spec| {
            let values = given(matches, spec)?;
            Some((spec.name(), values))
        });
        Self(found.collect())
    }

    /// `Given` holding `pairs`, for tests.
    #[cfg(test)]
    pub(crate) fn of(pairs: &[(&'static str, &str)]) -> Self {
        let mut given = Self::default();
        for (name, value) in pairs {
            given.0.entry(name).or_default().push((*value).to_owned());
        }
        given
    }
}

/// The values of `spec` in `matches`, if given.
fn given(matches: &ArgMatches, spec: &SwitchSpec) -> Option<Vec<String>> {
    if !spec.takes_value() {
        let on = matches.try_get_one::<bool>(spec.name()).ok()??;
        return on.then(|| vec![TRUE.to_owned()]);
    }
    let values = matches.try_get_many::<String>(spec.name()).ok()??;
    Some(values.cloned().collect())
}

/// How a flag that is on is held.
pub(crate) const TRUE: &str = "true";

impl Args for Given {
    fn augment_args(cmd: Command) -> Command {
        cmd.args(SWITCHES.iter().map(arg))
    }

    fn augment_args_for_update(cmd: Command) -> Command {
        Self::augment_args(cmd)
    }
}

impl FromArgMatches for Given {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        Ok(Self::from_matches(matches))
    }

    fn update_from_arg_matches(
        &mut self,
        matches: &ArgMatches,
    ) -> Result<(), clap::Error> {
        *self = Self::from_matches(matches);
        Ok(())
    }
}

/// The `clap` argument for `spec`: a flag, a single value, or a value
/// that may repeat (src/cli:V9).
fn arg(spec: &'static SwitchSpec) -> Arg {
    let base = Arg::new(spec.name())
        .long(spec.name())
        .help(help::switch_help(spec))
        .help_heading(heading(spec.role));
    let action = match (spec.takes_value(), spec.repeats()) {
        (false, _) => return base.action(ArgAction::SetTrue),
        (true, false) => ArgAction::Set,
        (true, true) => ArgAction::Append,
    };
    base.action(action).value_name(value_name(spec))
}

/// The `--help` section a switch of `role` is listed under.
const fn heading(role: Role) -> &'static str {
    match role {
        Role::Gate(..) | Role::Procs | Role::Custom => "Gates",
        Role::Select => "Selectors",
        Role::Wait => "Waiting",
        Role::Run => "Running",
        Role::Remote => "Remote",
        Role::Meta => "Output and profiles",
    }
}

/// The placeholder for `spec`'s value in `--help`.
fn value_name(spec: &SwitchSpec) -> &'static str {
    match spec.value {
        Value::Bytes | Value::MemNeed => "SIZE",
        Value::Rate => "RATE",
        Value::Duration => "D",
        Value::Celsius => "TEMP",
        Value::Window => "HH:MM-HH:MM",
        Value::Level | Value::LoadWindow | Value::Cgroup => "MODE",
        Value::Path => "PATH",
        Value::Text if spec.role == Role::Custom => "CMD",
        Value::Text | Value::Host => "HOST",
        Value::Device | Value::Iface | Value::Name => "NAME",
        Value::ProcLimit => "NAME=N",
        Value::StartRate => "N/D",
        Value::Flag | Value::Decimal | Value::Percent | Value::Count => "X",
    }
}

/// The verbs and top-level flags argv may start with; anything else
/// starts the default verb, `run` (src/cli:V1).
const LEADERS: [&str; 12] = [
    "run",
    "check",
    "status",
    "advise",
    "profiles",
    "completions",
    "man",
    "debug",
    "-h",
    "--help",
    "-V",
    "--version",
];

/// `args` with `run` put in front of the switches when no verb leads.
pub(crate) fn with_default_verb(mut args: Vec<OsString>) -> Vec<OsString> {
    let first = args.get(1).and_then(|a| a.to_str());
    if !first.is_some_and(|a| LEADERS.contains(&a)) {
        args.insert(1.min(args.len()), OsString::from("run"));
    }
    args
}

/// Parses `args` (with the program name first) into a verb.
///
/// # Errors
///
/// `clap`'s error: a usage error, or the help or version text to print.
pub(crate) fn parse(args: Vec<OsString>) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(with_default_verb(args))
}

/// The whole command, for help, completions and the man page.
pub(crate) fn command() -> Command {
    <Cli as clap::CommandFactory>::command()
}

#[cfg(test)]
mod tests;
