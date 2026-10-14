# SPEC

## §G GOAL

`src/gate` — gate model: threshold vs reading, strict compare, AND over ∀ gates, custom gates.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/gate|gate model, strict compare, AND evaluation, unsupported verdict, custom gates

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

## §T TASKS

id|status|task|cites
T1|.|`Gate`, `Verdict`, strict compare|V1
T2|.|AND evaluation w/ warming & unsupported|V2,V3,V6
T3|.|`--when`|V4
T4|.|`--when-file`|V5
T5|.|one read per signal per round|V7

## §B BUGS

id|date|cause|fix
