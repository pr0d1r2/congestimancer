# SPEC

## §G GOAL

`nix/` — repo gate: dev shell, guardrails, hk steps, spec & token gates, CI, release, real-machine runs. ids kept from root on move (V2–V17, V19, V20, V23; T1–T13) ∴ `nix:V11` = former root V11.

## §N NAV

rel|path|lens
up|.|-
self|nix|repo gate: dev shell, guardrails, hk steps, spec & token gates, CI, release, real-machine runs
sib|src|the tool: congestion domains, gates, waiting, launch, CLI
sib|tests|integration tests driving `cgmr` binary, dogfood tests, fixtures, real-machine tests
sib|docs|human reference: `CONGESTION.md`, CLI guide, LLM disclaimer, third-party notices
sib|dev|`congestimancer-dev`, `publish = false`: README badges, notices closure, AGENTS step table & their drift checks

## §C CONSTRAINTS

- dev shell = `flake.nix` + `.envrc` `use flake`; ∀ inputs follow `nixpkgs-lock`; substituter `pr0d1r2.cachix.org`. pklith-generated `nix/pklith.nix` lives here.
- guardrail tools, each pinned by tag in `flake.nix`: `pklith` (hooks from repo files) · `nix-hk` (hk) · `sherd` (SPEC-DAG, budgets, waves) · `microlith` (spec ops) · `itok` (context limits) · `characterminator` (allowed chars) · `rekall` (memory → rules, honesty gate) · `xenolith` (one language per file, 1:1 test mirror, TDD order).
- gate ops ∈ `hk.pkl` (repo root) + vendored `pkl/Config.pkl`; owned by this node.
- SPEC.md FORMAT = cavekit 4.1.0 as vendored by `microlith` `v0.7.4` (`FORMAT.md`, `FORMAT-EXTENSIONS.md`, `.format-upstream`).
- CI: 3 runners (ubuntu-latest, ubuntu-24.04-arm, macos-latest); `x86_64-darwin` verified by hand on Intel Mac.
- distribution: `cargo install congestimancer` & flake package (`nix build`, `nix run`). ⊥ Homebrew, ⊥ prebuilt binaries.
- release: `cargo-release` (`release.toml`, pre-release hook `hk check --all`), Keep a Changelog, version ladder (even minor = stable). first publish `0.1.0-rc.1` ! explicit user approval (`.:T14`).
- real-machine test hosts: `nix-builder.local` (Linux) · `hallucinogen.local` (Linux) · `unicomplex.local` (Linux, T440p, x86_64) · M4 Mac (aarch64-darwin, dev machine) · Intel Mac (x86_64-darwin).
- spec budget tooling reads `sherd budget --format json` (`own_tokens`, `chain_tokens`), ⊥ text output.

## §V INVARIANTS

