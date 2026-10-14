# SPEC

## §G GOAL

`src/power` — thermal & power state gates.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/power|thermal throttling, temperature, AC, battery, low power

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, `src/sys`.

## §I INTERFACES

- switches: `--cool` · `--temp X` · `--on-ac` · `--battery X` · `--no-low-power`.
- sources: Linux `/sys/class/thermal`, `thermal_throttle` counters, `/sys/class/power_supply`, power-profiles-daemon state · macOS IOKit thermal state, IOKit power sources, `lowpowermode`.

## §V INVARIANTS

V1: `--cool` ⇔ no active throttling: Linux throttle counters unchanged over last interval; macOS thermal state = nominal.
V2: `--temp X` ⇔ hottest thermal zone < X. Linux only; macOS `Unsupported` (no public API).
V3: `--on-ac` ⇔ mains power present; desktop w/o battery & w/ mains → passes.
V4: `--battery X` ⇔ charge % > X | no battery.
V5: `--no-low-power` ⇔ Low Power Mode | power-saver profile off; Linux w/o power-profiles-daemon → `Unsupported`.

## §T TASKS

id|status|task|cites
T1|.|Linux throttle counters + thermal zones parser|V1,V2
T2|.|macOS thermal state|V1
T3|.|`--cool` both OS|V1
T4|.|`--temp`|V2
T5|.|power supply parser Linux + IOKit macOS|V3,V4
T6|.|`--on-ac` `--battery`|V3,V4
T7|.|`--no-low-power` both OS|V5

## §B BUGS

id|date|cause|fix
