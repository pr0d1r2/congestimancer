# SPEC

## §G GOAL

`tests/` — integration tests driving `cgmr` binary, dogfood & real-machine runs.

## §N NAV

rel|path|lens
up|.|-
self|tests|integration tests driving `cgmr` binary, dogfood tests, fixtures, real-machine tests
sib|src|the tool: congestion domains, gates, waiting, launch, CLI
sib|docs|human reference: `CONGESTION.md`, CLI guide, LLM disclaimer, third-party notices
sib|dev|`congestimancer-dev`, `publish = false`: README badges, notices closure, AGENTS step table & their drift checks
sib|nix|repo gate: dev shell, guardrails, hk steps, spec & token gates, CI, release, real-machine runs

## §C CONSTRAINTS

- dev-deps: `assert_cmd`, `predicates`, `proptest`, `tempfile`.
- layout: `tests/cli.rs`, `tests/dogfood.rs`, `tests/real.rs`, `tests/fixtures/<node>/`.

## §V INVARIANTS

V1: CLI tests run binary w/ `CGMR_FIXTURE_ROOT` → fixture tree; ⊥ live machine.
V2: ∀ exit code ∈ `.` §I covered by a CLI test.
V3: ∀ switch ∈ `docs/CONGESTION.md` parsed & reaches its gate in a CLI test (parity, `.:V18`).
V4: real-machine tests `#[ignore]`; run by `cargo test -- --ignored` on test hosts (`nix:V20`); assert readings plausible, ⊥ exact values.
V5: dogfood test: `cgmr check` on own machine exits 0|1, ⊥ 64|69 for gates supported on host OS.
V6: fixtures captured from real hosts named by OS & arch in path (e.g. `cpu/linux-x86_64/stat`).

## §T TASKS

id|status|task|cites
T1|~|fixture tree filled by `cgmr debug capture` on ∀ test hosts|V6
T2|x|`tests/cli.rs` exit codes|V1,V2
T3|x|switch parity test from switch table|V3
T4|x|`tests/real.rs` ignored real-machine suite|V4
T5|x|`tests/dogfood.rs`|V5
T6|x|`--govern` jobs of a slot key take turns, through the binary|V3

## §B BUGS

id|date|cause|fix
