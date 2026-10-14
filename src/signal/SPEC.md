# SPEC

## §G GOAL

`src/signal` — common model ∀ congestion signals: what a reading is, how it is sampled, how tests fake it.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/signal|`Signal` & `Source` traits, `Reading`, sampler w/ two-sample deltas, monotonic clock, fixture root
sib|src/units|parse & render sizes, rates, percents, durations, temperatures, clock windows
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
sib|src/profile|named switch sets from project & user files, lookup & trust rule
sib|src/advise|`advise` verb: how many jobs fit now, what limits them, what to do
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- deps: `src/units`. ⊥ OS access.

## §I INTERFACES

- `SignalId`: one variant per signal ∈ `docs/CONGESTION.md`.
- `Reading`: `Value(f64|u64 w/ unit)` · `Warming` (first sample of a delta) · `Unsupported { reason }`.
- `Source` trait: `read(&mut self, id, now) -> Reading`. real impls ∈ domain nodes; fakes ∈ tests.
- `Clock` trait: monotonic `now()` & `sleep(d)`; real = `std::time::Instant`; fake = manual.
- `Sampler`: keeps previous counters per `SignalId`, computes rate|percent over elapsed monotonic time.
- `Root`: filesystem root for `/proc`, `/sys`, cgroup paths; default `/`; tests pass fixture dir.
- PSI text parser (`some|full avg10 avg60 avg300 total`) shared by `src/cpu`, `src/mem`, `src/io`, `src/cgroup`.

## §V INVARIANTS

V1: rate|percent signal → first sample = `Warming`, ⊥ fake zero; value after ≥ 1 interval.
V2: counter decrease (reset, wrap) → `Warming` again, ⊥ negative rate.
V3: only signals referenced by requested gates refresh; others ⊥ touched (cost, `docs/CONGESTION.md`).
V4: ∀ time math uses `Clock` (monotonic); ⊥ wall clock except `presence` window.
V5: fixture root passed as argument; ⊥ `set_var`, ⊥ global state.
V6: `Reading` carries unit ∴ `report` renders w/o knowing source.
V7: trend (rising|falling|flat) derivable from last 2 values per signal.

## §T TASKS

id|status|task|cites
T1|x|`SignalId` enum w/ ∀ signals + names = switch names|V6
T2|x|`Reading` & `Unsupported` reason|V6
T3|x|`Source` & `Clock` traits + fakes for tests|V4,V5
T4|x|`Sampler` deltas, warming, reset handling|V1,V2
T5|x|`Root` path joining|V5
T6|x|selective refresh set from requested gates|V3
T7|.|trend from last 2 values|V7
T8|.|PSI text parser shared by pressure gates|-

## §B BUGS

id|date|cause|fix
