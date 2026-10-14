# SPEC

## §G GOAL

`src/mem` — memory capacity & pressure signals & their gates.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/mem|available memory, memory pressure & level, swap rate, dirty pages

## §C CONSTRAINTS

- deps: `src/units`, `src/signal` (PSI parser), `src/sys`, `src/cgroup` (scope). reservation ledger ∈ `src/slot`.

## §I INTERFACES

- switches: `--mem X` · `--all-mem X` · `--needs-mem X` · `--mem-pressure X` · `--mem-pressure-full X` · `--mem-level normal|warning` · `--swap-rate X/s` · `--dirty X`.
- sources: Linux sysinfo (`MemAvailable`), `/proc/pressure/memory`, `/proc/vmstat` (`pswpin` `pswpout`), `/proc/meminfo` (`Dirty` `Writeback`) · macOS sysinfo, `kern.memorystatus_vm_pressure_level`, `kern.memorystatus_level`, `host_statistics64` swapins/swapouts.

## §V INVARIANTS

V1: capacity = AVAILABLE memory, ⊥ "free" ∵ page cache & compressor mislead.
V2: `--mem X` ⇔ available / N > X; `--all-mem X` ⇔ available > X.
V3: `--needs-mem X` ⇔ available − reserved by running cgmr jobs > X; reserved total from `src/slot`.
V4: `--mem-pressure X` ⇔ PSI `some avg10` < X; `--mem-pressure-full X` ⇔ `full avg10` < X. Linux only.
V5: `--mem-level normal` ⇔ level = 1; `--mem-level warning` ⇔ level ∈ {1,2}. macOS only; Linux `Unsupported`.
V6: `--swap-rate X/s` ⇔ (swap in + swap out) bytes/s < X; page counts × page size, checked.
V7: `--dirty X` ⇔ `Dirty` + `Writeback` < X. Linux only.
V8: `status` on macOS also shows `kern.memorystatus_level` %.

## §T TASKS

id|status|task|cites
T1|.|available memory both OS + `--mem` `--all-mem`|V1,V2
T2|.|`--needs-mem` using reservation total|V3
T3|.|`--mem-pressure` & `--mem-pressure-full`|V4
T4|.|macOS pressure level + `--mem-level`|V5
T5|.|`/proc/vmstat` parser + Linux swap rate|V6
T6|.|macOS swap rate from `host_statistics64`|V6
T7|.|`/proc/meminfo` parser + `--dirty`|V7
T8|.|`kern.memorystatus_level` reading for status|V8

## §B BUGS

id|date|cause|fix
