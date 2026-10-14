# SPEC

## §G GOAL

`src/cgroup` — container awareness: cgroup v2 limits & pressure replace host-wide values.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/cgroup|cgroup v2 detection, limits, throttling, pressure override

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
T1|.|detect cgroup v2 path from `/proc/self/cgroup`|V1,V5
T2|.|`cpu.max` `memory.max` `memory.current` parsers|V1,V2
T3|.|`cpu.stat` parser + `--cpu-throttle`|V4
T4|.|pressure source override ∀ PSI gates|V2
T5|.|`--load` warning in cgroup|V3
T6|.|`--cgroup` modes|V1
T7|.|status dual values|V6

## §B BUGS

id|date|cause|fix
