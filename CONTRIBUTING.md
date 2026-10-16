# Contributing

Thanks for looking. Bug reports with a machine's `cgmr status --json` are
the most useful thing you can send; a reading that is wrong on your
hardware is exactly what the fixtures and the real-machine suite exist to
catch.

## Prerequisites

- [Nix](https://nixos.org/download/) with flakes enabled.
- [direnv](https://direnv.net/) (recommended).

## Setup

```sh
git clone https://github.com/pr0d1r2/congestimancer.git
cd congestimancer
direnv allow   # or: nix develop
```

The dev shell pins every tool the gate runs: the Rust toolchain, hk,
clippy, cargo-llvm-cov, cargo-deny, the spec and character checkers, and
the linters. Entering it points git at the tracked hooks in `.githooks/`;
they refuse to run without `hk`, so commit from inside the shell.

## The gate

One definition, [`hk.pkl`](hk.pkl), runs on every commit (fast checks),
on every push (tests, coverage, packaging, benchmarks) and in CI on Linux
and macOS. [AGENTS.md](AGENTS.md#the-gate) lists every step, generated
from that file.

```sh
hk check --all   # everything a push runs
hk fix --all     # apply the fixers (formatting, generated tables)
cargo test --workspace
cargo test -- --ignored   # the real-machine suite: plausible readings here
```

Heavy push steps run under the repository's own `cgmr`. To skip that for
one run, set `CGMR_DOGFOOD=0`.

## How changes are made

- **The spec is the law.** [`SPEC.md`](SPEC.md) federates into one
  `SPEC.md` per directory. A behaviour change starts as an invariant (`V`
  row) and a task (`T` row) in the node it belongs to; a bug found later
  gets a `B` row naming the invariant that now catches it.
- **Test first.** A `test(scope): ...` commit with the failing test comes
  before the commit that makes it pass. The `tdd-order` step checks the
  order on push.
- **Unit tests sit beside their module**, in `<module>/tests.rs`, wired by
  `#[cfg(test)] mod tests;`. Integration tests that run the binary live in
  `tests/`.
- **One topic per commit.** Subjects are `type(scope): claim`, at most 72
  characters, with a body that says why. A `feat`, `fix` or `perf` change
  under `src/` adds a line under `## [Unreleased]` in
  [CHANGELOG.md](CHANGELOG.md).
- **No `unsafe` outside `src/sys`**, and every `unsafe` block there has a
  `// SAFETY:` comment.
- **No new dependency** without a row in
  [docs/THIRD-PARTY-NOTICES.md](docs/THIRD-PARTY-NOTICES.md) saying why it
  is not hand-written and which node owns it.

## Adding a gate

1. Describe it in [docs/CONGESTION.md](docs/CONGESTION.md): switch,
   semantics, source per OS, and the support matrix row.
2. Add the invariant and task to the domain node's `SPEC.md` that owns the
   signal; the `spec-parity` step fails until the switch is named there.
3. Add a fixture under `tests/fixtures/<node>/`, captured from a real
   machine where possible (`cgmr debug capture <dir>`).
4. RED, then GREEN: the reader, the switch table row in `src/cli`, and a
   case in `tests/cli/parity.rs`.

## Reporting a security problem

Use a private security advisory on GitHub, not a public issue. See
[docs/SECURITY.md](docs/SECURITY.md) for what cgmr defends.
