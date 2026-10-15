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
- `set_inheritable`: clears close-on-exec on an open file, so a slot lock
  passes to the command started with `exec`.
- `device_of`: the major and minor number of the device holding a path,
  following symlinks.
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
- `list_dir`: the sorted entry names of a kernel directory under a `Root`;
  a directory this machine lacks is `Unsupported`.
- `user_id`, `user_name` and `current_uid`: a user's uid by name and name
  by uid from the password database, `None` for no such user, and the
  real uid of this process.
- `local_time`: the hour and minute of now in local time, with the time
  zone and any DST change applied by the C library.
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
- `--on-ac` and `--battery` read macOS IOKit power state, or on Linux every
  supply listed under `/sys/class/power_supply`: a desktop counts as on
  mains power and passes any `--battery` gate. Linux without that
  directory is unsupported.
- `--no-low-power` passes while macOS Low Power Mode is off, or on Linux
  while the power-profiles-daemon profile is not `power-saver`; Linux
  without the daemon's state is unsupported (`parse_ppd_profile`,
  `PPD_STATE`).
- `LimitsSignal`: `--fds X` passes while open files are under X percent of
  the system maximum; on Linux from `/proc/sys/fs/file-nr` (`parse_file_nr`,
  `FileNr`). A maximum of 0 or an unreadable source is unsupported, never
  a division (`limit_share`).
- `--fds` on macOS reads `kern.num_files` over `kern.maxfiles`; a failed
  read or a negative count is unsupported.
- `--pids X` passes while processes are under X percent of the system
  maximum: on Linux the tasks in `/proc/loadavg` over
  `/proc/sys/kernel/pid_max` (`parse_loadavg_tasks`, `parse_pid_max`), on
  macOS the running processes over `kern.maxproc`.
- `Glob`: process and user name patterns, `*` for any run of characters
  and `?` for exactly one, matched against the whole name.
- `ProcLimit` and `parse_proc_limit`: the `PATTERN=N` value of
  `--max-procs` and `--max-user-procs`; a value with no `=`, an empty
  pattern or a bad count is a `ProcsError` (exit 64).
- `ProcsSignal` reads `--max-procs`: how many processes have a name
  matching the pattern (`count_by_name`), leaving out cgmr and its
  ancestors where the process table gives pids (`lineage`).
- `--max-user-procs` reads how many processes are owned by a user whose
  name matches the pattern, like `nixbld*` (`count_by_user`); a process
  with no known owner never matches.
- `ProcessInfo` carries each process's `pid` and `parent`, so a live
  process count can leave out cgmr and its ancestors.
- `--max-procs` and `--max-user-procs` leave out cgmr and its ancestors on
  the live machine, not only in fixture tables.
- `DiskSignal` reads `--disk`: the bytes an unprivileged user may still
  write on the filesystem holding a path (`free_space`), from `statvfs` on
  both OSes; a path that is gone is unsupported.
- `--inodes` reads the inodes an unprivileged user may still take on that
  filesystem (`free_inodes`); one that does not count inodes is
  unsupported.
- `DiskSignal::at` reads the working directory, or `--disk-path P`; a `P`
  that does not exist is `DiskError::Missing`, a usage error.
- `parse_net_dev`, `parse_if_flags` and `linux_interfaces`: Linux
  interface totals from `/proc/net/dev`, up and loopback from
  `/sys/class/net/<iface>/flags`; a torn line is a `NetDevError` naming it.
- `NetSignal` reads `--net-rx`, `--net-tx` and `--net`: bytes per second
  over every up, non-loopback interface (`selected`), from
  `/proc/net/dev` on Linux and 64-bit `NET_RT_IFLIST2` counters on macOS;
  an interface that wrapped, came or went warms up, never a huge rate.
- `NetSignal::iface` reads only `--net-iface I`, up or not (`selected`);
  an `I` this machine does not list is `NetError::UnknownIface`, a usage
  error, and one that goes away later is unsupported.
