# SPEC

## §G GOAL

`src/cpu` — CPU congestion signals & their gates.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/cpu|load, run queue, CPU idle, CPU pressure, steal

## §C CONSTRAINTS

- deps: `src/units`, `src/signal` (PSI parser), `src/sys`, `src/cgroup` (scope). N (logical CPUs) from `std::thread::available_parallelism`.

## §I INTERFACES

- switches: `--load X` · `--all-load X` · `--load-window 1m|5m|15m` · `--run-queue X` · `--all-run-queue X` · `--cpu-idle X` · `--cpu-pressure X` · `--steal X`.
- sources: Linux `/proc/loadavg`|sysinfo, `/proc/stat` (`procs_running`, cpu times, `steal`), `/proc/pressure/cpu` · macOS sysinfo load, `host_processor_info`.

## §V INVARIANTS

V1: `--load X` passes ⇔ load / N < X; `--all-load X` ⇔ load < X.
V2: `--load-window` default `1m`; selects 1|5|15 min average.
V3: `--run-queue X` ⇔ running tasks / N < X; `--all-run-queue X` ⇔ running < X. Linux `procs_running`; macOS closest native run-state counter.
V4: `--cpu-idle X` ⇔ idle % over last interval > X; first sample `Warming`.
V5: `--cpu-pressure X` ⇔ PSI `some avg10` < X. Linux only; macOS `Unsupported`. inside cgroup → cgroup `cpu.pressure` (`src/cgroup`).
V6: `--steal X` ⇔ steal % over last interval < X. Linux only.
V7: N ≥ 1 always; division checked.

## §T TASKS

id|status|task|cites
T1|.|N from `available_parallelism`, ≥ 1|V7
T2|.|load avg reader both OS + `--load` `--all-load`|V1
T3|.|`--load-window`|V2
T4|.|`/proc/stat` parser: `procs_running`, cpu times, steal|V3,V4,V6
T5|.|macOS run queue counter|V3
T6|.|`--cpu-idle` both OS (deltas)|V4
T7|.|`--cpu-pressure` via shared PSI parser|V5
T8|.|`--steal`|V6

## §B BUGS

id|date|cause|fix
