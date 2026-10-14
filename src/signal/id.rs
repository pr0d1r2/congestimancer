//! [`SignalId`]: one variant per congestion signal in
//! `docs/CONGESTION.md`, named by the switch that gates on it
//! (src/signal:T1, V6).

use std::fmt;

/// One congestion signal: a quantity a gate compares against a threshold.
///
/// Each variant is named by the switch that gates on it ([`name`]), so
/// `--io-busy` gates on [`SignalId::IoBusy`], named `io-busy`. Switches
/// that read the same quantity share one signal: `--all-load` reads
/// [`SignalId::Load`], `--all-mem` and `--needs-mem` read
/// [`SignalId::Mem`] ([`for_switch`]). Selectors (`--disk-path`,
/// `--io-device`, `--load-window`) and custom gates (`--when`) are not
/// signals.
///
/// [`name`]: SignalId::name
/// [`for_switch`]: SignalId::for_switch
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
#[non_exhaustive]
pub enum SignalId {
    /// Load average (`--load`, `--all-load`).
    Load,
    /// Running tasks (`--run-queue`, `--all-run-queue`).
    RunQueue,
    /// CPU idle percent over the last interval (`--cpu-idle`).
    CpuIdle,
    /// CPU stall time, `some avg10` (`--cpu-pressure`).
    CpuPressure,
    /// CPU steal percent over the last interval (`--steal`).
    Steal,
    /// Available memory (`--mem`, `--all-mem`, `--needs-mem`).
    Mem,
    /// Memory stall time, `some avg10` (`--mem-pressure`).
    MemPressure,
    /// Memory stall time, `full avg10` (`--mem-pressure-full`).
    MemPressureFull,
    /// Memory pressure level (`--mem-level`).
    MemLevel,
    /// Swap in plus swap out per second (`--swap-rate`).
    SwapRate,
    /// Dirty plus writeback pages (`--dirty`).
    Dirty,
    /// Free space on the disk path (`--disk`).
    Disk,
    /// Free inodes on the disk path (`--inodes`).
    Inodes,
    /// Device read rate (`--io-read`).
    IoRead,
    /// Device write rate (`--io-write`).
    IoWrite,
    /// Device read plus write rate (`--io`).
    Io,
    /// Device busy percent over the last interval (`--io-busy`).
    IoBusy,
    /// Average time per IO request over the last interval (`--io-latency`).
    IoLatency,
    /// IO requests in flight (`--io-queue`).
    IoQueue,
    /// IO stall time, `some avg10` (`--io-pressure`).
    IoPressure,
    /// IO stall time, `full avg10` (`--io-pressure-full`).
    IoPressureFull,
    /// Interface receive rate (`--net-rx`).
    NetRx,
    /// Interface transmit rate (`--net-tx`).
    NetTx,
    /// Interface receive plus transmit rate (`--net`).
    Net,
    /// Round-trip time to a host (`--rtt`).
    Rtt,
    /// Open files as a percent of the system maximum (`--fds`).
    Fds,
    /// Processes as a percent of the system maximum (`--pids`).
    Pids,
    /// Whether the CPU is free of thermal throttling (`--cool`).
    Cool,
    /// Hottest thermal zone (`--temp`).
    Temp,
    /// Whether the machine runs on mains power (`--on-ac`).
    OnAc,
    /// Battery charge (`--battery`).
    Battery,
    /// Whether Low Power Mode / power saver is off (`--no-low-power`).
    NoLowPower,
    /// Time since the last keyboard or mouse input (`--idle`).
    Idle,
    /// Local time of day (`--between`).
    Between,
    /// GPU utilisation (`--gpu`).
    Gpu,
    /// Free GPU memory (`--gpu-mem`).
    GpuMem,
    /// Processes running by name (`--max-procs`).
    MaxProcs,
    /// Processes owned by a user (`--max-user-procs`).
    MaxUserProcs,
    /// Throttled share of the cgroup's last interval (`--cpu-throttle`).
    CpuThrottle,
}

