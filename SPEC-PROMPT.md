congestimancer

Tool for load awareness in the system that allow to calculate machine congestion from:
- unix load
- memory pressure
- io load to advise
- potentially more

To:
- pause or perform other operations
- create mechanical workflows based on resource availability
- advise agents on actions.

Write in Rust.
Use atomic git commits per topic for human readability.

We should have nix flake dev shell loaded with direnv.
When we have basic shell - ask me to reload and continue session.

Guardrails is a first thing and we should:
- use pklith and nix-hk
- use sherd for SPEC-DAG federation and to do sherd wave builds in parallel subagents
- use microlith for spec operations
- use itok for context limiting
- use characterminator to keep characters at bays
Inspire all scrutinized checks from other our Rust repos which apply.
We can dogfood this project in guardrails as well.
Use 1-to-1 unit coverage with floor and ceiling aspiring towards 100%.
Work in TDD: RED, GREEN, REFACTOR cycles.
Guardrails should be installed when entering dev shell.
When we have guardrails ready - ask me to reload and continue session.

# cgmr CLI with swiches:

cgmr --load 2 -- echo example command # wait for load to be below two per logical CPU thread
cgmr --all-load 10 -- echo example command # wait for total system load to be below 10

cgmr --mem 2GB -- echo example command # wait for have at least 2GB free memory per processing thread
cgmr --all-mem 10GB -- echo example command # wait for system to have at least 10GB free memory

cgmr --disk 5GB -- echo example command # wait to have 5GB disk space in the working directory

cgmr --io-read 1MB/s -- echo example command # wait for IO reads to be below 1MB/s
cgmr --io-write 1GB/s -- echo example command # wait for IO writes to be below 1GB/s
cgmr --io 100MB/s -- echo example command # wait for total IO reads+writes to be below 100MB/s

Switch can be mixed together as and:
cgmr --load 1 --mem 1GB --disk 2GB --io-read 100MB/s --io-write 100MB/s -- example command

# Naming

- Project / crate name: `congestimancer` ("one who divines congestion"; hides "estim(ate)" pun).
- Binary name: `cgmr`.
- Availability checked 2026-10-14: `congestimancer` and `cgmr` free on crates.io, npm, PyPI,
  RubyGems, Homebrew core; no GitHub user/org `congestimancer`; no local binary/alias clash.
- Rejected binary alternatives:
  - `cong`: possible, but in US English it is a war-era slur for the Viet Cong.
  - `la`: taken on most registries and clashes with common `alias la='ls -lAh'`.
  - `cgm`: taken on PyPI; also means "continuous glucose monitor".
  - `gest`: taken on crates.io, npm and PyPI.
- Claim the crate name early with a 0.0.0 placeholder publish (needs explicit approval; permanent).

# Gate semantics

- Every switch is a gate; all given gates are combined with AND.
- Command follows `--`; cgmr execs it (passing through exit code) once all gates pass.
- Size units: B, KB, MB, GB, TB (decide and document 1000 vs 1024; accept KiB/MiB/GiB too).
- Rate units: same sizes suffixed with `/s`.
- "Per processing thread" means per logical CPU (`--load`, `--mem`); `--all-*` variants are
  absolute, system-wide values.
- `--disk` checks the filesystem holding the working directory (allow `--disk-path` override).
- `--io-*` measure the device holding the working directory by default (allow device override);
  on Linux consider all devices option.

# Control switches

- `--interval 2s`: how often gates are polled.
- `--timeout 1h`: give up after duration; exit with a distinct, documented exit code without
  running the command.
- `--settle 10s`: every gate must hold continuously for the whole period before the command
  starts (hysteresis; avoids flapping).
- `--jitter 5s`: random delay before starting, so many waiting cgmr processes do not start at
  the same instant.
- `--verbose`: report which gate blocks and current vs threshold values.
- `--check` (or similar): evaluate gates once, print status, exit 0/1 without running anything.
  Useful for scripts and for advising agents.
- Machine-readable output (e.g. `--json`) of current congestion readings for agents.

