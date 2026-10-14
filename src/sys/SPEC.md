# SPEC

## §G GOAL

`src/sys` — OS facade. ONLY node that calls the OS or allows `unsafe`; exposes safe fns to domain nodes.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/sys|OS facade: ∀ syscalls, FFI, `sysinfo`, `rustix`, `libc`; only node allowed `unsafe`
sib|src/units|parse & render sizes, rates, percents, durations, temperatures, clock windows
sib|src/signal|`Signal` & `Source` traits, `Reading`, sampler w/ two-sample deltas, monotonic clock, fixture root
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
sib|src/advise|`advise` verb: how many jobs fit now, what limits them, what to do
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- facade ∀ `sysinfo`, `rustix`, `libc` + spike-decided `mach2`, `core-foundation`, `io-kit-sys`, `objc2`, `libloading`, `zbus` (R1–R4). prefer safe crate over raw FFI when it covers the call.
- `#[allow(unsafe_code)]` scoped to FFI submodules here; ⊥ elsewhere (`nix:V11`).
- `cfg(target_os)` per platform; other OS → `Unsupported`.

## §I INTERFACES

- files: `read_text(root, path) -> io::Result<String>` (procfs, sysfs, cgroupfs).
- sysinfo: shared lazily refreshed `System` snapshot: load avg, memory, processes, per-disk IO (if T1 spike passes).
- rustix: `statvfs(path)`, `flock(fd)`, `setpriority`, `exec`-safe fd handling.
- macOS FFI: `sysctlbyname` (`kern.memorystatus_vm_pressure_level`, `kern.memorystatus_level`, `kern.num_files`, `kern.maxfiles`, `kern.num_tasks`, `kern.maxproc`, `vm.swapusage`), `host_processor_info`, `host_statistics64`, IOKit (block storage stats, power sources, `HIDIdleTime`, GPU `PerformanceStatistics`, thermal state), `setiopolicy_np`.
- Linux FFI: `ioprio_set`; NVML via runtime `dlopen` (absent lib → `Unsupported`).
- net: `getifaddrs` + `if_data` (macOS); TCP connect w/ timeout (std).

## §R RESEARCH

id|topic|finding|src
R1|mach & sysctl access|open: `mach2` vs raw `libc` for `host_processor_info`, `host_statistics64`|-
R2|IOKit & CoreFoundation|open: `io-kit-sys` + `core-foundation` vs hand-written FFI; licence & size|-
R3|NVML loading|open: `libloading` vs `libc::dlopen`|-
R4|Objective-C calls|open: `objc2` needed only if `src/power` R1/R2 find no C API|-

## §V INVARIANTS

V1: ∀ `unsafe` block ! `// SAFETY:` comment stating why sound (`undocumented_unsafe_blocks` deny).
V2: ∀ pub fn safe; errors → `Result`, ⊥ panic, ⊥ abort.
V3: raw bytes from FFI decoded by pure fns tested w/ captured fixtures; FFI call itself = thin.
V4: missing lib|sysctl|file → `Unsupported` w/ reason, ⊥ error exit.
V5: `sysinfo` refresh only parts requested (`src/signal:V3`).
V6: ⊥ shelling out to `sysctl`, `vm_stat`, `iostat`, `pmset` ∵ cost (`docs/CONGESTION.md` cheapness).
V7: `capture(root, out)` copies every raw source this node reads (procfs, sysfs, cgroupfs files) & writes decoded sysctl|mach|IOKit values as text into `out/<os>-<arch>/`; read-only on host; ⊥ secrets (cmdlines, environ, usernames beyond counts).

## §T TASKS

id|status|task|cites
T1|.|SPIKE: prove `sysinfo` per-disk IO stats on Linux & macOS; record outcome in `src/io` §C|V5
T2|.|`read_text` w/ `Root`|V4
T3|.|`sysinfo` snapshot w/ selective refresh|V5
T4|.|`statvfs`, `flock`, `setpriority` via rustix|V2
T5|.|macOS `sysctlbyname` wrapper + typed reads|V1,V2,V3
T6|.|macOS `host_processor_info` & `host_statistics64`|V1,V3
T7|.|macOS IOKit: block storage stats|V1,V3
T8|.|macOS IOKit: power sources, `HIDIdleTime`, thermal state|V1,V3
T9|.|macOS IOKit: GPU `PerformanceStatistics`|V1,V3,V4
T10|.|macOS `setiopolicy_np`; Linux `ioprio_set`|V1,V2
T11|.|Linux NVML via `dlopen`|V1,V4
T12|.|`getifaddrs` interface counters (macOS)|V1,V3
T13|.|TCP connect probe w/ timeout|V2
T14|.|SPIKE: settle R1–R4; record finding & chosen crate; update root §C deps|-
T15|.|fixture capture of ∀ sources|V7

## §B BUGS

id|date|cause|fix
