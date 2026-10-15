//! [`HostSource`]: this machine's signals, each routed to the domain node
//! that reads it, built only for the signals asked for (src/signal:V3),
//! and laid under the cgroup with [`machine`] (src/cgroup:V2).

use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::exit::Failure;
use super::layers::Resolved;
use crate::cgroup::{Scope, Scoped, load_warning, parse_cgroup_mode};
use crate::cpu::{CpuSignal, parse_load_window};
use crate::disk::DiskSignal;
use crate::gpu::GpuSignal;
use crate::io::{IoSignal, parse_io_device};
use crate::limits::LimitsSignal;
use crate::mem::MemSignal;
use crate::net::{NetSignal, RttSignal, parse_net_iface};
use crate::power::PowerSignal;
use crate::presence::PresenceSignal;
use crate::signal::{Reading, Root, Sampled, SignalId, SignalSet, Source};
use crate::units::{parse_count, parse_duration};

/// The connect timeout of an `--rtt` probe `status` makes without one.
const RTT_PROBE: Duration = Duration::from_secs(1);

/// The node that reads a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Domain {
    /// `src/cpu`.
    Cpu,
    /// `src/mem`.
    Mem,
    /// `src/disk`.
    Disk,
    /// `src/io`.
    Io,
    /// `src/net` rates.
    Net,
    /// `src/net` round trip.
    Rtt,
    /// `src/limits`.
    Limits,
    /// `src/power`.
    Power,
    /// `src/presence`.
    Presence,
    /// `src/gpu`.
    Gpu,
}

/// The domain that reads each signal on the shared source; the process
/// gates read their own sources and `--cpu-throttle` the cgroup's.
const DOMAINS: [(SignalId, Domain); 36] = {
    use Domain as D;
    use SignalId as S;
    [
        (S::Load, D::Cpu),
        (S::RunQueue, D::Cpu),
        (S::CpuIdle, D::Cpu),
        (S::CpuPressure, D::Cpu),
        (S::Steal, D::Cpu),
        (S::Mem, D::Mem),
        (S::MemPressure, D::Mem),
        (S::MemPressureFull, D::Mem),
        (S::MemLevel, D::Mem),
        (S::SwapRate, D::Mem),
        (S::Dirty, D::Mem),
        (S::Disk, D::Disk),
        (S::Inodes, D::Disk),
        (S::IoRead, D::Io),
        (S::IoWrite, D::Io),
        (S::Io, D::Io),
        (S::IoBusy, D::Io),
        (S::IoLatency, D::Io),
        (S::IoQueue, D::Io),
        (S::IoPressure, D::Io),
        (S::IoPressureFull, D::Io),
        (S::NetRx, D::Net),
        (S::NetTx, D::Net),
        (S::Net, D::Net),
        (S::Rtt, D::Rtt),
        (S::Fds, D::Limits),
        (S::Pids, D::Limits),
        (S::Cool, D::Power),
        (S::Temp, D::Power),
        (S::OnAc, D::Power),
        (S::Battery, D::Power),
        (S::NoLowPower, D::Power),
        (S::Idle, D::Presence),
        (S::Between, D::Presence),
        (S::Gpu, D::Gpu),
        (S::GpuMem, D::Gpu),
    ]
};

/// The domain that reads `id` on the shared source, if one does.
fn domain(id: SignalId) -> Option<Domain> {
    DOMAINS.iter().find(|(s, _)| *s == id).map(|(_, d)| *d)
}

/// One source per domain.
type Parts = Vec<(Domain, Box<dyn Source>)>;

/// What the selectors pick: the path, device, interface, GPU, window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Selectors {
    /// Where kernel files are read.
    root: Root,
    /// `--disk-path`.
    disk_path: Option<PathBuf>,
    /// `--rtt-host`.
    rtt_host: Option<String>,
    /// `--rtt`: the probe's timeout.
    rtt: Duration,
}

impl Selectors {
    /// The selectors `resolved` sets, reading under `root`.
    ///
    /// # Errors
    ///
    /// A usage error for a value that does not parse.
    pub(crate) fn new(
        resolved: &Resolved,
        root: Root,
    ) -> Result<Self, Failure> {
        Ok(Self {
            root,
            disk_path: resolved.text("disk-path").map(PathBuf::from),
            rtt_host: resolved.text("rtt-host").map(str::to_owned),
            rtt: resolved.parse("rtt", parse_duration)?.unwrap_or(RTT_PROBE),
        })
    }
}

/// This machine's shared signals: one source per domain asked for.
pub(crate) struct HostSource {
    /// The domains built, each with its source.
    parts: Parts,
}

