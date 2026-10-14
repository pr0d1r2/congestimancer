# SPEC

## §G GOAL

`src/limits` — open files & process count vs system maximum.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/limits|open files & processes vs system maximum

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, `src/sys`.

## §I INTERFACES

- switches: `--fds X` · `--pids X` (percent).
- sources: Linux `/proc/sys/fs/file-nr`, `/proc/sys/kernel/pid_max` + process count · macOS `kern.num_files`/`kern.maxfiles`, `kern.num_tasks`/`kern.maxproc`.

## §V INVARIANTS

V1: `--fds X` ⇔ open / max × 100 < X.
V2: `--pids X` ⇔ processes / max × 100 < X.
V3: max = 0 | unreadable → `Unsupported`, ⊥ divide.

## §T TASKS

id|status|task|cites
T1|.|`file-nr` parser + `--fds` Linux|V1,V3
T2|.|`--fds` macOS|V1,V3
T3|.|`--pids` both OS|V2,V3

## §B BUGS

id|date|cause|fix
