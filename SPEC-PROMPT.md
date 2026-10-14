congestimancer

Tool for load awareness in the system that allow to calculate machine congestion from:
- unix load
- memory pressure
- io load to advise
- potentially more

To:
- pause or perform other operations
- create mechanical workflows based on resource availability
- advise agents on actions.

Write in Rust.
Use atomic git commits per topic for human readability.

We should have nix flake dev shell loaded with direnv.
When we have basic shell - ask me to reload and continue session.

Guardrails is a first thing and we should:
- use pklith and nix-hk
- use sherd for SPEC-DAG federation and to do sherd wave builds in parallel subagents
- use microlith for spec operations
- use itok for context limiting
- use characterminator to keep characters at bays
Inspire all scrutinized checks from other our Rust repos which apply.
We can dogfood this project in guardrails as well.
Use 1-to-1 unit coverage with floor and ceiling aspiring towards 100%.
Work in TDD: RED, GREEN, REFACTOR cycles.
Guardrails should be installed when entering dev shell.
When we have guardrails ready - ask me to reload and continue session.

# cgmr CLI with swiches:

cgmr --load 2 -- echo example command # wait for load to be below two per logical CPU thread
cgmr --all-load 10 -- echo example command # wait for total system load to be below 10

cgmr --mem 2GB -- echo example command # wait for have at least 2GB free memory per processing thread
cgmr --all-mem 10GB -- echo example command # wait for system to have at least 10GB free memory

cgmr --disk 5GB -- echo example command # wait to have 5GB disk space in the working directory

cgmr --io-read 1MB/s -- echo example command # wait for IO reads to be below 1MB/s
cgmr --io-write 1GB/s -- echo example command # wait for IO writes to be below 1GB/s
cgmr --io 100MB/s -- echo example command # wait for total IO reads+writes to be below 100MB/s

Switch can be mixed together as and:
cgmr --load 1 --mem 1GB --disk 2GB --io-read 100MB/s --io-write 100MB/s -- example command

# Naming

- Project / crate name: `congestimancer` ("one who divines congestion"; hides "estim(ate)" pun).
- Binary name: `cgmr`.
- Availability checked 2026-10-14: `congestimancer` and `cgmr` free on crates.io, npm, PyPI,
  RubyGems, Homebrew core; no GitHub user/org `congestimancer`; no local binary/alias clash.
- Rejected binary alternatives:
  - `cong`: possible, but in US English it is a war-era slur for the Viet Cong.
  - `la`: taken on most registries and clashes with common `alias la='ls -lAh'`.
  - `cgm`: taken on PyPI; also means "continuous glucose monitor".
  - `gest`: taken on crates.io, npm and PyPI.
- Claim the crate name early with a 0.0.0 placeholder publish (needs explicit approval; permanent).

# Congestion methods

Read `docs/CONGESTION.md` together with this file. It is the authoritative
catalogue of congestion signals, gate switches, comparison rules, units,
waiting and release strategies, per-platform sources and the platform
support matrix. Base all verbs and switches on it. Every method in it is in
scope for building; nothing is deferred.

# Platform policy

- Targets: Linux and macOS (darwin).
- Define each gate per platform and map it to the native signal (e.g. PSI on Linux, pressure
  level / derived stall metrics on macOS).
- If a gate has no native signal on the current OS, fail with a clear "unsupported on this OS"
  error. Never pass a gate silently.

# Prior art

- GNU `parallel` `--load`, `--memfree`; `sem` (slots).
- `nice`, `batch` (at).
- Differentiator: standalone gate with disk, IO and pressure checks, plus agent-friendly output.

# Decisions (2026-10-14)

1. Units: accept both decimal (`KB`, `MB`, `GB`, `TB` = powers of 1000) and binary (`KiB`,
   `MiB`, `GiB`, `TiB` = powers of 1024); each means exactly what it says. Rates add `/s`.
2. `--load` uses the 1-minute load average by default; `--load-window 1m|5m|15m` overrides.
3. IO rates (`--io-read`, `--io-write`, `--io`) are averaged over the poll interval.
4. cgmr `exec`s the command once gates pass (no parent process left): exit code and signals
   pass through naturally. `--slots` locks use `flock` on an open file descriptor without
   close-on-exec, so the lock is held by the command and released when it exits.
5. Exit codes: the command's own code once it runs; reserved codes otherwise:
   124 = `--timeout` reached (like `timeout(1)`), 64 = bad arguments (EX_USAGE),
   69 = gate unsupported on this OS (EX_UNAVAILABLE).
6. Workspace split: `congestimancer` library crate (gates, metric readers, parsing) plus the
   `cgmr` binary as a thin CLI over it, so agents and other tools can reuse the library.
7. Testability invariant: every metric source sits behind a trait; tests use fakes (fake
   `/proc` / `sysctl` inputs), never the live machine. Required for 1-to-1 unit coverage.
8. Metric sources: one trait method per signal, exactly one source per signal per platform.
   Never mix sources within one signal. Keep `sysinfo` adapters thin; custom readers are pure
   parsers (text in, numbers out) unit-tested 1-to-1 with fixture files.

   | Signal | Source |
   |---|---|
   | Load average | `sysinfo` (or `libc::getloadavg`) |
   | Logical CPU count | `std::thread::available_parallelism` (respects cgroup CPU limits and affinity) |
   | Free / available memory | `sysinfo` |
   | Disk free for a path | `statvfs` on the path (`rustix` or `nix`); also gives free inodes |
   | Disk IO rates | `sysinfo` per-disk IO stats if verified on both OSes; otherwise `/proc/diskstats` (Linux) and IOKit (macOS) |
   | PSI cpu / memory / io | custom reader for `/proc/pressure/*` (Linux) |
   | macOS memory pressure | custom `sysctl` reader for `kern.memorystatus_*` |
   | cgroup limits | custom reader for `/sys/fs/cgroup` (Linux) |

   First task before building IO gates: spike test proving `sysinfo` disk IO stats work on
   Linux and macOS; record the outcome in the spec.
