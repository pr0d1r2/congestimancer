# SPEC

## §G GOAL

`src/sys` — OS facade. ONLY node that calls the OS or allows `unsafe`; exposes safe fns to domain nodes.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/sys|OS facade: ∀ syscalls, FFI, `sysinfo`, `rustix`, `libc`; only node allowed `unsafe`

## §C CONSTRAINTS

- facade ∀ `sysinfo`, `rustix`, `libc`. prefer safe crate over raw FFI when it covers the call.
- `#[allow(unsafe_code)]` scoped to FFI submodules here; ⊥ elsewhere (`.:V11`).
- `cfg(target_os)` per platform; other OS → `Unsupported`.

## §I INTERFACES

- files: `read_text(root, path) -> io::Result<String>` (procfs, sysfs, cgroupfs).
- sysinfo: shared lazily refreshed `System` snapshot: load avg, memory, processes, per-disk IO (if T1 spike passes).
- rustix: `statvfs(path)`, `flock(fd)`, `setpriority`, `exec`-safe fd handling.
- macOS FFI: `sysctlbyname` (`kern.memorystatus_vm_pressure_level`, `kern.memorystatus_level`, `kern.num_files`, `kern.maxfiles`, `kern.num_tasks`, `kern.maxproc`, `vm.swapusage`), `host_processor_info`, `host_statistics64`, IOKit (block storage stats, power sources, `HIDIdleTime`, GPU `PerformanceStatistics`, thermal state), `setiopolicy_np`.
- Linux FFI: `ioprio_set`; NVML via runtime `dlopen` (absent lib → `Unsupported`).
- net: `getifaddrs` + `if_data` (macOS); TCP connect w/ timeout (std).

## §V INVARIANTS

V1: ∀ `unsafe` block ! `// SAFETY:` comment stating why sound (`undocumented_unsafe_blocks` deny).
V2: ∀ pub fn safe; errors → `Result`, ⊥ panic, ⊥ abort.
V3: raw bytes from FFI decoded by pure fns tested w/ captured fixtures; FFI call itself = thin.
V4: missing lib|sysctl|file → `Unsupported` w/ reason, ⊥ error exit.
V5: `sysinfo` refresh only parts requested (`src/signal:V3`).
V6: ⊥ shelling out to `sysctl`, `vm_stat`, `iostat`, `pmset` ∵ cost (`docs/CONGESTION.md` cheapness).

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

## §B BUGS

id|date|cause|fix
