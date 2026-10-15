# SPEC

## §G GOAL

`src/wait` — poll loop: when to check, when to give up, when to release.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/wait|poll loop: interval, backoff, timeout, settle, jitter, heartbeat
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
sib|src/slot|per-user lock dir, slots, FIFO, rate, spacing, memory reservations, nested held marker
sib|src/launch|exec, no-cmd exit, govern, background priority, env cleanup
sib|src/remote|`--host` evaluation over ssh
sib|src/report|`status` human & JSON, verbose reasons, heartbeat text, colour
sib|src/profile|named switch sets from project & user files, lookup & trust rule
sib|src/advise|`advise` verb: how many jobs fit now, what limits them, what to do
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- deps: `src/units`, `src/signal` (`Clock`), `src/gate`, `src/report` (heartbeat text). tests use fake `Clock` ∴ ⊥ real sleeps.

## §I INTERFACES

- switches: `--interval D` (default `2s`) · `--backoff D` · `--timeout D` · `--settle D` · `--jitter D` · `--heartbeat D`.
- outcome: `Released` · `TimedOut` · `Unsupported`.

## §V INVARIANTS

V1: sleep between checks; ⊥ busy loop. 100 waiters ⊥ measurable load.
V2: `--backoff D`: each consecutive failed round doubles interval, capped at D; any pass resets to `--interval`.
V3: `--timeout D` reached → `TimedOut` → exit 124 (unless `--or-background`, `src/launch`).
V4: `--settle D`: overall pass ! hold continuously for D; any non-pass round restarts settle.
V5: `--jitter D`: after release, sleep random ∈ [0, D) before start.
V6: `--heartbeat D`: while waiting, one status line to stderr every D (text from `src/report`).
V7: ∀ durations measured on monotonic clock; suspend behaviour = OS monotonic semantics, documented.
V8: `Unsupported` → return at once (`src/gate:V3`).
V9: release order: gates pass → `--settle` holds → FIFO ticket (`--fifo`) → slot (`--slots`) → `--rate` & `--spacing` → memory reservation (`--needs-mem`) → `--jitter` → launch. failure at any step → back to gates.
V10: cost budget: one round w/ `--load` only ≤ 5 ms CPU; resident memory ≤ 10 MB; cold start of `cgmr check --load 1` ≤ 20 ms. benchmark gated in hk `all` (`nix:V25`).

## §T TASKS

id|status|task|cites
T1|x|loop w/ interval & fake clock tests|V1,V7
T2|x|backoff|V2
T3|x|timeout|V3
T4|.|settle|V4
T5|.|jitter|V5
T6|.|heartbeat|V6
T7|.|unsupported short-circuit|V8
T8|.|release pipeline order|V9
T9|.|benchmarks for round CPU, RSS, cold start|V10

## §B BUGS

id|date|cause|fix
