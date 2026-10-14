# SPEC

## §G GOAL

`src/mem` — memory capacity & pressure signals & their gates.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/mem|available memory, memory pressure & level, swap rate, dirty pages
sib|src/units|parse & render sizes, rates, percents, durations, temperatures, clock windows
sib|src/signal|`Signal` & `Source` traits, `Reading`, sampler w/ two-sample deltas, monotonic clock, fixture root
sib|src/sys|OS facade: ∀ syscalls, FFI, `sysinfo`, `rustix`, `libc`; only node allowed `unsafe`
sib|src/cpu|load, run queue, CPU idle, CPU pressure, steal
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
sib|src/advise|`advise` verb: how many jobs fit now, what limits them, what to do
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- deps: `src/units`, `src/signal` (PSI parser), `src/sys`, `src/cgroup` (scope). reservation ledger ∈ `src/slot`.

## §I INTERFACES

- switches: `--mem X` · `--all-mem X` · `--needs-mem X` · `--mem-pressure X` · `--mem-pressure-full X` · `--mem-level normal|warning` · `--swap-rate X/s` · `--dirty X`.
- sources: Linux sysinfo (`MemAvailable`), `/proc/pressure/memory`, `/proc/vmstat` (`pswpin` `pswpout`), `/proc/meminfo` (`Dirty` `Writeback`) · macOS sysinfo, `kern.memorystatus_vm_pressure_level`, `kern.memorystatus_level`, `host_statistics64` swapins/swapouts.

## §V INVARIANTS

V1: capacity = AVAILABLE memory, ⊥ "free" ∵ page cache & compressor mislead.
V2: `--mem X` ⇔ available / N > X; `--all-mem X` ⇔ available > X.
V3: `--needs-mem X` ⇔ available − reserved by running cgmr jobs > X; reserved total from `src/slot`.
V4: `--mem-pressure X` ⇔ PSI `some avg10` < X; `--mem-pressure-full X` ⇔ `full avg10` < X. Linux only.
V5: `--mem-level normal` ⇔ level = 1; `--mem-level warning` ⇔ level ∈ {1,2}. macOS only; Linux `Unsupported`.
V6: `--swap-rate X/s` ⇔ (swap in + swap out) bytes/s < X; page counts × page size, checked.
V7: `--dirty X` ⇔ `Dirty` + `Writeback` < X. Linux only.
V8: `status` on macOS also shows `kern.memorystatus_level` %.

## §T TASKS

id|status|task|cites
T1|.|available memory both OS + `--mem` `--all-mem`|V1,V2
T2|.|`--needs-mem` using reservation total|V3
T3|.|`--mem-pressure` & `--mem-pressure-full`|V4
T4|.|macOS pressure level + `--mem-level`|V5
T5|.|`/proc/vmstat` parser + Linux swap rate|V6
T6|.|macOS swap rate from `host_statistics64`|V6
T7|.|`/proc/meminfo` parser + `--dirty`|V7
T8|.|`kern.memorystatus_level` reading for status|V8

## §B BUGS

id|date|cause|fix
