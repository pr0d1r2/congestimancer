# SPEC

## §G GOAL

`src/report` — render readings & verdicts for humans & agents.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/report|`status` human & JSON, verbose reasons, heartbeat text, colour

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

## §T TASKS

id|status|task|cites
T1|.|JSON schema v1 + writer|V1,V6
T2|.|human status table|V5,V6
T3|.|stream routing|V2
T4|.|colour policy|V3
T5|.|verbose reasons & heartbeat line|V4

## §B BUGS

id|date|cause|fix
