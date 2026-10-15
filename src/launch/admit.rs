//! [`Slots`]: the slot steps as the wait's [`Admit`], so one
//! [`poll`](crate::wait::poll) serves a command with slot switches and one
//! without (src/wait:V9, src/slot:V9).

use std::time::SystemTime;

use crate::slot::{Release, SlotError, Waiter};
use crate::wait::Admit;

/// The slot steps of one command, or none when it asks for no slot
/// switch.
#[derive(Debug, Default)]
pub struct Slots(Option<Waiter>);

impl Slots {
    /// No slot steps: admits at once, holding nothing.
    #[must_use]
    pub const fn none() -> Self {
        Self(None)
    }

    /// The steps of `waiter`.
    #[must_use]
    pub const fn new(waiter: Waiter) -> Self {
        Self(Some(waiter))
    }
}

impl Admit for Slots {
    type Held = Option<Release>;
    type Error = SlotError;

    /// Runs the slot steps now; `Some(None)` when there are none.
    fn try_admit(&mut self) -> Result<Option<Option<Release>>, SlotError> {
        match &mut self.0 {
            None => Ok(Some(None)),
            Some(waiter) => {
                Ok(waiter.try_release(SystemTime::now())?.map(Some))
            }
        }
    }
}

#[cfg(test)]
mod tests;
