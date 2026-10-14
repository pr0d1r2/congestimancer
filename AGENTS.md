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
