# SPEC

## §G GOAL

`src/procs` — gates on how many processes match a name or user.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/procs|process counts by name & by user

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, `src/sys` (sysinfo process list). own glob matcher (zero-dep).

## §I INTERFACES

- switches: `--max-procs NAME=N` · `--max-user-procs USER=N`; both repeatable; NAME & USER = glob (`*`, `?`).

## §V INVARIANTS

V1: `--max-procs P=N` ⇔ count(processes w/ name matching P) ≤ N.
V2: `--max-user-procs U=N` ⇔ count(processes owned by user matching U) ≤ N.
V3: cgmr itself & its ancestors ⊥ counted.
V4: malformed `NAME=N` → usage 64.

## §T TASKS

id|status|task|cites
T1|.|glob matcher|V1,V2
T2|.|`NAME=N` parser|V4
T3|.|`--max-procs`|V1,V3
T4|.|`--max-user-procs` (e.g. `nixbld*`)|V2,V3

## §B BUGS

id|date|cause|fix