9. License: MIT.
10. MSRV and toolchain: same as the other Rust projects (`rust-version = "1.95"`).
11. Platforms: the same 4 systems as the other Rust projects' flakes: `aarch64-darwin`,
    `x86_64-darwin`, `x86_64-linux`, `aarch64-linux`. Windows and BSD are non-goals.
12. Agent interface for v1: `--check` and `--json` only. MCP server and daemon mode are
    non-goals for v1.
13. Dogfood always: the project's own guardrails and heavy hooks (tests, coverage, builds)
    run through `cgmr` as soon as it can gate them.
14. Claim the crate name on crates.io with the first real pre-release, `0.1.0-rc.1`
    (crates.io requires SemVer; `0.1.0.rc1` is not valid). Publishing needs explicit approval.

# Decisions, round 2 (2026-10-14)

15. CLI has verbs; `run` is the default verb:
    - `cgmr [run] [gates] [-- command...]`: wait for gates, then exec the command.
    - `cgmr check [gates]`: evaluate gates once; exit 0 if all pass, 1 if not; never waits.
    - `cgmr status [--json]`: print current readings for every supported signal (for agents).
    The command always follows `--`, so it never collides with a verb name.
16. Without a command, `cgmr [run] [gates]` waits until gates pass and exits 0. Combined with
    `--timeout`, it exits 124 when the timeout is reached first. Enables `cgmr --load 2 && make`.
17. Comparisons are strict, and a gate passes only when the system is strictly better than the
    threshold. With N = logical CPU count (decision 8):
    - `--load X`: load / N < X; `--all-load X`: load < X.
    - `--mem X`: available / N > X; `--all-mem X`: available > X.
    - `--disk X`: free space on the path > X.
    - `--io-read X`, `--io-write X`, `--io X`: measured rate < X.
    - `--*-pressure X`: PSI `some avg10` < X (percent) on Linux; macOS maps to its native
      signal per the platform policy (e.g. pressure level must be normal).
18. Fairness (FIFO queue among waiters) is a non-goal for v1; possible later as FIFO slots.
19. Every gate and control switch has an environment variable default (`CGMR_LOAD`,
    `CGMR_MEM`, `CGMR_TIMEOUT`, ...). Precedence: CLI flag > env var > built-in default.
20. Distribution: `cargo install congestimancer` and a nix flake package (`nix build`,
    `nix run`). No Homebrew tap or prebuilt binaries for now.
21. stdout belongs to the command. cgmr writes diagnostics only to stderr; the only exception is
    `status` / `--json` output, which goes to stdout.
22. JSON output has a `schema_version` field and stable field names; breaking changes bump it.
23. SIGINT / SIGTERM while waiting: exit 130 / 143; the command never runs.
24. Exit code collision: if the command itself exits 124, 64 or 69, the caller cannot tell it
    apart from cgmr's own code (same as `timeout(1)`). Document this; `--verbose` and `--json`
    show the source.
25. cgmr must be cheap: 100 waiting instances must not add measurable load. Sleep between
    polls, refresh only the metrics the requested gates need, never busy-loop.
26. Time: monotonic clock for `--settle`, `--timeout`, `--interval` and IO rate windows.
    Durations parsed with `humantime` (`10s`, `1h`, `1h30m`).
27. Slot lock directory: per user, mode 0700, in `$XDG_RUNTIME_DIR` if set, otherwise
    `$TMPDIR` (macOS) or a per-user directory under `/tmp`. Never share locks between users.
28. Non-standard working directories:
    - tmpfs, NFS, FUSE and other mounts without a block device: `--io-*` gates fail with the
      "unsupported" error (exit 69) and suggest `--io-device`.
    - macOS APFS: several volumes share one physical disk; measure IO at the physical disk.
29. Containers are supported (cgroup v2): inside a cgroup, use the cgroup's own limits and
    pressure files (`cpu.max`, `cpu.stat`, `cpu.pressure`, `memory.max`, `memory.current`,
    `memory.pressure`, `io.pressure`) instead of host-wide `/proc` values. N for per-CPU gates
    comes from `available_parallelism` (respects `cpu.max`). Host load average divided by the
    container's CPU count is wrong, so inside a cgroup `--load` uses cgroup CPU pressure or
    warns. cgroup v1 is a non-goal unless it turns out to be needed.
30. Testing on real machines: besides fixture-based unit tests and CLI tests (`assert_cmd`)
    with a hidden fixture-root env var (`CGMR_FIXTURE_ROOT`), run real-machine tests on the
    local network hosts:
    - `nix-builder.local` (Linux)
    - `hallucinogen.local` (Linux)
    - `unicomplex.local` (Linux, ThinkPad T440p, x86_64)
    - this development machine: M4 Mac (covers `aarch64-darwin`)
    - the Intel Mac (covers `x86_64-darwin`)
31. CLI tooling: `clap` (derive), shell completions with `clap_complete`, man page with
    `clap_mangen`, `--help` with examples.
32. CI runs on 3 platforms: `x86_64-linux` (ubuntu-latest), `aarch64-linux`
    (ubuntu-24.04-arm), `aarch64-darwin` (macos-latest). `x86_64-darwin` stays a supported
    target, verified manually on the Intel Mac (no CI runner).
