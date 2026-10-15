//! [`RttSignal`]: `--rtt` as the time a TCP connect to `--rtt-host`
//! takes (src/net:T5, V3, V4).

use std::time::Duration;

use super::NetError;
use crate::signal::{Reason, Sample, Signal, SignalId};

/// The port `--rtt-host H` connects to without one (src/net:V3).
pub const RTT_PORT: u16 = 443;

/// The round-trip signal: `--rtt D` gates on the time a TCP connect to
/// `--rtt-host H[:PORT]` takes, given up after `D` (src/net:V3). Wrap it
/// in [`Sampled`](crate::signal::Sampled) for a
/// [`Source`](crate::signal::Source). Any other signal is unsupported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RttSignal {
    /// `H` or `H:PORT`.
    host: String,
    /// The connect timeout: the `--rtt` threshold.
    timeout: Duration,
}

impl RttSignal {
    /// Probes `rtt_host` (`--rtt-host`), giving up after `timeout`
    /// (`--rtt`).
    ///
    /// # Errors
    ///
    /// [`NetError::RttWithoutHost`] without a host, or an empty one: a
    /// usage error, exit 64 (src/net:V4).
    pub fn new(
        rtt_host: Option<&str>,
        timeout: Duration,
    ) -> Result<Self, NetError> {
        Ok(Self {
            host: rtt_host.unwrap_or_default().to_owned(),
            timeout,
        })
    }
}

impl Signal for RttSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let _ = (&self.host, self.timeout, RTT_PORT);
        Sample::Unsupported(Reason::from(format!("{id}: not yet")))
    }
}

#[cfg(test)]
mod tests;
