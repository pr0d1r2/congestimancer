# SPEC

## §G GOAL

`src/wait` — poll loop: when to check, when to give up, when to release.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/wait|poll loop: interval, backoff, timeout, settle, jitter, heartbeat

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

## §T TASKS

id|status|task|cites
T1|.|loop w/ interval & fake clock tests|V1,V7
T2|.|backoff|V2
T3|.|timeout|V3
T4|.|settle|V4
T5|.|jitter|V5
T6|.|heartbeat|V6
T7|.|unsupported short-circuit|V8

## §B BUGS

id|date|cause|fix
