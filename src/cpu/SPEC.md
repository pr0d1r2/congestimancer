# SPEC

## §G GOAL

`src/cpu` — CPU congestion signals & their gates.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/cpu|load, run queue, CPU idle, CPU pressure, steal
sib|src/units|parse & render sizes, rates, percents, durations, temperatures, clock windows
sib|src/signal|`Signal` & `Source` traits, `Reading`, sampler w/ two-sample deltas, monotonic clock, fixture root
sib|src/sys|OS facade: ∀ syscalls, FFI, `sysinfo`, `rustix`, `libc`; only node allowed `unsafe`
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

- deps: `src/units`, `src/signal` (PSI parser), `src/sys`, `src/cgroup` (scope). N (logical CPUs) from `std::thread::available_parallelism`.

## §I INTERFACES

- switches: `--load X` · `--all-load X` · `--load-window 1m|5m|15m` · `--run-queue X` · `--all-run-queue X` · `--cpu-idle X` · `--cpu-pressure X` · `--steal X`.
- sources: Linux `/proc/loadavg`|sysinfo, `/proc/stat` (`procs_running`, cpu times, `steal`), `/proc/pressure/cpu` · macOS sysinfo load, `host_processor_info`, `processor_set_statistics` (run queue).

## §R RESEARCH

id|topic|finding|src
R1|macOS run queue|settled: `processor_set_statistics(processor_set_default, PROCESSOR_SET_LOAD_INFO).load_average` / 1000 = runnable threads, ~5s EWMA (xnu `compute_averages`); unprivileged, ~us. Rejected: `HOST_LOAD_INFO`/`vm.loadavg` (1m+ avg), `kern.*` (no runq), `proc_pidinfo` scan (misses root pids). M4 26.5: 5-8, 20 spinners 30-45 in 2s. Intel unverified|spike M4

## §V INVARIANTS

V1: `--load X` passes ⇔ load / N < X; `--all-load X` ⇔ load < X.
V2: `--load-window` default `1m`; selects 1|5|15 min average.
V3: `--run-queue X` ⇔ running tasks / N < X; `--all-run-queue X` ⇔ running < X. Linux `procs_running`; macOS pset `load_average` / 1000 (R1).
V4: `--cpu-idle X` ⇔ idle % over last interval > X; first sample `Warming`.
V5: `--cpu-pressure X` ⇔ PSI `some avg10` < X. Linux only; macOS `Unsupported`. inside cgroup → cgroup `cpu.pressure` (`src/cgroup`).
V6: `--steal X` ⇔ steal % over last interval < X. Linux only.
V7: N ≥ 1 always; division checked.

## §T TASKS

id|status|task|cites
T1|x|N from `available_parallelism`, ≥ 1|V7
T2|x|load avg reader both OS + `--load` `--all-load`|V1
T3|x|`--load-window`|V2
T4|x|`/proc/stat` parser: `procs_running`, cpu times, steal|V3,V4,V6
T5|x|macOS run queue: pset `load_average` / 1000 via `src/sys`|V3
T6|.|`--cpu-idle` both OS (deltas)|V4
T7|.|`--cpu-pressure` via shared PSI parser|V5
T8|.|`--steal`|V6
T9|x|SPIKE: settle R1|-

## §B BUGS

id|date|cause|fix