impl SignalId {
    /// Every signal, in declaration order.
    pub const ALL: [Self; 39] = [
        Self::Load,
        Self::RunQueue,
        Self::CpuIdle,
        Self::CpuPressure,
        Self::Steal,
        Self::Mem,
        Self::MemPressure,
        Self::MemPressureFull,
        Self::MemLevel,
        Self::SwapRate,
        Self::Dirty,
        Self::Disk,
        Self::Inodes,
        Self::IoRead,
        Self::IoWrite,
        Self::Io,
        Self::IoBusy,
        Self::IoLatency,
        Self::IoQueue,
        Self::IoPressure,
        Self::IoPressureFull,
        Self::NetRx,
        Self::NetTx,
        Self::Net,
        Self::Rtt,
        Self::Fds,
        Self::Pids,
        Self::Cool,
        Self::Temp,
        Self::OnAc,
        Self::Battery,
        Self::NoLowPower,
        Self::Idle,
        Self::Between,
        Self::Gpu,
        Self::GpuMem,
        Self::MaxProcs,
        Self::MaxUserProcs,
        Self::CpuThrottle,
    ];

    /// The switch name, without the leading `--`: `io-busy`.
    #[must_use]
    pub fn name(self) -> &'static str {
        NAMES
            .iter()
            .find(|(id, _)| *id == self)
            .map_or("", |(_, name)| name)
    }

    /// The signal a gate switch reads, if it reads one.
    ///
    /// Takes the switch name without `--`; `all-` variants and
    /// `needs-mem` map to the signal they share. Selectors, strategies and
    /// custom gates map to `None`.
    #[must_use]
    pub fn for_switch(switch: &str) -> Option<Self> {
        NAMES
            .iter()
            .chain(&ALIASES)
            .find(|(_, name)| *name == switch)
            .map(|(id, _)| *id)
    }

    /// Whether a value needs two samples: a rate or a percent over an
    /// interval, which reads [`Reading::Warming`] first (src/signal:V1).
    ///
    /// [`Reading::Warming`]: super::Reading::Warming
    #[must_use]
    pub fn is_delta(self) -> bool {
        DELTA.contains(&self)
    }
}

/// Every signal with its switch name.
const NAMES: [(SignalId, &str); 39] = [
    (SignalId::Load, "load"),
    (SignalId::RunQueue, "run-queue"),
    (SignalId::CpuIdle, "cpu-idle"),
    (SignalId::CpuPressure, "cpu-pressure"),
    (SignalId::Steal, "steal"),
    (SignalId::Mem, "mem"),
    (SignalId::MemPressure, "mem-pressure"),
    (SignalId::MemPressureFull, "mem-pressure-full"),
    (SignalId::MemLevel, "mem-level"),
    (SignalId::SwapRate, "swap-rate"),
    (SignalId::Dirty, "dirty"),
    (SignalId::Disk, "disk"),
    (SignalId::Inodes, "inodes"),
    (SignalId::IoRead, "io-read"),
    (SignalId::IoWrite, "io-write"),
    (SignalId::Io, "io"),
    (SignalId::IoBusy, "io-busy"),
    (SignalId::IoLatency, "io-latency"),
    (SignalId::IoQueue, "io-queue"),
    (SignalId::IoPressure, "io-pressure"),
    (SignalId::IoPressureFull, "io-pressure-full"),
    (SignalId::NetRx, "net-rx"),
    (SignalId::NetTx, "net-tx"),
    (SignalId::Net, "net"),
    (SignalId::Rtt, "rtt"),
    (SignalId::Fds, "fds"),
    (SignalId::Pids, "pids"),
    (SignalId::Cool, "cool"),
    (SignalId::Temp, "temp"),
    (SignalId::OnAc, "on-ac"),
    (SignalId::Battery, "battery"),
    (SignalId::NoLowPower, "no-low-power"),
    (SignalId::Idle, "idle"),
    (SignalId::Between, "between"),
    (SignalId::Gpu, "gpu"),
    (SignalId::GpuMem, "gpu-mem"),
    (SignalId::MaxProcs, "max-procs"),
    (SignalId::MaxUserProcs, "max-user-procs"),
    (SignalId::CpuThrottle, "cpu-throttle"),
];

/// Switches that read the signal of another, plainer switch.
const ALIASES: [(SignalId, &str); 4] = [
    (SignalId::Load, "all-load"),
    (SignalId::RunQueue, "all-run-queue"),
    (SignalId::Mem, "all-mem"),
    (SignalId::Mem, "needs-mem"),
];

/// The signals measured over an interval: rates and interval percents.
const DELTA: [SignalId; 12] = [
    SignalId::CpuIdle,
    SignalId::Steal,
    SignalId::SwapRate,
    SignalId::IoRead,
    SignalId::IoWrite,
    SignalId::Io,
    SignalId::IoBusy,
    SignalId::IoLatency,
    SignalId::NetRx,
    SignalId::NetTx,
    SignalId::Net,
    SignalId::CpuThrottle,
];

/// Renders the switch name: `io-busy`.
impl fmt::Display for SignalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests;
