# SPEC

## §G GOAL

`dev/` — `congestimancer-dev` crate, `publish = false`: generated docs bits & their drift checks.

## §N NAV

rel|path|lens
up|.|-
self|dev|`congestimancer-dev`, `publish = false`: README badges, notices closure, AGENTS step table & their drift checks
sib|src|the tool: congestion domains, gates, waiting, launch, CLI
sib|tests|integration tests driving `cgmr` binary, dogfood tests, fixtures, real-machine tests
sib|docs|human reference: `CONGESTION.md`, CLI guide, LLM disclaimer, third-party notices
sib|nix|repo gate: dev shell, guardrails, hk steps, spec & token gates, CI, release, real-machine runs

## §C CONSTRAINTS

- workspace member, `publish = false`, `release = false`. shape copied from fleet `dev` crates.

## §V INVARIANTS

V1: README `<!-- BEGIN badges -->` block generated; drift → gate fails.
V2: third-party notices closure generated from `Cargo.lock`; drift → gate fails.
V3: AGENTS.md step table generated from `hk.pkl`; drift → gate fails.

## §T TASKS

id|status|task|cites
T1|x|dev crate scaffold|-
T2|.|badges generator + check|V1
T3|.|notices generator + check|V2
T4|.|AGENTS step table generator + check|V3

## §B BUGS

id|date|cause|fix
