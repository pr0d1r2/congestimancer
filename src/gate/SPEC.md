# SPEC

## §G GOAL

`src/gate` — gate model: threshold vs reading, strict compare, AND over ∀ gates, custom gates.

## §N NAV

rel|path|lens
up|.|-
up|src|the tool: congestion domains, gates, waiting, launch, CLI
self|src/gate|gate model, strict compare, AND evaluation, unsupported verdict, custom gates
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
sib|src/wait|poll loop: interval, backoff, timeout, settle, jitter, heartbeat
sib|src/slot|per-user lock dir, slots, FIFO, rate, spacing, memory reservations, nested held marker
sib|src/launch|exec, no-cmd exit, govern, background priority, env cleanup
sib|src/remote|`--host` evaluation over ssh
sib|src/report|`status` human & JSON, verbose reasons, heartbeat text, colour
sib|src/profile|named switch sets from project & user files, lookup & trust rule
sib|src/advise|`advise` verb: how many jobs fit now, what limits them, what to do
sib|src/cli|clap verbs, flags, env defaults, exit codes, completions, man page

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, ∀ domain nodes (public surface only).

## §I INTERFACES

- `Gate { signal, op: Below|Above|Is, threshold }`; `Verdict`: `Pass` · `Fail { value, threshold, trend }` · `Warming` · `Unsupported { reason }`.
- `evaluate(&[Gate], &mut Source) -> Evaluation` (per-gate verdicts + overall).
- custom: `--when CMD` · `--when-file PATH`.

## §V INVARIANTS

V1: gate passes ⇔ system STRICTLY better than threshold (`<` for load-like, `>` for capacity-like). equality fails.
V2: overall pass ⇔ ∀ gates `Pass` (AND). ∃ `Warming` → ⊥ pass yet.
V3: ∃ `Unsupported` → overall `Unsupported` → exit 69 immediately; ⊥ waiting forever, ⊥ silent pass.
V4: `--when CMD` ⇔ `sh -c CMD` exits 0; stdout|stderr of CMD → cgmr stderr; ⊥ env default (`CGMR_WHEN` ignored).
V5: `--when-file P` ⇔ P exists.
V6: no gates given → overall pass immediately.
V7: evaluation reads each signal once per round even if ∃ several gates on it.
V8: `check` & `status`: any requested rate|percent signal → take 2nd sample after `--interval` (one wait, ⊥ loop) ∴ output ⊥ `Warming`.

## §T TASKS

id|status|task|cites
T1|x|`Gate`, `Verdict`, strict compare|V1
T2|x|AND evaluation w/ warming & unsupported|V2,V3,V6
T3|.|`--when`|V4
T4|.|`--when-file`|V5
T5|x|one read per signal per round|V7
T6|.|one-shot two-sample evaluation for `check` & `status`|V8

## §B BUGS

id|date|cause|fix
