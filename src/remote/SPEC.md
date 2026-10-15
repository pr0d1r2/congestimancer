# SPEC

## §G GOAL

`src/remote` — evaluate gates on another host.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/remote|`--host` evaluation over ssh
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
sib|src/gpu|GPU utilisation & free memory
sib|src/procs|process counts by name & by user
sib|src/cgroup|cgroup v2 detection, limits, throttling, pressure override
sib|src/gate|gate model, strict compare, AND evaluation, unsupported verdict, custom gates
sib|src/wait|poll loop: interval, backoff, timeout, settle, jitter, heartbeat
sib|src/slot|per-user lock dir, slots, FIFO, rate, spacing, memory reservations, nested held marker
sib|src/launch|exec, no-cmd exit, govern, background priority, env cleanup
sib|src/report|`status` human & JSON, verbose reasons, heartbeat text, colour
sib|src/profile|named switch sets from project & user files, lookup & trust rule
sib|src/advise|`advise` verb: how many jobs fit now, what limits them, what to do
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- deps: `src/gate`, `src/report` JSON schema. uses system `ssh` binary; ⊥ ssh library.

## §I INTERFACES

- switch: `--host H` (`run`, `check`, `status`, `advise`).
- call: `ssh -o BatchMode=yes H cgmr check --json <gates>`.

## §V INVARIANTS

V1: ∀ gates evaluated on H; slots, reservations, cmd stay local.
V2: remote JSON `schema_version` ≠ supported → exit 69 w/ reason.
V3: ssh failure (unreachable, auth) → round fails (keep waiting), reason in verbose|heartbeat; ⊥ exit.
V4: remote `cgmr` missing (exit 127) → exit 69.
V5: gate args passed as separate argv words, quoted for remote shell; ⊥ injection.
V6: host H ⊥ starting `-` (refuse, 64) & passed after `--` in ssh argv ∴ ⊥ option injection (`-oProxyCommand=...`).

## §T TASKS

id|status|task|cites
T1|x|build remote argv w/ safe quoting|V5
T2|.|run ssh, parse JSON, version check|V1,V2
T3|.|failure mapping|V3,V4
T4|.|host validation & `--` separator|V6

## §B BUGS

id|date|cause|fix
