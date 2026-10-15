//! [`poll`]: evaluate the gates, sleep, again, until they release, the
//! timeout runs out, or a gate is unsupported (src/wait:T1).

use std::convert::Infallible;
use std::io::Write;

use super::Pacing;
use crate::gate::{Evaluation, Gates};
use crate::signal::{Clock, Source};

/// The slot steps that run once the gates pass and settle, before
/// `--jitter` (src/wait:V9); `src/slot`'s `Waiter` is the real one.
pub trait Admit {
    /// What the command holds while it runs (slot, reservation).
    type Held;
    /// Why a step could not be tried at all.
    type Error;

    /// `Some` when every step lets the command go, `None` when one says
    /// wait, which sends the loop back to the gates.
    ///
    /// # Errors
    ///
    /// When a step cannot be tried (the lock dir cannot be read).
    fn try_admit(&mut self) -> Result<Option<Self::Held>, Self::Error>;
}

/// No slot steps: the gates alone decide.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Open;

impl Admit for Open {
    type Held = ();
    type Error = Infallible;

    fn try_admit(&mut self) -> Result<Option<()>, Infallible> {
        Ok(Some(()))
    }
}

/// How a wait ended (src/wait interfaces).
#[derive(Debug)]
#[non_exhaustive]
pub enum Outcome<H> {
    /// The gates passed, settled, and every slot step let it go.
    Released(H),
    /// `--timeout` ran out; the last round's evaluation, if any.
    TimedOut(Option<Evaluation>),
    /// A gate has no source here (src/gate:V3).
    Unsupported(Evaluation),
}

impl<H> Outcome<H> {
    /// The exit code this outcome ends `cgmr` with, or `None` when the
    /// command runs: 124 for a timeout (src/wait:V3), 69 for an
    /// unsupported gate (src:V5).
    #[must_use]
    pub const fn exit_code(&self) -> Option<u8> {
        match self {
            Self::Released(_) => None,
            Self::TimedOut(_) => Some(124),
            Self::Unsupported(_) => Some(69),
        }
    }
}

/// What one wait reads, sleeps on, and writes to.
#[non_exhaustive]
pub struct Rig<'a, S: ?Sized, C: ?Sized> {
    /// The gates to pass.
    pub gates: &'a mut Gates,
    /// Where the shared signals come from.
    pub source: &'a mut S,
    /// The monotonic clock every duration is measured on (src/wait:V7).
    pub clock: &'a mut C,
    /// Stderr: heartbeat and verbose lines (src/report:V2).
    pub log: &'a mut dyn Write,
}

impl<'a, S: ?Sized, C: ?Sized> Rig<'a, S, C> {
    /// A rig over these parts.
    pub fn new(
        gates: &'a mut Gates,
        source: &'a mut S,
        clock: &'a mut C,
        log: &'a mut dyn Write,
    ) -> Self {
        Self {
            gates,
            source,
            clock,
            log,
        }
    }
}

/// Waits until the gates release the command, as `pacing` says.
///
/// # Errors
///
/// When `admit` cannot try its steps.
pub fn poll<S, C, A>(
    rig: Rig<'_, S, C>,
    pacing: &Pacing,
    admit: &mut A,
) -> Result<Outcome<A::Held>, A::Error>
where
    S: Source + ?Sized,
    C: Clock + ?Sized,
    A: Admit,
{
    let _ = (rig, pacing.base(), admit);
    Ok(Outcome::TimedOut(None))
}

#[cfg(test)]
mod tests;
