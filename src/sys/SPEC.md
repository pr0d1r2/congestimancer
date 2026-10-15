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
sib|src/profile|named switch sets from project & user files, lookup & trust rule
sib|src/advise|`advise` verb: how many jobs fit now, what limits them, what to do
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- facade ∀ `sysinfo`, `rustix`, `libc`, `objc2-io-kit`, `objc2-core-foundation` (R1–R3); ⊥ `objc2*` (R4), `zbus`. prefer safe crate over raw FFI when it covers the call.
- `#[allow(unsafe_code)]` scoped to FFI submodules here; ⊥ elsewhere (`nix:V11`).
- `cfg(target_os)` per platform; other OS → `Unsupported`.

## §I INTERFACES

- files: `read_text`, `list_dir` under `Root` (procfs, sysfs, cgroupfs).
- users: `user_id`, `user_name`, `current_uid`, `local_time`.
- sysinfo: lazy `System` snapshot: load avg, memory, processes w/ ppid; disk IO: `/proc/diskstats`, IOKit (T1).
- rustix: `statvfs`, `flock`, `setpriority`; `fcntl` clear `FD_CLOEXEC` (slot lock fd survives exec); `stat` → `st_dev` (major, minor), follows symlinks.
- macOS FFI: `sysctlbyname` (`kern.memorystatus_*`, `kern.num_*`, `kern.max*`, `vm.swapusage`), `host_processor_info`, `host_statistics64`, IOKit (block stats, power, `HIDIdleTime`, GPU stats), `notify_get_state`, `setiopolicy_np`.
- Linux FFI: `ioprio_set`; NVML `dlopen` (R3).
- signals: `signal_group` (rustix `kill_process_group`); `block_forwarded` → `Blocked`: mask INT TERM HUP CHLD (`pthread_sigmask`), `wait_timeout` (Linux `sigtimedwait`; macOS kqueue `EVFILT_SIGNAL` + `sigwait`), old mask back on drop. ⊥ handler (`.:V22`).
- net: `NET_RT_IFLIST2` (macOS, R5); TCP connect w/ timeout (std); Linux system bus `UnixStream`, ⊥ codec.

## §R RESEARCH

id|topic|finding|src
R1|mach & sysctl access|`libc` ⊥ `mach2` 0.7 (⊥ fns). `mach_host_self` deprecated → scoped `allow(deprecated)`. probed kr=0|`libc` 0.2.190 `unix/bsd/apple/mod.rs`
R2|IOKit & CoreFoundation|`objc2-io-kit` + `objc2-core-foundation` 0.3: pulled by `sysinfo` ∴ 0 new crates; MIT; `CFRetained` RAII. probed ok|`sysinfo` 0.39.6 `Cargo.toml`
R3|NVML loading|`libc::dlopen` ⊥ `libloading`: ISC ⊥ `deny.toml`, still `unsafe`. Linux: by source, unverified on host|`libloading` 0.9 `Cargo.toml`
R4|Objective-C calls|settled: ⊥ ObjC. thermal & low power → C `notify_get_state`|`src/power` R1, R2
R5|macOS iface counters|`NET_RT_IFLIST2` `if_data64` ⊥ `getifaddrs` `if_data`: 32-bit, wraps|`net/if.h`

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
T1|x|SPIKE: `sysinfo` per-disk IO → `src/io` §C|V5
T2|x|`read_text` w/ `Root`|V4
T3|x|`sysinfo` snapshot w/ selective refresh|V5
T4|x|`statvfs`, `flock`, `setpriority` via rustix|V2
T5|x|macOS `sysctlbyname` wrapper + typed reads|V1,V2,V3
T6|x|macOS mach host stats & pset load|V1,V3
T7|x|macOS IOKit: block storage stats|V1,V3
T8|x|macOS IOKit: power sources, `HIDIdleTime`, thermal state|V1,V3
T9|x|macOS IOKit: GPU `PerformanceStatistics`|V1,V3,V4
T10|x|macOS `setiopolicy_np`; Linux `ioprio_set`|V1,V2
T11|x|Linux NVML via `dlopen`|V1,V4
T12|x|`NET_RT_IFLIST2` interface counters (macOS, R5)|V1,V3
T13|x|TCP connect probe w/ timeout|V2
T14|x|SPIKE: settle R1–R4|-
T15|x|fixture capture of ∀ sources|V7
T16|x|Linux system bus `UnixStream`|V2,V4
T17|x|`list_dir`: sorted names|V4
T18|x|`getpw*_r`, localtime_r|V1,V2
T19|x|`set_inheritable(&File)`: retired by T24 ∵ cleared `FD_CLOEXEC` in cgmr ∴ ∀ child got lock|V2
T20|x|`device_of(&Path)` → (major, minor) of `st_dev` via rustix `stat` ∵ `src/io` T2|V2
T21|x|`signal_group(pgid, GroupSignal)` STOP CONT INT TERM HUP via rustix ⊥ `unsafe` ∵ `src/launch` V4|V2
T22|x|`block_forwarded()` → `Blocked`; `wait_timeout(d)` → forwarded sig, child exit or none; drop restores mask ∵ `src/launch` V4|V1,V2
T23|x|`Blocked::unblock_in(&mut Command)`: child starts w/ mask from before block (`pre_exec` `sigprocmask`) ∵ `std` spawn inherits mask|V1,V2
T24|x|`inherit_in(&mut Command, &File)`: clear `FD_CLOEXEC` in that child only (`pre_exec` `fcntl`); own fd stays close-on-exec ∵ `src/launch:V9`|V1,V2

## §B BUGS

id|date|cause|fix
