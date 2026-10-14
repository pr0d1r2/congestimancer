# SPEC

## §G GOAL

`src/signal` — common model ∀ congestion signals: what a reading is, how it is sampled, how tests fake it.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/signal|`Signal` & `Source` traits, `Reading`, sampler w/ two-sample deltas, monotonic clock, fixture root

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
T1|.|`SignalId` enum w/ ∀ signals + names = switch names|V6
T2|.|`Reading` & `Unsupported` reason|V6
T3|.|`Source` & `Clock` traits + fakes for tests|V4,V5
T4|.|`Sampler` deltas, warming, reset handling|V1,V2
T5|.|`Root` path joining|V5
T6|.|selective refresh set from requested gates|V3
T7|.|trend from last 2 values|V7
T8|.|PSI text parser shared by pressure gates|-

## §B BUGS

id|date|cause|fix
