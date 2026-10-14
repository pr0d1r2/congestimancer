#!/usr/bin/env bash
#
# Line coverage gate (nix:V5).
#
#   nix/guard/coverage.sh          check: floor, then the .coverage claim
#   nix/guard/coverage.sh --fix    measure and rewrite .coverage
#
# FLOOR: a hard minimum; below it the change lacks tests. Lowering it is a
# reviewed decision in its own commit.
# CLAIM: `.coverage` holds what the README badge states. A measurement below
# the claim fails (the badge overstates); one more than 0.5 above it fails
# too (the claim is stale): run with --fix and commit `.coverage`.
# A workspace with no instrumented lines yet measures as 100.0.
# main.rs files are excluded: src:V1 keeps them dispatch-only, so their
# behaviour is pinned by the CLI tests (tests:V1), not by line counts.
set -euo pipefail

floor=95

command -v cargo-llvm-cov >/dev/null 2>&1 || {
  echo "coverage: cargo-llvm-cov is not on PATH, so coverage was NOT measured -- re-enter the dev shell. This is not a pass." >&2
  exit 1
}

# llvm-profdata must read what rustc's LLVM writes; a mismatch reads as a
# broken test rather than a pin to update.
rustc_llvm=$(rustc -vV | awk '/^LLVM version:/ { split($3, a, "."); print a[1] }')
tool_llvm=$(llvm-profdata --version | awk '/LLVM version/ { split($NF, a, "."); print a[1] }')
if [ "$rustc_llvm" != "$tool_llvm" ]; then
  echo "coverage: rustc uses LLVM $rustc_llvm but llvm-profdata is LLVM $tool_llvm. Update the coverage llvm in flake.nix to match." >&2
  exit 1
fi

mkdir -p target
cargo llvm-cov --workspace --summary-only --ignore-filename-regex '(^|/)main\.rs$' >target/coverage.txt 2>target/coverage.err || {
  cat target/coverage.err target/coverage.txt >&2
  exit 1
}
measured=$(awk '/^TOTAL/ { if ($8 + 0 == 0) { print "100.0" } else { gsub("%", "", $10); printf "%.1f", int($10 * 10) / 10 } }' target/coverage.txt)
measured=${measured:-100.0}

if awk -v m="$measured" -v f="$floor" 'BEGIN { exit !(m + 0 < f + 0) }'; then
  cat target/coverage.txt >&2
  echo "coverage: $measured% is below the floor of $floor%. Cover the gap." >&2
  exit 1
fi

if [ "${1:-}" = --fix ]; then
  printf '# Line coverage this repository claims (nix:V5), rewritten by\n# nix/guard/coverage.sh --fix. One decimal, truncated.\nlines %s\n' "$measured" >.coverage
  echo "coverage: wrote lines $measured to .coverage"
  exit 0
fi

claim=$(awk '/^lines / { printf "%.1f", $2 }' .coverage)
if awk -v m="$measured" -v c="$claim" 'BEGIN { exit !(m + 0 < c + 0) }'; then
  echo "coverage: .coverage claims $claim% but this machine measures $measured%: the claim overstates. Add tests, or run nix/guard/coverage.sh --fix." >&2
  exit 1
fi
if awk -v m="$measured" -v c="$claim" 'BEGIN { exit !(m + 0 - c > 0.5) }'; then
  echo "coverage: .coverage claims $claim% but this machine measures $measured%: the claim is stale. Run nix/guard/coverage.sh --fix." >&2
  exit 1
fi
echo "coverage: $measured% (floor $floor%, claim $claim%)"