V2: SPEC.md form gated by `mth fmt --check` & `mth check --records .spec-records` ∀ node SPEC.md. `mth` absent → gate FAILS, ⊥ skip.
V3: ∀ SPEC.md capped by `.context-limits` row, gated by `itok check`; ceiling ~12% over measured; cap: file ≤ 3500 tok, chain ≤ 5500 tok; over cap → `sherd split`, ⊥ raise.
V4: `sherd check`, `sherd budget`, `sherd sync --check` gate from commit one.
V5: line coverage GATED: floor `cargo llvm-cov --fail-under-lines` hard min; `.coverage` = ceiling claim; measured < claim → FAIL; measured − claim > 0.5 → FAIL. aim 100%.
V6: 1:1 unit mirror: ∀ `.rs` w/ fn bodies ! sibling `tests.rs` wired by `#[cfg(test)] mod tests;` (xenolith `rust-mirror`).
V7: TDD: RED → GREEN → REFACTOR. `test:` commit ! precede impl commit in push range (xenolith `tdd-order`).
V8: commits atomic, one topic each, human readable; commit-msg lint `type(scope): claim` ≤ 72 chars; feat|fix|perf ! touch CHANGELOG.
V9: ∀ flake input pins TAG, bumped in own reviewed commit. `nixpkgs-lock` & `nix-hk` exempt.
V10: `cargo deny check` gates: `licenses allow` = `MIT` + `Unicode-3.0` (`unicode-ident` via clap derive), `multiple-versions = "deny"`, unknown registry|git denied, yanked denied. `[graph] targets` = the 4 platform triples ∴ Windows-only crates ⊥ counted.
V11: `allow(unsafe_code)` ∈ `src/sys` only; ∀ unsafe block ! `// SAFETY:` comment; hk step greps tree & FAILS on any other `allow(unsafe_code)`.
V12: `ctrm check` gates tree via `.ctrm`, last match wins: `*` ascii · `*.md` ascii+spec · `README.md` `docs/**/*.md` ascii · `AGENTS.md` ascii+caveman · `**/SPEC.md` `FORMAT.md` `FORMAT-EXTENSIONS.md` ascii+spec (LAST ∵ `docs/SPEC.md` ∈ `docs/**`) · `pkl/Config.pkl` any (vendored).
V13: `xnl check` gates: one language per file, ⊥ embedded foreign code.
V14: `rekall` gate: rules & skills extracted from agent memory & CLAUDE.md stay honest.
V15: guardrails installed on dev-shell entry (pklith `lib.devShell`, tracked `.githooks/`); hook install refuses loudly when `hk` absent.
V16: module size ≤ 500 code lines, ≤ 1000 total (hk `module-size`).
V17: cargo steps run serially ∵ target-dir lock.
V19: hk `fast` = pre-commit; `all` = pre-push, `check`, `fix`; `fail_fast`.
V20: release ! real-machine run (`cargo test -- --ignored`) on ∀ test hosts; x86_64-darwin ! hand-verified on Intel Mac.
V23: CI = one `ci.yml`: `nix develop --command hk check --all --check --no-fail-fast` → `nix build .#default`; actions pinned by SHA, `permissions: contents: read`, `persist-credentials: false`, zizmor & actionlint gate.
V24: hk `all` adds fleet release-grade steps: `cargo doc` w/ `RUSTDOCFLAGS=-D warnings` · `cargo package --locked` + `must-package` · `cargo semver-checks` (from first publish) · `--no-default-features` build (`nvml` = optional default feature, `src/gpu`).
V25: hk `all` runs cost benchmarks (`src/wait:V10`); over budget → FAIL.
V26: flake exports ∀ 4 systems: `packages.default` (`buildRustPackage`, bin `cgmr`) · `apps.default` · `overlays.default` (adds `cgmr` to pkgs) ∴ other fleet repos use `cgmr` in their hk gates by pinning a tag.
V27: overnight build harness = Claude Code workflow (sherd cannot execute waves yet: sherd T106, issue #132): rounds follow hub order (`src` §C) after spike round (`src:V6`); one git worktree per node per round; each worker does RED → GREEN → REFACTOR on its node only & runs hk `fast`; round merges in hub order after hk `all`; failed node → reverted & logged, round continues. workers start under `cgmr --profile build` (`.cgmr`) once `run` exists ∴ machine ⊥ overcommitted.

## §T TASKS

id|status|task|cites
T1|x|vendor FORMAT files from microlith `v0.7.4`|V2
T2|x|`flake.nix` dev shell + `.envrc` `use flake`; inputs follow `nixpkgs-lock`; tools pinned by tag. then ASK user to reload & continue|V9
T3|x|guardrails: pklith registry & `hk.pkl` (fast/all), `.githooks/`, hygiene steps, ripsecrets, typos, taplo, nixfmt, actionlint, zizmor, lychee. then ASK user to reload & continue|V15,V19
T4|x|spec gates: `mth fmt/check`, `.spec-records`, `.context-limits` + `itok check`, `sherd check/budget/sync`, `ceiling-cap`|V2,V3,V4
T5|x|char & language gates: `.ctrm` + `ctrm check`, `xnl check`, rekall gate|V12,V13,V14
T6|x|cargo workspace: lib + bin `cgmr` + `dev` crate; lints, `clippy.toml`, `rustfmt.toml`, `deny.toml`, `exclude`, `must-package`|V10,V11,`.:V21`
T7|x|test gates: rust-mirror, tdd-order, module-size, coverage floor & `.coverage` ceiling w/ LLVM major check|V5,V6,V7,V16
T8|x|commit-msg lint & changelog-touched; CHANGELOG, README (badges block), LICENSE MIT, AGENTS.md|V8
T9|x|unsafe gate: hk step FAILS on `allow(unsafe_code)` outside `src/sys`|V11
T10|.|`ci.yml` 3-runner matrix|V23
T11|.|`release.toml` + cargo-release pre-release hook|V20
T12|.|dogfood wiring: shim + `CGMR_DOGFOOD`; heavy pre-push steps under `cgmr`|`.:V1`
T13|.|real-machine test run on ∀ hosts; Intel Mac by hand|V20
T16|.|hk `all`: rustdoc, package, semver-checks, no-default-features steps|V24
T17|.|benchmark step in hk `all`|V25
T18|.|flake `packages.default`, `apps.default`, `overlays.default`; `nix flake check` covers them|V26
T19|.|build harness workflow + `.cgmr` `build` profile|V27,`.:V1`

## §B BUGS

id|date|cause|fix
B1|2026-10-14|`nix:T6` added `src/lib.rs`, `src/main.rs` & `dev/src/*.rs` w/ ⊥ `test:` commit before them ∵ scaffold had ⊥ behaviour to pin (empty `fn main`, ⊥ re-exports)|exempted in `nix/guard/tdd-order.exempt` by full id; V7 unchanged
