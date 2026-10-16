# Agents

Instructions for coding agents working in this repository.

## Read first

- `SPEC.md` is the law. It federates into one `SPEC.md` per directory
  (`src/*`, `tests`, `docs`, `dev`, `nix`); load the chain for the node you
  work on (`sherd lens <dir>`).
- `docs/CONGESTION.md` explains every congestion method for humans.

## Working rules

- Work in TDD: a failing test first, then the code, then refactor.
- One topic per commit; commit through the hooks (never `--no-verify`).
- Commit subjects are `type(scope): claim` (at most 72 characters) with a
  body that says why. A `feat`, `fix` or `perf` change to `src/` adds an
  entry under `## [Unreleased]` in `CHANGELOG.md`.
- Enter the dev shell (`direnv allow` or `nix develop`) before committing:
  the hooks refuse without `hk`.
- Spec edits go through `/spec`; `/build` only flips task status.

## Wave builds (nix:V27)

The `src` nodes are built in rounds that `sherd wave` schedules from the
`use crate::` edges in each `src/<node>/mod.rs`.

1. Spike round first: every open `§R` row's SPIKE task (src:V6).
2. For each round, ask the machine for a budget:
   `nix/guard/workers.sh <nodes in round>` prints `workers=N jobs=J`.
   Start at most N workers; each sets `CARGO_BUILD_JOBS=J`.
3. One worker per node, in its own git worktree. A worker loads only its
   node's chain (`sherd lens src/<node>`), works only in `src/<node>/`
   (plus its fixtures under `tests/fixtures/<node>/`), and runs RED, GREEN,
   REFACTOR per task, committing through the hooks.
4. When a round is done, merge its branches in wave order and run
   `hk check --all` on the result. A node that fails is reverted and
   logged; the round still lands.
5. Ask for a new budget before every round; never start a round on a
   stale one.

`sherd tdd` and its Ollama endpoint are not used to write code.

## The gate

<!-- BEGIN steps: generated from hk.pkl by `cargo run -p congestimancer-dev -- agents`; do not edit -->
45 steps run on every commit and 54 on push and `hk check`; the commit-msg hook checks the message.

| step | runs on | files | fixes |
| --- | --- | --- | --- |
| `trailing-whitespace` | commit | `**/*` | yes |
| `final-newline` | commit | `**/*` | yes |
| `line-endings` | commit | `**/*` | yes |
| `no-bom` | commit | `**/*` | - |
| `no-merge-conflict` | commit | `**/*` | - |
| `no-case-conflict` | commit | `**/*` | - |
| `no-broken-symlinks` | commit | whole tree | - |
| `no-private-key` | commit | `**/*` | - |
| `ripsecrets` | commit | `**/*` | - |
| `no-local-paths` | commit | `**/*` | - |
| `nixfmt` | commit | `**/*.nix` | yes |
| `shfmt` | commit | `**/*.sh` `**/*.bash` | yes |
| `taplo` | commit | `**/*.toml` | yes |
| `rustfmt` | commit | `**/*.rs` | yes |
| `typos` | commit | `**/*` | - |
| `shellcheck` | commit | `**/*.sh` `**/*.bash` `.envrc` | - |
| `clippy` | commit | `**/*.rs` `Cargo.toml` `clippy.toml` | - |
| `actionlint` | commit | `.github/workflows/*.yml` `.github/workflows/*.yaml` | - |
| `zizmor` | commit | `.github/workflows/*.yml` `.github/workflows/*.yaml` | - |
| `statix` | commit | `**/*.nix` | yes |
| `deadnix` | commit | `**/*.nix` | yes |
| `markdownlint` | commit | `**/*.md` | yes |
| `yamllint` | commit | `**/*.yml` `**/*.yaml` | - |
| `gawk-lint` | commit | `**/*.awk` | - |
| `editorconfig-checker` | commit | `**/*` | - |
| `lychee` | commit | `**/*.md` | - |
| `ctrm` | commit | `**/*` | yes |
| `cgmr-profiles` | commit | `.cgmr` `nix/guard/cgmr.sh` | - |
| `rekall` | commit | `AGENTS.md` `CLAUDE.md` `rekall.toml` | - |
| `rust-mirror` | commit | `**/*.rs` `nix/guard/rust-mirror.sh` | - |
| `module-size` | commit | `**/*.rs` `nix/guard/module-size.sh` | - |
| `unsafe-scope` | commit | `**/*.rs` `nix/guard/unsafe-scope.sh` | - |
| `cargo-deny` | commit | `Cargo.toml` `Cargo.lock` `deny.toml` `dev/Cargo.toml` | - |
| `mth-fmt` | commit | `**/SPEC.md` | yes |
| `mth-check` | commit | `**/SPEC.md` | - |
| `sherd-check` | commit | `**/SPEC.md` `**/*.rs` | - |
| `sherd-nav` | commit | `**/SPEC.md` | yes |
| `sherd-budget` | commit | `**/SPEC.md` `.context-limits` | - |
| `itok` | commit | `**/*` `.context-limits` | - |
| `spec-records` | commit | `SPEC.md` `.spec-records` | - |
| `ceiling-cap` | commit | `.context-limits` `nix/ceiling-cap.awk` | - |
| `readme-badges` | commit | `README.md` `Cargo.toml` `.coverage` `hk.pkl` `hk.pklith.pkl` `flake.lock` `**/SPEC.md` `.github/workflows/ci.yml` `dev/**` | yes |
| `notices` | commit | `Cargo.toml` `Cargo.lock` `docs/THIRD-PARTY-NOTICES.md` `dev/**` | yes |
| `agents-table` | commit | `hk.pkl` `hk.pklith.pkl` `AGENTS.md` `dev/**` | yes |
| `gen-check` | commit | `.pklith` `hk.pklith.pkl` `nix/pklith.nix` | yes |
| `cargo-test` | push | `**/*.rs` `Cargo.toml` `Cargo.lock` `dev/Cargo.toml` | - |
| `rustdoc` | push | `**/*.rs` `Cargo.toml` | - |
| `package` | push | `**/*.rs` `Cargo.toml` `Cargo.lock` `.cgmr` `nix/guard/package.sh` `nix/guard/cgmr.sh` | - |
| `no-default-features` | push | `**/*.rs` `Cargo.toml` | - |
| `semver` | push | `**/*.rs` `Cargo.toml` `nix/guard/semver.sh` | - |
| `tdd-order` | push | `**/*.rs` `nix/guard/tdd-order.sh` `nix/guard/tdd-order.exempt` | - |
| `coverage` | push | `**/*.rs` `.coverage` `.cgmr` `nix/guard/coverage.sh` `nix/guard/cgmr.sh` | yes |
| `bench` | push | `**/*.rs` `Cargo.toml` `Cargo.lock` `nix/guard/bench.sh` | - |
| `pklith-check` | push | `**/*` | - |
<!-- END steps -->
