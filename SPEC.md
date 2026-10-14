# SPEC

## §G GOAL

`congestimancer` (bin `cgmr`): gate cmd on machine congestion. wait until ∀ requested gates pass → exec cmd; expose readings ∀ agents & scripts.

## §D DESCRIPTION

Rust lib + CLI for Linux & macOS. Measures congestion signals (CPU, memory, disk, IO, network, limits, thermal & power, user presence, GPU, processes, cgroup v2, remote host, custom), compares each to a threshold, holds a cmd until system has room, releases waiters w/o thundering herd. Human catalogue of every method: `docs/CONGESTION.md`.

## §F FEDERATION

dir|owns|⊥owns|tokens
src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI|repo gate, dogfood wiring, release|-
tests|integration tests driving `cgmr` binary, dogfood tests, fixtures, real-machine tests|unit tests (beside their module)|-
docs|human reference: `CONGESTION.md`, CLI guide, LLM disclaimer, third-party notices|rules (specs = law)|-
dev|`congestimancer-dev`, `publish = false`: README badges, notices closure, AGENTS step table & their drift checks|anything a consumer installs; gate ops (`hk.pkl`)|-

## §N NAV

rel|path|lens
up|-|-
self|.|-

## §C CONSTRAINTS

- Rust edition 2024, stable, MSRV 1.95 = fleet pin (`nixpkgs-lock` → nixos-26.05). ⊥ rust-toolchain file. MIT.
- workspace: crate `congestimancer` = lib + bin `cgmr` (V21); member `dev` (`publish = false`).
- direct deps: `clap` (derive) → `src/cli` · `sysinfo`, `rustix`, `libc` → `src/sys`. ∀ other dep ! justified in `docs/THIRD-PARTY-NOTICES.md` & owned by ONE facade node.
- ⊥ `humantime`: durations parsed by `src/units`. ⊥ signal-handler crate (V22).
- `unsafe_code = "deny"` crate-wide; `#[allow(unsafe_code)]` ONLY ∈ `src/sys` (V11). `forbid` ⊥ usable ∵ inner `allow` cannot lift `forbid`.
- clippy: fleet deny list (`unwrap_used expect_used panic indexing_slicing todo unimplemented dbg_macro arithmetic_side_effects too_many_lines cognitive_complexity excessive_nesting fn_params_excessive_bools struct_excessive_bools undocumented_unsafe_blocks unreachable_pub`) + fleet `clippy.toml` thresholds; `rustfmt.toml` `max_width = 80`.
- metric math ! checked | saturating ∵ `arithmetic_side_effects`.
- platforms: `aarch64-darwin` `x86_64-darwin` `x86_64-linux` `aarch64-linux`. ⊥ Windows, ⊥ BSD. CI: 3 runners (ubuntu-latest, ubuntu-24.04-arm, macos-latest); `x86_64-darwin` verified by hand on Intel Mac.
- SPEC.md FORMAT = cavekit 4.1.0 as vendored by `microlith` `v0.7.4` (`FORMAT.md`, `FORMAT-EXTENSIONS.md`, `.format-upstream`).
- federation = `sherd`: dir = node = Rust module (dir + `mod.rs`).
- guardrail tools, each pinned by tag in `flake.nix`: `pklith` (hooks from repo files) · `nix-hk` (hk) · `sherd` (SPEC-DAG, budgets, waves) · `microlith` (spec ops) · `itok` (context limits) · `characterminator` (allowed chars) · `rekall` (memory → rules, honesty gate) · `xenolith` (one language per file, 1:1 test mirror, TDD order).
- dev shell = `flake.nix` + `.envrc` `use flake`; ∀ inputs follow `nixpkgs-lock`; substituter `pr0d1r2.cachix.org`.
- distribution: `cargo install congestimancer` & flake package (`nix build`, `nix run`). ⊥ Homebrew, ⊥ prebuilt binaries.
- release: `cargo-release` (`release.toml`, pre-release hook `hk check --all`), Keep a Changelog, version ladder (even minor = stable). first publish `0.1.0-rc.1` ! explicit user approval.
- real-machine test hosts: `nix-builder.local` (Linux) · `hallucinogen.local` (Linux) · `unicomplex.local` (Linux, T440p, x86_64) · M4 Mac (aarch64-darwin, dev machine) · Intel Mac (x86_64-darwin).

## §I INTERFACES

- cmd: `cgmr [run] [gates] [-- cmd...]` · `cgmr check [gates]` · `cgmr status [--json]` · `cgmr completions <shell>` · `cgmr man`. contract → `src/cli` §I.
- gates & strategies: full list, semantics, sources → `docs/CONGESTION.md` (human) & domain nodes `src/*` §V (law).
- env: `CGMR_<SWITCH>` default ∀ switch except `--when`; precedence flag > env > builtin. `CGMR_DOGFOOD`, hidden `CGMR_FIXTURE_ROOT`.
- exit: cmd's own code once exec'd · 0 gates passed & no cmd · 1 `check` failed · 64 usage · 69 gate unsupported on this OS · 124 `--timeout` · 130/143 SIGINT/SIGTERM while waiting (default disposition).
- lib: `congestimancer` re-exports gate model, readers & evaluation for reuse by agents & tools → `src` §V.
- files: per-user slot dir (`src/slot` §I).

## §V INVARIANTS

