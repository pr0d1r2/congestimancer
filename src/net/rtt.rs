//! [`RttSignal`]: `--rtt` as the time a TCP connect to `--rtt-host`
//! takes (src/net:T5, V3, V4).

use std::time::Duration;

use super::NetError;
use crate::signal::{Measure, Reason, Sample, Signal, SignalId};
use crate::sys::tcp_connect_time;

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
        let host = rtt_host.map(str::trim).filter(|h| !h.is_empty());
        let host = host.ok_or(NetError::RttWithoutHost)?;
        Ok(Self {
            host: host.to_owned(),
            timeout,
        })
    }

    /// The connect time to the host; the timeout itself when no connect
    /// succeeds in time, so `--rtt D` fails rather than exits
    /// (src/net:V3).
    fn probe(&self) -> Duration {
        tcp_connect_time(&self.host, RTT_PORT, self.timeout)
            .unwrap_or(self.timeout)
    }
}

impl Signal for RttSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        match id {
            SignalId::Rtt => Sample::Gauge(Measure::Duration(self.probe())),
            _ => Sample::Unsupported(Reason::from(format!(
                "{id} is not a round-trip signal"
            ))),
        }
    }
}

#[cfg(test)]
mod tests;
