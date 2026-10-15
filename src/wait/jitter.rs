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
    let span = u128::from(u64::try_from(max.as_nanos()).unwrap_or(u64::MAX));
    let draw = u128::from(mix(seed));
    let nanos = draw
        .checked_mul(span)
        .and_then(|scaled| scaled.checked_shr(64))
        .and_then(|high| u64::try_from(high).ok())
        .unwrap_or(0);
    Duration::from_nanos(nanos)
}

/// `SplitMix64`'s output step: every bit of `seed` reaches every bit of
/// the result, so neighbouring seeds (pids) land far apart.
const fn mix(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests;
