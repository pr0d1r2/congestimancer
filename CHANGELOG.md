# Changelog

All notable changes to congestimancer are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Version ladder: an even minor (0.2, 0.4, ...) is a stable rung; an odd minor
is a development series, not for production.

## [Unreleased]

### Added

- `UnitError`: every parse error names the input and the form expected.
- Numbers in every value take `.` as the decimal separator, whatever the
  locale, and a value past 64 bits is an error, never wrapped.
- `Bytes` and `parse_bytes`: sizes like `10GB` or `1.5GiB`; `KB` is 1000,
  `KiB` is 1024, units are case insensitive, fractions round down.
- `Rate` and `parse_rate`: a size per second, like `50MB/s`.
- `Percent` and `parse_percent`: 0 to 100 with an optional `%`, like `12.5%`.
- `parse_duration`: durations like `1h30m` or `500ms`, units `d h m s ms`
  largest first; a bare number is an error.
- `Celsius` and `parse_celsius`: temperatures like `85C` or `72.5C`.
- `ClockWindow`, `ClockTime` and `parse_clock_window`: daily windows like
  `22:00-06:00` in local time, wrapping midnight when start is after end.
- `Count` and `parse_count`: whole numbers of things; a decimal is an error.
- `Decimal` and `parse_decimal`: load-like thresholds that take decimals,
  like `0.5`; no threshold of any kind is negative.
- `Name` and `parse_name`: safe identifiers, 1 to 64 of `A-Z a-z 0-9 . _ -`,
  never a leading `-`, never `.` or `..`.
- Every value renders back as the input that parses to it (`Display`, and
  `render_duration` for durations): `1KiB`, `12.5%`, `1h30m`, `22:00-06:00`.
- `SignalId`: one signal per gate switch in `docs/CONGESTION.md`, named by
  its switch (`io-busy`); `--all-load`, `--all-run-queue`, `--all-mem` and
  `--needs-mem` read the signal of their plain switch.
- `Reading`, `Measure`, `Reason` and `Level`: a signal reads a value with
  its unit (rendered as its switch takes it: `12.5%`, `50MB/s`, `72.5C`),
  `warming up` before a rate has two samples, or `unsupported: <reason>`.
- `Source` and `Clock` traits, and `SystemClock`: every reading is taken at
  a monotonic instant, and every interval is measured on a `Clock`.
- `Signal`, `Sample`, `Sampled`, `Sampler` and `Counter`: rates and
  interval percents come from two samples of a running total; the first
  sample, and any sample after a counter goes back, reads `warming up`,
  never zero and never negative.
- `Root`: the directory kernel files are read under, `/` on the live
  machine or a fixture tree; `..` never climbs above it.
- `SignalSet` and `refresh`: only the signals the requested gates read are
  refreshed; the others are never touched.
- `Trend` and `Trends`: each signal reads `rising`, `falling` or `flat`
  from its last two values.
- `parse_psi`, `Psi`, `PsiLine` and `PsiError`: one parser for Linux
  pressure stall text (`/proc/pressure/*` and cgroup `*.pressure`); a bad
  line is named in the error.
- `parse_profiles`, `ProfileFile`, `Profile`, `Switch` and `ProfileError`:
  profile files of `[name]` sections, one switch per line exactly as on the
  command line, `#` comments; every error names `file:line`, and switches
  are checked by the CLI's own parser through `SwitchTable`.
- `discover`, `Places`, `ReadText` and `Catalog`: profiles come from the
  `--profile-file` alone if given, else the nearest `.cgmr` up from the
  working directory, then `$XDG_CONFIG_HOME/cgmr/profiles` (else
  `~/.config/cgmr/profiles`); the project file wins a name both define, a
  missing file is skipped, an unreadable one is an error, and an unknown
  name lists the names there are.
- `--when` in a profile is allowed only in the user file: in a project
  `.cgmr` or an explicit `--profile-file` it is refused, naming the line,
  so a cloned repository cannot run commands.
- `Profile::layer`: a profile's switches as one config layer, below the
  command line and the environment; a switch set above wins without a
  repeat error, and a single-value switch twice in one profile is refused.
- `render_profiles`: the `cgmr profiles` listing, every profile with its
  file and switches, a name the project file shadows marked as such.
- `read_text`: a kernel text file read under a `Root`; a file this machine
  does not have is an `Unsupported` error naming the path.
- `Snapshot`: load averages, memory and swap, and the process list (name and
  uid only) from one sysinfo view; each part is read only when asked for.
- `statvfs` (space and inodes of the filesystem holding a path, byte totals
  saturating), `try_lock_exclusive` and `unlock` (`flock` locks that never
  wait), and `nice` and `set_nice` for this process.
- `sysctl_int` and `swap_usage`: macOS integer sysctls (such as
  `kern.maxfiles` or `kern.memorystatus_vm_pressure_level`) and
  `vm.swapusage`; an unknown sysctl, or any other OS, is `Unsupported`.
