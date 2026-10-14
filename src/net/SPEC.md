# SPEC

## §G GOAL

`src/net` — network throughput & latency gates.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/net|interface rates, RTT probe
sib|src/units|parse & render sizes, rates, percents, durations, temperatures, clock windows
sib|src/signal|`Signal` & `Source` traits, `Reading`, sampler w/ two-sample deltas, monotonic clock, fixture root
sib|src/sys|OS facade: ∀ syscalls, FFI, `sysinfo`, `rustix`, `libc`; only node allowed `unsafe`
sib|src/cpu|load, run queue, CPU idle, CPU pressure, steal
sib|src/mem|available memory, memory pressure & level, swap rate, dirty pages
sib|src/disk|free space & inodes on a path
sib|src/io|IO throughput, busy, latency, queue, pressure; path → device resolution
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
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, `src/sys`. ⊥ raw sockets, ⊥ root.

## §I INTERFACES

- switches: `--net-rx X/s` · `--net-tx X/s` · `--net X/s` · `--net-iface I` · `--rtt D` · `--rtt-host H[:PORT]`.
- sources: Linux `/proc/net/dev` · macOS `getifaddrs` + `if_data`.

## §V INVARIANTS

V1: default ifaces = ∀ up, non-loopback; `--net-iface I` selects one; unknown I → usage 64.
V2: `--net-rx X` ⇔ rx B/s < X; `--net-tx X` ⇔ tx B/s < X; `--net X` ⇔ sum < X. first sample `Warming`.
V3: `--rtt D` ⇔ TCP connect time to `--rtt-host` < D; port default 443; connect timeout = D (fail = gate fails, ⊥ error exit).
V4: `--rtt` w/o `--rtt-host` → usage 64.
V5: counter wrap (32-bit `if_data`) handled → `Warming`, ⊥ huge rate.

## §T TASKS

id|status|task|cites
T1|.|`/proc/net/dev` parser|V2
T2|.|macOS interface counters|V2,V5
T3|.|iface selection|V1
T4|.|`--net-rx` `--net-tx` `--net`|V2
T5|.|`--rtt` + `--rtt-host`|V3,V4

## §B BUGS

id|date|cause|fix