# Thundering herd

Problem: 20 waiting cgmr processes see load drop at the same time, all start, load spikes again.
The load average also lags by ~1 minute, so it does not yet show jobs that just started.
Mitigations (in priority order):
1. `--slots N --slot-key name`: at most N commands with the same key run at once (lock files /
   flock in `$XDG_RUNTIME_DIR` or a temp dir). Cheapest and most useful fix.
2. `--settle` and `--jitter` (above).
3. Optionally count jobs cgmr itself released recently as extra load until the load average
   catches up.

# Memory pressure

Gate on pressure, not only free memory. Free memory is misleading:
- Linux fills spare RAM with page cache ("free" looks low even when reclaimable);
  `MemAvailable` is better but still an estimate.
- macOS compresses memory and keeps many inactive pages, so "free" is almost always near 0.
Pressure measures stalls (real congestion); free memory measures capacity. Keep both:
- `--mem` / `--all-mem`: will the command fit.
- `--mem-pressure`: is the system already struggling.

Sources:
- Linux: PSI `/proc/pressure/memory` (`some avg10`, `full avg10`; `some` = % time at least one
  task stalled, `full` = % time all tasks stalled). Example: `--mem-pressure 10` waits for
  `some avg10 < 10%`. Inside a container use the cgroup's `memory.pressure` file.
- macOS: `sysctl kern.memorystatus_vm_pressure_level` (1 normal, 2 warning, 4 critical);
  `sysctl kern.memorystatus_level` (% memory available, 0–100); swapout rate from `vm_stat`.
- Both: swap *rate* (pages swapped per second) is a good thrashing signal; swap *used* is not.

# Other congestion factors

Ordered by usefulness for a gate:

| Factor | Why | Linux / macOS source |
|---|---|---|
| CPU pressure | Better than load average (load counts D-state IO waiters and lags) | `/proc/pressure/cpu` / CPU idle % via `host_processor_info` |
| IO pressure | Measures stalls; MB/s limits differ wildly between SSD and HDD | `/proc/pressure/io` / derive from IOKit disk stats |
| IO latency / queue depth | Waiting time reflects congestion better than throughput | `/proc/diskstats` / IOKit |
| Instance slots | Max N concurrent commands per key | lock files / flock |
| Process-count match | Wait while more than N `cargo`/`rustc` processes run | process list (`sysinfo` crate) |
| Thermal throttling | Hot laptop throttles; more work only adds heat | `/sys/class/thermal` / `NSProcessInfo.thermalState`, `pmset -g therm` |
| Power | On AC, battery above X%, Low Power Mode off | `/sys/class/power_supply` / `pmset -g batt`, IOKit |
| User idle | Run heavy jobs only when user is away | X11/Wayland idle / `ioreg -c IOHIDSystem` `HIDIdleTime` |
| Time window | `--between 22:00-06:00`, off-peak electricity | clock |
| Network bandwidth | Rx/tx below cap | `/proc/net/dev` / `getifaddrs` + `if_data` |
| Network latency | RTT to a host below X ms (bufferbloat) | ICMP/TCP probe |
| Disk inodes | Free space but no free inodes | `statvfs` `f_favail` |
| FD / PID limits | Near `ulimit -n` or max processes | `/proc/sys/fs/file-nr` / `kern.num_files` vs `kern.maxfiles` |
| GPU | Utilisation or free VRAM for ML/render jobs | NVML / IOKit `PerformanceStatistics` |
| cgroup limits | In Docker/Kubernetes host `/proc` lies; use `memory.max`, `memory.current`, `cpu.max`, `cpu.stat` | `/sys/fs/cgroup` (Linux only) |
| Custom gate | `--when 'curl -fs host/health'`, `--when-file /tmp/go`; extensible without code | exit code of command |

# Priority

1. v1 core: `--load`/`--all-load` (plus `--cpu-pressure` on Linux), `--mem`/`--all-mem`,
   `--mem-pressure`, `--disk`.
