# SPEC

## §G GOAL

`src/units` — parse & render every value a switch takes.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/units|parse & render sizes, rates, percents, durations, temperatures, clock windows

## §C CONSTRAINTS

- zero deps. pure fns. ⊥ I/O, ⊥ locale.

## §I INTERFACES

- types: `Bytes(u64)`, `Rate` (bytes/s), `Percent` (0..=100), `Duration` (std), `Celsius`, `ClockWindow`, `Count(u64)`.
- fns: `parse_<type>(&str) -> Result<T, UnitError>` & `Display` ∀ type.

## §V INVARIANTS

V1: `B` = 1. `KB` `MB` `GB` `TB` = 1000^n. `KiB` `MiB` `GiB` `TiB` = 1024^n. suffix case-insensitive; `KB` ≠ `KiB`. ⊥ bits.
V2: size number ? fractional (`1.5GB`); result rounds down to whole bytes.
V3: rate = size + `/s`; ⊥ other time base.
V4: percent = number + optional `%`; ∉ 0..=100 → error.
V5: duration = 1+ `<n><unit>` pairs, unit ∈ `ms s m h d`, e.g. `1h30m`; bare number → error. ⊥ `humantime`.
V6: temperature = number + `C`.
V7: clock window `HH:MM-HH:MM`, local time; start > end ∴ wraps midnight; start = end → error.
V8: decimal separator `.` only, locale ignored.
V9: overflow → error, ⊥ wrap, ⊥ saturate.
V10: ∀ parser never panics on any input (proptest).
V11: `parse(render(x)) == x` ∀ type (proptest).
V12: error names input & expected form → cli maps to exit 64.

## §T TASKS

id|status|task|cites
T1|.|`Bytes` parse decimal & binary suffixes|V1,V2,V9
T2|.|`Rate` parse|V3
T3|.|`Percent` parse|V4
T4|.|`Duration` parse, own grammar|V5,V9
T5|.|`Celsius` parse|V6
T6|.|`ClockWindow` parse & contains(time), midnight wrap|V7
T7|.|`Count` parse|V9
T8|.|locale-independent number lexer|V8
T9|.|render ∀ type + round-trip proptest|V11
T10|.|no-panic proptest ∀ parser|V10
T11|.|`UnitError` w/ input & expected form|V12

## §B BUGS

id|date|cause|fix