- `RttSignal` reads `--rtt D`: the TCP connect time to `--rtt-host H[:PORT]`,
  port `RTT_PORT` (443) by default, given up after `D`; a failed connect
  reads `D` so the gate fails, and `--rtt` without a host is
  `NetError::RttWithoutHost`, a usage error.
- `parse_net_iface`: `--net-iface I` is a name, so `..`, `/`, a leading `-`
  or a space is `NetError::BadIface`, a usage error naming the input.
- `PresenceSignal`: `--idle` on macOS is the time since the last keyboard
  or mouse input (IOKit `HIDIdleTime`); with no HID system it is unsupported.
- `--idle` on Linux is the least idle local user session logind knows, read
  over the system bus by a hand-written D-Bus client; a headless host or one
  without a bus is unsupported.
- `WallClock` and `SystemWallClock`: `--between` samples the local time of
  day from an injected wall clock; the machine clock reads it with the time
  zone and any DST change applied.
- `amd_gpus`, `parse_busy_percent` and `parse_vram`: AMD GPUs on Linux from
  each DRM card's `gpu_busy_percent` and `mem_info_vram_total|used`, as a
  `Gpu` with its busy percent and free VRAM; an unparsed file leaves its
  field empty, never a fake number.
- `Gpu::from(&NvmlGpu)`: NVIDIA busy percent and free memory from NVML;
  a reading NVML did not give stays empty.
- `Gpu::from(&GpuUsage)`: macOS GPU busy percent from IOKit; Apple Silicon
  has no free GPU memory (unified memory), an Intel Mac GPU reports VRAM.
- `GpuSignal` reads `--gpu` and `--gpu-mem`: every GPU by default, the
  busiest and the least free memory winning (`worst_busy`, `worst_free`);
  `--gpu-device I` reads GPU number `I`, and an index past them is
  `GpuError::UnknownDevice`, a usage error. No GPU source, or `--gpu-mem`
  on Apple Silicon, is unsupported with the reason.
- `LockDir`: the per-user slot dir (`$XDG_RUNTIME_DIR/cgmr`, else
  `$TMPDIR/cgmr`, else `/tmp/cgmr-<uid>`), made 0700 and refused when it
  is a symlink, not a directory, another user's or open to others
  (`SlotError::UnsafeDir`, exit 69).
- `slot_key`: `--slot-key` must be a safe name; without it the command's
  basename is the key, unsafe characters turned to `_`.
- `claim_slot` and `live_holders`: `--slots N` admits fewer than `N` live
  holders of a key, each holding the `flock` on the file `K.<i>`; a holder
  that dies frees its slot.
- `Ticket`: `--fifo` queues waiters by numbered ticket; only the lowest
  live ticket goes, and a ticket whose holder died is skipped.
- `parse_start_rate`, `StartLimits` and `record_start`: `--rate N/D` admits
  at most `N` starts in any window `D` and `--spacing D` keeps starts `D`
  apart, per key, across processes.
- `Reservation` and `reserved_total`: `--needs-mem` reservations held by
  their waiter; the ledger is the sum of the live ones.
- `is_held` and `mark_held`: `CGMR_HELD_SLOTS` names the slot keys an
  ancestor `cgmr` holds, so a nested `cgmr` on one does not deadlock.
- `Waiter`, `SlotPlan` and `Release`: once gates pass and settle, the slot
  steps run in `SLOT_ORDER` (FIFO, slot, rate and spacing, reservation);
  a step that says wait lets go of anything claimed that round but the
  FIFO ticket.
- `MemSignal`: `--all-mem X` passes while available memory, never "free",
  is over X, from `sysinfo` on Linux and macOS; `--mem X` compares
  available memory per CPU (`mem_per_cpu`), never divided by zero.
- `mem_unreserved`: `--needs-mem X` compares available memory less the
  memory running cgmr jobs reserved, never below zero.
