# SPEC

## §G GOAL

`src/net` — network throughput & latency gates.

## §N NAV

rel|path|lens
up|src|the tool: units, signal model, OS facade, every congestion domain, gates, waiting, slots, launch, remote, report, CLI
self|src/net|interface rates, RTT probe

## §C CONSTRAINTS

- deps: `src/units`, `src/signal`, `src/sys`. ⊥ raw sockets, ⊥ root.

## §I INTERFACES

- switches: `--net-rx X/s` · `--net-tx X/s` · `--net X/s` · `--net-iface I` · `--rtt D` · `--rtt-host H[:PORT]`.
- sources: Linux `/proc/net/dev` · macOS `getifaddrs` + `if_data`.

## §V INVARIANTS

V1: default ifaces = ∀ up, non-loopback; `--net-iface I` selects one; unknown I → usage 64.
V2: `--net-rx X` ⇔ rx B/s < X; `--net-tx X` ⇔ tx B/s < X; `--net X` ⇔ sum < X. first sample `Warming`.
V3: `--rtt D` ⇔ TCP connect time to `--rtt-host` < D; port default 443; connect timeout = D (fail = gate fails, ⊥ error exit).
V4: `--rtt` w/o `--rtt-host` → usage 64.
V5: counter wrap (32-bit `if_data`) handled → `Warming`, ⊥ huge rate.

## §T TASKS

id|status|task|cites
T1|.|`/proc/net/dev` parser|V2
T2|.|macOS interface counters|V2,V5
T3|.|iface selection|V1
T4|.|`--net-rx` `--net-tx` `--net`|V2
T5|.|`--rtt` + `--rtt-host`|V3,V4

## §B BUGS

id|date|cause|fix
