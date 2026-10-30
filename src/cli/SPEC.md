# SPEC

## §G GOAL

`src/cli` — argv → verb & config; config → exit code.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page
sib|src/units|parse & render sizes, rates, percents, durations, temperatures, clock windows
sib|src/signal|`Signal` & `Source` traits, `Reading`, sampler w/ two-sample deltas, monotonic clock, fixture root
sib|src/sys|OS facade: ∀ syscalls, FFI, `sysinfo`, `rustix`, `libc`; only node allowed `unsafe`
sib|src/cpu|load, run queue, CPU idle, CPU pressure, steal
sib|src/mem|available memory, memory pressure & level, swap rate, dirty pages
sib|src/disk|free space & inodes on a path
sib|src/io|IO throughput, busy, latency, queue, pressure; path → device resolution
sib|src/net|interface rates, RTT probe
sib|src/limits|open files & processes vs system maximum
sib|src/power|thermal throttling, temperature, AC, battery, low power
sib|src/presence|user idle time, clock window
sib|src/gpu|GPU utilisation & free memory
sib|src/procs|process counts by name & by user
sib|src/cgroup|cgroup v2 detection, limits, throttling, pressure override
sib|src/gate|gate model, strict compare, AND evaluation, unsupported verdict, custom gates
sib|src/wait|poll loop: interval, backoff, timeout, settle, jitter, heartbeat
sib|src/slot|per-user lock dir, slots, FIFO, rate, spacing, memory reservations, nested held marker
sib|src/launch|exec, no-cmd exit, govern, background priority, env cleanup
sib|src/remote|`--host` evaluation over ssh
sib|src/report|`status` human & JSON, verbose reasons, heartbeat text, colour
sib|src/profile|named switch sets from project & user files, lookup & trust rule
sib|src/advise|`advise` verb: how many jobs fit now, what limits them, what to do

## §C CONSTRAINTS

- facade ∀ `clap` (derive) & `clap_complete`, `clap_mangen`. deps: ∀ other nodes via public surface.

## §I INTERFACES

- verbs: `run` (default) · `check` · `status [--json]` · `advise [--jobs J] [--json]` · `profiles` · `completions <shell>` · `man`.
- `cgmr [run] [gates] [-- cmd...]`; cmd only after `--`.
- ∀ switch ∈ `docs/CONGESTION.md` w/ env `CGMR_<SWITCH>` (upper snake), except `--when`.
- hidden: `CGMR_FIXTURE_ROOT` (tests), `CGMR_DOGFOOD`, verb `debug capture <dir>` (fixture capture, `src/sys:V7`).

## §V INVARIANTS

V1: verb omitted → `run`.
V2: precedence: flag > env > profile > builtin default.
V3: `--when` ⊥ read from env.
V4: exit: cmd's own (exec) · 0 released & no cmd · `check`: 0 pass, 1 fail · 64 usage (clap errors, `UnitError`) · 69 unsupported · 124 timeout.
V5: exit-code collision (cmd itself exits 124|64|69) documented in `--help` & `docs/CLI.md`.
V6: `--help` shows examples ∀ verb.
V7: switch names & env names generated from one table ∴ `docs/CONGESTION.md` parity testable (`.:V18`).
V8: `CGMR_FIXTURE_ROOT` read once in `main`, passed down as `Root`; ⊥ `set_var`.
V9: single-value switch given twice → usage error 64; only `--max-procs`, `--max-user-procs` repeat.
V10: env formats: value switch = same text as CLI · boolean switch = `1|true|yes` on, `0|false|no` off, else 64 · repeatable switch = comma-separated items (`CGMR_MAX_PROCS="rustc*=4,cargo=2"`).
V11: unknown `CGMR_*` var (not in switch table & not `CGMR_DOGFOOD`, `CGMR_FIXTURE_ROOT`, `CGMR_HELD_SLOTS`, `CGMR_PROFILE`, `CGMR_PROFILE_FILE`) → warning on stderr naming nearest known var; ⊥ exit.
V12: `--quiet` suppresses warnings (⊥ errors, ⊥ `status` output); `--version` prints crate version.
V13: `--needs-mem auto[:X]` settles before any gate reads it: key = `--mem-key` else slot key else cmd; learned → peak + 1/10, else X (default 1GiB). A key → cmd spawned & watched by `PeakMeter` (`src/launch:V10`, `src/slot:V13`); `check` & `advise` only size it.

## §T TASKS

id|status|task|cites
T1|x|clap derive skeleton, verbs, default `run`|V1
T2|x|switch table + env names|V2,V3,V7
T3|x|exit code mapping|V4
T4|x|help text w/ examples & collision note|V5,V6
T5|x|`completions` & `man` verbs|V6
T6|x|fixture root & dogfood plumbing|V8
T7|x|repeat rule|V9
T8|x|env value formats|V10
T9|x|unknown env warning w/ nearest-name hint|V11
T10|x|`--quiet` & `--version`|V12
T11|x|`--needs-mem auto` settle, `--mem-key`, `PeakMeter`, learned keys in `status`|V13

## §B BUGS

id|date|cause|fix
