# SPEC

## §G GOAL

`src/disk` — free space & inodes on a path.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/disk|free space & inodes on a path

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, `src/sys` (`statvfs`).

## §I INTERFACES

- switches: `--disk X` · `--inodes X` · `--disk-path P`.

## §V INVARIANTS

V1: `--disk X` ⇔ `f_bavail × f_frsize` > X (space available to unprivileged user).
V2: `--inodes X` ⇔ `f_favail` > X.
V3: path default = cwd; `--disk-path P` overrides; P missing → usage error 64.
V4: both OS via `statvfs`; ⊥ mount-table matching.

## §T TASKS

id|status|task|cites
T1|.|`--disk` via statvfs|V1,V4
T2|.|`--inodes`|V2
T3|.|`--disk-path` default & validation|V3

## §B BUGS

id|date|cause|fix
