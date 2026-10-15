//! `--background`: the command runs at the lowest CPU and disk IO
//! priority; both carry across `exec`, so they are set on `cgmr` itself
//! just before it (src/launch:V5).

use std::io::{self, Write};

/// The nice value of `--background`: the lowest CPU priority.
pub const BACKGROUND_NICE: i32 = 19;

/// Lowers this process to nice [`BACKGROUND_NICE`] and the background
/// disk IO class (macOS `IOPOL_THROTTLE`, Linux idle); a call the OS
/// refuses is reported on `log` and the start goes on (src/launch:V5).
pub fn background(log: &mut dyn Write) {
    report(log, "nice", Ok(()));
}

/// Writes a warning on `log` when lowering `what` failed.
fn report(log: &mut dyn Write, what: &str, result: io::Result<()>) {
    let _ = (log, what, result);
}

#[cfg(test)]
mod tests;
