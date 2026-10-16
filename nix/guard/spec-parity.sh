#!/usr/bin/env bash
#
# spec-parity (.:V18): nothing deferred. Every switch docs/CONGESTION.md
# lists in its tables must be named, as `--switch`, by an invariant row
# (`V<n>:`) of some src/*/SPEC.md, and that node's task table must have a
# row citing that invariant. A method a human can read about but no node
# legislates and builds is a bug, not a backlog item.
#
#   nix/guard/spec-parity.sh [CATALOGUE] [SPEC...]
#
# Defaults: docs/CONGESTION.md and every tracked src/*/SPEC.md. Names are
# matched whole: `--io` never counts as `--io-busy`.
set -euo pipefail

catalogue=${1:-docs/CONGESTION.md}
if [ "$#" -ge 2 ]; then
  shift
  specs=("$@")
else
  mapfile -t specs < <(git ls-files 'src/*/SPEC.md')
fi
[ -f "$catalogue" ] && [ "${#specs[@]}" -gt 0 ] || {
  echo "spec-parity: no catalogue at $catalogue or no node specs -- nothing was checked. This is a failure, not a pass." >&2
  exit 1
}

awk '
  # The catalogue: every --switch in the first cell of a table row.
  FNR == 1 { file++ }
  file == 1 {
    if ($0 !~ /^\| `--/) next
    cell = $0
    sub(/^\| /, "", cell)
    sub(/ \|.*/, "", cell)
    while (match(cell, /--[a-z][a-z-]*/)) {
      wanted[substr(cell, RSTART + 2, RLENGTH - 2)] = 1
      cell = substr(cell, RSTART + RLENGTH)
    }
    next
  }
  # A node spec: remember each invariant line and each task row.
  match($0, /^V[0-9]+:/) {
    v = substr($0, 1, RLENGTH - 1)
    inv[FILENAME, v] = $0
    invs[FILENAME] = invs[FILENAME] " " v
    next
  }
  /^T[0-9]+\|/ {
    n = split($0, cells, "|")
    tasks[FILENAME] = tasks[FILENAME] "," cells[n] ","
  }
  END {
    bad = 0
    for (sw in wanted) {
      named = 0; built = 0
      for (key in inv) {
        split(key, k, SUBSEP)
        line = inv[key] " "
        if (index(line, "`--" sw) == 0) continue
        if (line !~ ("`--" sw "[^a-z-]")) continue
        named = 1
        if (tasks[k[1]] ~ ("[^A-Za-z0-9:]" k[2] "[^0-9]")) built = 1
      }
      if (!named) {
        printf "spec-parity: --%s is in the catalogue but no src node invariant names it (.:V18)\n", sw > "/dev/stderr"
        bad = 1
      } else if (!built) {
        printf "spec-parity: --%s is named by an invariant no task row cites (.:V18)\n", sw > "/dev/stderr"
        bad = 1
      }
    }
    exit bad
  }
' "$catalogue" "${specs[@]}"
