# SPEC

## §G GOAL

`src/cgroup` — container awareness: cgroup v2 limits & pressure replace host-wide values.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/cgroup|cgroup v2 detection, limits, throttling, pressure override
sib|src/units|parse & render sizes, rates, percents, durations, temperatures, clock windows
sib|src/signal|`Signal` & `Source` traits, `Reading`, sampler w/ two-sample deltas, monotonic clock, fixture root
sib|src/sys|OS facade: ∀ syscalls, FFI, `sysinfo`, `rustix`, `libc`; only node allowed `unsafe`
sib|src/cpu|load, run queue, CPU idle, CPU pressure, steal
sib|src/mem|available memory, memory pressure & level, swap rate, dirty pages
sib|src/disk|free space & inodes on a path
sib|src/io|IO throughput, busy, latency, queue, pressure; path → device resolution
sib|src/net|interface rates, RTT probe
sib|src/limits|open files & processes vs system maximum
sib|src/power|thermal throttling, temperature, AC, battery, low power
sib|src/presence|user idle time, clock window
sib|src/gpu|GPU utilisation & free memory
sib|src/procs|process counts by name & by user
sib|src/gate|gate model, strict compare, AND evaluation, unsupported verdict, custom gates
sib|src/wait|poll loop: interval, backoff, timeout, settle, jitter, heartbeat
sib|src/slot|per-user lock dir, slots, FIFO, rate, spacing, memory reservations, nested held marker
sib|src/launch|exec, no-cmd exit, govern, background priority, env cleanup
sib|src/remote|`--host` evaluation over ssh
sib|src/report|`status` human & JSON, verbose reasons, heartbeat text, colour
sib|src/profile|named switch sets from project & user files, lookup & trust rule
sib|src/advise|`advise` verb: how many jobs fit now, what limits them, what to do
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, `src/sys`. exposes `Scope` (cgroup|host) consumed by domain nodes. Linux only; macOS ∴ `--cgroup` & `--cpu-throttle` `Unsupported`.

## §I INTERFACES

- switches: `--cgroup auto|host|PATH` · `--cpu-throttle X`.
- files: `/proc/self/cgroup`, `<cg>/cpu.max`, `cpu.stat`, `cpu.pressure`, `memory.max`, `memory.current`, `memory.pressure`, `io.pressure`.

## §V INVARIANTS

V1: `auto` (default) → use cgroup iff it limits this process (`cpu.max` ≠ `max` | `memory.max` ≠ `max`) | runs in a container; else host.
V2: in cgroup: pressure gates read cgroup files; available memory = `memory.max` − `memory.current` (min w/ host available).
V3: in cgroup: `--load` warns (stderr) & suggests `--cpu-pressure` ∵ host load / container CPUs wrong.
V4: `--cpu-throttle X` ⇔ throttled share of last interval (`cpu.stat` `throttled_usec` delta / interval) < X.
V5: cgroup v1 → `Unsupported`.
V6: `status` shows cgroup & host values side by side.

## §T TASKS

id|status|task|cites
T1|x|detect cgroup v2 path from `/proc/self/cgroup`|V1,V5
T2|x|`cpu.max` `memory.max` `memory.current` parsers|V1,V2
T3|x|`cpu.stat` parser + `--cpu-throttle`|V4
T4|.|pressure source override ∀ PSI gates|V2
T5|.|`--load` warning in cgroup|V3
T6|x|`--cgroup` modes|V1
T7|.|status dual values|V6

## §B BUGS

id|date|cause|fix
