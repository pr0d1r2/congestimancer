# SPEC

## §G GOAL

`docs/` — human reference. explains; ⊥ legislates (specs = law).

## §N NAV

rel|path|lens
up|.|-
self|docs|human reference: `CONGESTION.md`, CLI guide, LLM disclaimer, third-party notices
sib|src|the tool: congestion domains, gates, waiting, launch, CLI
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
V5: `SECURITY.md`: `--when` runs shell & is refused in project `.cgmr` files; lock dir safety; `--host` uses ssh BatchMode, refuses `-` hosts & passes `--`; names used in paths restricted to `[A-Za-z0-9._-]`.

## §T TASKS

id|status|task|cites
T1|x|`CONGESTION.md` catalogue|V1
T2|x|`CLI.md`|V2
T3|x|`THIRD-PARTY-NOTICES.md`|V3
T4|x|`INTEGRATION.md`|V4
T5|x|`SECURITY.md`|V5
T6|x|`LLM-DISCLAIMER.md` as fleet|-

## §B BUGS

id|date|cause|fix
