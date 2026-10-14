# SPEC

## §G GOAL

`congestimancer` (bin `cgmr`): gate cmd on machine congestion. wait until ∀ requested gates pass → exec cmd; expose readings ∀ agents & scripts.

## §D DESCRIPTION

Rust lib + CLI for Linux & macOS. Measures congestion signals (CPU, memory, disk, IO, network, limits, thermal & power, user presence, GPU, processes, cgroup v2, remote host, custom), compares each to a threshold, holds a cmd until system has room, releases waiters w/o thundering herd. Human catalogue of every method: `docs/CONGESTION.md`.

## §F FEDERATION

dir|owns|⊥owns|tokens
src|the tool: congestion domains, gates, waiting, launch, CLI|repo gate, dogfood wiring, release|-
tests|integration tests driving `cgmr` binary, dogfood tests, fixtures, real-machine tests|unit tests (beside their module)|-
docs|human reference: `CONGESTION.md`, CLI guide, LLM disclaimer, third-party notices|rules (specs = law)|-
dev|`congestimancer-dev`, `publish = false`: README badges, notices closure, AGENTS step table & their drift checks|anything a consumer installs; gate ops (`hk.pkl`)|-
nix|repo gate: dev shell, guardrails, hk steps, spec & token gates, CI, release, real-machine runs|the tool|-

## §N NAV

rel|path|lens
up|-|-
self|.|-

## §C CONSTRAINTS

- Rust edition 2024, stable, MSRV 1.95 = fleet pin (`nixpkgs-lock` → nixos-26.05). ⊥ rust-toolchain file. MIT.
- workspace: crate `congestimancer` = lib + bin `cgmr` (V21); member `dev` (`publish = false`).
- direct deps: `clap` (derive) → `src/cli` · `sysinfo`, `rustix`, `libc` → `src/sys`. ∀ other dep ! justified in `docs/THIRD-PARTY-NOTICES.md` & owned by ONE facade node.
- ⊥ `humantime`: durations parsed by `src/units`. ⊥ signal-handler crate (V22).
- `unsafe_code = "deny"` crate-wide; `#[allow(unsafe_code)]` ONLY ∈ `src/sys` (`nix:V11`). `forbid` ⊥ usable ∵ inner `allow` cannot lift `forbid`.
- clippy: fleet deny list (`unwrap_used expect_used panic indexing_slicing todo unimplemented dbg_macro arithmetic_side_effects too_many_lines cognitive_complexity excessive_nesting fn_params_excessive_bools struct_excessive_bools undocumented_unsafe_blocks unreachable_pub`) + fleet `clippy.toml` thresholds; `rustfmt.toml` `max_width = 80`.
- metric math ! checked | saturating ∵ `arithmetic_side_effects`.
- platforms: `aarch64-darwin` `x86_64-darwin` `x86_64-linux` `aarch64-linux`. ⊥ Windows, ⊥ BSD. CI & hand-verified targets → `nix`.
- federation = `sherd`: dir = node = Rust module (dir + `mod.rs`). repo gate, tooling, release → `nix` node.
- rules & tasks moved to `nix` keep their numbers (`nix` §G) ∴ freed numbers ⊥ reused here.

## §I INTERFACES

- cmd: `cgmr [run] [gates] [-- cmd...]` · `cgmr check [gates]` · `cgmr status [--json]` · `cgmr completions <shell>` · `cgmr man`. contract → `src/cli` §I.
- gates & strategies: full list, semantics, sources → `docs/CONGESTION.md` (human) & domain nodes `src/*` §V (law).
- env: `CGMR_<SWITCH>` default ∀ switch except `--when`; precedence flag > env > builtin. `CGMR_DOGFOOD`, hidden `CGMR_FIXTURE_ROOT`.
- exit: cmd's own code once exec'd · 0 gates passed & no cmd · 1 `check` failed · 64 usage · 69 gate unsupported on this OS · 124 `--timeout` · 130/143 SIGINT/SIGTERM while waiting (default disposition).
- lib: `congestimancer` re-exports gate model, readers & evaluation for reuse by agents & tools → `src` §V.
- files: per-user slot dir (`src/slot` §I).

## §V INVARIANTS

V1: dogfood: own gate runs heavy pre-push steps (coverage, package) & parallel sherd wave builds under `cgmr` once `run` works; hk step = `cargo run --bin cgmr` shim ∴ compile error ⊥ locks shell.
V18: nothing deferred: ∀ method ∈ `docs/CONGESTION.md` ! own node §V & §T row. doc ⊥ spec parity = bug.
V21: crate `congestimancer`, bin `cgmr`. REJECTED: bin `la` (taken; clashes `alias la`), bin `cong` (US slur), bin `cgm` (PyPI taken; glucose meter), bin `gest` (taken).
V22: SIGINT|SIGTERM while waiting → default disposition (130|143); cmd ⊥ runs. ⊥ handler crate. `--govern` = only parent-mode exception (`src/launch`).

## §T TASKS

id|status|task|cites
T14|.|publish `0.1.0-rc.1` to crates.io ! user approval|V21
T15|.|parity check `docs/CONGESTION.md` ↔ node specs|V18

## §B BUGS

id|date|cause|fix
