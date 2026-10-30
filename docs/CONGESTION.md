# Congestion methods

This document lists every congestion signal and every waiting strategy that
congestimancer (`cgmr`) knows about. It is the source for the CLI: each signal
maps to one or more gate switches, and each strategy maps to a control switch.

All methods listed here are in scope. None is deferred.

## Contents

- [Concepts](#concepts)
- [Verbs](#verbs)
- [Profiles](#profiles)
- [Advice](#advice)
- [Values and units](#values-and-units)
- [CPU](#cpu)
- [Memory](#memory)
- [Disk space](#disk-space)
- [Disk IO](#disk-io)
- [Network](#network)
- [System limits](#system-limits)
- [Thermal and power](#thermal-and-power)
- [User presence and time](#user-presence-and-time)
- [GPU](#gpu)
- [Processes](#processes)
- [Containers (cgroup v2)](#containers-cgroup-v2)
- [Remote hosts](#remote-hosts)
- [Custom gates](#custom-gates)
- [Waiting and release strategies](#waiting-and-release-strategies)
- [Running the command](#running-the-command)
- [Platform support matrix](#platform-support-matrix)

## Concepts

- **Signal**: one measured value, such as the 1-minute load average or the
  memory stall percentage.
- **Gate**: a switch that compares one signal with a threshold. A gate
  passes when the system is strictly better than the threshold.
- **All gates must pass** (logical AND) before the command starts.
- **Capacity vs pressure**: capacity signals say whether the work will fit
  (free memory, free disk). Pressure signals say whether the system is
  already struggling (stall time, swap rate, run queue). Use both.
- **Per-CPU vs absolute**: a gate without a prefix is per logical CPU. A gate
  with the `--all-` prefix is absolute, for the whole system. N, the logical
  CPU count, comes from `std::thread::available_parallelism`, which respects
  cgroup CPU limits and CPU affinity.
- **Unsupported gates fail**: if the current OS has no source for a signal,
  the gate fails with exit code 69. A gate never passes silently.
- **Every switch has an environment variable**: `--foo-bar` reads
  `CGMR_FOO_BAR` when the flag is not given. A CLI flag beats the environment
  variable, which beats a profile, which beats the built-in default.
  - Value switches use the same text as on the command line.
  - Boolean switches accept `1`, `true`, `yes` (on) and `0`, `false`,
    `no` (off).
  - Repeatable switches take comma-separated items, for example
    `CGMR_MAX_PROCS="rustc*=4,cargo=2"`.
  - An unknown `CGMR_*` variable gives a warning that names the closest
    known one.
- **Switches given twice are an error** (exit 64), except the repeatable
  `--max-procs` and `--max-user-procs`.
- **`--quiet`** hides warnings. **`--version`** prints the version.

## Verbs

| Verb | Behaviour |
| --- | --- |
| `run` (default) | Wait for all gates, then exec the command after `--`. Without a command, exit 0 once gates pass. |
| `check` | Evaluate all gates once. Exit 0 if all pass, 1 if not. Never loops; rate and percentage signals take one extra sample after `--interval`. |
| `advise` | Say how many parallel jobs fit now, what limits them, and whether to run, reduce or wait. `--json` for agents. |
| `status` | Print every supported signal with its current value (rate signals take one extra sample). `--json` gives a versioned, machine-readable form. |

Examples:

```sh
cgmr --load 2 -- make -j8          # run is the default verb
cgmr run --mem 2GB -- cargo test
cgmr --load 2 && make              # no command: wait, then exit 0
cgmr check --load 1 --disk 5GB     # one-shot, for scripts and agents
cgmr status --json                 # readings for agents
cgmr advise --jobs 8 --needs-mem 2GB --json   # how many of 8 jobs fit
```

## Profiles

A profile is a named set of switches, so a gate combination used in many
places gets one name.

```ini
# .cgmr (project) or ~/.config/cgmr/profiles (user)
[heavy]
--load 1
--mem 2GB
--slots 2
--slot-key build

[quiet-hours]
--between 22:00-06:00
--idle 10m
```

```sh
cgmr --profile heavy -- cargo test
cgmr profiles                      # list names, files and switches
```

- Lookup: `--profile-file P` (or `CGMR_PROFILE_FILE`) alone if given;
  otherwise the nearest `.cgmr` walking up from the working directory,
  then the user file. A project profile beats a user profile of the same
  name.
- Precedence: CLI flag, then environment variable, then profile, then the
  built-in default.
- `--when` is allowed only in the user file. A cloned repository's
  `.cgmr` must not be able to run commands.
- Errors name the file and line.

## Advice

`cgmr advise` answers "how much work fits right now" without waiting.

- CPU fit: `floor(N * L - load)`, where L is the `--load` threshold
  (default 1.0). Never below 0.
- Memory fit: `floor((available - reserved) / M)`, where M is
  `--needs-mem` (default 1GiB) and reserved is memory held by running cgmr
  jobs.
- Jobs: the lowest of the CPU fit, the memory fit and `--jobs J` if given.
  Any other requested gate that fails makes it 0.
- Action: `run` when at least one job fits and J (if given) fits, `reduce`
  when fewer than J fit, `wait` when none fit.
- Output names the limiting resource, for example
  `jobs=3 limit=mem action=reduce`, or JSON with the same schema version as
  `status`.

## Values and units

- Sizes: `B`, `KB`, `MB`, `GB`, `TB` are powers of 1000. `KiB`, `MiB`,
  `GiB`, `TiB` are powers of 1024. Each unit means exactly what it says.
- Rates: a size followed by `/s`, for example `100MB/s`.
- Percentages: a number, optionally followed by `%`, for example `10` or
  `10%`.
- Durations: a number followed by `ms`, `s`, `m`, `h` or `d`, and
  combinations such as `1h30m`. Parsed by cgmr itself.
- Temperatures: a number followed by `C`, for example `80C`.
- Decimal separator is always a dot, regardless of locale.

## CPU

### Load average

The number of tasks running or waiting to run, averaged over 1, 5 or 15
minutes. On Linux it also counts tasks blocked on IO (D state). It lags: a job
that just started shows up fully only after about a minute.

| Switch | Passes when |
| --- | --- |
| `--load X` | load / N < X |
| `--all-load X` | load < X |
| `--load-window 1m\|5m\|15m` | selects the average (default `1m`) |

Sources: `sysinfo` on Linux and macOS.

### Run queue

The number of tasks ready to run right now. No lag, so it reacts to jobs that
just started. Reduces the thundering herd.

| Switch | Passes when |
| --- | --- |
| `--run-queue X` | running tasks / N < X |
| `--all-run-queue X` | running tasks < X |

Sources: `procs_running` in `/proc/stat` (Linux); summed per-CPU run state
from `host_processor_info` or the closest native counter (macOS).

### CPU idle

Percentage of CPU time spent idle, measured over the poll interval.

| Switch | Passes when |
| --- | --- |
| `--cpu-idle X` | idle percent > X |

Sources: `/proc/stat` deltas (Linux); `host_processor_info` deltas (macOS).
Needs two samples, so the first result arrives after one interval.

### CPU pressure

Percentage of time at least one task waited for a CPU (PSI `some avg10`).

| Switch | Passes when |
| --- | --- |
| `--cpu-pressure X` | `some avg10` < X |

Sources: `/proc/pressure/cpu` or the cgroup's `cpu.pressure` (Linux).
macOS: unsupported.

### CPU steal

Percentage of time the hypervisor gave this VM's CPU to someone else. High
steal means the host is congested even when local load looks low.

| Switch | Passes when |
| --- | --- |
| `--steal X` | steal percent < X |

Sources: `steal` column in `/proc/stat` (Linux). macOS: unsupported.

## Memory

### Available memory

Memory that can be given to new work without swapping. Not "free" memory:
Linux fills spare RAM with page cache, and macOS compresses memory and keeps
many inactive pages, so "free" is misleading on both.

| Switch | Passes when |
| --- | --- |
| `--mem X` | available / N > X |
| `--all-mem X` | available > X |

Sources: `sysinfo` (Linux: `MemAvailable`; macOS: native estimate).

### Memory reservation

Lets a command declare how much memory it will need. cgmr subtracts memory
already reserved by other cgmr jobs that are still running, so several
waiting jobs do not all count the same free memory.

| Switch | Passes when |
| --- | --- |
| `--needs-mem X` | available - reserved by running cgmr jobs > X |

The reservation is recorded in the per-user slot directory and released when
the command exits.

### Memory pressure (stall time)

Percentage of time tasks stalled waiting for memory (PSI). `some` means at
least one task stalled; `full` means all non-idle tasks stalled at once.

| Switch | Passes when |
| --- | --- |
| `--mem-pressure X` | `some avg10` < X |
| `--mem-pressure-full X` | `full avg10` < X |

Sources: `/proc/pressure/memory` or the cgroup's `memory.pressure` (Linux).
macOS: unsupported; use `--mem-level`.

### Memory pressure level

The kernel's own judgement of memory pressure.

| Switch | Passes when |
| --- | --- |
| `--mem-level normal` | level is normal |
| `--mem-level warning` | level is normal or warning |

Sources: `sysctl kern.memorystatus_vm_pressure_level` (1 normal, 2 warning,
4 critical) on macOS. Linux: unsupported; use `--mem-pressure`.

`sysctl kern.memorystatus_level` (percentage of memory available, 0 to 100)
is shown by `status` on macOS.

### Swap rate

Pages moved to and from swap per second. Shows thrashing. Swap *used* says
little, because old pages can sit in swap without harm.

| Switch | Passes when |
| --- | --- |
| `--swap-rate X/s` | swap in + swap out < X |

Sources: `pswpin`/`pswpout` in `/proc/vmstat` (Linux); swapins/swapouts from
`host_statistics64` / `vm_stat` counters (macOS).

### Dirty page backlog

Data written to the page cache but not yet written to disk. A large backlog
means writes are about to stall.

| Switch | Passes when |
| --- | --- |
| `--dirty X` | `Dirty` + `Writeback` < X |

Sources: `/proc/meminfo` (Linux). macOS: unsupported.

## Disk space

| Switch | Passes when |
| --- | --- |
| `--disk X` | free space on the path > X |
| `--inodes X` | free inodes on the path > X |
| `--disk-path P` | selects the path (default: working directory) |

Sources: `statvfs` on the path (`f_bavail`, `f_favail`) on Linux and macOS.

## Disk IO

All IO gates measure the device that holds `--disk-path` (default: working
directory) unless `--io-device` names one. On macOS with APFS, several
volumes share one physical disk; cgmr measures at the physical disk. Mounts
without a block device (tmpfs, NFS, FUSE) make IO gates fail with exit 69 and
a hint to use `--io-device`.

### Throughput

| Switch | Passes when |
| --- | --- |
| `--io-read X/s` | read rate < X |
| `--io-write X/s` | write rate < X |
| `--io X/s` | read + write rate < X |
| `--io-device D` | selects the device |

Rates are averaged over the poll interval. Sources: `sysinfo` per-disk IO
stats if they work on both OSes (verified by a spike test first); otherwise
`/proc/diskstats` (Linux) and IOKit block storage statistics (macOS).

### IO busy time

Percentage of time the device had at least one request in flight.

| Switch | Passes when |
| --- | --- |
| `--io-busy X` | busy percent < X |

Sources: `io_ticks` in `/proc/diskstats` (Linux); IOKit total read/write time
deltas (macOS).

### IO latency and queue depth

Waiting time per request reflects congestion better than throughput, because
SSD and HDD limits differ widely.

| Switch | Passes when |
| --- | --- |
| `--io-latency X` | average time per request < X (duration, e.g. `20ms`) |
| `--io-queue X` | requests in flight < X |

Sources: `/proc/diskstats` (Linux); IOKit statistics (macOS, latency only;
`--io-queue` unsupported).

### IO pressure

Percentage of time tasks stalled waiting for IO (PSI).

| Switch | Passes when |
| --- | --- |
| `--io-pressure X` | `some avg10` < X |
| `--io-pressure-full X` | `full avg10` < X |

Sources: `/proc/pressure/io` or the cgroup's `io.pressure` (Linux).
macOS: unsupported; use `--io-busy` or `--io-latency`.

## Network

| Switch | Passes when |
| --- | --- |
| `--net-rx X/s` | receive rate < X |
| `--net-tx X/s` | transmit rate < X |
| `--net X/s` | receive + transmit rate < X |
| `--net-iface I` | selects the interface (default: all non-loopback) |
| `--rtt X --rtt-host H` | round-trip time to H < X (duration) |

Sources: `/proc/net/dev` (Linux); `getifaddrs` with `if_data` (macOS). RTT
uses a TCP connect probe (no raw sockets, no root), to port 443 unless
`--rtt-host` gives `host:port`.

## System limits

| Switch | Passes when |
| --- | --- |
| `--fds X` | open files / system maximum < X percent |
| `--pids X` | processes / system maximum < X percent |

Sources: `/proc/sys/fs/file-nr` and `/proc/sys/kernel/pid_max` (Linux);
`kern.num_files` / `kern.maxfiles`, and the count of live processes /
`kern.maxproc` (macOS). `kern.num_tasks` is not used: it is a fixed cap,
not a count of what runs.

## Thermal and power

| Switch | Passes when |
| --- | --- |
| `--cool` | the CPU is not thermally throttled |
| `--temp X` | hottest thermal zone < X (e.g. `80C`) |
| `--on-ac` | running on mains power |
| `--battery X` | battery charge > X percent, or no battery |
| `--no-low-power` | Low Power Mode / power saver is off |

Sources:

- `--cool`: `thermal_throttle` counters and `/sys/class/thermal` (Linux);
  thermal state from `NSProcessInfo.thermalState` or `pmset -g therm`
  (macOS, passes when nominal).
- `--temp`: `/sys/class/thermal/thermal_zone*/temp` (Linux). macOS:
  unsupported (no public temperature API).
- Power: `/sys/class/power_supply` (Linux); IOKit power sources (macOS).
- Low power: `power-profiles-daemon` state (Linux, unsupported if absent);
  `pmset -g` `lowpowermode` (macOS).

## User presence and time

| Switch | Passes when |
| --- | --- |
| `--idle D` | no keyboard or mouse input for at least D |
| `--between HH:MM-HH:MM` | local time is in the window (may wrap midnight) |

Sources for `--idle`: `HIDIdleTime` from `IOHIDSystem` (macOS); the
session's idle hint from logind, or X11/Wayland idle where available (Linux,
unsupported on headless hosts).

## GPU

| Switch | Passes when |
| --- | --- |
| `--gpu X` | GPU utilisation < X percent |
| `--gpu-mem X` | free GPU memory > X |
| `--gpu-device I` | selects the GPU (default: all, worst wins) |

Sources: NVML for NVIDIA and `gpu_busy_percent` / `mem_info_vram_*` in sysfs
for AMD (Linux); IOKit `PerformanceStatistics` (macOS). Unsupported where no
source exists.

## Processes

| Switch | Passes when |
| --- | --- |
| `--max-procs NAME=N` | at most N processes named NAME are running |
| `--max-user-procs USER=N` | at most N processes are owned by USER |

NAME and USER accept glob patterns, for example `--max-procs 'rustc*'=4` or
`--max-user-procs 'nixbld*'=8` (active Nix builds). The switch can repeat.
Sources: process list from `sysinfo`.

## Containers (cgroup v2)

Inside a cgroup v2, host-wide `/proc` values describe the host, not the
container. cgmr detects its cgroup and uses the cgroup's own files:

| Signal | cgroup file |
| --- | --- |
| CPU limit (N) | `cpu.max` (via `available_parallelism`) |
| CPU throttling | `cpu.stat` (`nr_throttled`, `throttled_usec`) |
| CPU pressure | `cpu.pressure` |
| Memory limit and use | `memory.max`, `memory.current` |
| Memory pressure | `memory.pressure` |
| IO pressure | `io.pressure` |

| Switch | Effect |
| --- | --- |
| `--cgroup auto` (default) | use the cgroup if one limits this process |
| `--cgroup host` | always use host values |
| `--cgroup PATH` | use the given cgroup |
| `--cpu-throttle X` | passes when the throttled share of the last interval < X percent |

Inside a cgroup, `--load` is unreliable: the host load divided by the
container's CPU count is wrong. cgmr warns and suggests `--cpu-pressure`.
`status` shows both cgroup and host values. cgroup v1 is unsupported.

## Remote hosts

| Switch | Effect |
| --- | --- |
| `--host H` | evaluate all gates on host H instead of locally |

cgmr runs `ssh H cgmr check --json <gates>` and uses the result. H must have
cgmr installed. Slots, reservations and the command itself stay local. Useful
for gating work sent to Nix remote builders.

## Custom gates

| Switch | Passes when |
| --- | --- |
| `--when CMD` | `sh -c CMD` exits 0 |
| `--when-file PATH` | PATH exists |

`--when` runs arbitrary commands, so it has no environment variable default:
a variable in the environment must not be able to inject a command.

## Waiting and release strategies

| Switch | Effect |
| --- | --- |
| `--interval D` | time between gate checks (default `2s`) |
| `--backoff D` | while gates keep failing, double the interval up to D |
| `--timeout D` | give up after D; exit 124 without running the command |
| `--settle D` | all gates must pass continuously for D before the start |
| `--jitter D` | random delay up to D before the start |
| `--heartbeat D` | while waiting, print a status line to stderr every D (keeps CI from killing a silent job) |
| `--slots N --slot-key K` | at most N commands with key K run at once |
| `--fifo` | with `--slots`: waiters start in arrival order (fairness) |
| `--rate N/D --slot-key K` | at most N starts per D for key K (token bucket) |
| `--spacing D --slot-key K` | at least D between two starts for key K, so the next waiter sees the new load |
| `--alone-passes` | with `--slots`: when no running job holds key K, the job starts without the gates (it still takes the slot and reserves `--needs-mem`); later jobs wait on the gates |
| `--or-background` | when `--timeout` is reached, run anyway at background priority instead of exiting 124 |
| `--verbose` | report which gate blocks, with current value, threshold and trend |

Notes:

- Release order: gates pass, then `--settle` holds, then the FIFO ticket,
  then a slot, then `--rate` and `--spacing`, then the memory reservation,
  then `--jitter`, then the launch. A failure at any step goes back to the
  gates.
- Thresholds are never negative. Load values may have decimals; counts are
  whole numbers.
- Cost budget: one check round with `--load` uses at most 5 ms of CPU, the
  process stays under 10 MB of memory, and `cgmr check --load 1` starts in
  at most 20 ms.
- The first check of a rate or percentage signal needs two samples. cgmr
  waits one interval and never reports a fake zero.
- cgmr must stay cheap: it sleeps between checks and refreshes only the
  signals the requested gates need. 100 waiting instances must not add
  measurable load.
- Slot locks use `flock` on files in a per-user directory with mode 0700
  (`$XDG_RUNTIME_DIR`, else `$TMPDIR`, else a per-user directory under
  `/tmp`). The lock descriptor survives `exec`, so the command holds the
  slot until it exits.
- Nested cgmr: a command started by cgmr that runs cgmr again with the same
  slot key must not deadlock on its own slot. cgmr marks held slots in the
  environment of the command it starts.

## Running the command

| Switch | Effect |
| --- | --- |
| (default) | `exec` the command; exit code and signals pass through |
| `--govern` | keep watching after the start: stop the command (SIGSTOP to its process group) when gates fail, continue it (SIGCONT) when they pass again |
| `--background` | lower the command's CPU and IO priority: `setpriority` plus `ioprio_set` idle class (Linux); `setpriority` plus background QoS / `setiopolicy_np` (macOS) |
| `--export-jobs VAR` | repeatable: set VAR in the command's environment to its share of the CPUs, `max(1, CPUs / N)` with N the jobs holding the slot key (1 without `--slots`), counted when the slot is granted; for example `CARGO_BUILD_JOBS`; `CGMR_HELD_SLOTS` is never overwritten |

With `--govern`, a stopped command keeps its slot and memory reservation,
`--timeout` applies only before the start, and `--host` keeps working.

`--govern` needs cgmr to stay alive as the parent, so it spawns the command
instead of using `exec`. cgmr then forwards SIGINT, SIGTERM and SIGHUP to the
command and exits with the command's exit code.

cgmr removes its own `CGMR_*` variables from the command's environment
(except the held-slot marker), so nested calls do not inherit gates by
accident.

## Platform support matrix

`yes` = native source; `no` = gate fails with exit 69.

| Gate | Linux | macOS |
| --- | --- | --- |
| `--load`, `--all-load` | yes | yes |
| `--run-queue`, `--all-run-queue` | yes | yes |
| `--cpu-idle` | yes | yes |
| `--cpu-pressure` | yes | no |
| `--steal` | yes | no |
| `--mem`, `--all-mem`, `--needs-mem` | yes | yes |
| `--mem-pressure`, `--mem-pressure-full` | yes | no |
| `--mem-level` | no | yes |
| `--swap-rate` | yes | yes |
| `--dirty` | yes | no |
| `--disk`, `--inodes` | yes | yes |
| `--io-read`, `--io-write`, `--io` | yes | yes |
| `--io-busy`, `--io-latency` | yes | yes |
| `--io-queue` | yes | no |
| `--io-pressure`, `--io-pressure-full` | yes | no |
| `--net-rx`, `--net-tx`, `--net`, `--rtt` | yes | yes |
| `--fds`, `--pids` | yes | yes |
| `--cool` | yes | yes |
| `--temp` | yes | no |
| `--on-ac`, `--battery`, `--no-low-power` | yes | yes |
| `--idle` | desktop only | yes |
| `--between` | yes | yes |
| `--gpu`, `--gpu-mem` | NVIDIA, AMD | yes |
| `--max-procs`, `--max-user-procs` | yes | yes |
| `--cgroup`, `--cpu-throttle` | yes | no |
| `--host` | yes | yes |
| `--when`, `--when-file` | yes | yes |
