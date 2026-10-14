# SPEC

## §G GOAL

`src/units` — parse & render every value a switch takes.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/units|parse & render sizes, rates, percents, durations, temperatures, clock windows
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
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- zero deps. pure fns. ⊥ I/O, ⊥ locale.

## §I INTERFACES

- types: `Name` (safe identifier), `Bytes(u64)`, `Rate` (bytes/s), `Percent` (0..=100), `Duration` (std), `Celsius`, `ClockWindow`, `Count(u64)`.
- fns: `parse_<type>(&str) -> Result<T, UnitError>` & `Display` ∀ type.

## §V INVARIANTS

V1: `B` = 1. `KB` `MB` `GB` `TB` = 1000^n. `KiB` `MiB` `GiB` `TiB` = 1024^n. suffix case-insensitive; `KB` ≠ `KiB`. ⊥ bits.
V2: size number ? fractional (`1.5GB`); result rounds down to whole bytes.
V3: rate = size + `/s`; ⊥ other time base.
V4: percent = number + optional `%`; ∉ 0..=100 → error.
V5: duration = 1+ `<n><unit>` pairs, unit ∈ `ms s m h d`, e.g. `1h30m`; bare number → error. ⊥ `humantime`.
V6: temperature = number + `C`.
V7: clock window `HH:MM-HH:MM`, local time; start > end ∴ wraps midnight; start = end → error.
V8: decimal separator `.` only, locale ignored.
V9: overflow → error, ⊥ wrap, ⊥ saturate.
V10: ∀ parser never panics on any input (proptest).
V11: `parse(render(x)) == x` ∀ type (proptest).
V12: error names input & expected form → cli maps to exit 64.
V13: `Name` = 1..=64 chars of `[A-Za-z0-9._-]`; ⊥ leading `-`, ⊥ `.` | `..` alone ∴ safe as filename, path segment & argv word. else → error (64).
V14: thresholds ⊥ negative; load-like values accept decimals (`--load 0.5`); count-like values whole numbers; violation → 64.

## §T TASKS

id|status|task|cites
T1|x|`Bytes` parse decimal & binary suffixes|V1,V2,V9
T2|x|`Rate` parse|V3
T3|x|`Percent` parse|V4
T4|x|`Duration` parse, own grammar|V5,V9
T5|x|`Celsius` parse|V6
T6|x|`ClockWindow` parse & contains(time), midnight wrap|V7
T7|x|`Count` parse|V9
T8|x|locale-independent number lexer|V8
T9|x|render ∀ type + round-trip proptest|V11
T10|.|no-panic proptest ∀ parser|V10
T11|x|`UnitError` w/ input & expected form|V12
T12|x|`Name` parse & reject list|V13
T13|x|threshold range checks|V14

## §B BUGS

id|date|cause|fix
