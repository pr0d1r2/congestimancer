# SPEC

## §G GOAL

`src/advise` — tell agents & scripts how much work fits now and what to do.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/advise|`advise` verb: how many jobs fit now, what limits them, what to do
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
sib|src/report|`status` human & JSON, verbose reasons, heartbeat text, colour
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, `src/gate`, `src/report`, `src/slot` (reservations), domain nodes `cpu`, `mem`, `io` (public surface).

## §I INTERFACES

- verb: `cgmr advise [gates] [--jobs J] [--needs-mem X] [--json]`.
- output: `jobs` (how many parallel jobs fit now), `limit` (resource that bounds it), `action` (`run` | `reduce` | `wait`), `blocking` (failing gates w/ value & threshold).

## §V INVARIANTS

V1: cpu fit = floor(N × L − load), L = `--load` threshold (default 1.0); < 0 → 0.
V2: memory fit = floor((available − reserved) / M), M = `--needs-mem` (default 1GiB); reserved from `src/slot`.
V3: jobs = min(cpu fit, memory fit, J if `--jobs J`); any other requested gate failing → jobs = 0.
V4: action: `run` ⇔ jobs ≥ 1 & (no J | jobs ≥ J) · `reduce` ⇔ 1 ≤ jobs < J · `wait` ⇔ jobs = 0.
V5: `limit` names the resource giving the minimum (`cpu` | `mem` | `jobs` | gate switch name).
V6: never waits except one extra sample for rate signals (`src/gate:V8`); exit 0 always unless usage 64 | unsupported 69.
V7: JSON shares `schema_version` w/ `src/report`; human form = one line, e.g. `jobs=3 limit=mem action=reduce`.

## §T TASKS

id|status|task|cites
T1|.|cpu & memory fit|V1,V2
T2|.|jobs, limit & action|V3,V4,V5
T3|.|verb wiring & exit codes|V6
T4|.|human & JSON output|V7

## §B BUGS

id|date|cause|fix
