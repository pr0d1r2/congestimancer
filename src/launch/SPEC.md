# SPEC

## §G GOAL

`src/launch` — start the cmd once released: exec, govern, background priority.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/launch|exec, no-cmd exit, govern, background priority, env cleanup
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
sib|src/remote|`--host` evaluation over ssh
sib|src/report|`status` human & JSON, verbose reasons, heartbeat text, colour
sib|src/profile|named switch sets from project & user files, lookup & trust rule
sib|src/advise|`advise` verb: how many jobs fit now, what limits them, what to do
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- deps: `src/signal`, `src/sys`, `src/gate`, `src/wait`, `src/slot`. ⊥ `set_var`|`remove_var` (unsafe in 2024).

## §I INTERFACES

- switches: `--govern` · `--background` · `--or-background`.

## §V INVARIANTS

V1: default: `exec` cmd (`CommandExt::exec`); exit code & signals pass through; exec failure → 127 (not found) | 126 (not executable).
V2: no cmd → exit 0 once released.
V3: env cleanup: `Command::env_remove` ∀ `CGMR_*` except `CGMR_HELD_SLOTS` (updated w/ keys now held).
V4: `--govern`: spawn cmd in own process group; keep evaluating gates; gates fail → SIGSTOP group; pass again → SIGCONT; forward SIGINT SIGTERM SIGHUP; exit w/ cmd's code (signal death → 128+n).
V5: `--background`: lower CPU priority (`setpriority` nice 19) & IO priority (Linux `ioprio_set` idle class; macOS `setiopolicy_np` throttle) before exec.
V6: `--or-background`: `--timeout` reached → start anyway as `--background` instead of exit 124.
V7: stdout|stdin|stderr untouched; cmd owns stdout (`.` §I).
V8: `--govern`: stopped cmd keeps its slot & reservation (held fds inherited); `--timeout` applies only before start; `--host` allowed (remote checks continue each round).

## §T TASKS

id|status|task|cites
T1|x|exec + exit codes 126/127|V1
T2|x|no-cmd exit 0|V2
T3|x|env cleanup & held marker|V3
T4|~|`--govern` spawn, stop/cont, forward signals|V4
T5|x|`--background` both OS|V5
T6|x|`--or-background`|V6
T7|.|govern w/ slots, timeout & host|V8

## §B BUGS

id|date|cause|fix
