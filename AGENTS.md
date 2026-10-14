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
