# SPEC

## §G GOAL

`src/presence` — user idle & clock window gates.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/presence|user idle time, clock window

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, `src/sys`. only node using wall clock (`src/signal:V4`).

## §I INTERFACES

- switches: `--idle D` · `--between HH:MM-HH:MM`.
- sources: macOS IOKit `HIDIdleTime` · Linux logind session idle hint, X11|Wayland idle where available.

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

## §B BUGS

id|date|cause|fix
