# SPEC

## §G GOAL

`src/cli` — argv → verb & config; config → exit code.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- facade ∀ `clap` (derive) & `clap_complete`, `clap_mangen`. deps: ∀ other nodes via public surface.

## §I INTERFACES

- verbs: `run` (default) · `check` · `status [--json]` · `completions <shell>` · `man`.
- `cgmr [run] [gates] [-- cmd...]`; cmd only after `--`.
- ∀ switch ∈ `docs/CONGESTION.md` w/ env `CGMR_<SWITCH>` (upper snake), except `--when`.
- hidden: `CGMR_FIXTURE_ROOT` (tests), `CGMR_DOGFOOD`.

## §V INVARIANTS

V1: verb omitted → `run`.
V2: precedence: flag > env > builtin default.
V3: `--when` ⊥ read from env.
V4: exit: cmd's own (exec) · 0 released & no cmd · `check`: 0 pass, 1 fail · 64 usage (clap errors, `UnitError`) · 69 unsupported · 124 timeout.
V5: exit-code collision (cmd itself exits 124|64|69) documented in `--help` & `docs/CLI.md`.
V6: `--help` shows examples ∀ verb.
V7: switch names & env names generated from one table ∴ `docs/CONGESTION.md` parity testable (`.:V18`).
V8: `CGMR_FIXTURE_ROOT` read once in `main`, passed down as `Root`; ⊥ `set_var`.

## §T TASKS

id|status|task|cites
T1|.|clap derive skeleton, verbs, default `run`|V1
T2|.|switch table + env names|V2,V3,V7
T3|.|exit code mapping|V4
T4|.|help text w/ examples & collision note|V5,V6
T5|.|`completions` & `man` verbs|V6
T6|.|fixture root & dogfood plumbing|V8

## §B BUGS

id|date|cause|fix
