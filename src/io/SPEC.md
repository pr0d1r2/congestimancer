# SPEC

## §G GOAL

`src/io` — disk IO signals & gates.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/io|IO throughput, busy, latency, queue, pressure; path → device resolution
sib|src/units|parse & render sizes, rates, percents, durations, temperatures, clock windows
sib|src/signal|`Signal` & `Source` traits, `Reading`, sampler w/ two-sample deltas, monotonic clock, fixture root
sib|src/sys|OS facade: ∀ syscalls, FFI, `sysinfo`, `rustix`, `libc`; only node allowed `unsafe`
sib|src/cpu|load, run queue, CPU idle, CPU pressure, steal
sib|src/mem|available memory, memory pressure & level, swap rate, dirty pages
sib|src/disk|free space & inodes on a path
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

- deps: `src/units`, `src/signal` (PSI parser), `src/sys`, `src/cgroup` (scope).
- throughput source decided by `src/sys` T1 spike: sysinfo per-disk stats if proven on both OS; else `/proc/diskstats` & IOKit. outcome (`src/sys:T1`): sysinfo `Disk::usage` = read|written bytes only (macOS probed: `IOBlockStorageDriver` Statistics, APFS volumes repeat whole-disk counters; Linux by source: `/proc/diskstats` sectors, unverified on host); ⊥ ops, busy, latency, queue ∴ ⊥ sysinfo here → `/proc/diskstats` & IOKit Statistics (`Operations`, `Total Time`, bytes) for ∀ metrics.

## §I INTERFACES

- switches: `--io-read X/s` · `--io-write X/s` · `--io X/s` · `--io-device D` · `--io-busy X` · `--io-latency D` · `--io-queue X` · `--io-pressure X` · `--io-pressure-full X`.
- sources: Linux `/proc/diskstats`, `/proc/pressure/io` · macOS IOKit block storage stats.

## §R RESEARCH

id|topic|finding|src
R1|macOS IO latency|`IOBlockStorageDriver` `Statistics` R\|W `Bytes`, `Operations`, `Total Time` ns. latency = time/ops deltas. ⊥ in-flight ∴ queue ⊥. Intel unverified|`IOBlockStorageDriver.h`
R2|APFS physical disk|IOKit ⊥ `DADisk`: `f_mntfromname` → `IOBSDNameMatching` → parents until `IOBlockStorageDriver`|`IOBSD.h`

## §V INVARIANTS

V1: device = one holding `--disk-path` (default cwd) unless `--io-device D`.
V2: macOS APFS: measure physical disk under container, ⊥ volume.
V3: mount w/o block device (tmpfs, NFS, FUSE) → `Unsupported` (69) + hint `--io-device`.
V4: rates averaged over poll interval; first sample `Warming`.
V5: `--io-read X` ⇔ read B/s < X; `--io-write X` ⇔ write B/s < X; `--io X` ⇔ sum < X.
V6: `--io-busy X` ⇔ busy % < X (Linux `io_ticks`; macOS min(100, total time delta/interval), upper bound).
V7: `--io-latency D` ⇔ avg time per completed request over interval < D; 0 requests → passes.
V8: `--io-queue X` ⇔ in-flight < X. Linux only.
V9: `--io-pressure X` ⇔ PSI `some avg10` < X; `-full` ⇔ `full avg10` < X. Linux only; cgroup `io.pressure` when inside cgroup.
V10: `--io-device D`: D = `Name` (`src/units:V13`) ∴ ⊥ sysfs path escape.

## §T TASKS

id|status|task|cites
T1|x|`/proc/diskstats` parser|V5,V6,V7,V8
T2|.|path → device: Linux (mountinfo → diskstats)|V1,V3
T3|.|path → physical disk: macOS APFS|V1,V2,V3
T4|.|`--io-read` `--io-write` `--io` both OS|V4,V5
T5|.|`--io-busy` both OS|V6
T6|.|`--io-latency` both OS|V7
T7|.|`--io-queue`|V8
T8|.|`--io-pressure` & `-full`|V9
T9|.|`--io-device` override|V1,V10
T10|x|SPIKE: settle R1–R2|-

## §B BUGS

id|date|cause|fix
