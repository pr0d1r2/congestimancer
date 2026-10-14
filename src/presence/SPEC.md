# SPEC

## §G GOAL

`src/presence` — user idle & clock window gates.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/presence|user idle time, clock window
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

- deps: `src/units`, `src/signal`, `src/sys`. only node using wall clock (`src/signal:V4`).

## §I INTERFACES

- switches: `--idle D` · `--between HH:MM-HH:MM`.
- sources: macOS IOKit `HIDIdleTime` · Linux logind session idle hint, X11|Wayland idle where available.

## §R RESEARCH

id|topic|finding|src
R1|Linux idle source|open: logind `IdleHint`/`IdleSinceHint` via D-Bus (`zbus`) vs `/run/systemd/sessions/*` file; X11/Wayland without new deps?|-

## §V INVARIANTS

V1: `--idle D` ⇔ no keyboard|mouse input for ≥ D.
V2: headless Linux (no session idle source) → `Unsupported`.
V3: `--between W` ⇔ local wall time ∈ W; wraps midnight (`src/units:V7`).
V4: DST change → window evaluated on local time after change; ⊥ crash.

## §T TASKS

id|status|task|cites
T1|.|macOS idle via `HIDIdleTime`|V1
T2|.|Linux logind idle hint|V1,V2
T3|.|`--idle` gate|V1,V2
T4|.|`--between` gate w/ injectable wall clock|V3,V4
T5|.|SPIKE: settle R1 on Linux desktop & headless hosts|-

## §B BUGS

id|date|cause|fix
