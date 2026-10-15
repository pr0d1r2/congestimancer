//! [`Waiter`]: the slot steps run in one fixed order once gates pass and
//! settle, before jitter; any step that says wait sends the caller back
//! to the gates (src/slot:V9, src/wait:V9).

use std::time::SystemTime;

use super::{
    LockDir, Reservation, SlotError, SlotGuard, StartLimits, Ticket,
    claim_slot, is_held, mark_held, record_start,
};
use crate::units::{Bytes, Count, Name};

/// One slot step of the release order (src/wait:V9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SlotStep {
    /// `--fifo`: the lowest live ticket goes first.
    Fifo,
    /// `--slots`: claim a free slot.
    Slot,
    /// `--rate` and `--spacing`: record the start.
    Starts,
    /// `--needs-mem`: write the reservation.
    Reserve,
}

/// The slot steps in the order they run, after gates pass and `--settle`
/// holds, before `--jitter` (src/slot:V9, src/wait:V9).
pub const SLOT_ORDER: [SlotStep; 4] = [
    SlotStep::Fifo,
    SlotStep::Slot,
    SlotStep::Starts,
    SlotStep::Reserve,
];

/// What a command asks of the slot steps.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SlotPlan {
    /// The slot key (src/slot:V3, V10).
    pub key: Name,
    /// `--slots N`.
    pub slots: Option<Count>,
    /// `--fifo`.
    pub fifo: bool,
    /// `--rate` and `--spacing`.
    pub limits: StartLimits,
    /// `--needs-mem X`.
    pub needs_mem: Option<Bytes>,
}

impl SlotPlan {
    /// A plan for `key` that asks for nothing yet.
    #[must_use]
    pub fn new(key: Name) -> Self {
        Self {
            key,
            slots: None,
            fifo: false,
            limits: StartLimits::default(),
            needs_mem: None,
        }
    }
}

/// What a waiter holds once released: keep it alive until the command
/// exits.
#[derive(Debug)]
pub struct Release {
    /// The slot claimed, if `--slots` asked for one.
    slot: Option<SlotGuard>,
    /// The memory reserved, if `--needs-mem` asked for it.
    reservation: Option<Reservation>,
    /// The new `CGMR_HELD_SLOTS` for the command, when a slot was claimed.
    held_marker: Option<String>,
}

impl Release {
    /// The slot claimed, if any.
    #[must_use]
    pub const fn slot(&self) -> Option<&SlotGuard> {
        self.slot.as_ref()
    }

    /// The memory reserved, if any.
    #[must_use]
    pub const fn reservation(&self) -> Option<&Reservation> {
        self.reservation.as_ref()
    }

    /// The `CGMR_HELD_SLOTS` value to hand the command, when it changed.
    #[must_use]
    pub fn held_marker(&self) -> Option<&str> {
        self.held_marker.as_deref()
    }
}

/// One command waiting on its slot steps; its FIFO ticket lasts across
/// rounds until it is released.
#[derive(Debug)]
pub struct Waiter {
    /// The lock dir.
    dir: LockDir,
    /// What the command asks for.
    plan: SlotPlan,
    /// `CGMR_HELD_SLOTS` as inherited.
    marker: Option<String>,
    /// The FIFO ticket, taken on the first round.
    ticket: Option<Ticket>,
}

impl Waiter {
    /// A waiter for `plan` in `dir`; `marker` is the inherited
    /// `CGMR_HELD_SLOTS` (src/slot:V8).
    #[must_use]
    pub fn new(dir: LockDir, plan: SlotPlan, marker: Option<&str>) -> Self {
        Self {
            dir,
            plan,
            marker: marker.map(str::to_owned),
            ticket: None,
        }
    }

    /// The steps this waiter runs, in [`SLOT_ORDER`]; a key an ancestor
    /// holds skips FIFO and slot (src/slot:V8).
    #[must_use]
    pub fn steps(&self) -> Vec<SlotStep> {
        SLOT_ORDER.into_iter().filter(|&s| self.asks(s)).collect()
    }

    /// Whether this waiter runs `step`.
    fn asks(&self, step: SlotStep) -> bool {
        let held = is_held(self.marker.as_deref(), &self.plan.key);
        match step {
            SlotStep::Fifo => self.plan.fifo && !held,
            SlotStep::Slot => self.plan.slots.is_some() && !held,
            SlotStep::Starts => !self.plan.limits.is_none(),
            SlotStep::Reserve => self.plan.needs_mem.is_some(),
        }
    }

    /// Runs the steps in order at `now`: the release when all pass, or
    /// `None` when one says wait, having let go of anything claimed this
    /// round except the FIFO ticket.
    ///
    /// # Errors
    ///
    /// [`SlotError::Io`] when the lock dir cannot be read or written.
    pub fn try_release(
        &mut self,
        now: SystemTime,
    ) -> Result<Option<Release>, SlotError> {
        if !self.fifo_turn()? {
            return Ok(None);
        }
        let Some(slot) = self.claim()? else {
            return Ok(None);
        };
        if self.asks(SlotStep::Starts) {
            let plan = &self.plan;
            if !record_start(&self.dir, &plan.key, &plan.limits, now)? {
                return Ok(None);
            }
        }
        self.release(slot)
    }

    /// Whether FIFO lets this waiter go on, taking its ticket on the
    /// first round.
    fn fifo_turn(&mut self) -> Result<bool, SlotError> {
        if !self.asks(SlotStep::Fifo) {
            return Ok(true);
        }
        if self.ticket.is_none() {
            self.ticket = Some(Ticket::take(&self.dir, &self.plan.key)?);
        }
        self.ticket.as_ref().map_or(Ok(true), Ticket::is_first)
    }

    /// `Some` slot step outcome (a slot, or none needed), or `None` when
    /// every slot is held.
    fn claim(&self) -> Result<Option<Option<SlotGuard>>, SlotError> {
        match self.plan.slots {
            Some(n) if self.asks(SlotStep::Slot) => {
                Ok(claim_slot(&self.dir, &self.plan.key, n)?.map(Some))
            }
            _ => Ok(Some(None)),
        }
    }

    /// The release: the reservation written, the ticket given up.
    fn release(
        &mut self,
        slot: Option<SlotGuard>,
    ) -> Result<Option<Release>, SlotError> {
        let reservation = self
            .plan
            .needs_mem
            .map(|bytes| Reservation::write(&self.dir, bytes))
            .transpose()?;
        self.ticket = None;
        let held_marker = slot
            .as_ref()
            .map(|_| mark_held(self.marker.as_deref(), &self.plan.key));
        Ok(Some(Release {
            slot,
            reservation,
            held_marker,
        }))
    }
}

#[cfg(test)]
mod tests;
