# SPEC

## §G GOAL

`src/io` — disk IO congestion signals & their gates.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/io|IO throughput, busy, latency, queue, pressure; path → device resolution

## §C CONSTRAINTS

- deps: `src/units`, `src/signal` (PSI parser), `src/sys`, `src/cgroup` (scope).
- throughput source decided by `src/sys` T1 spike: sysinfo per-disk stats if proven on both OS; else `/proc/diskstats` & IOKit. outcome ? recorded here.

## §I INTERFACES

- switches: `--io-read X/s` · `--io-write X/s` · `--io X/s` · `--io-device D` · `--io-busy X` · `--io-latency D` · `--io-queue X` · `--io-pressure X` · `--io-pressure-full X`.
- sources: Linux `/proc/diskstats`, `/proc/pressure/io` · macOS IOKit block storage stats.

## §V INVARIANTS

V1: device = one holding `--disk-path` (default cwd) unless `--io-device D`.
V2: macOS APFS: measure physical disk under container, ⊥ volume.
V3: mount w/o block device (tmpfs, NFS, FUSE) → `Unsupported` (69) + hint `--io-device`.
V4: rates averaged over poll interval; first sample `Warming`.
V5: `--io-read X` ⇔ read B/s < X; `--io-write X` ⇔ write B/s < X; `--io X` ⇔ sum < X.
V6: `--io-busy X` ⇔ busy % < X (Linux `io_ticks`; macOS total time deltas).
V7: `--io-latency D` ⇔ avg time per completed request over interval < D; 0 requests → passes.
V8: `--io-queue X` ⇔ in-flight < X. Linux only.
V9: `--io-pressure X` ⇔ PSI `some avg10` < X; `-full` ⇔ `full avg10` < X. Linux only; cgroup `io.pressure` when inside cgroup.

## §T TASKS

id|status|task|cites
T1|.|`/proc/diskstats` parser|V5,V6,V7,V8
T2|.|path → device: Linux (`st_dev` → sysfs)|V1,V3
T3|.|path → physical disk: macOS APFS|V1,V2,V3
T4|.|`--io-read` `--io-write` `--io` both OS|V4,V5
T5|.|`--io-busy` both OS|V6
T6|.|`--io-latency` both OS|V7
T7|.|`--io-queue`|V8
T8|.|`--io-pressure` & `-full`|V9
T9|.|`--io-device` override & validation|V1

## §B BUGS

id|date|cause|fix
