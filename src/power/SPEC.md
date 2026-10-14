# SPEC

## §G GOAL

`src/power` — thermal & power state gates.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/power|thermal throttling, temperature, AC, battery, low power
sib|src/units|parse & render sizes, rates, percents, durations, temperatures, clock windows
sib|src/signal|`Signal` & `Source` traits, `Reading`, sampler w/ two-sample deltas, monotonic clock, fixture root
sib|src/sys|OS facade: ∀ syscalls, FFI, `sysinfo`, `rustix`, `libc`; only node allowed `unsafe`
sib|src/cpu|load, run queue, CPU idle, CPU pressure, steal
sib|src/mem|available memory, memory pressure & level, swap rate, dirty pages
sib|src/disk|free space & inodes on a path
sib|src/io|IO throughput, busy, latency, queue, pressure; path → device resolution
sib|src/net|interface rates, RTT probe
sib|src/limits|open files & processes vs system maximum
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

- switches: `--cool` · `--temp X` · `--on-ac` · `--battery X` · `--no-low-power`.
- sources: Linux `/sys/class/thermal`, `thermal_throttle` counters, `/sys/class/power_supply`, power-profiles-daemon state · macOS `notify` thermal pressure & `lowpowermode`, IOKit power sources.

## §R RESEARCH

id|topic|finding|src
R1|macOS thermal state|C `notify_get_state` `kOSThermalNotificationPressureLevelName`: 0 = nominal. probed = `thermalState`. `IOPMGetThermalWarningLevel` not found arm64|SDK `OSThermalNotification.h`
R2|macOS Low Power Mode|C `notify_get_state` `com.apple.system.lowpowermode` ≠ 0; Foundation `isLowPowerModeEnabled` reads same (disasm) ∴ ⊥ `objc2`. on-state unprobed (root)|dyld cache Foundation
R3|Linux throttle signal|`thermal_throttle` Intel-only ∴ ⊥ AMD, ARM. fallback: cooling device `Processor`\|`cpufreq-*` `cur_state` > 0; neither → `Unsupported`. unverified on host|`therm_throt.c`, `cpufreq_cooling.c`

## §V INVARIANTS

V1: `--cool` ⇔ no active throttling: Linux throttle counters unchanged over last interval, else CPU cooling devices idle; macOS thermal pressure = nominal.
V2: `--temp X` ⇔ hottest thermal zone < X. Linux only; macOS `Unsupported` (no public API).
V3: `--on-ac` ⇔ mains power present; desktop w/o battery & w/ mains → passes.
V4: `--battery X` ⇔ charge % > X | no battery.
V5: `--no-low-power` ⇔ Low Power Mode | power-saver profile off; Linux w/o power-profiles-daemon → `Unsupported`.

## §T TASKS

id|status|task|cites
T1|.|Linux throttle counters + thermal zones parser|V1,V2
T2|.|macOS thermal state|V1
T3|.|`--cool` both OS|V1
T4|.|`--temp`|V2
T5|.|power supply parser Linux + IOKit macOS|V3,V4
T6|.|`--on-ac` `--battery`|V3,V4
T7|.|`--no-low-power` both OS|V5
T8|x|SPIKE: settle R1–R3 on M4, Intel Mac & Linux hosts|-

## §B BUGS

id|date|cause|fix