- `--mem-pressure X` and `--mem-pressure-full X` pass while the `some` or
  `full` avg10 of `/proc/pressure/memory` is under X; macOS is
  unsupported, pointing at `--mem-level`.
- `--mem-level normal|warning` passes while the macOS memory pressure level
  (`kern.memorystatus_vm_pressure_level`) is at most that level
  (`pressure_level`, `level_within`); Linux is unsupported, pointing at
  `--mem-pressure`.
- `--swap-rate X/s` on Linux passes while pages swapped in plus out, from
  `pswpin` and `pswpout` in `/proc/vmstat` (`parse_vmstat`, `SwapPages`),
  move under X bytes a second; the page size is the smallest
  `KernelPageSize` in `/proc/self/smaps` (`parse_page_size`), and byte
  math past 64 bits is unsupported, never wrapped.
- `--swap-rate X/s` on macOS counts `host_statistics64` swapins and
  swapouts times `hw.pagesize`.
- `--dirty X` passes while `Dirty` plus `Writeback` in `/proc/meminfo`
  (`parse_meminfo`, `DirtyPages`) is under X; macOS is unsupported.
- `MemSignal::status_level`: on macOS, `status` can show the share of
  memory available from `kern.memorystatus_level` (`available_level`);
  Linux has none.
- `cpu_count`: the logical CPUs `--load` and `--run-queue` divide by, from
  `available_parallelism` and never below 1; `load_per_cpu` divides with
  a check, never by zero.
- `CpuSignal` reads the load average for `--load X` (per CPU) and
  `--all-load X`: Linux `/proc/loadavg` (`parse_loadavg`, `Loads`), macOS
  `sysinfo`; a cgroup scope still reads the host load.
- `--load-window 1m|5m|15m` picks the load average `--load` reads,
  `1m` by default (`LoadWindow`, `parse_load_window`).
- `parse_stat` reads `/proc/stat` (`ProcStat`, `CpuTimes`); on Linux
  `--run-queue X` (per CPU) and `--all-run-queue X` gate on
  `procs_running`.
- On macOS `--run-queue X` and `--all-run-queue X` read the default
  processor set's `load_average` / 1000, a ~5 s average of runnable
  threads.
- `--cpu-idle X` passes while the CPUs were more than X% idle over the
  last interval: Linux `/proc/stat` idle plus iowait, macOS
  `host_processor_info` ticks; the first sample is warming.
- `--cpu-pressure X` passes while `/proc/pressure/cpu` `some avg10` is
  under X; in a cgroup it reads the cgroup's `cpu.pressure`; macOS is
  unsupported.
- `--steal X` passes while the hypervisor stole under X% of the CPUs'
  ticks over the last interval (`/proc/stat` `steal`); macOS is
  unsupported.
- `parse_diskstats`, `DiskStat` and `Direction`: the Linux
  `/proc/diskstats` counters the IO gates read per device (reads,
  writes, sectors as 512 bytes, time, in flight, `io_ticks`).
- `parse_mountinfo`, `mount_of`, `linux_device` and `macos_device`: the
  IO gates measure the block device holding `--disk-path` (default the
  working directory): on Linux the device `stat` reports for the path,
  so a symlink into another mount counts its target's disk, else the
  deepest mount's device in `/proc/diskstats`; on macOS the volume's disk, which IOKit walks up to
  the physical disk under an APFS container. tmpfs, NFS, FUSE and devfs
  have no block device, and say to name one with `--io-device`.
- `IoSignal`: `--io-read X/s`, `--io-write X/s` and `--io X/s` pass
  while the device's read, write or summed bytes per second over the
  last interval are under X; `--io-busy X` while it was busy under X%
  of the interval (Linux `io_ticks`; macOS total request time, an upper
  bound capped at 100%); `--io-latency D` while the mean time per
  completed request is under D, passing when none completed. The first
  reading warms up.
- `--io-queue X` passes while fewer than X requests are in flight on the
  device (`/proc/diskstats`); macOS is unsupported.