2. `--io-pressure` on Linux; `--io-read`/`--io-write`/`--io` on both platforms.
3. `--slots N --slot-key name`.
4. `--settle`, `--interval`, `--timeout`, `--jitter`, `--verbose`, `--check`, `--when`.
5. Later: thermal, power, idle, time window, network, GPU, inodes, FD/PID limits, cgroups.

# Platform policy

- Targets: Linux and macOS (darwin).
- Define each gate per platform and map it to the native signal (e.g. PSI on Linux, pressure
  level / derived stall metrics on macOS).
- If a gate has no native signal on the current OS, fail with a clear "unsupported on this OS"
  error. Never pass a gate silently.

# Prior art

- GNU `parallel` `--load`, `--memfree`; `sem` (slots).
- `nice`, `batch` (at).
- Differentiator: standalone gate with disk, IO and pressure checks, plus agent-friendly output.

# Decisions (2026-10-14)

1. Units: accept both decimal (`KB`, `MB`, `GB`, `TB` = powers of 1000) and binary (`KiB`,
   `MiB`, `GiB`, `TiB` = powers of 1024); each means exactly what it says. Rates add `/s`.
2. `--load` uses the 1-minute load average by default; `--load-window 1m|5m|15m` overrides.
3. IO rates (`--io-read`, `--io-write`, `--io`) are averaged over the poll interval.
4. cgmr `exec`s the command once gates pass (no parent process left): exit code and signals
   pass through naturally. `--slots` locks use `flock` on an open file descriptor without
   close-on-exec, so the lock is held by the command and released when it exits.
5. Exit codes: the command's own code once it runs; reserved codes otherwise:
   124 = `--timeout` reached (like `timeout(1)`), 64 = bad arguments (EX_USAGE),
   69 = gate unsupported on this OS (EX_UNAVAILABLE).
6. Workspace split: `congestimancer` library crate (gates, metric readers, parsing) plus the
   `cgmr` binary as a thin CLI over it, so agents and other tools can reuse the library.
7. Testability invariant: every metric source sits behind a trait; tests use fakes (fake
   `/proc` / `sysctl` inputs), never the live machine. Required for 1-to-1 unit coverage.
8. Metric sources: one trait method per signal, exactly one source per signal per platform.
   Never mix sources within one signal. Keep `sysinfo` adapters thin; custom readers are pure
   parsers (text in, numbers out) unit-tested 1-to-1 with fixture files.

   | Signal | Source |
   |---|---|
   | Load average | `sysinfo` (or `libc::getloadavg`) |
   | Logical CPU count | `std::thread::available_parallelism` (respects cgroup CPU limits and affinity) |
   | Free / available memory | `sysinfo` |
   | Disk free for a path | `statvfs` on the path (`rustix` or `nix`); also gives free inodes |
   | Disk IO rates | `sysinfo` per-disk IO stats if verified on both OSes; otherwise `/proc/diskstats` (Linux) and IOKit (macOS) |
   | PSI cpu / memory / io | custom reader for `/proc/pressure/*` (Linux) |
   | macOS memory pressure | custom `sysctl` reader for `kern.memorystatus_*` |
   | cgroup limits | custom reader for `/sys/fs/cgroup` (Linux) |

   First task before building IO gates: spike test proving `sysinfo` disk IO stats work on
   Linux and macOS; record the outcome in the spec.
9. License: MIT.
10. MSRV and toolchain: same as the other Rust projects (`rust-version = "1.95"`).
11. Platforms: the same 4 systems as the other Rust projects' flakes: `aarch64-darwin`,
    `x86_64-darwin`, `x86_64-linux`, `aarch64-linux`. Windows and BSD are non-goals.
12. Agent interface for v1: `--check` and `--json` only. MCP server and daemon mode are
    non-goals for v1.
13. Dogfood always: the project's own guardrails and heavy hooks (tests, coverage, builds)
    run through `cgmr` as soon as it can gate them.
14. Claim the crate name on crates.io with the first real pre-release, `0.1.0-rc.1`
    (crates.io requires SemVer; `0.1.0.rc1` is not valid). Publishing needs explicit approval.