- `cpu_ticks`, `vm_stats` and `pset_load`: macOS per-CPU ticks, VM page
  counts and swap and page events, and the default processor set load (the
  run queue); `Unsupported` on other OSes.
- `mount_source`, `physical_disk_stats` and `block_devices`: on macOS, the
  device holding a path, and read and write bytes, operations and time of a
  physical disk from IOKit, an APFS volume walked up to its disk;
  `PropValue` and `Props` carry IOKit property tables.
- `power_state`, `hid_idle_ns` and `notify_state`: on macOS, whether the
  machine is on wall power with its battery charge, the time since the last
  input, and notify state such as thermal pressure (`THERMAL_PRESSURE`) and
  Low Power Mode (`LOW_POWER_MODE`).
- `gpu_usage`: on macOS, every GPU's utilization and free VRAM (where it has
  dedicated memory) from IOKit `PerformanceStatistics`, with the whole table
  kept; no GPU table is `Unsupported`.
- `set_io_priority` and `io_priority`: this process's disk IO priority,
  `Background` (macOS `IOPOL_THROTTLE`, Linux idle class) or `Default`; it
  carries across exec.
- `nvml_gpus`: NVIDIA GPU utilization and memory through NVML, loaded at run
  time (the `nvml` feature, on by default); no library, no driver or a build
  without the feature is `Unsupported`.
- `interface_counters`: on macOS, every network interface with 64-bit
  received and sent bytes, packets and errors, and whether it is up or
  loopback.
- `tcp_connect_time`: how long a TCP connect to `host[:port]` takes, every
  resolved address tried within one timeout.
- `system_bus_connect`: on Linux, a connection to the D-Bus system bus at
  `DBUS_SYSTEM_BUS_ADDRESS` or `/run/dbus/system_bus_socket`; no bus is
  `Unsupported`.
- `capture`: copies the kernel files the gates read into `out/<os>-<arch>/`
  as a fixture tree, and on macOS writes the sysctl, mach, network, IOKit
  and notify values as text, without serial numbers, link addresses or
  anything that names a person.
- `Cgroup` and `parse_proc_cgroup`: on Linux, the cgroup v2 this process
  sits in, from `/proc/self/cgroup`, with its files and ancestors under
  `/sys/fs/cgroup`; cgroup v1 (or a hybrid host) is unsupported.
- `CpuMax`, `parse_cpu_max`, `parse_memory_max`, `parse_memory_current` and
  `memory_available`: a cgroup's CPU quota and memory limit (`max` is no
  limit), and the memory left to it, capped by what the host has free.
- `CpuStat`, `Throttling`, `parse_cpu_stat` and `CgroupSignal`: a cgroup's
  `cpu.stat`, and `--cpu-throttle` read as the share of the last interval
  its CPU quota stopped it; no `cpu` controller is unsupported.
- `CgroupMode`, `parse_cgroup_mode` and `Scope`: `--cgroup auto|host|PATH`.
  `auto` reads the cgroup when it or an ancestor caps CPU or memory, or in
  a container, and otherwise keeps host values and says why.
- `Scoped` and `CGROUP_SIGNALS`: inside a cgroup, every pressure gate reads
  the cgroup's own `cpu.pressure`, `memory.pressure` and `io.pressure`, and
  `--mem` reads the least memory any limited cgroup above has left, never
  more than the host has available.
- `load_warning`: inside a cgroup, a `--load` gate gets a warning that the
  host's load does not match the container's CPUs, suggesting
  `--cpu-pressure`.
- `Dual` and `Scoped::dual`: `status` can show each cgroup signal as the
  cgroup reads it beside the host's value, and why the cgroup column is
  empty on the host.
- `parse_millicelsius`, `parse_sysfs_u64`, `parse_cpu_list` and
  `is_cpu_cooling`: Linux thermal zone temperatures, CPU throttle counters,
  online CPU lists and which cooling devices throttle the CPU.
- `Supply`, `parse_supply_uevent`, `supplies_of`, `on_mains` and
  `battery_charge`: Linux power supply `uevent` files and macOS IOKit power
  state as one supply list; a desktop without a battery counts as on mains
  power, and a mouse battery or an empty bay is never the machine's.
- `PowerSignal`: `--cool` passes while the CPU is not thermally throttled:
  on Linux the `thermal_throttle` counters did not grow since the last
  sample, else no CPU cooling device is active; on macOS thermal pressure is
  nominal.
- `--temp` reads the hottest Linux thermal zone; on macOS it is unsupported,
  since there is no public temperature API.
- `--on-ac` and `--battery` read macOS IOKit power state: a desktop counts as
  on mains power and passes any `--battery` gate. On Linux both are
  unsupported until the power supply directory can be listed.
- `--no-low-power` passes while macOS Low Power Mode is off, or on Linux
  while the power-profiles-daemon profile is not `power-saver`; Linux
  without the daemon's state is unsupported (`parse_ppd_profile`,
  `PPD_STATE`).
