# SPEC

## §G GOAL

`src/disk` — free space & inodes on a path.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/disk|free space & inodes on a path
sib|src/units|parse & render sizes, rates, percents, durations, temperatures, clock windows
sib|src/signal|`Signal` & `Source` traits, `Reading`, sampler w/ two-sample deltas, monotonic clock, fixture root
sib|src/sys|OS facade: ∀ syscalls, FFI, `sysinfo`, `rustix`, `libc`; only node allowed `unsafe`
sib|src/cpu|load, run queue, CPU idle, CPU pressure, steal
sib|src/mem|available memory, memory pressure & level, swap rate, dirty pages
sib|src/io|IO throughput, busy, latency, queue, pressure; path → device resolution
sib|src/net|interface rates, RTT probe
sib|src/limits|open files & processes vs system maximum
sib|src/power|thermal throttling, temperature, AC, battery, low power
sib|src/presence|user idle time, clock window
sib|src/gpu|GPU utilisation & free memory
sib|src/procs|process counts by name & by user
sib|src/cgroup|cgroup v2 detection, limits, throttling, pressure override
sib|src/gate|gate model, strict compare, AND evaluation, unsupported verdict, custom gates
sib|src/wait|poll loop: interval, backoff, timeout, settle, jitter, heartbeat
sib|src/slot|per-user lock dir, slots, FIFO, rate, spacing, memory reservations, nested held marker
sib|src/launch|exec, no-cmd exit, govern, background priority, env cleanup
sib|src/remote|`--host` evaluation over ssh
sib|src/report|`status` human & JSON, verbose reasons, heartbeat text, colour
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, `src/sys` (`statvfs`).

## §I INTERFACES

- switches: `--disk X` · `--inodes X` · `--disk-path P`.

## §V INVARIANTS

V1: `--disk X` ⇔ `f_bavail × f_frsize` > X (space available to unprivileged user).
V2: `--inodes X` ⇔ `f_favail` > X.
V3: path default = cwd; `--disk-path P` overrides; P missing → usage error 64.
V4: both OS via `statvfs`; ⊥ mount-table matching.

## §T TASKS

id|status|task|cites
T1|.|`--disk` via statvfs|V1,V4
T2|.|`--inodes`|V2
T3|.|`--disk-path` default & validation|V3

## §B BUGS

id|date|cause|fix
