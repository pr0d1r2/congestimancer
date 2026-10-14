# SPEC

## §G GOAL

`src/gpu` — GPU utilisation & free memory gates.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/gpu|GPU utilisation & free memory
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

- deps: `src/units`, `src/signal`, `src/sys`. NVML loaded at runtime ∴ ⊥ link-time dep; behind cargo feature `nvml` (default on); feature off → NVIDIA `Unsupported`.

## §I INTERFACES

- switches: `--gpu X` · `--gpu-mem X` · `--gpu-device I`.
- sources: Linux NVML (NVIDIA), sysfs `gpu_busy_percent` & `mem_info_vram_total|used` (AMD) · macOS IOKit `PerformanceStatistics` (R1).

## §R RESEARCH

id|topic|finding|src
R1|Apple Silicon GPU stats|settled: ∀ `IOAccelerator` svc, dict `PerformanceStatistics`, util = `Device Utilization %`; unprivileged. M4: idle ~20, Metal load 99. no memory total (unified RAM) ∴ `--gpu-mem` `Unsupported`. Intel Mac AMD: + `vramFreeBytes`, unverified|spike M4

## §V INVARIANTS

V1: `--gpu X` ⇔ utilisation % < X.
V2: `--gpu-mem X` ⇔ free GPU memory > X.
V3: default = ∀ GPUs, worst wins; `--gpu-device I` selects index; unknown I → usage 64.
V4: no GPU source → `Unsupported`.
V5: `--gpu-device I`: I = index (`Count`) ∴ ⊥ path segment from user text.
V6: device w/o free-memory key (Apple Silicon) → `--gpu-mem` `Unsupported`; util still read.

## §T TASKS

id|status|task|cites
T1|.|AMD sysfs parser|V1,V2
T2|.|NVML reads|V1,V2,V4
T3|.|macOS IOKit GPU stats|V1,V2,V4,V6
T4|.|device selection & worst-wins|V3,V5
T5|~|SPIKE: settle R1 on M4 & Intel Mac|-

## §B BUGS

id|date|cause|fix