- `--io-pressure X` and `--io-pressure-full X` pass while IO pressure
  stall (`/proc/pressure/io` `some` or `full` `avg10`) is under X; in a
  cgroup they read the cgroup's `io.pressure`; macOS is unsupported.
- `--io-device D` measures device D (`sda`, `nvme0n1`; `disk0` on
  macOS) instead of the one holding `--disk-path`; D must be a name, so
  it cannot reach outside `/sys` or `/dev` (`parse_io_device`).
- `Gate` and `Verdict`: a gate passes only when the system is strictly
  better than its threshold (`Gate::below`, `above`, `is`, `within`);
  equality fails, and a reading the threshold cannot be compared with is
  unsupported, never a pass. `per_cpu` and `unreserved` turn the raw
  reading into the `--load`/`--mem` per-CPU and `--needs-mem` values.
- `evaluate`: every gate in one round, joined by AND. No gates pass at once; a
  warming gate never passes; a gate with no source on this OS makes the
  whole `Overall::Unsupported` (exit 69), never a silent pass. Several
  gates on one signal read it once per round, and a failure carries
  which way its signal moves.
- `Custom` gates: `--when CMD` passes while `sh -c CMD` exits 0, with
  the command's output on cgmr's stderr; `--when-file P` passes while P
  exists. Both join the AND with the signal gates.
- `evaluate_once` and `sample_once`: `check` and `status` read a rate or
  interval percent twice, one `--interval` apart, so they never show a
  warming value; without one they read once and never wait.
- `write_json` and `StatusReport`: `status --json` and `check --json`
  write schema v1 (`schema_version`, `host`, `signals`, `gates`) with
  stable field names; numbers use a `.` whatever the locale.
- `Style`: output is coloured only on a terminal with `NO_COLOR` unset.
- `write_status`: the `status` table lists every signal with its source,
  value and trend; an unsupported signal shows its reason, never hidden.
- `Streams`: JSON and the `status` table go to stdout; verbose reasons,
  heartbeats, warnings and errors go to stderr.
- `write_verbose` and `heartbeat_line`: `--verbose` names each blocking
  gate with its value, threshold and trend, such as
  `--load: 3 (needs < 2, rising)`; the heartbeat puts them on one line.
- `Warnings`: every warning goes through one sink that `--quiet` silences.
- `poll`, `Pacing`, `Rig`, `Admit`, `Open` and `Outcome`: the wait loop
  evaluates the gates every `--interval` (default `2s`) on the monotonic
  clock, sleeping between rounds and never spinning, even on a zero
  interval.
- With `--backoff D`, each further failed round doubles the pause up to `D`;
  a passing round puts it back to `--interval`.
- With `--timeout D`, the wait takes one last round on the deadline and then
  gives up with exit 124, keeping that round's verdicts for the report.
- With `--settle D`, the gates must pass without a break for `D` before the
  command is let go; a failing or warming round starts the span again.
- A gate with no source on this machine ends the wait at once with exit 69,
  even mid-wait, instead of waiting out the timeout.
- `jitter_delay`: with `--jitter D`, a released command sleeps a pick in
  `[0, D)` before it starts, drawn from a seed (pid and start time) so
  waiters let go together spread out.
- With `--heartbeat D`, a waiting command writes one line to stderr every
  `D` saying how long it has waited and which gates block it; `--verbose`
  names the blocking gates after every failing round.
- `cpu_fit` and `mem_fit`: how many jobs the CPUs hold now,
  `floor(N * L - load)`, and the memory, `floor((available - reserved) / M)`,
  neither below 0.
- `decide`, `AdviseRequest`, `Advice`, `AdviceLimit`, `AdviceAction`: the
  `advise` verdict, jobs as the least of the cpu fit, the memory fit and
  `--jobs`, the resource that bounds it, and `run`, `reduce` or `wait`; any
  other requested gate failing means 0 jobs, bounded by that gate.
