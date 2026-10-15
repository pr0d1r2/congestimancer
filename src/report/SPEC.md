# SPEC

## §G GOAL

`src/report` — render readings & verdicts for humans & agents.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/report|`status` human & JSON, verbose reasons, heartbeat text, colour
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
sib|src/cgroup|cgroup v2 detection, limits, throttling, pressure override
sib|src/gate|gate model, strict compare, AND evaluation, unsupported verdict, custom gates
sib|src/wait|poll loop: interval, backoff, timeout, settle, jitter, heartbeat
sib|src/slot|per-user lock dir, slots, FIFO, rate, spacing, memory reservations, nested held marker
sib|src/launch|exec, no-cmd exit, govern, background priority, env cleanup
sib|src/remote|`--host` evaluation over ssh
sib|src/profile|named switch sets from project & user files, lookup & trust rule
sib|src/advise|`advise` verb: how many jobs fit now, what limits them, what to do
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, `src/gate`. JSON written by hand or tiny serializer; new dep ! justified.

## §I INTERFACES

- `status` human table · `status --json` · `check --json` · `--verbose` block reasons · heartbeat line.
- JSON: `{ "schema_version": 1, "host": .., "signals": [{ "id", "value", "unit", "trend", "supported", "source" }], "gates": [{ "switch", "threshold", "verdict" }] }`.

## §V INVARIANTS

V1: JSON has `schema_version`; field names stable; breaking change bumps version.
V2: JSON & `status` output → stdout; ∀ else → stderr.
V3: colour only when stream is a TTY & `NO_COLOR` unset.
V4: `--verbose` names each blocking gate w/ value, threshold, trend.
V5: unsupported signals listed in `status` w/ reason, ⊥ hidden.
V6: numbers rendered w/ `.` decimal, locale ignored.
V7: warnings go through one fn honouring `--quiet` (`src/cli:V12`).

## §T TASKS

id|status|task|cites
T1|x|JSON schema v1 + writer|V1,V6
T2|x|human status table|V5,V6
T3|x|stream routing|V2
T4|x|colour policy|V3
T5|x|verbose reasons & heartbeat line|V4
T6|.|warning sink w/ quiet|V7

## §B BUGS

id|date|cause|fix
