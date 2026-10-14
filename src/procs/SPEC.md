# SPEC

## §G GOAL

`src/procs` — gates on how many processes match a name or user.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/procs|process counts by name & by user
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
sib|src/cgroup|cgroup v2 detection, limits, throttling, pressure override
sib|src/gate|gate model, strict compare, AND evaluation, unsupported verdict, custom gates
sib|src/wait|poll loop: interval, backoff, timeout, settle, jitter, heartbeat
sib|src/slot|per-user lock dir, slots, FIFO, rate, spacing, memory reservations, nested held marker
sib|src/launch|exec, no-cmd exit, govern, background priority, env cleanup
sib|src/remote|`--host` evaluation over ssh
sib|src/report|`status` human & JSON, verbose reasons, heartbeat text, colour
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, `src/sys` (sysinfo process list). own glob matcher (zero-dep).

## §I INTERFACES

- switches: `--max-procs NAME=N` · `--max-user-procs USER=N`; both repeatable; NAME & USER = glob (`*`, `?`).

## §V INVARIANTS

V1: `--max-procs P=N` ⇔ count(processes w/ name matching P) ≤ N.
V2: `--max-user-procs U=N` ⇔ count(processes owned by user matching U) ≤ N.
V3: cgmr itself & its ancestors ⊥ counted.
V4: malformed `NAME=N` → usage 64.

## §T TASKS

id|status|task|cites
T1|.|glob matcher|V1,V2
T2|.|`NAME=N` parser|V4
T3|.|`--max-procs`|V1,V3
T4|.|`--max-user-procs` (e.g. `nixbld*`)|V2,V3

## §B BUGS

id|date|cause|fix
