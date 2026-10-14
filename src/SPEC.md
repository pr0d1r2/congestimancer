# SPEC

## §G GOAL

`src/` — hub. code nodes of `congestimancer`. ⊥ own logic itself.

## §F FEDERATION

dir|owns|⊥owns|tokens
units|parse & render sizes, rates, percents, durations, temperatures, clock windows|what any value means for a gate|-
signal|`Signal` & `Source` traits, `Reading`, sampler w/ two-sample deltas, monotonic clock, fixture root|any concrete OS source|-
sys|OS facade: ∀ syscalls, FFI, `sysinfo`, `rustix`, `libc`; only node allowed `unsafe`|parsing text, gate logic|-
cpu|load, run queue, CPU idle, CPU pressure, steal|cgroup CPU limits|-
mem|available memory, memory pressure & level, swap rate, dirty pages|reservations ledger|-
disk|free space & inodes on a path|IO rates|-
io|IO throughput, busy, latency, queue, pressure; path → device resolution|free space|-
net|interface rates, RTT probe|remote gate evaluation|-
limits|open files & processes vs system maximum|per-name process counts|-
power|thermal throttling, temperature, AC, battery, low power|CPU load|-
presence|user idle time, clock window|thermal & power|-
gpu|GPU utilisation & free memory|CPU|-
procs|process counts by name & by user|system limits|-
cgroup|cgroup v2 detection, limits, throttling, pressure override|host-wide sources|-
gate|gate model, strict compare, AND evaluation, unsupported verdict, custom gates|waiting, slots|-
wait|poll loop: interval, backoff, timeout, settle, jitter, heartbeat|gate semantics, slots|-
slot|per-user lock dir, slots, FIFO, rate, spacing, memory reservations, nested held marker|gate semantics|-
launch|exec, no-cmd exit, govern, background priority, env cleanup|waiting|-
remote|`--host` evaluation over ssh|local gates|-
report|`status` human & JSON, verbose reasons, heartbeat text, colour|what is measured|-
cli|clap verbs, flags, env defaults, exit codes, completions, man page|every verb's logic|-

## §N NAV

rel|path|lens
up|.|-
self|src|the tool: congestion domains, gates, waiting, launch, CLI
sib|tests|integration tests driving `cgmr` binary, dogfood tests, fixtures, real-machine tests
sib|docs|human reference: `CONGESTION.md`, CLI guide, LLM disclaimer, third-party notices
sib|dev|`congestimancer-dev`, `publish = false`: README badges, notices closure, AGENTS step table & their drift checks
sib|nix|repo gate: dev shell, guardrails, hk steps, spec & token gates, CI, release, real-machine runs

## §C CONSTRAINTS

- module = dir + `mod.rs`. `mod.rs` composes, ⊥ implements.
- ∀ external dep ! ONE facade node: `sysinfo`, `rustix`, `libc` → `src/sys` · `clap` → `src/cli`.
- dep order (waves): `units` → `signal` → `sys` → `cgroup` → domains (`cpu mem disk io net limits power presence gpu procs`) → `gate` → `report` → `wait`, `slot`, `remote` → `launch` → `cli`.
- domain node = pure parsers (text|bytes in, numbers out) + mapping to `Signal`; ∀ OS access via `src/sys`.

## §V INVARIANTS

V1: `main.rs` & `lib.rs` dispatch & re-export only, ⊥ logic.
V2: node ⊥ dep sibling node except through its public surface.
V3: lib API = `lib.rs` re-exports ONLY, ∀ node `mod` private; exported error enums & growable structs `#[non_exhaustive]`.
V4: ∀ domain parser unit-tested w/ fixture files `tests/fixtures/<node>/`; ⊥ live machine in unit tests.
V5: ∀ gate in a domain node: one source per platform; platform w/o source → `Reading::Unsupported` → exit 69 (`src/gate`).
V6: ∀ node w/ open §R row: its SPIKE task runs FIRST (round 0 of build); finding & chosen source recorded in §R before any task citing that area starts; ⊥ guessing APIs.
