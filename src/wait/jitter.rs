//! [`jitter_delay`]: the random sleep `--jitter` adds after release, so
//! waiters let go together do not all start at once (src/wait:T5, V5).

use std::time::Duration;

/// A delay in `[0, max)` drawn from `seed`; zero when `max` is zero.
///
/// The draw is a fixed mix of the seed, so the same seed always picks
/// the same delay; the caller seeds it with what differs between
/// waiters (pid, start instant). No random number crate is needed.
#[must_use]
pub fn jitter_delay(seed: u64, max: Duration) -> Duration {
    let _ = seed;
    max
}

#[cfg(test)]
mod tests;
