//! [`PowerSignal`]: the thermal and power gates, from Linux sysfs under a
//! [`Root`] or macOS notify and IOKit state (src/power:T2, T3, T4, T6,
//! T7).

use std::io;

use super::{
    Supply, battery_charge, is_cpu_cooling, on_mains, parse_cpu_list,
    parse_millicelsius, parse_supply_uevent, parse_sysfs_u64, supplies_of,
};
use crate::signal::{Measure, Reason, Root, Sample, Signal, SignalId};
use crate::sys::{THERMAL_PRESSURE, notify_state, power_state, read_text};
use crate::units::{Celsius, Percent};

/// The highest sysfs index probed for zones and cooling devices.
const MAX_INDEX: u32 = 1024;

/// The per-CPU `thermal_throttle` counters summed for `--cool`.
const THROTTLE_FILES: [&str; 2] =
    ["core_throttle_count", "package_throttle_count"];

/// Which OS's sources a [`PowerSignal`] reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Os {
    /// sysfs under the root.
    Linux,
    /// notify(3) and IOKit through `crate::sys`.
    Macos,
}

/// The thermal and power signals: `--cool`, `--temp`, `--on-ac`,
/// `--battery`, `--no-low-power`. Wrap it in
/// [`Sampled`](crate::signal::Sampled) for a
/// [`Source`](crate::signal::Source).
///
/// `--cool` on Linux compares the CPUs' summed `thermal_throttle`
/// counters with the previous sample; where there are none (AMD, ARM) or
/// on the first sample, it asks whether any CPU cooling device is active
/// (src/power:V1, R3). macOS reads thermal pressure (R1). Any other
/// signal is unsupported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerSignal {
    /// The directory that stands for `/`.
    root: Root,
    /// The OS whose sources are read.
    os: Os,
    /// The summed throttle counters at the previous `--cool` sample.
    throttles: Option<u64>,
    /// The `/sys/class/power_supply` entries to read; `None` until
    /// `crate::sys` can list a directory.
    supply_names: Option<Vec<String>>,
}

impl PowerSignal {
    /// Reads this OS's sources, Linux files under `root`.
    #[must_use]
    pub fn new(root: Root) -> Self {
        let os = if cfg!(target_os = "macos") {
            Os::Macos
        } else {
            Os::Linux
        };
        Self::on(root, os)
    }

    /// Reads `os`'s sources.
    const fn on(root: Root, os: Os) -> Self {
        Self {
            root,
            os,
            throttles: None,
            supply_names: None,
        }
    }

    /// The text of the absolute `path` under the root.
    fn text(&self, path: &str) -> io::Result<String> {
        read_text(&self.root, path)
    }

    /// `(index, text)` of `<prefix><index>/<file>` from index 0 up to the
    /// first that does not exist; unreadable ones are skipped.
    fn indexed<'a>(
        &'a self,
        prefix: &'a str,
        file: &'a str,
    ) -> impl Iterator<Item = (u32, String)> + 'a {
        (0..MAX_INDEX)
            .map(move |i| (i, self.text(&format!("{prefix}{i}/{file}"))))
            .take_while(|(_, text)| {
                text.as_ref().map_or_else(
                    |e| e.kind() != io::ErrorKind::Unsupported,
                    |_| true,
                )
            })
            .filter_map(|(i, text)| Some((i, text.ok()?)))
    }
}

impl PowerSignal {
    /// `--cool`: no thermal throttling now (src/power:V1).
    fn cool(&mut self) -> Result<bool, Reason> {
        match self.os {
            Os::Linux => self.linux_cool(),
            Os::Macos => notify_state(THERMAL_PRESSURE)
                .map(|level| level == 0)
                .map_err(reason),
        }
    }

    /// Throttle counters unchanged since the last sample, else no CPU
    /// cooling device active, else (first sample) cool.
    fn linux_cool(&mut self) -> Result<bool, Reason> {
        let total = self.throttle_total();
        let previous = std::mem::replace(&mut self.throttles, total);
        if let (Some(before), Some(now)) = (previous, total) {
            return Ok(now <= before);
        }
        match self.cooling_active() {
            Some(active) => Ok(!active),
            None if total.is_some() => Ok(true),
            None => Err(Reason::from(
                "no thermal_throttle counters or CPU cooling devices",
            )),
        }
    }

    /// The online CPUs' throttle counters summed; `None` when none has
    /// them (src/power:R3).
    fn throttle_total(&self) -> Option<u64> {
        let online = self.text("/sys/devices/system/cpu/online").ok()?;
        parse_cpu_list(&online)
            .ok()?
            .into_iter()
            .flat_map(|cpu| THROTTLE_FILES.map(|name| (cpu, name)))
            .filter_map(|(cpu, name)| {
                let dir = format!("/sys/devices/system/cpu/cpu{cpu}");
                let path = format!("{dir}/thermal_throttle/{name}");
                parse_sysfs_u64(name, &self.text(&path).ok()?).ok()
            })
            .reduce(u64::saturating_add)
    }

    /// Whether any CPU cooling device is active; `None` without one.
    fn cooling_active(&self) -> Option<bool> {
        let prefix = "/sys/class/thermal/cooling_device";
        let states: Vec<u64> = self
            .indexed(prefix, "type")
            .filter(|(_, kind)| is_cpu_cooling(kind))
            .filter_map(|(i, _)| {
                let text = self.text(&format!("{prefix}{i}/cur_state")).ok()?;
                parse_sysfs_u64("cur_state", &text).ok()
            })
            .collect();
        (!states.is_empty()).then(|| states.iter().any(|&state| state > 0))
    }

    /// `--temp`: the hottest thermal zone (src/power:V2).
    fn temp(&self) -> Result<Celsius, Reason> {
        if self.os == Os::Macos {
            return Err(Reason::from("no public temperature API on macOS"));
        }
        self.indexed("/sys/class/thermal/thermal_zone", "temp")
            .filter_map(|(_, text)| parse_millicelsius(&text).ok())
            .max()
            .ok_or_else(|| {
                Reason::from("no thermal zones under /sys/class/thermal")
            })
    }
}

impl PowerSignal {
    /// The machine's power supplies (src/power:V3, V4).
    fn supplies(&self) -> Result<Vec<Supply>, Reason> {
        let _ = (&self.supply_names, power_state, supplies_of);
        let _ = parse_supply_uevent;
        Err(Reason::from("stub"))
    }

    /// `--battery`: the machine's charge; full with no battery
    /// (src/power:V4).
    fn battery(&self) -> Result<Percent, Reason> {
        let _ = battery_charge;
        Err(Reason::from("stub"))
    }
}

impl Signal for PowerSignal {
    fn sample(&mut self, id: SignalId) -> Sample {
        let sample = match id {
            SignalId::Cool => self.cool().map(Measure::Flag),
            SignalId::Temp => self.temp().map(Measure::Celsius),
            SignalId::OnAc => {
                self.supplies().map(|s| Measure::Flag(on_mains(&s)))
            }
            SignalId::Battery => self.battery().map(Measure::Percent),
            _ => Err(Reason::from(format!("{id} is not a power signal"))),
        };
        sample.map_or_else(Sample::Unsupported, Sample::Gauge)
    }
}

/// An error as the reason a signal has no value.
fn reason(err: impl ToString) -> Reason {
    Reason::from(err.to_string())
}

#[cfg(test)]
mod tests;
