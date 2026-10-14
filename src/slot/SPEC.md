# SPEC

## §G GOAL

`src/slot` — coordinate waiters: concurrency slots, fair order, start rate, memory reservations.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/slot|per-user lock dir, slots, FIFO, rate, spacing, memory reservations, nested held marker

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, `src/sys` (`flock`). ⊥ daemon: state = files + flocks only.

## §I INTERFACES

- switches: `--slots N` · `--slot-key K` · `--fifo` · `--rate N/D` · `--spacing D` (+ `--needs-mem` ledger for `src/mem`).
- dir: `$XDG_RUNTIME_DIR/cgmr` else `$TMPDIR/cgmr` else `/tmp/cgmr-<uid>`; mode 0700; per user.
- env marker: `CGMR_HELD_SLOTS` (keys held by ancestor cgmr).

## §V INVARIANTS

V1: lock dir per user, 0700; existing dir w/ other owner | wider mode → refuse → exit 69 w/ reason ∵ symlink & hijack.
V2: `--slots N --slot-key K` ⇔ < N live holders of K; holder = flock on `K.<i>` file, fd ⊥ close-on-exec ∴ cmd holds slot until exit; crash → kernel releases.
V3: `--slots` w/o `--slot-key` → key = cmd basename.
V4: `--fifo`: waiters take ticket (monotonic seq file under flock); only lowest live ticket may claim a free slot; dead ticket holders skipped.
V5: `--rate N/D`: ≤ N starts per window D for key (token bucket in file under flock).
V6: `--spacing D`: ≥ D between consecutive starts for key.
V7: reservation: `--needs-mem X` writes reservation file held by flock until cmd exits; total = Σ live reservations.
V8: nested: key ∈ `CGMR_HELD_SLOTS` → slot treated as held, ⊥ deadlock.
V9: slot checks run AFTER gates pass & settle, BEFORE jitter.

## §T TASKS

id|status|task|cites
T1|.|lock dir resolve & safety checks|V1
T2|.|slots via flock files, inherited fd|V2,V3
T3|.|FIFO tickets|V4
T4|.|token bucket rate|V5
T5|.|spacing|V6
T6|.|memory reservation ledger|V7
T7|.|nested held marker|V8
T8|.|ordering w/ gates, settle, jitter|V9

## §B BUGS

id|date|cause|fix
