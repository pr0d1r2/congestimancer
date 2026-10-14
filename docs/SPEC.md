# SPEC

## §G GOAL

`docs/` — human reference. explains; ⊥ legislates (specs = law).

## §N NAV

rel|path|lens
up|.|-
self|docs|human reference: `CONGESTION.md`, CLI guide, LLM disclaimer, third-party notices
sib|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
sib|tests|integration tests driving `cgmr` binary, dogfood tests, fixtures, real-machine tests
sib|dev|`congestimancer-dev`, `publish = false`: README badges, notices closure, AGENTS step table & their drift checks
sib|nix|repo gate: dev shell, guardrails, hk steps, spec & token gates, CI, release, real-machine runs

## §C CONSTRAINTS

- files: `CONGESTION.md` (exists), `CLI.md`, `INTEGRATION.md`, `SECURITY.md`, `THIRD-PARTY-NOTICES.md`, `LLM-DISCLAIMER.md`.
- plain prose, ASCII only (`nix:V12`).

## §V INVARIANTS

V1: `CONGESTION.md` lists ∀ signal, switch, strategy w/ semantics & sources; ∀ entry ↔ node spec (`.:V18`).
V2: `CLI.md` documents verbs, exit codes & exit-code collision.
V3: `THIRD-PARTY-NOTICES.md` justifies ∀ direct dep; generated closure checked by `dev`.
V4: `INTEGRATION.md`: hk|lefthook|CI|agent usage examples, incl. dogfood pattern.
V5: `SECURITY.md`: `--when` runs shell; lock dir safety; `--host` uses ssh BatchMode.

## §T TASKS

id|status|task|cites
T1|x|`CONGESTION.md` catalogue|V1
T2|.|`CLI.md`|V2
T3|.|`THIRD-PARTY-NOTICES.md`|V3
T4|.|`INTEGRATION.md`|V4
T5|.|`SECURITY.md`|V5
T6|.|`LLM-DISCLAIMER.md` as fleet|-

## §B BUGS

id|date|cause|fix
