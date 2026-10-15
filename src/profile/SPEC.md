# SPEC

## §G GOAL

`src/profile` — named sets of switches, so repeated gate combinations get one name.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/profile|named switch sets from project & user files, lookup & trust rule
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
sib|src/remote|`--host` evaluation over ssh
sib|src/report|`status` human & JSON, verbose reasons, heartbeat text, colour
sib|src/advise|`advise` verb: how many jobs fit now, what limits them, what to do
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- deps: `src/units`. own line parser; ⊥ TOML|serde dep.

## §I INTERFACES

- switches: `--profile NAME` (env `CGMR_PROFILE`) · `--profile-file P` (env `CGMR_PROFILE_FILE`).
- verb: `cgmr profiles` lists names, their file & switches.
- files: project `.cgmr` (nearest, walking up from cwd, stopping at filesystem root) · user `$XDG_CONFIG_HOME/cgmr/profiles` (else `~/.config/cgmr/profiles`).
- format: `[name]` section headers; each line = one switch exactly as on CLI (`--load 2`); `#` comments; blank lines ignored.

## §V INVARIANTS

V1: precedence: CLI flag > env var > profile > builtin default (`src/cli:V2`).
V2: lookup: `--profile-file` | `CGMR_PROFILE_FILE` alone if given; else project file, then user file; same name in both → project wins.
V3: `--when` allowed only in user file ∵ a cloned repo's `.cgmr` ⊥ may run commands (like direnv trust); `--when` in project file → 64 naming file & line.
V4: unknown name → 64 listing available names; unknown switch | bad value in file → 64 w/ `file:line`.
V5: profile switches go through same parser & `Name`|unit validation as CLI (`src/units:V13`).
V6: single-value switch set in profile & on CLI → CLI wins (⊥ repeat error); twice in one profile → 64.
V7: file read errors other than not-found → 64; not-found → skip.

## §T TASKS

id|status|task|cites
T1|x|file discovery & precedence|V2,V7
T2|x|line parser w/ `file:line` errors|V4,V5
T3|x|merge into config w/ precedence|V1,V6
T4|x|`--when` trust rule|V3
T5|x|`profiles` verb|V4

## §B BUGS

id|date|cause|fix