impl HostSource {
    /// The sources for the signals in `wanted`.
    ///
    /// # Errors
    ///
    /// A usage error for a selector this machine lacks: a missing
    /// `--disk-path`, an unknown `--net-iface` or `--gpu-device`, `--rtt`
    /// without `--rtt-host`.
    pub(crate) fn new(
        resolved: &Resolved,
        selectors: &Selectors,
        wanted: SignalSet,
    ) -> Result<Self, Failure> {
        let mut parts = Parts::new();
        for d in wanted.iter().filter_map(domain) {
            if parts.iter().all(|(have, _)| *have != d) {
                parts.push((d, part(d, resolved, selectors)?));
            }
        }
        Ok(Self { parts })
    }
}

impl Source for HostSource {
    fn read(&mut self, id: SignalId, now: Instant) -> Reading {
        let wanted = domain(id);
        let part = self.parts.iter_mut().find(|(d, _)| Some(*d) == wanted);
        part.map_or_else(
            || Reading::unsupported(format!("--{id} has no source here")),
            |(_, source)| source.read(id, now),
        )
    }
}

/// The source of domain `d`.
fn part(
    d: Domain,
    resolved: &Resolved,
    sel: &Selectors,
) -> Result<Box<dyn Source>, Failure> {
    let root = sel.root.clone();
    Ok(match d {
        Domain::Cpu => Box::new(Sampled::new(cpu(resolved, root)?)),
        Domain::Mem => Box::new(Sampled::new(MemSignal::new(root))),
        Domain::Disk => Box::new(Sampled::new(disk(sel)?)),
        Domain::Io => Box::new(Sampled::new(io(resolved, sel)?)),
        Domain::Net => Box::new(net(resolved, root)?),
        Domain::Rtt => Box::new(Sampled::new(rtt(sel)?)),
        Domain::Limits => Box::new(Sampled::new(LimitsSignal::new(root))),
        Domain::Power => Box::new(Sampled::new(PowerSignal::new(root))),
        Domain::Presence => Box::new(Sampled::new(PresenceSignal::new())),
        Domain::Gpu => Box::new(Sampled::new(gpu(resolved, root)?)),
    })
}

/// A usage error from a domain's selector error.
fn usage(err: impl std::fmt::Display) -> Failure {
    Failure::usage(err)
}

/// `src/cpu` over `--load-window`.
fn cpu(resolved: &Resolved, root: Root) -> Result<CpuSignal, Failure> {
    let window = resolved.parse("load-window", parse_load_window)?;
    Ok(CpuSignal::new(root).with_window(window.unwrap_or_default()))
}

/// `src/disk` on `--disk-path`.
fn disk(sel: &Selectors) -> Result<DiskSignal, Failure> {
    DiskSignal::at(sel.disk_path.as_deref()).map_err(usage)
}

/// `src/io` on `--disk-path`'s device, or `--io-device`.
fn io(resolved: &Resolved, sel: &Selectors) -> Result<IoSignal, Failure> {
    let io = IoSignal::new(sel.root.clone(), sel.disk_path.as_deref());
    let device = resolved.parse("io-device", parse_io_device)?;
    Ok(match device {
        Some(name) => io.with_device(name),
        None => io,
    })
}

/// `src/net` rates on `--net-iface`.
fn net(resolved: &Resolved, root: Root) -> Result<NetSignal, Failure> {
    let net = NetSignal::new(root);
    match resolved.parse("net-iface", parse_net_iface)? {
        Some(name) => net.iface(name).map_err(usage),
        None => Ok(net),
    }
}

/// `src/net` round trip to `--rtt-host`.
fn rtt(sel: &Selectors) -> Result<RttSignal, Failure> {
    RttSignal::new(sel.rtt_host.as_deref(), sel.rtt).map_err(usage)
}

/// `src/gpu` on `--gpu-device`.
fn gpu(resolved: &Resolved, root: Root) -> Result<GpuSignal, Failure> {
    let gpu = GpuSignal::new(root);
    match resolved.parse("gpu-device", parse_count)? {
        Some(index) => gpu.device(index).map_err(usage),
        None => Ok(gpu),
    }
}

/// The machine's source for `wanted`: the host's, with the cgroup's own
/// files over the signals they answer (src/cgroup:V2); writes the
/// `--load` in a cgroup warning through `warn`.
///
/// # Errors
///
/// A usage error for a bad `--cgroup` or selector.
pub(crate) fn machine(
    resolved: &Resolved,
    root: &Root,
    wanted: SignalSet,
    warn: &mut dyn FnMut(&str),
) -> Result<Scoped<HostSource>, Failure> {
    let mode = resolved.parse("cgroup", parse_cgroup_mode)?;
    let scope = Scope::resolve(root, &mode.unwrap_or_default());
    let scope = scope.map_err(|r| Failure::usage(format!("--cgroup: {r}")))?;
    if let Some(warning) = load_warning(&scope, wanted) {
        warn(&warning);
    }
    let selectors = Selectors::new(resolved, root.clone())?;
    let host = HostSource::new(resolved, &selectors, wanted)?;
    Ok(Scoped::new(root.clone(), scope, host))
}

#[cfg(test)]
mod tests;
