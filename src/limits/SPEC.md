# SPEC

## §G GOAL

`src/limits` — open files & process count vs system maximum.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/limits|open files & processes vs system maximum
sib|src/units|parse & render sizes, rates, percents, durations, temperatures, clock windows
sib|src/signal|`Signal` & `Source` traits, `Reading`, sampler w/ two-sample deltas, monotonic clock, fixture root
sib|src/sys|OS facade: ∀ syscalls, FFI, `sysinfo`, `rustix`, `libc`; only node allowed `unsafe`
sib|src/cpu|load, run queue, CPU idle, CPU pressure, steal
sib|src/mem|available memory, memory pressure & level, swap rate, dirty pages
sib|src/disk|free space & inodes on a path
sib|src/io|IO throughput, busy, latency, queue, pressure; path → device resolution
sib|src/net|interface rates, RTT probe
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

- deps: `src/units`, `src/signal`, `src/sys`.

## §I INTERFACES

- switches: `--fds X` · `--pids X` (percent).
- sources: Linux `/proc/sys/fs/file-nr`, `/proc/sys/kernel/pid_max` + process count · macOS `kern.num_files`/`kern.maxfiles`, `kern.num_tasks`/`kern.maxproc`.

## §V INVARIANTS

V1: `--fds X` ⇔ open / max × 100 < X.
V2: `--pids X` ⇔ processes / max × 100 < X.
V3: max = 0 | unreadable → `Unsupported`, ⊥ divide.

## §T TASKS

id|status|task|cites
T1|x|`file-nr` parser + `--fds` Linux|V1,V3
T2|x|`--fds` macOS|V1,V3
T3|.|`--pids` both OS|V2,V3

## §B BUGS

id|date|cause|fix