V1: dogfood: own gate runs heavy pre-push steps (coverage, package) & parallel sherd wave builds under `cgmr` once `run` works; hk step = `cargo run --bin cgmr` shim ∴ compile error ⊥ locks shell.
V2: SPEC.md form gated by `mth fmt --check` & `mth check --records .spec-records` ∀ node SPEC.md. `mth` absent → gate FAILS, ⊥ skip.
V3: ∀ SPEC.md capped by `.context-limits` row, gated by `itok check`; ceiling ~12% over measured; cap: file ≤ 3500 tok, chain ≤ 5500 tok; over cap → `sherd split`, ⊥ raise.
V4: `sherd check`, `sherd budget`, `sherd sync --check` gate from commit one.
V5: line coverage GATED: floor `cargo llvm-cov --fail-under-lines` hard min; `.coverage` = ceiling claim; measured < claim → FAIL; measured − claim > 0.5 → FAIL. aim 100%.
V6: 1:1 unit mirror: ∀ `.rs` w/ fn bodies ! sibling `tests.rs` wired by `#[cfg(test)] mod tests;` (xenolith `rust-mirror`).
V7: TDD: RED → GREEN → REFACTOR. `test:` commit ! precede impl commit in push range (xenolith `tdd-order`).
V8: commits atomic, one topic each, human readable; commit-msg lint `type(scope): claim` ≤ 72 chars; feat|fix|perf ! touch CHANGELOG.
V9: ∀ flake input pins TAG, bumped in own reviewed commit. `nixpkgs-lock` & `nix-hk` exempt.
V10: `cargo deny check` gates: `licenses allow` per `deny.toml`, `multiple-versions = "deny"`, unknown registry|git denied, yanked denied.
V11: `allow(unsafe_code)` ∈ `src/sys` only; ∀ unsafe block ! `// SAFETY:` comment; hk step greps tree & FAILS on any other `allow(unsafe_code)`.
V12: `ctrm check` gates tree via `.ctrm`: `*` ascii · `*.md` ascii+spec · `README.md` `docs/**/*.md` ascii · `AGENTS.md` ascii+caveman.
V13: `xnl check` gates: one language per file, ⊥ embedded foreign code.
V14: `rekall` gate: rules & skills extracted from agent memory & CLAUDE.md stay honest.
V15: guardrails installed on dev-shell entry (pklith `lib.devShell`, tracked `.githooks/`); hook install refuses loudly when `hk` absent.
V16: module size ≤ 500 code lines, ≤ 1000 total (hk `module-size`).
V17: cargo steps run serially ∵ target-dir lock.
V18: nothing deferred: ∀ method ∈ `docs/CONGESTION.md` ! own node §V & §T row. doc ⊥ spec parity = bug.
V19: hk `fast` = pre-commit; `all` = pre-push, `check`, `fix`; `fail_fast`.
V20: release ! real-machine run (`cargo test -- --ignored`) on ∀ test hosts; x86_64-darwin ! hand-verified on Intel Mac.
V21: crate `congestimancer`, bin `cgmr`. REJECTED: bin `la` (taken; clashes `alias la`), bin `cong` (US slur), bin `cgm` (PyPI taken; glucose meter), bin `gest` (taken).
V22: SIGINT|SIGTERM while waiting → default disposition (130|143); cmd ⊥ runs. ⊥ handler crate. `--govern` = only parent-mode exception (`src/launch`).
V23: CI = one `ci.yml`: `nix develop --command hk check --all --check --no-fail-fast` → `nix build .#default`; actions pinned by SHA, `permissions: contents: read`, `persist-credentials: false`, zizmor & actionlint gate.

## §T TASKS

id|status|task|cites
T1|x|vendor FORMAT files from microlith `v0.7.4`|V2
T2|.|`flake.nix` dev shell + `.envrc` `use flake`; inputs follow `nixpkgs-lock`; tools pinned by tag. then ASK user to reload & continue|V9
T3|.|guardrails: pklith registry & `hk.pkl` (fast/all), `.githooks/`, hygiene steps, ripsecrets, typos, taplo, nixfmt, actionlint, zizmor, lychee. then ASK user to reload & continue|V15,V19
T4|.|spec gates: `mth fmt/check`, `.spec-records`, `.context-limits` + `itok check`, `sherd check/budget/sync`, `ceiling-cap`|V2,V3,V4
T5|.|char & language gates: `.ctrm` + `ctrm check`, `xnl check`, rekall gate|V12,V13,V14
T6|.|cargo workspace: lib + bin `cgmr` + `dev` crate; lints, `clippy.toml`, `rustfmt.toml`, `deny.toml`, `exclude`, `must-package`|V10,V11,V21
T7|.|test gates: rust-mirror, tdd-order, module-size, coverage floor & `.coverage` ceiling w/ LLVM major check|V5,V6,V7,V16
T8|.|commit-msg lint & changelog-touched; CHANGELOG, README (badges block), LICENSE MIT, AGENTS.md|V8
T9|.|unsafe gate: hk step FAILS on `allow(unsafe_code)` outside `src/sys`|V11
T10|.|`ci.yml` 3-runner matrix|V23
T11|.|`release.toml` + cargo-release pre-release hook|V20
T12|.|dogfood wiring: shim + `CGMR_DOGFOOD`; heavy pre-push steps under `cgmr`|V1
T13|.|real-machine test run on ∀ hosts; Intel Mac by hand|V20
T14|.|publish `0.1.0-rc.1` to crates.io ! user approval|V21
T15|.|parity check `docs/CONGESTION.md` ↔ node specs|V18

## §B BUGS

id|date|cause|fix
