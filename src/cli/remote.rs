//! `--host H`: the gates go to H over ssh (`src/remote`); the wait, the
//! slots and the command stay here (src/remote:V1). [`Remote`] asks one
//! round; [`wait_remote`] keeps asking until the gates pass, and it is a
//! `--govern` [`Judge`] too.

use std::io::Write;
use std::time::Instant;

use super::SWITCHES;
use super::args::TRUE;
use super::exit::{Failure, check_code, slot_failure};
use super::layers::Resolved;
use super::table::Role;
use crate::gate::{Evaluation, Overall};
use crate::launch::{Judge, Slots};
use crate::remote::{Host, Round, Transport, check_remote};
use crate::signal::Clock;
use crate::slot::Release;
use crate::wait::{Admit, MIN_INTERVAL, Outcome, Pacing};

/// Whether a switch of `role` goes to H: gates, selectors and custom
/// gates do; waiting, running and output stay here (src/remote:V1).
const fn goes_remote(role: Role) -> bool {
    matches!(
        role,
        Role::Gate(..) | Role::Procs | Role::Custom | Role::Select
    )
}

/// The switches H evaluates, each value its own argv word
/// (src/remote:V5); an off flag is left out.
pub(crate) fn gate_args(resolved: &Resolved) -> Vec<String> {
    let mut args = Vec::new();
    for spec in SWITCHES.iter().filter(|s| goes_remote(s.role)) {
        let flag = format!("--{}", spec.name());
        for value in resolved.all(spec.name()) {
            if spec.takes_value() {
                args.extend([flag.clone(), value.clone()]);
            } else if value == TRUE {
                args.push(flag.clone());
            }
        }
    }
    args
}

/// One host's gates, asked over a transport.
pub(crate) struct Remote<'a> {
    /// How ssh is run.
    pub(crate) ssh: &'a mut dyn Transport,
    /// The host.
    pub(crate) host: Host,
    /// The gate switches, as argv words.
    pub(crate) args: Vec<String>,
    /// Where `--verbose` and failures are written.
    pub(crate) log: &'a mut dyn Write,
    /// Whether to say why each round blocks.
    pub(crate) verbose: bool,
}

impl Remote<'_> {
    /// One round: what H says of the gates; an ssh failure is a failed
    /// round, never an exit (src/remote:V3).
    ///
    /// # Errors
    ///
    /// The remote error's exit: 64 usage, 69 missing cgmr or schema.
    pub(crate) fn round(&mut self) -> Result<Overall, Failure> {
        let round = check_remote(self.ssh, &self.host, &self.args)
            .map_err(|e| Failure::new(e.exit_code(), e))?;
        let (overall, reasons) = match round {
            Round::Report(report) => (report.overall(), report.block_reasons()),
            Round::Retry(reason) => (Overall::Fail, vec![reason.to_string()]),
        };
        if self.verbose || overall == Overall::Unsupported {
            for reason in reasons {
                let _ = writeln!(self.log, "cgmr: {reason}");
            }
        }
        Ok(overall)
    }
}

/// `--govern --host H`: each round asks H; a round that cannot be asked
/// counts as failed, so the command pauses rather than runs blind.
impl Judge for Remote<'_> {
    fn judge(&mut self, _now: Instant) -> Overall {
        self.round().unwrap_or(Overall::Fail)
    }
}

/// `check --host H`: 0 every gate passed, 1 one failed or H could not be
/// reached, 69 one is unsupported there.
///
/// # Errors
///
/// The remote error's exit.
pub(crate) fn check_on(remote: &mut Remote<'_>) -> Result<u8, Failure> {
    remote.round().map(check_code)
}

/// Waits until H says every gate passes and the slot steps admit, as
/// `pacing` times it (`--interval`, `--timeout`).
///
/// # Errors
///
/// The remote error's exit, or a slot step that cannot run.
pub(crate) fn wait_remote(
    remote: &mut Remote<'_>,
    pacing: &Pacing,
    clock: &mut dyn Clock,
    slots: &mut Slots,
) -> Result<Outcome<Option<Release>>, Failure> {
    let start = clock.now();
    loop {
        if let Some(outcome) = step(remote.round()?, slots)? {
            return Ok(outcome);
        }
        let waited = clock.now().saturating_duration_since(start);
        if pacing.timeout.is_some_and(|t| waited >= t) {
            return Ok(Outcome::TimedOut(None));
        }
        clock.sleep(pacing.interval.max(MIN_INTERVAL));
    }
}

/// How a wait ends.
type Waited = Outcome<Option<Release>>;

/// The outcome after a round that said `overall`, or `None` to wait on.
fn step(
    overall: Overall,
    slots: &mut Slots,
) -> Result<Option<Waited>, Failure> {
    match overall {
        Overall::Pass => {
            let held = slots.try_admit().map_err(slot_failure)?;
            Ok(held.map(Outcome::Released))
        }
        Overall::Unsupported => Ok(Some(Outcome::Unsupported(Evaluation {
            verdicts: Vec::new(),
            overall,
        }))),
        Overall::Fail | Overall::Warming => Ok(None),
    }
}

/// A transport that answers from a script, for tests: each call takes
/// the next reply, the last one repeats; no reply is a spawn error.
#[cfg(test)]
#[derive(Debug, Default)]
pub(crate) struct ScriptedSsh {
    /// The replies still to give.
    pub(crate) replies: Vec<crate::remote::SshReply>,
    /// The argv of every call.
    pub(crate) calls: Vec<Vec<String>>,
}

#[cfg(test)]
impl ScriptedSsh {
    /// A transport that always answers `code` with `stdout`.
    pub(crate) fn always(code: i32, stdout: &str) -> Self {
        let reply = crate::remote::SshReply::new(Some(code), stdout, "");
        Self {
            replies: vec![reply],
            calls: Vec::new(),
        }
    }
}

#[cfg(test)]
impl Transport for ScriptedSsh {
    fn run(
        &mut self,
        _program: &str,
        argv: &[String],
    ) -> std::io::Result<crate::remote::SshReply> {
        self.calls.push(argv.to_vec());
        if self.replies.len() > 1 {
            return Ok(self.replies.remove(0));
        }
        let last = self.replies.first().cloned();
        last.ok_or_else(|| std::io::ErrorKind::NotFound.into())
    }
}

#[cfg(test)]
mod tests;
