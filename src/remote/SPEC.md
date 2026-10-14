# SPEC

## §G GOAL

`src/remote` — evaluate gates on another host.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/remote|`--host` evaluation over ssh

## §C CONSTRAINTS

- deps: `src/gate`, `src/report` JSON schema. uses system `ssh` binary; ⊥ ssh library.

## §I INTERFACES

- switch: `--host H`.
- call: `ssh -o BatchMode=yes H cgmr check --json <gates>`.

## §V INVARIANTS

V1: ∀ gates evaluated on H; slots, reservations, cmd stay local.
V2: remote JSON `schema_version` ≠ supported → exit 69 w/ reason.
V3: ssh failure (unreachable, auth) → round fails (keep waiting), reason in verbose|heartbeat; ⊥ exit.
V4: remote `cgmr` missing (exit 127) → exit 69.
V5: gate args passed as separate argv words, quoted for remote shell; ⊥ injection.

## §T TASKS

id|status|task|cites
T1|.|build remote argv w/ safe quoting|V5
T2|.|run ssh, parse JSON, version check|V1,V2
T3|.|failure mapping|V3,V4

## §B BUGS

id|date|cause|fix
