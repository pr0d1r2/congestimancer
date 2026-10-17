# Built by an LLM, deliberately and in the open

<!-- hallucinogen:tending-disclaimer start -->
**Tended by an autonomous loop running Codex with GPT-5.6 luna at low
reasoning effort.** When Codex is unavailable the loop falls back to Claude,
then to a local model. The loop opens pull requests, reviews them itself
and merges them once they are green, without a human reading the diff: the
merge gate is this repository's own checks plus that automated review, not
human approval. On top of that, the maintainer runs periodic meta-reviews
with agents and corrects drift or bugs they find.

What keeps that checkable is mechanical rather than a matter of trust:
the loop generates tests and linter configurations, so its gates give
the same answer every time, and it builds command-line tools, with clear
documentation, that a person can use to inspect the same state the loop sees.

Some classes of change are held for a human by design: releases, anything
touching the loop's own safety rails, and anything that could publish to
a package registry. Everything else is not.

**Origin.** This project was originally written with LLM assistance,
with a human reviewing every change. The text below records that period:
what it says about human review holds for the code written then, not for
the merges the loop makes now.

---
<!-- hallucinogen:tending-disclaimer end -->

This repository (code, spec, tests and prose) was written by
[Claude Code](https://claude.com/claude-code) running Anthropic's **Claude
Opus 5.5**. Its commits carry a `Co-Authored-By: Claude Opus 5.5` trailer,
except a handful of the earliest; `git log` shows which. A human owns every
decision, reviews every diff, and is accountable for what ships.

That is the disclaimer. The rest of this file is why it is a design note
rather than an apology, and what you can check for yourself.

## Why say it at all

An LLM writes plausible code, and plausible is not correct. A reader who
does not know how a repository was produced cannot calibrate how hard to
look at it.

cgmr makes the point sharper. It reads kernel counters on two operating
systems and decides when other people's builds may start; a wrong reading
either stalls work or lets a machine drown. So the standard here is: **a
claim in prose is a defect until something executes it.**

## The method is spec-driven development

[`SPEC.md`](../SPEC.md) and one `SPEC.md` per node under `src/`, `tests/`,
`docs/`, `dev/` and `nix/` are the law, not a description written
afterwards. They carry invariants (what must stay true, with the
reasoning), tasks (what is decided and what is not), bugs (each paired with
the invariant that now catches it) and research (the measurements the
constraints rest on). The README badges count the invariants, bugs and
nodes, generated from the specs themselves.

The spec's format is checked mechanically by
[`microlith`](https://github.com/pr0d1r2/microlith) and its federation by
[`sherd`](https://github.com/pr0d1r2/sherd), so "the spec says so" is a
statement about a file that was parsed, not one that was skimmed. Every
congestion method a human can read about in
[`CONGESTION.md`](CONGESTION.md) has its own node invariant, and a test
fails when the two disagree.

## The guardrails are git hooks that also run on CI

Entering the dev shell (`nix develop`, or `direnv allow`) points git at the
tracked hooks, which run [hk](https://github.com/jdx/hk) against one
definition of the gate in [`hk.pkl`](../hk.pkl); the table in
[`AGENTS.md`](../AGENTS.md) lists every step and is generated from it. The
push half adds line coverage against a floor and a claim, the packaged
crate built from its own tarball, the build without default features, the
test-before-code order of the commits, and a benchmark of cgmr's own CPU,
memory and start-up cost. Heavy steps run under cgmr itself.
[`ci.yml`](../.github/workflows/ci.yml) calls that same definition on
Linux (x86_64 and arm) and macOS, so a laptop and a runner cannot disagree.
`unsafe` is denied everywhere except the OS facade, `src/sys`.

## The record is deliberately unflattering

Defects that were green before they were found. Each names the commit that
fixed or recorded it.

**`43cee77`: a waiting command's helpers could hold its slot.** To hand a
slot to the command, cgmr cleared close-on-exec on its own lock files, so
the `--when` shell and ssh children under `--govern` inherited the lock
and could keep a slot taken after the command had finished.

**`ec7cc54`: the process gate read a cap as a count.** On macOS the spec
named `kern.num_tasks` as the number of running tasks; it is the system's
task limit (4096), so `--pids` compared a constant.

**`fb67c89` and `e657adc`: a "closed port" test could connect.** The test
bound a port, dropped the listener and connected to it; another test
thread could bind the same port in between, so the connect succeeded and
the test flaked. The fix keeps the port held by a live socket.

**`89a237f`: a lint was put in the wrong table.** The constraints listed
`unreachable_pub` among the clippy lints; it is a rustc lint, so Cargo
rejected it. The compiler caught this one before any reviewer did.

**`nix/guard/tdd-order.exempt`: scaffolds landed without a test commit.**
The workspace scaffold and the 23 empty node modules (build-order edges
only) were committed without the `test:` commit the gate demands before
code. They are exempted by full commit id, with a bug row each, rather
than rewritten out of history.

None of these was found by reviewing a diff. Each was found by running the
thing, which is the argument for the gate, and also the argument for
reading the bug rows before trusting the invariants.

## What a reader should check

1. **Does the gate run for you?** `nix develop`, then `hk check --all`. If
   a claim here is false, that is where it shows.
2. **Do the readings hold on your machine?** `cargo test -- --ignored`
   runs the real-machine suite: every supported reading must be plausible
   for its unit.
3. **Do the bug rows look like a real log or a curated one?** The ones
   above are unedited; judge the rest.
4. **Do the invariants have runners?** An invariant that nothing executes
   is exactly the defect this project exists to remove.

## Accountability

The human named in [`LICENSE`](../LICENSE) is responsible for this code,
including the parts a model wrote and the parts nobody caught. "The LLM
wrote it" explains provenance; it never transfers responsibility.

Unflattering bug reports are the most useful kind. See
[`SECURITY.md`](SECURITY.md) for the ones that should not be public.