- `advise`, `advise_readings`, `advise_exit_code` and `AdviseUnsupported`:
  the `advise` verb reads the gates once (one extra sample only for a rate
  gate), then the load and available memory, and never waits; any advice
  exits 0, even `wait`, and a gate or reading with no source exits 69.
- `write_advice` and `write_advice_json`: `advise` prints one line such as
  `jobs=3 limit=mem action=reduce`, or with `--json` one line sharing
  `status`'s `schema_version` with `jobs`, `limit`, `action` and the
  `blocking` gates, each with its value and threshold.
- `ssh_argv` and `shell_quote`: `--host` runs `ssh -o BatchMode=yes H
  cgmr check --json` with each gate word its own argv word, single-quoted
  for the remote shell so nothing in it expands.
- `Host` and `parse_host`: `--host` takes any host form ssh does, but
  refuses an empty host, one starting with `-` or holding whitespace (exit
  64), and the host always follows `--` in ssh's argv.
- `check_remote`, `Transport`, `SystemSsh`, `SshReply` and `Round`: one
  `--host` round runs `cgmr check --json` on the remote host over the
  system `ssh` and reads its reply back.
- `parse_reply`, `RemoteReport`, `RemoteGate`, `RemoteSignal` and
  `RemoteVerdict`: the remote JSON read back, its gates joined as the local
  gate joins them; another `schema_version` exits 69 naming both.
- `--host` failures: ssh's own failure (exit 255 or killed) fails only
  that round and the wait goes on, with ssh's last stderr line as the
  reason; a remote host with no `cgmr` exits 69 and a remote usage error
  exits 64.
- `exec_command`: the released command replaces `cgmr`, so its exit code
  and signals pass through; a command not found exits 127, one that
  cannot run exits 126.
- `launch`: with no command after `--`, `cgmr` exits 0 once released.
- The command runs without `cgmr`'s own `CGMR_*` defaults; only
  `CGMR_HELD_SLOTS` passes on, naming the slot keys now held.
- `--background`: the command runs at nice 19 and the background disk IO
  class (macOS throttle, Linux idle); a refused call is a warning.
- `--or-background`: when `--timeout` runs out the command starts anyway,
  at background priority, instead of exiting 124.
- `Slots`: the slot steps as the wait's admission; the command inherits
  its slot and memory reservation locks, so they last until it exits.
- `govern_step` and `status_code`: the rules `--govern` will follow; a
  failing round stops the command's group, a pass resumes it, and its
  end passes on as its own exit code or 128 + n for signal n.
- `signal_group` and `GroupSignal`: stop, continue, interrupt, terminate
  or hang up a whole process group; group 0 (`cgmr`'s own) is refused.
- `block_forwarded`, `Blocked`, `ForwardSignal` and `Wake`: block SIGINT,
  SIGTERM, SIGHUP and SIGCHLD on this thread and take them one at a time
  with a timeout (`sigtimedwait` on Linux, kqueue on macOS), no handler;
  dropping it puts the old signal mask back.
- `govern`, `GovernRig`, `Judge`, `LocalGates` and `Group`: the `--govern`
  loop; each interval the gates are judged again, a failing round pauses
  the command's group and a passing one resumes it, caught signals pass
  on to the group, and the command's own exit code ends `cgmr`.
- `Blocked::unblock_in`: a command spawned while `cgmr` blocks signals
  starts with the mask from before, so it still dies of what it is sent.
- `launch_governed` and `Governing`: `--govern` spawns the command in its
  own process group, pauses it while the gates fail, forwards SIGINT,
  SIGTERM and SIGHUP to it, and exits with its code; a paused command
  keeps its slot and memory reservation.
- `inherit_in`: hands an open file's descriptor to the one child a
  `Command` starts, leaving it close-on-exec in `cgmr` itself.

### Fixed

- `--govern` no longer lets every child `cgmr` starts (a custom gate's
  shell, ssh for `--host`) inherit the slot and memory reservation; only
  the command does, so nothing else can keep the slot after it ends.
