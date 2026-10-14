# SPEC

## §G GOAL

`src/launch` — start the cmd once released: exec, govern, background priority.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/launch|exec, no-cmd exit, govern, background priority, env cleanup

## §C CONSTRAINTS

- deps: `src/signal`, `src/sys`, `src/gate`, `src/wait`, `src/slot`. ⊥ `set_var`|`remove_var` (unsafe in 2024).

## §I INTERFACES

- switches: `--govern` · `--background` · `--or-background`.

## §V INVARIANTS

V1: default: `exec` cmd (`CommandExt::exec`); exit code & signals pass through; exec failure → 127 (not found) | 126 (not executable).
V2: no cmd → exit 0 once released.
V3: env cleanup: `Command::env_remove` ∀ `CGMR_*` except `CGMR_HELD_SLOTS` (updated w/ keys now held).
V4: `--govern`: spawn cmd in own process group; keep evaluating gates; gates fail → SIGSTOP group; pass again → SIGCONT; forward SIGINT SIGTERM SIGHUP; exit w/ cmd's code (signal death → 128+n).
V5: `--background`: lower CPU priority (`setpriority` nice 19) & IO priority (Linux `ioprio_set` idle class; macOS `setiopolicy_np` throttle) before exec.
V6: `--or-background`: `--timeout` reached → start anyway as `--background` instead of exit 124.
V7: stdout|stdin|stderr untouched; cmd owns stdout (`.` §I).

## §T TASKS

id|status|task|cites
T1|.|exec + exit codes 126/127|V1
T2|.|no-cmd exit 0|V2
T3|.|env cleanup & held marker|V3
T4|.|`--govern` spawn, stop/cont, forward signals|V4
T5|.|`--background` both OS|V5
T6|.|`--or-background`|V6

## §B BUGS

id|date|cause|fix
